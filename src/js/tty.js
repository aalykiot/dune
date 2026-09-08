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
import { makeDeferredPromise } from 'util';

const binding = process.binding('tty');
const kAsyncGenerator = Symbol('kAsyncGenerator');

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

export class ReadStream extends EventEmitter {
  #tty;
  #pushQueue;
  #pullQueue;
  #encoding;
  #active;

  /**
   * Creates a new TTY read stream.
   *
   * @returns {ReadStream}
   */
  constructor() {
    super();
    this.#tty = binding.tty();
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#active = false;
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
    if (signal)
      signal.on('uncaughtStreamException', () => {
        /** TODO: close the TTY stream */
      });

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
    const iterator = {
      return: () => {
        /** TODO: close the TTY stream */
      },
    };
    return Object.assign(this[kAsyncGenerator](signal), iterator);
  }
}

export default {
  isatty,
  ReadStream,
};
