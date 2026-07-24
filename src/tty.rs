use crate::bindings::set_function_to;
use std::io::stderr;
use std::io::stdin;
use std::io::stdout;
use std::io::IsTerminal;

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "isTTY", is_tty);

    v8::Global::new(scope, target)
}

/// Queries if the provided file descriptor is a TTY.
fn is_tty(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get the provided file descriptor.
    let fd = args.get(0).int32_value(scope).unwrap() as u8;
    let is_tty = match fd {
        0 => stdout().is_terminal(),
        1 => stdin().is_terminal(),
        2 => stderr().is_terminal(),
        // Currently there is no way to determine whether a user-provided,
        // non-standard file descriptor is a TTY.
        _ => false,
    };

    rv.set(v8::Boolean::new(scope, is_tty).into());
}
