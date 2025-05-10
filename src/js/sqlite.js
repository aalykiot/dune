/**
 * SQLite APIs
 *
 * The sqlite module facilitates working with SQLite databases.
 *
 * @see {@link https://nodejs.org/api/sqlite.html}
 *
 * @module SQLite
 */

const binding = process.binding('sqlite');

/**
 * This class represents a single connection to a SQLite database.
 */
export class Database {
  #conn;
  #readOnly;
  #allowExtention;
  #path;

  constructor(path, options) {
    // Check if the path argument is a valid type.
    if (typeof path !== 'string') {
      throw new TypeError(`The "path" argument must be of type string.`);
    }

    this.#conn = null;
    this.#readOnly = options?.readOnly || false;
    this.#allowExtention = options?.allowExtention || false;
    this.#path;

    if (options?.open ?? true) {
      this.#conn = binding.open(path, this.#readOnly, this.#allowExtention);
    }
  }

  /**
   * Opens the database specified in the path argument.
   */
  open() {
    // Check if connection is open.
    if (this.#conn) {
      throw new Error('Connection is already open.');
    }

    this.#conn = binding.open(this.#path, this.#readOnly, this.#allowExtention);
  }

  /**
   * Executes SQL statements without returning any results.
   */
  exec(sql) {
    // Check if the sql argument is a valid type.
    if (typeof sql !== 'string') {
      throw new TypeError(`The "sql" argument must be of type string.`);
    }

    // Check if the connection is open.
    if (!this.#conn) {
      throw new Error('Connection is closed.');
    }

    binding.execute(this.#conn, sql);
  }

  /**
   * Compiles a SQL statement into a prepared statement.
   */
  prepare(sql) {
    // Check if the sql argument is a valid type.
    if (typeof sql !== 'string') {
      throw new TypeError(`The "sql" argument must be of type string.`);
    }

    // Check if the connection is open.
    if (!this.#conn) {
      throw new Error('Connection is closed.');
    }

    // Get an internal reference for the compiled SQL statement.
    const id = binding.prepare(this.#conn, sql);
    const statement = new Statement(this.#conn, id, sql);

    return statement;
  }

  /**
   * Loads a shared library into the database connection.
   */
  loadExtension(path) {
    // Check if the path argument is a valid type.
    if (typeof path !== 'string') {
      throw new TypeError(`The "path" argument must be of type string.`);
    }

    // Check if the connection is open.
    if (!this.#conn) {
      throw new Error('Connection is closed.');
    }

    // Check if loading extentions is enabled.
    if (!this.#allowExtention) {
      throw new Error('Loading extentions is disabled for this DB connection.');
    }

    binding.loadExtension(this.#conn, path);
  }

  /**
   * Enables or disables the loadExtension SQL function.
   */
  enableLoadExtension(allow = true) {
    // Check if allow param is boolean.
    if (typeof allow !== 'boolean') {
      throw new Error(`The "allow" argument must be of type boolean.`);
    }

    // Check if the connection is open.
    if (!this.#conn) {
      throw new Error('Connection is closed.');
    }

    // Note: When allowExtension is false when constructing, you cannot enable
    // loading extensions for security reasons.
    if (!this.#allowExtention) {
      throw new Error(
        'Cannot enable extensions: allowExtension was set to false during construction.'
      );
    }

    binding.enableExtentions(this.#conn, allow);
  }

  /**
   * Returns whether the database is currently open or not.
   */
  get isOpen() {
    return !!this.#conn;
  }

  /**
   * Closes the database connection.
   */
  close() {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    binding.close(this.#conn);
    this.#conn = null;
  }
}

/**
 * This class represents a single prepared statement.
 */
class Statement {
  #conn;
  #reference;
  #sql;
  #useBigInt;

  constructor(conn, reference, sql) {
    this.#conn = conn;
    this.#reference = reference;
    this.#sql = sql;
    this.#useBigInt = false;
  }

  /**
   * Executes a prepared statement and returns all results.
   */
  all(...params) {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.query(this.#conn, this.#reference, params, this.#useBigInt);
  }

  /**
   * Returns the first result.
   */
  get(...params) {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.queryOne(
      this.#conn,
      this.#reference,
      params,
      this.#useBigInt
    );
  }

  /**
   * Executes a prepared statement and returns the resulting changes.
   */
  run(...params) {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.run(this.#conn, this.#reference, params, this.#useBigInt);
  }

  /**
   * Returns information about the columns used by the prepared statement.
   */
  columns() {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.columns(this.#conn, this.#reference);
  }

  /**
   * Enables or disables the use of BigInts when reading INTEGER fields.
   */
  setReadBigInts(enable = false) {
    // Check if flag is a boolean value.
    if (typeof enable !== 'boolean') {
      throw new TypeError(`The "enable" argument must be of type boolean.`);
    }

    this.#useBigInt = enable;
  }

  /**
   * The source SQL text with parameter placeholders replaced.
   */
  get expandedSQL() {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.expandedSql(this.#conn, this.#reference);
  }

  /**
   * The source SQL text of the prepared statement.
   */
  get sourceSQL() {
    return this.#sql;
  }
}
