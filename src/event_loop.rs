use anyhow::Result;
use downcast_rs::impl_downcast;
use downcast_rs::Downcast;
use std::any::type_name;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use threadpool::ThreadPool;

type Index = u32;

pub type TaskResult = Option<Result<Vec<u8>>>;

/// All objects that are tracked by the event-loop should implement the `Resource` trait.
pub trait Resource: Downcast + 'static {
    /// Returns a string representation of the resource.
    fn name(&self) -> Cow<str> {
        type_name::<Self>().into()
    }
}

impl_downcast!(Resource);

struct TimerWrap {
    cb: Box<dyn FnMut() + 'static>,
    expires_at: Duration,
    repeat: bool,
}

impl Resource for TimerWrap {}

struct TaskWrap {
    inner: Option<Box<dyn FnOnce(TaskResult) + 'static>>,
}

impl Resource for TaskWrap {}

enum Action {
    NewTimer(Index, TimerWrap),
    RemoveTimer(Index),
    SpawnTask(Index, Box<dyn FnOnce() -> TaskResult + Send>, TaskWrap),
}

enum Event {
    Interrupt,
    ThreadPool(Index, TaskResult),
}

pub struct EventLoop {
    index: Rc<Cell<Index>>,
    resources: HashMap<Index, Box<dyn Resource>>,
    timer_queue: BTreeMap<Instant, Index>,
    action_queue: mpsc::Receiver<Action>,
    action_queue_empty: Rc<Cell<bool>>,
    action_dispatcher: Rc<mpsc::Sender<Action>>,
    thread_pool: ThreadPool,
    event_dispatcher: Arc<Mutex<mpsc::Sender<Event>>>,
    event_queue: mpsc::Receiver<Event>,
    pending_tasks: u32,
}

impl EventLoop {
    pub fn new() -> Self {
        let (action_dispatcher, action_queue) = mpsc::channel();
        let (event_dispatcher, event_queue) = mpsc::channel();

        EventLoop {
            index: Rc::new(Cell::new(1)),
            resources: HashMap::new(),
            timer_queue: BTreeMap::new(),
            action_queue,
            action_queue_empty: Rc::new(Cell::new(true)),
            action_dispatcher: Rc::new(action_dispatcher),
            thread_pool: ThreadPool::new(4),
            event_dispatcher: Arc::new(Mutex::new(event_dispatcher)),
            event_queue,
            pending_tasks: 0,
        }
    }

    pub fn handle(&self) -> LoopHandle {
        LoopHandle {
            index: self.index.clone(),
            actions: self.action_dispatcher.clone(),
            actions_queue_empty: self.action_queue_empty.clone(),
        }
    }

    pub fn interrupt_handle(&self) -> LoopInterruptHandle {
        LoopInterruptHandle {
            events: self.event_dispatcher.clone(),
        }
    }

    pub fn has_pending_events(&self) -> bool {
        !(self.resources.is_empty() && self.action_queue_empty.get())
    }

    pub fn tick(&mut self) {
        self.prepare();
        self.run_timers();
        self.run_poll();
    }

    fn prepare(&mut self) {
        while let Ok(action) = self.action_queue.try_recv() {
            match action {
                Action::NewTimer(index, timer) => self.ev_new_timer(index, timer),
                Action::RemoveTimer(index) => self.ev_remove_timer(&index),
                Action::SpawnTask(index, task, t_wrap) => self.ev_spawn_task(index, task, t_wrap),
            };
        }
        self.action_queue_empty.set(true);
    }

    fn run_timers(&mut self) {
        // Note: We use this intermediate vector so we don't have Rust complaining
        // about holding multiple references.
        let timers_to_remove: Vec<Instant> = self
            .timer_queue
            .range(..Instant::now())
            .map(|(k, _)| *k)
            .collect();

        let indexes: Vec<Index> = timers_to_remove
            .iter()
            .filter_map(|instant| self.timer_queue.remove(instant))
            .collect();

        indexes.iter().for_each(|index| {
            if let Some(timer) = self
                .resources
                .get_mut(index)
                .map(|resource| resource.downcast_mut::<TimerWrap>().unwrap())
            {
                // Run timer's callback.
                (timer.cb)();

                // If the timer is repeatable reschedule him, otherwise drop him.
                if timer.repeat {
                    let time_key = Instant::now() + timer.expires_at;
                    self.timer_queue.insert(time_key, *index);
                } else {
                    self.resources.remove(index);
                }
            }
        });

        self.prepare();
    }

    fn run_poll(&mut self) {
        // Based on what resources the event-loop is currently running will decide
        // how long we should wait on the this phase.
        let timeout = match self.timer_queue.iter().next() {
            Some((t, _)) => *t - Instant::now(),
            None if self.pending_tasks > 0 => Duration::MAX,
            None => Duration::ZERO,
        };

        if let Ok(event) = self.event_queue.recv_timeout(timeout) {
            match event {
                Event::Interrupt => return,
                Event::ThreadPool(index, result) => self.run_task_callback(index, result),
            }
            self.pending_tasks -= 1;
            self.prepare();
        }
    }

    fn run_task_callback(&mut self, index: Index, result: TaskResult) {
        if let Some(mut resource) = self.resources.remove(&index) {
            let task_wrap = resource.downcast_mut::<TaskWrap>().unwrap();
            let callback = task_wrap.inner.take().unwrap();
            (callback)(result);
        }
    }

    fn ev_new_timer(&mut self, index: Index, timer: TimerWrap) {
        let time_key = Instant::now() + timer.expires_at;
        self.resources.insert(index, Box::new(timer));
        self.timer_queue.insert(time_key, index);
    }

    fn ev_remove_timer(&mut self, index: &Index) {
        self.resources.remove(index);
        self.timer_queue.retain(|_, v| *v != *index);
    }

    fn ev_spawn_task(
        &mut self,
        index: Index,
        task: Box<dyn FnOnce() -> TaskResult + Send>,
        task_wrap: TaskWrap,
    ) {
        let notifier = self.event_dispatcher.clone();

        if task_wrap.inner.is_some() {
            self.resources.insert(index, Box::new(task_wrap));
        }

        self.thread_pool.execute(move || {
            let result = (task)();
            let notifier = notifier.lock().unwrap();

            notifier.send(Event::ThreadPool(index, result)).unwrap();
        });

        self.pending_tasks += 1;
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LoopHandle {
    index: Rc<Cell<Index>>,
    actions: Rc<mpsc::Sender<Action>>,
    actions_queue_empty: Rc<Cell<bool>>,
}

#[allow(dead_code)]
impl LoopHandle {
    /// Returns the next available resource index.
    pub fn index(&self) -> Index {
        let index = self.index.get();
        self.index.set(index + 1);
        index
    }

    /// Schedules a new timer to the event-loop.
    pub fn timer<F>(&self, delay: u64, repeat: bool, cb: F) -> Index
    where
        F: FnMut() + 'static,
    {
        let index = self.index();
        let expires_at = Duration::from_millis(delay);

        let timer = TimerWrap {
            cb: Box::new(cb),
            expires_at,
            repeat,
        };

        self.actions.send(Action::NewTimer(index, timer)).unwrap();
        self.actions_queue_empty.set(false);

        index
    }

    /// Removes a scheduled timer from the event-loop.
    pub fn remove_timer(&self, index: &Index) {
        self.actions.send(Action::RemoveTimer(*index)).unwrap();
        self.actions_queue_empty.set(false);
    }

    /// Spawns a new task without blocking the main thread.
    pub fn spawn<F, U>(&self, task: F, task_cb: Option<U>) -> Index
    where
        F: FnOnce() -> TaskResult + Send + 'static,
        U: FnOnce(TaskResult) + 'static,
    {
        let index = self.index();

        // Note: I tried to use `.and_then` instead of this ugly match statement but Rust complains
        // about mismatch types having no idea why.
        let task_cb: Option<Box<dyn FnOnce(TaskResult)>> = match task_cb {
            Some(cb) => Some(Box::new(cb)),
            None => None,
        };

        let task_wrap = TaskWrap { inner: task_cb };

        self.actions
            .send(Action::SpawnTask(index, Box::new(task), task_wrap))
            .unwrap();

        self.actions_queue_empty.set(false);

        index
    }
}

pub struct LoopInterruptHandle {
    events: Arc<Mutex<mpsc::Sender<Event>>>,
}

impl LoopInterruptHandle {
    // Interrupts the poll phase of the event-loop.
    pub fn interrupt(&self) {
        self.events.lock().unwrap().send(Event::Interrupt).unwrap();
    }
}
