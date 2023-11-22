// Process APIs
//
// This module contains part of the functions/attributes of the Node.js' process object.
// https://nodejs.org/dist/latest-v17.x/docs/api/process.html

use crate::bindings::create_object_under;
use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::throw_exception;
use crate::bindings::BINDINGS;
use crate::JsRuntime;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;
use std::process::Command;

lazy_static! {
    static ref VERSIONS: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("dune", env!("CARGO_PKG_VERSION"));
        map.insert("v8", v8::V8::get_version());
        map
    };
}

pub fn initialize<'s>(
    scope: &mut v8::ContextScope<'s, v8::EscapableHandleScope>,
    global: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Object> {
    // This represents the global `process` object.
    let process = create_object_under(scope, global, "process");

    // `process.cwd()` - current working directory.
    set_function_to(scope, process, "cwd", cwd);

    // `process.exit([code])` - exits the program with the given code.
    set_function_to(scope, process, "exit", exit);

    // `process.memoryUsage()` - an object describing the memory usage.
    set_function_to(scope, process, "memoryUsage", memory_usage);

    // `process.nextTick()` - adds callback to the "next tick queue".
    set_function_to(scope, process, "nextTick", next_tick);

    // `process.uptime()` - a number describing the amount of time (in seconds) the process is running.
    set_function_to(scope, process, "uptime", uptime);

    // `process.kill()` - sends the signal to the process identified by pid.
    set_function_to(scope, process, "kill", kill);

    // `process.binding()` - exposes native modules to JavaScript.
    set_function_to(scope, process, "binding", bind);

    process
}

/// Refreshes the static values of the process object.
pub fn refresh(scope: &mut v8::HandleScope) {
    // Get access to the process object.
    let context = scope.get_current_context();
    let global = context.global(scope);
    let key = v8::String::new(scope, "process").unwrap();
    let process = global.get(scope, key.into()).unwrap();
    let process = v8::Local::<v8::Object>::try_from(process).unwrap();

    // `process.argv` - an array containing the command-line arguments passed
    //  when the dune process was launched.
    let arguments: Vec<String> = env::args().collect();
    let argv = v8::Array::new(scope, arguments.len() as i32);

    arguments.iter().enumerate().for_each(|(i, arg)| {
        let index = i as u32;
        let value = v8::String::new(scope, arg.as_str()).unwrap();
        argv.set_index(scope, index, value.into()).unwrap();
    });

    set_property_to(scope, process, "argv", argv.into());

    // `process.env` - an object containing the user environment.
    let environment: Vec<(String, String)> = env::vars().collect();
    let env = v8::Object::new(scope);

    environment.iter().for_each(|(key, value)| {
        let value = v8::String::new(scope, value.as_str()).unwrap();
        set_constant_to(scope, env, key.as_str(), value.into());
    });

    set_property_to(scope, process, "env", env.into());

    // `process.pid` - PID of the current process.
    let id = v8::Number::new(scope, std::process::id() as f64);

    set_property_to(scope, process, "pid", id.into());

    // `process.platform` - a string identifying the operating system platform.
    let platform = v8::String::new(scope, env::consts::OS).unwrap();

    set_property_to(scope, process, "platform", platform.into());

    // `process.version` - the dune version.
    let version = format!("v{}", VERSIONS.get("dune").unwrap());
    let version = v8::String::new(scope, version.as_str()).unwrap();

    set_property_to(scope, process, "version", version.into());

    // `process.versions` - an object listing the version strings of dune and its dependencies.
    let versions = v8::Object::new(scope);

    VERSIONS.iter().for_each(|(name, version)| {
        let version = v8::String::new(scope, version).unwrap();
        set_constant_to(scope, versions, name, version.into());
    });

    set_property_to(scope, process, "versions", versions.into());
}

/// Current working directory.
fn cwd(scope: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    match env::current_dir() {
        Ok(path) => {
            let path = path.into_os_string().into_string().unwrap();
            let path = v8::String::new(scope, path.as_str()).unwrap();

            rv.set(path.into());
        }
        Err(_) => {
            let undefined = v8::undefined(scope);
            rv.set(undefined.into());
        }
    }
}

/// Exits the program with the given code.
fn exit(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Exit the program when value is not valid i32.
    match args.get(0).to_int32(scope) {
        Some(code) => std::process::exit(code.value()),
        None => std::process::exit(0),
    }
}

/// Returns an object describing the memory usage.
fn memory_usage(
    scope: &mut v8::HandleScope,
    _: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get HeapStatistics from v8.
    let mut stats = v8::HeapStatistics::default();
    scope.get_heap_statistics(&mut stats);

    let total_heap = v8::Number::new(scope, stats.total_heap_size() as f64);
    let used_heap = v8::Number::new(scope, stats.used_heap_size() as f64);
    let external = v8::Number::new(scope, stats.external_memory() as f64);

    let memory_usage = v8::Object::new(scope);

    set_property_to(scope, memory_usage, "heapTotal", total_heap.into());
    set_property_to(scope, memory_usage, "heapUsed", used_heap.into());
    set_property_to(scope, memory_usage, "external", external.into());

    rv.set(memory_usage.into());
}

/// Adds callback to the "next tick queue".
fn next_tick(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Make a global handle out the the function.
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let callback = v8::Global::new(scope, callback);

    // Convert params argument (Array<Local<Value>>) to Rust vector.
    let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
        Ok(params) => {
            (0..params.length()).fold(Vec::<v8::Global<v8::Value>>::new(), |mut acc, i| {
                let param = params.get_index(scope, i).unwrap();
                acc.push(v8::Global::new(scope, param));
                acc
            })
        }
        Err(_) => vec![],
    };

    let state_rc = JsRuntime::state(scope);

    state_rc
        .borrow_mut()
        .next_tick_queue
        .push((callback, params));
}

/// A number describing the amount of time (in seconds) the process is running.
fn uptime(scope: &mut v8::HandleScope, _: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get access to runtime's state.
    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();

    // Calculate uptime duration in seconds with millis precision.
    let uptime = state.startup_moment.elapsed().as_millis() as f64 / 1000.0;
    let uptime = v8::Number::new(scope, uptime);

    rv.set(uptime.into());
}

#[cfg(target_family = "unix")]
fn kill(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get PID and SIGNAL arguments
    let pid = args.get(0).to_rust_string_lossy(scope);
    let signal = args.get(1).to_rust_string_lossy(scope);

    // Check if the value is a valid NIX signal.
    if !nix::sys::signal::Signal::iterator()
        .map(|s| s.as_str())
        .any(|v| *v == signal)
    {
        throw_exception(scope, &format!("Invalid signal: {signal}"));
        return;
    }

    // Try to kill the process.
    if let Err(e) = Command::new("kill")
        .args([&format!("-{signal}"), &pid])
        .output()
    {
        throw_exception(scope, &e.to_string());
    }
}

#[cfg(target_family = "windows")]
fn kill(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    // Get PID argument.
    let pid = args.get(0).to_rust_string_lossy(scope);
    // Try to kill the process.
    if let Err(e) = Command::new("Taskkill").args(["/F", "/PID", &pid]).output() {
        throw_exception(scope, &e.to_string());
    }
}

/// Exposes native modules to JavaScript.
fn bind(scope: &mut v8::HandleScope, args: v8::FunctionCallbackArguments, mut rv: v8::ReturnValue) {
    // Get requested native binding.
    let request = args.get(0).to_rust_string_lossy(scope);

    match BINDINGS.get(request.as_str()) {
        // Initialize binding.
        Some(initializer) => {
            let binding = initializer(scope);
            let binding = v8::Local::new(scope, binding);
            rv.set(binding.into());
        }
        // Throw exception.
        None => {
            let message = format!("No such module: \"{request}\"");
            let message = v8::String::new(scope, &message).unwrap();
            let exception = v8::Exception::error(scope, message);
            scope.throw_exception(exception);
        }
    };
}
