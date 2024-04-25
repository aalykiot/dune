import { EventEmitter } from 'events';
import { cloneFunction as clone } from 'util';

const cache = new Map();

const internalBinding = clone(process.binding);
const kill = clone(process.kill);
const nextTick = clone(process.nextTick);

// Note: Integrating a caching layer into process.binding enables us
// to avoid traversing the JavaScript - Rust bridge for native method
// access on every occasion.
process.binding = (name) => {
  // Check bindings cache.
  if (cache.has(name)) return cache.get(name);

  // Load binding (from rust), save it to cache.
  const binding = internalBinding(name);
  cache.set(name, binding);

  return binding;
};

process.kill = (pid, signal = 'SIGKILL') => {
  // Check arguments.
  if (!pid || Number.isNaN(Number.parseInt(pid))) {
    throw new TypeError(`The "pid" argument must be of type number.`);
  }
  kill(pid, signal);
};

process.nextTick = (callback, ...args) => {
  // Check if callback is a valid function.
  if (typeof callback !== 'function') {
    throw new TypeError(`The "callback" argument must be of type function.`);
  }
  nextTick(() => callback(...args));
};

function defineStream(name, getter) {
  Object.defineProperty(process, name, {
    get: getter,
    configurable: true,
    enumerable: true,
  });
}

const io = process.binding('stdio');

defineStream('stdout', () => ({
  write: io.write,
  end() {},
}));

defineStream('stdin', () => ({
  read: io.read,
}));

defineStream('stderr', () => ({
  write: io.writeError,
}));

const os = process.binding('signals');

// Note: To transform the process object, initialized in Rust, into
// an event emitter, we must manually instantiate the object fields
// and extend the prototype.
process._events = {};
process._eventsCount = 0;

Object.setPrototypeOf(process, EventEmitter.prototype);

const activeSignals = new Map();
const isSignal = (type) => os.signals.includes(type);

const signalEmitFunction = (type) => {
  process.emit(type);
  if (process.listenerCount(type) === 0) {
    stopListeningIfNoListener(type);
  }
};

function startListeningIfSignal(type) {
  // Check if the type is a valid signal.
  if (!isSignal(type) || activeSignals.has(type)) return;

  // Define a handler function for the signal.
  const callback = signalEmitFunction.bind(this, type);
  const signal = os.startSignal(type, callback);

  activeSignals.set(type, signal);
}

function stopListeningIfNoListener(type) {
  // Retrieve the internal ID of the signal.
  const signal = activeSignals.get(type);

  // Remove the signal.
  if (signal && process.listenerCount(type) === 0) {
    os.cancelSignal(signal);
    activeSignals.delete(type);
  }
}

const exceptions = process.binding('exceptions');

const exceptionEmitFunction = (type, ...args) => {
  // Emit the event.
  process.emit(type, ...args);
  // Remove captures if no listeners.
  if (process.listenerCount(type) === 0) {
    setCapturesIfExceptionEvent(type, true);
  }
};

function setCapturesIfExceptionEvent(type, unset = false) {
  // Note: We use the same function for both setting and removing the internal
  // capture callbacks. To remove one, simply pass null as the JS callback.
  const cb = !unset ? exceptionEmitFunction.bind(this, type) : null;

  // https://nodejs.org/docs/latest/api/process.html#event-uncaughtexception
  if (type === 'uncaughtException') {
    exceptions.setUncaughtExceptionCallback(cb);
    return;
  }
  // https://nodejs.org/docs/latest/api/process.html#event-unhandledrejection
  if (type === 'unhandledRejection') {
    exceptions.setUnhandledRejectionCallback(cb);
    return;
  }
}

function removeCapturesIfNoListener(type) {
  // Remove the internal capture callback.
  if (process.listenerCount(type) === 0) {
    setCapturesIfExceptionEvent(type, true);
  }
}

// Note: To ensure the full functionality, it's essential to 'override'
// specific methods inherited from the EventEmitter prototype.

for (const method of ['on', 'once']) {
  process[method] = (event, ...args) => {
    EventEmitter.prototype[method].call(process, event, ...args);
    startListeningIfSignal(event);
    setCapturesIfExceptionEvent(event);
  };
}

for (const method of ['removeListener', 'removeAllListeners']) {
  process[method] = (event, ...args) => {
    EventEmitter.prototype[method].call(process, event, ...args);
    stopListeningIfNoListener(event);
    removeCapturesIfNoListener(event);
  };
}

export default process;
