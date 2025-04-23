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
