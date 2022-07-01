use crate::bindings;
use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::event_loop::EventLoop;
use crate::event_loop::LoopHandle;
use crate::hooks::module_resolve_cb;
use crate::modules::create_origin;
use crate::modules::fetch_module_tree;
use crate::modules::resolve_import;
use crate::modules::ModuleMap;
use anyhow::bail;
use anyhow::Error;
use rusty_v8 as v8;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

/// Completion type of an asynchronous operation.
pub enum JsAsyncHandle {
    // JavaScript callback.
    Callback(v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>),
    // JavaScript promise.
    Promise(v8::Global<v8::PromiseResolver>),
}

/// The state to be stored per v8 isolate.
pub struct JsRuntimeState {
    /// A sand-boxed execution context with its own set of built-in objects and functions.
    pub context: v8::Global<v8::Context>,
    /// Holds information about resolved ES modules.
    pub modules: ModuleMap,
    /// A handle to the runtime's event-loop.
    pub handle: LoopHandle,
    /// Holds JS pending async handles scheduled by the event-loop.
    pub pending_js_tasks: Vec<JsAsyncHandle>,
}

pub struct JsRuntime {
    /// A VM instance with its own heap.
    /// https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
    /// The event-loop instance that takes care of polling for I/O and scheduling callbacks
    /// to be run based on different sources of events.
    pub event_loop: EventLoop,
}

impl JsRuntime {
    pub fn new() -> JsRuntime {
        // Fire up the v8 engine.
        static V8_INIT: Once = Once::new();
        V8_INIT.call_once(move || {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let flags = concat!(
            " --harmony-import-assertions",
            " --harmony-top-level-await false"
        );
        v8::V8::set_flags_from_string(flags);

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);

        let context = {
            let scope = &mut v8::HandleScope::new(&mut *isolate);
            let context = bindings::create_new_context(scope);
            v8::Global::new(scope, context)
        };

        let event_loop = EventLoop::new();

        // Store state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState {
            context,
            modules: ModuleMap::default(),
            handle: event_loop.handle(),
            pending_js_tasks: Vec::new(),
        })));

        let mut runtime = JsRuntime {
            isolate,
            event_loop,
        };

        // Initialize core environment. (see lib/main.js)
        let main = include_str!("../lib/main.js");
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
                bail!(JsError::from_v8_exception(tc_scope, exception));
            }
        };

        match script.run(tc_scope) {
            Some(value) => Ok(v8::Global::new(tc_scope, value)),
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                bail!(JsError::from_v8_exception(tc_scope, exception));
            }
        }
    }

    /// Executes JavaScript code as ES module.
    pub fn execute_module(
        &mut self,
        filename: &str,
        source: Option<&str>,
    ) -> Result<v8::Global<v8::Value>, Error> {
        // The following code allows the runtime to load the core JavaScript
        // environment (lib/main.js) that does not have a valid
        // filename since it's loaded from memory.
        let filename = match source.is_some() {
            true => filename.to_string(),
            false => unwrap_or_exit(resolve_import(None, filename)),
        };

        let scope = &mut self.handle_scope();
        let tc_scope = &mut v8::TryCatch::new(scope);

        let module = match fetch_module_tree(tc_scope, &filename, source) {
            Some(module) => module,
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                bail!(JsError::from_v8_exception(tc_scope, exception));
            }
        };

        if module
            .instantiate_module(tc_scope, module_resolve_cb)
            .is_none()
        {
            assert!(tc_scope.has_caught());
            let exception = tc_scope.exception().unwrap();
            bail!(JsError::from_v8_exception(tc_scope, exception));
        }

        let module_result = module.evaluate(tc_scope);

        if module.get_status() == v8::ModuleStatus::Errored {
            let exception = module.get_exception();
            bail!(JsError::from_v8_exception(tc_scope, exception));
        }

        match module_result {
            Some(value) => Ok(v8::Global::new(tc_scope, value)),
            None => bail!(generic_error(
                "Cannot evaluate module, because JavaScript execution has been terminated."
            )),
        }
    }

    /// Runs a single tick of the event-loop.
    pub fn poll_event_loop(&mut self) {
        self.event_loop.poll();
        self.run_pending_js_tasks();
    }

    /// Runs the event-loop until no more pending events exists.
    pub fn run_event_loop(&mut self) {
        while self.event_loop.has_pending_events() {
            self.poll_event_loop();
        }
    }

    /// Runs all the pending javascript tasks.
    fn run_pending_js_tasks(&mut self) {
        // Get a handle-scope and a reference to the runtime's state.
        let scope = &mut self.handle_scope();
        let state_rc = Self::state(scope);

        // NOTE: The reason we move all the async handles to a separate vec is because
        // we need to drop the `state` borrow before we start iterating through all
        // the handles to avoid borrowing panics at runtime.
        //
        // Example: setTimeout schedules another setTimeout.
        //
        let tasks: Vec<JsAsyncHandle> = state_rc.borrow_mut().pending_js_tasks.drain(..).collect();
        let undefined = v8::undefined(scope).into();

        for task in tasks {
            match task {
                JsAsyncHandle::Callback(callback, params) => {
                    // Create local v8 handles for the callback and the params.
                    let callback = v8::Local::new(scope, callback);
                    let args: Vec<v8::Local<v8::Value>> = params
                        .iter()
                        .map(|arg| v8::Local::new(scope, arg))
                        .collect();

                    // Run callback.
                    callback.call(scope, undefined, &args);
                }
                _ => unimplemented!(),
            }
        }
    }
}

// --- State management specific methods. ---

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
