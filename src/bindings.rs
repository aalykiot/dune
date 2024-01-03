use crate::dns;
use crate::errors::extract_error_code;
use crate::errors::IoError;
use crate::file;
use crate::http_parser;
use crate::net;
use crate::perf_hooks;
use crate::process;
use crate::promise;
use crate::stdio;
use crate::timers;
use anyhow::Error;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ffi::c_void;

/// Function pointer for the bindings initializers.
type BindingInitFn = fn(&mut v8::HandleScope<'_>) -> v8::Global<v8::Object>;

lazy_static! {
    pub static ref BINDINGS: HashMap<&'static str, BindingInitFn> = {
        let bindings: Vec<(&'static str, BindingInitFn)> = vec![
            ("stdio", stdio::initialize),
            ("timers", timers::initialize),
            ("fs", file::initialize),
            ("perf_hooks", perf_hooks::initialize),
            ("dns", dns::initialize),
            ("net", net::initialize),
            ("promise", promise::initialize),
            ("http_parser", http_parser::initialize),
        ];
        HashMap::from_iter(bindings.into_iter())
    };
}

/// Populates a new JavaScript context with low-level Rust bindings.
pub fn create_new_context<'s>(scope: &mut v8::HandleScope<'s, ()>) -> v8::Local<'s, v8::Context> {
    // Here we need an EscapableHandleScope so V8 doesn't drop the
    // newly created HandleScope on return. (https://v8.dev/docs/embed#handles-and-garbage-collection)
    let scope = &mut v8::EscapableHandleScope::new(scope);

    // Create and enter a new JavaScript context.
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
            println!("{value}");
        },
    );

    // Expose low-level functions to JavaScript.
    process::initialize(scope, global);
    scope.escape(context)
}

/// Adds a property with the given name and value, into the given object.
pub fn set_property_to(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value);
}

/// Adds a read-only property with the given name and value, into the given object.
pub fn set_constant_to(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    name: &str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), value, v8::PropertyAttribute::READ_ONLY);
}

/// Adds a `Function` object which calls the given Rust function
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

/// Creates an object with a given name under a `target` object.
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

/// Stores a Rust type inside a v8 object.
pub fn set_internal_ref<T>(
    scope: &mut v8::HandleScope<'_>,
    target: v8::Local<v8::Object>,
    index: usize,
    data: T,
) {
    let boxed_ref = Box::new(data);
    let addr = Box::leak(boxed_ref) as *mut T as *mut c_void;
    let v8_ext = v8::External::new(scope, addr);

    target.set_internal_field(index, v8_ext.into());
}

/// Gets a previously stored Rust type from a v8 object.
pub fn get_internal_ref<'s, T>(
    scope: &mut v8::HandleScope<'s>,
    source: v8::Local<v8::Object>,
    index: usize,
) -> &'s mut T {
    let v8_ref = source.get_internal_field(scope, index).unwrap();
    let stored_item = unsafe { v8::Local::<v8::External>::cast(v8_ref) };
    let stored_item = stored_item.value() as *mut T;

    unsafe { &mut *stored_item }
}

/// Sets error code to exception if possible.
pub fn set_exception_code(
    scope: &mut v8::HandleScope<'_>,
    exception: v8::Local<v8::Value>,
    error: &Error,
) {
    let exception = exception.to_object(scope).unwrap();
    if let Some(error) = error.downcast_ref::<IoError>() {
        if let Some(code) = extract_error_code(error) {
            let key = v8::String::new(scope, "code").unwrap();
            let value = v8::String::new(scope, &format!("ERR_{code}")).unwrap();
            exception.set(scope, key.into(), value.into());
        }
    }
}

/// Useful utility to throw v8 exceptions.
pub fn throw_exception(scope: &mut v8::HandleScope, err: &Error) {
    let message = err.to_string().to_owned();
    let message = v8::String::new(scope, &message).unwrap();
    let exception = v8::Exception::error(scope, message);
    set_exception_code(scope, exception, err);
    scope.throw_exception(exception);
}

/// Useful utility to throw v8 type errors.
pub fn throw_type_error(scope: &mut v8::HandleScope, message: &str) {
    let message = v8::String::new(scope, message).unwrap();
    let exception = v8::Exception::type_error(scope, message);
    scope.throw_exception(exception);
}
