# Dune

![ci workflow](https://github.com/aalykiot/dune/actions/workflows/ci.yml/badge.svg)

A hobby javascript runtime written in **Rust**, based on **V8**, and developed completely for fun and experimentation.

## API

### Globals

- `global`: reference to the global object.
- `globalThis`: same as `global`.
- `console`: a subset of the WHATWG console.
- `TextEncoder` / `TextDecoder`: WHATWG encoding API.
- <s>`ReadableStream` / `WritableStream`: WHATWG streams API.</s>
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

## Supported platforms

- GNU/Linux
- MacOS
- Windows

## License

This project is licensed under the <a href="./LICENSE.md">MIT</a> license.
