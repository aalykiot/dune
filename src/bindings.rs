use crate::console;
use crate::dns;
use crate::errors::report_and_exit;
use crate::exceptions;
use crate::file;
use crate::http_parser;
use crate::net;
use crate::perf_hooks;
use crate::process;
use crate::promise;
use crate::runtime::check_exceptions;
use crate::runtime::JsRuntime;
use crate::signals;
use crate::sqlite;
use crate::timers;
use crate::tty;
use crate::v8_utils::set_function_to;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Function pointer for the bindings initializers.
type BindingInitFn = fn(&mut v8::PinScope) -> v8::Global<v8::Object>;

pub static BINDINGS: LazyLock<HashMap<&'static str, BindingInitFn>> = LazyLock::new(|| {
    let bindings: Vec<(&'static str, BindingInitFn)> = vec![
        ("timers", timers::initialize),
        ("fs", file::initialize),
        ("console", console::initialize),
        ("perf_hooks", perf_hooks::initialize),
        ("dns", dns::initialize),
        ("net", net::initialize),
        ("promise", promise::initialize),
        ("http_parser", http_parser::initialize),
        ("signals", signals::initialize),
        ("exceptions", exceptions::initialize),
        ("sqlite", sqlite::initialize),
        ("tty", tty::initialize),
    ];
    HashMap::from_iter(bindings)
});

/// Populates a new JavaScript context with low-level Rust bindings.
pub fn create_new_context<'s, 'i>(
    scope: &mut v8::PinScope<'s, 'i, ()>,
) -> v8::Local<'s, v8::Context> {
    // Create and enter a new JavaScript context.
    let context = v8::Context::new(scope, Default::default());
    let global = context.global(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    set_function_to(scope, global, "print", global_print);
    set_function_to(scope, global, "reportError", global_report_error);
    set_function_to(scope, global, "$$queueMicro", global_queue_micro);

    // Expose low-level functions to JavaScript.
    process::initialize(scope, global);
    context
}

// Simple print function bound to Rust's println! macro.
fn global_print(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, _: v8::ReturnValue) {
    let value = args.get(0).to_rust_string_lossy(scope);
    println!("{value}");
}

// This method may be used to report errors to global event handlers.
// https://html.spec.whatwg.org/multipage/webappapis.html#report-the-exception
fn global_report_error(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    let exception = v8::Global::new(scope, args.get(0));
    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    state.exceptions.capture_exception(exception);
    drop(state);

    if let Some(error) = check_exceptions(scope) {
        report_and_exit(error);
    }
}

// This method queues a microtask to invoke callback.
fn global_queue_micro(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
    let state_rc = JsRuntime::state(scope);
    let state = state_rc.borrow();
    let ctx = unsafe { state.context.open(scope) };

    if let Some(queue) = ctx.get_microtask_queue() {
        queue.enqueue_microtask(scope, callback);
    }
}
