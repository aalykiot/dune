use crate::bindings;
use crate::exceptions;
use crate::modules::{create_origin, ModuleMap};
use crate::stdio;
use anyhow::{bail, Error};
use rusty_v8 as v8;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Once;

// Function pointer for the bindings initializers.
type BindingInitFn = fn(&mut v8::HandleScope<'_>) -> v8::Global<v8::Object>;

// `JsRuntimeState` defines a state that will be stored per v8 isolate.
pub struct JsRuntimeState {
    // A sand-boxed execution context with its own set of built-in objects and functions.
    // https://v8.dev/docs/embed#contexts
    pub context: v8::Global<v8::Context>,
    // Holds information about resolved ES modules.
    // https://hacks.mozilla.org/2018/03/es-modules-a-cartoon-deep-dive/
    pub module_map: ModuleMap,
    // Holds initializers for Rust native bindings.
    // https://stackoverflow.com/questions/20382396/what-are-node-js-bindings
    pub bindings: HashMap<&'static str, BindingInitFn>,
}

pub struct JsRuntime {
    // A VM instance with its own heap.
    // https://v8docs.nodesource.com/node-0.8/d5/dda/classv8_1_1_isolate.html
    isolate: v8::OwnedIsolate,
}

impl JsRuntime {
    pub fn new() -> JsRuntime {
        // Firing up the v8 engine.
        static V8_INIT: Once = Once::new();
        V8_INIT.call_once(move || {
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        });

        let mut isolate = v8::Isolate::new(v8::CreateParams::default());

        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let context = bindings::create_new_context(scope);
            v8::Global::new(scope, context)
        };

        // Creating the native bindings map.
        let mut bindings: HashMap<&'static str, BindingInitFn> = HashMap::new();

        bindings.insert("stdio", stdio::initialize);

        // Storing state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState {
            context,
            module_map: ModuleMap::default(),
            bindings,
        })));

        JsRuntime { isolate }
    }

    pub fn eval(&mut self, filename: &str, source: &str) -> Result<String, Error> {
        // Getting a reference to isolate's handle scope.
        let scope = &mut self.get_handle_scope();

        let origin = create_origin(scope, filename, false);
        let source = v8::String::new(scope, source).unwrap();

        // The `TryCatch` scope allows us to catch runtime errors rather than panicking.
        let mut tc_scope = v8::TryCatch::new(scope);
        let script = match v8::Script::compile(&mut tc_scope, source, Some(&origin)) {
            Some(script) => script,
            None => {
                assert!(tc_scope.has_caught());
                bail!("{}", exceptions::to_pretty_string(tc_scope));
            }
        };

        match script.run(&mut tc_scope) {
            Some(result) => Ok(result
                .to_string(&mut tc_scope)
                .unwrap()
                .to_rust_string_lossy(&mut tc_scope)),
            None => {
                assert!(tc_scope.has_caught());
                bail!("{}", exceptions::to_pretty_string(tc_scope));
            }
        }
    }
}

// State management implementation.
impl JsRuntime {
    // Returns the runtime state stored in the given isolate.
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JsRuntimeState>> {
        isolate
            .get_slot::<Rc<RefCell<JsRuntimeState>>>()
            .unwrap()
            .clone()
    }

    // Returns the runtime's state.
    pub fn get_state(&self) -> Rc<RefCell<JsRuntimeState>> {
        Self::state(&self.isolate)
    }

    // Returns a v8 handle scope for the runtime.
    // https://v8docs.nodesource.com/node-0.8/d3/d95/classv8_1_1_handle_scope.html.
    pub fn get_handle_scope(&mut self) -> v8::HandleScope {
        let context = self.get_context();
        v8::HandleScope::with_context(&mut self.isolate, context)
    }

    // Returns a context created for the runtime.
    // https://v8docs.nodesource.com/node-0.8/df/d69/classv8_1_1_context.html
    pub fn get_context(&mut self) -> v8::Global<v8::Context> {
        let state = self.get_state();
        let state = state.borrow();
        state.context.clone()
    }
}
