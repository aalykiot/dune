// DOM Style Timers
//
// The Timers API provides functionality to allow developers to create DOM style timers.
// https://www.w3schools.com/js/js_timing.asp

const TIMEOUT_MAX = Math.pow(2, 31) - 1;

const { createTimeout, removeTimeout } = process.binding('timer_wrap');

let counter = 1; // global timers count.

/**
 * Sets a timer which executes a function or specified piece of code once the timer expires.
 *
 * @param {Function} callback - A function to be executed after the timer expires.
 * @param {Number} delay - The time, in milliseconds that the timer should wait before the specified function or code is executed.
 * @returns {Number}
 */
function setTimeout(callback, delay) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's bounds.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  let args;
  switch (arguments.length) {
    case 0:
    case 1:
    case 2:
      break;
    case 3:
      args = [arguments[2]];
      break;
    case 4:
      args = [arguments[2], arguments[3]];
      break;
    case 5:
      args = [arguments[2], arguments[3], arguments[4]];
      break;
    // Extend arguments dynamically.
    default:
      args = Object.keys(arguments).reduce((acc, i) => {
        if (i < 2) return acc; // we don't care about the first 2 arguments.
        return [...acc, arguments[i]];
      }, []);
      break;
  }

  // Return timer's ID.
  return createTimeout(counter++, callback, delay, args, false);
}

/**
 * The global clearTimeout() method cancels a timeout previously established by calling setTimeout().
 *
 * @param {Number} timeout - The identifier of the timeout you want to cancel.
 * @returns
 */
function clearTimeout(timeout) {
  if (!timeout) return;
  removeTimeout(timeout);
}

/**
 * Repeatedly calls a function or executes a code snippet, with a fixed time delay between each call.
 *
 * @param {Function} callback - A function to be executed every delay milliseconds.
 * @param {Number} delay - The time, in milliseconds, the timer should delay in between executions of the specified function or code.
 * @returns {Number}
 */
function setInterval(callback, delay) {
  // Coalesce to number or NaN.
  delay *= 1;

  // Check delay's bounds.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  let args;
  switch (arguments.length) {
    case 0:
    case 1:
    case 2:
      break;
    case 3:
      args = [arguments[2]];
      break;
    case 4:
      args = [arguments[2], arguments[3]];
      break;
    case 5:
      args = [arguments[2], arguments[3], arguments[4]];
      break;
    // Extend arguments dynamically.
    default:
      args = Object.keys(arguments).reduce((acc, i) => {
        if (i < 2) return acc; // we don't care about the first 2 arguments.
        return [...acc, arguments[i]];
      }, []);
      break;
  }

  // Return timer's ID.
  return createTimeout(counter++, callback, delay, args, true);
}

/**
 * The global clearInterval() method cancels an interval previously established by calling setInterval().
 *
 * @param {Timer} interval - The identifier of the interval you want to cancel.
 * @returns
 */
function clearInterval(interval) {
  if (!interval) return;
  clearTimeout(interval);
}

export { setTimeout, setInterval, clearTimeout, clearInterval };
