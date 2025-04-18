use crate::bindings::get_internal_ref;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::throw_exception;
use anyhow::anyhow;
use anyhow::Result;
use rusqlite::Connection;
use rusqlite::LoadExtensionGuard;
use rusqlite::OpenFlags;
use std::path::Path;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "enable_extentions", enable_extentions);
    set_function_to(scope, target, "load_extension", load_extension);
    set_function_to(scope, target, "close", close);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

/// Opens a new connection to an SQLite database.
fn open(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get database path.
    let path = args.get(0).to_rust_string_lossy(scope);

    // Get flags which can be used to configure how a DB is opened.
    let read_only = args.get(1).to_boolean(scope);
    let allow_extention = args.get(2).to_boolean(scope);

    let mut flags = OpenFlags::default();

    if read_only.is_true() {
        flags.remove(OpenFlags::SQLITE_OPEN_READ_WRITE);
        flags.insert(OpenFlags::SQLITE_OPEN_READ_ONLY);
    }

    // When the ":memory:" is provided as path we should open the
    // DB connection in memory.
    let connection = match path.as_str() {
        ":memory:" => Connection::open_in_memory_with_flags(flags),
        _ => Connection::open_with_flags(path, flags),
    };

    let connection = match connection {
        Ok(conn) => conn,
        Err(e) => {
            throw_exception(scope, &anyhow!(e));
            return;
        }
    };

    // Note: By default the extentions are disabled in SQLite, so we're gonna
    // enable it here only if the caller requests it during initialization.
    if allow_extention.is_true() {
        unsafe {
            if let Err(e) = connection.load_extension_enable() {
                throw_exception(scope, &anyhow!(e));
                return;
            }
        }
    }

    let connection = Rc::new(Some(connection));
    let connection_wrap = v8::ObjectTemplate::new(scope);

    connection_wrap.set_internal_field_count(2);

    // Store Rust instance inside a V8 handle.
    let connection_wrap = connection_wrap.new_instance(scope).unwrap();

    // Note: To automatically close the connection (i.e., drop the instance) when
    // V8 garbage collects the object that internally holds the Rust connection,
    // we use a Weak reference and a finalizer callback. This is why the connection
    // is wrapped in an Rc<Option<T>>.
    let mut connection_rc = connection.clone();
    let connection_weak = v8::Weak::with_guaranteed_finalizer(
        scope,
        connection_wrap,
        Box::new(move || {
            drop(std::mem::take(&mut connection_rc));
        }),
    );

    set_internal_ref(scope, connection_wrap, 0, connection);
    set_internal_ref(scope, connection_wrap, 1, connection_weak);

    rv.set(connection_wrap.into());
}

/// Enables or disables loading extetnions to SQLite.
fn enable_extentions(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the connection and flag from the params.
    let connection_wrap = args.get(0).to_object(scope).unwrap();
    let allow = args.get(1).to_boolean(scope);

    // Extract the connection from V8 handle.
    let connection = get_internal_ref::<Rc<Option<Connection>>>(scope, connection_wrap, 0);
    let connection = match connection.as_ref() {
        Some(connection) => connection,
        None => {
            throw_exception(scope, &anyhow!("Connection is closed."));
            return;
        }
    };

    // Try enable extentions for the SQLite database.
    if allow.is_true() {
        unsafe {
            if let Err(e) = connection.load_extension_enable() {
                throw_exception(scope, &anyhow!(e));
                return;
            }
        }
    }

    if let Err(e) = connection.load_extension_disable() {
        throw_exception(scope, &anyhow!(e));
    }
}

/// Loads a shared library into an SQLite database.
fn load_extension(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the connection and extension path.
    let connection_wrap = args.get(0).to_object(scope).unwrap();
    let path = args.get(1).to_rust_string_lossy(scope);

    // Extract the connection from V8 handle.
    let connection = get_internal_ref::<Rc<Option<Connection>>>(scope, connection_wrap, 0);
    let connection = match connection.as_ref() {
        Some(connection) => connection,
        None => {
            throw_exception(scope, &anyhow!("Connection is closed."));
            return;
        }
    };

    if let Err(e) = load_sqlite_extetnion_op(connection, path) {
        throw_exception(scope, &anyhow!(e));
    }
}

/// Closes the database connection.
fn close(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get the connection wrap object.
    let connection_wrap = args.get(0).to_object(scope).unwrap();
    let connection = get_internal_ref::<Rc<Option<Connection>>>(scope, connection_wrap, 0);

    // Drop the connection.
    drop(std::mem::take(connection));
}

// Load the SQLite extension.
// https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#example-11
fn load_sqlite_extetnion_op<P: AsRef<Path>>(conn: &Connection, path: P) -> Result<()> {
    // Safety: we don't execute any SQL statements while
    // extension loading is enabled.
    let _guard = unsafe { LoadExtensionGuard::new(conn)? };

    unsafe { conn.load_extension(path, None).map_err(|e| anyhow!(e)) }
}
