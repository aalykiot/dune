use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::throw_exception;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
use dune_event_loop::LoopHandle;
use dune_event_loop::Signal;
use std::rc::Rc;

#[cfg(windows)]
const SIGNALS: [(&str, i32); 6] = [
    ("SIGABRT", Signal::SIGABRT),
    ("SIGFPE", Signal::SIGFPE),
    ("SIGILL", Signal::SIGILL),
    ("SIGINT", Signal::SIGINT),
    ("SIGSEGV", Signal::SIGSEGV),
    ("SIGTERM", Signal::SIGTERM),
];

#[cfg(not(windows))]
const SIGNALS: [(&str, i32); 29] = [
    ("SIGABRT", Signal::SIGABRT),
    ("SIGALRM", Signal::SIGALRM),
    ("SIGBUS", Signal::SIGBUS),
    ("SIGCHLD", Signal::SIGCHLD),
    ("SIGCONT", Signal::SIGCONT),
    ("SIGFPE", Signal::SIGFPE),
    ("SIGHUP", Signal::SIGHUP),
    ("SIGILL", Signal::SIGILL),
    ("SIGINT", Signal::SIGINT),
    ("SIGIO", Signal::SIGIO),
    ("SIGKILL", Signal::SIGKILL),
    ("SIGPIPE", Signal::SIGPIPE),
    ("SIGPROF", Signal::SIGPROF),
    ("SIGQUIT", Signal::SIGQUIT),
    ("SIGSEGV", Signal::SIGSEGV),
    ("SIGSTOP", Signal::SIGSTOP),
    ("SIGSYS", Signal::SIGSYS),
    ("SIGTERM", Signal::SIGTERM),
    ("SIGTRAP", Signal::SIGTRAP),
    ("SIGTSTP", Signal::SIGTSTP),
    ("SIGTTIN", Signal::SIGTTIN),
    ("SIGTTOU", Signal::SIGTTOU),
    ("SIGURG", Signal::SIGURG),
    ("SIGUSR1", Signal::SIGUSR1),
    ("SIGUSR2", Signal::SIGUSR2),
    ("SIGVTALRM", Signal::SIGVTALRM),
    ("SIGWINCH", Signal::SIGWINCH),
    ("SIGXCPU", Signal::SIGXCPU),
    ("SIGXFSZ", Signal::SIGXFSZ),
];

pub fn initialize(scope: &mut v8::HandleScope) -> v8::Global<v8::Object> {
    // Create local JS object.
    let target = v8::Object::new(scope);
    let signals = v8::Array::new(scope, SIGNALS.len() as i32);

    set_function_to(scope, target, "startSignal", start_signal);
    set_function_to(scope, target, "cancelSignal", cancel_signal);

    // Create a JS array containing the available signals.
    SIGNALS.iter().enumerate().for_each(|(i, (signal, _))| {
        let index = i as u32;
        let signal = v8::String::new(scope, signal).unwrap();
        signals.set_index(scope, index, signal.into()).unwrap();
    });

    set_property_to(scope, target, "signals", signals.into());

    // Return v8 global handle.
    v8::Global::new(scope, target)
}

struct SignalFuture(Rc<v8::Global<v8::Function>>);

impl JsFuture for SignalFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let undefined = v8::undefined(scope).into();
        let callback = v8::Local::new(scope, (*self.0).clone());
        let tc_scope = &mut v8::TryCatch::new(scope);

        callback.call(tc_scope, undefined, &[]);

        // On exception, report it and exit.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let state = JsRuntime::state(tc_scope);
            state.borrow_mut().exceptions.emit_exception(exception);
        }
    }
}

/// Registers a signal listener to the event-loop.
fn start_signal(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Get signal type from javascript.
    let signal_type = args.get(0).to_rust_string_lossy(scope);
    let signal_type = match SIGNALS
        .iter()
        .find(|(signal, _)| *signal == signal_type.as_str())
    {
        Some((_, signum)) => signum.to_owned(),
        None => {
            let exception = anyhow!("Invalid signal provided.");
            throw_exception(scope, &exception);
            return;
        }
    };

    // Get signal's listener handler.
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));

    let state_rc = JsRuntime::state(scope);

    let signal_cb = {
        let state_rc = state_rc.clone();
        move |_: LoopHandle, _: i32| {
            let mut state = state_rc.borrow_mut();
            let future = SignalFuture(Rc::clone(&callback));
            state.pending_futures.push(Box::new(future));
        }
    };

    // Schedule a new signal listener to the event-loop.
    let state = state_rc.borrow();
    let id = state.handle.signal_start(signal_type, signal_cb).unwrap();

    // Return timeout's internal id.
    rv.set(v8::Number::new(scope, id as f64).into());
}

/// Removes a signal listener to the event-loop.
fn cancel_signal(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get handlers internal token.
    let id = args.get(0).int32_value(scope).unwrap() as u32;
    let state_rc = JsRuntime::state(scope);

    state_rc.borrow().handle.signal_stop(&id);
}
