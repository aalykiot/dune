use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "parseResponse", parse_response);
    set_function_to(scope, target, "parseChunk", parse_chunk);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Parses the HTTP response for statusCode, method and headers.
fn parse_response(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the HTTP (partial) response as ArrayBuffer.
    let http_response: v8::Local<v8::ArrayBufferView> = args.get(0).try_into().unwrap();

    let mut data = vec![0; http_response.byte_length()];
    http_response.copy_contents(&mut data);

    // Try parse the HTTP response bytes.
    let mut response_headers = [httparse::EMPTY_HEADER; 16];
    let mut response = httparse::Response::new(&mut response_headers);

    let status = match response.parse(&data) {
        Ok(status) => status,
        Err(e) => {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            scope.throw_exception(exception);
            return;
        }
    };

    // Check if the HTTP response is still incomplete.
    if status.is_partial() {
        rv.set(v8::null(scope).into());
        return;
    }

    let status_code = response.code.unwrap_or_default();
    let status_code = v8::Integer::new(scope, status_code as i32);

    let headers = response
        .headers
        .iter()
        .map(|h| {
            let name = h.name.to_owned().to_lowercase();
            let value = String::from_utf8(h.value.to_vec()).unwrap();
            (name, value)
        })
        .fold(v8::Object::new(scope), |acc, (name, value)| {
            let value = v8::String::new(scope, &value).unwrap();
            set_constant_to(scope, acc, &name, value.into());
            acc
        });

    // Get the position the HTTP body starts.
    let body_at = status.unwrap();
    let body_at = v8::Integer::new(scope, body_at as i32);

    // Build the v8 result object.
    let target = v8::Object::new(scope);
    set_constant_to(scope, target, "statusCode", status_code.into());
    set_constant_to(scope, target, "headers", headers.into());
    set_constant_to(scope, target, "bodyAt", body_at.into());

    rv.set(target.into());
}

/// Parses the next body chunk of the HTTP request/response.
fn parse_chunk(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    todo!()
}
