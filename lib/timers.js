import BTree from '_internals/btree';

// ===============================================

let id = 1;

class Timer {
  constructor(delay, repeat = false) {
    this.id = id++;
    this.delay = delay;
    this.repeat = repeat;
  }
  static now() {
    return new Date().getTime();
  }
}

// ===============================================

const TIMEOUT_MAX = Math.pow(2, 31) - 1;

// B-tree is a self-balancing tree data structure that maintains sorted data and allows searches,
// sequential access, insertions, and deletions in logarithmic time.

const L = new BTree();

/**
 * Sets a timer which executes a function or specified piece of code once the timer expires.
 *
 * @param {fn} callback - A function to be executed after the timer expires.
 * @param {number} delay - The time, in milliseconds that the timer should wait before the specified function or code is executed.
 * @returns {Timer}
 */
function setTimeout(callback, delay) {
  // Coalesce to number or NaN.
  delay *= 1;
  // Make sure the delay is bigger than zero and smaller than TIMEOUT_MAX.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  const timer = new Timer(delay);

  let onTimeout = callback;

  // Micro optimizing timeout callback.
  switch (arguments.length) {
    case 0:
    case 1:
    case 2:
      break;
    case 3:
      onTimeout = () => callback.call(timer, arguments[2]);
      break;
    case 4:
      onTimeout = () => callback.call(timer, arguments[2], arguments[3]);
      break;
    case 5:
      onTimeout = () =>
        callback.call(timer, arguments[2], arguments[3], arguments[4]);
      break;
    // Let's handle the slow case.
    default:
      const args = Object.keys(arguments).reduce((acc, i) => {
        if (i < 2) return acc; // we don't care about the first 2 arguments.
        return [...acc, arguments[i]];
      }, []);
      onTimeout = () => callback.apply(timer, args);
      break;
  }
  timer.callback = onTimeout;

  // Appends a timer onto the timers shorted list (btree).
  L.set(Timer.now() + delay, timer);

  return timer;
}

/**
 * The global clearTimeout() method cancels a timeout previously established by calling setTimeout().
 *
 * @param {Timer} timeout - The identifier of the timeout you want to cancel.
 * @returns
 */
function clearTimeout(timeout) {
  if (!timeout) return;

  L.editRange(L.minKey(), L.maxKey(), true, (_, timer) => {
    if (timer === timeout) {
      return { delete: true, break: true };
    }
  });
}

/**
 * Repeatedly calls a function or executes a code snippet, with a fixed time delay between each call.
 *
 * @param {fn} callback - A function to be executed every delay milliseconds.
 * @param {number} delay - The time, in milliseconds, the timer should delay in between executions of the specified function or code.
 * @returns {Timer}
 */
function setInterval(callback, delay) {
  // Coalesce to number or NaN.
  delay *= 1;
  // Make sure the delay is bigger than zero and smaller than TIMEOUT_MAX.
  if (!(delay >= 1 && delay <= TIMEOUT_MAX)) {
    delay = 1;
  }

  const timer = new Timer(delay, true);

  let onTimeout = callback;

  // Micro optimizing timeout callback.
  switch (arguments.length) {
    case 0:
    case 1:
    case 2:
      break;
    case 3:
      onTimeout = () => callback.call(timer, arguments[2]);
      break;
    case 4:
      onTimeout = () => callback.call(timer, arguments[2], arguments[3]);
      break;
    case 5:
      onTimeout = () =>
        callback.call(timer, arguments[2], arguments[3], arguments[4]);
      break;
    // Let's handle the slow case.
    default:
      const args = Object.keys(arguments).reduce((acc, i) => {
        if (i < 2) return acc; // we don't care about the first 2 arguments.
        return [...acc, arguments[i]];
      }, []);
      onTimeout = () => callback.apply(timer, args);
      break;
  }
  timer.callback = onTimeout;

  // Appends a timer onto the timers shorted list (btree).
  L.set(Timer.now() + delay, timer);

  return timer;
}

/**
 * The global clearInterval() method cancels an interval previously established by calling setInterval().
 *
 * @param {Timer} interval - The identifier of the interval you want to cancel.
 * @returns
 */
function clearInterval(interval) {
  if (!interval) return;

  L.editRange(L.minKey(), L.maxKey(), true, (_, timer) => {
    if (timer === interval) {
      timer.repeat = false;
      return { delete: true, break: true };
    }
  });
}

/**
 * Processes expired timers when called.
 *
 * @returns {number} - the future time the most recent timer should run.
 */
function processTimers() {
  // Orphan timers to handle at the end of the function.
  const orphanTimers = [];

  L.forRange(L.minKey(), Timer.now(), true, (key, timer, i) => {
    // If timers btree is empty or we reached the timers execution limit, stop!
    if (L.isEmpty || i >= 20) return { break: true };

    timer.callback();
    orphanTimers.push(timer);
    // Delete timer from the timers shorted list.
    L.delete(key);
  });

  // Reschedule repeatable timers.
  orphanTimers
    .filter((t) => t.repeat)
    .forEach((timer) => {
      L.set(Timer.now() + timer.delay, timer);
    });

  return L.minKey();
}

// ========= MAIN =========

setTimeout(() => {
  console.log('Hello!');
}, 5000);

while (processTimers() !== undefined) {}
