// Console APIs
//
// The Console APIs provide functionality to allow developers to perform debugging tasks,
// such as logging messages or the values of variables at set points in your code.
//
// https://developer.mozilla.org/en-US/docs/Web/API/console

/* eslint-disable no-control-regex */

import { performance } from 'perf_hooks';
import { green, yellow, cyan, bright_black } from 'colors';

// Returns a string with as many spaces as the parameter specified.
function pre(amount) {
  return ' '.repeat(amount);
}

/**
 * Stringifies almost all JavaScript built-in types.
 *
 * @param {*} value
 * @param {WeakSet} seen
 * @param {number} depth
 * @returns {string}
 */

function stringify(value, seen, depth = 0) {
  switch (typeof value) {
    case 'string':
      return depth > 0 ? green(`"${value}"`) : value;
    case 'number':
    case 'boolean':
      return yellow(String(value));
    case 'undefined':
      return bright_black(String(value));
    case 'symbol':
      return green(String(value));
    case 'bigint':
      return yellow(String(value) + 'n');
    case 'object':
      return !value ? 'null' : stringifyObject(value, seen, ++depth);
    case 'function':
      return !value.name
        ? cyan('[Function (anonymous)]')
        : cyan(`[Function: ${value.name}]`);
    default:
      return '[Unknown]';
  }
}

function isArray(value) {
  return Array.isArray(value);
}

function stringifyArray(arr, seen, depth) {
  // Checks if all the elements in the array have the same type.
  const firstElementType = typeof arr[0];
  const isUniform = arr.every((elem) => typeof elem === firstElementType);

  const entries = [];
  for (const elem of arr) {
    entries.push(stringify(elem, seen, depth));
  }

  // Multiline formatting.
  if (entries.join('').length > 50) {
    const start = '[\n';
    const end = `\n${pre((depth - 1) * 2)}]`;
    const entriesPretty = prettifyArray(entries, depth, isUniform);
    return `${start}${entriesPretty}${end}`;
  }

  // Inline formatting.
  return entries.length > 0 ? `[ ${entries.join(', ')} ]` : `[]`;
}

function isTypedArray(value) {
  switch (Object.prototype.toString.call(value)) {
    case '[object Int8Array]':
    case '[object Uint8Array]':
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

function prettifyArray(arr, depth = 0, isUniform) {
  // Remove the color characters so we can calculate the AVG and MAX correctly.
  const uncolored = arr.map(
    (elem) => elem.replace(/\u001b\[[0-9;]*m/g, '').length
  );

  const maxElementLength = Math.max(...uncolored);
  const avgElementLength = uncolored.reduce((a, b) => a + b) / uncolored.length;

  // Calculate the grid size (trying to make perfect squares and minimizing empty space)
  // or max out at 12xN;
  const maxElementsPerRow = Math.min(
    Math.max(
      Math.floor((Math.sqrt(arr.length) * avgElementLength) / maxElementLength),
      1
    ),
    12
  );

  // Tries to align the columns.
  const alignColumn = (elem, i) => {
    const length = elem.replace(/\u001b\[[0-9;]*m/g, '').length;
    const shift = maxElementLength - length;
    if (isUniform) {
      return i === arr.length - 1
        ? pre(shift) + elem
        : pre(shift) + elem + ', ';
    } else {
      return i === arr.length - 1
        ? elem + pre(shift)
        : elem + ', ' + pre(shift);
    }
  };

  // Creates rows of length `maxElementsPerRow`.
  const groupRows = (acc, elem, i) => {
    if (acc.atRow === maxElementsPerRow || i === 0) {
      acc.list.push([elem]);
      acc.atRow = 0;
    } else {
      acc.list[acc.list.length - 1].push(elem);
      acc.atRow++;
    }
    return acc;
  };

  // Indents row based on the depth we're currently in.
  const indentRow = (row) => pre(depth * 2) + row.join('');

  let output = arr.map(alignColumn);

  output = output.reduce(groupRows, { atRow: 0, list: [] });
  output = output.list.map(indentRow).join('\n');

  return output;
}

function stringifyTypedArray(arr, depth = 0) {
  // Colorize internal values.
  let pretty = arr
    .toString()
    .split(',')
    .map((elem) => yellow(elem));

  // Get typed-array's specific type.
  const type = Object.prototype.toString
    .call(arr)
    .replace('[object ', '')
    .replace(']', '');

  if (pretty.length > 50) {
    pretty = prettifyArray(pretty, depth, true);
    return `${type}(${arr.length}) [\n${pretty}\n${pre((depth - 1) * 2)}]`;
  }

  return `${type}(${arr.length}) [ ${pretty.join(', ')} ]`;
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
    return stringifyTypedArray(value, depth);
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
   * Clears the console if the environment allows it.
   */

  clear() {
    try {
      process.binding('stdio').clear();
    } catch (e) {
      this.warn('This environment does not support console clearing');
    }
  }

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
