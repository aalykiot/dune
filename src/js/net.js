// TCP Networking APIs
//
// The TCP Networking APIs provide an asynchronous network API for creating
// stream-based TCP servers and clients.
//
// https://nodejs.org/dist/latest-v18.x/docs/api/net.html

import dns from 'dns';
import assert from 'assert';
import { EventEmitter } from 'events';

const binding = process.binding('net');

/**
 * A Socket object is a wrapper for a low-level TCP stream.
 */
export class Socket extends EventEmitter {
  /**
   * Creates a new Socket instance.
   *
   * @returns {Socket}
   */
  constructor() {
    super();
    this._rid = undefined;
    this._connecting = false;
    this._reading = false;
    this._encoding = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }

  /**
   * Initiates a connection on a given remote host.
   *
   * @param {Number} port
   * @param {String} [host]
   * @param {Function} [onConnection]
   */
  connect(port, hostname = '127.0.0.1', onConnection) {
    // Check the port parameter type.
    if (Number.isNaN(Number.parseInt(port))) {
      throw new TypeError(`The "port" argument must be castable to number.`);
    }

    // Check the host parameter type.
    if (hostname && typeof hostname !== 'string') {
      throw new TypeError(`The "host" argument must be of type string.`);
    }

    // Check if socket is already connected.
    if (this._rid) {
      throw new Error(
        `Socket is already connected to <${this.remoteAddress}:${this.remotePort}>.`
      );
    }

    // Check if a connection is happening.
    if (this._connecting) {
      throw new Error('Socket is trying to connect.');
    }

    // Subscribe to the emitter the on-connect callback if specified.
    if (onConnection) {
      assert.isFunction(onConnection);
      this.on('connect', onConnection);
    }

    const socketOnRead = (err, arrayBufferView) => {
      // Use event-emitter to throw reading errors (if registered).
      if (err && this.listenerCount('error') !== 0) {
        return this.emit('error', err);
      }

      if (err) throw err;

      // Check if the remote host closed the connection.
      if (arrayBufferView.byteLength === 0) {
        this.destroy();
        return this.emit('end');
      }

      this.bytesRead += arrayBufferView.byteLength;

      // Transform ArrayBuffer into a Uint8Array we can use.
      const data = new Uint8Array(arrayBufferView);
      const data_transform = !this._encoding
        ? data
        : new TextDecoder(this._encoding).decode(new Uint8Array(data));

      this.emit('data', data_transform);
    };

    // Note: We're wrapping the connection logic inside an async function
    // since the async/await syntax makes the code more readable.
    const try_connect = async () => {
      try {
        // Use DNS lookup to resolve the hostname.
        const addresses = await dns.lookup(hostname);

        // Prefer IPv4 address.
        const host = addresses.some((addr) => addr.family === 'IPv4')
          ? addresses.filter((addr) => addr.family === 'IPv4')[0].address
          : addresses[0].address;

        // Try to connect to the remote host.
        const socketInfo = await binding.connect(host, port);

        // Update socket's local state.
        this._rid = socketInfo.rid;
        this._connecting = false;
        this.remoteAddress = socketInfo.remoteAddress;
        this.remotePort = socketInfo.remotePort;

        this.emit('connect', socketInfo);
        binding.readStart(this._rid, socketOnRead);
      } catch (err) {
        // Use event-emitter to throw connection errors (if registered).
        if (this.listenerCount('error') > 0) {
          return this.emit('error', err);
        }
        throw err;
      }
    };

    this._connecting = true;
    try_connect();

    return this;
  }

  /**
   * Sets the encoding for the current socket.
   *
   * @param {String} [encoding]
   */
  setEncoding(encoding) {
    // Check the parameter type.
    if (typeof encoding !== 'string') {
      throw new TypeError('The "encoding" argument must be of type string.');
    }
    this._encoding = encoding;
  }

  /**
   * Writes contents to a TCP socket stream.
   *
   * @param {String|Uint8Array} data
   * @param {Function} onWrite
   */
  write(data, onWrite) {
    // Check the data argument type.
    if (!(data instanceof Uint8Array) && typeof data !== 'string') {
      throw new TypeError(
        `The "data" argument must be of type string or Uint8Array.`
      );
    }

    // Check the type of the onWrite param.
    if (onWrite) {
      assert.isFunction(onWrite);
    }

    // Check if the socket is connected.
    if (!this._rid) {
      throw new Error(`Socket is not connected to a remote host.`);
    }

    // Default to UTF-8.
    const encoding = this._encoding || 'utf-8';

    const try_write = async () => {
      try {
        // Try write some bytes.
        const bytes = await binding.write(
          this._rid,
          data instanceof Uint8Array
            ? data
            : new TextEncoder(encoding).encode(data)
        );

        this.bytesWritten += bytes;

        // Run callback if specified.
        if (onWrite) {
          onWrite(this.bytesWritten);
        }
      } catch (err) {
        // Use event-emitter to throw connection errors (if registered).
        if (this.listenerCount('error') > 0) {
          return this.emit('error', err);
        }
        throw err;
      }
    };

    try_write();
  }

  /**
   * Closes both sides of the TCP sockets.
   */
  destroy() {
    // Check if the socket is indeed connected.
    if (!this._rid) {
      throw new Error('Socket is not connected to a remote host.');
    }
    // Close the socket.
    binding.close(this._rid).then(() => {
      this.emit('close');
      this._reset();
    });
  }

  /**
   * Resets socket's internal state (not to be called manually).
   */
  _reset() {
    this._rid = undefined;
    this._connecting = false;
    this._reading = false;
    this._encoding = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }
}

/**
 * Initiates a connection on a given remote host.
 *
 * @param {Number} port
 * @param {String} [host]
 * @param {Function} [onConnection]
 */
export function createConnection(port, hostname, onConnection) {
  const socket = new Socket();
  socket.connect(port, hostname, onConnection);
  return socket;
}

export default {
  Socket,
  createConnection,
};
