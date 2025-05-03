use crate::bindings::get_internal_ref;
use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use crate::bindings::set_internal_ref;
use crate::bindings::throw_exception;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use rusqlite::params_from_iter;
use rusqlite::types::Value as SqlValue;
use rusqlite::types::ValueRef;
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
use std::str::FromStr;
use std::vec;
use uuid::Uuid;

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);

    set_function_to(scope, target, "open", open);
    set_function_to(scope, target, "execute", execute);
    set_function_to(scope, target, "prepare", prepare);
    set_function_to(scope, target, "queryAll", query_all);
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

/// Executes a prepared statement and returns all results.
fn query_all(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get connection and required params.
    let connection = args.get(0).to_object(scope).unwrap();
    let stmt_reference = args.get(1).to_rust_string_lossy(scope);
    let params = v8::Local::<v8::Array>::try_from(args.get(2)).unwrap();
    let use_big_int = args.get(3).to_boolean(scope);

    // Extract prepared statement.
    let connection = get_internal_ref::<SQLiteConnection>(scope, connection, 0);
    let stmt_reference = Uuid::from_str(&stmt_reference).unwrap();
    let mut statements = connection.statements();
    let statement = match statements.get_mut(&stmt_reference) {
        Some(statement) => statement,
        None => {
            throw_exception(scope, &anyhow!("Invalid statement reference."));
            return;
        }
    };

    // Convert JavaScript values to SQLite values.
    let mut params = (0..params.length()).map(|i| {
        let value = params.get_index(scope, i).unwrap();
        to_sql_value(scope, value)
    });

    // Check for conversion errors.
    if let Some(Err(e)) = params.find(|value| value.is_err()) {
        throw_exception(scope, &anyhow!(e));
        return;
    }

    // Note: Since this is a prepared statement we can extract the names
    // of the result columns.
    let column_names: Vec<String> = statement
        .column_names()
        .iter()
        .map(|name| name.to_string())
        .collect();

    // Execute prepared statement with provided params.
    let params: Vec<SqlValue> = params.map(|value| value.unwrap()).collect();
    let mut rows = match statement.query(params_from_iter(params)) {
        Ok(rows) => rows,
        Err(e) => {
            throw_exception(scope, &anyhow!(e));
            return;
        }
    };

    let mut entries = vec![];

    // Convert each database row into a V8 object and collect them into a vector.
    while let Ok(Some(row)) = rows.next() {
        let target = v8::Object::new(scope);
        for (i, name) in column_names.iter().enumerate() {
            let value = row.get_ref_unwrap(i);
            let value = to_js_value(scope, value, use_big_int.is_true());
            set_constant_to(scope, target, name, value);
        }
        entries.push(target.into());
    }

    rv.set(v8::Array::new_with_elements(scope, entries.as_slice()).into());
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

// Converts V8 values to SQLite values. (https://www.sqlite.org/datatype3.html)
fn to_sql_value(scope: &mut v8::HandleScope, value: v8::Local<'_, v8::Value>) -> Result<SqlValue> {
    // Note: This approach is a bit messy, but it's the only reliable way
    // to map JavaScript values to SQLite-supported types.
    //
    // SQLite::NULL
    if value.is_null_or_undefined() {
        return Ok(SqlValue::Null);
    }
    // SQLite::REAL
    if value.is_number() {
        let value = value.number_value(scope).unwrap();
        return Ok(SqlValue::Real(value));
    }
    // SQLite::TEXT
    if value.is_string() {
        let value = value.to_rust_string_lossy(scope);
        return Ok(SqlValue::Text(value));
    }
    // SQLite::INTEGER
    if value.is_big_int() {
        return match value.integer_value(scope) {
            Some(value) => Ok(SqlValue::Integer(value)),
            None => bail!("BigInt value couldn't be converted to SQLite integer."),
        };
    }
    // SQLite::BLOB
    if value.is_array_buffer_view() {
        // Get data as ArrayBuffer.
        let data: v8::Local<v8::ArrayBufferView> = value.try_into().unwrap();
        let mut buffer = vec![0; data.byte_length()];
        data.copy_contents(&mut buffer);

        return Ok(SqlValue::Blob(buffer));
    }

    bail!("JavaScript value cannot be converted to a SQLite value.");
}

// Converts SQLite values to JavaScript values.
fn to_js_value<'a>(
    scope: &mut v8::HandleScope<'a>,
    sql_value: ValueRef<'_>,
    use_big_int: bool,
) -> v8::Local<'a, v8::Value> {
    match sql_value {
        ValueRef::Null => v8::null(scope).into(),
        ValueRef::Real(value) => v8::Number::new(scope, value).into(),
        ValueRef::Integer(value) => match use_big_int {
            true => v8::BigInt::new_from_i64(scope, value).into(),
            false => v8::Number::new(scope, value as f64).into(),
        },
        ValueRef::Text(bytes) => {
            let value = String::from_utf8(bytes.to_vec()).unwrap();
            v8::String::new(scope, &value).unwrap().into()
        }
        ValueRef::Blob(bytes) => {
            // Create array buffer to store the blob.
            let buffer = v8::ArrayBuffer::new(scope, bytes.len());
            let buffer_store = buffer.get_backing_store();
            // Copy the slice's bytes into v8's typed-array backing store.
            for (i, value) in bytes.iter().enumerate() {
                buffer_store[i].set(*value);
            }
            buffer.into()
        }
    }
}
