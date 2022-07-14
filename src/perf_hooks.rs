// Performance measurement APIs
//
// This module provides an implementation of a subset of the W3C Web Performance APIs
// https://nodejs.org/api/perf_hooks.html#performance-measurement-apis

use crate::bindings::create_object_under;
use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::JsRuntime;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);
    let performance = create_object_under(scope, target, "performance");

    // `performance.now()` - returns the current high resolution millisecond timestamp.
    set_function_to(scope, performance, "now", now);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let time_origin = v8::Number::new(scope, state.time_origin as f64);

    // `performance.timeOrigin` - the UNIX timestamp which the current process began.
    set_property_to(scope, performance, "timeOrigin", time_origin.into());

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

fn now(scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get a reference to runtime's state.
    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // Get elapsed time from the start of the process.
    let elapsed_time = state.time.elapsed().as_millis() as f64;
    let elapsed_time = v8::Number::new(scope, elapsed_time);

    rv.set(elapsed_time.into());
}
