use colored::*;
use rusty_v8 as v8;

// NOTE: See the following to get full error information.
// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
pub fn to_pretty_string(try_catch: &mut v8::TryCatch<v8::HandleScope>) -> String {
    let exception_string = try_catch
        .exception()
        .unwrap()
        .to_string(try_catch)
        .unwrap()
        .to_rust_string_lossy(try_catch);

    let message = match try_catch.message() {
        Some(message) => message,
        None => {
            return exception_string;
        }
    };

    let filename = message.get_script_resource_name(try_catch).map_or_else(
        || "(unknown)".into(),
        |s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
        },
    );

    let line = message.get_line_number(try_catch).unwrap_or_default();
    let column = message.get_start_column();

    format!("{} ({}:{}:{})", exception_string, filename, line, column)
}

// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
pub fn report_exceptions(try_catch: &mut v8::TryCatch<v8::HandleScope>) {
    let exception = try_catch.exception().unwrap();
    let exception_string = exception
        .to_string(try_catch)
        .unwrap()
        .to_rust_string_lossy(try_catch);
    let message = if let Some(message) = try_catch.message() {
        message
    } else {
        eprintln!("{}", exception_string);
        return;
    };

    eprintln!("{} {}", "Uncaught".red().bold(), exception_string);

    // Print line of source code.
    let source_line = message
        .get_source_line(try_catch)
        .map(|s| {
            s.to_string(try_catch)
                .unwrap()
                .to_rust_string_lossy(try_catch)
        })
        .unwrap();

    eprintln!("{}", source_line);

    // Print wavy underline (GetUnderline is deprecated).
    let start_column = message.get_start_column();
    let end_column = message.get_end_column();

    for _ in 0..start_column {
        eprint!(" ");
    }

    for _ in start_column..end_column {
        let mark = "^".red();
        eprint!("{}", mark);
    }

    eprintln!();

    // Print stack trace
    let stack_trace = match try_catch.stack_trace() {
        Some(stack_trace) => stack_trace,
        None => return,
    };
    let stack_trace = unsafe { v8::Local::<v8::String>::cast(stack_trace) };
    let stack_trace = stack_trace
        .to_string(try_catch)
        .map(|s| s.to_rust_string_lossy(try_catch));

    if let Some(stack_trace) = stack_trace {
        eprintln!("{}", stack_trace.dimmed());
    }
}
