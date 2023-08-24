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
   * @param {Uint8Array} buffer
   * @param {Number} offset
   * @returns {Promise<Number>}
   */
  async read(buffer, offset = 0) {
    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    // Provided buffers must be Uint8Arrays.
    if (!(buffer instanceof Uint8Array)) {
      throw new TypeError(`The "buffer" argument must be of type Uint8Array.`);
    }

    // Copy bytes into buffer and return bytes read.
    return binding.read(this._handle, buffer.buffer, offset);
  }

  /**
   * Reads synchronously some bytes from the file.
   *
   * @param {Uint8Array} buffer
   * @param {Number} offset
   * @returns {Number}
   */
  readSync(buffer, offset = 0) {
    // Check if the file is open.
    if (!this._handle) {
      throw new Error('The file is not open.');
    }

    // Provided buffers must be Uint8Arrays.
    if (!(buffer instanceof Uint8Array)) {
      throw new TypeError(`The "buffer" argument must be of type Uint8Array.`);
    }

    // Copy bytes into buffer and return bytes read.
    return binding.readSync(this._handle, buffer.buffer, offset);
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
   * The `File` instances are asynchronously iterable objects.
   */
  async *[Symbol.asyncIterator](signal) {
    // Close the file on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.close());

    let buffer = new Uint8Array(BUFFER_SIZE);
    let bytesRead = 0;
    let offset = 0;
    while ((bytesRead = await this.read(buffer, offset))) {
      if (bytesRead === 0) break;
      offset += bytesRead;
      yield buffer.subarray(0, bytesRead);
    }
  }

  /**
   * The `File` instances are iterable objects.
   */
  *[Symbol.iterator]() {
    let buffer = new Uint8Array(BUFFER_SIZE);
    let bytesRead = 0;
    let offset = 0;
    while ((bytesRead = this.readSync(buffer, offset))) {
      if (bytesRead === 0) break;
      offset += bytesRead;
      yield buffer.subarray(0, bytesRead);
    }
  }
}

function makeDeferredPromise() {
  // Extract the resolve method from the promise.
  const promiseExt = {};
  const promise = new Promise((resolve, reject) => {
    promiseExt.resolve = resolve;
    promiseExt.reject = reject;
  });

  return { promise, promiseExt };
}

/**
 * An async iterator yielding file-system events.
 */
class FsWatcher {
  #id;
  #pushQueue;
  #pullQueue;

  /**
   * Creates a new FsWatcher instance.
   *
   * @param {String} path
   * @param {Boolean} recursive
   * @returns {FsWatcher}
   */
  constructor(path, recursive = false) {
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#id = binding.watch(path, recursive, (event) =>
      this._asyncDispatch(event)
    );
  }

  /**
   * Stops watching the file system and closes the watcher resource.
   */
  close() {
    // Check if the resource id is not undefined.
    if (!this.#id) {
      throw new Error(`FsWatcher is not attached to a resource ID.`);
    }
    binding.unwatch(this.#id);

    this._asyncDispatch(null);
    this.#id = undefined;
  }

  _asyncDispatch(value) {
    if (this.#pullQueue.length === 0) {
      this.#pushQueue.push(value);
      return;
    }
    const promise = this.#pullQueue.shift();
    const action = value instanceof Error ? promise.reject : promise.resolve;
    action(value);
  }

  /**
   * Returns a promise which is fulfilled when a new FS event is available.
   *
   * @returns {Promise<object>}
   */
  _next() {
    // Check if the resource id is not undefined.
    if (!this.#id) {
      throw new Error(`FsWatcher is not attached to a resource ID.`);
    }

    // No available event yet.
    if (this.#pushQueue.length === 0) {
      const { promise, promiseExt } = makeDeferredPromise();
      this.#pullQueue.push(promiseExt);
      return promise;
    }

    const value = this.#pushQueue.shift();
    const action = value instanceof Error ? Promise.reject : Promise.resolve;

    return action.call(Promise, value);
  }

  /**
   * The FsWatcher should be async iterable.
   */
  async *[Symbol.asyncIterator](signal) {
    // Close watcher on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.close());

    let data;
    while ((data = await this._next())) {
      if (!data) break;
      yield data;
    }
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
 * @param {(String|Object)} options
 * @returns {Promise<(String|Uint8Array)>}
 */

export async function readFile(path, options = {}) {
  // Create a new file instance.
  const file = new File(path, 'r');
  await file.open();

  // Allocate a buffer to store all the bytes from the file.
  const stat = await file.stat();
  const data = new Uint8Array(stat.size);

  let bytesRead = 0;

  // Note: Since the file object is async iterable will read the entire content
  // of the file using the for-await loop.
  for await (let chunk of file) {
    data.set(chunk, bytesRead);
    bytesRead += chunk.length;
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
 * @param {(String|Object)} options
 * @returns {(String|Uint8Array)}
 */

export function readFileSync(path, options = {}) {
  // Create a new file instance.
  const file = new File(path, 'r');
  file.openSync();

  // Allocate a buffer to store all the bytes from the file.
  const stat = file.statSync();
  const data = new Uint8Array(stat.size);

  let bytesRead = 0;

  // Note: Since the file object is iterable will read the entire content
  // of the file using the for-of loop.
  for (let chunk of file) {
    data.set(chunk, bytesRead);
    bytesRead += chunk.length;
  }

  file.closeSync();

  // Decode given an encoder.
  const encoding = typeof options === 'string' ? options : options.encoding;

  if (encoding) {
    return new TextDecoder(encoding).decode(data);
  }

  return data;
}

function toUint8Array(data, encoding) {
  if (!(data instanceof Uint8Array)) {
    return new TextEncoder(encoding).encode(data);
  }
  return data;
}

/**
 * Writes asynchronously contents to a file.
 *
 * @param {String} path
 * @param {String|Uint8Array} data
 * @param {String|Object} options
 * @returns {Promise}
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

  const data_u8 = toUint8Array(data, encoding);

  // Create a file instance.
  const file = new File(path, 'w');

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

  const data_u8 = toUint8Array(data, encoding);

  // Create a file instance.
  const file = new File(path, 'w');

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
 * @returns {Promise}
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
 * @returns {Promise<Object>}
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
 * @returns {Object}
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
 * @returns {Promise}
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
 * @returns {Promise}
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

/**
 * Reads asynchronously the contents of a directory.
 *
 * @param {String} path
 * @returns {Promise<String[]>}
 */
export async function readdir(path) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  return binding.readdir(path);
}

/**
 * Reads the contents of a directory.
 *
 * @param {String} path
 * @returns {String[]}
 */
export function readdirSync(path) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  return binding.readdirSync(path);
}

/**
 * Removes files and directories asynchronously.
 *
 * @param {String} path
 * @param {Object} options
 * @returns {Promise}
 */
export async function rm(path, options = {}, __retries = 0) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Set default options if not specified.
  const recursive = options?.recursive || false;
  const maxRetries = options?.maxRetries || 0;
  const retryDelay = options?.retryDelay || 100;

  // Get path's statistics.
  const pathStat = await stat(path);

  if (pathStat.isDirectory && !recursive) {
    await rmdir(path, options);
    return;
  }

  try {
    // Try removing file or directory.
    await binding.rm(path);
  } catch (err) {
    // If we maxed out the retries accept failure.
    if (__retries >= maxRetries) throw err;

    // Note: Wrapping the setTimeout into a promise is necessary otherwise the
    // outer rm call won't wait for all the inner ones.
    await new Promise((success, failure) => {
      // Back-off and retry later.
      setTimeout(
        () =>
          rm(path, options, __retries + 1)
            .then(success)
            .catch(failure),
        retryDelay
      );
    });
  }
}

/**
 * Removes files and directories synchronously.
 *
 * @param {String} path
 * @param {Object} options
 */
export function rmSync(path, options = {}, __retries = 0) {
  // Check the path argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  // Set default options if not specified.
  const recursive = options?.recursive || false;
  const maxRetries = options?.maxRetries || 0;
  const retryDelay = options?.retryDelay || 100;

  // Get path's statistics.
  const pathStat = statSync(path);

  if (pathStat.isDirectory && !recursive) {
    rmdirSync(path, options);
    return;
  }

  try {
    // Try removing file or directory.
    binding.rmSync(path);
  } catch (err) {
    // If we maxed out the retries accept failure.
    if (__retries >= maxRetries) throw err;

    // Back-off and retry later.
    setTimeout(() => {
      rmSync(path, options, __retries + 1);
    }, retryDelay);
  }
}

/**
 * Renames oldPath to newPath asynchronously.
 *
 * @param {String} from
 * @param {String} to
 * @returns {Promise<undefined>}
 */
export async function rename(from, to) {
  // Check the `from` argument type.
  if (typeof from !== 'string') {
    throw new TypeError('The "from" argument must be of type string.');
  }

  // Check the `to` argument type.
  if (typeof to !== 'string') {
    throw new TypeError('The "to" argument must be of type string.');
  }

  return binding.rename(from, to);
}

/**
 * Renames oldPath to newPath synchronously.
 *
 * @param {String} from
 * @param {String} to
 */
export function renameSync(from, to) {
  // Check the `from` argument type.
  if (typeof from !== 'string') {
    throw new TypeError('The "from" argument must be of type string.');
  }

  // Check the `to` argument type.
  if (typeof to !== 'string') {
    throw new TypeError('The "to" argument must be of type string.');
  }

  binding.renameSync(from, to);
}

/**
 * Returns an async iterator that watches for changes over a path.
 *
 * @param {String} path
 * @param {Object} options
 * @returns {FsWatcher}
 */
export function watch(path, options = {}) {
  // Check the `path` argument type.
  if (typeof path !== 'string') {
    throw new TypeError('The "path" argument must be of type string.');
  }

  return new FsWatcher(path, options.recursive);
}

/**
 * Returns a new readable IO stream.
 *
 * @param {String} path
 * @param {(String|Object)} options
 * @returns {AsyncGeneratorFunction}
 */
export function createReadStream(path, options = {}) {
  // Use passed encoding or default to UTF-8.
  const encoding = typeof options === 'string' ? options : options.encoding;
  const textDecoder = new TextDecoder(encoding || 'utf-8');

  // Create the async generator.
  return async function* readStream(signal) {
    // Open file and handle broken pipeline clean-ups.
    const file = await open(path, options?.mode);
    signal.on('uncaughtStreamException', () => file.close());
    // Start consuming chunks.
    for await (const chunk of file) {
      yield encoding ? textDecoder.decode(chunk) : chunk;
    }
    file.close();
  };
}

/**
 * Returns a new writable IO stream.
 *
 * @param {String} path
 * @param {(String|Object)} options
 * @returns {Object}
 */
export function createWriteStream(path, options = {}) {
  // We want to open the file the moment the stream becomes active.
  let _handle;
  const encoding = typeof options === 'string' ? options : options.encoding;

  // Every object with `.write()` and `.end()` is a writable stream.
  return {
    async write(chunk) {
      if (!_handle) _handle = await open(path, options.mode || 'w');
      const data = toUint8Array(chunk, encoding || 'utf-8');
      await _handle.write(data);
    },
    async end(chunk) {
      if (chunk) await this.write(chunk);
      if (_handle) await _handle.close();
    },
  };
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
  readdir,
  readdirSync,
  rm,
  rmSync,
  rename,
  renameSync,
  watch,
  createReadStream,
  createWriteStream,
};
