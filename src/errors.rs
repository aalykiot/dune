use anyhow::Error;
use colored::*;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;

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

pub fn unhandled_promise_rejection_error(message: impl Into<Cow<'static, str>>) -> Error {
    CustomError::new(
        "Uncaught",
        format!("Unhandled promise rejection: {}", message.into()),
    )
}

/// Represents an exception coming from V8.
#[derive(PartialEq, Clone)]
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
        scope: &'a mut v8::HandleScope,
        exception: v8::Local<'a, v8::Value>,
    ) -> Self {
        // Create a new HandleScope.
        let scope = &mut v8::HandleScope::new(scope);
        let message = v8::Exception::create_message(scope, exception);

        // Get error from thrown exception.
        let exception_string = exception
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        let resource_name = message.get_script_resource_name(scope).map_or_else(
            || "(unknown)".into(),
            |s| s.to_string(scope).unwrap().to_rust_string_lossy(scope),
        );

        let source_line = message
            .get_source_line(scope)
            .map(|s| s.to_string(scope).unwrap().to_rust_string_lossy(scope));

        let line_number = message
            .get_line_number(scope)
            .and_then(|v| v.try_into().ok());

        let start_column = message.get_start_column().try_into().ok();
        let end_column = message.get_end_column().try_into().ok();

        let exception: v8::Local<v8::Object> = exception.try_into().unwrap();

        // Access error.stack to ensure `prepareStackTrace()` has been called.
        let stack = v8::String::new(scope, "stack").unwrap();
        let stack = exception.get(scope, stack.into());
        let stack: Option<v8::Local<v8::String>> = stack.and_then(|s| s.try_into().ok());
        let stack = stack.map(|s| s.to_rust_string_lossy(scope));

        JsError {
            message: exception_string,
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
    /// Displays a full version of the error with stacktrace.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Output exception information.
        writeln!(f, "{} {}", "Uncaught".red().bold(), self.message)?;
        writeln!(f, "{}", self.source_line.as_ref().unwrap())?;

        // Indicate the position where the error was thrown.
        let start_column = self.start_column.unwrap_or_default();
        let end_column = self.end_column.unwrap_or_default();

        for _ in 0..start_column {
            write!(f, " ")?;
        }

        for _ in start_column..end_column {
            let mark = "^".red();
            write!(f, "{}", mark)?;
        }

        // Print stacktrace if available.
        if let Some(stack) = self.stack.as_ref() {
            write!(f, "\n{}", stack.dimmed())?;
        }

        Ok(())
    }
}

pub fn unwrap_or_exit<T>(result: Result<T, Error>) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{:?}", e);
            std::process::exit(1);
        }
    }
}
