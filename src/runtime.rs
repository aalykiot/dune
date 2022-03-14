use crate::bindings;
use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::hooks::module_resolve_cb;
use crate::modules::create_origin;
use crate::modules::fetch_module_tree;
use crate::modules::resolve_import;
use crate::modules::ModuleMap;
use crate::timers::Timeout;
use anyhow::bail;
use anyhow::Error;
use nanoid::nanoid;
use rusty_v8 as v8;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;
use std::time::Duration;
use std::time::Instant;

/// Completion type of an asynchronous operation.
pub enum AsyncHandle {
    /// JavaScript promise.
    Promise(v8::Global<v8::PromiseResolver>),
    /// JavaScript callback.
    Callback(v8::Global<v8::Function>),
}

/// The state to be stored per v8 isolate.
pub struct JsRuntimeState {
    /// A sand-boxed execution context with its own set of built-in objects and functions.
    pub context: v8::Global<v8::Context>,
    /// Holds information about resolved ES modules.
    pub modules: ModuleMap,
    /// The number of events keeping the event-loop alive.
    pub(crate) pending_events: usize,
    /// Holds the timers.
    pub(crate) timers: BTreeMap<Instant, Timeout>,
    /// Holds timers that are removed manually by JavaScript.
    pub(crate) timers_bin: Vec<usize>,
    /// Holds completion handles for async operations.
    pub(crate) async_handles: HashMap<String, AsyncHandle>,
}

pub struct JsRuntime {
    /// A VM instance with its own heap.
    /// https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
}

impl JsRuntime {
    pub fn new() -> JsRuntime {
        // Firing up the v8 engine under the hood.
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
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = bindings::create_new_context(scope);
            v8::Global::new(scope, context)
        };

        // Storing state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState {
            context,
            modules: ModuleMap::default(),
            pending_events: 0,
            timers: BTreeMap::new(),
            timers_bin: vec![],
            async_handles: HashMap::new(),
        })));

        let mut runtime = JsRuntime { isolate };

        // Initializing the core environment. (see lib/main.js)
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
        // Getting a reference to isolate's handle scope.
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

    /// Executes JavaScript ES modules.
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
}

// ----------------------------------------------------
// State management implementation.
// ----------------------------------------------------

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

// ----------------------------------------------------
// Event-Loop specific methods.
// ----------------------------------------------------

impl JsRuntime {
    /// Registers an async handle to the event-loop.
    pub fn ev_register_async_handle(isolate: &v8::Isolate, handle: AsyncHandle) -> String {
        // We need to get a mut reference to the isolate's state first.
        let state = Self::state(isolate);
        let mut state = state.borrow_mut();
        // The key (aka ID) of the handle will be a short UUID value.
        let key = nanoid!();
        state.async_handles.insert(key.clone(), handle);

        key
    }

    /// Registers a new timeout to the timers shorted list.
    pub fn ev_register_timeout(isolate: &v8::Isolate, timeout: Timeout) {
        // We need to get a mut reference to the isolate's state first.
        let state = Self::state(isolate);
        let mut state = state.borrow_mut();

        // Calculate the next time the timeout will go OFF!
        let now = Instant::now();
        let duration = now + Duration::from_millis(timeout.delay);

        state.timers.insert(duration, timeout);

        // Increase the pending events
        state.pending_events += 1;
    }

    /// Removes a timeout from the event-loop.
    pub fn ev_remove_timeout(isolate: &v8::Isolate, id: usize) {
        // We need to get a mut reference to the isolate's state first.
        let state = Self::state(isolate);
        let mut state = state.borrow_mut();

        state.timers_bin.push(id);

        // Find the timestamp of the timeout we want to remove.
        let timestamp = match state.timers.iter().find(|(_, t)| t.id == id) {
            Some((t, _)) => *t,
            None => return,
        };

        // If it's a valid timeout, remove it from the list and delete
        // it's async_handle.
        if let Some(timeout) = state.timers.remove(&timestamp) {
            state.async_handles.remove(&timeout.handle).unwrap();
            state.pending_events -= 1;
        }
    }

    /// Processes all the expired timeouts.
    fn ev_run_timers(&mut self) {
        // Get runtime's handle scope.
        let scope = &mut self.handle_scope();
        let undefined = v8::undefined(scope).into();

        // We need to get a pointer to isolate's state.
        let state = Self::state(scope);

        // Finding the timeouts we have to process.
        let timestamps =
            state
                .borrow()
                .timers
                .range(..=Instant::now())
                .fold(vec![], |mut acc, (t, _)| {
                    acc.push(*t);
                    acc
                });

        timestamps.iter().for_each(|key| {
            // Removing the timeout from the list is an acceptable approach
            // since if it's an one-off timeout it will be removed anyway and,
            // if it's a recurring will be rescheduled with a different timestamp key.
            let timeout = match state.borrow_mut().timers.remove(key) {
                Some(timeout) => timeout,
                None => return,
            };

            // Create a local v8 handle for each argument.
            let args: Vec<v8::Local<v8::Value>> = timeout
                .args
                .iter()
                .map(|arg| v8::Local::new(scope, arg))
                .collect();

            // Get timeout's completion handle.
            let handle = {
                let state = state.borrow();
                let handle = state.async_handles.get(&timeout.handle).unwrap();
                let handle = match &handle {
                    AsyncHandle::Callback(cb) => cb,
                    _ => unreachable!(),
                };
                handle.clone()
            };

            let callback = v8::Local::new(scope, handle);

            // Run timeout's callback.
            callback.call(scope, undefined, args.as_slice());

            // We'll decrement the pending events count for every timeout
            // and let the `ev_register_timeout` method  later handle the
            // increments for the recurring ones.
            state.borrow_mut().pending_events -= 1;

            // The second part of the condition tries to prevent re-registering recurring
            // timers that unregister themselves while running their callback.
            if timeout.repeat && !state.borrow().timers_bin.contains(&timeout.id) {
                Self::ev_register_timeout(scope, timeout);
                return;
            } else {
                // If the timeout is a one-off, remove it's handle from async_handles.
                state
                    .borrow_mut()
                    .async_handles
                    .remove(&timeout.handle)
                    .unwrap();
            }

            // Clean-up the timers bin.
            state.borrow_mut().timers_bin.clear();
        });
    }

    /// Starts the event-loop.
    pub fn run_event_loop(&mut self) {
        let state = self.get_state();
        while state.borrow().pending_events > 0 {
            // Timers.
            self.ev_run_timers();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_script_return_value() {
        let mut runtime = JsRuntime::new();
        let value_global = runtime.execute_script("<anon>", "a = 1 + 2").unwrap();
        {
            let scope = &mut runtime.handle_scope();
            let value = value_global.open(scope);
            assert_eq!(value.integer_value(scope).unwrap(), 3);
        }
        let value_global = runtime.execute_script("<anon>", "b = 'foobar'").unwrap();
        {
            let scope = &mut runtime.handle_scope();
            let value = value_global.open(scope);
            assert!(value.is_string());
            assert_eq!(
                value.to_string(scope).unwrap().to_rust_string_lossy(scope),
                "foobar"
            );
        }
    }

    #[test]
    fn syntax_error() {
        let mut runtime = JsRuntime::new();
        let src = "hocuspocus(";
        let r = runtime.execute_script("i.js", src);
        let e = r.unwrap_err();
        let js_error = e.downcast::<JsError>().unwrap();
        assert_eq!(js_error.end_column, Some(11));
    }
}
