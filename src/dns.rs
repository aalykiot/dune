use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::event_loop::LoopHandle;
use crate::event_loop::TaskResult;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::Result;
use dns_lookup::lookup_host;
use std::net::IpAddr;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "lookup", dns_lookup);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Describes what will run after the async dns_lookup completes.
struct DnsLookupFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for DnsLookupFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // Extract the result.
        let result = self.maybe_result.take().unwrap();

        // Handle when something goes wrong on the DNS lookup.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Otherwise, get the result and deserialize it.
        let result = result.unwrap();
        let result: Vec<(String, String)> = bincode::deserialize(&result).unwrap();

        let ips: Vec<v8::Local<v8::Value>> = result
            .iter()
            .map(|(address, family)| {
                // Create new v8 handles.
                let ip = v8::Object::new(scope);
                let address = v8::String::new(scope, address).unwrap().into();
                let family = v8::String::new(scope, family).unwrap().into();

                // Set properties to IP object.
                set_property_to(scope, ip, "address", address);
                set_property_to(scope, ip, "family", family);

                ip.into()
            })
            .collect();

        let ips_array = v8::Array::new_with_elements(scope, &ips);

        self.promise
            .open(scope)
            .resolve(scope, ips_array.into())
            .unwrap();
    }
}

/// Resolves a host name into the first found A (IPv4) or AAAA (IPv6) record.
fn dns_lookup(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get host from the arguments provided.
    let host = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match dns_lookup_op(&host) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = DnsLookupFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Pure rust implementation of a DNS lookup.
fn dns_lookup_op(hostname: &str) -> Result<Vec<(String, String)>> {
    Ok(lookup_host(hostname)?
        .iter()
        .map(|ip| match ip {
            IpAddr::V4(address) => (address.to_string(), "IPv4".into()),
            IpAddr::V6(address) => (address.to_string(), "IPv6".into()),
        })
        .collect())
}
