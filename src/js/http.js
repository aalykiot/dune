// Hypertext Transfer Protocol (HTTP) APIs
//
// The HTTP interfaces are designed to support features of the protocol which have been traditionally difficult to use.
// The interface is careful to never buffer entire requests or responses, so the user is able to stream data.
//
// https://undici.nodejs.org/#/

import net from 'net';

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

const encoder = new TextEncoder('utf-8');

function concatUint8Arrays(...arrays) {
  return arrays.reduce(
    (acc, array) => new Uint8Array([...acc, ...array]),
    new Uint8Array(0)
  );
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

const urlRegex = new RegExp('^(.*:)//([A-Za-z0-9-.]+)(:[0-9]+)?(.*)$');

class HttpRequest {
  #protocol;
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

  constructor(url, options) {
    // Include protocol in URL.
    const checkedUrl = url.includes('://') ? url : 'http://' + url;
    const [_, protocol, hostname, port, path] = urlRegex.exec(checkedUrl); // eslint-disable-line no-unused-vars

    this.#protocol = protocol.replace(':', '');
    this.#hostname = hostname;
    this.#port = port ? Number(port.replace(':', '')) : 80;
    this.#path = path || '/';
    this.#method = options.method.toUpperCase();

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
    this.#headers.set('user-agent', 'Dune HTTP client');
    this.#headers.set('connection', 'keep-alive');
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
    for await (const data of this.#socket) {
      chunks.push(data);
      const buffer = concatUint8Arrays(...chunks);
      const response = binding.parseHttpResponse(buffer);

      // Response headers are still incomplete.
      if (!response) continue;

      // Check status code and throw if requsted.
      if (response.statusCode >= 400 && this.#throwOnError) {
        const message = STATUS_CODES[response.statusCode];
        throw new Error(`HTTP request failed with error: "${message}"`);
      }

      return new HttpResponse(response, buffer, this.#socket);
    }
  }
}

class HttpResponse {
  #socket;
  #statusCode;
  #headers;
  #body;
  #bodyLength;
  #isChunked;
  #isComplete;

  constructor(response, buffer, socket) {
    this.#socket = socket;
    this.#statusCode = response.statusCode;
    this.#headers = response.headers;
    this.#body = buffer.subarray(response.bodyAt);
    this.#bodyLength = Number.parseInt(this.#headers['content-length']) || 0;
    this.#isChunked = this.#headers['transfer-encoding']?.includes('chunked');
    this.#isComplete = this.#body?.length === this.#bodyLength;

    if (this.#isComplete && !this.#isChunked) {
      this.#socket.end();
      this.#socket = undefined;
    }
  }

  get statusCode() {
    return this.#statusCode;
  }

  get headers() {
    return this.#headers;
  }

  get body() {
    return this[Symbol.asyncIterator]();
  }

  _parseAvailableChunks(data) {
    // Mix current body with new data.
    const buffer = concatUint8Arrays(this.#body, data);
    const { chunks, position, done } = binding.parseHttpChunks(buffer);

    // Update body based on parser's cursor.
    this.#body = buffer.subarray(position);

    return {
      chunks: chunks.map((v) => new Uint8Array(v)),
      done,
    };
  }

  async *[Symbol.asyncIterator](signal) {
    // Close socket on stream pipeline errors.
    if (signal) signal.on('uncaughtStreamException', () => this.#socket.end());

    // Check if the full body has been received.
    if (this.#isComplete && !this.#isChunked) {
      yield this.#body;
      return;
    }

    let bytesReceived = this.#body.length;

    // Process incoming data from the TCP socket.
    for await (const data of this.#socket) {
      // HTTP body is received in chunks.
      if (this.#isChunked) {
        const { chunks, done } = this._parseAvailableChunks(data);
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

    // Close the underline TCP socket.
    this.#socket.end();
  }
}

// =========================================================

const defaultOptions = {
  method: 'GET',
  headers: {},
  body: null,
  timeout: 30000,
  throwOnError: false,
};

const options = Object.assign(defaultOptions, {});

const request = new HttpRequest('http://localhost:3000/', options);
const response = await request.send();

console.log(response.headers);

for await (const data of response.body) {
  console.log(new TextDecoder().decode(data));
}
