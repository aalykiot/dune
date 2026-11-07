/**
 * TTY APIs
 *
 * The TTY module provides the ReadStream and WriteStream classes. In most cases,
 * however, it is not necessary to use this module directly.
 *
 * @see {@link https://nodejs.org/dist/latest-v18.x/docs/api/tty.html}
 *
 * @module TTY
 */

import assert from 'assert';
import { EventEmitter } from 'events';
import { makeDeferredPromise, toUint8Array } from 'util';

const binding = process.binding('tty');
const kAsyncGenerator = Symbol('kAsyncGenerator');

const CSI = '\x1b[';
const CLEAR_TO_LINE_BEGINNING = '\x1b[1K';
const CLEAR_TO_LINE_END = '\x1b[0K';
const CLEAR_LINE = '\x1b[2K';
const CLEAR_SCREEN_DOWN = '\x1b[0J';

/**
 * Returns true if the given fd is associated with a TTY and false if it is not.
 *
 * @param {number} fd - File descriptor
 * @returns {boolean}
 */
export function isatty(fd) {
  assert.integer(fd);
  return binding.isTTY(fd);
}

export class WriteStream extends EventEmitter {
  #tty;
  #encoding;
  #writable;

  /**
   * Creates a new TTY write stream.
   *
   * @param {number} fd - The file descriptor to write to.
   * @returns {WriteStream}
   */
  constructor(fd) {
    super();
    assert.integer(fd, 'The "fd" must be a positive integer.');
    assert.greaterThanOrEqual(0, fd, 'The "fd" must be a positive integer.');

    this.#tty = binding.tty(fd);
    this.#writable = true;
    this.isTTY = true;
    this.fd = fd;

    this.#refreshSize();

    process.on('SIGWINCH', () => this.#refreshSize());
  }

  /**
   * Writes contents to the TTY stream.
   *
   * @param {String|Uint8Array} data - The data to be written to the TTY.
   * @param {String} [encoding] - The character encoding to use.
   */
  write(data, encoding) {
    // Check the data argument type.
    if (!(data instanceof Uint8Array) && typeof data !== 'string') {
      throw new TypeError(
        'The "data" argument must be of type string or Uint8Array.'
      );
    }

    if (!this.#writable) {
      throw new Error('The stream has already been closed.');
    }

    // Default to UTF-8 encoding.
    encoding = encoding || this.#encoding || 'utf-8';

    binding.write(this.#tty, toUint8Array(data, encoding));
  }

  /**
   * Marks the TTY stream as no longer writable.
   *
   * @param {String|Uint8Array} data - The last data to be written to the TTY stream.
   * @param {String} [encoding] - The character encoding to use.
   */
  end(data, encoding) {
    if (data) this.write(data, encoding);
    this.#writable = false;
  }

  /**
   * Sets the encoding for the TTY write stream.
   *
   * @param {String} [encoding] - The character encoding to use.
   */
  setEncoding(encoding = 'utf-8') {
    assert.string(encoding, 'The "encoding" argument must be of type string.');
    this.#encoding = encoding;
  }

  /**
   * Clears the current line in this WriteStream.
   *
   *  @param {number} [direction=1] - Which part of the line to clear:
   *  `-1` clears to the left of the cursor, `1` clears to the right,
   *  and `0` clears the entire line.
   */
  clearLine(direction) {
    // Validate the type of the parameter.
    assert.integer(direction, 'The "direction" argument must be an integer.');

    if (direction < 0) return this.write(CLEAR_TO_LINE_BEGINNING);
    if (direction > 0) return this.write(CLEAR_TO_LINE_END);

    this.write(CLEAR_LINE);
  }

  /**
   * Clears the WriteStream downward from the cursor.
   */
  clearScreenDown() {
    this.write(CLEAR_SCREEN_DOWN);
  }

  /**
   * Moves this WriteStream's cursor to the specified position.
   *
   * @param {number} x - The horizontal position (column).
   * @param {number} [y] - The vertical position (row). If omitted, only the horizontal position is changed.
   */
  cursorTo(x, y) {
    // The X coordinate must aleays be defined.
    assert.integer(x, 'The "x" argument must be an integer.');
    const sequense =
      typeof y === 'number' ? `${CSI}${y + 1};${x + 1}H` : `${CSI}${x + 1}G`;

    this.write(sequense);
  }

  /**
   * Moves this WriteStream's cursor relative to its current position.
   *
   * @param {number} dx - The horizontal displacement.
   * @param {number} dy - The vertical displacement.
   */
  moveCursor(dx, dy) {
    // Dx and Dy must always be defined.
    assert.integer(dx, 'The "dx" argument must be an integer.');
    assert.integer(dy, 'The "dy" argument must be an integer.');

    let sequense = '';

    if (dx < 0) sequense += `${CSI}${-dx}D`;
    if (dx > 0) sequense += `${CSI}${dx}C`;
    if (dy < 0) sequense += `${CSI}${-dy}A`;
    if (dy > 0) sequense += `${CSI}${dy}B`;

    this.write(sequense);
  }

  /**
   * Returns the size of the TTY.
   *
   * @returns {number[]} An array containing 2 numbers [numColumns, numRows].
   */
  getWindowSize() {
    return binding.getWindowSize(this.#tty);
  }

  #refreshSize() {
    const [columns, rows] = this.getWindowSize();

    this.columns = columns;
    this.rows = rows;
    this.emit('resize');
  }
}

export class ReadStream extends EventEmitter {
  #tty;
  #pushQueue;
  #pullQueue;
  #encoding;
  #active;

  /**
   * Creates a new TTY read stream.
   *
   * @param {number} fd - The file descriptor to read from.
   * @returns {ReadStream}
   */
  constructor(fd) {
    super();
    assert.integer(fd, 'The "fd" must be a positive integer.');
    assert.greaterThanOrEqual(0, fd, 'The "fd" must be a positive integer.');

    this.#tty = binding.tty(fd);
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#active = false;
    this.fd = fd;
    this.isTTY = true;
    this.isRaw = false;
  }

  /**
   * Returns a promise which is fulfilled when stdin can return a chunk.
   *
   * @returns {Promise<(Uint8Array|string)>} The chunk read from stdin.
   */
  read() {
    // Check if a TTY is attached to the stream.
    if (!this.#tty) return null;

    // Start reading from the TTY if the stream is not yet active.
    if (!this.#active) {
      binding.readStart(this.#tty, this.#onAvailableTTYData.bind(this));
      this.#active = true;
    }

    // HACK: The following is used to handle uncaught errors thrown
    // from the event-emitter when no one is subscribed to the `error` event.
    if (this.listenerCount('error') === 0) this.on('error', () => {});

    // No available value to read yet.
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
   * Configures tty.ReadStream to operate in raw mode.
   *
   * @param {boolean} [mode] -  Whether to enable raw mode. Defaults to false.
   */
  setRawMode(mode = false) {
    // Check the parameter type.
    if (typeof mode !== 'boolean') {
      throw new TypeError('The "mode" argument must be of type boolean.');
    }
    binding.setRawMode(this.#tty, mode);
    this.isRaw = mode;
  }

  /**
   * Sets the encoding for the TTY read stream.
   *
   * @param {String} [encoding] - The character encoding to use.
   */
  setEncoding(encoding = 'utf-8') {
    // Check the parameter type.
    if (typeof encoding !== 'string') {
      throw new TypeError('The "encoding" argument must be of type string.');
    }
    this.#encoding = encoding;
  }

  /**
   * Stops the TTY stream from reading input.
   */
  destroy() {
    // Check whether the TTY stream is valid and, if active,
    // stop reading from it.
    if (!this.#tty) return;
    if (this.#active) binding.readStop(this.#tty);

    // Ignore pending reads.
    for (const promise of this.#pullQueue) {
      promise.resolve(null);
    }

    this.#tty = undefined;
    this.#active = false;

    this.emit('close');
  }

  #asyncDispatch(value) {
    if (this.#pullQueue.length === 0) {
      this.#pushQueue.push(value);
      return;
    }
    const promise = this.#pullQueue.shift();
    const action = value instanceof Error ? promise.reject : promise.resolve;
    action(value);
  }

  #onAvailableTTYData(err, arrayBufferView) {
    // Check for errors during TTY read.
    if (err) {
      this.#asyncDispatch(err);
      this.emit('error', err);
      return;
    }

    // Transform ArrayBuffer into a Uint8Array we can use.
    const data = new Uint8Array(arrayBufferView);
    const transformed = this.#encoding
      ? new TextDecoder(this.#encoding).decode(new Uint8Array(data))
      : data;

    // Use the EE mode instead of the async-iterator.
    if (this.listenerCount('data') > 0) {
      this.emit('data', transformed);
      return;
    }

    this.#asyncDispatch(transformed);
  }

  async *[kAsyncGenerator](signal) {
    // Close socket on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.destroy());

    let data;
    while ((data = await this.read())) {
      if (!data) break;
      yield data;
    }
  }

  /**
   * The TTY stream should be async iterable.
   * @ignore
   */
  [Symbol.asyncIterator](signal) {
    return Object.assign(this[kAsyncGenerator](signal), {
      return: () => {
        this.destroy();
        return Promise.resolve({ done: true });
      },
    });
  }
}

export default {
  isatty,
  WriteStream,
  ReadStream,
};
