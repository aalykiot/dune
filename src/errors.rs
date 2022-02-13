use anyhow::Error;
use colored::*;
use rusty_v8 as v8;
use std::borrow::Cow;
use std::fmt::Debug;
use std::fmt::Display;

// A simple error type that lets the creator specify both the error message and
// the error class name.
#[derive(Debug)]
pub struct CustomError {
    class: &'static str,
    message: Cow<'static, str>,
}

impl CustomError {
    pub fn generic(message: impl Into<Cow<'static, str>>) -> Error {
        CustomError {
            class: "Error",
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

// Represents an exception coming from V8.
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
        // Create a new HandleScope so we can create local handles.
        let scope = &mut v8::HandleScope::new(scope);
        let message = v8::Exception::create_message(scope, exception);

        // Getting the error type from the exception.
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

        // Access error.stack to ensure that prepareStackTrace() has been called.
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

// Should display the minified version of the error. (used in repl)
impl Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Unwrapping values.
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

// Should display the full version of the error with stacktrace.
impl Debug for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Basic exception information.
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
