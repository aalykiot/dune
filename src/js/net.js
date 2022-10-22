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

function parseOptionsArgs(args) {
  // Use options overloading.
  if (typeof args[0] === 'object') {
    return [args[0]?.port, args[0]?.host, args[1]];
  }
  return args;
}

function toUint8Array(data, encoding) {
  if (!(data instanceof Uint8Array)) {
    return new TextEncoder(encoding).encode(data);
  }
  return data;
}

function makeDeferredPromise() {
  // Extract the resolve method from the promise.
  const promiseExt = {};
  const promise = new Promise((r) => (promiseExt.resolve = r));
  // Attach it to the promise.
  promise.resolve = promiseExt.resolve;
  return promise;
}

/**
 * Initiates a connection to a given remote host.
 *
 * @param {Object} options
 * @param {Function} [onConnection]
 * @returns Socket
 */
export function createConnection(...args) {
  const socket = new Socket();
  socket.connect(...args);
  return socket;
}

/**
 * Creates a new TCP server.
 *
 * @param {Function} [onConnection]
 * @returns Server
 */
export function createServer(onConnection) {
  // Create the server instance.
  const server = new Server();
  // Check onConnection callback.
  if (onConnection) {
    assert.isFunction(onConnection);
    server.on('connection', onConnection);
  }
  return server;
}

/**
 * A Socket object is a JS wrapper around a low-level TCP socket.
 */
export class Socket extends EventEmitter {
  #id;
  #host;
  #connecting;
  #encoding;
  #writable;

  /**
   * Creates a new Socket instance.
   *
   * @returns {Socket}
   */
  constructor() {
    super();
    this.#connecting = 0;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }

  /**
   * Initiates a connection on a given remote host.
   *
   * @param  {...any} args
   * @returns {Promise<*>}
   */
  async connect(...args) {
    // Parse arguments.
    const [port, hostUnchecked, onConnection] = parseOptionsArgs(args);
    const hostname = hostUnchecked || '0.0.0.0';
    this.#connecting += 1;

    // Check the port parameter type.
    if (Number.isNaN(Number.parseInt(port))) {
      throw new TypeError(`The "port" option must be castable to number.`);
    }

    // Check the host parameter type.
    if (hostname && typeof hostname !== 'string') {
      throw new TypeError(`The "host" option must be of type string.`);
    }

    // Check if socket is already connected.
    if (this.#id) {
      throw new Error(
        `Socket is already connected to <${this.remoteAddress}:${this.remotePort}>.`
      );
    }

    // Check if a connection is happening.
    if (this.#connecting > 1) {
      throw new Error('Socket is trying to connect.');
    }

    // Subscribe to the emitter, the on-connect callback if specified.
    if (onConnection) {
      assert.isFunction(onConnection);
      this.on('connect', onConnection);
    }

    try {
      // Use DNS lookup to resolve the hostname.
      const addresses = await dns.lookup(hostname);

      // Prefer IPv4 address.
      const host = addresses.some((addr) => addr.family === 'IPv4')
        ? addresses.filter((addr) => addr.family === 'IPv4')[0].address
        : addresses[0].address;

      // Try to connect to the remote host.
      const socketInfo = await binding.connect(host, port);

      this.#id = socketInfo.id;
      this.#connecting -= 1;
      this.#writable = true;
      this.#host = socketInfo.host;
      this.remoteAddress = socketInfo.remote.address;
      this.remotePort = socketInfo.remote.port;

      this.emit('connect', socketInfo);
      binding.readStart(this.#id, this._onSocketRead.bind(this));

      return socketInfo;
    } catch (err) {
      this.emit('error', err);
    }
  }

  /**
   * Returns the bound address, the address family name and port of the socket.
   *
   * @returns {Object}
   */
  address() {
    return this.#host;
  }

  _onSocketRead(err, arrayBufferView) {
    // Check for read errors.
    if (err) {
      this.emit('error', err);
      return;
    }

    // Check if the remote host closed the connection.
    if (arrayBufferView.byteLength === 0) {
      this.destroy();
      return this.emit('end');
    }

    this.bytesRead += arrayBufferView.byteLength;

    // Transform ArrayBuffer into a Uint8Array we can use.
    const data = new Uint8Array(arrayBufferView);
    const data_transform = this.#encoding
      ? new TextDecoder(this.#encoding).decode(new Uint8Array(data))
      : data;

    this.emit('data', data_transform);
  }

  /**
   * Sets the encoding for the current socket.
   *
   * @param {String} [encoding]
   */
  setEncoding(encoding = 'utf-8') {
    // Check the parameter type.
    if (typeof encoding !== 'string') {
      throw new TypeError('The "encoding" argument must be of type string.');
    }
    this.#encoding = encoding;
  }

  /**
   * Writes contents to a TCP socket stream.
   *
   * @param {String|Uint8Array} data
   * @param {String} [encoding]
   * @param {Function} [onWrite]
   * @returns {Promise<Number>}
   */
  async write(data, encoding, onWrite) {
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
    if (!this.#id) {
      throw new Error(`Socket is not connected to a remote host.`);
    }

    // Check if socket is half-closed.
    if (!this.#writable) {
      throw new Error(`The socket stream is not writable.`);
    }

    // Default tu UTF-8 encoding.
    encoding = encoding || this.#encoding || 'utf-8';

    const bytes = toUint8Array(data, encoding);
    const bytesWritten = await binding.write(this.#id, bytes);

    this.bytesWritten += bytesWritten;

    if (onWrite) onWrite(bytesWritten);

    return bytesWritten;
  }

  /**
   * Half-closes the TCP stream.
   *
   * @param {String|Uint8Array} data
   * @param {String} [encoding]
   * @returns {Promise<*>}
   */
  async end(data, encoding = 'utf-8') {
    // Check socket connection.
    if (!this.#id) {
      throw new Error(`Socket is not connected to a remote host.`);
    }
    // If data is specified, write it to stream.
    if (data) {
      await this.write(data, encoding);
    }
    await binding.shutdown(this.#id);
    this.#writable = false;
  }

  /**
   * Closes both sides of the TCP sockets.
   */
  async destroy() {
    // Check if the socket is indeed connected.
    if (!this.#id) {
      throw new Error('Socket is not connected to a remote host.');
    }
    // Close the socket.
    await binding.close(this.#id);
    this.emit('close');
    this._reset();
  }

  /**
   * Resets socket's internal state (not to be called manually).
   */
  _reset() {
    this.#id = undefined;
    this.#connecting = 0;
    this.#writable = false;
    this.#encoding = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }

  /**
   * Hard sets the ID of the socket (ONLY for internal use).
   *
   * @param {Number} id
   */
  _set_socket_id_unchecked(id) {
    this.#id = id;
    this.#writable = true;
    binding.readStart(this.#id, this._onSocketRead.bind(this));
  }

  /**
   * Socket should be an async iterator.
   */
  async *[Symbol.asyncIterator]() {
    const queue = [makeDeferredPromise()];
    let done = false;
    let idx = 0;

    this.on('data', (data) => {
      queue[idx].resolve(data);
      const promise = makeDeferredPromise();
      idx++;
      queue.push(promise);
    });

    this.on('end', () => (done = true));

    while (!done) {
      const data = await queue[0];
      queue.shift();
      idx--;
      yield data;
    }
  }
}

/**
 * A Server object is a wrapper around a TCP listener.
 */
export class Server extends EventEmitter {
  #id;
  #host;
  #connections;

  /**
   * Creates a new Server instance.
   *
   * @returns {Server}
   */
  constructor() {
    super();
    this.#connections = 0;
  }

  /**
   * Starts listening for incoming connections.
   *
   * @param  {...any} args
   * @returns Promise<undefined>
   */
  async listen(...args) {
    // Parse arguments.
    const [port, hostUnchecked, onListening] = parseOptionsArgs(args);
    const hostname = hostUnchecked || '127.0.0.1';

    // Check the port parameter type.
    if (Number.isNaN(Number.parseInt(port))) {
      throw new TypeError(`The "port" option must be castable to number.`);
    }

    // Check the host parameter type.
    if (hostname && typeof hostname !== 'string') {
      throw new TypeError(`The "host" option must be of type string.`);
    }

    // Check if the server already is on listening state.
    if (this.#id) {
      throw new Error(`Server is already listening for connections.`);
    }

    // Subscribe to the emitter, the on-connect callback if specified.
    if (onListening) {
      assert.isFunction(onListening);
      this.on('listening', onListening);
    }

    try {
      // Use DNS lookup to resolve the local listening interface.
      const addresses = await dns.lookup(hostname);

      // Prefer IPv4 address.
      const host = addresses.some((addr) => addr.family === 'IPv4')
        ? addresses.filter((addr) => addr.family === 'IPv4')[0].address
        : addresses[0].address;

      // Bind server to address, and start listening for connections.
      const socketInfo = binding.listen(
        host,
        port,
        this._onNewConnection.bind(this)
      );

      // Update internal state.
      this.#id = socketInfo.id;
      this.#host = socketInfo.host;

      // Everything OK, emit the listening event.
      this.emit('listening', this.#host);
    } catch (err) {
      this.emit('error', err);
    }
  }

  /**
   * Returns the number of concurrent connections on the server.
   *
   * @returns Number
   */
  getConnections() {
    return this.#connections;
  }

  /**
   * Returns the bound address, the address family name and port of the socket.
   *
   * @returns {Object}
   */
  address() {
    return this.#host;
  }

  /**
   * Stops the server from accepting new connections.
   *
   * @param {Function} [onClose]
   */
  async close(onClose) {
    // Check if the server is already closed.
    if (!this.#id) {
      throw new Error('Server is already closed.');
    }

    // Check the type of onClose.
    if (onClose) {
      assert.isFunction(onClose);
      this.once('close', () => onClose());
    }

    await binding.close(this.#id);

    this.emit('close');
  }

  _onNewConnection(err, sockInfo) {
    // Check for socket connection errors.
    if (err) {
      this.emit('error', err);
      return;
    }

    // Create a new socket instance.
    const socket = new Socket();
    socket._set_socket_id_unchecked(sockInfo.id);
    socket.remoteAddress = sockInfo.remoteAddress;
    socket.remotePort = sockInfo.remotePort;

    // Update active concurrent connections when socket is closed.
    socket.on('close', () => {
      this.#connections--;
    });

    // Update active concurrent connections.
    this.#connections++;

    // Notify listeners for the new connection.
    this.emit('connection', socket);
  }

  /**
   * Server should be an async iterator.
   */
  async *[Symbol.asyncIterator]() {
    const queue = [makeDeferredPromise()];
    let done = false;
    let idx = 0;

    this.on('connection', (socket) => {
      queue[idx].resolve(socket);
      const promise = makeDeferredPromise();
      idx++;
      queue.push(promise);
    });

    this.on('close', () => (done = true));

    while (!done) {
      const data = await queue[0];
      queue.shift();
      idx--;
      yield data;
    }
  }
}

export default {
  Socket,
  createConnection,
  Server,
  createServer,
};
