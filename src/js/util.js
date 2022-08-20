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

export function parseEnvVariable(variable) {
  // Stringify variable.
  const value = variable.toString().trim();

  // If the value is wrapped in `'`, `"` or backtick return it's value.
  if (
    (value.indexOf(`'`) === 0 && value.lastIndexOf(`'`) === value.length - 1) ||
    (value.indexOf(`"`) === 0 && value.lastIndexOf(`"`) === value.length - 1) ||
    (value.indexOf('`') === 0 && value.lastIndexOf('`') === value.length - 1)
  ) {
    return value.slice(1, value.length - 1);
  }

  // If value ends with an asterisk ignore further parsing.
  if (value.lastIndexOf('*') === value.length - 1 && !value.includes(',')) {
    return value.slice(0, value.length - 1);
  }

  // Boolean parsing.
  if (value.toLowerCase() === 'true' || value.toLowerCase() === 'false') {
    return value.toLowerCase() === 'true';
  }

  // Number parsing.
  if (value !== '' && !Number.isNaN(Number(value))) {
    return Number(value);
  }

  // Array parsing.
  if (Array.isArray(value) || value.includes(',')) {
    return value
      .split(',')
      .filter((str) => str !== '')
      .map(parseEnvVariable);
  }

  return value;
}
