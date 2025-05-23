# Dune

Dune is an open-source, cross-platform, shell around the **V8** engine, written in **Rust** and capable of running JavaScript (dah) and TypeScript code out of the box.

Developed completely for fun and experimentation.

![GitHub](https://img.shields.io/github/license/aalykiot/dune?style=flat-square)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/aalykiot/dune/dune-ci.yml?branch=main&style=flat-square)

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
- [x] `structuredClone`: Creates a deep clone of a given value.
- [x] `AbortController` / `AbortSignal`: Allows you to communicate with a request and abort it.
- [x] `fetch`: A wrapper around `http.request` (not fully compatible with WHATWG fetch).
- [x] `queueMicrotask`: Queues a microtask to invoke a callback.

### Module Metadata

- [x] `import.meta.url`: A string representation of the fully qualified module URL.
- [x] `import.meta.main`: A flag that indicates if the current module is the main module.
- [x] `import.meta.resolve(specifier)`: A function that returns resolved specifier.

### Process

- [x] `argv`: An array containing the command-line arguments passed when the dune process was launched.
- [x] `cwd()`: Current working directory.
- [x] `env`: An object containing the user environment.
- [x] `exit(code?)`: Exits the program with the given code.
- [ ] `getActiveResourcesInfo()`: An array of strings containing the types of the active resources that are currently keeping the event loop alive. 🚧
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

##### Events

- [x] `uncaughtException`: Emitted when an uncaught exception bubbles up to Dune.
- [x] `unhandledRejection`: Emitted when a Promise is rejected with no handler.

> Signal events will be emitted when the Dune process receives a signal. Please refer to [signal(7)](https://man7.org/linux/man-pages/man7/signal.7.html) for a listing of standard POSIX signal names.

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

- [x] `createServer(connectionHandler?)`: Creates a new TCP server.
- [x] `createConnection(options)`: Creates unix socket connection to a remote host.
- [x] `connect(options)`: An alias of `createConnection()`.
- [x] `TimeoutError`: Custom error signalling a socket (read) timeout.

#### `net.Server`

> net.Server is a class extending `EventEmitter` and implements `@@asyncIterator`.

- [x] `listen(port, host?)`: Begin accepting connections on the specified port and host.
- [x] `accept()`: Waits for a TCP client to connect and accepts the connection.
- [x] `address()`: Returns the bound address.
- [x] `close()`: Stops the server from accepting new connections and keeps existing connections.

##### Events

- [x] `listening`: Emitted when the server has been bound after calling `server.listen`.
- [x] `connection`: Emitted when a new connection is made.
- [x] `close`: Emitted when the server stops accepting new connections.
- [x] `error`: Emitted when an error occurs.

#### `net.Socket`

> net.Socket is a class extending `EventEmitter` and implements `@@asyncIterator`.

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

##### Events

- [x] `connect`: Emitted when a socket connection is successfully established.
- [x] `data`: Emitted when data is received.
- [x] `end`: Emitted when the other end of the socket sends a FIN packet.
- [x] `error`: Emitted when an error occurs.
- [x] `close`: Emitted once the socket is fully closed.
- [x] `timeout`: Emitted if the socket times out from (read) inactivity.

### HTTP

> The HTTP package is inspired by Node.js' [undici](https://undici.nodejs.org/) package.

- [x] `METHODS`: A list of the HTTP methods that are supported by the parser.
- [x] `STATUS_CODES`: A collection of all the standard HTTP response status codes.
- [x] `request(url, options?)`: Performs an HTTP request.
- [x] `createServer(requestHandler?)`: Creates a new HTTP server.

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
- `signal`: (AbortSignal) - Default: `null` - Allows you to communicate with the request and abort it.

Body Mixins

> The body mixins are the most common way to format the response body.

- [x] `text()`: Produces a UTF-8 string representation of the body.
- [x] `json()`: Formats the body using JSON parsing.
</details>

#### `http.Server`

> http.Server is a class extending `EventEmitter` and implements `@@asyncIterator`.

- [x] `listen(port, host?)`: Starts the HTTP server listening for connections.
- [x] `close()`: Stops the server from accepting new connections.
- [x] `accept()`: Waits for a client to connect and accepts the HTTP request.

##### Events

- [x] `request`: Emitted each time there is a request.
- [x] `close`: Emitted when the server closes.
- [x] `clientError`: Emitted when a client connection emits an 'error' event.

#### `http.ServerRequest`

> http.ServerRequest implements `@@asyncIterator`.

- [x] `headers`: The request headers object.
- [x] `httpVersion`: The HTTP version sent by the client.
- [x] `method`: The request method as a string.
- [x] `url`: Request URL string.
- [x] `text()`: Produces a UTF-8 string representation of the body.
- [x] `json()`: Formats the body using JSON parsing.

#### `http.ServerResponse`

> http.ServerResponse implements `stream.Writable`.

- [x] `write(data)`: This sends a chunk of the response body.
- [x] `end(data?)`: Signals that all of the response headers and body have been sent.
- [x] `writeHead(code, message?, headers?)`: Sends the response headers to the client.
- [x] `setHeader(name, value)`: Sets a single header value for implicit headers.
- [x] `getHeader(name)`: Reads out a header that's already been queued but not sent to the client.
- [x] `getHeaderNames()`: Returns an array containing the unique names of the current outgoing headers.
- [x] `getHeaders()`: Returns a copy of the current outgoing headers.
- [x] `hasHeader(name)`: Returns true if the header identified is currently set.
- [x] `removeHeader(name)`: Removes a header that's queued for implicit sending.
- [x] `headersSent`: Boolean (read-only). True if headers were sent, false otherwise.
- [x] `socket`: Reference to the underlying socket.

##### Events

- [x] `finish`: Emitted when the (full) response has been sent.

### Stream

> Streams are very different from Node.js and are based on [async-generators](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/AsyncGenerator).

- [x] `pipe(source, ...targets)`: An alias of `pipeline()`.
- [x] `pipeline(source, ...targets)`: Pipes between streams while forwarding errors.
- [x] `compose(...targets)`: Combines two or more streams into a Duplex stream.

### Performance Measurement

- [x] `timeOrigin`: Specifies the millisecond timestamp at which the current process began.
- [x] `now()`: Returns the millisecond timestamp, where 0 represents the start of the current process.

### SQLite

> All APIs exposed by this module execute synchronously.

#### `sqlite.Database`

- [x] `open()`: Opens the database specified in the constructor.
- [x] `loadExtension(path)`: Loads a shared library into the database connection.
- [x] `enableLoadExtension(flag)`: Enables or disables the loadExtension SQL function.
- [x] `exec(sql)`: SQL statements to be executed without returning any results.
- [x] `prepare(sql)`: Compiles a SQL statement into a [prepared statement](https://sqlite.org/c3ref/stmt.html).
- [x] `close()`: Closes the database connection.
- [x] `isOpen`: Whether the database is currently open or not.

> To use an in-memory database, the path should be the special name `:memory:`.

#### `sqlite.Statement`

- [x] `run(...params?)`: Executes a prepared statement and returns the resulting changes.
- [x] `columns()`: Used to retrieve information about the columns.
- [x] `all(...params?)`: Executes a prepared statement and returns all results.
- [x] `get(...params?)`: Returns the first result.
- [x] `setReadBigInts(flag)`: Enables or disables the use of `BigInt`s when reading `INTEGER` fields.
- [x] `sourceSQL`: The source SQL text of the prepared statement.
- [x] `expandedSQL`: The source SQL text of the prepared statement with parameter placeholders replaced.

### Test Runner

- [x] `test(description, [options], testFn)`: Registers a test with the default test runner.
- [x] `TestRunner`: (Class) A main executor to run JavaScript and TypeScript tests.

<details><summary>Details</summary>
<p></p>

Options

- `ignore`: (boolean) - Default: `false` - Ignore test based on a runtime check.

Custom Executors

> You can attach tests to custom runners and run them manually.

```js
import { TestRunner } from 'test';
import assert from 'assert';

const runner = new TestRunner();

runner.failFast = true;
runner.filter = null;

runner.test('a simple test', () => {
  assert.equal(1 + 2, 3);
});

await runner.run();
```

</details>

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

## Testing

Dune has a built-in test runner that you can use for testing JavaScript or TypeScript code.

```js
import test from 'test';
import assert from 'assert';

function doSomeMath(a, b) {
  return a + b;
}

test('checking multiple addition values', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});
```

You can run the above suite using the `dune test` subcommand:

```sh
$ dune test example_test.js

OK  checking multiple addition values

Test result: 1 ok; 0 failed; 0 ignored (0 ms)
```

For more testing examples look at the <a href="./examples/testing/">examples/testing</a> directory.

## Debugging Your Code

Dune embraces the [V8 Inspector Protocol](https://v8.dev/docs/inspector), a standard employed by Chrome, Edge, and Node.js. This enables the debugging of Dune programs through the utilization of Chrome DevTools or other clients that are compatible with this protocol.

To enable debugging capabilities, execute Dune with either the `--inspect` or `--inspect-brk` flags.

The `--inspect` flag permits attaching the debugger at any moment, whereas the `--inspect-brk` option will await the debugger to attach and will pause the execution on the next statement.

> When employing the `--inspect` flag, the code will commence execution promptly. If your program is brief, there may not be sufficient time to establish a debugger connection before the program concludes its execution. In such scenarios, consider using the `--inspect-brk` flag instead.

### Chrome DevTools

Let's attempt debugging a program using Chrome DevTools:

```sh
$ dune run examples/httpServer.js --inspect-brk
Debugger listening on ws://127.0.0.1:9229/1513ff37-2f3e-48a3-a4bf-9e3330dc4544
Visit chrome://inspect to connect to the debugger.
...
```

In a Chromium-based browser like Google Chrome or Microsoft Edge, navigate to `chrome://inspect` and select "Inspect" next to the target.

### VS Code

Currently, there is no extension available for Dune in VS Code. However, you can debug your application in VS Code, by utilizing the following launch configuration in `.vscode/launch.json`:

```json
{
  "type": "node",
  "request": "launch",
  "name": "Debug 'Dune' application",
  "cwd": "${workspaceFolder}",
  "program": "<SCRIPT>",
  "runtimeExecutable": "dune",
  "runtimeArgs": ["run", "--inspect-brk"],
  "attachSimplePort": 9229,
  "console": "integratedTerminal"
}
```

## Contributing

Contributions are always welcome!

See [CONTRIBUTING.md](https://github.com/aalykiot/dune/blob/main/CONTRIBUTING.md) for ways to get started.
