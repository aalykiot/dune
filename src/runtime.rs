use crate::bindings;
use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::event_loop::EventLoop;
use crate::event_loop::LoopHandle;
use crate::hooks::host_initialize_import_meta_object_cb;
use crate::hooks::module_resolve_cb;
use crate::hooks::promise_reject_cb;
use crate::modules::create_origin;
use crate::modules::fetch_module_tree;
use crate::modules::resolve_import;
use crate::modules::ModuleMap;
use anyhow::bail;
use anyhow::Error;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
    /// Holds JS pending futures scheduled by the event-loop.
    pub pending_futures: Vec<Box<dyn JsFuture>>,
    /// Indicates the start time of the process.
    pub startup_moment: Instant,
    /// Specifies the timestamp which the current process began in Unix time.
    pub time_origin: u128,
    /// Holds exceptions from promises with no rejection handler.
    pub promise_exceptions: Vec<v8::Global<v8::Value>>,
}

pub struct JsRuntime {
    /// A VM instance with its own heap.
    /// https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
    /// The event-loop instance that takes care of polling for I/O.
    pub event_loop: EventLoop,
}

impl JsRuntime {
    pub fn new() -> JsRuntime {
        let flags = concat!(
            " --harmony-import-assertions",
            " --turbo_fast_api_calls",
            " --no-validate-asm",
            " --noexperimental-async-stack-tagging-api"
        );
        v8::V8::set_flags_from_string(flags);

        // Fire up the v8 engine.
        static V8_INIT: Once = Once::new();
        V8_INIT.call_once(move || {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        isolate.set_capture_stack_trace_for_uncaught_exceptions(true, 10);
        isolate.set_promise_reject_callback(promise_reject_cb);
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
            pending_futures: Vec::new(),
            startup_moment: Instant::now(),
            time_origin,
            promise_exceptions: Vec::new(),
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
    pub fn tick_event_loop(&mut self) {
        self.event_loop.tick();
        self.run_pending_futures();
    }

    /// Runs the event-loop until no more pending events exists.
    pub fn run_event_loop(&mut self) {
        while self.event_loop.has_pending_events() || self.has_promise_rejections() {
            // Tick the event loop.
            self.tick_event_loop();
            // Report (and exit) if any unhandled promise rejection has been caught.
            if self.has_promise_rejections() {
                let rejection = self.promise_rejections().remove(0);
                let rejection = format!("{:?}", rejection);
                let rejection = rejection.replacen(" ", " (in promise) ", 1);

                println!("{}", rejection);
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

        // Run all pending js tasks.
        for mut future in futures {
            future.run(scope);
        }
    }

    /// Returns if unhandled promise rejections where caught.
    pub fn has_promise_rejections(&mut self) -> bool {
        !self.get_state().borrow().promise_exceptions.is_empty()
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
            .drain(..)
            .map(|value| {
                let exception = v8::Local::new(scope, value);
                JsError::from_v8_exception(scope, exception)
            })
            .collect()
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
