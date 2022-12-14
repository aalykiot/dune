use crate::bindings;
use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::event_loop::EventLoop;
use crate::event_loop::LoopHandle;
use crate::event_loop::LoopInterruptHandle;
use crate::event_loop::TaskResult;
use crate::hooks::host_import_module_dynamically_cb;
use crate::hooks::host_initialize_import_meta_object_cb;
use crate::hooks::module_resolve_cb;
use crate::hooks::promise_reject_cb;
use crate::modules::create_origin;
use crate::modules::fetch_module_tree;
use crate::modules::load_import;
use crate::modules::resolve_import;
use crate::modules::DynamicImportFuture;
use crate::modules::ImportMap;
use crate::modules::ModuleMap;
use anyhow::bail;
use anyhow::Error;
use std::cell::RefCell;
use std::collections::HashMap;
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
    pub modules: ModuleMap,
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
    /// Holds exceptions from promises with no rejection handler.
    pub promise_exceptions: HashMap<v8::Global<v8::Promise>, v8::Global<v8::Value>>,
    /// Runtime options.
    pub options: JsRuntimeOptions,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct JsRuntimeOptions {
    // The seed used in Math.random() method.
    pub seed: Option<i64>,
    // Reloads every URL import.
    pub reload: bool,
    // Holds user defined import maps for module loading.
    pub import_map: Option<ImportMap>,
}

pub struct JsRuntime {
    /// A VM instance with its own heap.
    /// https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
    /// The event-loop instance that takes care of polling for I/O.
    pub event_loop: EventLoop,
}

impl JsRuntime {
    /// Creates a new JsRuntime.
    pub fn new() -> JsRuntime {
        Self::with_options(JsRuntimeOptions::default())
    }

    /// Creates a new JsRuntime based on provided options.
    pub fn with_options(options: JsRuntimeOptions) -> JsRuntime {
        // Configuration flags for V8.
        let flags = concat!(
            " --harmony-import-assertions",
            " --turbo_fast_api_calls",
            " --no-validate-asm",
            " --harmony-change-array-by-copy"
        );

        if options.seed.is_some() {
            v8::V8::set_flags_from_string(&format!(
                "{} --predictable --random-seed={}",
                flags,
                options.seed.unwrap()
            ));
        } else {
            v8::V8::set_flags_from_string(flags);
        }

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

        let event_loop = EventLoop::new();

        let time_origin = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Store state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState {
            context,
            modules: ModuleMap::default(),
            handle: event_loop.handle(),
            interrupt_handle: event_loop.interrupt_handle(),
            pending_futures: Vec::new(),
            startup_moment: Instant::now(),
            time_origin,
            next_tick_queue: Vec::new(),
            promise_exceptions: HashMap::new(),
            options,
        })));

        let mut runtime = JsRuntime {
            isolate,
            event_loop,
        };

        // Initialize core environment. (see lib/main.js)
        let main = include_str!("./js/main.js");
        unwrap_or_exit(runtime.execute_module("dune:environment/main", Some(main)));

        runtime
    }

    /// Executes traditional JavaScript code (traditional = not ES modules).
    pub fn execute_script(
        &mut self,
        filename: &str,
        source: &str,
    ) -> Result<v8::Global<v8::Value>, Error> {
        // Get the handle-scope.
        let scope = &mut self.handle_scope();

        let origin = create_origin(scope, filename, false);
        let source = v8::String::new(scope, source).unwrap();

        // The `TryCatch` scope allows us to catch runtime errors rather than panicking.
        let tc_scope = &mut v8::TryCatch::new(scope);

        let script = match v8::Script::compile(tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                bail!(JsError::from_v8_exception(tc_scope, exception, None));
            }
        };

        match script.run(tc_scope) {
            Some(value) => Ok(v8::Global::new(tc_scope, value)),
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                bail!(JsError::from_v8_exception(tc_scope, exception, None));
            }
        }
    }

    /// Executes JavaScript code as ES module.
    pub fn execute_module(
        &mut self,
        filename: &str,
        source: Option<&str>,
    ) -> Result<v8::Global<v8::Value>, Error> {
        let scope = &mut self.handle_scope();
        let import_map = JsRuntime::state(scope).borrow().options.import_map.clone();

        // The following code allows the runtime to load the core JavaScript
        // environment (lib/main.js) that does not have a valid
        // filename since it's loaded from memory.
        let filename = match source.is_some() {
            true => filename.to_string(),
            false => unwrap_or_exit(resolve_import(None, filename, import_map)),
        };

        let tc_scope = &mut v8::TryCatch::new(scope);

        let module = match fetch_module_tree(tc_scope, &filename, source) {
            Some(module) => module,
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                bail!(JsError::from_v8_exception(tc_scope, exception, None));
            }
        };

        if module
            .instantiate_module(tc_scope, module_resolve_cb)
            .is_none()
        {
            assert!(tc_scope.has_caught());
            let exception = tc_scope.exception().unwrap();
            bail!(JsError::from_v8_exception(tc_scope, exception, None));
        }

        let module_result = module.evaluate(tc_scope);

        if module.get_status() == v8::ModuleStatus::Errored {
            let exception = module.get_exception();
            bail!(JsError::from_v8_exception(tc_scope, exception, None));
        }

        match module_result {
            Some(value) => Ok(v8::Global::new(tc_scope, value)),
            None => bail!(generic_error(
                "Cannot evaluate module, because JavaScript execution has been terminated."
            )),
        }
    }

    /// Runs a single tick of the event-loop.
    pub fn tick_event_loop(&mut self) {
        self.event_loop.tick();
        self.run_pending_futures();
        self.prepare_dynamic_imports();
    }

    /// Runs the event-loop until no more pending events exists.
    pub fn run_event_loop(&mut self) {
        // Run the default MicrotaskQueue/NextTickQueue until they get empty.
        run_next_tick_callbacks(&mut self.handle_scope());

        while self.event_loop.has_pending_events()
            || self.has_promise_rejections()
            || self.isolate.has_pending_background_tasks()
            || self.has_pending_dynamic_imports()
        {
            // Tick the event loop.
            self.tick_event_loop();
            // Report (and exit) if any unhandled promise rejection has been caught.
            if self.has_promise_rejections() {
                println!("{:?}", self.promise_rejections().remove(0));
                std::process::exit(1);
            }
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
        // the MicrotaskQueue and then the NextTickQueue.

        for mut fut in futures {
            fut.run(scope);
            run_next_tick_callbacks(scope);
        }
    }

    /// Returns if unhandled promise rejections where caught.
    pub fn has_promise_rejections(&mut self) -> bool {
        !self.get_state().borrow().promise_exceptions.is_empty()
    }

    /// Returns if we have dynamic imports in pending state.
    pub fn has_pending_dynamic_imports(&mut self) -> bool {
        !self.get_state().borrow().modules.dynamic_imports.is_empty()
    }

    /// Returns all promise unhandled rejections.
    pub fn promise_rejections(&mut self) -> Vec<JsError> {
        // Get a v8 handle-scope.
        let scope = &mut self.handle_scope();

        // Get access to the state.
        let state_rc = JsRuntime::state(scope);
        let mut state = state_rc.borrow_mut();

        state
            .promise_exceptions
            .drain()
            .map(|(_, value)| {
                let exception = v8::Local::new(scope, value);
                JsError::from_v8_exception(scope, exception, Some("(in promise) "))
            })
            .collect()
    }

    /// Loads pending dynamic imports using the event-loop.
    pub fn prepare_dynamic_imports(&mut self) {
        // Get a v8 handle-scope.
        let scope = &mut self.handle_scope();
        let state_rc = JsRuntime::state(scope);

        // NOTE: The reason we move all the dynamic imports to a separate vec is because
        // we need to drop the `state` borrow before we start iterating through all
        // of them to avoid borrowing panics at runtime.
        let dynamic_imports: Vec<(String, v8::Global<v8::PromiseResolver>)> = state_rc
            .borrow_mut()
            .modules
            .dynamic_imports
            .drain(..)
            .collect();

        for (specifier, promise) in dynamic_imports {
            // Borrow runtime's state.
            let mut state = state_rc.borrow_mut();

            // Note: The `dynamic_imports_seen` is there to help us identify concurrent
            // imports with the same specifier. In that case we should only execute the
            // module once and return it's namespace object for every request.
            if state.modules.dynamic_imports_seen.contains(&specifier) {
                // Reschedule since another import with the same specifier is pending
                // (will use the cache to resolve the import later).
                state.modules.new_dynamic_import(scope, &specifier, promise);

                continue;
            }

            state.modules.dynamic_imports_seen.insert(specifier.clone());

            // Use the event-loop to asynchronously load the import.
            let task = {
                let specifier = specifier.clone();
                move || match load_import(&specifier, false) {
                    Ok(source) => Some(Ok(bincode::serialize(&source).unwrap())),
                    Err(e) => Some(Result::Err(e)),
                }
            };

            let task_cb = {
                let state_rc = state_rc.clone();
                move |_: LoopHandle, maybe_result: TaskResult| {
                    let mut state = state_rc.borrow_mut();
                    let future = DynamicImportFuture {
                        specifier,
                        promise,
                        maybe_result,
                    };
                    state.pending_futures.push(Box::new(future));
                }
            };

            state.handle.spawn(task, Some(task_cb));
        }
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

        // On exception, report it and exit.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = JsError::from_v8_exception(tc_scope, exception, None);
            println!("{:?}", exception);
            std::process::exit(1);
        }
    }

    tc_scope.perform_microtask_checkpoint();
}
