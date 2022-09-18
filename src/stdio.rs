use crate::bindings::set_function_to;
use crate::bindings::throw_exception;
use std::io;
use std::io::Write;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "writeError", write_error);
    set_function_to(scope, target, "read", read);
    set_function_to(scope, target, "clear", clear);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Writes data to the stdout stream.
fn write(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Convert string to bytes.
    let content = args.get(0).to_rust_string_lossy(scope);
    let content = content.as_bytes();
    // Flush bytes to stdout.
    io::stdout().write_all(content).unwrap();
    io::stdout().flush().unwrap();
}

/// Writes data to the stderr stream.
fn write_error(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Convert string to bytes.
    let content = args.get(0).to_rust_string_lossy(scope);
    let content = content.as_bytes();
    // Flush bytes to stderr.
    io::stderr().write_all(content).unwrap();
    io::stderr().flush().unwrap();
}

/// Reads (synchronously) a string from the stdin.
fn read(scope: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, mut ret: v8::ReturnValue) {
    // Read input from system's stdin stream.
    let mut input = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut input).unwrap();

    // Return data back to JavaScript.
    let input = v8::String::new(scope, input.trim_end()).unwrap();
    ret.set(input.into());
}

/// Clears the terminal if the environment allows it.
fn clear(scope: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    if let Err(e) = clearscreen::clear() {
        throw_exception(scope, &e.to_string());
    }
}
