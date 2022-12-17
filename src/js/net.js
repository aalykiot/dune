// TCP Networking APIs
//
// The TCP Networking APIs provide an asynchronous network API for creating
// stream-based TCP servers and clients.
//
// https://nodejs.org/dist/latest-v18.x/docs/api/net.html

import dns from 'dns';

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

/**
 * Initiates a connection to a given remote host.
 *
 * @param {Object} options
 * @returns Promise<Socket>
 */
export async function createConnection(...args) {
  const socket = new Socket();
  await socket.connect(...args);
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
  server.onConnection = onConnection;
  return server;
}

const kSetSocketIdUnchecked = Symbol('kSetSocketIdUnchecked');

/**
 * A Socket object is a JS wrapper around a low-level TCP socket.
 */
export class Socket {
  #id;
  #host;
  #connecting;
  #encoding;
  #writable;
  #pushQueue;
  #pullQueue;

  /**
   * Creates a new Socket instance.
   *
   * @returns {Socket}
   */
  constructor() {
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#connecting = false;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
  }

  /**
   * Initiates a connection on a given remote host.
   *
   * @param  {...any} args
   * @returns {Promise<object>}
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

    const { id, host, remote } = await binding.connect(remoteHost, port);

    this.#id = id;
    this.#connecting = false;
    this.#writable = true;
    this.#host = host;
    this.remoteAddress = remote.address;
    this.remotePort = remote.port;

    binding.readStart(this.#id, this._onAvailableSocketData.bind(this));

    return { host, remote };
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
   * Returns a promise which is fulfilled when the TCP stream can return a chunk.
   *
   * @returns {Promise<Uint8Array|string>}
   */
  read() {
    // Check if the socket is connected to a host.
    if (!this.#id) {
      throw new Error(`Socket is not connected to a remote host.`);
    }

    // No available value to read wet.
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
   * Writes contents to a TCP socket stream.
   *
   * @param {String|Uint8Array} data
   * @param {String} [encoding]
   * @returns {Promise<Number>}
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

    // Default tu UTF-8 encoding.
    encoding = encoding || this.#encoding || 'utf-8';

    const bytes = toUint8Array(data, encoding);
    const bytesWritten = await binding.write(this.#id, bytes);

    this.bytesWritten += bytesWritten;

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
    if (!this.#id) {
      throw new Error('Socket is not connected to a remote host.');
    }
    // Close the socket.
    await binding.close(this.#id);
    this._reset();
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
   * Resets socket's internal state (not to be called manually).
   */
  _reset() {
    this.#pushQueue = [];
    this.#pullQueue = [];
    this.#connecting = false;
    this.bytesRead = 0;
    this.bytesWritten = 0;
    this.remotePort = undefined;
    this.remoteAddress = undefined;
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

  _onAvailableSocketData(error, arrayBufferView) {
    // Check for errors during socket read.
    if (error) {
      this._asyncDispatch(error);
      return;
    }

    // Check if the remote host closed the connection.
    if (arrayBufferView.byteLength === 0) {
      this.destroy();
      return;
    }

    this.bytesRead += arrayBufferView.byteLength;

    // Transform ArrayBuffer into a Uint8Array we can use.
    const data = new Uint8Array(arrayBufferView);
    const data_transform = this.#encoding
      ? new TextDecoder(this.#encoding).decode(new Uint8Array(data))
      : data;

    this._asyncDispatch(data_transform);
  }

  /**
   * Hard-sets the ID of the socket (ONLY for internal use).
   *
   * @param {Number} id
   */
  [kSetSocketIdUnchecked](id) {
    this.#id = id;
    this.#writable = true;
    binding.readStart(this.#id, this._onAvailableSocketData.bind(this));
  }

  /**
   * The socket should be async iterable.
   */
  async *[Symbol.asyncIterator]() {
    let data;
    while ((data = await this.read())) {
      yield data;
    }
  }
}

/**
 * A Server object is a wrapper around a TCP listener.
 */
export class Server {
  #id;
  #host;
  #pushQueue;
  #pullQueue;

  /**
   * Creates a new Server instance.
   *
   * @returns {Server}
   */
  constructor() {
    this.onConnection = undefined;
    this.#pushQueue = [];
    this.#pullQueue = [];
  }

  /**
   * Starts listening for incoming connections.
   *
   * @param  {...any} args
   * @returns Promise<Object>
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
      this._onAvailableConnection.bind(this)
    );

    this.#id = socketInfo.id;
    this.#host = socketInfo.host;

    if (!this.onConnection) return this.#host;

    for await (const socket of this) {
      this.onConnection(socket);
    }
  }

  /**
   * Waits for a TCP client to connect and accepts the connection.
   *
   * @returns {Promise<Socket>}
   */
  accept() {
    // Check if the server is listening.
    if (!this.#id) {
      throw new Error(`Server is not bound to a port.`);
    }

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
  }

  /**
   * Returns the bound address, the address family name and port of the socket.
   *
   * @returns {Object}
   */
  address() {
    return this.#host;
  }

  _asyncDispatch(socket) {
    if (this.#pullQueue.length === 0) {
      this.#pushQueue.push(socket);
      return;
    }
    const promise = this.#pullQueue.shift();
    const action = socket instanceof Error ? promise.reject : promise.resolve;
    action(socket);
  }

  _onAvailableConnection(error, sockInfo) {
    // Check for socket connection errors.
    if (error) {
      this._asyncDispatch(error);
      return;
    }

    // Create a new socket instance.
    const socket = new Socket();
    const { id, remoteAddress, remotePort } = sockInfo;

    socket[kSetSocketIdUnchecked](id);
    socket.remoteAddress = remoteAddress;
    socket.remotePort = remotePort;

    this._asyncDispatch(socket);
  }

  /**
   * The server should be async iterable.
   */
  async *[Symbol.asyncIterator]() {
    let socket;
    while ((socket = await this.accept())) {
      yield socket;
    }
  }
}

export default {
  Socket,
  createConnection,
  Server,
  createServer,
};
