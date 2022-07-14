// Console APIs
//
// The Console APIs provide functionality to allow developers to perform debugging tasks,
// such as logging messages or the values of variables at set points in your code.
//
// https://developer.mozilla.org/en-US/docs/Web/API/console

/**
 * Returns a string with as many spaces as the parameter specified.
 *
 * @param {string} amount
 * @returns {string}
 */

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

/**
 * Checks if given object is an Array.
 *
 * @param {*} o
 * @returns {Bool}
 */

function isArray(o) {
  return Array.isArray(o);
}

/**
 * Stringifies JavaScript arrays.
 *
 * @param {*} value
 * @param {WeakSet} seen
 * @param {Number} depth
 * @returns {String}
 */

function stringifyArray(value, seen, depth) {
  const entries = [];
  for (const elem of value) {
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

/**
 * Checks if given object is a TypedArray.
 *
 * @param {*} o
 * @returns {bool}
 */

function isTypedArray(o) {
  switch (Object.prototype.toString.call(o)) {
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

/**
 * Stringifies JavaScript TypedArrays.
 *
 * @param {*} o
 * @returns {string}
 */

function stringifyTypedArray(o) {
  const type = Object.prototype.toString
    .call(o)
    .replace('[object ', '')
    .replace(']', '');
  return `${type}(${o.length}) [ ${o.toString().split(',').join(', ')} ]`;
}

/**
 * Checks if given object is a Date object.
 *
 * @param {*} o
 * @returns {bool}
 */

function isDate(o) {
  return Object.prototype.toString.call(o) === '[object Date]';
}

/**
 * Stringifies JavaScript Date objects.
 *
 * @param {*} o
 * @returns {string}
 */

function stringifyDate(o) {
  return o.toISOString();
}

/**
 * Checks if given object is a RexExp.
 *
 * @param {*} o
 * @returns {bool}
 */

function isRexExp(o) {
  return Object.prototype.toString.call(o) === '[object RegExp]';
}

/**
 * Stringifies JavaScript RexExp objects.
 *
 * @param {*} o
 * @returns {string}
 */

function stringifyRexExp(o) {
  return o.toString();
}

/**
 * Checks if given object is an Error object.
 *
 * @param {*} o
 * @returns {bool}
 */
function isError(o) {
  return Object.prototype.toString.call(o) === '[object Error]';
}

/**
 * Stringifies JavaScript Error objects.
 *
 * @param {*} o
 * @returns {string}
 */

function stringifyError(o) {
  return o.stack;
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
  // Stringify Array.
  if (isArray(value)) {
    return stringifyArray(value, seen, depth);
  }
  // Stringify TypedArray.
  if (isTypedArray(value)) {
    return stringifyTypedArray(value);
  }
  // Stringify Date.
  if (isDate(value)) {
    return stringifyDate(value);
  }
  // Stringify RegExp.
  if (isRexExp(value)) {
    return stringifyRexExp(value);
  }
  // Stringify Errors.
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
  /**
   * Outputs data to the stdout stream.
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
}

export { Console };
