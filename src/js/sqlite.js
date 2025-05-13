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

  /**
   * Creates a new SQLite database instance.
   *
   * @param {String} path - The path of the database.
   * @param {Object} [options] - Configuration options for the database connection.
   * @param {boolean} [options.open] - If true, the database is opened by the constructor.
   * @param {boolean} [options.readOnly] - If true, the database is opened in read-only mode.
   * @param {boolean} [options.allowExtention] -  If true, loading extentions is enabled.
   * @returns {Database}
   */
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
   *
   * @param {String} sql - A SQL string to execute.
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
   *
   * @param {String} - A SQL string to execute.
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
   *
   * @param {String} path - The path to the shared library to load.
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
   *
   * @param {boolean} allow - Whether to allow loading extensions.
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
   *
   * @returns {Boolean}
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
 * Alias for the Database class.
 * https://nodejs.org/api/sqlite.html#class-databasesync
 */
export const DatabaseSync = Database;

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
   *
   * @param {...*} params - Zero or more values to bind to positional parameters.
   * @returns {Array<Object>} - An array of objects.
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
   *
   * @param {...*} params - Zero or more values to bind to positional parameters.
   * @returns {Object|undefined} - An object corresponding to the first row.
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
   * Information about the executed query.
   *
   * @typedef Changes
   * @property {number|BigInt} changes - The number of rows modified.
   * @property {number|BigInt} lastInsertRowid - The most recently inserted rowid.
   */

  /**
   * Executes a prepared statement and returns the resulting changes.
   *
   * @param {...*} params - Zero or more values to bind to positional parameters.
   * @returns {Changes}
   */
  run(...params) {
    // Check if connection is closed.
    if (!this.#conn) {
      throw new Error('Connection is already closed.');
    }

    return binding.run(this.#conn, this.#reference, params, this.#useBigInt);
  }

  /**
   * Column information for the prepared statement.
   *
   * @typedef Column
   * @property {String|null} column - The unaliased name of the column in the origin table.
   * @property {String|null} database - The unaliased name of the origin database.
   * @property {String|mull} name - The name assigned to the column in the result set of a SELECT statement.
   * @property {String|null} table - The unaliased name of the origin table.
   * @property {String|null} type - The declared data type of the column.
   */

  /**
   * Returns information about the columns used by the prepared statement.
   *
   * @returns {Array<Column>}
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
   *
   * @param {boolean} enable - Flag to enable or disable BigInts.
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
