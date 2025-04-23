use crate::bindings::get_internal_ref;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::throw_exception;
use anyhow::anyhow;
use anyhow::Result;
use rusqlite::Connection;
use rusqlite::LoadExtensionGuard;
use rusqlite::OpenFlags;
use rusqlite::Statement;
use std::cell::Cell;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Drop;
use std::path::Path;
use std::rc::Rc;
use uuid::Uuid;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "execute", execute);
    set_function_to(scope, target, "prepare", prepare);
    set_function_to(scope, target, "enableExtentions", enable_extentions);
    set_function_to(scope, target, "loadExtension", load_extension);
    set_function_to(scope, target, "close", close);

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

type StatementMap<'s> = HashMap<Uuid, Statement<'s>>;

/// A connection wrapper that also stores SQLite prepared statements.
struct SQLiteConnection<'s> {
    // The actual SQLite connection.
    conn: Option<Connection>,
    // Prepared statements associated with the connection.
    statements: RefCell<StatementMap<'s>>,
}

impl<'s> SQLiteConnection<'s> {
    // Creates a new SQLite connection wrapper.
    pub fn new(conn: Connection) -> Self {
        SQLiteConnection {
            conn: Some(conn),
            statements: RefCell::new(HashMap::new()),
        }
    }

    // Returns a mut reference to statements.
    pub fn statements(&self) -> RefMut<'s, StatementMap> {
        self.statements.borrow_mut()
    }
}

impl Drop for SQLiteConnection<'_> {
    // Note: We need to first drop all the prepared statements tied
    // to the connection and then the connection itself.
    fn drop(&mut self) {
        self.statements.borrow_mut().clear();
    }
}

impl Deref for SQLiteConnection<'_> {
    // We should return a ref to the inner connection.
    type Target = Option<Connection>;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for SQLiteConnection<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
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
        Ok(conn) => SQLiteConnection::new(conn),
        Err(e) => {
            throw_exception(scope, &anyhow!(e));
            return;
        }
    };

    // Note: By default the extentions are disabled in SQLite, so we're gonna
    // enable it here only if the caller requests it during initialization.
    if allow_extention.is_true() {
        unsafe {
            // We know connection is not None.
            if let Err(e) = connection.as_ref().unwrap().load_extension_enable() {
                throw_exception(scope, &anyhow!(e));
                return;
            }
        }
    }

    connection_wrap.set_internal_field_count(2);

    // Store Rust instance inside a V8 handle.
    let connection_wrap = connection_wrap.new_instance(scope).unwrap();
    let connection_ptr = set_internal_ref(scope, connection_wrap, 0, connection);
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
    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
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

/// Compiles a SQL statement into a prepared statement.
fn prepare(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get connection and SQL query.
    let connection = args.get(0).to_object(scope).unwrap();
    let sql = args.get(1).to_rust_string_lossy(scope);

    // Compile a new SQL statement.
    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
    let statement = match connection.as_ref() {
        Some(connection) => connection.prepare(&sql).map_err(|e| anyhow!(e)),
        None => Err(anyhow!("Connection is closed.")),
    };

    let (id, statement) = match statement {
        Ok(statement) => (Uuid::new_v4(), statement),
        Err(e) => {
            throw_exception(scope, &anyhow!(e));
            return;
        }
    };

    // Save SQL prepared statement into the hash-map.
    let _ = connection.statements().insert(id, statement);
    let reference = v8::String::new(scope, &id.to_string()).unwrap();

    rv.set(reference.into());
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

    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
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

    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
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
    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
    let connection = match connection.take() {
        Some(connection) => connection,
        None => {
            throw_exception(scope, &anyhow!("Connection is closed."));
            return;
        }
    };

    if let Err((_, e)) = connection.close() {
        // TODO(aalykiot): Move connection back to the Option.
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
