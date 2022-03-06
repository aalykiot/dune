use crate::bindings::set_function_to;
use rusty_v8 as v8;

pub struct Timer {
    pub id: usize,
    pub handle: usize,
    pub delay: u64,
    pub args: Vec<v8::Global<v8::Value>>,
    pub repeat: bool,
}

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // A local object that we'll attach all methods to it.
    let target = v8::Object::new(scope);
    set_function_to(scope, target, "createTimeout", create_timeout);
    // Return it as a global reference.
    v8::Global::new(scope, target)
}

// Creates a new timer instance in JavaScript.
fn create_timeout(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get timer's ID.
    let id = args.get(0).int32_value(scope).unwrap() as i32;
    // Get timer's timeout callback.
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = v8::Global::new(scope, callback);
    // Get timer's delay.
    let delay = args.get(2).int32_value(scope).unwrap() as i32;

    // Converting the params argument (Array<Local<Value>>) to a vector.
    let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
        Ok(params) => {
            (0..params.length()).fold(Vec::<v8::Global<v8::Value>>::new(), |mut acc, i| {
                let param = params.get_index(scope, i).unwrap();
                acc.push(v8::Global::new(scope, param));
                acc
            })
        }
        Err(_) => Vec::default(),
    };

    // Is this a recurring timeout ??
    let repeat = args.get(4).to_rust_string_lossy(scope).as_str() == "true";

    println!("DEBUG: {}", id);
    println!("DEBUG: {}", delay);
    println!("DEBUG: {:?}", params);
    println!("DEBUG: {}", repeat);

    // Return timer's ID.
    rv.set(v8::Number::new(scope, id as f64).into());
}