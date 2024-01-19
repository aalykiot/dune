/**
 * Console APIs
 *
 * The Console APIs provide functionality to allow developers to perform debugging tasks,
 * such as logging messages or the values of variables at set points in your code.
 *
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/console}
 *
 * @module Console
 */

/* eslint-disable no-control-regex */

import { performance } from 'perf_hooks';
import { green, yellow, cyan, red, bright_black } from 'colors';

const { callConsole } = process.binding('stdio');

// Returns a string with as many spaces as the parameter specified.
function pre(amount) {
  return ' '.repeat(amount);
}

// Small util for objects that might not have a `.toString` method.
function objectToString(value) {
  return Object.prototype.toString.call(value);
}

/**
 * Stringifies almost all JavaScript built-in types.
 *
 * @ignore
 * @param {*} value
 * @param {WeakSet} seen
 * @param {number} depth
 * @returns {string}
 */

function stringify(value, seen, depth = 0) {
  switch (typeof value) {
    case 'string':
      return depth > 0 ? stringifyText(value) : value;
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

function stringifyText(value) {
  const text = value.length > 100 ? `${value.slice(0, 100)}...` : value;
  const textEscaped = JSON.stringify(text);
  return green(textEscaped);
}

function isArray(value) {
  return Array.isArray(value);
}

function stringifyArray(arr, seen, depth) {
  // Special formatting required if array has only numbers.
  const hasOnlyNumbers = arr.every((elem) => typeof elem === 'number');

  const entries = [];
  for (const elem of arr) {
    entries.push(stringify(elem, seen, depth));
  }

  // Remove the color characters to get the proper length.
  const uncoloredEntries = entries.join('').replace(/\u001b\[[0-9;]*m/g, '');

  // Multiline formatting.
  if (uncoloredEntries.length > 60) {
    const start = '[\n';
    const end = `\n${pre((depth - 1) * 2)}]`;
    const entriesPretty = prettifyArray(entries, depth, hasOnlyNumbers);
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

// Calculate the grid size (trying to make perfect squares and minimizing empty space).
// 1. Max out at 12xN.
// 2. Max out at 01xN (if the lengthier element is too big).
function getMaxElementsPerRow(arr, avgElementLength, maxElementLength) {
  if (maxElementLength > 30) return 1;
  return Math.min(
    Math.max(
      Math.floor((Math.sqrt(arr.length) * avgElementLength) / maxElementLength),
      1
    ),
    12
  );
}

function prettifyArray(arr, depth = 0, hasOnlyNumbers) {
  // Remove the color characters so we can calculate the AVG and MAX correctly.
  const uncolored = arr.map(
    (elem) => elem.replace(/\u001b\[[0-9;]*m/g, '').length
  );

  const maxElementLength = Math.max(...uncolored);
  const avgElementLength = uncolored.reduce((a, b) => a + b) / uncolored.length;

  // Calculate the grid size.
  const maxElementsPerRow = getMaxElementsPerRow(
    arr,
    avgElementLength,
    maxElementLength
  );

  // Tries to align the columns.
  const alignColumn = (elem, i) => {
    const length = elem.replace(/\u001b\[[0-9;]*m/g, '').length;
    const shift = maxElementsPerRow === 1 ? 0 : maxElementLength - length;
    if (hasOnlyNumbers) {
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
      acc.atRow = 1;
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

function isArrayBuffer(value) {
  return value instanceof ArrayBuffer;
}

function stringifyArrayBuffer(value) {
  return `ArrayBuffer { byteLength: ${stringify(value.byteLength)} }`;
}

function isPromise(value) {
  return value instanceof Promise;
}

function stringifyPromise(value) {
  // We have to use a Rust binding to inspect the contents of a promise
  // object because JS doesn't expose that kind of functionality.
  const binding = process.binding('promise');
  const { state, value: promiseValue } = binding.peek(value);

  if (state === 'PENDING') {
    return `Promise { ${cyan('<pending>')} }`;
  }

  const output = stringify(promiseValue, undefined, 1);
  const end = `${output.length > 50 ? '\n' : ' '}}`;

  const prefix =
    state === 'FULFILLED'
      ? `${output.length > 50 ? '\n  ' : ''}`
      : `${output.length > 50 ? '\n  ' : ''}${red('<rejected>')} `;

  return 'Promise { ' + prefix + output + end;
}

const specialCharsRegex = new RegExp('[^A-Za-z0-9|_]+');

/**
 * Specifically stringifies JavaScript objects.
 *
 * @ignore
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

  if (isArrayBuffer(value)) {
    return stringifyArrayBuffer(value);
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

  if (isPromise(value)) {
    return stringifyPromise(value);
  }

  // It's an object type that console does not support.
  if (objectToString(value) !== '[object Object]') {
    const type = objectToString(value).replace('[object ', '').replace(']', '');
    return `${type} {}`;
  }

  // Looks like it's a regular object.
  const entries = [];
  for (const key of Object.keys(value)) {
    if (seen.has(value[key])) {
      entries.push(`${pre(depth * 2)}${key}: [Circular]`);
      continue;
    }
    // The following wraps in quotes object keys that contain special
    // characters like { "Foo-Bar": 123 }.
    const keyValue = specialCharsRegex.test(key) ? `"${key}"` : key;

    seen.add(value);
    entries.push(
      `${pre(depth * 2)}${keyValue}: ${stringify(value[key], seen, depth)}`
    );
  }

  // Output the class name if the object is a class instance.
  const className = value?.constructor?.name;
  const prefix = !className || className === 'Object' ? '' : className + ' ';

  // Apply multi-line formatting on long properties.
  if (entries.map((v) => v.trim()).join('').length > 50) {
    const start = `${prefix}{\n`;
    const end = `\n${pre((depth - 1) * 2)}}`;
    return `${start}${entries.join(',\n')}${end}`;
  }

  // Inline formatting.
  const entriesPretty = entries.map((v) => v.trim());
  const content = entries.length > 0 ? `{ ${entriesPretty.join(', ')} }` : `{}`;

  return `${prefix}${content}`;
}

/**
 * Shows the given message and waits for the user's input.
 *
 * @param {String} [message] - A string of text to display to the user.
 * @param {String} [defaultValue] - A string containing the default value displayed in the text input field.
 */
export function prompt(message = 'Prompt', defaultValue = null) {
  // Write prompt message to stdout.
  process.stdout.write(
    `${message} ${defaultValue ? `[${defaultValue}] ` : ''}`
  );
  // Read and return user's input.
  return process.stdin.read() || defaultValue;
}

/**
 * Console is a subset implementation of MDN's Console API.
 */
export class Console {
  // Holds timers initialized by console.
  // https://developer.mozilla.org/en-US/docs/Web/API/Console/time
  #timers = new Map();

  /**
   * Outputs data to the `stdout` stream.
   *
   * @param  {...*} args - Prints to stdout with newline.
   */
  log(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stdout.write(`${output}\n`);
  }

  /**
   * An alias to `console.log()`.
   *
   * @param  {...*} args - Prints to stdout with newline.
   */
  info(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stdout.write(`${output}\n`);
  }

  /**
   * An alias to `console.log()`.
   *
   * @param  {...*} args - Prints to stdout with newline.
   */
  debug(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stdout.write(`${output}\n`);
  }

  /**
   * Same as `console.log` but prepends the output with "WARNING".
   *
   * @param  {...*} args - Prints to stdout with newline.
   */
  warn(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stderr.write(`WARNING: ${output}\n`);
  }

  /**
   * Same as `console.log` but prepends the output with "WARNING".
   *
   * @param  {...*} args - Prints to stdout with newline.
   */
  error(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stderr.write(`WARNING: ${output}\n`);
  }

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
   * @param {String} [label] - A string representing the name to give the new timer.
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
   * @param {String} [label] - The name of the timer to log to the console.
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
   * @param {String} [label] - A string representing the name of the timer to stop.
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

// This wrapper forwards console messages to V8's internal console implementation,
// triggering the `Runtime.consoleAPICalled` event. This ensures that the
// attached debugger (if exists) is notified about the console call.
//
// https://github.com/v8/v8/blob/master/src/inspector/v8-console.cc
// https://chromedevtools.github.io/devtools-protocol/tot/Runtime/#event-consoleAPICalled
//
export function wrapConsole(console, consoleFromV8) {
  // Get the property names of the console prototype.
  const prototype = Object.getPrototypeOf(console);
  const propertyNames = Object.getOwnPropertyNames(prototype);

  for (const key of Object.keys(consoleFromV8)) {
    // If global console has the same method as inspector console,
    // then wrap these two methods into one.
    if (propertyNames.includes(key)) {
      console[key] = callConsole.bind(
        console,
        consoleFromV8[key],
        console[key]
      );
    } else {
      // Add additional console APIs from the inspector.
      console[key] = consoleFromV8[key];
    }
  }
}

export default { Console, prompt, wrapConsole };
