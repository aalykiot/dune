'use strict';

globalThis.global = globalThis;
global.GLOBAL = globalThis;
global.root = globalThis;

Function.prototype.clone = function () {
  let that = this;
  let temp = function temporary() {
    return that.apply(this, arguments);
  };
  for (let key in this) {
    if (this.hasOwnProperty(key)) {
      temp[key] = this[key];
    }
  }
  return temp;
};

let cache = new Map();
let internalBinding = process.binding.clone();

Object.defineProperty(process, 'binding', {
  get() {
    return (name) => {
      // Check if binding exists in cache.
      if (cache.has(name)) return cache.get(name);
      // Load binding using the internal call, and save it to cache.
      const binding = internalBinding(name);
      cache.set(name, binding);

      return binding;
    };
  }
});

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
