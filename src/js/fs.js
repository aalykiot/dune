// File System APIs
//
// The File System APIs enable interacting with the file system in a way modeled
// on standard POSIX functions.
//
// https://nodejs.org/api/fs.html

const binding = process.binding('fs');

const BUFFER_SIZE = 40 * 1024; // 40KB bytes buffer when reading.

/**
 * A File object is an object wrapper for a numeric file descriptor.
 */
export class File {
  /**
   * Creates a new File instance given a file path.
   *
   * @param {String} path
   * @param {String} [mode]
   * @returns {File}
   */
  constructor(path, mode) {
    // Check if the path argument is a valid type.
    if (typeof path !== 'string') {
      throw new TypeError(`The "path" argument must be of type string.`);
    }

    this._handle = null;
    this.path = path;
    this.mode = mode;
    this.fd = null;
  }

  /**
   * Asynchronously opens the file.
   *
   * @param {string} mode
   */
  async open(mode = 'r') {
    // Check if the file is already open.
    if (this._handle) {
      throw new Error(`The file is already open with fd: ${this.fd}`);
    }

    this._handle = await binding.open(this.path, this.mode || mode);
    this.fd = this._handle.fd;
  }

  /**
   * Synchronously opens the file.
   *
   * @param {string} mode
   */
  openSync(mode = 'r') {
    // Check if the file is already open.
    if (this._handle) {
      throw new Error(`The file is already open with fd: ${this.fd}`);
    }

    this._handle = binding.openSync(this.path, this.mode || mode);
    this.fd = this._handle.fd;
  }

  /**
   * Reads asynchronously some bytes from the file.
   *
   * @param {*} size
   * @param {*} offset
   * @returns {Promise<Uint8Array>}
   */
  async read(size = BUFFER_SIZE, offset = 0) {
    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    const bytes = await binding.read(this._handle, size, offset);
    const bytes_u8 = new Uint8Array(bytes);

    return bytes_u8;
  }

  /**
   * Reads synchronously some bytes from the file.
   *
   * @param {*} size
   * @param {*} offset
   * @returns {Uint8Array}
   */
  readSync(size = BUFFER_SIZE, offset = 0) {
    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    const bytes = binding.readSync(this._handle, size, offset);
    const bytes_u8 = new Uint8Array(bytes);

    return bytes_u8;
  }

  /**
   * Writes asynchronously a binary buffer to the file.
   *
   * @param {Uint8Array} data
   */
  async write(data) {
    // Check the data argument type.
    if (!(data instanceof Uint8Array)) {
      throw new TypeError(`The "data" argument must be of type Uint8Array.`);
    }

    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    return binding.write(this._handle, data);
  }

  /**
   * Writes synchronously a binary buffer to the file.
   *
   * @param {Uint8Array} data
   */
  writeSync(data) {
    // Check the data argument type.
    if (!(data instanceof Uint8Array)) {
      throw new TypeError(`The "data" argument must be of type Uint8Array.`);
    }

    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    binding.writeSync(this._handle, data);
  }

  /**
   * Retrieves asynchronously statistics for the file.
   */
  async stat() {
    // Check if the file is already closed.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    return binding.stat(this.path);
  }

  /**
   * Retrieves synchronously statistics for the file.
   */
  statSync() {
    // Check if the file is already closed.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    return binding.statSync(this.path);
  }

  /**
   * Closes the file asynchronously.
   */
  async close() {
    // Check if the file is already closed.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    await binding.close(this._handle);

    // Reset file object's attributes.
    this._handle = null;
    this.fd = null;
  }

  /**
   * Closes the file synchronously.
   */
  closeSync() {
    // Check if the file is already closed.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    binding.closeSync(this._handle);

    // Reset file object's attributes.
    this._handle = null;
    this.fd = null;
  }

  /**
   * The File objects should be asynchronously iterable.
   */
  [Symbol.asyncIterator]() {
    return {
      handle: this._handle,
      offset: 0,

      async next() {
        // Try read some bytes from the file.
        const bytes = await binding.read(this.handle, BUFFER_SIZE, this.offset);
        const bytes_u8 = new Uint8Array(bytes);

        // Update offset.
        this.offset += bytes_u8.length;

        return {
          done: bytes_u8.length === 0,
          value: bytes_u8,
        };
      },
    };
  }

  /**
   * The File objects should be iterable.
   */
  [Symbol.iterator]() {
    return {
      handle: this._handle,
      offset: 0,

      next() {
        // Try read some bytes from the file.
        const bytes = binding.readSync(this.handle, BUFFER_SIZE, this.offset);
        const bytes_u8 = new Uint8Array(bytes);

        // Update offset.
        this.offset += bytes_u8.length;

        return {
          done: bytes_u8.length === 0,
          value: bytes_u8,
        };
      },
    };
  }
}

/**
 * Asynchronously opens a file.
 *
 * @param {String} path
 * @param {String} mode
 * @returns {Promise<File>}
 */
export async function open(path, mode = 'r') {
  // Check the data argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Create a new file instance.
  const file = new File(path, mode);
  await file.open();

  return file;
}

/**
 * Synchronously opens a file.
 *
 * @param {String} path
 * @param {String} mode
 * @returns {File}
 */
export function openSync(path, mode = 'r') {
  // Check the data argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Create a new file instance.
  const file = new File(path, mode);
  file.openSync();

  return file;
}

/**
 * Reads asynchronously the entire contents of a file.
 *
 * @param {String} path
 * @param {String|Object} options
 * @returns {Promise<String|Uint8Array>}
 */

export async function readFile(path, options = {}) {
  // Create a new file instance.
  const file = new File(path, 'r');
  await file.open();

  // Buffer to fill the file bytes into.
  let data = new Uint8Array([]);

  // Note: Since the file object is async iterable will read the entire contents
  // of the file using the for-await loop.
  for await (let chunk of file) {
    data = new Uint8Array([...data, ...chunk]);
  }

  await file.close();

  // Decode given an encoder.
  const encoding = typeof options === 'string' ? options : options.encoding;

  if (encoding) {
    return new TextDecoder(encoding).decode(data);
  }

  return data;
}

/**
 * Reads synchronously the entire contents of a file.
 *
 * @param {String} path
 * @param {String|Object} options
 * @returns {String|Uint8Array}
 */

export function readFileSync(path, options = {}) {
  // Create a new file instance.
  const file = new File(path, 'r');
  file.openSync();

  // Buffer to fill the file bytes into.
  let data = new Uint8Array([]);

  // Note: Since the file object is iterable will read the entire contents
  // of the file using the for-of loop.
  for (let chunk of file) {
    data = new Uint8Array([...data, ...chunk]);
  }

  file.closeSync();

  // Decode given an encoder.
  const encoding = typeof options === 'string' ? options : options.encoding;

  if (encoding) {
    return new TextDecoder(encoding).decode(data);
  }

  return data;
}

/**
 * Writes asynchronously contents to a file.
 *
 * @param {String} path
 * @param {String|Uint8Array} data
 * @param {String|Object} options
 */

export async function writeFile(path, data, options = {}) {
  // Check the data argument type.
  if (!(data instanceof Uint8Array) && typeof data !== 'string') {
    throw new TypeError(
      `The "data" argument must be of type string or Uint8Array.`
    );
  }

  let encoding = typeof options === 'string' ? options : options.encoding;

  // Default to utf-8 encoding.
  if (!encoding) encoding = 'utf-8';

  // Create a file instance.
  const file = new File(path, 'w');
  const data_u8 = new TextEncoder(encoding).encode(data);

  // Open file, write data, and close it.
  await file.open();
  await file.write(data_u8);
  await file.close();
}

/**
 * Writes synchronously contents to a file.
 *
 * @param {String} path
 * @param {String|Uint8Array} data
 * @param {String|Object} options
 */

export function writeFileSync(path, data, options = {}) {
  // Check the data argument type.
  if (!(data instanceof Uint8Array) && typeof data !== 'string') {
    throw new TypeError(
      `The "data" argument must be of type string or Uint8Array.`
    );
  }

  let encoding = typeof options === 'string' ? options : options.encoding;

  // Default to utf-8 encoding.
  if (!encoding) encoding = 'utf-8';

  // Create a file instance.
  const file = new File(path, 'w');
  const data_u8 = new TextEncoder(encoding).encode(data);

  // Open file, write data, and close it.
  file.openSync();
  file.writeSync(data_u8);
  file.closeSync();
}

/**
 * Copies asynchronously a file from the source path to destination path.
 *
 * @param {String} path
 * @param {String|Uint8Array} data
 * @param {String} encoding
 */

export async function copyFile(source, destination) {
  // Check the source argument type.
  if (typeof source !== 'string') {
    throw new TypeError(`The "source" argument must be of type string.`);
  }

  // Check the destination argument type.
  if (typeof destination !== 'string') {
    throw new TypeError(`The "destination" argument must be of type string.`);
  }

  return writeFile(destination, await readFile(source));
}

/**
 * Copies synchronously a file from the source path to destination path.
 *
 * @param {String} path
 * @param {String|Uint8Array} data
 * @param {String} encoding
 */

export function copyFileSync(source, destination) {
  // Check the source argument type.
  if (typeof source !== 'string') {
    throw new TypeError(`The "source" argument must be of type string.`);
  }

  // Check the destination argument type.
  if (typeof destination !== 'string') {
    throw new TypeError(`The "destination" argument must be of type string.`);
  }

  return writeFileSync(destination, readFileSync(source));
}

/**
 * Retrieves asynchronously statistics for the file.
 *
 * @param {String} path
 */
export async function stat(path) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Get path statistics.
  const stats = await binding.stat(path);

  return stats;
}

/**
 * Retrieves synchronously statistics for the file.
 *
 * @param {String} path
 */
export function statSync(path) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Get path statistics.
  const stats = binding.statSync(path);

  return stats;
}

/**
 * Creates directories asynchronously.
 *
 * @param {String} path
 * @param {Object} options
 */
export async function mkdir(path, options = {}) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  await binding.mkdir(path, options?.recursive || false);
}

/**
 * Creates directories synchronously.
 *
 * @param {String} path
 * @param {Object} options
 */
export function mkdirSync(path, options = {}) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  binding.mkdirSync(path, options?.recursive || false);
}

/**
 * Removes empty directories asynchronously.
 *
 * @param {String} path
 * @param {Object} options
 */
export async function rmdir(path, options = {}, __retries = 0) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  const maxRetries = options?.maxRetries || 0;
  const retryDelay = options?.retryDelay || 100;

  try {
    // Try removing the empty directory.
    await binding.rmdir(path);
  } catch (err) {
    // If we maxed out the retries accept failure.
    if (__retries >= maxRetries) throw err;

    // Note: Wrapping the setTimeout into a promise is necessary otherwise the
    // outer rmdir call won't wait for all the inner ones.
    await new Promise((success, failure) => {
      // Back-off and retry later.
      setTimeout(
        () =>
          rmdir(path, options, __retries + 1)
            .then(success)
            .catch(failure),
        retryDelay
      );
    });
  }
}

/**
 * Removes empty directories synchronously.
 *
 * @param {String} path
 * @param {Object} options
 */
export function rmdirSync(path, options = {}, __retries = 0) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  const maxRetries = options?.maxRetries || 0;
  const retryDelay = options?.retryDelay || 100;

  try {
    // Try removing the empty directory.
    binding.rmdirSync(path);
  } catch (err) {
    // If we maxed out the retries accept failure.
    if (__retries >= maxRetries) throw err;

    // Back-off and retry later.
    setTimeout(() => {
      rmdirSync(path, options, __retries + 1);
    }, retryDelay);
  }
}

export default {
  File,
  open,
  openSync,
  readFile,
  readFileSync,
  writeFile,
  writeFileSync,
  copyFile,
  copyFileSync,
  stat,
  statSync,
  mkdir,
  mkdirSync,
  rmdir,
  rmdirSync,
};
