// Fetch API
//
// The global fetch() method starts the process of fetching a resource from the network,
// returning a promise which is fulfilled once the response is available.
//
// https://developer.mozilla.org/en-US/docs/Web/API/fetch

import http from 'http';

// Utility function that combines uint8arrays.
function concatUint8Arrays(...arrays) {
  return arrays.reduce(
    (acc, array) => new Uint8Array([...acc, ...array]),
    new Uint8Array(0)
  );
}

/**
 * The Response interface of the Fetch API represents the response to a request.
 * https://developer.mozilla.org/en-US/docs/Web/API/Response
 */
class Response {
  #statusCode;
  #headers;
  #body;
  #bodyUsed;

  /**
   * Creates a new Response object.
   *
   * @returns {Response}
   */
  constructor({ statusCode, headers, body }) {
    this.#statusCode = statusCode;
    this.#headers = headers;
    this.#body = body;
    this.#bodyUsed = false;
  }

  /**
   * Resolves with a text representation of the response body.
   *
   * @returns Promise<String>
   */
  async text() {
    const content = await this.#body.text();
    this.#bodyUsed = true;
    return content;
  }

  /**
   * Resolves with the result of parsing the response body text as JSON.
   *
   * @returns Promise<Object>
   */
  async json() {
    const content = await this.#body.json();
    this.#bodyUsed = true;
    return content;
  }

  /**
   * Resolves with an ArrayBuffer representation of the response body.
   *
   * @returns Promise<ArrayBuffer>
   */
  async arrayBuffer() {
    const chunks = [];
    for await (const data of this.#body) {
      const chunk = new Uint8Array(data);
      chunks.push(chunk);
    }
    this.#bodyUsed = true;
    const content = concatUint8Arrays(...chunks);
    return content.buffer;
  }

  /**
   * A ReadableStream of the body contents.
   */
  get body() {
    return this.#body;
  }

  /**
   * Stores a boolean value that declares whether the body has been used in a response yet.
   */
  get bodyUsed() {
    return this.#bodyUsed;
  }

  /**
   * The Headers object associated with the response.
   */
  get headers() {
    return this.#headers;
  }

  /**
   * A boolean indicating whether the response was successful.
   */
  get ok() {
    // Should be in the range (200 – 299).
    return this.#statusCode >= 200 && this.#statusCode <= 299;
  }

  /**
   * The status code of the response. (This will be 200 for a success).
   */
  get status() {
    return this.#statusCode;
  }

  /**
   * The status message corresponding to the status code. (e.g., OK for 200).
   */
  get statusText() {
    return http.STATUS_CODES[this.#statusCode];
  }
}

/**
 * Starts the process of fetching a resource from the network.
 *
 * @param {String} url
 * @param {Object} options
 *
 * @returns Promise<Response>
 */
async function fetch(url, options = {}) {
  // Fetch is a wrapper around `http.request`.
  return new Response(await http.request(url, options));
}

export default fetch;
