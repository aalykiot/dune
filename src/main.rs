mod bindings;
mod exceptions;
mod process;
mod runtime;

use colored::*;
use runtime::JsRuntime;
use rustyline::{error::ReadlineError, Editor};

fn main() {
    let mut editor = Editor::<()>::new();
    let mut rt = JsRuntime::new();

    println!("Welcome to Quixel v{}", env!("CARGO_PKG_VERSION"));

    let prompt = ">> ".color(Color::Cyan).bold().to_string();

    loop {
        match editor.readline(&prompt) {
            Ok(line) if line == ".exit" => break,
            Ok(line) => match rt.execute("<anonymous>", line.trim_end()) {
                Ok(v) => println!("{}", v),
                Err(v) => {
                    eprintln!("{}: {}", "Uncaught".red().bold(), v);
                }
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("{}: {:?}", "Unknown".red().bold(), err);
                break;
            }
        }
    }
}
