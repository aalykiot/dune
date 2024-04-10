use crate::bindings::set_function_to;
use crate::errors::JsError;
use crate::runtime::JsRuntime;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum Policy {
    Exit,
    KeepAlive,
}

pub struct ExceptionState {
    /// Indicates the report policy.
    policy: Policy,
    /// Holds the current uncaught exception.
    exception: Option<v8::Global<v8::Value>>,
    /// Holds uncaught promise rejections.
    promise_rejections: HashMap<v8::Global<v8::Promise>, v8::Global<v8::Value>>,
    /// Pre-hook to run when an uncaught exception is thrown.
    uncaught_exception_monitor_cb: Option<v8::Global<v8::Function>>,
    /// Hook to run on an uncaught exception.
    uncaught_exception_cb: Option<v8::Global<v8::Function>>,
    /// Hook to run on an uncaught promise rejection.
    unhandled_rejection_cb: Option<v8::Global<v8::Function>>,
}

impl ExceptionState {
    /// Creates a new store with given report policy.
    pub fn new_with_policy(policy: Policy) -> Self {
        ExceptionState {
            policy,
            exception: None,
            promise_rejections: HashMap::new(),
            uncaught_exception_monitor_cb: None,
            uncaught_exception_cb: None,
            unhandled_rejection_cb: None,
        }
    }

    /// Registers the uncaught exception.
    pub fn emit_exception(&mut self, exception: v8::Global<v8::Value>) {
        if self.exception.is_none() {
            self.exception = Some(exception);
        }
    }

    /// Registers a promise rejection to the store.
    pub fn emit_promise_rejection(
        &mut self,
        promise: v8::Global<v8::Promise>,
        reason: v8::Global<v8::Value>,
    ) {
        self.promise_rejections.insert(promise, reason);
    }

    /// Processes all uncaught exceptions and uncaught promise rejections.
    pub fn report(&mut self, scope: &mut v8::HandleScope<'_>) {
        // Check for normal uncaught exceptions
        if let Some(exception) = self.exception.take() {
            let exception = v8::Local::new(scope, exception);
            let error = JsError::from_v8_exception(scope, exception, None);

            self.run_exception_monitor_cb(scope, exception, "uncaughtException");
            eprintln!("{error:?}");

            match self.policy {
                Policy::KeepAlive => return,
                Policy::Exit if self.uncaught_exception_cb.is_some() => return,
                Policy::Exit => std::process::exit(1),
            };
        }

        // Check for uncaught promise rejections and report them.
        if let Some((_, rejection)) = self.promise_rejections.iter().next() {
            let rejection = v8::Local::new(scope, rejection);
            let error = JsError::from_v8_exception(scope, rejection, Some("(in promise) "));

            self.run_exception_monitor_cb(scope, rejection, "unhandledRejection");
            self.promise_rejections.clear();

            eprintln!("{error:?}");

            match self.policy {
                Policy::KeepAlive => {}
                Policy::Exit if self.unhandled_rejection_cb.is_some() => {}
                Policy::Exit => std::process::exit(1),
            };
        }
    }

    pub fn run_exception_monitor_cb(
        &self,
        scope: &mut v8::HandleScope<'_>,
        exception: v8::Local<'_, v8::Value>,
        origin: &str,
    ) {
        if let Some(callback) = self.uncaught_exception_monitor_cb.as_ref() {
            let undefined = v8::undefined(scope).into();
            let callback = v8::Local::new(scope, callback);
            let origin = v8::String::new(scope, origin).unwrap();

            let tc_scope = &mut v8::TryCatch::new(scope);
            callback.call(tc_scope, undefined, &[exception, origin.into()]);

            // Note: To avoid infinite recursion with these hooks, if this
            // function throws, exit immediately.
            if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                let exception = v8::Local::new(tc_scope, exception);
                let error = JsError::from_v8_exception(tc_scope, exception, None);
                eprintln!("{error:?}");
                std::process::exit(1);
            }
        }
    }

    pub fn remove_promise_rejection(&mut self, promise: &v8::Global<v8::Promise>) {
        self.promise_rejections.remove(promise);
    }

    pub fn has_promise_rejection(&self) -> bool {
        !self.promise_rejections.is_empty()
    }

    pub fn set_report_policy(&mut self, policy: Policy) {
        self.policy = policy;
    }

    pub fn set_uncaught_exception_monitor_callback(&mut self, callback: v8::Global<v8::Function>) {
        self.uncaught_exception_monitor_cb = Some(callback);
    }

    pub fn set_uncaught_exception_callback(&mut self, callback: v8::Global<v8::Function>) {
        self.uncaught_exception_cb = Some(callback);
    }

    pub fn set_unhandled_rejection_callback(&mut self, callback: v8::Global<v8::Function>) {
        self.unhandled_rejection_cb = Some(callback);
    }
}

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(
        scope,
        target,
        "setUncaughtExceptionMonitorCallback",
        set_uncaught_exception_monitor_callback,
    );

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

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Setting the `uncaught_exception_monitor_callback` from JavaScript.
fn set_uncaught_exception_monitor_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the callback from JavaScript.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = v8::Global::new(scope, callback);

    let state_rc = JsRuntime::state(scope);

    state_rc
        .borrow_mut()
        .exceptions
        .set_uncaught_exception_monitor_callback(callback);

    rv.set(v8::Boolean::new(scope, true).into());
}

/// Setting the `uncaught_exception_callback` from JavaScript.
fn set_uncaught_exception_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the callback from JavaScript.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = v8::Global::new(scope, callback);

    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    state.exceptions.set_uncaught_exception_callback(callback);

    rv.set(v8::Boolean::new(scope, true).into());
}

/// Setting the `unhandled_rejection_callback` from JavaScript.
fn set_unhandled_rejection_callback(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the callback from JavaScript.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = v8::Global::new(scope, callback);

    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    state.exceptions.set_unhandled_rejection_callback(callback);

    rv.set(v8::Boolean::new(scope, true).into());
}
