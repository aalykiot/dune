use crate::bindings::set_function_to;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "parse", parse);
    set_function_to(scope, target, "parseChunk", parse_chunk);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Parses the HTTP request for method, headers, etc.
fn parse(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {}

/// Parses the next body chunk of the HTTP request.
fn parse_chunk(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    todo!()
}
