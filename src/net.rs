use crate::bindings::get_internal_ref;
use crate::bindings::set_exception_code;
use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::wrap_gc_dropped;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::Result;
use crabuv::tcp_listener::TcpListenerHandle;
use crabuv::tcp_stream::TcpStreamHandle;
use crabuv::LoopHandle;
use std::net::IpAddr;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "connect", connect);
    set_function_to(scope, target, "readStart", read_start);
    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "listen", listen);
    set_function_to(scope, target, "shutdown", shutdown);
    set_function_to(scope, target, "close", close);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

struct TcpConnectFuture {
    stream: Result<TcpStreamHandle>,
    promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for TcpConnectFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        match self.stream.as_ref() {
            Ok(stream) => {
                // Extract info from the TcpSocketInfo.
                let socket = stream.info.as_ref().as_ref().unwrap();
                let host_port = socket.host.port();
                let host_address = socket.host.ip().to_string();
                let host_family = match socket.host.ip() {
                    IpAddr::V4(_) => "IPv4",
                    IpAddr::V6(_) => "IPv6",
                };

                let host = v8::Object::new(scope);

                // Host IP attributes.
                let port = v8::Integer::new(scope, host_port as i32);
                let family = v8::String::new(scope, host_family).unwrap();
                let address = v8::String::new(scope, &host_address).unwrap();

                set_property_to(scope, host, "port", port.into());
                set_property_to(scope, host, "family", family.into());
                set_property_to(scope, host, "address", address.into());

                // Remote IP attributes.
                let port = socket.remote.port();
                let address = socket.remote.ip().to_string();

                let remote = v8::Object::new(scope);

                let port = v8::Integer::new(scope, port as i32);
                let address = v8::String::new(scope, &address).unwrap();

                set_property_to(scope, remote, "port", port.into());
                set_property_to(scope, remote, "address", address.into());

                // Create a JavaScript socket info object.
                let socket = v8::Object::new(scope);
                let stream_wrap = wrap_gc_dropped(scope, stream.clone());

                set_property_to(scope, socket, "stream", stream_wrap.into());
                set_property_to(scope, socket, "host", host.into());
                set_property_to(scope, socket, "remote", remote.into());

                self.promise
                    .open(scope)
                    .resolve(scope, socket.into())
                    .unwrap();
            }
            Err(e) => {
                // Reject the promise.
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                let exception = v8::Exception::error(scope, message);
                set_exception_code(scope, exception, e);
                self.promise.open(scope).reject(scope, exception).unwrap();
            }
        }
    }
}

/// Creates a new TCP stream and issue a non-blocking connect.
fn connect(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get IP and PORT from arguments.
    let ip = args.get(0).to_rust_string_lossy(scope);
    let port = args.get(1).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let on_connection = {
        let state_rc = state_rc.clone();
        let promise = v8::Global::new(scope, promise_resolver);
        move |stream: Result<TcpStreamHandle>| {
            let mut state = state_rc.borrow_mut();
            let promise = promise.clone();
            let future = TcpConnectFuture { stream, promise };

            state.pending_futures.push(Box::new(future));
        }
    };

    // Try open a TCP stream with the remote host.
    let address = format!("{ip}:{port}").parse().unwrap();
    let connection = state.handle.tcp_connect(address, on_connection);

    // Check if the tcp_connect failed early.
    if let Err(e) = connection {
        // Drop state to avoid panics.
        drop(state);
        // Create the JavaScript error.
        let message = v8::String::new(scope, &e.to_string()).unwrap();
        let exception = v8::Exception::error(scope, message);
        set_exception_code(scope, exception, &e);
        promise_resolver.reject(scope, exception).unwrap();
        return;
    }

    rv.set(promise.into());
}

struct ReadStartFuture {
    data: Result<Vec<u8>>,
    on_read: Rc<v8::Global<v8::Function>>,
}

impl JsFuture for ReadStartFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        // Create the v8 value for the data parameter.
        let data_value: v8::Local<v8::Value> = match self.data.as_mut() {
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
        let error_value: v8::Local<v8::Value> = match self.data.as_mut() {
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

        on_read.call(scope, undefined, &[error_value, data_value]);
    }
}

/// Starts reading from an open TCP socket.
fn read_start(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get the stream wrapper object.
    let stream = args.get(0).to_object(scope).unwrap();
    let stream = get_internal_ref::<TcpStreamHandle>(scope, stream, 0);

    // Get reading callback.
    let on_read = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let on_read = Rc::new(v8::Global::new(scope, on_read));

    let state_rc = JsRuntime::state(scope);

    // Start reading from the ipen TCP stream.
    stream.set_read_callback({
        move |_: TcpStreamHandle, data: Result<Vec<u8>>| {
            let mut state = state_rc.borrow_mut();
            let on_read = Rc::clone(&on_read);
            let future = ReadStartFuture { data, on_read };

            state.pending_futures.push(Box::new(future));
        }
    });
}

struct TcpWriteFuture {
    result: Result<usize>,
    promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for TcpWriteFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        match self.result.as_ref() {
            Ok(bytes) => {
                // Create a v8 value from the usize.
                let bytes = *bytes as i32;
                let bytes = v8::Integer::new(scope, bytes);

                self.promise
                    .open(scope)
                    .resolve(scope, bytes.into())
                    .unwrap();
            }
            Err(e) => {
                // Reject the promise.
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                let exception = v8::Exception::error(scope, message);
                set_exception_code(scope, exception, e);
                self.promise.open(scope).reject(scope, exception).unwrap();
            }
        }
    }
}

fn write(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get the stream wrapper object.
    let stream = args.get(0).to_object(scope).unwrap();
    let stream = get_internal_ref::<TcpStreamHandle>(scope, stream, 0);

    let data: v8::Local<v8::ArrayBufferView> = args.get(1).try_into().unwrap();
    let store = data.get_backing_store().unwrap();
    let store_length = store.byte_length();

    let buffer = unsafe {
        // For performance, we avoid copying and instead (unsafely) create a u8 slice
        // directly from the backing store’s raw c_void pointer.
        let store_ptr = store.data().unwrap();
        std::slice::from_raw_parts(store_ptr.as_ptr() as *const u8, store_length)
    };

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);

    let on_write = {
        let promise = v8::Global::new(scope, promise_resolver);
        move |_: TcpStreamHandle, result: Result<usize>| {
            let mut state = state_rc.borrow_mut();
            let promise = promise.clone();
            let future = TcpWriteFuture { result, promise };

            state.pending_futures.push(Box::new(future));
        }
    };

    stream.write(buffer.to_vec(), on_write);
    rv.set(promise.into());
}

struct TcpListenFuture {
    stream: Result<TcpStreamHandle>,
    on_connection: Rc<v8::Global<v8::Function>>,
}

impl JsFuture for TcpListenFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        // Create the v8 value for the data parameter.
        let socket_value: v8::Local<v8::Value> = match self.stream.as_mut() {
            Ok(stream) => {
                // Extract info from the TcpSocketInfo.
                let socket = stream.info.as_ref().as_ref().unwrap();
                let address = socket.remote.ip().to_string();
                let port = socket.remote.port();

                let stream = wrap_gc_dropped(scope, stream.clone());

                let socket_info = v8::Object::new(scope);
                let address = v8::String::new(scope, &address).unwrap();
                let port = v8::Integer::new(scope, port as i32);

                set_property_to(scope, socket_info, "stream", stream.into());
                set_property_to(scope, socket_info, "remoteAddress", address.into());
                set_property_to(scope, socket_info, "remotePort", port.into());

                socket_info.into()
            }
            Err(_) => v8::null(scope).into(),
        };

        // Create the v8 value for the error parameter.
        let error_value: v8::Local<v8::Value> = match self.stream.as_mut() {
            Err(e) => {
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                v8::Exception::error(scope, message)
            }
            Ok(_) => v8::null(scope).into(),
        };

        v8::tc_scope!(let tc_scope, scope);

        // Get access to the on_connection callback.
        let on_connection = v8::Local::new(tc_scope, (*self.on_connection).clone());
        let undefined = v8::undefined(tc_scope).into();

        on_connection.call(tc_scope, undefined, &[error_value, socket_value]);

        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let state = JsRuntime::state(tc_scope);
            state.borrow_mut().exceptions.capture_exception(exception);
        }
    }
}

/// Starts listening for incoming connections.
fn listen(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get INTERFACE and PORT from arguments.
    let interface = args.get(0).to_rust_string_lossy(scope);
    let port = args.get(1).to_rust_string_lossy(scope);
    let address = format!("{interface}:{port}").parse().unwrap();

    // Get on_connection callback.
    let on_connection = v8::Local::<v8::Function>::try_from(args.get(2)).unwrap();
    let on_connection = Rc::new(v8::Global::new(scope, on_connection));

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // Start listening for incoming tcp connections.
    let server = state.handle.tcp_listen(address, {
        let state_rc = state_rc.clone();
        move |_: TcpListenerHandle, stream: Result<TcpStreamHandle>| {
            let mut state = state_rc.borrow_mut();
            let on_connection = Rc::clone(&on_connection);
            let future = TcpListenFuture {
                stream,
                on_connection,
            };

            state.pending_futures.push(Box::new(future));
        }
    });

    // Check mostly for address bind errors.
    if let Err(e) = server {
        let message = v8::String::new(scope, &e.to_string()).unwrap();
        let exception = v8::Exception::error(scope, message);
        set_exception_code(scope, exception, &e);
        scope.throw_exception(exception);
        return;
    }

    let server = wrap_gc_dropped(scope, server);

    let host = v8::Object::new(scope);
    let port = args.get(1).to_int32(scope).unwrap();
    let address = args.get(0).to_string(scope).unwrap();

    let family = match interface.parse().unwrap() {
        IpAddr::V4(_) => v8::String::new(scope, "IPv4").unwrap(),
        IpAddr::V6(_) => v8::String::new(scope, "IPv6").unwrap(),
    };

    set_property_to(scope, host, "port", port.into());
    set_property_to(scope, host, "family", family.into());
    set_property_to(scope, host, "address", address.into());

    // The actual object we'll return.
    let ret_value = v8::Object::new(scope);

    set_property_to(scope, ret_value, "server", server.into());
    set_property_to(scope, ret_value, "host", host.into());

    rv.set(ret_value.into());
}

struct TcpShutdownFuture {
    promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for TcpShutdownFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        let undefined = v8::undefined(scope);
        self.promise
            .open(scope)
            .resolve(scope, undefined.into())
            .unwrap();
    }
}

// Half-closes the TCP stream.
fn shutdown(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the stream wrapper object.
    let stream = args.get(0).to_object(scope).unwrap();
    let stream = get_internal_ref::<TcpStreamHandle>(scope, stream, 0);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);

    stream.shutdown({
        let promise = v8::Global::new(scope, promise_resolver);
        move |_: LoopHandle| {
            let promise = promise.clone();
            let mut state = state_rc.borrow_mut();
            let future = TcpShutdownFuture { promise };
            state.pending_futures.push(Box::new(future));
        }
    });

    rv.set(promise.into());
}

struct TcpCloseFuture {
    promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for TcpCloseFuture {
    fn run(&mut self, scope: &mut v8::PinScope) {
        let undefined = v8::undefined(scope);
        self.promise
            .open(scope)
            .resolve(scope, undefined.into())
            .unwrap();
    }
}

/// Closes the TCP socket.
fn close(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get the stream wrapper object.
    let stream = args.get(0).to_object(scope).unwrap();
    let stream = get_internal_ref::<TcpStreamHandle>(scope, stream, 0);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);

    stream.close({
        let state_rc = state_rc.clone();
        let promise = v8::Global::new(scope, promise_resolver);
        move |_: LoopHandle| {
            let mut state = state_rc.borrow_mut();
            let promise = promise.clone();
            let future = TcpCloseFuture { promise };
            state.pending_futures.push(Box::new(future));
        }
    });

    rv.set(promise.into());
}
