// AbortController API
//
// The AbortController interface represents a controller object that allows
// you to abort one or more Web requests as and when desired.
//
// https://developer.mozilla.org/en-US/docs/Web/API/AbortController

import { EventEmitter } from 'events';

/**
 * The AbortSignal interface represents a signal object that allows you
 * to communicate with a request and abort it.
 */
export class AbortSignal {
  /**
   * Creates a new AbortSignal instance.
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
   * Returns an AbortSignal instance that is already set as aborted.
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
   * Returns an AbortSignal instance that will automatically abort after a specified time.
   *
   * @param {Number} ms
   * @returns
   */
  static timeout(ms) {
    const controller = new AbortController();
    setTimeout(() => controller.abort(new Error('TimeoutError')), ms);
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
}

/**
 * The AbortController interface represents a controller object that allows
 * you to abort one or more Web requests as and when desired.
 */
export class AbortController {
  /**
   * Creates a new AbortController instance.
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
  abort(reason) {
    // If it's already aborted, don't do anything.
    if (this.signal.aborted) return;

    this.signal.aborted = true;
    this.signal.reason = reason ? reason : new Error('AbortError');
    this.signal.dispatchEvent('abort');
  }
}

export default { AbortController, AbortSignal };
