use crate::bindings::get_internal_ref;
use crate::bindings::set_function_to;
use crate::bindings::set_property_to;
use crate::bindings::throw_exception;
use crate::bindings::wrap_gc_dropped;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
use crabuv::signals::Kind;
use crabuv::signals::Policy;
use crabuv::signals::SignalHandle;
use std::rc::Rc;

#[cfg(windows)]
const SIGNALS: [(&str, i32); 6] = [
    ("SIGABRT", Kind::SIGABRT),
    ("SIGFPE", Kind::SIGFPE),
    ("SIGILL", Kind::SIGILL),
    ("SIGINT", Kind::SIGINT),
    ("SIGSEGV", Kind::SIGSEGV),
    ("SIGTERM", Kind::SIGTERM),
];

#[cfg(not(windows))]
const SIGNALS: [(&str, i32); 29] = [
    ("SIGABRT", Kind::SIGABRT),
    ("SIGALRM", Kind::SIGALRM),
    ("SIGBUS", Kind::SIGBUS),
    ("SIGCHLD", Kind::SIGCHLD),
    ("SIGCONT", Kind::SIGCONT),
    ("SIGFPE", Kind::SIGFPE),
    ("SIGHUP", Kind::SIGHUP),
    ("SIGILL", Kind::SIGILL),
    ("SIGINT", Kind::SIGINT),
    ("SIGIO", Kind::SIGIO),
    ("SIGKILL", Kind::SIGKILL),
    ("SIGPIPE", Kind::SIGPIPE),
    ("SIGPROF", Kind::SIGPROF),
    ("SIGQUIT", Kind::SIGQUIT),
    ("SIGSEGV", Kind::SIGSEGV),
    ("SIGSTOP", Kind::SIGSTOP),
    ("SIGSYS", Kind::SIGSYS),
    ("SIGTERM", Kind::SIGTERM),
    ("SIGTRAP", Kind::SIGTRAP),
    ("SIGTSTP", Kind::SIGTSTP),
    ("SIGTTIN", Kind::SIGTTIN),
    ("SIGTTOU", Kind::SIGTTOU),
    ("SIGURG", Kind::SIGURG),
    ("SIGUSR1", Kind::SIGUSR1),
    ("SIGUSR2", Kind::SIGUSR2),
    ("SIGVTALRM", Kind::SIGVTALRM),
    ("SIGWINCH", Kind::SIGWINCH),
    ("SIGXCPU", Kind::SIGXCPU),
    ("SIGXFSZ", Kind::SIGXFSZ),
];

pub fn initialize(scope: &mut v8::PinScope) -> v8::Global<v8::Object> {
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
    fn run(&mut self, scope: &mut v8::PinScope) {
        let undefined = v8::undefined(scope).into();
        let callback = v8::Local::new(scope, (*self.0).clone());
        v8::tc_scope!(let tc_scope, scope);

        callback.call(tc_scope, undefined, &[]);

        // On exception, report it and exit.
        if tc_scope.has_caught() {
            let exception = tc_scope.exception().unwrap();
            let exception = v8::Global::new(tc_scope, exception);
            let state = JsRuntime::state(tc_scope);
            state.borrow_mut().exceptions.capture_exception(exception);
        }
    }
}

/// Registers a signal listener to the event-loop.
fn start_signal(
    scope: &mut v8::PinScope,
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
        move |_: SignalHandle, _: i32| {
            let mut state = state_rc.borrow_mut();
            let future = SignalFuture(Rc::clone(&callback));
            state.pending_futures.push(Box::new(future));
        }
    };

    // Schedule a new signal listener to the event-loop.
    let state = state_rc.borrow();
    let signal = state
        .handle
        .signal(signal_type, Policy::Persistent, signal_cb)
        .unwrap();

    let signal = wrap_gc_dropped(scope, signal);
    rv.set(signal.into());
}

/// Removes a signal listener to the event-loop.
fn cancel_signal(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _: v8::ReturnValue,
) {
    // Get the signal handle from the object.
    let wrapper = args.get(0).to_object(scope).unwrap();
    let signal = get_internal_ref::<SignalHandle>(scope, wrapper, 0);

    signal.stop();
}
