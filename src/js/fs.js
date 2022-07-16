// File System APIs
//
// The File System APIs enable interacting with the file system in a way modeled
// on standard POSIX functions.
//
// https://nodejs.org/api/fs.html

const binding = process.binding('fs');

const BUFFER_SIZE = 40 * 1024; // 40KB bytes buffer when reading.

/**
 * Reads asynchronously the entire contents of a file.
 *
 * @param {String} path
 * @param {String} encoding
 * @returns {Promise<String|Uint8Array>}
 */

async function readFile(path, encoding) {
  // Read the entire contents of a file.
  const data = await __readFile(path);

  // Decode given an encoder.
  if (encoding) {
    return new TextDecoder(encoding).decode(data);
  }

  return data;
}

async function __readFile(path, data = new Uint8Array([])) {
  // Try read some bytes from the file.
  const offset = data.length === 0 ? 0 : data.length + 1;
  const bytes = await binding.read(path, BUFFER_SIZE, offset);
  const bytes_u8 = new Uint8Array(bytes);

  // Check EOF.
  if (bytes_u8.length === 0) {
    return data;
  }

  // Recursively read more bytes.
  return __readFile(path, new Uint8Array([...data, ...bytes_u8]));
}

/**
 * Reads synchronously the entire contents of a file.
 *
 * @param {string} path
 * @param {string} encoding
 * @returns {string|Uint8Array}
 */

function readFileSync(path, encoding) {
  // Buffer to fill the file bytes into.
  let data = new Uint8Array([]);

  // Read bytes until EOF.
  for (;;) {
    const offset = data.length === 0 ? 0 : data.length + 1;
    const bytes = binding.readSync(path, BUFFER_SIZE, offset);
    const bytes_u8 = new Uint8Array(bytes);

    // Check EOF.
    if (bytes_u8.length === 0) {
      break;
    }

    // Append bytes to data.
    data = new Uint8Array([...data, ...bytes_u8]);
  }

  // Decode given an encoder.
  if (encoding) {
    return new TextDecoder(encoding).decode(data);
  }

  return data;
}

/**
 * Writes asynchronously contents to a file.
 *
 * @param {*} path
 * @param {string|Uint8Array} data
 * @param {string} encoding
 */

async function writeFile(path, data, encoding = 'utf8') {
  // Check the data argument type.
  if (!(data instanceof Uint8Array) && typeof data !== 'string') {
    throw new TypeError(
      `The "data" argument must be of type string or Uint8Array.`
    );
  }
  // Write asynchronously buffer to file.
  return binding.write(
    path,
    data instanceof Uint8Array ? data : new TextEncoder(encoding).encode(data)
  );
}

/**
 * Writes synchronously contents to a file.
 *
 * @param {*} path
 * @param {string|Uint8Array} data
 * @param {string} encoding
 */

function writeFileSync(path, data, encoding = 'utf8') {
  // Check the data argument type.
  if (!(data instanceof Uint8Array) && typeof data !== 'string') {
    throw new TypeError(
      `The "data" argument must be of type string or Uint8Array.`
    );
  }
  // Write buffer to file.
  binding.writeSync(
    path,
    data instanceof Uint8Array ? data : new TextEncoder(encoding).encode(data)
  );
}

export default { readFile, readFileSync, writeFile, writeFileSync };
