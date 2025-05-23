use crate::bindings;
use crate::errors::report_and_exit;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::exceptions::ExceptionState;
use crate::exceptions::PromiseRejectionEntry;
use crate::hooks::host_import_module_dynamically_cb;
use crate::hooks::host_initialize_import_meta_object_cb;
use crate::hooks::module_resolve_cb;
use crate::hooks::promise_reject_cb;
use crate::inspector::JsRuntimeInspector;
use crate::modules::create_origin;
use crate::modules::fetch_module_tree;
use crate::modules::load_import;
use crate::modules::resolve_import;
use crate::modules::EsModuleFuture;
use crate::modules::ImportKind;
use crate::modules::ImportMap;
use crate::modules::ModuleGraph;
use crate::modules::ModuleMap;
use crate::modules::ModuleStatus;
use crate::process;
use anyhow::bail;
use anyhow::Error;
use anyhow::Ok;
use dune_event_loop::EventLoop;
use dune_event_loop::LoopHandle;
use dune_event_loop::LoopInterruptHandle;
use dune_event_loop::TaskResult;
use std::cell::RefCell;
use std::cmp;
use std::net::SocketAddrV4;
use std::rc::Rc;
use std::sync::Once;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// A vector with JS callbacks and parameters.
type NextTickQueue = Vec<(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>)>;

/// An abstract interface for something that should run in respond to an
/// async task, scheduled previously and is now completed.
pub trait JsFuture {
    fn run(&mut self, scope: &mut v8::HandleScope);
}

/// The state to be stored per v8 isolate.
pub struct JsRuntimeState {
    /// A sand-boxed execution context with its own set of built-in objects and functions.
    pub context: v8::Global<v8::Context>,
    /// Holds information about resolved ES modules.
    pub module_map: ModuleMap,
    /// A handle to the runtime's event-loop.
    pub handle: LoopHandle,
    /// A handle to the event-loop that can interrupt the poll-phase.
    pub interrupt_handle: LoopInterruptHandle,
    /// Holds JS pending futures scheduled by the event-loop.
    pub pending_futures: Vec<Box<dyn JsFuture>>,
    /// Indicates the start time of the process.
    pub startup_moment: Instant,
    /// Specifies the timestamp which the current process began in Unix time.
    pub time_origin: u128,
    /// Holds callbacks scheduled by nextTick.
    pub next_tick_queue: NextTickQueue,
    /// Stores and manages uncaught exceptions.
    pub exceptions: ExceptionState,
    /// Runtime options.
    pub options: JsRuntimeOptions,
    /// Tracks wake event for current loop iteration.
    pub wake_event_queued: bool,
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)]
pub struct JsRuntimeOptions {
    // The seed used in Math.random() method.
    pub seed: Option<i64>,
    // Reloads every URL import.
    pub reload: bool,
    // The main entry point for the program.
    pub root: Option<String>,
    // Holds user defined import maps for module loading.
    pub import_map: Option<ImportMap>,
    // The numbers of threads used by the thread-pool.
    pub num_threads: Option<usize>,
    // Indicates if we're running JavaScript tests.
    pub test_mode: bool,
    // Defines the inspector listening options.
    pub inspect: Option<(SocketAddrV4, bool)>,
    // Exposes v8's garbage collector.
    pub expose_gc: bool,
}

pub struct JsRuntime {
    /// A VM instance with its own heap.
    /// https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
    /// A structure responsible for providing inspector interface to the runtime.
    pub inspector: Option<Rc<RefCell<JsRuntimeInspector>>>,
    /// The event-loop instance that takes care of polling for I/O.
    pub event_loop: EventLoop,
    /// The state of the runtime.
    #[allow(unused)]
    pub state: Rc<RefCell<JsRuntimeState>>,
}

impl JsRuntime {
    /// Creates a new JsRuntime.
    pub fn new() -> JsRuntime {
        Self::with_options(JsRuntimeOptions::default())
    }

    /// Creates a new JsRuntime based on provided options.
    pub fn with_options(options: JsRuntimeOptions) -> JsRuntime {
        // Configuration flags for V8.
        let mut flags = String::from(concat!(
            " --no-validate-asm",
            " --turbo_fast_api_calls",
            " --harmony-temporal",
            " --js-float16array",
        ));

        if let Some(seed) = options.seed {
            flags.push_str(&format!(" --predictable --random-seed={seed}"))
        }

        if options.expose_gc {
            flags.push_str(" --expose-gc")
        }

        v8::V8::set_flags_from_string(&flags);

        // Fire up the v8 engine.
        static V8_INIT: Once = Once::new();
        V8_INIT.call_once(move || {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        isolate.set_microtasks_policy(v8::MicrotasksPolicy::Explicit);
        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
        isolate.set_promise_reject_callback(promise_reject_cb);
        isolate.set_host_import_module_dynamically_callback(host_import_module_dynamically_cb);

        isolate
            .set_host_initialize_import_meta_object_callback(host_initialize_import_meta_object_cb);

        let context = {
            let scope = &mut v8::HandleScope::new(&mut *isolate);
            let context = bindings::create_new_context(scope);
            v8::Global::new(scope, context)
        };

        const MIN_POOL_SIZE: usize = 1;

        let event_loop = match options.num_threads {
            Some(n) => EventLoop::new(cmp::max(n, MIN_POOL_SIZE)),
            None => EventLoop::default(),
        };

        let time_origin = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Initialize the v8 inspector.
        let address = options.inspect.map(|(address, _)| (address));
        let inspector = options.inspect.map(|(_, waiting_for_session)| {
            JsRuntimeInspector::new(
                &mut isolate,
                context.clone(),
                event_loop.interrupt_handle(),
                waiting_for_session,
                options.root.clone(),
            )
        });

        // Store state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        let state = Rc::new(RefCell::new(JsRuntimeState {
            context,
            module_map: ModuleMap::new(),
            handle: event_loop.handle(),
            interrupt_handle: event_loop.interrupt_handle(),
            pending_futures: Vec::new(),
            startup_moment: Instant::now(),
            time_origin,
            next_tick_queue: Vec::new(),
            exceptions: ExceptionState::new(),
            options,
            wake_event_queued: false,
        }));

        isolate.set_slot(state.clone());

        let mut runtime = JsRuntime {
            isolate,
            event_loop,
            state,
            inspector,
        };

        runtime.load_main_environment();

        // Start inspector agent is requested.
        if let Some(inspector) = runtime.inspector().as_mut() {
            let address = address.unwrap();
            inspector.borrow_mut().start_agent(address);
        }

        runtime
    }

    /// Initializes synchronously the core environment (see lib/main.js).
    fn load_main_environment(&mut self) {
        let name = "dune:environment/main";
        let source = include_str!("./js/main.js");

        let scope = &mut self.handle_scope();
        let tc_scope = &mut v8::TryCatch::new(scope);

        let module = match fetch_module_tree(tc_scope, name, Some(source)) {
            Some(module) => module,
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                let exception = JsError::from_v8_exception(tc_scope, exception, None);
                eprintln!("{exception:?}");
                std::process::exit(1);
            }
        };

        if module
            .instantiate_module(tc_scope, module_resolve_cb)
            .is_none()
        {
            assert!(tc_scope.has_caught());
            let exception = tc_scope.exception().unwrap();
            let exception = JsError::from_v8_exception(tc_scope, exception, None);
            report_and_exit(exception);
        }

        let _ = module.evaluate(tc_scope);

        if module.get_status() == v8::ModuleStatus::Errored {
            let exception = module.get_exception();
            let exception = JsError::from_v8_exception(tc_scope, exception, None);
            report_and_exit(exception);
        }

        // Initialize process static values.
        process::refresh(tc_scope);
    }

    /// Executes traditional JavaScript code (traditional = not ES modules).
    pub fn execute_script(
        &mut self,
        filename: &str,
        source: &str,
    ) -> Result<Option<v8::Global<v8::Value>>, Error> {
        // Get the handle-scope.
        let scope = &mut self.handle_scope();
        let state_rc = JsRuntime::state(scope);

        let origin = create_origin(scope, filename, false);
        let source = v8::String::new(scope, source).unwrap();

        // The `TryCatch` scope allows us to catch runtime errors rather than panicking.
        let tc_scope = &mut v8::TryCatch::new(scope);
        type ExecuteScriptResult = Result<Option<v8::Global<v8::Value>>, Error>;

        let handle_exception =
            |scope: &mut v8::TryCatch<'_, v8::HandleScope<'_>>| -> ExecuteScriptResult {
                // Extract the exception during compilation.
                assert!(scope.has_caught());
                let exception = scope.exception().unwrap();
                let exception = v8::Global::new(scope, exception);
                let mut state = state_rc.borrow_mut();
                // Capture the exception internally.
                state.exceptions.capture_exception(exception);
                drop(state);
                // Force an exception check.
                if let Some(error) = check_exceptions(scope) {
                    bail!(error)
                }
                Ok(None)
            };

        let script = match v8::Script::compile(tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => return handle_exception(tc_scope),
        };

        match script.run(tc_scope) {
            Some(value) => Ok(Some(v8::Global::new(tc_scope, value))),
            None => handle_exception(tc_scope),
        }
    }

    /// Executes JavaScript code as ES module.
    pub fn execute_module(&mut self, filename: &str, source: Option<&str>) -> Result<(), Error> {
        // Get a reference to v8's scope.
        let scope = &mut self.handle_scope();
        let state_rc = JsRuntime::state(scope);
        let mut state = state_rc.borrow_mut();

        // The following code allows the runtime to execute code with no valid
        // location passed as parameter as an ES module.
        let path = match source.is_some() {
            true => filename.to_string(),
            false => unwrap_or_exit(resolve_import(None, filename, false, None)),
        };

        // Create static import module graph.
        let graph = ModuleGraph::static_import(&path);
        let graph_rc = Rc::new(RefCell::new(graph));
        let status = ModuleStatus::Fetching;

        state.module_map.pending.push(Rc::clone(&graph_rc));
        state.module_map.seen.insert(path.clone(), status);

        // If we have a source, create the es-module future.
        if let Some(source) = source {
            state.pending_futures.push(Box::new(EsModuleFuture {
                path,
                module: Rc::clone(&graph_rc.borrow().root_rc),
                maybe_result: Some(Ok(bincode::serialize(&source).unwrap())),
            }));
            return Ok(());
        }

        /*  Use the event-loop to asynchronously load the requested module. */

        let task = {
            let specifier = path.clone();
            move || match load_import(&specifier, true) {
                anyhow::Result::Ok(source) => Some(Ok(bincode::serialize(&source).unwrap())),
                Err(e) => Some(Result::Err(e)),
            }
        };

        let task_cb = {
            let state_rc = state_rc.clone();
            move |_: LoopHandle, maybe_result: TaskResult| {
                let mut state = state_rc.borrow_mut();
                let future = EsModuleFuture {
                    path,
                    module: Rc::clone(&graph_rc.borrow().root_rc),
                    maybe_result,
                };
                state.pending_futures.push(Box::new(future));
            }
        };

        state.handle.spawn(task, Some(task_cb));

        Ok(())
    }

    /// Runs a single tick of the event-loop.
    pub fn tick_event_loop(&mut self) {
        run_next_tick_callbacks(&mut self.handle_scope());
        self.fast_forward_imports();
        self.event_loop.tick();
        self.run_pending_futures();
    }

    /// Polls the inspector for new devtools messages.
    pub fn poll_inspect_session(&mut self) {
        if let Some(inspector) = self.inspector.as_mut() {
            inspector.borrow_mut().poll_session();
        }
    }

    /// Runs the event-loop until no more pending events exists.
    pub fn run_event_loop(&mut self) {
        // Check for pending devtools messages.
        self.poll_inspect_session();
        // Run callbacks/promises from next-tick and micro-task queues.
        run_next_tick_callbacks(&mut self.handle_scope());

        while self.event_loop.has_pending_events()
            || self.has_promise_rejections()
            || self.isolate.has_pending_background_tasks()
            || self.has_pending_imports()
            || self.has_next_tick_callbacks()
        {
            // Check for pending devtools messages.
            self.poll_inspect_session();
            // Tick the event-loop one cycle.
            self.tick_event_loop();

            // Report any unhandled promise rejections.
            if let Some(error) = check_exceptions(&mut self.handle_scope()) {
                report_and_exit(error);
            }
        }

        // We can now notify debugger that the program has finished running
        // and we're ready to exit the process.
        if let Some(inspector) = self.inspector() {
            let context = self.context();
            let scope = &mut self.handle_scope();
            inspector.borrow_mut().context_destroyed(scope, context);
        }
    }

    /// Runs all the pending javascript tasks.
    fn run_pending_futures(&mut self) {
        // Get a handle-scope and a reference to the runtime's state.
        let scope = &mut self.handle_scope();
        let state_rc = Self::state(scope);

        // NOTE: The reason we move all the js futures to a separate vec is because
        // we need to drop the `state` borrow before we start iterating through all
        // of them to avoid borrowing panics at runtime.

        let futures: Vec<Box<dyn JsFuture>> =
            state_rc.borrow_mut().pending_futures.drain(..).collect();

        // NOTE: After every future executes (aka v8's call stack gets empty) we will drain
        // the MicroTask and NextTick Queue.

        for mut fut in futures {
            fut.run(scope);
            if let Some(error) = check_exceptions(scope) {
                report_and_exit(error);
            }
            run_next_tick_callbacks(scope);
        }

        state_rc.borrow_mut().wake_event_queued = false;
    }

    /// Checks for imports (static/dynamic) ready for execution.
    fn fast_forward_imports(&mut self) {
        // Get a v8 handle-scope.
        let scope = &mut self.handle_scope();
        let state_rc = JsRuntime::state(scope);
        let mut state = state_rc.borrow_mut();

        let mut ready_imports = vec![];

        // Note: The following is a trick to get multiple `mut` references in the same
        // struct called splitting borrows (https://doc.rust-lang.org/nomicon/borrow-splitting.html).
        let state_ref = &mut *state;
        let pending_graphs = &mut state_ref.module_map.pending;
        let seen_modules = &mut state_ref.module_map.seen;

        pending_graphs.retain(|graph_rc| {
            // Get a usable ref to graph's root module.
            let graph = graph_rc.borrow();
            let mut graph_root = graph.root_rc.borrow_mut();

            // Check for exceptions in the graph (dynamic imports).
            if let Some(message) = graph_root.exception.borrow_mut().take() {
                // Create a v8 exception.
                let exception = v8::String::new(scope, &message).unwrap();
                let exception = v8::Exception::error(scope, exception);

                // We need to resolve all identical dynamic imports.
                match graph.kind.clone() {
                    ImportKind::Static => unreachable!(),
                    ImportKind::Dynamic(main_promise) => {
                        for promise in [main_promise].iter().chain(graph.same_origin.iter()) {
                            promise.open(scope).reject(scope, exception);
                        }
                    }
                }

                return false;
            }

            // If the graph is still loading, fast-forward the dependencies.
            if graph_root.status != ModuleStatus::Ready {
                graph_root.fast_forward(seen_modules);
                return true;
            }

            ready_imports.push(Rc::clone(graph_rc));
            false
        });

        // Note: We have to drop the sate ref here to avoid borrow panics
        // during the module instantiation/evaluation process.
        drop(state);

        // Execute the root module from the graph.
        for graph_rc in ready_imports {
            // Create a tc scope.
            let tc_scope = &mut v8::TryCatch::new(scope);

            let graph = graph_rc.borrow();
            let path = graph.root_rc.borrow().path.clone();

            let module = state_rc.borrow().module_map.get(&path).unwrap();
            let module = v8::Local::new(tc_scope, module);

            if module
                .instantiate_module(tc_scope, module_resolve_cb)
                .is_none()
            {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                let exception = JsError::from_v8_exception(tc_scope, exception, None);
                report_and_exit(exception);
            }

            let _ = module.evaluate(tc_scope);
            let is_root_module = !graph.root_rc.borrow().is_dynamic_import;

            // Note: Due to the architecture, when a module errors, the `promise_reject_cb`
            // v8 hook will also trigger, resulting in the same exception being registered
            // as an unhandled promise rejection. Therefore, we need to manually remove it.
            if module.get_status() == v8::ModuleStatus::Errored && is_root_module {
                let mut state = state_rc.borrow_mut();
                let exception = module.get_exception();
                let exception = v8::Global::new(tc_scope, exception);

                state.exceptions.capture_exception(exception.clone());
                state.exceptions.remove_promise_rejection_entry(&exception);

                drop(state);

                if let Some(error) = check_exceptions(tc_scope) {
                    report_and_exit(error);
                }
            }

            if let ImportKind::Dynamic(main_promise) = graph.kind.clone() {
                // Note: Since this is a dynamic import will resolve the promise
                // with the module's namespace object instead of it's evaluation result.
                let namespace = module.get_module_namespace();

                // We need to resolve all identical dynamic imports.
                for promise in [main_promise].iter().chain(graph.same_origin.iter()) {
                    promise.open(tc_scope).resolve(tc_scope, namespace);
                }
            }
        }

        // Note: It's important to perform a nextTick checkpoint at this
        // point to allow resources behind a promise to be scheduled correctly
        // to the event-loop.
        run_next_tick_callbacks(scope);
    }

    /// Returns if unhandled promise rejections where caught.
    pub fn has_promise_rejections(&mut self) -> bool {
        self.get_state().borrow().exceptions.has_promise_rejection()
    }

    /// Returns if we have imports in pending state.
    pub fn has_pending_imports(&mut self) -> bool {
        self.get_state().borrow().module_map.has_pending_imports()
    }

    /// Returns if we have scheduled any next-tick callbacks.
    pub fn has_next_tick_callbacks(&mut self) -> bool {
        !self.get_state().borrow().next_tick_queue.is_empty()
    }
}

impl std::ops::Drop for JsRuntime {
    fn drop(&mut self) {
        // HACK: This is a BIGGGG hack to prevent V8 from segfaulting when the inspector
        // is discarded. The exact cause is unclear, but avoiding manual memory cleanup
        // and allowing the the OS to handle memory purging at the program's
        // termination resolves the issue.
        std::mem::forget(self.inspector.take());
    }
}

// State management specific methods.
// https://github.com/lmt-swallow/puppy-browser/blob/main/src/javascript/runtime.rs

impl JsRuntime {
    /// Returns the runtime state stored in the given isolate.
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JsRuntimeState>> {
        isolate
            .get_slot::<Rc<RefCell<JsRuntimeState>>>()
            .unwrap()
            .clone()
    }

    /// Returns the runtime's state.
    pub fn get_state(&self) -> Rc<RefCell<JsRuntimeState>> {
        Self::state(&self.isolate)
    }

    /// Returns a v8 handle scope for the runtime.
    /// https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html.
    pub fn handle_scope(&mut self) -> v8::HandleScope {
        let context = self.context();
        v8::HandleScope::with_context(&mut self.isolate, context)
    }

    /// Returns a context created for the runtime.
    /// https://v8docs.nodesource.com/node-0.8/df/d69/classv8_1_1_context.html
    pub fn context(&mut self) -> v8::Global<v8::Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }

    /// Returns the inspector created for the runtime.
    pub fn inspector(&mut self) -> Option<Rc<RefCell<JsRuntimeInspector>>> {
        self.inspector.as_ref().cloned()
    }
}

/// Runs callbacks stored in the next-tick queue.
fn run_next_tick_callbacks(scope: &mut v8::HandleScope) {
    let state_rc = JsRuntime::state(scope);
    let callbacks: NextTickQueue = state_rc.borrow_mut().next_tick_queue.drain(..).collect();

    let undefined = v8::undefined(scope);
    let tc_scope = &mut v8::TryCatch::new(scope);

    for (cb, params) in callbacks {
        // Create a local handle for the callback and its parameters.
        let cb = v8::Local::new(tc_scope, cb);
        let args: Vec<v8::Local<v8::Value>> = params
            .iter()
            .map(|arg| v8::Local::new(tc_scope, arg))
            .collect();

        cb.call(tc_scope, undefined.into(), &args);

        // On exception, report it and handle the error.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let mut state = state_rc.borrow_mut();
            state.exceptions.capture_exception(exception);

            drop(state);

            // Check for uncaught errors (capture callbacks might be in place).
            if let Some(error) = check_exceptions(tc_scope) {
                report_and_exit(error);
            }
        }
    }

    tc_scope.perform_microtask_checkpoint();
}

// Returns an error if an uncaught exception or unhandled rejection has been captured.
pub fn check_exceptions(scope: &mut v8::HandleScope) -> Option<JsError> {
    let state_rc = JsRuntime::state(scope);
    let maybe_exception = state_rc.borrow_mut().exceptions.exception.take();

    // Check for uncaught exceptions first.
    if let Some(exception) = maybe_exception {
        let state = state_rc.borrow();
        let exception = v8::Local::new(scope, exception);
        if let Some(callback) = state.exceptions.uncaught_exception_cb.as_ref() {
            let callback = v8::Local::new(scope, callback);
            let undefined = v8::undefined(scope).into();
            let origin = v8::String::new(scope, "uncaughtException").unwrap();
            let tc_scope = &mut v8::TryCatch::new(scope);
            drop(state);

            callback.call(tc_scope, undefined, &[exception, origin.into()]);

            // Note: To avoid infinite recursion with these hooks, if this
            // function throws, return it as error.
            if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                let exception = v8::Local::new(tc_scope, exception);
                let error = JsError::from_v8_exception(tc_scope, exception, None);
                return Some(error);
            }

            return None;
        }

        let error = JsError::from_v8_exception(scope, exception, None);
        return Some(error);
    }

    let promise_rejections: Vec<PromiseRejectionEntry> = state_rc
        .borrow_mut()
        .exceptions
        .promise_rejections
        .drain(..)
        .collect();

    // Then, check for unhandled rejections.
    for (promise, exception) in promise_rejections.iter() {
        let state = state_rc.borrow_mut();
        let promise = v8::Local::new(scope, promise);
        let exception = v8::Local::new(scope, exception);

        // If the `unhandled_rejection_cb` is set, invoke it to handle the promise rejection.
        if let Some(callback) = state.exceptions.unhandled_rejection_cb.as_ref() {
            let callback = v8::Local::new(scope, callback);
            let undefined = v8::undefined(scope).into();
            let tc_scope = &mut v8::TryCatch::new(scope);
            drop(state);

            callback.call(tc_scope, undefined, &[exception, promise.into()]);

            // Note: To avoid infinite recursion with these hooks, if this
            // function throws, return it as error.
            if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                let exception = v8::Local::new(tc_scope, exception);
                let error = JsError::from_v8_exception(tc_scope, exception, None);
                return Some(error);
            }

            continue;
        }

        // If the `uncaught_exception_cb` is set, invoke it to handle the promise rejection.
        if let Some(callback) = state.exceptions.uncaught_exception_cb.as_ref() {
            let callback = v8::Local::new(scope, callback);
            let undefined = v8::undefined(scope).into();
            let origin = v8::String::new(scope, "unhandledRejection").unwrap();
            let tc_scope = &mut v8::TryCatch::new(scope);
            drop(state);

            callback.call(tc_scope, undefined, &[exception, origin.into()]);

            // Note: To avoid infinite recursion with these hooks, if this
            // function throws, return it as error.
            if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                let exception = v8::Local::new(tc_scope, exception);
                let error = JsError::from_v8_exception(tc_scope, exception, None);
                return Some(error);
            }

            continue;
        }

        let prefix = Some("(in promise) ");
        let error = JsError::from_v8_exception(scope, exception, prefix);

        return Some(error);
    }

    None
}
