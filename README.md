# Dune

Dune is an open-source, cross-platform, shell around the **V8** engine, written in **Rust** and capable of running JavaScript (dah) and TypeScript code out of the box.

Developed completely for fun and experimentation.

![GitHub](https://img.shields.io/github/license/aalykiot/dune?style=flat-square)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/aalykiot/dune/ci?style=flat-square)

## Installation

**Mac, Linux:**

```sh
curl -fsSL https://raw.githubusercontent.com/aalykiot/dune/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://raw.githubusercontent.com/aalykiot/dune/main/install.ps1 | iex
```

> Otherwise you have to manually download and unzip the <a href="https://github.com/aalykiot/dune/releases/latest/download/dune-x86_64-pc-windows-msvc.zip">release</a> build.

**From Source:**

Clone the repo and build it using <a href="https://rustup.rs/">Cargo</a>.

```bash
git clone https://github.com/aalykiot/dune.git && cd ./dune && cargo build --release
```

> Make sure to create a `.dune` directory under your user.

## Getting Started

A simple example.

```js
import shortid from 'https://cdn.skypack.dev/shortid';

console.log(shortid()); //=> "lXN1aGba2"
```

Another example using the net module.

```js
import net from 'net';

const server = net.createServer(async (socket) => {
  console.log('Got new connection!');
  await socket.write('Hello! 👋\n');
  await socket.destroy();
});

server.listen(3000, '127.0.0.1', () => {
  console.log('Server is listening on port 3000...');
});
```

For more examples look at the <a href="./examples">examples</a> directory.

## Target API

### Globals

- [x] `global`: reference to the global object.
- [x] `globalThis`: same as `global`.
- [x] `console`: a subset of the WHATWG console.
- [x] `prompt`: shows the given message and waits for the user's input.
- [x] `TextEncoder` / `TextDecoder`: WHATWG encoding API.
- [x] `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`: DOM style timers.
- [x] `setImmediate` / `clearImmediate`: node.js like immediate timers.
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
- [x] `uptime()`: a number describing the amount of time (in seconds) the process is running.
- [x] `version`: the dune version.
- [x] `versions`: an object listing the version strings of dune and its dependencies.
- [x] `binding(module)`: exposes modules with bindings to Rust.
- [x] `kill(pid, [signal])`: sends the signal to the process identified by pid.
- [x] `stdout`: points to system's `stdout` stream.
- [x] `stdin`: points to system's `stdin` stream.
- [x] `stderr`: points to system's `stderr` stream.

### File System

> This module should also include a `Sync` method for every async operation available.

- [x] `copyFile(src, dest)`: copies `src` to `dest`.
- [ ] `createReadStream(path, [options])`: creates a readable IO stream. 🚧
- [ ] `createWriteStream(path, [options])`: creates a writable IO stream. 🚧
- [x] `open(path, [mode])`: asynchronous file open.
- [x] `mkdir(path, [options])`: creates a directory.
- [x] `readFile(path, [options])`: reads the entire contents of a file.
- [x] `rmdir(path, [options])`: deletes a directory (must be empty).
- [x] `readdir(path)`: reads the contents of a directory.
- [x] `rm(path, [options])`: removes files and directories.
- [x] `stat(path)`: retrieves statistics for the file.
- [x] `writeFile(String|Uint8Array , data, [options])`: writes data to the file, replacing the file if it already exists.

### File

- [x] `fd`: the numeric file descriptor.
- [x] `close()`: closes the file.
- [ ] `createReadStream()`: creates a readable IO stream. 🚧
- [ ] `createWriteStream()`: creates a writable IO stream. 🚧
- [x] `read([size, [offset]])`: reads data from the file.
- [x] `stat()`: retrieves statistics for the file.
- [x] `write(String|Uint8Array, [offset])`: writes data to the file.

### Net

- [x] `createServer([connectionListener])`: Creates a new TCP server.
- [x] `createConnection(options, [connectionListener])`: Creates unix socket connection to a remote host.

### Net.Server

> Net.Server is a class extending `EventEmitter`.

- [x] `listen(port, [host], [callback])`: Begin accepting connections on the specified port and host.
- [x] `close([callback])`: Stops the server from accepting new connections and keeps existing connections.
- [x] `address()`: Returns the bound address.
- [x] `getConnections()`: Get the number of concurrent connections on the server.
- [x] `Event: 'listening'`: Emitted when the server has been bound after calling `server.listen`.
- [x] `Event: 'connection'`: Emitted when a new connection is made.
- [x] `Event: 'close'`: Emitted when the server closes.
- [x] `Event: 'error'`: Emitted when an error occurs.

### Net.Socket

> Net.Socket is a class extending `EventEmitter`.

- [x] `connect(options, [connectionListener])`: Opens the connection for a given socket.
- [x] `setEncoding(encoding)`: Set the encoding for the socket.
- [x] `write(data, [callback])`: Sends data on the socket.
- [x] `end([data])`: Half-closes the socket. i.e., it sends a FIN packet.
- [x] `destroy()`: Closes and discards the TCP socket stream.
- [x] `address()`: Returns the bound address.
- [x] `remoteAddress`: The string representation of the remote IP address.
- [x] `remotePort`: The numeric representation of the remote port.
- [x] `bytesRead`: The amount of received bytes.
- [x] `bytesWritten`: The amount of bytes sent.
- [x] `Event: 'connect'`: Emitted when a socket connection is successfully established.
- [x] `Event: 'data'`: Emitted when data is received.
- [x] `Event: 'end'`: Emitted when the other end of the socket sends a FIN packet.
- [x] `Event: 'error'`: Emitted when an error occurs.
- [x] `Event: 'close'`: Emitted once the socket is fully closed.

### Performance Measurement

- [x] `timeOrigin`: specifies the millisecond timestamp at which the current process began.
- [x] `now()`: returns the millisecond timestamp, where 0 represents the start of the current process.

### Assert

> The assertion API is copied from: https://assert-js.norbert.tech/

- [x] `true(value)`: asserts that value is equal to true.
- [x] `false(value)`: asserts that value is equal to false.
- [x] `instanceOf(value, class)`: asserts that value is an instance of specific class.
- [x] `integer(value)`: asserts that value is valid integer.
- [x] `number(value)`: asserts that value is valid number (integer, float).
- [x] `oddNumber(value)`: asserts that value is odd number.
- [x] `evenNumber(value)`: asserts that value is event number.
- [x] `greaterThan(value, limit)`: asserts that number is greater than.
- [x] `greaterThanOrEqual(value, limit)`: asserts that number is greater than or equal.
- [x] `lessThan(value, limit)`: asserts that number is less than.
- [x] `lessThanOrEqual(value, limit)`: asserts that number is less than or equal.
- [x] `string(value)`: asserts that value is valid string.
- [x] `boolean(value)`: asserts that value is valid boolean.
- [x] `equal(actual, expected)`: asserts that value is equal to expected value.
- [x] `objectEqual(actual, expected)`: asserts that value is equal to expected value.
- [x] `object(value)`: asserts that value is valid object.
- [x] `hasFunction(name, object)`: asserts that object has function.
- [x] `hasProperty(name, object)`: asserts that object has property.
- [x] `isFunction(fn)`: asserts that value is valid function.
- [x] `array(value)`: asserts that value is valid array.
- [x] `count(expected, arrayValue)`: asserts that array have specific number of elements.
- [x] `notEmpty(arrayValue)`: asserts that array is not empty.
- [x] `throws(fn, error)`: asserts that function throws expected exception.

## License

This project is licensed under the <a href="./LICENSE.md">MIT</a> license.
