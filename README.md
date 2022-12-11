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

JSX/TSX files are also supported for server side rendering.

```jsx
import { h, Component } from 'https://esm.sh/preact@10.11.3';
import { render } from 'https://esm.sh/preact-render-to-string@5.2.6';

/** @jsx h */

// Classical components work.
class Fox extends Component {
  render({ name }) {
    return <span class="fox">{name}</span>;
  }
}

// ... and so do pure functional components:
const Box = ({ type, children }) => (
  <div class={`box box-${type}`}>{children}</div>
);

let html = render(
  <Box type="open">
    <Fox name="Finn" />
  </Box>
);

console.log(html);
```

For more examples look at the <a href="./examples">examples</a> directory.

## Available APIs

### Globals

- [x] `global`: Reference to the global object.
- [x] `globalThis`: Same as `global`.
- [x] `console`: A subset of the WHATWG console.
- [x] `prompt`: Shows the given message and waits for the user's input.
- [x] `TextEncoder` / `TextDecoder`: WHATWG encoding API.
- [x] `setTimeout` / `setInterval` / `clearTimeout` / `clearInterval`: DOM style timers.
- [x] `setImmediate` / `clearImmediate`: Node.js like immediate timers.
- [x] `process`: An object that provides info about the current dune process.

### Module Metadata

- [x] `import.meta.url`: A string representation of the fully qualified module URL.
- [x] `import.meta.main`: A flag that indicates if the current module is the main module.
- [x] `import.meta.resolve(specifier)`: A function that returns resolved specifier.

### Process

- [x] `argv`: An array containing the command-line arguments passed when the dune process was launched.
- [x] `cwd()`: Current working directory.
- [x] `env`: An object containing the user environment.
- [x] `exit(code?)`: Exits the program with the given code.
- [ ] `getActiveResourcesInfo()`: An array of strings containing the types of the active resources that are currently keeping the event loop alive.
- [x] `memoryUsage()`: An object describing the memory usage.
- [x] `nextTick(cb, ...args?)`: Adds callback to the "next tick queue".
- [x] `pid`: PID of the process.
- [x] `platform`: A string identifying the operating system platform.
- [x] `uptime()`: A number describing the amount of time (in seconds) the process is running.
- [x] `version`: The dune version.
- [x] `versions`: An object listing the version strings of dune and its dependencies.
- [x] `binding(module)`: Exposes modules with bindings to Rust.
- [x] `kill(pid, signal?)`: Sends the signal to the process identified by pid.
- [x] `stdout`: Points to system's `stdout` stream.
- [x] `stdin`: Points to system's `stdin` stream.
- [x] `stderr`: Points to system's `stderr` stream.

### File System

> This module should also include a `Sync` method for every async operation available.

- [x] `copyFile(src, dest)`: Copies `src` to `dest`.
- [ ] `createReadStream(path, options?)`: Creates a readable IO stream. 🚧
- [ ] `createWriteStream(path, options?)`: Creates a writable IO stream. 🚧
- [x] `open(path, mode?)`: Asynchronous file open.
- [x] `mkdir(path, options?)`: Creates a directory.
- [x] `readFile(path, options?)`: Reads the entire contents of a file.
- [x] `rmdir(path, options?)`: Deletes a directory (must be empty).
- [x] `readdir(path)`: Reads the contents of a directory.
- [x] `rm(path, options?)`: Removes files and directories.
- [x] `stat(path)`: Retrieves statistics for the file.
- [x] `writeFile(String|Uint8Array , data, options?)`: Writes data to the file, replacing the file if it already exists.

### File

- [x] `fd`: The numeric file descriptor.
- [x] `close()`: Closes the file.
- [ ] `createReadStream()`: Creates a readable IO stream. 🚧
- [ ] `createWriteStream()`: Creates a writable IO stream. 🚧
- [x] `read(size?, offset?)`: Reads data from the file.
- [x] `stat()`: Retrieves statistics for the file.
- [x] `write(String|Uint8Array, offset?)`: Writes data to the file.

### Net

- [x] `createServer(connectionListener?)`: Creates a new TCP server.
- [x] `createConnection(options, connectionListener?)`: Creates unix socket connection to a remote host.

### Net.Server

> Net.Server is a class extending `EventEmitter`.

- [x] `listen(port, host?, callback?)`: Begin accepting connections on the specified port and host.
- [x] `close()`: Stops the server from accepting new connections and keeps existing connections.
- [x] `address()`: Returns the bound address.
- [x] `getConnections()`: Get the number of concurrent connections on the server.
- [x] `Event: 'listening'`: Emitted when the server has been bound after calling `server.listen`.
- [x] `Event: 'connection'`: Emitted when a new connection is made.
- [x] `Event: 'close'`: Emitted when the server closes.
- [x] `Event: 'error'`: Emitted when an error occurs.

### Net.Socket

> Net.Socket is a class extending `EventEmitter`.

- [x] `connect(options, connectionListener?)`: Opens the connection for a given socket.
- [x] `setEncoding(encoding)`: Set the encoding for the socket.
- [x] `write(data)`: Sends data on the socket.
- [x] `end(data?)`: Half-closes the socket. i.e., it sends a FIN packet.
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

- [x] `timeOrigin`: Specifies the millisecond timestamp at which the current process began.
- [x] `now()`: Returns the millisecond timestamp, where 0 represents the start of the current process.

### Assert

> The assertion API is copied from: https://github.com/browserify/commonjs-assert

- [x] `fail(message?)`: Throws an AssertionError with the provided error message or a default error message.
- [x] `AssertionError`: Indicates the failure of an assertion.
- [x] `ok(value, message?)`: Tests if value is truthy.
- [x] `equal(actual, expected, message?)`: An alias of `strictEqual()`.
- [x] `notEqual(actual, expected, message?)`: An alias of `notStrictEqual()`.
- [x] `deepEqual(actual, expected, message?)`: An alias of `deepStrictEqual()`.
- [x] `notDeepEqual(actual, expected, message?)`: An alias of `notDeepStrictEqual()`.
- [x] `deepStrictEqual(actual, expected, message?)`: Tests for deep equality between the params.
- [x] `notDeepStrictEqual(actual, expected, message?)`: Tests for deep strict inequality.
- [x] `strictEqual(actual, expected, message?)`: Tests strict equality between the parameters.
- [x] `notStrictEqual(actual, expected, message?)`: Tests strict inequality between the parameters.
- [x] `throws(fn, error?, message?)`: Expects the function fn to throw an error.
- [x] `rejects(asyncFn, error?, message?)`: It will check that the promise is rejected.
- [x] `doesNotThrow(fn, error?, message?)`: Asserts that the function fn does not throw an error.
- [x] `doesNotReject(asyncFn, error?, message?)`: It will check that the promise is not rejected.
- [x] `ifError(value)`: Throws value if value is not undefined or null.

## Contributing

Contributions are always welcome!

See [CONTRIBUTING.md](https://github.com/aalykiot/dune/blob/main/CONTRIBUTING.md) for ways to get started.
