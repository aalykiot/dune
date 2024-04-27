use crate::bindings::set_function_to;
use crate::runtime::JsRuntime;
use std::collections::HashMap;

pub struct ExceptionState {
    /// Holds the current uncaught exception.
    pub exception: Option<v8::Global<v8::Value>>,
    /// Holds uncaught promise rejections.
    pub promise_rejections: HashMap<v8::Global<v8::Promise>, v8::Global<v8::Value>>,
    /// Hook to run on an uncaught exception.
    pub uncaught_exception_cb: Option<v8::Global<v8::Function>>,
    /// Hook to run on an uncaught promise rejection.
    pub unhandled_rejection_cb: Option<v8::Global<v8::Function>>,
}

impl ExceptionState {
    /// Creates a new store with given report policy.
    pub fn new() -> Self {
        ExceptionState {
            exception: None,
            promise_rejections: HashMap::new(),
            uncaught_exception_cb: None,
            unhandled_rejection_cb: None,
        }
    }

    /// Registers the uncaught exception.
    pub fn capture_exception(&mut self, exception: v8::Global<v8::Value>) {
        if self.exception.is_none() {
            self.exception = Some(exception);
        }
    }

    /// Registers a promise rejection to the store.
    pub fn capture_promise_rejection(
        &mut self,
        promise: v8::Global<v8::Promise>,
        reason: v8::Global<v8::Value>,
    ) {
        self.promise_rejections.insert(promise, reason);
    }

    pub fn has_promise_rejection(&self) -> bool {
        !self.promise_rejections.is_empty()
    }

    pub fn remove_promise_rejection(&mut self, promise: &v8::Global<v8::Promise>) {
        self.promise_rejections.remove(promise);
    }

    pub fn remove_promise_rejection_entry(&mut self, exception: &v8::Global<v8::Value>) {
        // Find the correct entry to remove.
        let mut key_to_remove = None;
        for (key, value) in self.promise_rejections.iter() {
            if value == exception {
                key_to_remove = Some(key.clone());
                break;
            }
        }

        if let Some(promise) = key_to_remove {
            self.promise_rejections.remove(&promise);
        }
    }

    pub fn set_uncaught_exception_callback(&mut self, callback: Option<v8::Global<v8::Function>>) {
        self.uncaught_exception_cb = callback;
    }

    pub fn set_unhandled_rejection_callback(&mut self, callback: Option<v8::Global<v8::Function>>) {
        self.unhandled_rejection_cb = callback;
    }
}

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(
        scope,
        target,
        "setUncaughtExceptionCallback",
        set_uncaught_exception_callback,
    );

    set_function_to(
        scope,
        target,
        "setUnhandledRejectionCallback",
        set_unhandled_rejection_callback,
    );

    set_function_to(scope, target, "emitException", emit_exception);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Setting the `uncaught_exception_callback` from JavaScript.
fn set_uncaught_exception_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Note: Passing `null` from JavaScript essentially will unset the defined callback.
    let callback = match v8::Local::<v8::Function>::try_from(args.get(0)) {
        Ok(callback) => Some(v8::Global::new(scope, callback)),
        Err(_) => None,
    };

    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    state.exceptions.set_uncaught_exception_callback(callback);
}

/// Setting the `unhandled_rejection_callback` from JavaScript.
fn set_unhandled_rejection_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Note: Passing `null` from JavaScript essentially will unset the defined callback.
    let callback = match v8::Local::<v8::Function>::try_from(args.get(0)) {
        Ok(callback) => Some(v8::Global::new(scope, callback)),
        Err(_) => None,
    };

    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    state.exceptions.set_unhandled_rejection_callback(callback);
}

/// Manually setting the current exception from JavaScript.
fn emit_exception(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();
    let exception = v8::Global::new(scope, args.get(0));

    state.exceptions.capture_exception(exception);
}
