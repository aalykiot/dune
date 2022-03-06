import { Console } from 'console';
import { setTimeout, setInterval, clearTimeout, clearInterval } from 'timers';

/**
 * Initializing global aliases.
 */

globalThis.global = globalThis;

global.GLOBAL = globalThis;
global.root = globalThis;

/**
 * Clones a provided function.
 *
 * @param {Function} fn - the function we want to clone.
 * @returns {Function}
 */
function cloneFunction(fn) {
  let that = fn;
  let temp = function temporary() {
    return that.apply(this, arguments);
  };
  for (let key in this) {
    if (this.hasOwnProperty(key)) {
      temp[key] = this[key];
    }
  }
  return temp;
}

/**
 * Initializing internal bindings.
 */

let cache = new Map();
let internalBinding = cloneFunction(process.binding);

process.binding = function (name) {
  // Check if binding exists in cache.
  if (cache.has(name)) return cache.get(name);
  // Load binding using the internal call, and save it to cache.
  const binding = internalBinding(name);
  cache.set(name, binding);

  return binding;
};

/**
 * Initializing the stdout stream.
 */

let stdout = process.stdout;

Object.defineProperty(process, 'stdout', {
  get() {
    // Do not initialize stdout more than once.
    if (stdout) return stdout;
    // Setup the stdout stream.
    const binding = process.binding('stdio');
    return {
      write(value) {
        binding.write(value);
      }
    };
  },
  configurable: true
});

/**
 * Initializing the stderr stream.
 */

let stderr = process.stderr;

Object.defineProperty(process, 'stderr', {
  get() {
    // Do not initialize stderr more than once.
    if (stderr) return stderr;
    // Setup the stdout stream.
    const binding = process.binding('stdio');
    return {
      write(value) {
        binding.writeError(value);
      }
    };
  },
  configurable: true
});

/**
 * Initializing console to global scope.
 */

global.console = new Console();

/**
 * Initializing DOM style timers to global scope.
 */

global.setTimeout = setTimeout;
global.setInterval = setInterval;
global.clearTimeout = clearTimeout;
global.clearInterval = clearInterval;
