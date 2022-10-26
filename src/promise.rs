use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::throw_exception;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "peek", peek);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Inspects the status and contents of a Promise object.
fn peek(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Cast provided argument into a Promise.
    let promise: v8::Local<v8::Promise> = match args.get(0).try_into() {
        Ok(value) => value,
        Err(_) => {
            throw_exception(scope, "The provided object is not a Promise.");
            return;
        }
    };

    // The Promise must not be pending in order to get the result.
    let value = match promise.state() {
        v8::PromiseState::Fulfilled | v8::PromiseState::Rejected => promise.result(scope),
        v8::PromiseState::Pending => v8::undefined(scope).into(),
    };

    let state = match promise.state() {
        v8::PromiseState::Pending => v8::String::new(scope, "PENDING").unwrap(),
        v8::PromiseState::Fulfilled => v8::String::new(scope, "FULFILLED").unwrap(),
        v8::PromiseState::Rejected => v8::String::new(scope, "REJECTED").unwrap(),
    };

    let result = v8::Object::new(scope);

    set_property_to(scope, result, "state", state.into());
    set_property_to(scope, result, "value", value);

    rv.set(result.into());
}
