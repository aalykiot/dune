use crate::v8_utils::set_function_to;

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);
    set_function_to(scope, target, "callConsole", call_console);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Native wrapper that will preserve the original stack.
/// https://github.com/denoland/deno_core/blob/main/core/runtime/bindings.rs#L504-L529
fn call_console(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    assert!(args.length() >= 2);
    assert!(args.get(0).is_function());
    assert!(args.get(1).is_function());

    // Collect arguments that will be passed to the console call.
    let params = (2..args.length()).fold(vec![], |mut params, i| {
        params.push(args.get(i));
        params
    });

    let this = args.this();
    let console_method = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let console_v8_method = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();

    console_v8_method.call(scope, this.into(), &params);
    console_method.call(scope, this.into(), &params);
}
