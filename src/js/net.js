/**
 * TCP Networking APIs
 *
 * The TCP Networking APIs provide an asynchronous network API for creating
 * stream-based TCP servers and clients.
 *
 * @see {@link https://nodejs.org/dist/latest-v18.x/docs/api/net.html}
 *
 * @module Net
 */

import dns from 'dns';
import assert from 'assert';
import { EventEmitter } from 'events';

const binding = process.binding('net');

function parseOptionsArgs(args) {
  // Use options overloading.
  if (typeof args[0] === 'object') {
    return [args[0]?.port, args[0]?.host];
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
  const promise = new Promise((resolve, reject) => {
    promiseExt.resolve = resolve;
    promiseExt.reject = reject;
  });

  return { promise, promiseExt };
}

const TIMEOUT_MAX = Math.pow(2, 31) - 1;

// Error type referring to socket connection timeout.
export class TimeoutError extends Error {
  constructor(message) {
    super();
    this.name = 'TimeoutError';
    this.message = message;
  }
}

// Utility function that wraps a promise with a timeout.
function timeout(promise, time = 0) {
  // When the time is 0ms it means that we don't want to
  // have a timeout for the provided promise.
  if (time === 0) return promise;

  const timer = {};
  return Promise.race([
    promise,
    new Promise(
      (_, reject) => (timer.id = setTimeout(reject, time, new TimeoutError()))
    ),
  ]).finally(() => clearTimeout(timer.id));
}

/**
 *  Utility function that wraps a `repeatable` callback with a timeout.
 *
 *  The following implementation does not create a new timer every time an
 *  I/O activity happens on the socket. The logic is a bit more complex
 *  but it is more performant on high I/O scenarios.
 *
 *  http://pod.tst.eu/http://cvs.schmorp.de/libev/ev.pod#Be_smart_about_timeouts
 * @ignore
 */
function callbackTimeout(callback, time = 0, onTimeout) {
  // The reason of the event-emitter is to allow the "outside" world
  // to signal us for changes in the timeout value.
  const timer = { time, lastActivity: Date.now() };
  const timerSignal = new EventEmitter();

  const timeout = () => {
    // Calculate when the timeout would happen.
    const after = timer.lastActivity - Date.now() + timer.time;

    if (timer.time === 0) return;

    // Timeout occurred, take action.
    if (after < 0) {
      onTimeout();
      return (timer.id = setTimeout(timeout, timer.time));
    }

    // There was some recent activity, simply restart the timer.
    timer.id = setTimeout(timeout, after);
  };

  timerSignal.on('timeoutUpdate', (timeMs) => {
    if (timer.id) clearTimeout(timer.id);
    timer.time = timeMs;
    timeout();
  });

  if (time > 0) timer.id = setTimeout(timeout, time);

  return [
    (...args) => {
      callback(...args);
      timer.lastActivity = Date.now();
    },
    timerSignal,
  ];
}

/**
 * Initiates a connection to a given remote host.
 *
 * Additional signatures:
 * - createConnection(port: number | string, host?: string)
 *
 * @param {Object} options - Configuration options for the connection.
 * @param {string} options.host - The hostname or IP address of the remote server to connect to.
 * @param {(string|number)} options.port - The port number on the remote host to connect to.
 * @returns {Socket} An instance of the `Socket` class
 */
export function createConnection(...args) {
  const socket = new Socket();
  socket.connect(...args);
  return socket;
}

/**
 * Aliases to `net.createConnection()`.
 *
 * Additional signatures:
 * - connect(port: number | string, host?: string)
 */
export const connect = createConnection;

/**
 * Creates a new TCP server.
 *
 * @param {Function} [onConnection] - A function that is called whenever a connection is made to the server.
 * @returns {Server} An instance of the `Server` class.
 */
export function createServer(onConnection) {
  // Instantiate a new TCP server.
  const server = new Server();
  if (onConnection) {
    assert.isFunction(onConnection);
    server.on('connection', onConnection);
  }
  return server;
}

/**
 * Information about the connected TCP socket.
 *
 * @typedef socketInfo
 * @property {SocketHost} host - Information about the local endpoint of the socket.
 * @property {SocketRemote} remote -  Information about the remote endpoint of the socket.
 */

/**
 * Information about the host TCP socket.
 *
 * @typedef SocketHost
 * @property {number} port - The port number on the local machine.
 * @property {string} family - The IP family of the local address (`IPv4` or `IPv6`).
 * @property {string} address - The local IP address.
 */

/**
 * Information about the remote TCP socket.
 *
 * @typedef SocketRemote
 * @property {number} port - The port number on the remote machine.
 * @property {string} address - The remote IP address.
 */

const kSetSocketIdUnchecked = Symbol('kSetSocketIdUnchecked');
const kAsyncGenerator = Symbol('kAsyncGenerator');

/**
 * A Socket object is a JS wrapper around a low-level TCP socket.
 *
 * @fires connect - Emitted when a socket connection is successfully established.
 * @fires data - Emitted when data is received.
 * @fires end - Emitted when the other end of the socket sends a FIN packet.
 * @fires error - Emitted when an error occurs.
 * @fires close - Emitted once the socket is fully closed.
 * @fires timeout - Emitted if the socket times out from (read) inactivity.
 */
export class Socket extends EventEmitter {
  #id;
  #host;
  #connecting;
  #encoding;
  #writable;
  #pushQueue;
  #pullQueue;
  #timeoutHandle;

  /**
   * Creates a new Socket instance.
   *
   * @returns {Socket}
   */
  constructor() {
    super();
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#connecting = false;
    this.#timeoutHandle = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
    this.timeout = 0;
  }

  /**
   * Initiates a connection on a given remote host.
   *
   * Additional signatures:
   * - connect(port: number | string, host?: string)
   *
   * @param {Object} options - Configuration options for the connection.
   * @param {string} options.host - The hostname or IP address of the remote server to connect to.
   * @param {(string|number)} options.port - The port number on the remote host to connect to.
   * @returns {Promise<socketInfo>} Information about the connected TCP socket.
   */
  async connect(...args) {
    // Parse arguments.
    const [port, hostUnchecked] = parseOptionsArgs(args);
    const hostname = hostUnchecked || '0.0.0.0';

    if (this.#connecting) {
      throw new Error('Socket is trying to connect.');
    }

    if (Number.isNaN(Number.parseInt(port))) {
      throw new TypeError(`The "port" option must be castable to number.`);
    }

    if (hostname && typeof hostname !== 'string') {
      throw new TypeError(`The "host" option must be of type string.`);
    }

    if (this.#id) {
      throw new Error(
        `Socket is already connected to <${this.remoteAddress}:${this.remotePort}>.`
      );
    }

    this.#connecting = true;

    // Use DNS lookup to resolve the hostname.
    const addresses = await dns.lookup(hostname);

    // Prefer IPv4 address.
    const remoteHost = addresses.some((addr) => addr.family === 'IPv4')
      ? addresses.filter((addr) => addr.family === 'IPv4')[0].address
      : addresses[0].address;

    const { id, host, remote } = await binding.connect(
      remoteHost,
      Number.parseInt(port)
    );

    this.#id = id;
    this.#connecting = false;
    this.#writable = true;
    this.#host = host;
    this.remoteAddress = remote.address;
    this.remotePort = remote.port;

    const [onAvailableSocketData, signal] = callbackTimeout(
      this.#onAvailableSocketData.bind(this),
      this.timeout,
      () => this.emit('timeout')
    );

    this.#timeoutHandle = signal;
    this.emit('connect', { host, remote });

    binding.readStart(this.#id, onAvailableSocketData);

    return { host, remote };
  }

  /**
   * Sets the encoding for the current socket.
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
   * Sets the socket to timeout after timeout milliseconds of (read) inactivity on the socket.
   *
   * @param {Number} timeout - The duration after which the socket should timeout due to inactivity.
   */
  setTimeout(timeout = 0) {
    // Coalesce to number or NaN.
    timeout *= 1;

    // Check timeout's boundaries.
    if (!(timeout >= 0 && timeout <= TIMEOUT_MAX)) {
      timeout = 0;
    }

    if (this.#id) {
      // Timeout value changed after the socket began waiting.
      this.#timeoutHandle.emit('timeoutUpdate', timeout);
    }

    this.timeout = timeout;
  }

  /**
   * Returns a promise which is fulfilled when the TCP stream can return a chunk.
   *
   * @returns {Promise<(Uint8Array|string)>} The chunk read from the socket.
   */
  read() {
    // Check if the socket is connected to a host.
    if (!this.#id) return null;

    // HACK: The following is used to handle uncaught errors thrown
    // from the event-emitter when no one is subscribed to the `error` event.
    if (this.listenerCount('error') === 0) this.on('error', () => {});

    // No available value to read yet.
    if (this.#pushQueue.length === 0) {
      const { promise, promiseExt } = makeDeferredPromise();
      this.#pullQueue.push(promiseExt);
      return timeout(promise, this.timeout);
    }

    const value = this.#pushQueue.shift();
    const action = value instanceof Error ? Promise.reject : Promise.resolve;

    return action.call(Promise, value);
  }

  /**
   * Writes contents to a TCP socket stream.
   *
   * @param {String|Uint8Array} data - The data to be written to the socket.
   * @param {String} [encoding] - The character encoding to use.
   * @returns {Promise<Number>} The number of bytes written.
   */
  async write(data, encoding = 'utf-8') {
    // Check the data argument type.
    if (!(data instanceof Uint8Array) && typeof data !== 'string') {
      throw new TypeError(
        `The "data" argument must be of type string or Uint8Array.`
      );
    }

    if (!this.#id) {
      throw new Error(`Socket is not connected to a remote host.`);
    }

    if (!this.#writable) {
      throw new Error(`The socket stream is not writable.`);
    }

    // Default to UTF-8 encoding.
    encoding = encoding || this.#encoding || 'utf-8';

    const bytes = toUint8Array(data, encoding);
    const bytesWritten = await binding.write(this.#id, bytes);

    this.bytesWritten += bytesWritten;

    return bytesWritten;
  }

  /**
   * Half-closes the TCP stream.
   *
   * @param {String|Uint8Array} data - The (last) data to be written to the socket.
   * @param {String} [encoding] - The character encoding to use.
   * @returns {Promise<void>}
   */
  async end(data, encoding = 'utf-8') {
    // Check socket connection.
    if (!this.#id) return;

    // If data is given, write to stream.
    if (data) {
      await this.write(data, encoding);
    }
    this.#writable = false;
    await binding.shutdown(this.#id);
  }

  /**
   * Closes both sides of the TCP sockets.
   */
  async destroy() {
    // Check if the socket is indeed connected.
    if (!this.#id) return;

    this.#timeoutHandle?.emit('timeoutUpdate', 0);
    await binding.close(this.#id);

    // Ignore pending reads.
    for (const promise of this.#pullQueue) {
      promise.resolve(null);
    }

    this.#reset();
    this.emit('close');
  }

  /**
   * Returns the bound address, the address family name and port of the socket.
   *
   * @returns {SocketHost} - Information about the local endpoint of the socket.
   */
  address() {
    return this.#host;
  }

  /**
   * Resets socket's internal state (not to be called manually).
   * @ignore
   */
  #reset() {
    this.#id = undefined;
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#connecting = false;
    this.#timeoutHandle = undefined;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
    this.timeout = 0;
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

  #onAvailableSocketData(err, arrayBufferView) {
    // Check for errors during socket read.
    if (err) {
      this.#asyncDispatch(err);
      this.emit('error', err);
      return;
    }

    // Check if the remote host closed the connection.
    if (arrayBufferView.byteLength === 0) {
      this.#asyncDispatch(null);
      this.emit('end');
      this.destroy();
      return;
    }

    this.bytesRead += arrayBufferView.byteLength;

    // Transform ArrayBuffer into a Uint8Array we can use.
    const data = new Uint8Array(arrayBufferView);
    const data_transform = this.#encoding
      ? new TextDecoder(this.#encoding).decode(new Uint8Array(data))
      : data;

    // Use the EE mode instead of the async-iterator.
    if (this.listenerCount('data') > 0) {
      this.emit('data', data_transform);
      return;
    }

    this.#asyncDispatch(data_transform);
  }

  /**
   * Hard-sets the ID of the socket (ONLY for internal use).
   *
   * @param {Number} id - The resource ID existing in the event-loop.
   * @ignore
   */
  [kSetSocketIdUnchecked](id) {
    this.#id = id;
    this.#writable = true;

    const [onAvailableSocketData, signal] = callbackTimeout(
      this.#onAvailableSocketData.bind(this),
      this.timeout,
      () => this.emit('timeout')
    );

    this.#timeoutHandle = signal;
    binding.readStart(this.#id, onAvailableSocketData);
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
   * The socket should be async iterable.
   * @ignore
   */
  [Symbol.asyncIterator](signal) {
    const iterator = { return: () => this.end() };
    return Object.assign(this[kAsyncGenerator](signal), iterator);
  }
}

/**
 * A Server object is a wrapper around a TCP listener.
 *
 * @fires listening - Emitted when the server has been bound.
 * @fires connection - Emitted when a new connection is made.
 * @fires close - Emitted when the server stops accepting new connections.
 * @fires error - Emitted when an error occurs.
 */
export class Server extends EventEmitter {
  #id;
  #host;
  #pushQueue;
  #pullQueue;

  /**
   * Creates a new Server instance.
   *
   * @returns {Server} An instance of the TCP `Server` class.
   */
  constructor() {
    super();
    this.#pushQueue = [];
    this.#pullQueue = [];
  }

  /**
   * Starts listening for incoming connections.
   *
   * @param {(string|number)} port - The port number or string on which the server should listen.
   * @param {string} host - The hostname or IP address on which the server will listen.
   * @returns {Promise<SocketHost>} The host information where the server is listening.
   */
  async listen(...args) {
    // Parse arguments.
    const [port, hostUnchecked] = parseOptionsArgs(args);
    const hostname = hostUnchecked || '127.0.0.1';

    if (Number.isNaN(Number.parseInt(port))) {
      throw new TypeError(`The "port" option must be castable to number.`);
    }

    if (hostname && typeof hostname !== 'string') {
      throw new TypeError(`The "host" option must be of type string.`);
    }

    if (this.#id) {
      throw new Error(`Server is already listening for connections.`);
    }

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
      this.#onAvailableConnection.bind(this)
    );

    this.#id = socketInfo.id;
    this.#host = socketInfo.host;

    this.emit('listening', this.#host);

    return this.#host;
  }

  /**
   * Waits for a TCP client to connect and accepts the connection.
   *
   * @returns {Promise<Socket>} - A `Socket` object representing the connected client.
   */
  accept() {
    // Check if the server is listening.
    if (!this.#id) {
      throw new Error(`Server is not bound to a port.`);
    }

    // HACK: The following is used to handle uncaught errors thrown
    // from the event-emitter when no one is subscribed to the `error` event.
    if (this.listenerCount('error') === 0) this.on('error', () => {});

    // No available connection yet.
    if (this.#pushQueue.length === 0) {
      const { promise, promiseExt } = makeDeferredPromise();
      this.#pullQueue.push(promiseExt);
      return promise;
    }

    const socket = this.#pushQueue.shift();
    const action = socket instanceof Error ? Promise.reject : Promise.resolve;

    return action.call(Promise, socket);
  }

  /**
   * Stops the server from accepting new connections.
   */
  async close() {
    // Check if the server is already closed.
    if (!this.#id) {
      throw new Error('Server is already closed.');
    }
    await binding.close(this.#id);
    this.emit('close');
  }

  /**
   * Returns the bound address, the address family name and port of the socket.
   *
   * @returns {SocketHost} The host information where the server is listening.
   */
  address() {
    return this.#host;
  }

  #asyncDispatch(socket) {
    if (this.#pullQueue.length === 0) {
      this.#pushQueue.push(socket);
      return;
    }
    const promise = this.#pullQueue.shift();
    const action = socket instanceof Error ? promise.reject : promise.resolve;
    action(socket);
  }

  #onAvailableConnection(err, sockInfo) {
    // Check for socket connection errors.
    if (err) {
      this.#asyncDispatch(err);
      this.emit('error', err);
      return;
    }

    // Create a new socket instance.
    const socket = new Socket();
    const { id, remoteAddress, remotePort } = sockInfo;

    socket[kSetSocketIdUnchecked](id);
    socket.remoteAddress = remoteAddress;
    socket.remotePort = remotePort;

    // Check if a connection handler is specified.
    if (this.listenerCount('connection') > 0) {
      this.emit('connection', socket);
      return;
    }

    this.#asyncDispatch(socket);
  }

  async *[kAsyncGenerator]() {
    let socket;
    while ((socket = await this.accept())) {
      yield socket;
    }
  }

  /**
   * The server should be async iterable.
   * @ignore
   */
  [Symbol.asyncIterator]() {
    const iterator = { return: () => this.close() };
    return Object.assign(this[kAsyncGenerator](), iterator);
  }
}

export default {
  TimeoutError,
  Socket,
  connect,
  createConnection,
  Server,
  createServer,
};
