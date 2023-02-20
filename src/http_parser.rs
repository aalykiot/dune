use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use anyhow::Error;
use anyhow::Result;
use httparse::Status;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "parseHttpResponse", parse_http_response);
    set_function_to(scope, target, "parseHttpChunks", parse_http_chunks);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Parses the HTTP response for statusCode, method and headers.
fn parse_http_response(
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
fn parse_http_chunks(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the (current) HTTP body as ArrayBuffer.
    let response_body: v8::Local<v8::ArrayBufferView> = args.get(0).try_into().unwrap();

    let mut data = vec![0; response_body.byte_length()];
    response_body.copy_contents(&mut data);

    // Get all available chunks.
    let (chunks, position, received_last_chunk) = match get_available_chunks(&mut data) {
        Ok(values) => values,
        Err(e) => {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            scope.throw_exception(exception);
            return;
        }
    };

    // Create a v8 typed-array for each chunk.
    let chunks: Vec<v8::Local<v8::Value>> = chunks
        .iter()
        .map(|chunk| {
            let store = chunk.to_owned().into_boxed_slice();
            let store = v8::ArrayBuffer::new_backing_store_from_boxed_slice(store).make_shared();
            v8::ArrayBuffer::with_backing_store(scope, &store).into()
        })
        .collect();

    // Create a v8 array holding all the chunks.
    let chunks = v8::Array::new_with_elements(scope, &chunks);
    let position = v8::Integer::new(scope, position as i32);
    let done = v8::Boolean::new(scope, received_last_chunk);

    // Build the v8 result object.
    let target = v8::Object::new(scope);
    set_constant_to(scope, target, "chunks", chunks.into());
    set_constant_to(scope, target, "position", position.into());
    set_constant_to(scope, target, "done", done.into());

    rv.set(target.into());
}

const CRLF_LENGTH: usize = 2;

/// Extracts available chunks from a buffer.
fn get_available_chunks(buffer: &mut Vec<u8>) -> Result<(Vec<Vec<u8>>, usize, bool)> {
    let mut chunks = vec![];
    let mut cursor_position = 0;
    let mut received_last_chunk = false;

    // Loop over the buffer until all available chunks have been extracted.
    loop {
        // Parse the buffer as a chunk size and exit the loop if incomplete.
        let status = httparse::parse_chunk_size(&buffer).map_err(|e| Error::msg(e.to_string()))?;
        if let Status::Partial = status {
            break;
        }

        let (chunk_start, chunk_length) = status.unwrap();
        let chunk_end = chunk_start + chunk_length as usize;

        // If this is the last chunk, set a flag and exit the loop.
        if chunk_length == 0 {
            cursor_position += chunk_end + CRLF_LENGTH;
            received_last_chunk = true;
            break;
        }

        // Extract the chunk as a byte vector and update the cursor position.
        let chunk = buffer[chunk_start..chunk_end].to_vec();
        cursor_position += chunk_end + CRLF_LENGTH;
        buffer.drain(0..(chunk_end + CRLF_LENGTH));
        chunks.push(chunk);
    }

    Ok((chunks, cursor_position, received_last_chunk))
}
