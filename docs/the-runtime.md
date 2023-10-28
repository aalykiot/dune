# Dune - The Runtime

Hi! 👋 You've found the technical overview of Dune as a JavaScript runtime. This README aims to help you understand the inner workings of the runtime and the reason behind some architectural decisions.

### V8 Engine

Dune leverages the powerful capabilities of the [V8](https://v8.dev/) engine through the usage of the [rusty_v8](https://github.com/denoland/rusty_v8) crate, an impressive creation by the skilled Deno team. Serving as a convenient wrapper for V8's C++ APIs, this crate allows seamless communication with V8, ensuring alignment with the original API to a high degree.

<br />
<img src="./assets/the-runtime-01.svg" height="85px" />
<br /><br />

All rusty_v8 available APIs can be found [here](https://docs.rs/v8/latest/v8/).

The purpose of this README is not to comprehensively cover all V8 concepts. However, we will touch on some key ones to enhance understanding in this technical overview. Among these concepts are the [Isolate](#isolate) and JS [Handles](#handles).

#### `Isolate`

The `v8::Isolate` is like a little world inside the JavaScript engine. Imagine it as a container that holds all the JavaScript objects, known as the **"heap."** These objects can interact with each other, and the isolate manages this interaction.

In the world of the `v8::Isolate`, there are special tools to control how things work. For example, they can be used to check how much memory the engine is using, or to clean up unnecessary objects (a process known as garbage collection) to keep the engine running efficiently.

Each `v8::Isolate` operates independently. It means that if something needs to be cleaned up, it only affects that specific world, not the entire engine. Think of it like having separate rooms in a house – cleaning up one room doesn't impact the others. This isolation is important because it keeps different parts of a program from accidentally affecting each other, ensuring that everything works as intended.

#### `Handles`

Every object returned from V8 must be monitored by the garbage collector to confirm its active status. Directly pointing to an object is unsafe due to potential object movement during garbage collection. Consequently, all objects are stored in handles, recognized by the garbage collector and updated whenever an object relocates.

<br/>
<img src="./assets/the-runtime-02.svg" height="280px" />
<br/><br/><br/>

There are two types of handles: **local** and **persistent** handles.

A `v8::Local` handle serves as a momentary pointer to a JavaScript object, typically no longer required after the current function finishes execution. These handles are restricted to allocation on the Rust stack. Local handles are lightweight and short-lived, primarily employed in local operations.

They are supervised by [HandleScopes](#handlescopes), necessitating the presence of a `v8::HandleScope` on the stack during their creation. Furthermore, they remain valid only within the active HandleScope during their instantiation.

Most of the V8 API uses Local handles to work with JavaScript values or return them from functions.

The following JavaScript and Rust code are mostly equivalent:

```js
function getFoo(obj) {
  return obj.foo;
}
```

```rust
fn getFoo(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    let object = v8::Local::<v8::Object>::try_from(args.get(0)).unwrap();
    let field = v8::String::new(scope, "foo").unwrap();
    let foo = object.get(scope, field.into()).unwrap();
    rv.set(foo.into());
}
```

`v8::Global` handles are suitable for storing objects across multiple independent operations. It is crucial to explicitly deallocate them when they are no longer in use.

Because of their persistent nature, these handles are deemed to have a lifetime equivalent to `'static`.

```rust
let number_1 = v8::Integer::new(scope, 1);
let number_1_global = v8::Global::new(scope, number_1);
```

Safely extracting the object stored in the handle, such as retrieving `*Object` from a Local, can be done by dereferencing the handle. The value remains under the control of a handle behind the scenes, and the same rules governing handles are applicable to these values.

#### `HandleScopes`

Obviously, creating a local handle for every object can result in an excessive number of handles. The `v8::HandleScope` allocated on the stack, manages several local handles. Once a handle scope is established, all local handles are allocated within that scope. If a handle scope already exists and a new one is initiated, all allocations occur within the new handle scope until its deletion. After that, new handles will once more be allocated within the original handle scope.

Once the handle scope of a local handle is deleted, the garbage collector will cease tracking the object stored in the handle, and it might deallocate it.

<br/>
<img src="./assets/the-runtime-03.svg" height="280px" />
<br/><br/>

For more in-depth information about isolates, handles, etc. visit v8's advanced [guide](https://v8.dev/docs/embed#advanced-guide).

### Architecture

Dune is composed of mainly 2 separate parts:

<img src="./assets/the-runtime-04.svg" height="150px" />

#### `JavaScript Frontend`

This part encompasses public interfaces, APIs, and crucial functionalities that operate without direct sys-calls.

JavaScript operates within the "unprivileged side", lacking default access to the file system or network due to its sandboxed environment in V8. For file and network access, the frontend relies on `bindings` to establish connections with the more "privileged" Rust backend.

In simpler terms, numerous Dune APIs, particularly those related to file system operations, are executed on the JavaScript side by primarily managing and converting data. These transformed data are then dispatched to the Rust backend via the `bindings` interface, with JavaScript subsequently awaiting the result to be returned, whether this occurs synchronously or asynchronously.

#### `Rust Backend`

The powerful part of the system, known as the "privileged side", possesses the capability to access files, networks, and the system environment, it is built using **Rust**.

For those unfamiliar with the language, Rust is a systems programming language developed by Mozilla, emphasizing memory safety and concurrency. It finds applications in projects such as [Deno](https://deno.com/) and [SurrealDB](https://surrealdb.com/), among others.

### Bindings

As previously explained, bindings act as the `bridge` linking the JavaScript frontend to the Rust backend. These bindings primarily consist of Rust functions that are initialized during the system's startup process and subsequently made accessible to the JavaScript side.

These bindings are structured into **namespaces**, which means that in practical terms, JavaScript can request specific Rust functions from designated namespaces such as `stdio` or `net`. Each namespace comprises functions customized to execute specific actions within its designated scope.

**Example: How console.log actually works?**

To understand this process, let's explore how something passed to `console.log` ultimately appears in our terminal.

```js
// File: /src/js/console.js

/**
 * Outputs data to the stdout stream.
 *
 * @param  {...any} args
 */

log(...args) {
  const output = args.map((arg) => stringify(arg)).join(' ');
  process.stdout.write(`${output}\n`);
}
```

The provided code snippet is part of the **console** module. It takes multiple arguments, transforms each argument into a string (which includes formatting JavaScript values), and then passes the resulting string to `process.stdout.write`.

Let's take a look how the `process.stdout.write` is implemented.

```js
// File: /src/js/main.js

Object.defineProperty(process, 'stdout', {
  get() {
    return {
      write: process.binding('stdio').write,
      end() {},
    };
  },
  configurable: true,
});
```

Let's break it down:

1. **`Object.defineProperty(process, 'stdout', {...});`**: This line alters the stdout property of the process object using the `Object.defineProperty` method. It defines how the stdout property can be accessed and manipulated.

2. **`write: process.binding('stdio').write`**: This line assigns the write property to the write method from the **stdio** namespace. In simpler terms, it means that when `process.stdout.write` is invoked, it calls the write method from the `stdio` binding, governing the handling of output data.

This approach is widespread throughout the codebase. Whenever we need to use a Rust function, we employ the `process.binding` method and specify the desired namespace.

**Let's dig into the Rust backend then** 👷‍♂️

```rust
// File: /src/stdio.rs

set_function_to(scope, target, "write", write);

/* .. more code .. */

/// Writes data to the stdout stream.
fn write(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Convert string to bytes.
    let content = args.get(0).to_rust_string_lossy(scope);
    let content = content.as_bytes();
    // Flush bytes to stdout.
    io::stdout().write_all(content).unwrap();
    io::stdout().flush().unwrap();
}

```

This looks simple enough, even for Rust:

1. **`set_function_to(...)`**: This line is setting a function named `write` under the **stdio** binding namespace.

2. **`args.get(0).to_rust_string_lossy(scope)`**: Retrieves the initial argument passed from the JavaScript side when the `write` function was called. It then converts this argument into a Rust string.

3. **`content.as_bytes()`**: The `content` string is then converted into a byte slice, which is necessary for writing to standard output.

4. **`io::stdout().write_all(content);`**: This line uses the Rust standard library `io::stdout()` to write the content byte slice to the **standard output**. `write_all` writes all the bytes at once.

The **standard output** (stdout) is the default file descriptor where a process can write output. In simpler terms, any program wanting to display information in the `terminal` must send that information to this output `stream`.

Certainly, the provided example represents the fundamental workings of bindings in Dune. This process remains uniform across all scenarios. The variation lies in the complexity of the code, which escalates according to the required functionality.
