use crate::bindings::set_function_to;
use rusty_v8 as v8;
use std::io;
use std::io::Write;

pub fn initialize<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Global<v8::Object> {
    // A local object that we'll attach all methods to it.
    let target = v8::Object::new(scope);
    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "writeError", write_error);
    // Return it as a global reference.
    v8::Global::new(scope, target)
}

fn write(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
    // Transform contents to bytes.
    let contents = args.get(0).to_rust_string_lossy(scope);
    let contents = contents.as_bytes();
    // Flush bytes to stdout.
    io::stdout().write_all(&contents).unwrap();
    io::stdout().flush().unwrap();
}

fn write_error(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Transform contents to bytes.
    let contents = args.get(0).to_rust_string_lossy(scope);
    let contents = contents.as_bytes();
    // Flush bytes to stderr.
    io::stderr().write_all(&contents).unwrap();
    io::stderr().flush().unwrap();
}
