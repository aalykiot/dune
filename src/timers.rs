use crate::bindings::set_function_to;
use crate::runtime::AsyncHandle;
use crate::runtime::JsRuntime;
use rusty_v8 as v8;

#[derive(Debug)]
pub struct Timeout {
    pub id: usize,
    pub handle: String,
    pub delay: u64,
    pub args: Vec<v8::Global<v8::Value>>,
    pub repeat: bool,
}

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // A local object that we'll attach all methods to it.
    let target = v8::Object::new(scope);
    set_function_to(scope, target, "createTimeout", create_timeout);
    set_function_to(scope, target, "removeTimeout", remove_timeout);
    // Return it as a global reference.
    v8::Global::new(scope, target)
}

/// Creates a new timeout instance and registers it to the event-loop.
fn create_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get timer's ID.
    let id = args.get(0).int32_value(scope).unwrap() as usize;

    // Get timer's timeout callback.
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = v8::Global::new(scope, callback);

    // Get timer's delay.
    let delay = args.get(2).int32_value(scope).unwrap() as u64;

    // Converting the params argument (Array<Local<Value>>) to a Rust vector.
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

    // Check if this is a recurring timeout.
    let repeat = args.get(4).to_rust_string_lossy(scope).as_str() == "true";

    // Create a new async handle from the callback.
    let handle = JsRuntime::ev_set_handle(scope, AsyncHandle::Callback(callback));

    let timeout = Timeout {
        id,
        handle,
        delay,
        args: params,
        repeat,
    };

    JsRuntime::ev_set_timeout(scope, timeout);

    // Return timeout's ID.
    rv.set(v8::Number::new(scope, id as f64).into());
}

/// Removes a timeout from the event-loop.
fn remove_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the timeout's ID and call the remove method.
    let id = args.get(0).int32_value(scope).unwrap() as usize;
    JsRuntime::ev_unset_timeout(scope, id);
}
