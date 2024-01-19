/**
 * DOM Style Timers
 *
 * The Timers API provides functionality to allow developers to create DOM style timers.
 *
 * @see {@link https://www.w3schools.com/js/js_timing.asp}
 *
 * @module Timers
 */

import assert from 'assert';

const TIMEOUT_MAX = Math.pow(2, 31) - 1;

const binding = process.binding('timers');

let nextId = 1;

/**
 * This map keeps at sync the JavaScript timer IDs and their equivalent Rust
 * timer indexes (resource IDs) for all currently active timers.
 *
 * @ignore
 * @type {Map<number, number>}
 */

const activeTimers = new Map();

/**
 * Sets a timer which executes a function or specified piece of code once the
 * timer expires.
 *
 * @param {Function} callback - A function to be executed after the timer expires.
 * @param {Number} delay - The milliseconds that the timer should wait before the function is executed.
 * @param {...any} [args] - Additional arguments which are passed through to the function.
 * @returns {Number} The ID which identifies the timer created.
 */
export function setTimeout(callback, delay, ...args) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's boundaries.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  // Check if callback is a valid function.
  assert.isFunction(callback);

  // Pin down the correct ID value.
  const id = nextId++;

  const timer = binding.createTimeout(
    () => {
      callback(...args);
      activeTimers.delete(id);
    },
    delay,
    false
  );

  // Update `activeTimers` map.
  activeTimers.set(id, timer);

  return id;
}

/**
 * The global clearTimeout() method cancels a timeout previously established
 * by calling setTimeout().
 *
 * @param {Number} id - The ID which identifies the timer.
 */
export function clearTimeout(id) {
  // Check parameter's type.
  assert.integer(id);

  if (activeTimers.has(id)) {
    binding.removeTimeout(activeTimers.get(id));
    activeTimers.delete(id);
  }
}

/**
 * Repeatedly calls a function or executes a code snippet, with a fixed time
 * delay between each call.
 *
 * @param {Function} callback - A function to be executed every `delay` milliseconds.
 * @param {Number} delay - The milliseconds the timer should delay in between executions.
 * @param {...any} [args] - Additional arguments which are passed through to the function.
 * @returns {Number} The ID which identifies the timer created.
 */
export function setInterval(callback, delay, ...args) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's boundaries.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  // Check if callback is a valid function.
  assert.isFunction(callback);

  // Pin down the correct ID value.
  const id = nextId++;
  const timer = binding.createTimeout(callback, delay, true, args);

  // Update `activeTimers` map.
  activeTimers.set(id, timer);

  return id;
}

/**
 * The global clearInterval() method cancels an interval previously established
 * by calling setInterval().
 *
 * @param {Number} id - The ID which identifies the timer.
 */
export function clearInterval(id) {
  clearTimeout(id);
}

/**
 * Schedules the "immediate" execution of the callback after the I/O phase.
 *
 * @param {Function} callback - A function to be executed after the I/O (`poll`) phase.
 * @param  {...any} [args] - Additional arguments which are passed through to the function.
 * @returns {Number} The ID which identifies the timer created.
 */
export function setImmediate(callback, ...args) {
  // Check arg type.
  assert.isFunction(callback);

  // Pin down the correct ID value.
  const id = nextId++;
  const immediate = binding.createImmediate(() => {
    callback(...args);
    activeTimers.delete(id);
  });

  // Update `activeTimers` map.
  activeTimers.set(id, immediate);

  return id;
}

/**
 * Cancels an Immediate timer created by setImmediate().
 *
 * @param {Number} id - The ID which identifies the timer.
 */
export function clearImmediate(id) {
  // Check parameter's type.
  assert.integer(id);

  if (activeTimers.has(id)) {
    binding.removeImmediate(activeTimers.get(id));
    activeTimers.delete(id);
  }
}

export default {
  setTimeout,
  setInterval,
  setImmediate,
  clearTimeout,
  clearInterval,
  clearImmediate,
};
