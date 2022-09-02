use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::event_loop::Index;
use crate::event_loop::LoopHandle;
use crate::event_loop::TcpSocketInfo;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::Result;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "connect", connect);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

struct TcpConnectFuture {
    sock: Result<TcpSocketInfo>,
    promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for TcpConnectFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        match self.sock.as_ref() {
            Ok(sock) => {
                // Extract info from the TcpSocketInfo.
                let address = sock.remote.ip().to_string();
                let port = sock.remote.port();

                // Create a JavaScript socket info object.
                let socket_info = v8::Object::new(scope);

                let rid = v8::Number::new(scope, sock.id as f64);
                let address = v8::String::new(scope, &address).unwrap();
                let port = v8::Number::new(scope, port as f64);

                set_property_to(scope, socket_info, "rid", rid.into());
                set_property_to(scope, socket_info, "remoteAddress", address.into());
                set_property_to(scope, socket_info, "remotePort", port.into());

                self.promise
                    .open(scope)
                    .resolve(scope, socket_info.into())
                    .unwrap();
            }
            Err(e) => {
                // Reject the promise.
                let message = v8::String::new(scope, &e.to_string()).unwrap();
                let exception = v8::Exception::error(scope, message);

                self.promise.open(scope).reject(scope, exception).unwrap();
            }
        }
    }
}

/// Creates a new TCP stream and issue a non-blocking connect.
fn connect(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get IP and PORT from arguments.
    let ip = args.get(0).to_rust_string_lossy(scope);
    let port = args.get(1).to_rust_string_lossy(scope);
    let address = format!("{}:{}", ip, port);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let on_connection = {
        let state_rc = state_rc.clone();
        let promise = v8::Global::new(scope, promise_resolver);
        move |_: LoopHandle, index: Index, sock: Result<TcpSocketInfo>| {
            let mut state = state_rc.borrow_mut();
            // If connection did't happen, remove the resource.
            if sock.is_err() {
                state.handle.tcp_close(index, |_: LoopHandle| {});
            }
            // Create a new JsFuture.
            let future = TcpConnectFuture { sock, promise };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Try open a TCP stream with the remote host.
    let connect = state.handle.tcp_connect(&address, on_connection);

    // Check if the tcp_connect failed early.
    if let Err(e) = connect {
        // Create the JavaScript error.
        let message = v8::String::new(scope, &e.to_string()).unwrap();
        let exception = v8::Exception::error(scope, message);

        promise_resolver.reject(scope, exception).unwrap();
        return;
    }

    rv.set(promise.into());
}
