use crate::bindings::get_internal_ref;
use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::throw_exception;
use crate::event_loop::TaskResult;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::bail;
use anyhow::Result;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;

#[cfg(target_family = "unix")]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

#[cfg(target_family = "windows")]
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "openSync", open_sync);
    set_function_to(scope, target, "close", close);
    set_function_to(scope, target, "closeSync", close_sync);
    set_function_to(scope, target, "read", read);
    set_function_to(scope, target, "readSync", read_sync);
    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "writeSync", write_sync);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Describes what will run after the async open_file_op completes.
struct FsOpenFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsOpenFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let result = self.maybe_result.take().unwrap();

        // Handle when something goes wrong with opening the file.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Otherwise, get the result and deserialize it.
        let result = result.unwrap();

        // Deserialize bytes into a file-descriptor.
        let file_ptr: usize = bincode::deserialize(&result).unwrap();
        let file_wrapper = v8::ObjectTemplate::new(scope);

        // Allocate space for the wrapped Rust type.
        file_wrapper.set_internal_field_count(1);

        let file_wrapper = file_wrapper.new_instance(scope).unwrap();
        let fd = v8::Number::new(scope, file_ptr as f64);

        set_constant_to(scope, file_wrapper, "fd", fd.into());
        set_internal_ref(scope, file_wrapper, 0, Some(file_ptr));

        self.promise
            .open(scope)
            .resolve(scope, file_wrapper.into())
            .unwrap();
    }
}

/// Opens a file asynchronously.
fn open(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get file path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match open_file_op(path) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsOpenFuture {
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

/// Opens a file synchronously.
fn open_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get file path.
    let path = args.get(0).to_rust_string_lossy(scope);

    match open_file_op(path) {
        Ok(file_ptr) => {
            let file = v8::ObjectTemplate::new(scope);

            // Allocate space for the wrapped Rust type.
            file.set_internal_field_count(1);

            let file_wrapper = file.new_instance(scope).unwrap();
            let fd = v8::Number::new(scope, file_ptr as f64);

            set_constant_to(scope, file_wrapper, "fd", fd.into());
            set_internal_ref(scope, file_wrapper, 0, Some(file_ptr));

            rv.set(file_wrapper.into());
        }
        Err(e) => {
            throw_exception(scope, &e.to_string());
        }
    }
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
        let (n, mut buffer): (usize, Vec<u8>) = bincode::deserialize(&result).unwrap();

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

/// Closes a file asynchronously.
fn close(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();
    let file_ptr = get_internal_ref::<Option<usize>>(scope, file_wrap, 0);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    if let Some(ptr) = file_ptr {
        let undefined = v8::undefined(scope);
        let file = get_file_reference(*ptr);

        // Note: By creating a file reference and immediately dropping it will
        // make rust to close the file.
        drop(file);
        set_internal_ref(scope, file_wrap, 0, None::<usize>);

        promise_resolver.resolve(scope, undefined.into());
        rv.set(promise.into());
        return;
    }

    let message = v8::String::new(scope, "File is closed.").unwrap();
    let exception = v8::Exception::error(scope, message);

    promise_resolver.reject(scope, exception);
    rv.set(promise.into());
}

/// Closes a file synchronously.
fn close_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();

    if let Some(file_ptr) = get_internal_ref::<Option<usize>>(scope, file_wrap, 0) {
        // Note: By creating a file reference and immediately dropping it will
        // make rust to close the file.
        let file = get_file_reference(*file_ptr);

        drop(file);
        set_internal_ref(scope, file_wrap, 0, None::<usize>);
        return;
    }

    throw_exception(scope, "File is closed.");
}

/// Reads asynchronously a chunk of a file (as bytes).
fn read(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();

    // Get chunk size and byte offset.
    let size = args.get(1).to_integer(scope).unwrap().value();
    let offset = args.get(2).to_integer(scope).unwrap().value();

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let file = get_internal_ref::<Option<usize>>(scope, file_wrap, 0);

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match file {
        Some(ptr) => get_file_reference(*ptr),
        None => {
            let message = v8::String::new(scope, "File is closed.").unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise.
            promise_resolver.reject(scope, exception);
            rv.set(promise.into());
            return;
        }
    };

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match read_file_op(&mut file, size, offset) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
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
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();
    let file = get_internal_ref::<Option<usize>>(scope, file_wrap, 0);

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match file {
        Some(ptr) => get_file_reference(*ptr),
        None => {
            throw_exception(scope, "File is closed.");
            return;
        }
    };

    // Get chunk size and byte offset.
    let size = args.get(1).to_integer(scope).unwrap().value();
    let offset = args.get(2).to_integer(scope).unwrap().value();

    match read_file_op(&mut file, size, offset) {
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
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();

    // Get data as ArrayBuffer.
    let data: v8::Local<v8::ArrayBufferView> = args.get(1).try_into().unwrap();

    let mut buffer = vec![0; data.byte_length()];
    data.copy_contents(&mut buffer);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let file = get_internal_ref::<Option<usize>>(scope, file_wrap, 0);

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match file {
        Some(ptr) => get_file_reference(*ptr),
        None => {
            let message = v8::String::new(scope, "File is closed.").unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise.
            promise_resolver.reject(scope, exception);
            rv.set(promise.into());
            return;
        }
    };

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match write_file_op(&mut file, &buffer) {
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
    // Get the file_wrap object.
    let file_wrap = args.get(0).to_object(scope).unwrap();
    let file = get_internal_ref::<Option<usize>>(scope, file_wrap, 0);

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match file {
        Some(ptr) => get_file_reference(*ptr),
        None => {
            throw_exception(scope, "File is closed.");
            return;
        }
    };

    // Get data as ArrayBuffer.
    let data: v8::Local<v8::ArrayBufferView> = args.get(1).try_into().unwrap();

    let mut buffer = vec![0; data.byte_length()];
    data.copy_contents(&mut buffer);

    if let Err(e) = write_file_op(&mut file, &buffer) {
        throw_exception(scope, &e.to_string());
    }
}

#[cfg(target_family = "unix")]
fn get_file_reference(fd: usize) -> File {
    unsafe { fs::File::from_raw_fd(fd as RawFd) }
}

#[cfg(target_family = "windows")]
fn get_file_reference(handle: usize) -> File {
    unsafe { fs::File::from_raw_handle(handle as RawHandle) }
}

#[cfg(target_family = "unix")]
/// Pure rust implementation of opening a file.
fn open_file_op<P: AsRef<Path>>(path: P) -> Result<usize> {
    // Note: The reason we leak the wrapped file handle is to prevent rust
    // from dropping the handle (a.k.a close the file) when current scope ends.
    match fs::File::open(path) {
        Ok(file) => Ok(Box::leak(Box::new(file)).as_raw_fd() as usize),
        Err(e) => bail!(e),
    }
}

#[cfg(target_family = "windows")]
/// Pure rust implementation of opening a file.
fn open_file_op<P: AsRef<Path>>(path: P) -> Result<usize> {
    // Note: The reason we leak the wrapped file handle is to prevent rust
    // from dropping the handle (a.k.a close the file) when current scope ends.
    match fs::File::open(path) {
        Ok(file) => Ok(Box::leak(Box::new(file)).as_raw_handle() as usize),
        Err(e) => bail!(e),
    }
}

/// Pure rust implementation of reading a chunk from a file.
fn read_file_op(file: &mut File, size: i64, offset: i64) -> Result<(usize, Vec<u8>)> {
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
fn write_file_op(file: &mut File, buffer: &[u8]) -> Result<()> {
    // Write buffer to file.
    if let Err(e) = file.write_all(buffer) {
        bail!(e);
    }
    Ok(())
}
