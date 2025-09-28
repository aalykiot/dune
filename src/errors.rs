use anyhow::Error;
use colored::*;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;
pub use std::io::Error as IoError;
use std::io::ErrorKind;

/// A simple error type that lets the creator specify both the error message and
/// the error class name.
#[derive(Debug)]
struct CustomError {
    class: &'static str,
    message: Cow<'static, str>,
}

impl CustomError {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(class: &'static str, message: impl Into<Cow<'static, str>>) -> Error {
        CustomError {
            class,
            message: message.into(),
        }
        .into()
    }
}

impl std::error::Error for CustomError {}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class.red().bold(), self.message)
    }
}

pub fn generic_error(message: impl Into<Cow<'static, str>>) -> Error {
    CustomError::new("Error", message)
}

/// Represents an exception coming from V8.
#[derive(Eq, PartialEq, Clone, Default)]
pub struct JsError {
    pub message: String,
    pub resource_name: String,
    pub source_line: Option<String>,
    pub line_number: Option<i64>,
    pub start_column: Option<i64>,
    pub end_column: Option<i64>,
    pub stack: Option<String>,
}

impl JsError {
    // https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
    pub fn from_v8_exception<'a>(
        scope: &'a mut v8::PinScope,
        rejection: v8::Local<'a, v8::Value>,
        prefix: Option<&str>,
    ) -> Self {
        let message = v8::Exception::create_message(scope, rejection);
        let mut message_value =
            message
                .get(scope)
                .to_rust_string_lossy(scope)
                .replacen("Uncaught ", "", 1);

        // Check if message needs prefixing.
        if let Some(value) = prefix {
            message_value.insert_str(0, value);
        }

        let resource_name = message.get_script_resource_name(scope).map_or_else(
            || "(unknown)".into(),
            |s| s.to_string(scope).unwrap().to_rust_string_lossy(scope),
        );

        let source_line = message
            .get_source_line(scope)
            .map(|s| s.to_string(scope).unwrap().to_rust_string_lossy(scope));

        let line_number = message.get_line_number(scope).map(|num| num as i64);

        let start_column = Some(message.get_start_column() as i64);
        let end_column = Some(message.get_end_column() as i64);

        // Cast v8::PromiseRejectMessage to v8::Object so we can take it's `.stack` property.
        let exception = v8::Local::<v8::Object>::try_from(rejection);

        // Ignore source line when no stack-trace is available.
        let source_line = exception.map(|_| source_line).map(|s| s.unwrap()).ok();

        let stack = exception
            .map(|exception| {
                let stack = v8::String::new(scope, "stack").unwrap();
                let stack = exception.get(scope, stack.into());
                let stack: Option<v8::Local<v8::String>> = stack.and_then(|s| s.try_into().ok());
                stack.map(|s| s.to_rust_string_lossy(scope))
            })
            .map(|stack| stack.unwrap_or_default())
            .ok();

        JsError {
            message: message_value,
            resource_name,
            source_line,
            line_number,
            start_column,
            end_column,
            stack,
        }
    }
}

impl std::error::Error for JsError {}

impl Display for JsError {
    /// Displays a minified version of the error.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Unwrap values.
        let line = self.line_number.unwrap_or_default();
        let column = self.start_column.unwrap_or_default();
        write!(
            f,
            "{} {} ({}:{}:{})",
            "Uncaught".red().bold(),
            self.message,
            self.resource_name,
            line,
            column
        )
    }
}

impl Debug for JsError {
    /// Displays a full version of the error with stack-trace.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Output exception information.
        write!(f, "{} {}", "Uncaught".red().bold(), self.message)?;

        // Output source-line if exists.
        match self.source_line.as_ref() {
            Some(source_line) if !source_line.is_empty() => {
                // Log the source-line.
                writeln!(f, "\n{source_line}")?;

                // Indicate the position where the error was thrown.
                let start_column = self.start_column.unwrap_or_default();
                let end_column = self.end_column.unwrap_or_default();

                for _ in 0..start_column {
                    write!(f, " ")?;
                }

                for _ in start_column..end_column {
                    let mark = "^".red();
                    write!(f, "{mark}")?;
                }

                // Print stacktrace if available.
                if let Some(stack) = self.stack.as_ref() {
                    write!(f, "\n{}", stack.dimmed())?;
                }
            }
            _ => {}
        };

        Ok(())
    }
}

pub fn unwrap_or_exit<T>(result: Result<T, Error>) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    }
}

pub fn report_and_exit(error: JsError) {
    eprint!("{error:?}");
    std::process::exit(1);
}

/// Returns a string representation of the IO error's code.
pub fn extract_error_code(err: &IoError) -> Option<&'static str> {
    match err.kind() {
        ErrorKind::AddrInUse => Some("ADDR_IN_USE"),
        ErrorKind::AddrNotAvailable => Some("ADDR_NOT_AVAILABLE"),
        ErrorKind::AlreadyExists => Some("ALREADY_EXISTS"),
        ErrorKind::BrokenPipe => Some("BROKEN_PIPE"),
        ErrorKind::ConnectionAborted => Some("CONNECTION_ABORTED"),
        ErrorKind::ConnectionRefused => Some("CONNECTION_REFUSED"),
        ErrorKind::ConnectionReset => Some("CONNECTION_RESET"),
        ErrorKind::Interrupted => Some("INTERRUPTED"),
        ErrorKind::InvalidData => Some("INVALID_DATA"),
        ErrorKind::NotConnected => Some("NOT_CONNECTED"),
        ErrorKind::NotFound => Some("NOT_FOUND"),
        ErrorKind::PermissionDenied => Some("PERMISSION_DENIED"),
        ErrorKind::TimedOut => Some("TIMED_OUT"),
        ErrorKind::UnexpectedEof => Some("UNEXPECTED_EOF"),
        ErrorKind::WouldBlock => Some("WOULD_BLOCK"),
        ErrorKind::WriteZero => Some("WRITE_ZERO"),
        _ => None,
    }
}
