use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Once;

use rusty_v8 as v8;

// `JsRuntimeState` defines a state of JS runtime that will be stored per v8 isolate.
pub struct JsRuntimeState {
    context: v8::Global<v8::Context>,
}

pub struct JsRuntime {
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
            let context = v8::Context::new(scope);
            v8::Global::new(scope, context)
        };

        // Storing state inside the v8 isolate slot.
        // https://v8docs.nodesource.com/node-4.8/d5/dda/classv8_1_1_isolate.html#a7acadfe7965997e9c386a05f098fbe36
        isolate.set_slot(Rc::new(RefCell::new(JsRuntimeState { context })));

        JsRuntime { isolate }
    }
}

// State management implementation
impl JsRuntime {
    // Returns the runtime state stored in the given isolate.
    pub fn state(isolate: &v8::Isolate) -> Rc<RefCell<JsRuntimeState>> {
        isolate
            .get_slot::<Rc<RefCell<JsRuntimeState>>>()
            .unwrap()
            .clone()
    }

    // Returns the runtime state for the runtime.
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
