# Dune

Dune is an open-source, cross-platform, shell around the **V8** engine, written in **Rust** and capable of running JavaScript (dah) and TypeScript code out of the box.

Developed completely for fun and experimentation.

![GitHub](https://img.shields.io/github/license/aalykiot/dune?style=flat-square)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/aalykiot/dune/ci.yml?branch=main&style=flat-square)

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

await server.listen(3000, '127.0.0.1');

console.log('Server is listening on port 3000...');
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

> This module also includes a `Sync` method for every async operation available.

- [x] `copyFile(src, dest)`: Copies `src` to `dest`.
- [x] `createReadStream(path, options?)`: Returns a new readable IO stream.
- [x] `createWriteStream(path, options?)`: Returns a new writable IO stream.
- [x] `open(path, mode?)`: Asynchronous file open.
- [x] `mkdir(path, options?)`: Creates a directory.
- [x] `readFile(path, options?)`: Reads the entire contents of a file.
- [x] `rmdir(path, options?)`: Deletes a directory (must be empty).
- [x] `readdir(path)`: Reads the contents of a directory.
- [x] `rm(path, options?)`: Removes files and directories.
- [x] `rename(from, to)`: Renames the file from oldPath to newPath.
- [x] `stat(path)`: Retrieves statistics for the file.
- [x] `watch(path, options?)`: Returns an async iterator that watches for changes over a path.
- [x] `writeFile(path, data, options?)`: Writes data to the file, replacing the file if it already exists.

> Data (to be written) must be of type String|Uint8Array.

### File

- [x] `fd`: The numeric file descriptor.
- [x] `close()`: Closes the file.
- [x] `read(size?, offset?)`: Reads data from the file.
- [x] `stat()`: Retrieves statistics for the file.
- [x] `write(data, offset?)`: Writes data to the file.

### Net

- [x] `createServer(connectionListener?)`: Creates a new TCP server.
- [x] `createConnection(options)`: Creates unix socket connection to a remote host.
- [x] `TimeoutError`: Custom error signalling a socket (read) timeout.

### Net.Server

> Net.Server is a class extending `EventEmitter` and implements `@@asyncIterator`.

- [x] `listen(port, host?)`: Begin accepting connections on the specified port and host.
- [x] `accept()`: Waits for a TCP client to connect and accepts the connection.
- [x] `address()`: Returns the bound address.
- [x] `close()`: Stops the server from accepting new connections and keeps existing connections.
- [x] `Event: 'listening'`: Emitted when the server has been bound after calling `server.listen`.
- [x] `Event: 'connection'`: Emitted when a new connection is made.
- [x] `Event: 'close'`: Emitted when the server stops accepting new connections.
- [x] `Event: 'error'`: Emitted when an error occurs.

### Net.Socket

> Net.Socket is a class extending `EventEmitter` and implements `@@asyncIterator`.

- [x] `connect(options)`: Opens the connection for a given socket.
- [x] `setEncoding(encoding)`: Sets the encoding for the socket.
- [x] `setTimeout(timeout)`: Sets the socket's timeout threshold when reading.
- [x] `read()`: Reads data out of the socket.
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
- [x] `Event: 'timeout'`: Emitted if the socket times out from (read) inactivity.

### HTTP

> The HTTP package is inspired by Node.js' [undici](https://undici.nodejs.org/) package.

- [x] `request(url, options?)`: Performs an HTTP request.

<details><summary>Details</summary>
<p></p>

```js
const URL = 'http://localhost:3000/foo';
const { statusCode, headers, body } = await http.request(URL);
```

RequestOptions

- `method`: (string) - Default: `GET`
- `headers`: (object) - Default: `null`
- `body`: (string | Uint8Array | stream.Readable) - Default: `null`
- `timeout`: (number) - Default: `30000` (30 seconds) - Use `0` to disable it entirely.
- `throwOnError`: (boolean) - Default: `false` - Whether should throw an error upon receiving a 4xx or 5xx response.

Body Mixins

> The body mixins are the most common way to format the response body.

- [x] `text()`: Formats the body to a UTF-8 string.
- [x] `json()`: Formats the body to an actual JSON object.
</details>

### Stream

> Streams are very different from Node.js and are based on [async-generators](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator).

- [x] `pipe(source, ...targets)`: An alias of `pipeline()`.
- [x] `pipeline(source, ...targets)`: Pipes between streams while forwarding errors.
- [x] `compose(...targets)`: Combines two or more streams into a Duplex stream.

### Performance Measurement

- [x] `timeOrigin`: Specifies the millisecond timestamp at which the current process began.
- [x] `now()`: Returns the millisecond timestamp, where 0 represents the start of the current process.

### Assert

> The assertion API is copied from: https://assert-js.norbert.tech/

- [x] `true(value)`: Asserts that value is equal to true.
- [x] `false(value)`: Asserts that value is equal to false.
- [x] `instanceOf(value, class)`: Asserts that value is an instance of specific class.
- [x] `integer(value)`: Asserts that value is valid integer.
- [x] `number(value)`: Asserts that value is valid number (integer, float).
- [x] `oddNumber(value)`: Asserts that value is odd number.
- [x] `evenNumber(value)`: Asserts that value is event number.
- [x] `greaterThan(value, limit)`: Asserts that number is greater than.
- [x] `greaterThanOrEqual(value, limit)`: Asserts that number is greater than or equal.
- [x] `lessThan(value, limit)`: Asserts that number is less than.
- [x] `lessThanOrEqual(value, limit)`: Asserts that number is less than or equal.
- [x] `string(value)`: Asserts that value is valid string.
- [x] `boolean(value)`: Asserts that value is valid boolean.
- [x] `equal(actual, expected)`: Asserts that value is equal to expected value.
- [x] `objectEqual(actual, expected)`: Asserts that value is equal to expected value.
- [x] `object(value)`: Asserts that value is valid object.
- [x] `hasFunction(name, object)`: Asserts that object has function.
- [x] `hasProperty(name, object)`: Asserts that object has property.
- [x] `isFunction(fn)`: Asserts that value is valid function.
- [x] `array(value)`: Asserts that value is valid array.
- [x] `count(expected, arrayValue)`: Asserts that array have specific number of elements.
- [x] `notEmpty(arrayValue)`: Asserts that array is not empty.
- [x] `throws(fn, error)`: Asserts that function throws expected exception.

## Contributing

Contributions are always welcome!

See [CONTRIBUTING.md](https://github.com/aalykiot/dune/blob/main/CONTRIBUTING.md) for ways to get started.
