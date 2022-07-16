// Console APIs
//
// The Console APIs provide functionality to allow developers to perform debugging tasks,
// such as logging messages or the values of variables at set points in your code.
//
// https://developer.mozilla.org/en-US/docs/Web/API/console

import { performance } from 'perf_hooks';

// Returns a string with as many spaces as the parameter specified.
function pre(amount) {
  return ' '.repeat(amount);
}

/**
 * Transforms a JavaScript object/primitive into a string.
 *
 * @param {*} value
 * @param {WeakSet} seen
 * @param {Number} depth
 * @returns {String}
 * @public
 */

function stringify(value, seen, depth = 0) {
  switch (typeof value) {
    case 'string':
      return depth > 0 ? `"${value}"` : value;
    case 'number':
    case 'undefined':
    case 'boolean':
    case 'symbol':
      return String(value);
    case 'bigint': {
      return String(value) + 'n';
    }
    case 'object':
      return !value ? 'null' : stringifyObject(value, seen, ++depth);
    case 'function':
      return !value.name
        ? '[Function (anonymous)]'
        : `[Function: ${value.name}]`;
    default:
      return '[Unknown]';
  }
}

function isArray(value) {
  return Array.isArray(value);
}

function stringifyArray(arr, seen, depth) {
  const entries = [];
  for (const elem of arr) {
    entries.push(stringify(elem, seen, depth));
  }

  // Multiline formatting.
  if (entries.join('').length > 50) {
    const start = '[\n';
    const end = `\n${pre((depth - 1) * 2)}]`;
    const entriesPretty = entries.map((v) => `${pre(depth * 2)}${v}`);
    return `${start}${entriesPretty.join(',\n')}${end}`;
  }

  // Inline formatting.
  return entries.length > 0 ? `[ ${entries.join(', ')} ]` : `[]`;
}

function isTypedArray(value) {
  switch (Object.prototype.toString.call(value)) {
    case '[object Int8Array]':
    case '[object Uint8Array]':
    case '[object Uint8ClampedArray]':
    case '[object Uint8ClampedArray]':
    case '[object Uint16Array]':
    case '[object Int32Array]':
    case '[object Uint32Array]':
    case '[object Float32Array]':
    case '[object Float64Array]':
      return true;
    default:
      return false;
  }
}

function stringifyTypedArray(arr) {
  const pretty = arr.toString().split(',').join(', ');
  const type = Object.prototype.toString
    .call(arr)
    .replace('[object ', '')
    .replace(']', '');

  return `${type}(${arr.length}) [ ${pretty} ]`;
}

function isDate(value) {
  return Object.prototype.toString.call(value) === '[object Date]';
}

function stringifyDate(date) {
  return date.toISOString();
}

function isRexExp(value) {
  return Object.prototype.toString.call(value) === '[object RegExp]';
}

function stringifyRexExp(exp) {
  return exp.toString();
}

function isError(value) {
  return Object.prototype.toString.call(value) === '[object Error]';
}

function stringifyError(error) {
  return error.stack;
}

/**
 * Specifically stringifies JavaScript objects.
 *
 * @param {*} value
 * @param {WeakSet} seen
 * @param {number} depth
 * @returns {string}
 */

function stringifyObject(value, seen = new WeakSet(), depth) {
  // We have to check the type of the value parameter to decide which stringify
  // transformer we should use.
  if (isArray(value)) {
    return stringifyArray(value, seen, depth);
  }

  if (isTypedArray(value)) {
    return stringifyTypedArray(value);
  }

  if (isDate(value)) {
    return stringifyDate(value);
  }

  if (isRexExp(value)) {
    return stringifyRexExp(value);
  }

  if (isError(value)) {
    return stringifyError(value);
  }

  // Looks like it's a regular object.
  const entries = [];
  for (const key of Object.keys(value)) {
    if (seen.has(value[key])) {
      entries.push(`${pre(depth * 2)}${key}: [Circular]`);
      continue;
    }
    seen.add(value);
    entries.push(
      `${pre(depth * 2)}${key}: ${stringify(value[key], seen, depth)}`
    );
  }

  // Apply multi-line formatting on long properties.
  if (entries.map((v) => v.trim()).join('').length > 50) {
    const start = '{\n';
    const end = `\n${pre((depth - 1) * 2)}}`;
    return `${start}${entries.join(',\n')}${end}`;
  }

  // Inline formatting.
  const entriesPretty = entries.map((v) => v.trim());
  return entries.length > 0 ? `{ ${entriesPretty.join(', ')} }` : `{}`;
}

/**
 * Console is a subset implementation of MDN's Console API.
 */
class Console {
  // Holds timers initialized by console.
  // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
  #timers = new Map();

  /**
   * Outputs data to the stdout stream.
   *
   * @param  {...any} args
   */

  log(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stdout.write(`${output}\n`);
  }

  info = this.log;
  debug = this.log;

  /**
   * Same as console.log but prepends the output with "WARNING".
   *
   * @param  {...any} args
   */

  warn(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stderr.write(`WARNING: ${output}\n`);
  }

  error = this.warn;

  /**
   * Starts a timer you can use to track how long an operation takes.
   *
   * @param String label
   */

  time(label = 'default') {
    if (this.#timers.has(label)) {
      this.warn(`Timer '${label}' already exists`);
      return;
    }

    this.#timers.set(label, performance.now());
  }

  /**
   * Logs the current value of a timer that was previously started by calling
   * console.time() to the console.
   *
   * @param String label
   */

  timeLog(label = 'default') {
    if (!this.#timers.has(label)) {
      this.warn(`Timer '${label}' does not exist`);
      return;
    }

    const difference = performance.now() - this.#timers.get(label);
    this.log(`${label}: ${difference} ms`);
  }

  /**
   * Stops a timer that was previously started by calling console.time().
   *
   * @param String label
   */

  timeEnd(label = 'default') {
    if (!this.#timers.has(label)) {
      this.warn(`Timer '${label}' does not exist`);
      return;
    }

    this.timeLog(label);
    this.#timers.delete(label);
  }
}

export { Console };
