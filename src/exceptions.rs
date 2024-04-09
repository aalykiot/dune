use crate::errors::JsError;
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
}

impl ExceptionState {
    /// Creates a new store with given report policy.
    pub fn new_with_policy(policy: Policy) -> Self {
        ExceptionState {
            policy,
            exception: None,
            promise_rejections: HashMap::new(),
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
            eprintln!("{error:?}");

            if self.policy == Policy::Exit {
                std::process::exit(1);
            }
        }
        // Check for uncaught promise rejections and report them.
        if let Some((_, rejection)) = self.promise_rejections.drain().next() {
            let rejection = v8::Local::new(scope, rejection);
            let error = JsError::from_v8_exception(scope, rejection, Some("(in promise) "));
            eprintln!("{error:?}");

            if self.policy == Policy::Exit {
                std::process::exit(1);
            }
        }
    }

    /// Removes a promise rejection from the store.
    pub fn remove_promise_rejection(&mut self, promise: &v8::Global<v8::Promise>) {
        self.promise_rejections.remove(promise);
    }

    /// Returns if we have uncaught promise rejections.
    pub fn has_promise_rejection(&self) -> bool {
        !self.promise_rejections.is_empty()
    }

    /// Sets the reporting policy on uncaught exceptions.
    pub fn set_report_policy(&mut self, policy: Policy) {
        self.policy = policy;
    }
}
