use crate::bindings::get_internal_ref;
use crate::bindings::set_function_to;
use crate::bindings::throw_exception;
use crabuv::tty::Mode;
use crabuv::tty::TtyHandle;

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "isTTY", is_tty);
    set_function_to(scope, target, "setRawMode", set_raw_mode);

    v8::Global::new(scope, target)
}

/// Sets the TTY to raw or normal mode based on the provided mode argument.
fn set_raw_mode(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get the tty wrapper object.
    let tty = args.get(0).to_object(scope).unwrap();
    let tty = get_internal_ref::<TtyHandle>(scope, tty, 0);

    let mode = args.get(1).to_boolean(scope);
    let mode = if mode.is_true() {
        Mode::Raw
    } else {
        Mode::Normal
    };

    // Try to set the mode of the provided TTY. If it fails,
    // throw an exception on the JavaScript side.
    if let Err(e) = tty.set_mode(mode) {
        throw_exception(scope, &e);
    };
}

/// Queries if the provided file descriptor is a TTY.
#[cfg(unix)]
fn is_tty(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    use std::os::fd::BorrowedFd;
    // Get the provided file descriptor.
    let fd = args.get(0).int32_value(scope).unwrap();
    let fd = unsafe { BorrowedFd::borrow_raw(fd.into()) };
    let is_tty = rustix::termios::isatty(fd);

    rv.set(v8::Boolean::new(scope, is_tty).into());
}

/// Queries if the provided file descriptor is a TTY.
#[cfg(windows)]
fn is_tty(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // TODO: implement a version for windows..
    panic!("not yet implemented");
}
