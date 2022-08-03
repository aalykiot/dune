use crate::bindings::set_function_to;
use crate::bindings::throw_exception;
use crate::event_loop::TaskResult;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "read", read);
    set_function_to(scope, target, "readSync", read_sync);
    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "writeSync", write_sync);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Describes what will run after the async read_file_op completes.
struct FsReadFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsReadFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let result = self.maybe_result.take().unwrap();

        // Handle when something goes wrong with reading.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Otherwise, resolve the promise passing the result.
        let result = result.unwrap();

        // Decompress message-pack binary into actual rust types.
        let (n, mut buffer): (usize, Vec<u8>) = rmp_serde::from_slice(&result).unwrap();

        // We reached the end of the file.
        if n == 0 {
            // Create an empty ArrayBuffer and return it to JavaScript.
            let store = v8::ArrayBuffer::new_backing_store(scope, 0).make_shared();
            let bytes = v8::ArrayBuffer::with_backing_store(scope, &store);

            self.promise.open(scope).resolve(scope, bytes.into());
            return;
        }

        // Resize buffer given bytes read.
        buffer.resize(n, 0);

        // Create ArrayBuffer's backing store from Vec<u8>.
        let store = buffer.into_boxed_slice();
        let store = v8::ArrayBuffer::new_backing_store_from_boxed_slice(store).make_shared();

        // Initialize ArrayBuffer.
        let bytes = v8::ArrayBuffer::with_backing_store(scope, &store);

        self.promise
            .open(scope)
            .resolve(scope, bytes.into())
            .unwrap();
    }
}

/// Reads asynchronously a chunk of a file (as bytes).
fn read(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get source path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Get chunk size and byte offset.
    let size = args.get(1).to_integer(scope).unwrap().value();
    let offset = args.get(2).to_integer(scope).unwrap().value();

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match read_file_op(path, size, offset) {
        Ok(result) => Some(Ok(rmp_serde::to_vec(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsReadFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
            state.check_and_interrupt();
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Reads a chunk of a file (as bytes).
fn read_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get source path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Get chunk size and byte offset.
    let size = args.get(1).to_integer(scope).unwrap().value();
    let offset = args.get(2).to_integer(scope).unwrap().value();

    match read_file_op(path, size, offset) {
        Ok((n, mut buffer)) => {
            // We reached the end of the file.
            if n == 0 {
                // Create an empty ArrayBuffer and return it to JavaScript.
                let store = v8::ArrayBuffer::new_backing_store(scope, 0).make_shared();
                let bytes = v8::ArrayBuffer::with_backing_store(scope, &store);

                rv.set(bytes.into());
                return;
            }

            // Resize buffer given bytes read.
            buffer.resize(n, 0);

            // Create ArrayBuffer's backing store from Vec<u8>.
            let store = buffer.into_boxed_slice();
            let store = v8::ArrayBuffer::new_backing_store_from_boxed_slice(store).make_shared();

            // Initialize ArrayBuffer.
            let bytes = v8::ArrayBuffer::with_backing_store(scope, &store);

            rv.set(bytes.into());
        }
        Err(e) => {
            throw_exception(scope, &e.to_string());
        }
    }
}

/// Describes what will run after the async write_file_op completes.
struct FsWriteFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsWriteFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // If the `task_result` is None it means everything is fine.
        if self.maybe_result.is_none() {
            let undefined = v8::undefined(scope);
            self.promise
                .open(scope)
                .resolve(scope, undefined.into())
                .unwrap();
            return;
        }

        // Something went wrong.
        let result = self.maybe_result.take().unwrap();

        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Note: The result from the `write_file_op` should be None or some Error.
        // Based on that assumption we should never reach this part of the
        // function thus we use the unreachable! macro.

        unreachable!();
    }
}

// Writes asynchronously contents to a file.
fn write(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get file path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Get data as ArrayBuffer.
    let data: v8::Local<v8::ArrayBufferView> = args.get(1).try_into().unwrap();

    let mut buffer = vec![0; data.byte_length()];
    data.copy_contents(&mut buffer);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match write_file_op(path, &buffer) {
        Ok(_) => None,
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |maybe_result: TaskResult| {
            // Get a mut reference to the runtime's state.
            let mut state = state_rc.borrow_mut();
            let fs_write_handle = FsWriteFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(fs_write_handle));
            state.check_and_interrupt();
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Writes contents to a file.
fn write_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get file path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Get data as ArrayBuffer.
    let data: v8::Local<v8::ArrayBufferView> = args.get(1).try_into().unwrap();

    let mut buffer = vec![0; data.byte_length()];
    data.copy_contents(&mut buffer);

    if let Err(e) = write_file_op(path, &buffer) {
        throw_exception(scope, &e.to_string());
    }
}

/// Pure rust implementation of reading a chunk from a file.
fn read_file_op<P: AsRef<Path>>(path: P, size: i64, offset: i64) -> Result<(usize, Vec<u8>)> {
    // Try open requested file.
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => bail!(e),
    };

    // Move file cursor to requested position.
    if let Err(e) = file.seek(SeekFrom::Start(offset as u64)) {
        bail!(e);
    }

    let mut buffer = vec![0; size as usize];

    // Read at most `size` bytes from the file.
    match file.take(size as u64).read(&mut buffer) {
        Ok(n) => Ok((n, buffer)),
        Err(e) => bail!(e),
    }
}

/// Pure rust implementation of writing bytes to a file.
fn write_file_op<P: AsRef<Path>>(path: P, buffer: &[u8]) -> Result<()> {
    // Try open file.
    let mut file = match fs::File::create(path) {
        Ok(file) => file,
        Err(e) => bail!(e),
    };

    // Write buffer to file.
    if let Err(e) = file.write_all(buffer) {
        bail!(e);
    }

    Ok(())
}
