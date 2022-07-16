use crate::bindings::set_function_to;
use crate::bindings::throw_exception;
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "readSync", read_sync);
    set_function_to(scope, target, "writeSync", write_sync);

    // Return v8 global handle.
    v8::Global::new(scope, target)
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

    // Open file.
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            throw_exception(scope, &e.to_string());
            return;
        }
    };

    // Move file cursor to requested position.
    if let Err(e) = file.seek(SeekFrom::Start(offset as u64)) {
        throw_exception(scope, &e.to_string());
        return;
    }

    let mut buffer = vec![0; size as usize];

    match file.take(size as u64).read(&mut buffer) {
        Ok(n) => {
            // No more bytes to read.
            if n == 0 {
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
        Err(e) => throw_exception(scope, &e.to_string()),
    }
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

    // Open file.
    let mut file = match fs::File::create(path) {
        Ok(file) => file,
        Err(e) => {
            throw_exception(scope, &e.to_string());
            return;
        }
    };

    let mut buffer = vec![0; data.byte_length()];

    data.copy_contents(&mut buffer);

    // Write buffer to file.
    if let Err(e) = file.write_all(&buffer) {
        throw_exception(scope, &e.to_string());
    }
}
