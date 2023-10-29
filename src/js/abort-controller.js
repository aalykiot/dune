// Abort Controller API
//
// The AbortController interface represents a controller object that allows
// you to abort one or more Web requests as and when desired.
//
// https://developer.mozilla.org/en-US/docs/Web/API/AbortController

import { EventEmitter } from 'events';

/**
 * Error type referring to an operation being aborted.
 */
class AbortError extends Error {
  constructor(message) {
    super();
    this.name = 'AbortError';
    this.message = message;
  }
}

/**
 * The `AbortSignal` interface represents a signal object that allows you
 * to communicate with a request and abort it.
 */
export class AbortSignal {
  /**
   * Creates a new abort-signal instance.
   *
   * @returns {AbortSignal}
   */
  constructor() {
    this.eventEmitter = new EventEmitter();
    this.onabort = null;
    this.aborted = false;
    this.reason = undefined;
  }

  /**
   * Returns an abort-signal instance that is already set as aborted.
   *
   * @param {String} [reason]
   * @returns {AbortSignal}
   */
  static abort(reason) {
    const controller = new AbortController();
    controller.abort(reason);
    return controller.signal;
  }

  /**
   * Returns an abort-signal instance that will automatically abort after a specified time.
   * https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal/timeout_static
   *
   * @param {Number} milliseconds
   * @returns {AbortSignal}
   */
  // eslint-disable-next-line no-unused-vars
  static timeout(milliseconds) {
    // Note: Implementing the static `timeout` method adds a lot of complexity, plus
    // the `http.request` method supports nativly the consept of timeouts.
    //
    // https://github.com/mo/abortcontroller-polyfill/issues/73#issuecomment-1660420796

    throw new Error('Not implemented!');
  }

  addEventListener(name, handler) {
    this.eventEmitter.on(name, handler);
  }

  removeEventListener(name, handler) {
    this.eventEmitter.removeListener(name, handler);
  }

  dispatchEvent(type) {
    const event = { type, target: this };
    const handlerName = `on${type}`;

    if (typeof this[handlerName] === 'function') this[handlerName](event);
    this.eventEmitter.emit(type, event);
  }

  /**
   * Throws the signal's abort reason if the signal has been aborted.
   */
  throwIfAborted() {
    if (this.aborted) throw this.reason;
  }
}

/**
 * The `AbortController` interface represents a controller object that allows
 * you to abort one or more web requests as and when desired.
 */
export class AbortController {
  /**
   * Creates a new abort-controller instance.
   *
   * @returns {AbortController}
   */
  constructor() {
    this.signal = new AbortSignal();
  }

  /**
   * Aborts a request before it has completed.
   *
   * @param {String} reason
   */
  abort(reason = 'The operation was aborted.') {
    // If it's already aborted, don't do anything.
    if (this.signal.aborted) return;

    this.signal.aborted = true;
    this.signal.reason = new AbortError(reason);
    this.signal.dispatchEvent('abort');
  }
}

export default { AbortController, AbortSignal };
