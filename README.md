# Dune

Dune is a hobby JavaScript and TypeScript runtime written in **Rust**, based on **V8**, and developed completely for fun and experimentation.

<p>
<img src="https://img.shields.io/badge/version-v0.1.0-lightgray?style=for-the-badge" />
<img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" />
</p>

## Branches

- main -> this is what is currently working.
- develop -> new features are developed here (things might be broken).

## Installation

Clone the repo from GitHub:

```bash
$ git clone https://github.com/aalykiot/dune.git
```

Run dune using cargo:

```bash
$ cd dune/ && cargo run -- <FILE>
```

## API

### Globals

- `global`: reference to the global object.
- `globalThis`: same as `global`.
- `console`: a subset of the WHATWG console.
- `TextEncoder` / `TextDecoder`: WHATWG encoding API.
- `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`: DOM style timers.
- `setImmediate` / `clearImmediate`: node.js like immediate timers.
- `process`: an object that provides info about the current dune process.

### Process

- `argv`: an array containing the command-line arguments passed when the dune process was launched.
- `cwd()`: current working directory.
- `env`: an object containing the user environment.
- `exit([code])`: exits the program with the given code.
- `getActiveResourcesInfo()`: an array of strings containing the types of the active resources that are currently keeping the event loop alive.
- `memoryUsage()`: an object describing the memory usage.
- `pid`: PID of the process.
- `platform`: a string identifying the operating system platform.
- `uptime()`: a number describing the amount of time (in seconds) the process is running.
- `version`: the dune version.
- `versions`: an object listing the version strings of dune and its dependencies.
- `binding(module)`: exposes modules with bindings to Rust.
- `kill(pid, [signal])`: sends the signal to the process identified by pid.
- `stdout`: points to system's `stdout` stream.
- `stdin`: points to system's `stdin` stream.
- `stderr`: points to system's `stderr` stream.

### File System

> This module should also include a `Sync` method for every async operation available.

- `copyFile(src, dest)`: copies `src` to `dest`.
- `createReadStream(path, [options])`: creates a readable IO stream.
- `createWriteStream(path, [options])`: creates a writable IO stream.
- `open(path, [flags, [mode]])`: asynchronous file open.
- `mkdir(path)`: creates a directory.
- `readFile(path)`: reads the entire contents of a file.
- `rmdir(path)`: deletes a directory (must be empty).
- `rm(path, [options])`: removes files and directories.
- `stat(path)`: retrieves statistics for the file.
- `writeFile(path, String|Uint8Array, [options])`: writes data to the file, replacing the file if it already exists.

### File

- `fd`: the numeric file descriptor.
- `close()`: closes the file.
- `createReadStream()`: creates a readable IO stream.
- `createWriteStream()`: creates a writable IO stream.
- `read([size, [offset]])`: reads data from the file.
- `stat()`: retrieves statistics for the file.
- `write(String|Uint8Array, [offset])`: writes data to the file.

### Performance Measurement

- `timeOrigin`: specifies the millisecond timestamp at which the current process began.
- `now()`: returns the millisecond timestamp, where 0 represents the start of the current process.

### Assert

> The assertion API is copied from: https://assert-js.norbert.tech/

- `true(value)`: asserts that value is equal to true.
- `false(value)`: asserts that value is equal to false.
- `instanceOf(value, class)`: asserts that value is an instance of specific class.
- `integer(value)`: asserts that value is valid integer.
- `number(value)`: asserts that value is valid number (integer, float).
- `oddNumber(value)`: asserts that value is odd number.
- `evenNumber(value)`: asserts that value is event number.
- `greaterThan(value, limit)`: asserts that number is greater than.
- `greaterThanOrEqual(value, limit)`: asserts that number is greater than or equal.
- `lessThan(value, limit)`: asserts that number is less than.
- `lessThanOrEqual(value, limit)`: asserts that number is less than or equal.
- `string(value)`: asserts that value is valid string.
- `boolean(value)`: asserts that value is valid boolean.
- `equal(actual, expected)`: asserts that value is equal to expected value.
- `objectEqual(actual, expected)`: asserts that value is equal to expected value.
- `object(value)`: asserts that value is valid object.
- `hasFunction(name, object)`: asserts that object has function.
- `hasProperty(name, object)`: asserts that object has property.
- `isFunction(fn)`: asserts that value is valid function.
- `array(value)`: asserts that value is valid array.
- `count(expected, arrayValue)`: asserts that array have specific number of elements.
- `notEmpty(arrayValue)`: asserts that array is not empty.
- `throws(fn, error)`: asserts that function throws expected exception.

## License

This project is licensed under the <a href="./LICENSE.md">MIT</a> license.
