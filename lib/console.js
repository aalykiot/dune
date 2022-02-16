function pre(amount) {
  return ' '.repeat(amount);
}

function stringify(value, seen, depth = 0) {
  switch (typeof value) {
    case 'string':
      return depth > 0 ? `"${value}"` : value;
    case 'number':
    case 'undefined':
    case 'boolean':
    case 'symbol':
      return String(value);
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

function stringifyArray(value, seen, depth) {
  const entries = [];
  for (const elem of value) {
    entries.push(stringify(elem, seen, depth));
  }
  // Check if the raw array is too long so we can apply multi-line formatting.
  if (entries.join('').length > 50) {
    const start = '[\n';
    const end = `\n${pre((depth - 1) * 2)}]`;
    const entriesPretty = entries.map((v) => `${pre(depth * 2)}${v}`);
    return `${start}${entriesPretty.join(',\n')}${end}`;
  }
  // Ff not do an inline formatting.
  return entries.length > 0 ? `[ ${entries.join(', ')} ]` : `[]`;
}

function stringifyObject(value, seen = new WeakSet(), depth) {
  // We need special handling if it's an array.
  if (Array.isArray(value)) {
    return stringifyArray(value, seen, depth);
  }
  // It turns out, it's an object,
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
  // Check if the raw array is too long so we can apply multi-line formatting.
  if (entries.map((v) => v.trim()).join('').length > 50) {
    const start = '{\n';
    const end = `\n${pre((depth - 1) * 2)}}`;
    return `${start}${entries.join(',\n')}${end}`;
  }
  // If not do an inline formatting.
  const entriesPretty = entries.map((v) => v.trim());
  return entries.length > 0 ? `{ ${entriesPretty.join(', ')} }` : `{}`;
}

class Console {
  log(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stdout.write(`${output}\n`);
  }

  info = this.log;
  debug = this.log;

  warn(...args) {
    const output = args.map((arg) => stringify(arg)).join(' ');
    process.stderr.write(`WARNING: ${output}\n`);
  }

  error = this.warn;
}

export { Console };
