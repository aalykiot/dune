import { Console } from 'console';
import { setTimeout, setInterval, clearTimeout, clearInterval } from 'timers';
import { cloneFunction } from 'util';

/**
 * Initializing global aliases.
 */

globalThis.global = globalThis;

global.GLOBAL = globalThis;
global.root = globalThis;

/**
 * Initializing internal bindings.
 */

let cache = new Map();
let internalBinding = cloneFunction(process.binding);

process.binding = function (name) {
  // Check bindings cache.
  if (cache.has(name)) return cache.get(name);

  // Load binding (from rust), save it to cache.
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
    // Don't initialize twice.
    if (stdout) return stdout;

    // Setup stdout stream.
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
    // Don't initialize twice.
    if (stderr) return stderr;

    // Setup stderr stream.
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
