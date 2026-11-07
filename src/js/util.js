/* eslint-disable no-prototype-builtins */

export function cloneFunction(fn) {
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

export function toUint8Array(data, encoding) {
  if (!(data instanceof Uint8Array)) {
    return new TextEncoder(encoding).encode(data);
  }
  return data;
}

export function makeDeferredPromise() {
  // Extract the resolve method from the promise.
  const promiseExt = {};
  const promise = new Promise((resolve, reject) => {
    promiseExt.resolve = resolve;
    promiseExt.reject = reject;
  });

  return { promise, promiseExt };
}
