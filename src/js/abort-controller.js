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
 * Error type referring to an operation being aborted due to timeout.
 */
export class TimeoutError extends Error {
  constructor(message) {
    super();
    this.name = 'TimeoutError';
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
    this.timeoutRef = null;
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
   *
   * @param {Number} ms
   * @returns
   */
  static timeout(ms) {
    const controller = new AbortController();
    const reason = 'The operation was aborted due to timeout.';
    const abort = () => controller.abort(new TimeoutError(reason));
    controller.signal.timeoutRef = setTimeout(abort, ms);
    return controller.signal;
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

  /**
   * Note: Since the AbortSignal timeout cannot be canceled, we must prevent the timer
   * from prolonging the Dune process. By calling the following method from the
   * "ouside" world, we ensure its removal (FOR INTERNAL USE ONLY).
   */
  clearTimeout() {
    if (this.timeoutRef) {
      clearTimeout(this.timeoutRef);
    }
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
  abort(reason = 'This operation was aborted.') {
    // If it's already aborted, don't do anything.
    if (this.signal.aborted) return;

    this.signal.aborted = true;
    this.signal.reason =
      reason instanceof Error ? reason : new AbortError(reason);

    this.signal.dispatchEvent('abort');
  }
}

export default { AbortController, AbortSignal };
