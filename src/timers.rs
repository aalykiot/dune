use crate::bindings::set_function_to;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use dune_event_loop::LoopHandle;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "createTimeout", create_timeout);
    set_function_to(scope, target, "removeTimeout", remove_timeout);
    set_function_to(scope, target, "createImmediate", create_immediate);
    set_function_to(scope, target, "removeImmediate", remove_immediate);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

struct TimeoutFuture {
    cb: Rc<v8::Global<v8::Function>>,
    params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimeoutFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let undefined = v8::undefined(scope).into();
        let callback = v8::Local::new(scope, (*self.cb).clone());
        let args: Vec<v8::Local<v8::Value>> = self
            .params
            .iter()
            .map(|arg| v8::Local::new(scope, arg))
            .collect();

        let tc_scope = &mut v8::TryCatch::new(scope);

        callback.call(tc_scope, undefined, &args);

        // Report if callback threw an exception.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let state = JsRuntime::state(tc_scope);
            state.borrow_mut().exceptions.emit_exception(exception);
        }
    }
}

/// Schedules a new timeout to the event-loop.
fn create_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get timer's callback.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));

    // Get timer's expiration time in millis.
    let millis = args.get(1).int32_value(scope).unwrap() as u64;

    // Decide if the timer is an interval.
    let repeatable = args.get(2).to_rust_string_lossy(scope) == "true";

    // Convert params argument (Array<Local<Value>>) to Rust vector.
    let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
        Ok(params) => {
            (0..params.length()).fold(Vec::<v8::Global<v8::Value>>::new(), |mut acc, i| {
                let param = params.get_index(scope, i).unwrap();
                acc.push(v8::Global::new(scope, param));
                acc
            })
        }
        Err(_) => vec![],
    };

    let state_rc = JsRuntime::state(scope);
    let params = Rc::new(params);

    let timeout_cb = {
        let state_rc = state_rc.clone();
        move |_: LoopHandle| {
            let mut state = state_rc.borrow_mut();
            let future = TimeoutFuture {
                cb: Rc::clone(&callback),
                params: Rc::clone(&params),
            };
            state.pending_futures.push(Box::new(future));

            // Note: It's important to send an interrupt signal to the event-loop to prevent the
            // event-loop from idling in the poll phase, waiting for I/O, while the timer's JS
            // future is ready in the runtime level.
            if !state.wake_event_queued {
                state.interrupt_handle.interrupt();
                state.wake_event_queued = true;
            }
        }
    };

    // Schedule a new timer to the event-loop.
    let state = state_rc.borrow();
    let id = state.handle.timer(millis, repeatable, timeout_cb);

    // Return timeout's internal id.
    rv.set(v8::Number::new(scope, id as f64).into());
}

/// Removes a scheduled timeout from the event-loop.
fn remove_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get timeout's ID, and remove it.
    let id = args.get(0).int32_value(scope).unwrap() as u32;
    let state_rc = JsRuntime::state(scope);

    state_rc.borrow().handle.remove_timer(&id);
}

struct ImmediateFuture {
    cb: Rc<v8::Global<v8::Function>>,
    params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for ImmediateFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let undefined = v8::undefined(scope).into();
        let callback = v8::Local::new(scope, (*self.cb).clone());
        let args: Vec<v8::Local<v8::Value>> = self
            .params
            .iter()
            .map(|arg| v8::Local::new(scope, arg))
            .collect();

        let tc_scope = &mut v8::TryCatch::new(scope);

        callback.call(tc_scope, undefined, &args);

        // On exception, report it and exit.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let state = JsRuntime::state(tc_scope);
            state.borrow_mut().exceptions.emit_exception(exception);
        }
    }
}

/// Schedules a new immediate timer (aka check callback).
fn create_immediate(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get timer's callback.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));

    // Convert params argument (Array<Local<Value>>) to Rust vector.
    let params = match v8::Local::<v8::Array>::try_from(args.get(2)) {
        Ok(params) => {
            (0..params.length()).fold(Vec::<v8::Global<v8::Value>>::new(), |mut acc, i| {
                let param = params.get_index(scope, i).unwrap();
                acc.push(v8::Global::new(scope, param));
                acc
            })
        }
        Err(_) => vec![],
    };

    let state_rc = JsRuntime::state(scope);
    let params = Rc::new(params);

    let immediate_cb = {
        let state_rc = state_rc.clone();
        move |_: LoopHandle| {
            let mut state = state_rc.borrow_mut();
            let future = ImmediateFuture {
                cb: Rc::clone(&callback),
                params: Rc::clone(&params),
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Schedule a check callback.
    let state = state_rc.borrow();
    let id = state.handle.check(immediate_cb);

    // Return immediate's internal id.
    rv.set(v8::Number::new(scope, id as f64).into());
}

/// Removes a scheduled immediate timer.
fn remove_immediate(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get timeout's ID, and remove it.
    let id = args.get(0).int32_value(scope).unwrap() as u32;
    let state_rc = JsRuntime::state(scope);

    state_rc.borrow().handle.remove_check(&id);
}
