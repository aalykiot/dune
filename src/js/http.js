// HTTP Networking APIs
//
// The HTTP interfaces are built to make it easier to use traditionally difficult protocol
// features. By never buffering complete requests or responses, users can stream data
// instead, making data transmission more efficient and flexible.
//
// https://undici.nodejs.org/#/

import net from 'net';
import { EventEmitter } from 'events';

const binding = process.binding('http_parser');

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

const encoder = new TextEncoder('utf-8');
const decoder = new TextDecoder('utf-8');

const urlRegex = new RegExp('^(.*:)//([A-Za-z0-9-.]+)(:[0-9]+)?(.*)$');

/**
 * An outgoing HTTP request to a remote host.
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
    const [_, __, hostname, port, path] = urlRegex.exec(checkedUrl); // eslint-disable-line no-unused-vars

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
    // Content-Length and Transfer-Encoding (chunked) are mutual exclusive HTTP headers.
    if (
      isIterable(this.#body) &&
      !(isString(this.#body) || isTypedArray(this.#body))
    ) {
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

    // Write body to the socket. (sized)
    if (this.#body && !this.#isChunkedEncoding) {
      this.#socket.write(this.#body);
    }

    // Write body to the socket. (chunked)
    if (this.#body && this.#isChunkedEncoding) {
      for await (const chunk of this.#body) {
        assertChunkType(chunk);
        await this.#socket.write(`${chunk.length}\r\n`);
        await this.#socket.write(chunk);
        await this.#socket.write('\r\n');
      }
      // Write the final chunk of size 0 to indicate the end of the body.
      await this.#socket.write('0\r\n\r\n');
    }

    const chunks = [];
    this.#socket.setTimeout(this.#timeout);

    // Await and parse response from the server.
    for await (const data of wrapIterable(this.#socket)) {
      chunks.push(data);
      const buffer = concatUint8Arrays(...chunks);
      const metadata = binding.parseResponse(buffer);

      // Response headers are still incomplete.
      if (!metadata) continue;

      // Check status code and throw if requested.
      if (metadata.statusCode >= 400 && this.#throwOnError) {
        const message = STATUS_CODES[metadata.statusCode];
        throw new Error(`HTTP request failed with error: "${message}"`);
      }

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
    this.#body = new Body(metadata, this.#headers, buffer, socket, false);
  }

  get statusCode() {
    return this.#statusCode;
  }

  get headers() {
    return this.#headers;
  }

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

  constructor({ headers, bodyAt }, buffer, socket, keepAlive = true) {
    this.#socket = socket;
    this.#body = buffer.subarray(bodyAt);
    this.#bodyLength = Number.parseInt(headers['content-length']) || 0;
    this.#isChunked = headers['transfer-encoding']?.includes('chunked');
    this.#isComplete = this.#body?.length === this.#bodyLength;
    this.#keepAlive = keepAlive;

    if (this.#isComplete && !this.#isChunked && !keepAlive) {
      this.#socket.end();
      this.#socket = undefined;
    }
  }

  /**
   * Formats the body to a UTF-8 string.
   *
   * @returns Promise<String>
   */
  async text() {
    const string = [];
    const asyncIterator = this[Symbol.asyncIterator]();
    for await (const chunk of asyncIterator) {
      string.push(decoder.decode(chunk));
    }
    return string.join('');
  }

  /**
   * Formats the body to an actual JSON object.
   *
   * @returns Promise<Object>
   */
  async json() {
    const data = await this.text();
    return JSON.parse(data);
  }

  #parseAvailableChunks(data) {
    // Mix current body with new data.
    const buffer = concatUint8Arrays(this.#body, data);
    const { chunks, position, done } = binding.parseChunks(buffer);

    // Update body based on parser's cursor.
    this.#body = buffer.subarray(position);

    return {
      chunks: chunks.map((v) => new Uint8Array(v)),
      done,
    };
  }

  /**
   * The HTTP body should be async iterable.
   */
  async *[Symbol.asyncIterator](signal) {
    // Close socket on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.#socket.end());

    // Check if the full body has been received.
    if (this.#isComplete && !this.#isChunked) {
      yield this.#body;
      return;
    }

    // TODO: Check if chunks are available from the start.
    // Node.js for example combines the first chunk with the HTTP headers when
    // sending responses with chunked encoding.

    let bytesReceived = this.#body.length;

    // Process incoming data from the TCP socket.
    for await (const data of wrapIterable(this.#socket)) {
      // HTTP body is received in chunks.
      if (this.#isChunked) {
        const { chunks, done } = this.#parseAvailableChunks(data);
        yield* chunks;
        if (done) break;
        continue;
      }

      // Note: The following code handles the case when the HTTP's body
      // length is already known from the `Content-Length` header
      // but, it comes to us in multiple TCP packets.
      yield data;
      bytesReceived += data.length;

      if (bytesReceived === this.#bodyLength) {
        break;
      }
    }

    // Close TCP socket on not keep-alive connections.
    if (!this.#keepAlive) {
      this.#socket.end();
    }
  }
}

/**
 * A server-side request object for handling HTTP requests.
 */
export class ServerRequest {
  #body;

  constructor(metadata, buffer, socket) {
    this.httpVersion = `1.${metadata.version}`;
    this.method = metadata.method;
    this.url = metadata.url || '/';
    this.headers = metadata.headers;
    this.#body = new Body(metadata, buffer, socket);
  }

  /**
   * Formats the body to a UTF-8 string.
   *
   * @returns Promise<String>
   */
  async text() {
    return this.#body.text();
  }

  /**
   * Formats the body to an actual JSON object.
   *
   * @returns Promise<Object>
   */
  async json() {
    return this.#body.json();
  }

  /**
   * The HTTP request should be async iterable.
   */
  async *[Symbol.asyncIterator](signal) {
    yield* this.#body[Symbol.asyncIterator](signal);
  }
}

/**
 * A server-side response object for handling HTTP requests.
 */
export class ServerResponse extends EventEmitter {
  #socket;
  #headers;
  #headersSent;
  #code;
  #message;

  constructor(socket, keepAlive) {
    super();
    this.#socket = socket;
    this.#headersSent = false;
    this.#code = 200;
    this.#message = STATUS_CODES[this.#code];

    // Set default headers.
    this.#headers = new Map();
    this.#headers.set('date', new Date());
    this.#headers.set('connection', keepAlive ? 'keep-alive' : 'close');
    this.#headers.set('transfer-encoding', 'chunked');
  }

  /**
   * Writes a chunk of the response body.
   *
   * @param {String|Uint8Array} data
   * @param {String} encoding
   */
  async write(data, encoding = 'utf-8') {
    // Check the data argument type.
    if (!(data instanceof Uint8Array) && typeof data !== 'string') {
      throw new TypeError(
        `The "data" argument must be of type string or Uint8Array.`
      );
    }

    const chunkedEncoding = this.getHeader('content-length') === undefined;
    const content = toUint8Array(data, encoding);

    // Make sure headers are sent to the client.
    if (!this.#headersSent) {
      // Update headers on known content-type.
      if (!chunkedEncoding) {
        this.setHeader('content-length', content.length);
        this.removeHeader('transfer-encoding');
      }
      await this.#sendHeaders();
    }

    // Chunkify the provided content.
    if (chunkedEncoding) {
      await this.#socket.write(`${content.length}\r\n`);
      await this.#socket.write(content);
      await this.#socket.write('\r\n');
      return;
    }

    await this.#socket.write(content);
  }

  /**
   * Signals that all of the response headers and body have been sent.
   *
   * @param {String|Uint8Array} data
   * @param {String} encoding
   */
  async end(data, encoding = 'utf-8') {
    // If data is given, write to stream.
    if (data) {
      const content = toUint8Array(data, encoding);
      this.write(content);
    }

    // On chunked reponses send end-chunk.
    if (this.getHeader('content-length') === undefined) {
      await this.write(`0\r\n\r\n`);
    }

    this.emit('finish');
  }

  /**
   * Sends a response header to the request.
   *
   * @param  {...any} args
   */
  async writeHead(...args) {
    // Extract statusMessage and headers from variadic.
    const [code, message, headers] = this.#parseWriteHeadArgs(...args);

    // Check if statusCode has a valid type.
    if (typeof code !== 'number') {
      throw new TypeError('The "code" argument must be of type number.');
    }

    // Check for statusCode range validity.
    if (STATUS_CODES[code] === undefined) {
      throw new RangeError(`Inavalid HTTP status code: ${code}`);
    }

    if (typeof message !== 'string') {
      throw new TypeError('The "message" argument must be of type string.');
    }

    this.#code = code;
    this.#message = message;

    // Override headers with user-defined ones.
    for (const [name, value] of Object.entries(headers)) {
      this.#headers.set(name.toLowerCase(), String(value));
    }

    await this.#sendHeaders();
  }

  #parseWriteHeadArgs(args) {
    // Check if statusMessage was provided.
    if (typeof args[1] === 'object') {
      return [args[0], STATUS_CODES[args[0]], args[1]];
    }

    return [args[0], args[1], args[3] || {}];
  }

  /**
   * Writes raw headers to the TCP stream.
   */
  async #sendHeaders() {
    // Start building the HTTP message.
    const resHeaders = [`HTTP/1.1 ${this.#code} ${this.#message}`];

    // Content-Length and Transfer-Encoding are mutual exclusive HTTP headers.
    if (this.hasHeader('content-length')) {
      this.removeHeader('transfer-encoding');
    }

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
   * @param {String} name
   * @param {String} value
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
   * @param {String} name
   * @returns String
   */
  getHeader(name) {
    // Check for correct types on provided params.
    if (typeof name !== 'string') {
      throw new TypeError('The "name" argument must be of type string.');
    }

    return this.#headers.get(name.toLowerCase());
  }

  /**
   * Returns true if the header identified is currently set.
   *
   * @param {String} name
   * @returns Boolean
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
   * @param {String} name
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
   * @returns Object
   */
  getHeaders() {
    return Object.fromEntries(this.#headers);
  }

  /**
   * True if headers were sent, false otherwise (read-only).
   */
  get headersSent() {
    return this.#headersSent;
  }

  /**
   * Reference to the underlying TCP socket.
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
 * @param {String} url
 * @param {Object} options
 * @returns Promise<HttpResponse>
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

export default { METHODS, STATUS_CODES, request };
