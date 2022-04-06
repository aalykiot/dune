/**
 * Clones a provided function.
 *
 * @param {Function} fn - the function we want to clone.
 * @returns {Function}
 */
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
