// Process API
//
// This module contains part of the functions/attributes of the Node.js' process object.
// https://nodejs.org/dist/latest-v17.x/docs/api/process.html

use crate::bindings::create_object_under;
use crate::bindings::set_constant_to;
use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::BINDINGS;
use lazy_static::lazy_static;
use rusty_v8 as v8;
use std::collections::HashMap;
use std::env;

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

    // `process.cwd()` - current working directory.
    set_function_to(
        scope,
        process,
        "cwd",
        |scope: &mut v8::HandleScope,
         _args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
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
        },
    );

    // `process.env` - an object containing the user environment.
    let environment: Vec<(String, String)> = env::vars().collect();
    let env = v8::Object::new(scope);

    environment.iter().for_each(|(key, value)| {
        let value = v8::String::new(scope, value.as_str()).unwrap();
        set_constant_to(scope, env, key.as_str(), value.into());
    });

    set_property_to(scope, process, "env", env.into());

    // `process.exit([code])` - exits the program with the given code.
    set_function_to(
        scope,
        process,
        "exit",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut _rv: v8::ReturnValue| {
            // Exit the program when value is not valid i32.
            match args.get(0).to_int32(scope) {
                Some(code) => std::process::exit(code.value() as i32),
                None => std::process::exit(0),
            }
        },
    );

    // `process.memoryUsage()` - an object describing the memory usage.
    set_function_to(
        scope,
        process,
        "memoryUsage",
        |scope: &mut v8::HandleScope,
         _args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
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
        },
    );

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

    // `process.binding` - exposes native modules to JavaScript.
    set_function_to(
        scope,
        process,
        "binding",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut rv: v8::ReturnValue| {
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
                    let message = format!("No such module: \"{}\"", request);
                    let message = v8::String::new(scope, &message).unwrap();
                    let exception = v8::Exception::error(scope, message);

                    scope.throw_exception(exception);
                }
            };
        },
    );

    process
}
