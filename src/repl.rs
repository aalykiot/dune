use crate::runtime::JsRuntime;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;

// Type of messages the Repl thread can send.
enum ReplMessage {
    // Evaluate a given JavaScript expression.
    Evaluate(String),
    // Terminate main process.
    Terminate,
}

/// Starts the REPL thread.
pub fn start(mut runtime: JsRuntime) {
    // Create a channel for thread communication.
    let (sender, receiver) = mpsc::channel::<ReplMessage>();

    // Spawn the REPL thread.
    thread::spawn(move || {
        let mut editor = Editor::<()>::new();

        println!("Welcome to Dune v{}", env!("CARGO_PKG_VERSION"));
        let prompt = ">> ".color(Color::Cyan).bold().to_string();

        loop {
            match editor.readline(&prompt) {
                Ok(line) if line == ".exit" => {
                    sender.send(ReplMessage::Terminate).unwrap();
                    break;
                }
                Ok(line) => {
                    let message = ReplMessage::Evaluate(line.trim_end().into());
                    sender.send(message).unwrap();
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    sender.send(ReplMessage::Terminate).unwrap();
                    break;
                }
                Err(e) => {
                    eprintln!("{}", e);
                    sender.send(ReplMessage::Terminate).unwrap();
                    break;
                }
            }
        }
    });

    loop {
        // Check if the REPL sent any new messages.
        let maybe_message = receiver.try_recv();

        // If not, poll the event-loop one more time.
        if let Err(e) = maybe_message {
            match e {
                TryRecvError::Empty => {
                    runtime.poll_event_loop();
                    continue;
                }
                TryRecvError::Disconnected => panic!("{}", e),
            }
        }

        // If it did, try execute the provided input.
        match maybe_message.unwrap() {
            ReplMessage::Evaluate(line) => {
                match runtime.execute_script("<anonymous>", &line) {
                    Ok(value) => {
                        let scope = &mut runtime.handle_scope();
                        let value = value.open(scope);
                        println!("{}", value.to_rust_string_lossy(scope));
                    }
                    Err(e) => eprintln!("{}", e),
                };
            }
            ReplMessage::Terminate => break,
        }
    }
}
