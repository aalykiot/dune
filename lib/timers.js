// DOM Style Timers
//
// The Timers API provides functionality to allow developers to create DOM style timers.
// https://www.w3schools.com/js/js_timing.asp

const TIMEOUT_MAX = Math.pow(2, 31) - 1;

const { createTimeout, destroyTimeout } = process.binding('timers');

let nextId = 1;

/**
 * This map keeps at sync the JavaScript timer IDs and their equivalent Rust
 * timer indexes (resource IDs) for all currently active timers.
 *
 * @type {Map<number, number>}
 */
export const activeTimers = new Map();

/**
 * Sets a timer which executes a function or specified piece of code once the timer expires.
 *
 * @param {Function} callback - a function to be executed after the timer expires.
 * @param {Number} delay - the time, in milliseconds that the timer should wait before the specified function or code is executed.
 * @param {Array|undefined} args - additional arguments which are passed through to the function.
 *
 * @returns {Number}
 */
function setTimeout(callback, delay, ...args) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's boundaries.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  // Check if callback is a valid function.
  if (typeof callback !== 'function') {
    throw new TypeError(`The "callback" argument must be of type function.`);
  }

  return initializeTimer(callback, delay, args, false);
}

/**
 * The global clearTimeout() method cancels a timeout previously established by calling setTimeout().
 *
 * @param {Number} id - the identifier of the timeout you want to cancel.
 * @returns {void}
 */
function clearTimeout(id) {
  if (typeof id !== 'number') {
    throw new TypeError(`The "timeout" argument must be of type number.`);
  }
  if (!activeTimers.has(id)) return;

  destroyTimeout(activeTimers.get(id));
  activeTimers.delete(id);
}

/**
 * Repeatedly calls a function or executes a code snippet, with a fixed time delay between each call.
 *
 * @param {Function} callback - a function to be executed every delay milliseconds.
 * @param {Number} delay - the time, in milliseconds, the timer should delay in between executions of the specified function or code.
 * @param {Array|undefined} args - additional arguments which are passed through to the function.
 *
 * @returns {Number}
 */
function setInterval(callback, delay, ...args) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's boundaries.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  // Check if callback is a valid function.
  if (typeof callback !== 'function') {
    throw new TypeError(`The "callback" argument must be of type function.`);
  }

  return initializeTimer(callback, delay, args, true);
}

/**
 * The global clearInterval() method cancels an interval previously established by calling setInterval().
 *
 * @param {Number} id - the identifier of the interval you want to cancel.
 */
function clearInterval(id) {
  clearTimeout(id);
}

/**
 * Initializes a timeout or an interval based on the receiving parameters.
 *
 * @param {Function} callback - a function to be executed in a latter time.
 * @param {Number} delay - the time, in milliseconds that the timer should wait before the specified function or code is executed.
 * @param {Array|undefined} args - additional arguments which are passed through to the function.
 *
 * @param {Boolean|undefined} repeat - identifies if the timer is an interval.
 * @param {Number|undefined} prevId - the ID given to the timer in a previous iteration.
 *
 * @returns {Number}
 */
function initializeTimer(callback, delay, args, repeat, prevId) {
  const id = prevId ?? nextId++;
  const task = () => {
    // We're handling repeated timers (aka intervals) by continuously creating
    // new event-loop timers and keeping the JS timer ID constant.
    if (repeat) {
      callback(...args);
      if (activeTimers.has(id)) {
        initializeTimer(callback, delay, args, true, id);
      }
      return;
    }
    // This branch executes on one-off timers (aka timeouts).
    callback(...args);
    activeTimers.delete(id);
  };

  // Schedule a new timer to the event-loop and update the `activeTimers` map.
  const index = createTimeout(task, delay);
  activeTimers.set(id, index);

  return id;
}

export { setTimeout, setInterval, clearTimeout, clearInterval };
