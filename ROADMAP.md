# Roadmap

### Globals

- [x] `global`: reference to the global object.
- [x] `globalThis`: same as `global`.
- [x] `console`: a subset of the WHATWG console.
- [ ] `TextEncoder` / `TextDecoder`: WHATWG encoding API.
- [ ] <s>`ReadableStream` / `WritableStream`: WHATWG streams API.</s>
- [ ] `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`: WHATWG timers.
- [ ] `setImmediate` / `clearImmediate`: node.js like immediate timers.
- [x] `process`: an object that provides info about the current dune process.

### Process

- [x] `argv`: an array containing the command-line arguments passed when the dune process was launched.
- [x] `cwd()`: current working directory.
- [x] `env`: an object containing the user environment.
- [x] `exit([code])`: exits the program with the given code.
- [ ] `getActiveResourcesInfo()`: an array of strings containing the types of the active resources that are currently keeping the event loop alive.
- [x] `memoryUsage()`: an object describing the memory usage.
- [x] `pid`: PID of the process.
- [x] `platform`: a string identifying the operating system platform.
- [ ] `uptime()`: a number describing the amount of time (in seconds) the process is running.
- [x] `version`: the dune version.
- [x] `versions`: an object listing the version strings of dune and its dependencies.
- [x] `binding(module)`: exposes modules with bindings to Rust.
- [ ] `kill(pid, [signal])`: sends the signal to the process identified by pid.
- [x] `stdout`: points to system's `stdout` stream.
- [ ] `stdin`: points to system's `stdin` stream.
- [x] `stderr`: points to system's `stderr` stream.

### File System

> This module should also include a `Sync` method for every async operation available.

- [ ] `copyFile(src, dest)`: copies `src` to `dest`.
- [ ] `createReadStream(path, [options])`: creates a readable IO stream.
- [ ] `createWriteStream(path, [options])`: creates a writable IO stream.
- [ ] `open(path, [flags, [mode]])`: asynchronous file open.
- [ ] `mkdir(path)`: creates a directory.
- [ ] `readFile(path)`: reads the entire contents of a file.
- [ ] `rmdir(path)`: deletes a directory (must be empty).
- [ ] `rm(path, [options])`: removes files and directories.
- [ ] `stat(path)`: retrieves statistics for the file.
- [ ] `writeFile(path, String|Uint8Array, [options])`: writes data to the file, replacing the file if it already exists.

### File

- [ ] `fd`: the numeric file descriptor.
- [ ] `close()`: closes the file.
- [ ] `createReadStream()`: creates a readable IO stream.
- [ ] `createWriteStream()`: creates a writable IO stream.
- [ ] `read([size, [offset]])`: reads data from the file.
- [ ] `stat()`: retrieves statistics for the file.
- [ ] `write(String|Uint8Array, [offset])`: writes data to the file.
