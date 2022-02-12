use colored::*;
use rusty_v8 as v8;

#[derive(Debug, PartialEq, Clone)]
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
    pub fn from_v8_exception(scope: &mut v8::TryCatch<v8::HandleScope>) -> Self {
        // Getting the error type from the exception.
        let exception = scope.exception().unwrap();
        let exception_string = scope
            .exception()
            .unwrap()
            .to_string(scope)
            .unwrap()
            .to_rust_string_lossy(scope);

        let message = v8::Exception::create_message(scope, exception);

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

        let stack = match scope.stack_trace() {
            Some(value) => value,
            None => v8::undefined(scope).into(),
        };
        let stack = unsafe { v8::Local::<v8::String>::cast(stack) };
        let stack = stack
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope));

        JsError {
            message: exception_string,
            resource_name: resource_name,
            source_line,
            line_number,
            start_column,
            end_column,
            stack,
        }
    }
    // Prints full error with stacktrace.
    pub fn show(&self) {
        // Basic exception information.
        println!("{} {}", "Uncaught".red().bold(), self.message);
        println!("{}", self.source_line.as_ref().unwrap());

        // Indicate the position where the error was thrown.
        let start_column = self.start_column.unwrap_or_default();
        let end_column = self.end_column.unwrap_or_default();

        for _ in 0..start_column {
            print!(" ");
        }

        for _ in start_column..end_column {
            let mark = "^".red();
            print!("{}", mark);
        }

        // Print stacktrace if available.
        if let Some(stack) = self.stack.as_ref() {
            println!("\n{}", stack.dimmed());
        }
    }
}

impl std::error::Error for JsError {}

// Should display the minified version of the error. (used in repl)
impl std::fmt::Display for JsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Unwrapping error values.
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
