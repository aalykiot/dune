use crate::bindings::get_internal_ref;
use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::set_property_to;
use crate::bindings::throw_exception;
use crate::event_loop::FsEvent;
use crate::event_loop::LoopHandle;
use crate::event_loop::TaskResult;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use notify::EventKind;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;
use std::time::UNIX_EPOCH;

#[cfg(target_family = "unix")]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

#[cfg(target_family = "windows")]
use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

#[derive(Default, Debug, Serialize, Deserialize)]
/// Struct that provides information about a file.
struct FileStatistics {
    size: u64,
    access_time: Option<Duration>,
    modified_time: Option<Duration>,
    birth_time: Option<Duration>,
    is_directory: bool,
    is_file: bool,
    is_symbolic_link: bool,
    is_socket: Option<bool>,
    is_fifo: Option<bool>,
    is_block_device: Option<bool>,
    is_character_device: Option<bool>,
    blocks: Option<u64>,
    block_size: Option<u64>,
    mode: Option<u32>,
    device: Option<u64>,
    group_id: Option<u32>,
    inode: Option<u64>,
    hard_links: Option<u64>,
    rdev: Option<u64>,
}

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "openSync", open_sync);
    set_function_to(scope, target, "read", read);
    set_function_to(scope, target, "readSync", read_sync);
    set_function_to(scope, target, "write", write);
    set_function_to(scope, target, "writeSync", write_sync);
    set_function_to(scope, target, "stat", stat);
    set_function_to(scope, target, "statSync", stat_sync);
    set_function_to(scope, target, "mkdir", mkdir);
    set_function_to(scope, target, "mkdirSync", mkdir_sync);
    set_function_to(scope, target, "rmdir", rmdir);
    set_function_to(scope, target, "rmdirSync", rmdir_sync);
    set_function_to(scope, target, "readdir", readdir);
    set_function_to(scope, target, "readdirSync", readdir_sync);
    set_function_to(scope, target, "rm", rm);
    set_function_to(scope, target, "rmSync", rm_sync);
    set_function_to(scope, target, "close", close);
    set_function_to(scope, target, "closeSync", close_sync);
    set_function_to(scope, target, "rename", rename);
    set_function_to(scope, target, "renameSync", rename_sync);
    set_function_to(scope, target, "watch", watch);
    set_function_to(scope, target, "unwatch", unwatch);

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
        let file = get_file_reference(file_ptr);

        let file_wrapper = v8::ObjectTemplate::new(scope);

        // Allocate space for the wrapped Rust type.
        file_wrapper.set_internal_field_count(1);

        let file_wrapper = file_wrapper.new_instance(scope).unwrap();
        let fd = v8::Number::new(scope, file_ptr as f64);

        set_constant_to(scope, file_wrapper, "fd", fd.into());
        set_internal_ref(scope, file_wrapper, 0, Some(file));

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

    // Get flags which can be used to configure how a file is opened.
    let flags = args.get(1).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match open_file_op(path, flags) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsOpenFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
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

    // Get flags which can be used to configure how a file is opened.
    let flags = args.get(1).to_rust_string_lossy(scope);

    match open_file_op(path, flags) {
        Ok(file_ptr) => {
            let file = get_file_reference(file_ptr);
            let file_wrapper = v8::ObjectTemplate::new(scope);

            // Allocate space for the wrapped Rust type.
            file_wrapper.set_internal_field_count(1);

            let file_wrapper = file_wrapper.new_instance(scope).unwrap();
            let fd = v8::Number::new(scope, file_ptr as f64);

            set_constant_to(scope, file_wrapper, "fd", fd.into());
            set_internal_ref(scope, file_wrapper, 0, Some(file));

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

        // Deserialize bincode binary into actual rust types.
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

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match get_internal_ref::<Option<File>>(scope, file_wrap, 0) {
        Some(file) => file.try_clone().unwrap(),
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

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsReadFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
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

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match get_internal_ref::<Option<File>>(scope, file_wrap, 0) {
        Some(file) => file.try_clone().unwrap(),
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

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match get_internal_ref::<Option<File>>(scope, file_wrap, 0) {
        Some(file) => file.try_clone().unwrap(),
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

        move |_: LoopHandle, maybe_result: TaskResult| {
            // Get a mut reference to the runtime's state.
            let mut state = state_rc.borrow_mut();
            let fs_write_handle = FsWriteFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(fs_write_handle));
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

    // Check if the file is already closed, otherwise create a file reference.
    let mut file = match get_internal_ref::<Option<File>>(scope, file_wrap, 0) {
        Some(file) => file.try_clone().unwrap(),
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

/// Describes what will run after the async stats_op completes.
struct FsStatFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsStatFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // Unwrap the result.
        let result = self.maybe_result.take().unwrap();

        // Something went wrong while getting the file's stats.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Otherwise, resolve the promise passing the result.
        let result = result.unwrap();

        // Deserialize bincode binary into actual rust types.
        let stats: FileStatistics = bincode::deserialize(&result).unwrap();
        let stats = create_v8_stats_object(scope, stats);

        self.promise
            .open(scope)
            .resolve(scope, stats.into())
            .unwrap();
    }
}

/// Get's asynchronously file statistics.
fn stat(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get the path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let task = move || match stats_op(path) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsStatFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Get's file statistics.
fn stat_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the path.
    let path = args.get(0).to_rust_string_lossy(scope);

    match stats_op(path) {
        Ok(stats) => rv.set(create_v8_stats_object(scope, stats).into()),
        Err(e) => throw_exception(scope, &e.to_string()),
    };
}

/// Describes what will run after the async mkdir_op completes.
struct FsMkdirFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsMkdirFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // If the result is None then mkdir worked.
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

        // Something went wrong while getting the file's stats.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        unreachable!();
    }
}

/// Creates a directory asynchronously.
fn mkdir(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get desired folder location.
    let path = args.get(0).to_rust_string_lossy(scope);
    let recursive = args.get(1).to_rust_string_lossy(scope) == "true";

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let task = move || match mkdir_op(path, recursive) {
        Ok(_) => None,
        Err(e) => Some(Result::Err(e)),
    };

    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsMkdirFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Creates a directory synchronously.
fn mkdir_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get desired folder location.
    let path = args.get(0).to_rust_string_lossy(scope);
    let recursive = args.get(1).to_rust_string_lossy(scope) == "true";

    if let Err(e) = mkdir_op(path, recursive) {
        throw_exception(scope, &e.to_string());
    }
}

/// Describes what will run after the async rmdir_op completes.
struct FsRmdirFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsRmdirFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // If the result is None then mkdir worked.
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

        // Something went wrong while getting the file's stats.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        unreachable!();
    }
}

/// Removes empty directories asynchronously.
fn rmdir(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get to be removed folder location.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let task = move || match rmdir_op(path) {
        Ok(_) => None,
        Err(e) => Some(Result::Err(e)),
    };

    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsRmdirFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Removes empty directories.
fn rmdir_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get to be removed folder location.
    let path = args.get(0).to_rust_string_lossy(scope);

    if let Err(e) = rmdir_op(path) {
        throw_exception(scope, &e.to_string());
    }
}

/// Describes what will run after the async read_dir_op completes.
struct ReadDirFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for ReadDirFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // Unwrap the result.
        let result = self.maybe_result.take().unwrap();

        // Check if something went wrong on directory read.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Otherwise, resolve the promise passing the result.
        let result = result.unwrap();

        // Deserialize bincode binary into an actual rust type.
        let directory: Vec<OsString> = bincode::deserialize(&result).unwrap();
        let directory: Vec<v8::Local<v8::Value>> = directory
            .iter()
            .map(|entry| entry.to_str().unwrap())
            .map(|entry| v8::String::new(scope, entry).unwrap())
            .map(|entry_value| entry_value.into())
            .collect();

        let directory_value = v8::Array::new_with_elements(scope, &directory);

        self.promise
            .open(scope)
            .resolve(scope, directory_value.into())
            .unwrap();
    }
}

/// Reads the contents of a directory asynchronously.
fn readdir(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get desired folder location.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let task = move || match readdir_op(path) {
        Ok(result) => Some(Ok(bincode::serialize(&result).unwrap())),
        Err(e) => Some(Result::Err(e)),
    };

    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = ReadDirFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Reads the contents of a directory synchronously.
fn readdir_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get the path.
    let path = args.get(0).to_rust_string_lossy(scope);

    match readdir_op(path) {
        Ok(directory) => {
            // Cast OsString values to v8::Locals.
            let directory: Vec<v8::Local<v8::Value>> = directory
                .iter()
                .map(|entry| entry.to_str().unwrap())
                .map(|entry| v8::String::new(scope, entry).unwrap())
                .map(|entry_value| entry_value.into())
                .collect();

            let directory_value = v8::Array::new_with_elements(scope, &directory);

            rv.set(directory_value.into());
        }
        Err(e) => throw_exception(scope, &e.to_string()),
    }
}

/// Describes what will run after the async rm_op completes.
struct FsRmFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsRmFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // If the result is None then mkdir worked.
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

        // Something went wrong while getting the file's stats.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        unreachable!();
    }
}

/// Removes files and directories asynchronously.
fn rm(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get to be removed folder location.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    let task = move || match rm_op(path) {
        Ok(_) => None,
        Err(e) => Some(Result::Err(e)),
    };

    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsRmFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Removes files and directories.
fn rm_sync(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get to be removed folder location.
    let path = args.get(0).to_rust_string_lossy(scope);

    if let Err(e) = rm_op(path) {
        throw_exception(scope, &e.to_string());
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

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    if let Some(file) = get_internal_ref::<Option<File>>(scope, file_wrap, 0).take() {
        // Note: By taking the file reference out of the option and immediately dropping
        // it will make rust to close the file.
        drop(file);

        let success = v8::Boolean::new(scope, true);
        promise_resolver.resolve(scope, success.into());
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

    if let Some(file) = get_internal_ref::<Option<File>>(scope, file_wrap, 0).take() {
        // Note: By taking the file reference out of the option and immediately dropping
        // it will make rust to close the file.
        return drop(file);
    }

    throw_exception(scope, "File is closed.");
}

/// Describes what will run after the async rename_op completes.
struct FsRenameFuture {
    promise: v8::Global<v8::PromiseResolver>,
    maybe_result: TaskResult,
}

impl JsFuture for FsRenameFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // If the result is None then renaming worked.
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

        // Something went wrong while renaming the file.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        unreachable!();
    }
}

/// Renames a file asynchronously.
fn rename(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get `from` and `to` values.
    let from = args.get(0).to_rust_string_lossy(scope);
    let to = args.get(1).to_rust_string_lossy(scope);

    // Create a promise resolver and extract the actual promise.
    let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
    let promise = promise_resolver.get_promise(scope);

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // The actual async task.
    let task = move || match rename_op(from, to) {
        Ok(_) => None,
        Err(e) => Some(Result::Err(e)),
    };

    // The callback that will run after the above task completes.
    let task_cb = {
        let promise = v8::Global::new(scope, promise_resolver);
        let state_rc = state_rc.clone();

        move |_: LoopHandle, maybe_result: TaskResult| {
            let mut state = state_rc.borrow_mut();
            let future = FsRenameFuture {
                promise,
                maybe_result,
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Spawn the async task using the event-loop.
    state.handle.spawn(task, Some(task_cb));

    rv.set(promise.into());
}

/// Renames a file synchronously.
fn rename_sync(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get `from` and `to` values.
    let from = args.get(0).to_rust_string_lossy(scope);
    let to = args.get(1).to_rust_string_lossy(scope);

    if let Err(e) = rename_op(from, to) {
        throw_exception(scope, &e.to_string());
    }
}

struct WatchFuture {
    event: FsEvent,
    on_event_cb: Rc<v8::Global<v8::Function>>,
}

impl JsFuture for WatchFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // Create a v8 array.
        let paths: Vec<v8::Local<v8::Value>> = self
            .event
            .paths
            .iter()
            .map(|path| path.to_str().unwrap())
            .map(|path| v8::String::new(scope, path).unwrap())
            .map(|path_value| path_value.into())
            .collect();

        let paths_value = v8::Array::new_with_elements(scope, &paths);

        // Format the event type.
        let kind = match self.event.kind {
            EventKind::Any => v8::String::new(scope, "any"),
            EventKind::Access(_) => v8::String::new(scope, "access"),
            EventKind::Create(_) => v8::String::new(scope, "create"),
            EventKind::Modify(_) => v8::String::new(scope, "modify"),
            EventKind::Remove(_) => v8::String::new(scope, "remove"),
            EventKind::Other => v8::String::new(scope, "other"),
        };

        let event = v8::Object::new(scope);
        set_constant_to(scope, event, "paths", paths_value.into());
        set_constant_to(scope, event, "kind", kind.unwrap().into());

        // Get access to the on_read callback.
        let on_event = v8::Local::new(scope, (*self.on_event_cb).clone());
        let undefined = v8::undefined(scope).into();

        on_event.call(scope, undefined, &[event.into()]);
    }
}

/// Starts a watcher for a requested path.
fn watch(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get path and recursive option.
    let path = args.get(0).to_rust_string_lossy(scope);
    let recursive = args.get(1).boolean_value(scope);

    // Get the on_event callback.
    let on_event_cb = v8::Local::<v8::Function>::try_from(args.get(2)).unwrap();
    let on_event_cb = Rc::new(v8::Global::new(scope, on_event_cb));

    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // A Rust wrapper around the JS on_event callback.
    let on_event = {
        let state_rc = state_rc.clone();
        move |_: LoopHandle, event: FsEvent| {
            let mut state = state_rc.borrow_mut();
            let future = WatchFuture {
                event,
                on_event_cb: Rc::clone(&on_event_cb),
            };
            state.pending_futures.push(Box::new(future));
        }
    };

    // Start the watcher.
    let index = match state.handle.fs_event_start(path, recursive, on_event) {
        Ok(index) => v8::Integer::new(scope, index as i32),
        Err(e) => {
            throw_exception(scope, &e.to_string());
            return;
        }
    };

    rv.set(index.into());
}

/// Stops a running watcher.
fn unwatch(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get the rid of the watcher.
    let index = args.get(0).int32_value(scope).unwrap() as u32;
    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    state.handle.fs_event_stop(&index);
}

#[cfg(target_family = "unix")]
fn get_file_reference(fd: usize) -> File {
    unsafe { fs::File::from_raw_fd(fd as RawFd) }
}

#[cfg(target_family = "windows")]
fn get_file_reference(handle: usize) -> File {
    unsafe { fs::File::from_raw_handle(handle as RawHandle) }
}

/// Pure rust implementation of opening a file.
fn open_file_op<P: AsRef<Path>>(path: P, flags: String) -> Result<usize> {
    // Options and flags which can be used to configure how a file is opened.
    let read = flags == "r" || flags == "r+" || flags == "w+" || flags == "a+";
    let write = flags == "r+" || flags == "w" || flags == "w+";
    let create = flags == "w" || flags == "w+" || flags == "a" || flags == "a+";
    let truncate = flags == "w+";
    let append = flags == "a" || flags == "a+";

    // Note: The reason we leak the wrapped file handle is to prevent rust
    // from dropping the handle (a.k.a close the file) when current scope ends.
    match OpenOptions::new()
        .read(read)
        .write(write)
        .create(create)
        .append(append)
        .truncate(truncate)
        .open(path)
    {
        #[cfg(target_family = "unix")]
        Ok(file) => Ok(Box::leak(Box::new(file)).as_raw_fd() as usize),
        #[cfg(target_family = "windows")]
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

/// Pure rust implementation of getting file statistics.
fn stats_op<P: AsRef<Path>>(path: P) -> Result<FileStatistics> {
    // Try get file's metadata information.
    match fs::metadata(path) {
        Ok(metadata) => {
            // Returns the size of the file, in bytes, this metadata is for.
            let size = metadata.len();

            // Returns the last access time of this metadata.
            let access_time = metadata
                .accessed()
                .ok()
                .map(|time| time.duration_since(UNIX_EPOCH).unwrap());

            // Returns the last modification time listed in this metadata.
            let modified_time = metadata
                .modified()
                .ok()
                .map(|time| time.duration_since(UNIX_EPOCH).unwrap());

            // Returns the creation time listed in this metadata.
            let birth_time = metadata
                .created()
                .ok()
                .map(|time| time.duration_since(UNIX_EPOCH).unwrap());

            let is_directory = metadata.is_dir();
            let is_file = metadata.is_file();
            let is_symbolic_link = metadata.is_symlink();

            #[allow(unused_mut)]
            let mut stats = FileStatistics {
                size,
                access_time,
                modified_time,
                birth_time,
                is_directory,
                is_file,
                is_symbolic_link,
                ..Default::default()
            };

            // In UNIX systems we can get some extra info.
            #[cfg(target_family = "unix")]
            {
                use std::os::unix::fs::FileTypeExt;
                use std::os::unix::fs::MetadataExt;

                stats.is_socket = Some(metadata.file_type().is_socket());
                stats.is_fifo = Some(metadata.file_type().is_fifo());
                stats.is_block_device = Some(metadata.file_type().is_block_device());
                stats.is_character_device = Some(metadata.file_type().is_char_device());
                stats.blocks = Some(metadata.blocks());
                stats.block_size = Some(metadata.blksize());
                stats.mode = Some(metadata.mode());
                stats.device = Some(metadata.dev());
                stats.group_id = Some(metadata.gid());
                stats.inode = Some(metadata.ino());
                stats.hard_links = Some(metadata.nlink());
                stats.rdev = Some(metadata.rdev());
            }

            Ok(stats)
        }
        Err(e) => bail!(e),
    }
}

/// Pure rust implementation of creating directories.
fn mkdir_op<P: AsRef<Path>>(path: P, recursive: bool) -> Result<()> {
    if recursive {
        fs::create_dir_all(path).map_err(|e| anyhow!(e))?;
        return Ok(());
    }
    fs::create_dir(path).map_err(|e| anyhow!(e))
}

/// Pure rust implementation of deleting (empty) directories.
fn rmdir_op<P: AsRef<Path>>(path: P) -> Result<()> {
    fs::remove_dir(path).map_err(|e| anyhow!(e))
}

/// Pure rust implementation of reading a directory.
fn readdir_op<P: AsRef<Path>>(path: P) -> Result<Vec<OsString>> {
    fs::read_dir(path)
        .map(|directory| directory.map(|entry| entry.unwrap().file_name()).collect())
        .map_err(|e| anyhow!(e))
}

/// Pure rust implementation of deleting files and directories.
fn rm_op<P: AsRef<Path>>(path: P) -> Result<()> {
    if stats_op(&path)?.is_directory {
        fs::remove_dir_all(&path).map_err(|e| anyhow!(e))?;
        return Ok(());
    }
    fs::remove_file(&path).map_err(|e| anyhow!(e))
}

/// Pure rust implementation of renaming a file/directory.
fn rename_op<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    fs::rename(from, to).map_err(|e| anyhow!(e))
}

/// Creates a JavaScript file stats object.
fn create_v8_stats_object<'a>(
    scope: &mut v8::HandleScope<'a>,
    stats: FileStatistics,
) -> v8::Local<'a, v8::Object> {
    // This will be out stats object.
    let target = v8::Object::new(scope);
    let undefined = v8::undefined(scope);

    // The size of the file in bytes.
    let size = v8::Number::new(scope, stats.size as f64);

    // The timestamp indicating the last time this file was accessed.
    let access_time: v8::Local<v8::Value> = match stats.access_time {
        Some(value) => v8::Number::new(scope, value.as_millis() as f64).into(),
        None => undefined.into(),
    };

    // The timestamp indicating the last time this file was modified.
    let modified_time: v8::Local<v8::Value> = match stats.modified_time {
        Some(value) => v8::Number::new(scope, value.as_millis() as f64).into(),
        None => undefined.into(),
    };

    // The timestamp indicating the creation time of this file.
    let birth_time: v8::Local<v8::Value> = match stats.birth_time {
        Some(value) => v8::Number::new(scope, value.as_millis() as f64).into(),
        None => undefined.into(),
    };

    set_property_to(scope, target, "size", size.into());
    set_property_to(scope, target, "atimeMs", access_time);
    set_property_to(scope, target, "mtimeMs", modified_time);
    set_property_to(scope, target, "birthtimeMs", birth_time);

    let is_file = v8::Boolean::new(scope, stats.is_file);
    let is_directory = v8::Boolean::new(scope, stats.is_directory);
    let is_symbolic_link = v8::Boolean::new(scope, stats.is_symbolic_link);

    set_property_to(scope, target, "isFile", is_file.into());
    set_property_to(scope, target, "isDirectory", is_directory.into());
    set_property_to(scope, target, "isSymbolicLink", is_symbolic_link.into());

    // UNIX only metrics.

    let is_socket: v8::Local<v8::Value> = match stats.is_socket {
        Some(value) => v8::Boolean::new(scope, value).into(),
        None => undefined.into(),
    };

    let is_fifo: v8::Local<v8::Value> = match stats.is_fifo {
        Some(value) => v8::Boolean::new(scope, value).into(),
        None => undefined.into(),
    };

    let is_block_device: v8::Local<v8::Value> = match stats.is_block_device {
        Some(value) => v8::Boolean::new(scope, value).into(),
        None => undefined.into(),
    };

    let is_character_device: v8::Local<v8::Value> = match stats.is_character_device {
        Some(value) => v8::Boolean::new(scope, value).into(),
        None => undefined.into(),
    };

    let blocks: v8::Local<v8::Value> = match stats.blocks {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let block_size: v8::Local<v8::Value> = match stats.block_size {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let mode: v8::Local<v8::Value> = match stats.mode {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let device: v8::Local<v8::Value> = match stats.device {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let group_id: v8::Local<v8::Value> = match stats.group_id {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let inode: v8::Local<v8::Value> = match stats.inode {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let hard_links: v8::Local<v8::Value> = match stats.hard_links {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    let rdev: v8::Local<v8::Value> = match stats.rdev {
        Some(value) => v8::Number::new(scope, value as f64).into(),
        None => undefined.into(),
    };

    set_property_to(scope, target, "isSocket", is_socket);
    set_property_to(scope, target, "isFIFO", is_fifo);
    set_property_to(scope, target, "isBlockDevice", is_block_device);
    set_property_to(scope, target, "isCharacterDevice", is_character_device);
    set_property_to(scope, target, "blocks", blocks);
    set_property_to(scope, target, "blksize", block_size);
    set_property_to(scope, target, "mode", mode);
    set_property_to(scope, target, "dev", device);
    set_property_to(scope, target, "gid", group_id);
    set_property_to(scope, target, "inode", inode);
    set_property_to(scope, target, "nlink", hard_links);
    set_property_to(scope, target, "rdev", rdev);

    target
}
