/**
 * HTTP Networking APIs
 *
 * The HTTP interfaces are built to make it easier to use traditionally difficult protocol
 * features. By never buffering complete requests or responses, users can stream data
 * instead, making data transmission more efficient and flexible.
 *
 * @see {@link https://undici.nodejs.org/#/}
 *
 * @module HTTP
 */

import net from 'net';
import assert from 'assert';
import { EventEmitter } from 'events';

const binding = process.binding('http_parser');

/**
 * A list of the HTTP methods that are supported by the parser.
 * @constant {string[]}
 */
export const METHODS = [
  'ACL',
  'BIND',
  'CHECKOUT',
  'CONNECT',
  'COPY',
  'DELETE',
  'GET',
  'HEAD',
  'LINK',
  'LOCK',
  'M-SEARCH',
  'MERGE',
  'MKACTIVITY',
  'MKCALENDAR',
  'MKCOL',
  'MOVE',
  'NOTIFY',
  'OPTIONS',
  'PATCH',
  'POST',
  'PROPFIND',
  'PROPPATCH',
  'PURGE',
  'PUT',
  'REBIND',
  'REPORT',
  'SEARCH',
  'SOURCE',
  'SUBSCRIBE',
  'TRACE',
  'UNBIND',
  'UNLINK',
  'UNLOCK',
  'UNSUBSCRIBE',
];

/**
 * A collection of all the standard HTTP response status codes.
 * @constant {string[]}
 */
export const STATUS_CODES = {
  100: 'Continue',
  101: 'Switching Protocols',
  102: 'Processing',
  103: 'Early Hints',
  200: 'OK',
  201: 'Created',
  202: 'Accepted',
  203: 'Non-Authoritative Information',
  204: 'No Content',
  205: 'Reset Content',
  206: 'Partial Content',
  207: 'Multi-Status',
  208: 'Already Reported',
  226: 'IM Used',
  300: 'Multiple Choices',
  301: 'Moved Permanently',
  302: 'Found',
  303: 'See Other',
  304: 'Not Modified',
  305: 'Use Proxy',
  307: 'Temporary Redirect',
  308: 'Permanent Redirect',
  400: 'Bad Request',
  401: 'Unauthorized',
  402: 'Payment Required',
  403: 'Forbidden',
  404: 'Not Found',
  405: 'Method Not Allowed',
  406: 'Not Acceptable',
  407: 'Proxy Authentication Required',
  408: 'Request Timeout',
  409: 'Conflict',
  410: 'Gone',
  411: 'Length Required',
  412: 'Precondition Failed',
  413: 'Payload Too Large',
  414: 'URI Too Long',
  415: 'Unsupported Media Type',
  416: 'Range Not Satisfiable',
  417: 'Expectation Failed',
  418: "I'm a Teapot",
  421: 'Misdirected Request',
  422: 'Unprocessable Entity',
  423: 'Locked',
  424: 'Failed Dependency',
  425: 'Too Early',
  426: 'Upgrade Required',
  428: 'Precondition Required',
  429: 'Too Many Requests',
  431: 'Request Header Fields Too Large',
  451: 'Unavailable For Legal Reasons',
  500: 'Internal Server Error',
  501: 'Not Implemented',
  502: 'Bad Gateway',
  503: 'Service Unavailable',
  504: 'Gateway Timeout',
  505: 'HTTP Version Not Supported',
  506: 'Variant Also Negotiates',
  507: 'Insufficient Storage',
  508: 'Loop Detected',
  509: 'Bandwidth Limit Exceeded',
  510: 'Not Extended',
  511: 'Network Authentication Required',
};

function makeDeferredPromise() {
  // Extract the resolve method from the promise.
  const promiseExt = {};
  const promise = new Promise((resolve, reject) => {
    promiseExt.resolve = resolve;
    promiseExt.reject = reject;
  });

  return { promise, promiseExt };
}

function concatUint8Arrays(...arrays) {
  return arrays.reduce(
    (acc, array) => new Uint8Array([...acc, ...array]),
    new Uint8Array(0)
  );
}

function toUint8Array(data, encoding) {
  if (!(data instanceof Uint8Array)) {
    return new TextEncoder(encoding).encode(data);
  }
  return data;
}

function isIterable(input) {
  if (input === null || input === undefined) return false;
  return (
    typeof input[Symbol.iterator] === 'function' ||
    typeof input[Symbol.asyncIterator] === 'function'
  );
}

function isString(input) {
  if (input === null || input === undefined) return false;
  return typeof input === 'string';
}

function isTypedArray(input) {
  if (input === null || input === undefined) return false;
  return input instanceof Uint8Array;
}

function isAcceptableBodyType(input) {
  if (input === null || input === undefined) return true;
  return isString(input) || isTypedArray(input) || isIterable(input);
}

function assertChunkType(chunk) {
  if (!isString(chunk) && !isTypedArray(chunk)) {
    throw new Error('Each chunk must be of type string or Uint8Array.');
  }
}

const capitalizeFirstLetter = (s) => s.charAt(0).toUpperCase() + s.slice(1);

function formatHeaders(headers) {
  const kHeaders = {};
  for (const [key, value] of headers.entries()) {
    const name = key.split('-').map(capitalizeFirstLetter).join('-');
    kHeaders[name] = value;
  }
  return kHeaders;
}

async function* wrapIterable(iterable) {
  let result;
  let iterator = iterable[Symbol.asyncIterator]();
  while ((result = await iterator.next())) {
    if (result.done) break;
    yield result.value;
  }
}

const urlRegex = new RegExp('^(.*:)//([A-Za-z0-9-.]+)(:[0-9]+)?(.*)$');

/**
 * An outgoing HTTP request to a remote host.
 * @ignore
 */
class Request {
  #hostname;
  #port;
  #path;
  #method;
  #timeout;
  #throwOnError;
  #body;
  #bodyLength;
  #socket;
  #headers;
  #isChunkedEncoding;
  #signal;

  constructor(url, options) {
    // Include protocol in URL.
    const checkedUrl = url.includes('://') ? url : 'http://' + url;
    const [_, protocol, hostname, port, path] = urlRegex.exec(checkedUrl); // eslint-disable-line no-unused-vars

    // Only HTTP requests are supported.
    if (protocol !== 'http:') {
      throw new Error(`Protocol "${protocol}" not supported.`);
    }

    this.#hostname = hostname;
    this.#port = port ? Number(port.replace(':', '')) : 80;
    this.#path = path || '/';
    this.#method = options.method.toUpperCase();
    this.#signal = options.signal;

    // Check if HTTP method is valid.
    if (!METHODS.includes(this.#method)) {
      throw new Error(`HTTP method "${this.#method}" is not recognized.`);
    }

    this.#timeout = options.timeout;
    this.#throwOnError = options.throwOnError;

    this.#body = options.body;
    this.#bodyLength = this.#body?.length || 0;

    // Check HTTP's body (if specified)
    if (this.#body && !isAcceptableBodyType(this.#body)) {
      throw new TypeError(
        'The body must be of type string, Uint8Array or an iterable object.'
      );
    }

    this.#headers = new Map();
    this.#headers.set('host', this.#hostname + ':' + this.#port);
    this.#headers.set('user-agent', `dune/${process.version}`);
    this.#headers.set('accept', '*/*');
    this.#headers.set('connection', 'close');
    this.#headers.set('content-length', this.#bodyLength);

    // Check if encoding should be chunked.
    if (
      isIterable(this.#body) &&
      !(isString(this.#body) || isTypedArray(this.#body))
    ) {
      // Content-Length and Transfer-Encoding are mutual exclusive HTTP headers.
      this.#isChunkedEncoding = true;
      this.#headers.set('transfer-encoding', 'chunked');
      this.#headers.delete('content-length');
    }

    // Override headers with user-defined ones.
    for (const [name, value] of Object.entries(options.headers)) {
      this.#headers.set(name.toLowerCase(), value);
    }

    this.#socket = new net.Socket();
  }

  async send() {
    // Start building the HTTP message.
    const encoder = new TextEncoder();
    const reqHeaders = [`${this.#method} ${this.#path} HTTP/1.1`];

    // Format and append HTTP headers to message.
    const headers = formatHeaders(this.#headers);
    for (const [name, value] of Object.entries(headers)) {
      reqHeaders.push(`${name.trim()}: ${value}`);
    }

    const reqHeadersString = reqHeaders.join('\r\n');
    const reqHeadersBytes = encoder.encode(`${reqHeadersString}\r\n\r\n`);

    // Write headers to the socket.
    await this.#socket.connect(this.#port, this.#hostname);
    await this.#socket.write(reqHeadersBytes);

    // Subscribe to the abort-controller if provided.
    if (this.#signal) {
      this.#signal.addEventListener('abort', () => this.#socket.destroy());
    }

    // Write body to the socket (sized).
    if (this.#body && !this.#isChunkedEncoding) {
      this.#socket.write(this.#body);
    }

    // Write body to the socket (chunked).
    if (this.#body && this.#isChunkedEncoding) {
      for await (const chunk of this.#body) {
        assertChunkType(chunk);
        await this.#socket.write(`${chunk.length.toString(16)}\r\n`);
        await this.#socket.write(chunk);
        await this.#socket.write('\r\n');
      }
      // Write the final chunk of size 0 to indicate the end of the body.
      await this.#socket.write('0\r\n\r\n');
    }

    this.#socket.setTimeout(this.#timeout);

    // Set up a buffer to hold the incoming data.
    let buffer = new Uint8Array();

    for await (const data of wrapIterable(this.#socket)) {
      // Concatenate existing buffer with new data.
      buffer = concatUint8Arrays(buffer, data);
      const metadata = binding.parseResponse(buffer);

      // Response headers are still incomplete.
      if (!metadata) continue;

      // Check status code and throw if requested.
      if (metadata.statusCode >= 400 && this.#throwOnError) {
        const message = STATUS_CODES[metadata.statusCode];
        throw new Error(`HTTP request failed with error: "${message}"`);
      }

      // Remove headers data from buffer.
      buffer = buffer.subarray(metadata.marker);

      return new IncomingResponse(metadata, buffer, this.#socket);
    }
  }
}

/**
 * An HTTP response from a remote host.
 */
class IncomingResponse {
  #statusCode;
  #headers;
  #body;

  constructor(metadata, buffer, socket) {
    this.#statusCode = metadata.statusCode;
    this.#headers = metadata.headers;
    this.#body = new Body(metadata, buffer, socket, false);
  }

  /**
   * Retrieves the status code of the HTTP response.
   *
   * @returns {number} - The HTTP status code.
   */
  get statusCode() {
    return this.#statusCode;
  }

  /**
   * Retrieves the headers of the HTTP response.
   *
   * @returns {Object} - An object containing the HTTP response headers.
   */
  get headers() {
    return this.#headers;
  }

  /**
   * Retrieves the body of the HTTP response.
   *
   * @returns {Body} - An instance of the HTTP `Body` class.
   */
  get body() {
    return this.#body;
  }
}

/**
 * A wrapper around a request/response HTTP body.
 */
class Body {
  #socket;
  #body;
  #bodyLength;
  #isChunked;
  #isComplete;
  #keepAlive;

  constructor({ headers }, buffer, socket, keepAlive = true) {
    this.#body = buffer;
    this.#bodyLength = Number.parseInt(headers['content-length']) || 0;
    this.#isChunked = headers['transfer-encoding']?.includes('chunked');
    this.#isComplete = this.#body?.length === this.#bodyLength;
    this.#keepAlive = keepAlive;
    this.#socket = socket;

    if (this.#isComplete && !this.#isChunked && !keepAlive) {
      this.#socket.end();
      this.#socket = undefined;
    }
  }

  /**
   * Formats the body to a UTF-8 string.
   *
   * @returns {Promise<string>} The complete body as a UTF-8 string.
   */
  async text() {
    const string = [];
    const decoder = new TextDecoder();
    const asyncIterator = this[Symbol.asyncIterator]();
    for await (const chunk of asyncIterator) {
      string.push(decoder.decode(chunk));
    }
    return string.join('');
  }

  /**
   * Formats the body to an actual JSON object.
   *
   * @returns {Promise<Object>} The complete body as a JavaScript object.
   */
  async json() {
    const data = await this.text();
    return JSON.parse(data);
  }

  /**
   * The HTTP body should be async iterable.
   * @ignore
   */
  async *[Symbol.asyncIterator](signal) {
    // Close socket on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.#socket.end());

    if (this.#isComplete && !this.#isChunked) {
      const remainingContent = this.#body.subarray(0, this.#bodyLength);
      this.#body = this.#body.subarray(remainingContent.length);
      yield remainingContent;
      return;
    }

    // TODO: Check if chunks are available from the start.
    // Node.js for example combines the first chunk with the HTTP headers when
    // sending responses with chunked encoding.

    for await (const newData of wrapIterable(this.#socket)) {
      // Mix current body with new data.
      this.#body = concatUint8Arrays(this.#body, newData);

      if (this.#isChunked) {
        // Try extracting available chunks.
        const result = binding.parseChunks(this.#body);
        // No results means not enough bytes to extract the next chunk.
        if (result) {
          this.#body = this.#body.subarray(result.position);
          yield* result.chunks;
          if (result.done) break;
        }
      } else {
        // Note: The following code handles the case when the HTTP's body
        // length is already known from the `Content-Length` header
        // but, it comes to us in multiple TCP packets.
        if (this.#body.length >= this.#bodyLength) {
          yield this.#body.subarray(0, this.#bodyLength);
          this.#body = this.#body.subarray(this.#bodyLength);
          break;
        }
      }
    }

    // Close TCP socket on not keep-alive connections.
    if (!this.#keepAlive) this.#socket.end();
  }
}

const kAsyncGenerator = Symbol('kAsyncGenerator');

/**
 * An object capable of serving HTTP requests.
 *
 * @fires request - Emitted each time there is a request.
 * @fires close - Emitted when the server closes.
 * @fires clientError - Emitted when a client connection emits an 'error' event.
 */
export class Server extends EventEmitter {
  #tcp;
  #pushQueue;
  #pullQueue;

  /**
   * Creates a new Server instance.
   *
   * @returns {Server} An instance of the HTTP `Server` class.
   */
  constructor() {
    super();
    this.#pushQueue = [];
    this.#pullQueue = [];

    // Setting up the underling TCP server.
    this.#tcp = net.createServer(this.#handleConnectionSafely.bind(this));
    this.#tcp.on('close', () => this.emit('close'));
  }

  /**
   * An object representing an HTTP connection.
   *
   * @typedef Connection
   * @property {ServerRequest} request - The incoming HTTP request.
   * @property {ServerResponse} response - The HTTP response that will be sent by the server.
   */

  /**
   * Waits for a client to connect and accepts the HTTP request.
   *
   * @returns {Promise<Connection>} An object representing an HTTP connection.
   */
  accept() {
    // No available requests yet.
    if (this.#pushQueue.length === 0) {
      const { promise, promiseExt } = makeDeferredPromise();
      this.#pullQueue.push(promiseExt);
      return promise;
    }

    const conn = this.#pushQueue.shift();
    const action = conn instanceof Error ? Promise.reject : Promise.resolve;

    return action.call(Promise, conn);
  }

  async #handleConnectionSafely(socket) {
    try {
      await this.#handleConnection(socket);
    } catch (err) {
      // Don't crash the server for a single misbehaving socket.
      if (err?.code !== 'ERR_CONNECTION_RESET') {
        throw err;
      }
    }
  }

  async #handleConnection(socket) {
    // Set-up client event dispatcher.
    socket.on('error', (err) => this.emit('clientError', err));

    // Set up a buffer to hold the incoming data.
    let buffer = new Uint8Array();

    for await (const data of socket) {
      // Concatenate existing buffer with new data.
      buffer = concatUint8Arrays(buffer, data);

      // Try parsing the HTTP headers.
      let metadata;
      try {
        metadata = binding.parseRequest(buffer);
      } catch (_) {
        const message = 'HTTP/1.1 400 Bad Request\r\nConnection: close\r\n\r\n';
        await socket.write(message);
        break;
      }

      // Request headers are still incomplete.
      if (!metadata) continue;

      buffer = buffer.subarray(metadata.marker);

      // Create the request and response streams.
      const request = new ServerRequest(metadata, buffer, socket);
      const response = new ServerResponse(metadata, socket);

      // Check if a request handler is specified; if so, emit the 'request' event.
      const hasRequestHandler = this.listenerCount('request') > 0;

      hasRequestHandler
        ? this.emit('request', request, response)
        : this.#asyncDispatch({ request, response });

      // Hack: To support persistent connections, we employ this technique to delay
      // accepting a new request from the same socket until the current
      // request-response cycle is complete.
      await new Promise((resolve) => response.once('finish', resolve));

      // Connection should close based on headers.
      if (response.getHeader('connection') === 'close') break;
    }
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

  /**
   * Information about the host TCP socket.
   *
   * @typedef SocketHost
   * @property {number} port - The port number on the local machine.
   * @property {string} family - The IP family of the local address (`IPv4` or `IPv6`).
   * @property {string} address - The local IP address.
   */

  /**
   * Starts listening for incoming HTTP connections.
   *
   * @param {(string|number)} port - The port number or string on which the server should listen.
   * @param {string} host - The hostname or IP address on which the server will listen.
   * @returns {Promise<SocketHost>} The host information where the server is listening.
   */
  async listen(...args) {
    return this.#tcp.listen(...args);
  }

  /**
   * Stops the server from accepting new connections.
   */
  async close() {
    await this.#tcp.close();
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

/**
 * A server-side request object for handling HTTP requests.
 *
 * @property {string} httpVersion - The HTTP version specified in the request.
 * @property {string} method - The HTTP method used for the request
 * @property {string} url - The URL being requested.
 * @property {Object} headers - An object containing the request headers.
 */
export class ServerRequest {
  #body;

  constructor(metadata, buffer, socket) {
    this.httpVersion = `1.${metadata.version}`;
    this.method = metadata.method;
    this.url = metadata.path;
    this.headers = metadata.headers;
    this.#body = new Body(metadata, buffer, socket);
  }

  /**
   * Formats the body to a UTF-8 string.
   *
   * @returns {Promise<string>} The body content as a UTF-8 string.
   */
  async text() {
    return this.#body.text();
  }

  /**
   * Formats the body to an actual JSON object.
   *
   * @returns {Promise<Object>} The body content as JavaScript object.
   */
  async json() {
    return this.#body.json();
  }

  /**
   * The HTTP request should be async iterable.
   * @ignore
   */
  async *[Symbol.asyncIterator](signal) {
    yield* this.#body[Symbol.asyncIterator](signal);
  }
}

/**
 * A server-side response object for handling HTTP requests.
 */
class ServerResponse extends EventEmitter {
  #socket;
  #headers;
  #headersSent;
  #code;
  #message;
  #writtenOnce;
  #version;

  constructor({ version, headers }, socket) {
    super();
    this.#socket = socket;
    this.#headersSent = false;
    this.#code = 200;
    this.#message = STATUS_CODES[this.#code];
    this.#writtenOnce = false;
    this.#version = version;

    const keepAlive = headers?.connection !== 'close';

    // Set default headers.
    this.#headers = new Map();
    this.#headers.set('date', new Date().toGMTString());
    this.#headers.set('connection', keepAlive ? 'keep-alive' : 'close');
    this.#headers.set('transfer-encoding', 'chunked');
  }

  /**
   * Writes a chunk of the response body.
   *
   * @param {(String|Uint8Array)} data - The data to be written to the response body.
   * @param {String} [encoding] - The character encoding to use.
   */
  async write(data, encoding = 'utf-8') {
    // Check the data argument type.
    if (!(data instanceof Uint8Array) && typeof data !== 'string') {
      throw new TypeError(
        `The "data" argument must be of type string or Uint8Array.`
      );
    }

    const content = toUint8Array(data, encoding);

    // Make sure headers are sent to client.
    if (!this.#headersSent) {
      await this.#sendHeaders();
    }

    const chunkLength = content.length.toString(16);
    const chunkedEncoding = this.hasHeader('transfer-encoding');

    // Chunkify the provided content.
    if (chunkedEncoding) {
      await this.#socket.write(`${chunkLength}\r\n`);
      await this.#socket.write(content);
      await this.#socket.write('\r\n');
      this.#writtenOnce = true;
      return;
    }

    await this.#socket.write(content);
    this.#writtenOnce = true;
  }

  /**
   * Signals that all of the response headers and body have been sent.
   *
   * @param {string|Uint8Array} [data] - The final data chunk to be sent with the response.
   * @param {string} [encoding] - The character encoding to use.
   */
  async end(data, encoding = 'utf-8') {
    // If data is given, write to stream.
    if (data) {
      const content = toUint8Array(data, encoding);
      const shouldSetLength = !this.#writtenOnce && !this.#headersSent;
      // Note: If the `.end` is called without any `.write` we can
      // set the content-length of the response.
      if (shouldSetLength) {
        this.setHeader('content-length', content.length);
      }
      await this.write(content);
    }

    // On chunked response send end-chunk.
    if (this.getHeader('transfer-encoding')?.includes('chunked')) {
      await this.#socket.write(`0\r\n\r\n`);
    }

    this.emit('finish');
  }

  /**
   * Sends a response header to the request.
   *
   * @param {number} code - The HTTP status code for the response.
   * @param {string} [message] - A human-readable status message corresponding to the status code.
   * @param {Object} [headers] - Additional header fields to include in the response.
   */
  async writeHead(...args) {
    // Do not send headers multiple times.
    if (this.#headersSent) {
      throw new Error('Cannot set headers after they are sent.');
    }

    // Parse variadic arguments.
    const [code, message, headers] = this.#parseWriteHeadArgs(args);

    if (STATUS_CODES[code] === undefined) {
      throw new RangeError(`Not valid HTTP status code "${code}".`);
    }

    if (typeof message !== 'string') {
      throw new TypeError('The "message" argument must be of type string.');
    }

    this.#code = code;
    this.#message = message;

    // Override headers with user-defined ones.
    for (const [name, value] of Object.entries(headers)) {
      assert.string(name);
      this.#headers.set(name.toLowerCase(), String(value));
    }

    await this.#sendHeaders();
  }

  #parseWriteHeadArgs(args) {
    // Use default values on empty or single argument(s).
    if (args.length < 2) return [args[0], STATUS_CODES[args[0]], {}];

    return typeof args[1] === 'object'
      ? [args[0], STATUS_CODES[args[0]], args[1]]
      : [args[0], args[1], args[2] || {}];
  }

  /**
   * Checks for HTTP header violations, mutual exclusions, etc.
   * @ignore
   */
  #checkHeaders() {
    // HTTP 1.0 doesn't support other encoding.
    if (this.#version === 0) {
      this.removeHeader('transfer-encoding');
      this.removeHeader('content-length');
      this.setHeader('connection', 'close');
    }

    // Per section 3.3.1 of RFC7230:
    // A server MUST NOT send a Transfer-Encoding header field in any response
    // with a status code of 1xx (Informational) or 204 (No Content).
    if (this.#code < 200 || this.#code === 204) {
      this.removeHeader('transfer-encoding');
    }

    // Content-Length and Transfer-Encoding are mutual exclusive HTTP headers.
    let hasLength = this.hasHeader('content-length');
    let hasEncoding = this.hasHeader('transfer-encoding') && !hasLength;

    if (hasLength) {
      this.removeHeader('transfer-encoding');
    }

    if (hasEncoding) {
      this.removeHeader('content-length');
    }
  }

  /**
   * Writes raw headers to the TCP stream.
   * @ignore
   */
  async #sendHeaders() {
    // Start building the HTTP message.
    const encoder = new TextEncoder();
    const resHeaders = [
      `HTTP/1.${this.#version} ${this.#code} ${this.#message}`,
    ];

    // Check for HTTP header rule violations.
    this.#checkHeaders();

    // Format and append HTTP headers to message.
    const headers = formatHeaders(this.#headers);
    for (const [name, value] of Object.entries(headers)) {
      resHeaders.push(`${name.trim()}: ${value}`);
    }

    const resHeadersString = resHeaders.join('\r\n');
    const resHeadersBytes = encoder.encode(`${resHeadersString}\r\n\r\n`);

    // Write headers to the socket.
    await this.#socket.write(resHeadersBytes);
    this.#headersSent = true;
  }

  /**
   * Sets a single header value for implicit headers.
   *
   * @param {String} name - The name of the header.
   * @param {String} value - The value to be set for the header.
   */
  setHeader(name, value = '') {
    // Check for correct types on provided params.
    if (typeof name !== 'string') {
      throw new TypeError('The "name" argument must be of type string.');
    }

    if (this.#headersSent) {
      throw new Error('Cannot set headers after they are sent.');
    }

    this.#headers.set(name.toLowerCase(), String(value));
  }

  /**
   * Reads out a header value from raw headers.
   *
   * @param {string} name - The name of the header to retrieve.
   * @returns {string} The value of the requested header.
   */
  getHeader(name) {
    // Check for correct types on provided params.
    if (typeof name !== 'string') {
      throw new TypeError('The "name" argument must be of type string.');
    }

    return this.#headers.get(name.toLowerCase());
  }

  /**
   * Returns an array containing the unique names of the current outgoing headers.
   *
   * @returns {Array<string>} An array of strings, each representing the name of headers.
   */
  getHeaderNames() {
    return Array.from(this.#headers.keys());
  }

  /**
   * Returns true if the header identified is currently set.
   *
   * @param {string} name - The name of the header to check.
   * @returns {boolean} Indicating whether the specified header is set.
   */
  hasHeader(name) {
    // Check for correct types on provided params.
    if (typeof name !== 'string') {
      throw new TypeError('The "name" argument must be of type string.');
    }

    return this.#headers.has(name.toLowerCase());
  }

  /**
   * Removes a header that's queued for implicit sending.
   *
   * @param {String} name - The name of the header to remove.
   */
  removeHeader(name) {
    // Check for correct types on provided params.
    if (typeof name !== 'string') {
      throw new TypeError('The "name" argument must be of type string.');
    }

    if (this.#headersSent) {
      throw new Error('Cannot remove headers after they are sent.');
    }

    this.#headers.delete(name.toLowerCase());
  }

  /**
   * Returns a copy of the current outgoing headers.
   *
   * @returns {Object} Key-value pair corresponds to a header name and its value.
   */
  getHeaders() {
    return Object.fromEntries(this.#headers);
  }

  /**
   * True if headers were sent, false otherwise (read-only).
   *
   * @returns {boolean} Indicates if the headers have been sent.
   */
  get headersSent() {
    return this.#headersSent;
  }

  /**
   * Reference to the underlying TCP socket.
   *
   * @returns {Socket} An instance of the underline TPC connection.
   */
  get socket() {
    return this.#socket;
  }
}

/**
 * Creates a promise that rejects when the 'abort' event is triggered.
 *
 * @param {AbortSignal} signal
 * @returns Promise<void>
 * @ignore
 */
function cancelation(signal) {
  return new Promise((_, reject) => {
    signal.addEventListener('abort', () => reject(signal.reason));
  });
}

// Default options for HTTP requests.
const defaultOptions = {
  method: 'GET',
  headers: {},
  body: null,
  timeout: 30000,
  throwOnError: false,
  signal: null,
};

/**
 * Performs an HTTP request.
 *
 * @param {string} url - The URL to which the HTTP request is sent.
 * @param {Object} [options] - Configuration options for the HTTP request.
 * @param {string} [options.method] - The HTTP method to be used (e.g., `GET`, `POST`).
 * @param {Object} [options.headers] - An object containing request headers.
 * @param {(string|Uint8Array|Readable)} [options.body] - The body of the request.
 * @param {Number} [options.timeout] - A timeout in milliseconds for the request.
 * @param {boolean} [options.throwOnError] - Will throw an error for non-2xx response codes.
 * @param {AbortSignal} [options.signal] - An AbortSignal to cancel the request.
 * @returns {Promise<IncomingResponse>} Containing the HTTP response.
 */
export function request(url, options = {}) {
  // Check URL param type.
  if (typeof url !== 'string') {
    throw new TypeError('The "url" argument must be of type string.');
  }

  // Check if the operation has been already aborted.
  options?.signal?.throwIfAborted();

  const configuration = Object.assign(defaultOptions, options);
  const request = new Request(url, configuration);
  const { signal } = configuration;

  // Note: In case an abort-signal has been provided we should wrap
  // a promise around its event emitter.
  return signal
    ? Promise.race([request.send(), cancelation(signal)])
    : request.send();
}

/**
 * Creates a new HTTP server.
 *
 * @param {Function} [onRequest] - A function that is called whenever the server receives a HTTP request.
 * @returns {Server} Representing the newly created HTTP server.
 */
export function createServer(onRequest) {
  // Instantiate a new HTTP server.
  const server = new Server();
  if (onRequest) {
    assert.isFunction(onRequest);
    server.on('request', onRequest);
  }
  return server;
}

export default { METHODS, STATUS_CODES, Server, createServer, request };
