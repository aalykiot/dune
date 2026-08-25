use crate::bindings::get_internal_ref;
use crate::bindings::set_exception_code;
use crate::bindings::set_function_to;
use crate::bindings::throw_exception;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::Result;
use crabuv::tty::Mode;
use crabuv::tty::TtyHandle;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "isTTY", is_tty);
    set_function_to(scope, target, "setRawMode", set_raw_mode);
    set_function_to(scope, target, "readStart", read_start);

    v8::Global::new(scope, target)
}

struct TTYReadFuture {
    data: Result<Vec<u8>>,
    on_read: Rc<v8::Global<v8::Function>>,
}

impl JsFuture for TTYReadFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        // Create the v8 value for the data parameter.
        let data: v8::Local<v8::Value> = match self.data.as_mut() {
            Ok(data) => {
                // Create ArrayBuffer's backing store from Vec<u8>.
                let store = data.clone().into_boxed_slice();
                let store = v8::ArrayBuffer::new_backing_store_from_boxed_slice(store);
                let store = store.make_shared();

                // Initialize ArrayBuffer.
                let bytes = v8::ArrayBuffer::with_backing_store(scope, &store);
                bytes.into()
            }
            Err(_) => v8::null(scope).into(),
        };

        // Create the v8 value for the error parameter.
        let error: v8::Local<v8::Value> = match self.data.as_mut() {
            Ok(_) => v8::null(scope).into(),
            Err(e) => {
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                let exception = v8::Exception::error(scope, message);
                set_exception_code(scope, exception, e);
                exception
            }
        };

        // Get access to the on_read callback.
        let on_read = v8::Local::new(scope, (*self.on_read).clone());
        let undefined = v8::undefined(scope).into();

        on_read.call(scope, undefined, &[error, data]);
    }
}

/// Starts reading from a TTY stream.
fn read_start(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get the tty wrapper object.
    let tty = args.get(0).to_object(scope).unwrap();
    let tty = get_internal_ref::<TtyHandle>(scope, tty, 0);

    // Get reading callback.
    let on_read = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let on_read = Rc::new(v8::Global::new(scope, on_read));

    let state_rc = JsRuntime::state(scope);

    // Start reading from the TTY stream.
    tty.start_reading({
        move |_: TtyHandle, data: Result<Vec<u8>>| {
            let mut state = state_rc.borrow_mut();
            let on_read = Rc::clone(&on_read);
            let future = TTYReadFuture { data, on_read };

            state.pending_futures.push(Box::new(future));
        }
    });
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
