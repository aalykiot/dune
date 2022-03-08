use crate::runtime::JsRuntime;
use colored::*;
use rustyline::{error::ReadlineError, Editor};

pub fn start() {
    let mut editor = Editor::<()>::new();
    let mut rt = JsRuntime::new();

    println!("Welcome to Dune v{}", env!("CARGO_PKG_VERSION"));
    let prompt = ">> ".color(Color::Cyan).bold().to_string();

    loop {
        match editor.readline(&prompt) {
            Ok(line) if line == ".exit" => break,
            Ok(line) => match rt.execute_script("<anonymous>", line.trim_end()) {
                Ok(value) => {
                    let scope = &mut rt.handle_scope();
                    let value = value.open(scope);
                    println!("{}", value.to_rust_string_lossy(scope));
                }
                Err(e) => eprintln!("{}", e),
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
