# Dune - The Repo Structure

In this section, we'll dive into the current structure of the Dune repository so you can navigate it comfortably.

Here is the link to the repo: https://github.com/aalykiot/dune

```
# DIRECTORIES
src/: Rust Backend
js/: JavaScript Frontend
tools/: Standalone commands
tests/: Functional tests
examples/: Useful dune use-cases
scripts/: Build scripts for some platforms

# FILES
Cargo.toml: Dependency info for Rust
package.json: Node dependencies (mostly repo metadata)
build.rs: Cargo build rules
```

#### src/

The Rust backend lives here.

1. **`main.rs`**: contains the main Rust function. It serves as the **actual** starting point when you execute the Duno binary.

2. **`runtime.rs`**: contains the logic for initializing the runtime, executing JavaScript code, and managing the event-loop execution.

3. **`event_loop.rs`**: contains all the event-loop code that is required by Dune.

4. **`loaders.rs`**: contains the logic necessary to resolve and load various imports, such as URLs and modules.

5. **`bindings.rs`**: exposes the registered `bindings` to the JavaScript frontend.

6. **`modules.rs`**: contains the logic related to ECMAScript module resolution.

7. **`errors.rs`**: contains logic responsible for converting V8 exceptions into custom Rust errors, along with associated metadata information.

8. **`repl.rs`**: contains all the logic around Dune's available REPL interface.

These represent the core components, with the remaining files primarily defining bindings within a specific namespace.

#### js/

This is where code for the JavaScript frontend is located. Most APIs are defined under a `namespace` known in the Node.js world as `core modules`. For example, `readFile` is defined in the `fs.js` file.

1. **`console.js`**: A subset of the WHATWG console.

2. **`fs.js`**: Equivalent to Node.js' `fs` core module.

3. **`net.js`**: Equivalent to Node.js' `net` core module.

4. **`http.js`**: Equivalent to Node.js' `http` core module.

5. etc.

#### tools/

Standalone commands that Dune's CLI provides.

1. **`bundle.rs`**: Bundles everything into a single file.

2. **`compile.rs`**: Compiles script to standalone executable.

3. **`upgrade.rs`**: Upgrades to the latest dune version.

#### tests/

Functional tests for Dune.

Files with the extension `*.test.js` or `*.test.ts` contain the code executed for each test suite. These tests are designed to verify the proper functioning of all Dune's APIs.

See the [testing](https://github.com/aalykiot/dune/tree/main#testing) section for more details.

#### build.rs

Defines `cargo build` logic. Internally makes the current `version` available as an env variable.
