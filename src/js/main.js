import timers from 'timers';
import fetch from '@web/fetch';
import structuredClone from '@web/clone';
import { cloneFunction } from 'util';
import { Console, prompt, wrapConsole } from 'console';
import { AbortController, AbortSignal } from '@web/abort';
import { TextEncoder, TextDecoder } from '@web/text_encoding';

globalThis.global = globalThis;

global.GLOBAL = globalThis;
global.root = globalThis;

function makeGlobal(name, value) {
  globalThis[name] = value;
}

// Note: Adding a caching layer to `process.binding` allows us to not
// cross the JavaScript <-> Rust bridge every time we need native methods.

let cache = new Map();
let internalBinding = cloneFunction(process.binding);

process.binding = (name) => {
  // Check bindings cache.
  if (cache.has(name)) return cache.get(name);

  // Load binding (from rust), save it to cache.
  const binding = internalBinding(name);
  cache.set(name, binding);

  return binding;
};

const kill = cloneFunction(process.kill);

process.kill = (pid, signal = 'SIGKILL') => {
  // Check arguments.
  if (!pid || Number.isNaN(Number.parseInt(pid))) {
    throw new TypeError(`The "pid" argument must be of type number.`);
  }
  kill(pid, signal);
};

const nextTick = cloneFunction(process.nextTick);

process.nextTick = (callback, ...args) => {
  // Check if callback is a valid function.
  if (typeof callback !== 'function') {
    throw new TypeError(`The "callback" argument must be of type function.`);
  }
  nextTick(() => callback(...args));
};

/* Setting up STDOUT, STDIN and STDERR streams. */

Object.defineProperty(process, 'stdout', {
  get() {
    return {
      write: process.binding('stdio').write,
      end() {},
    };
  },
  configurable: true,
});

Object.defineProperty(process, 'stdin', {
  get() {
    return {
      read: process.binding('stdio').read,
    };
  },
  configurable: true,
});

Object.defineProperty(process, 'stderr', {
  get() {
    return {
      write: process.binding('stdio').writeError,
    };
  },
  configurable: true,
});

const console = new Console();
const consoleFromV8 = globalThis['console'];

wrapConsole(console, consoleFromV8);

/* Initialize global environment for user script */

makeGlobal('console', console);
makeGlobal('prompt', prompt);

makeGlobal('setTimeout', timers.setTimeout);
makeGlobal('setInterval', timers.setInterval);
makeGlobal('setImmediate', timers.setImmediate);
makeGlobal('clearTimeout', timers.clearTimeout);
makeGlobal('clearInterval', timers.clearInterval);
makeGlobal('clearImmediate', timers.clearImmediate);

makeGlobal('TextEncoder', TextEncoder);
makeGlobal('TextDecoder', TextDecoder);
makeGlobal('structuredClone', structuredClone);
makeGlobal('AbortController', AbortController);
makeGlobal('AbortSignal', AbortSignal);
makeGlobal('fetch', fetch);
