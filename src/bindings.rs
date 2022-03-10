use crate::process;
use crate::stdio;
use crate::timers;
use lazy_static::lazy_static;
use rusty_v8 as v8;
use std::collections::HashMap;

/// Function pointer for the bindings initializers.
type BindingInitFn = fn(&mut v8::HandleScope<'_>) -> v8::Global<v8::Object>;

lazy_static! {
    pub static ref BINDINGS: HashMap<&'static str, BindingInitFn> = {
        let bindings: Vec<(&'static str, BindingInitFn)> = vec![
            ("stdio", stdio::initialize),
            ("timer_wrap", timers::initialize),
        ];
        HashMap::from_iter(bindings.into_iter())
    };
}

// Populates a new JavaScript context with low-level Rust bindings.
pub fn create_new_context<'s>(scope: &mut v8::HandleScope<'s, ()>) -> v8::Local<'s, v8::Context> {
    // Here we need an EscapableHandleScope so V8 doesn't drop the
    // newly created HandleScope on return. (https://v8.dev/docs/embed#handles-and-garbage-collection)
    let scope = &mut v8::EscapableHandleScope::new(scope);
    // Creating and entering a new JavaScript context.
    let context = v8::Context::new(scope);
    let global = context.global(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    // Simple print function bound to Rust's println! macro (synchronous call).
    set_function_to(
        scope,
        global,
        "print",
        |scope: &mut v8::HandleScope,
         args: v8::FunctionCallbackArguments,
         mut _rv: v8::ReturnValue| {
            let value = args.get(0).to_rust_string_lossy(scope);
            println!("{}", value);
        },
    );
    // Here we're exposing low-level functionality to JavaScript.
    process::initialize(scope, global);
    scope.escape(context)
}

// Adds a property with the given name and value, into the given object.
pub fn set_property_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value);
}

// Adds a read-only property with the given name and value, into the given object.
pub fn set_constant_to<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), value, v8::READ_ONLY);
}

// Adds a `Function` object which calls the given Rust function
pub fn set_function_to(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let template = v8::FunctionTemplate::new(scope, callback);
    let val = template.get_function(scope).unwrap();
    target.set(scope, key.into(), val.into());
}

// Creates an object with a given name under a `target` object.
pub fn create_object_under<'s>(
    scope: &mut v8::HandleScope<'s>,
    target: v8::Local<v8::Object>,
    name: &'static str,
) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);
    let key = v8::String::new(scope, name).unwrap();
    let value = template.new_instance(scope).unwrap();
    target.set(scope, key.into(), value.into());
    value
}
