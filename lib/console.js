// Console API
//
// The Console API provides functionality to allow developers to perform debugging tasks,
// such as logging messages or the values of variables at set points in your code.
//
// https://developer.mozilla.org/en-US/docs/Web/API/console

/**
 * Returns a string with as many spaces as the parameter specified.
 *
 * @param {string} amount - the length of the returned string.
 * @returns {string}
 */
function pre(amount) {
  return ' '.repeat(amount);
}

/**
 * Transforms a JavaScript object/primitive into a string.
 *
 * @param {*} value - the value we want to stringify.
 * @param {WeakSet} seen - used to identify circular references in objects.
 * @param {number} depth - how deep we are in an object traversal.
 * @returns {string}
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
 * Specifically stringifies JavaScript arrays.
 *
 * @param {*} value - the value we want to stringify.
 * @param {WeakSet} seen - used to identify circular references in objects.
 * @param {number} depth - the depth we're in on object traversal.
 * @returns {string}
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
 * Specifically stringifies JavaScript objects.
 *
 * @param {*} value - the value we want to stringify.
 * @param {WeakSet} seen - used to identify circular references in objects.
 * @param {number} depth - how deep we are in an object traversal.
 * @returns {string}
 */
function stringifyObject(value, seen = new WeakSet(), depth) {
  // Arrays need special handling.
  if (Array.isArray(value)) {
    return stringifyArray(value, seen, depth);
  }

  // It's an object.
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
