use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

type Index = u32;

type Callback = dyn FnMut() + 'static;

pub struct EventLoop {
    index: Rc<Cell<Index>>,
    timers_callbacks: HashMap<Index, Rc<RefCell<Callback>>>,
    timers_queue: BTreeMap<Instant, Index>,
    events_queue: mpsc::Receiver<Event>,
    events_queue_empty: Rc<Cell<bool>>,
    events_notifier: Rc<mpsc::Sender<Event>>,
}

impl EventLoop {
    pub fn new() -> Self {
        let (events_notifier, events_queue) = mpsc::channel();

        EventLoop {
            index: Rc::new(Cell::new(1)),
            timers_callbacks: HashMap::new(),
            timers_queue: BTreeMap::new(),
            events_queue,
            events_queue_empty: Rc::new(Cell::new(true)),
            events_notifier: Rc::new(events_notifier),
        }
    }

    pub fn handle(&self) -> LoopHandle {
        LoopHandle {
            index: self.index.clone(),
            notifier: self.events_notifier.clone(),
            events_queue_empty: self.events_queue_empty.clone(),
        }
    }

    pub fn has_pending_events(&self) -> bool {
        !(self.timers_queue.is_empty() && self.events_queue_empty.get())
    }

    pub fn poll(&mut self) {
        self.prepare();
        self.run_timers();
    }

    fn prepare(&mut self) {
        while let Ok(event) = self.events_queue.try_recv() {
            match event {
                Event::NewTimer(index, delay, cb) => self.ev_new_timer(index, delay, cb),
                Event::RemoveTimer(index) => self.ev_remove_timer(&index),
            };
        }
        self.events_queue_empty.set(true);
    }

    fn run_timers(&mut self) {
        let timers_to_remove: Vec<Instant> = self
            .timers_queue
            .range(..Instant::now())
            .map(|(k, _)| *k)
            .collect();

        timers_to_remove.iter().for_each(|key| {
            let index = match self.timers_queue.remove(key) {
                Some(index) => index,
                None => return,
            };
            if let Some(callback) = self.timers_callbacks.remove(&index) {
                (callback.borrow_mut())();
            }
        });

        self.prepare();
    }

    fn ev_new_timer(&mut self, index: Index, delay: Duration, cb: Rc<RefCell<Callback>>) {
        let time_key = Instant::now() + delay;
        self.timers_callbacks.insert(index, cb);
        self.timers_queue.insert(time_key, index);
    }

    fn ev_remove_timer(&mut self, index: &Index) {
        self.timers_callbacks.remove(index);
        self.timers_queue.retain(|_, v| *v != *index);
    }
}

pub struct LoopHandle {
    index: Rc<Cell<Index>>,
    notifier: Rc<mpsc::Sender<Event>>,
    events_queue_empty: Rc<Cell<bool>>,
}

#[allow(dead_code)]
impl LoopHandle {
    pub fn index(&self) -> Index {
        let index = self.index.get();
        self.index.set(index + 1);
        index
    }

    pub fn timer<F>(&self, delay: u64, cb: F) -> Index
    where
        F: FnMut() + 'static,
    {
        let index = self.index();
        let expires_at = Duration::from_millis(delay);
        let cb = Rc::new(RefCell::new(cb));

        self.notifier
            .send(Event::NewTimer(index, expires_at, cb))
            .unwrap();

        self.events_queue_empty.set(false);

        index
    }

    pub fn remove_timer(&self, index: &Index) {
        self.notifier.send(Event::RemoveTimer(*index)).unwrap();
        self.events_queue_empty.set(false);
    }
}

#[allow(dead_code)]
enum Event {
    NewTimer(Index, Duration, Rc<RefCell<Callback>>),
    RemoveTimer(Index),
}
