use crate::bindings::get_internal_ref;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::throw_exception;
use anyhow::anyhow;
use anyhow::Result;
use rusqlite::Connection;
use rusqlite::LoadExtensionGuard;
use rusqlite::OpenFlags;
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "execute", execute);
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

    let connection_wrap = v8::ObjectTemplate::new(scope);
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

    connection_wrap.set_internal_field_count(2);

    // Store Rust instance inside a V8 handle.
    let connection_wrap = connection_wrap.new_instance(scope).unwrap();
    let connection_ptr = set_internal_ref(scope, connection_wrap, 0, Some(connection));
    let weak_rc = Rc::new(Cell::new(None));

    // Note: To automatically close the connection (i.e., drop the instance) when
    // V8 garbage collects the object that internally holds the Rust connection,
    // we use a Weak reference with a finalizer callback.
    let connection_weak = v8::Weak::with_finalizer(
        scope,
        connection_wrap,
        Box::new({
            let weak_rc = weak_rc.clone();
            move |isolate| unsafe {
                drop(Box::from_raw(connection_ptr));
                drop(v8::Weak::from_raw(isolate, weak_rc.get()));
            }
        }),
    );

    // Store the weak ref pointer into the "shared" cell.
    weak_rc.set(connection_weak.into_raw());
    set_internal_ref(scope, connection_wrap, 1, weak_rc);

    rv.set(connection_wrap.into());
}

/// Run multiple SQL statements (that cannot take any parameters).
fn execute(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get connection and SQL query.
    let connection = args.get(0).to_object(scope).unwrap();
    let sql = args.get(1).to_rust_string_lossy(scope);

    // Extract connection and execute SQL (batch) query.
    let connection = get_internal_ref::<Option<Connection>>(scope, connection, 0);
    let connection = match connection.as_ref() {
        Some(connection) => connection,
        None => {
            throw_exception(scope, &anyhow!("Connection is closed."));
            return;
        }
    };

    if let Err(e) = connection.execute_batch(&sql) {
        throw_exception(scope, &anyhow!(e));
    }
}

/// Enables or disables loading extetnions to SQLite.
fn enable_extentions(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the connection and flag from the params.
    let connection = args.get(0).to_object(scope).unwrap();
    let allow = args.get(1).to_boolean(scope);

    let connection = get_internal_ref::<Option<Connection>>(scope, connection, 0);
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
    let connection = args.get(0).to_object(scope).unwrap();
    let path = args.get(1).to_rust_string_lossy(scope);

    let connection = get_internal_ref::<Option<Connection>>(scope, connection, 0);
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
    let connection = args.get(0).to_object(scope).unwrap();
    let connection = get_internal_ref::<Option<Connection>>(scope, connection, 0);
    let connection = match connection.take() {
        Some(connection) => connection,
        None => {
            throw_exception(scope, &anyhow!("Connection is closed."));
            return;
        }
    };

    if let Err((_, e)) = connection.close() {
        throw_exception(scope, &anyhow!(e));
    }
}

// Load the SQLite extension.
// https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html#example-11
fn load_sqlite_extetnion_op<P: AsRef<Path>>(conn: &Connection, path: P) -> Result<()> {
    // Safety: we don't execute any SQL statements while
    // extension loading is enabled.
    let _guard = unsafe { LoadExtensionGuard::new(conn)? };

    unsafe { conn.load_extension(path, None).map_err(|e| anyhow!(e)) }
}
