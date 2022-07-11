// File System API
//
// The File System API enables interacting with the file system in a way modeled
// on standard POSIX functions.
//
// https://nodejs.org/api/fs.html

const binding = process.binding('fs');

const BUFFER_SIZE = 40 * 1024; // 40KB bytes buffer when reading.

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
  while (true) {
    const bytes = binding.readSync(path, BUFFER_SIZE, data.length + 1);
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

export default { readFileSync, writeFileSync };
