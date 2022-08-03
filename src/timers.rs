use crate::bindings::set_function_to;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "createTimeout", create_timeout);
    set_function_to(scope, target, "destroyTimeout", destroy_timeout);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

struct TimerFuture {
    callback: Rc<v8::Global<v8::Function>>,
    params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimerFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let undefined = v8::undefined(scope).into();
        let callback = v8::Local::new(scope, (*self.callback).clone());
        let args: Vec<v8::Local<v8::Value>> = self
            .params
            .iter()
            .map(|arg| v8::Local::new(scope, arg))
            .collect();

        callback.call(scope, undefined, &args);
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

    let timer_cb = {
        let state_rc = state_rc.clone();
        move || {
            let mut state = state_rc.borrow_mut();
            let future = TimerFuture {
                callback: Rc::clone(&callback),
                params: Rc::clone(&params),
            };
            state.pending_futures.push(Box::new(future));
            state.check_and_interrupt();
        }
    };

    // Schedule a new timer to the event-loop.
    let state = state_rc.borrow();
    let id = state.handle.timer(millis, false, timer_cb);

    // Return timeout's internal id.
    rv.set(v8::Number::new(scope, id as f64).into());
}

/// Removes a scheduled timeout from the event-loop.
fn destroy_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get timeout's ID, and remove it.
    let id = args.get(0).int32_value(scope).unwrap() as u32;

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    state.handle.remove_timer(&id);
}
