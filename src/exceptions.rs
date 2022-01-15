use rusty_v8 as v8;

// NOTE: See the following to get full error information.
// https://github.com/denoland/rusty_v8/blob/0d093a02f658781d52e6d70d138768fc19a79d54/examples/shell.rs#L158
pub fn to_pretty_string(mut try_catch: v8::TryCatch<v8::HandleScope>) -> String {
    let exception_string = try_catch
        .exception()
        .unwrap()
        .to_string(&mut try_catch)
        .unwrap()
        .to_rust_string_lossy(&mut try_catch);

    let message = match try_catch.message() {
        Some(message) => message,
        None => {
            return exception_string;
        }
    };

    let filename = message
        .get_script_resource_name(&mut try_catch)
        .map_or_else(
            || "(unknown)".into(),
            |s| {
                s.to_string(&mut try_catch)
                    .unwrap()
                    .to_rust_string_lossy(&mut try_catch)
            },
        );

    let line = message.get_line_number(&mut try_catch).unwrap_or_default();
    let column = message.get_start_column();

    format!("{} ({}:{}:{})", exception_string, filename, line, column)
}
