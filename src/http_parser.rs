use crate::bindings::set_constant_to;
use crate::bindings::set_exception_code;
use crate::bindings::set_function_to;
use anyhow::bail;
use anyhow::Result;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "parseRequest", parse_incoming_request);
    set_function_to(scope, target, "parseResponse", parse_incoming_response);
    set_function_to(scope, target, "parseChunks", parse_body_chunks);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Parses an HTTP request received from a client.
fn parse_incoming_request(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the HTTP (partial) request as ArrayBuffer.
    let http_request: v8::Local<v8::ArrayBufferView> = args.get(0).try_into().unwrap();
    let mut data = vec![0; http_request.byte_length()];

    http_request.copy_contents(&mut data);

    // Try parse the HTTP request bytes.
    let mut request_headers = [httparse::EMPTY_HEADER; 32];
    let mut request = httparse::Request::new(&mut request_headers);

    let status = match request.parse(&data) {
        Ok(status) => status,
        Err(e) => {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            set_exception_code(scope, exception, &e.into());
            scope.throw_exception(exception);
            return;
        }
    };

    // Check if the HTTP request is still incomplete.
    if status.is_partial() {
        rv.set(v8::null(scope).into());
        return;
    }

    let method = request.method.unwrap_or_default().to_ascii_uppercase();
    let method = v8::String::new(scope, &method).unwrap();

    let path = request.path.unwrap_or("/");
    let path = v8::String::new(scope, path).unwrap();

    let version = request.version.unwrap_or_default();
    let version = v8::Integer::new(scope, version as i32);

    let headers = request
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
    let marker = status.unwrap();
    let marker = v8::Integer::new(scope, marker as i32);

    // Build the v8 result object.
    let target = v8::Object::new(scope);

    set_constant_to(scope, target, "method", method.into());
    set_constant_to(scope, target, "path", path.into());
    set_constant_to(scope, target, "version", version.into());
    set_constant_to(scope, target, "headers", headers.into());
    set_constant_to(scope, target, "marker", marker.into());

    rv.set(target.into());
}

/// Parses an HTTP response received from a server.
fn parse_incoming_response(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the HTTP (partial) response as ArrayBuffer.
    let http_response: v8::Local<v8::ArrayBufferView> = args.get(0).try_into().unwrap();
    let mut data = vec![0; http_response.byte_length()];

    http_response.copy_contents(&mut data);

    // Try parse the HTTP response bytes.
    let mut response_headers = [httparse::EMPTY_HEADER; 32];
    let mut response = httparse::Response::new(&mut response_headers);

    let status = match response.parse(&data) {
        Ok(status) => status,
        Err(e) => {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            set_exception_code(scope, exception, &e.into());
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
    let marker = status.unwrap();
    let marker = v8::Integer::new(scope, marker as i32);

    // Build the v8 result object.
    let target = v8::Object::new(scope);

    set_constant_to(scope, target, "statusCode", status_code.into());
    set_constant_to(scope, target, "headers", headers.into());
    set_constant_to(scope, target, "marker", marker.into());

    rv.set(target.into());
}

/// Gets available chunks from a streaming HTTP message.
fn parse_body_chunks(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the (current) HTTP body as ArrayBuffer.
    let buffer: v8::Local<v8::ArrayBufferView> = args.get(0).try_into().unwrap();

    let mut data = vec![0; buffer.byte_length()];
    buffer.copy_contents(&mut data);

    // Get all available chunks.
    let (chunks, position, received_last_chunk) = match get_available_chunks(&mut data) {
        Ok(values) => values,
        Err(e) => {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            set_exception_code(scope, exception, &e.into());
            scope.throw_exception(exception);
            return;
        }
    };

    // Provided bytes are incomplete for processing.
    if position == 0 {
        rv.set(v8::null(scope).into());
        return;
    }

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

type RawChunk = Vec<u8>;

/// Extracts available chunks from a buffer.
fn get_available_chunks(buffer: &mut Vec<u8>) -> Result<(Vec<Vec<u8>>, usize, bool)> {
    // Initialize chunk parser.
    let mut parser = ChunkParser::new();

    let mut chunks: Vec<RawChunk> = Vec::new();
    let mut cursor_position = 0;
    let mut received_last_chunk = false;

    // Loop over the buffer until all available chunks have been extracted.
    loop {
        // Parse the buffer as a chunk size and exit the loop if incomplete.
        let remaining = match parser.block_parse(buffer) {
            (done, _) if !done => break,
            (_done, remaining) => remaining,
        };

        let chunk = match parser.finish() {
            Some(chunk) => chunk,
            None => bail!("Couldn't process HTTP chunk."),
        };

        // Calculate how many bytes when consumed for the chunk.
        let consumed = buffer.len() - remaining;

        cursor_position += consumed;

        // If this is the last chunk, set a flag and exit the loop.
        if chunk.size == 0 {
            received_last_chunk = true;
            parser.clear();
            buffer.drain(..consumed);
            break;
        }

        // Append current chunk to chunks vector.
        chunks.push(chunk.body.clone());

        buffer.drain(..consumed);
        parser.clear();
    }

    Ok((chunks, cursor_position, received_last_chunk))
}

/* --------------------------------------------------------------------------------------*/
// The following code is copied from the `streaming_httparse` crate.                      /
// A fast and simple to use HTTP-Parsing crate (https://crates.io/crates/stream-httparse) /
/* --------------------------------------------------------------------------------------*/

/// A single HTTP-Chunk used for sending data with `Transfer-Encoding: Chunked`.
#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub size: usize,
    pub body: Vec<u8>,
}

impl Chunk {
    /// Creates a new chunk with the given data as its state.
    pub fn new(size: usize, data: Vec<u8>) -> Self {
        Self { size, body: data }
    }
}

enum ParseState {
    Size,
    Content(usize),
}

/// A single ChunkParser instance used to parse multiple chunks one after the other.
struct ChunkParser {
    state: ParseState,
    head: Vec<u8>,
    body: Vec<u8>,
}

const MAX_CHUNK_SIZE: usize = 64 * 2usize.pow(20); // 64 MiB.

impl ChunkParser {
    /// Creates a new empty instance of the ChunkParser that is ready to start parsing.
    pub fn new() -> ChunkParser {
        Self {
            state: ParseState::Size,
            head: Vec::with_capacity(16),
            body: Vec::new(),
        }
    }

    /// Clears and resets the internal state.
    pub fn clear(&mut self) {
        // Clear the internal buffer
        self.head.clear();
        self.body.clear();
        // Reset the internal state
        self.state = ParseState::Size;
    }

    /// Parses and handles each individual byte.
    fn parse_size(&mut self) -> Option<usize> {
        match self.head.last() {
            Some(byte) if *byte != b'\n' => return None,
            None => return None,
            _ => {}
        };
        self.head.pop();
        self.head.pop();
        let head_str = match std::str::from_utf8(&self.head) {
            Ok(t) => t,
            Err(_) => {
                return None;
            }
        };
        let result = match usize::from_str_radix(head_str, 16) {
            Ok(n) => n,
            Err(_) => {
                return None;
            }
        };
        // Safety check to prevent large chunk sizes from allocating too much memory.
        if result > MAX_CHUNK_SIZE {
            return None;
        }
        Some(result)
    }

    /// Parses the given block of data, returns the size it parsed as well
    /// as if it is done with parsing.
    ///
    /// Returns:
    /// * If it is done and the `finish` function should be called.
    /// * The amount of data that is still left in the Buffer (at the end).
    pub fn block_parse(&mut self, data: &[u8]) -> (bool, usize) {
        match self.state {
            ParseState::Size => {
                for (index, tmp) in data.iter().enumerate() {
                    self.head.push(*tmp);
                    if let Some(n_size) = self.parse_size() {
                        let n_state = ParseState::Content(n_size);
                        self.state = n_state;
                        self.body.reserve(n_size);
                        return self.block_parse(&data[index + 1..]);
                    }
                }
                (false, 0)
            }
            ParseState::Content(size) => {
                let body_length = self.body.len();
                let left_to_read = size - body_length;
                let data_length = data.len();
                let read_size = std::cmp::min(left_to_read, data_length);
                self.body.extend_from_slice(&data[..read_size]);
                (
                    self.body.len() >= size,
                    data_length.saturating_sub(read_size + 2),
                )
            }
        }
    }

    /// Finishes the parsing and returns the finished chunk.
    pub fn finish(&mut self) -> Option<Chunk> {
        let size = match self.state {
            ParseState::Size => return None,
            ParseState::Content(s) => s,
        };
        let body = std::mem::take(&mut self.body);
        Some(Chunk::new(size, body))
    }
}
