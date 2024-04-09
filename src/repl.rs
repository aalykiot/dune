use crate::runtime::JsRuntime;
use colored::*;
use phf::phf_set;
use phf::Set;
use regex::Captures;
use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::validate::ValidationContext;
use rustyline::validate::ValidationResult;
use rustyline::validate::Validator;
use rustyline::Editor;
use rustyline_derive::Completer;
use rustyline_derive::Helper;
use rustyline_derive::Hinter;
use std::borrow::Cow;
use std::fs;
use std::sync::mpsc;
use std::thread;

const STRING_COLOR: Color = Color::Green;
const NUMBER_COLOR: Color = Color::Yellow;
const KEYWORD_COLOR: Color = Color::Cyan;

const UNDEFINED_COLOR: Color = Color::TrueColor {
    r: 100,
    g: 100,
    b: 100,
};

static KEYWORDS: Set<&'static str> = phf_set! {
    "await",
    "const",
    "do",
    "let",
    "typeof",
    "yield",
    "break",
    "continue",
    "else",
    "finally",
    "import",
    "new",
    "this",
    "var",
    "case",
    "debugger",
    "for",
    "in",
    "return",
    "throw",
    "void",
    "catch",
    "default",
    "export",
    "function",
    "instanceof",
    "super",
    "try",
    "while",
    "class",
    "delete",
    "extends",
    "if",
    "switch",
    "with",
};

#[derive(Completer, Helper, Hinter)]
pub(crate) struct RLHelper {
    highlighter: LineHighlighter,
    validator: MatchingBracketValidator,
}

impl RLHelper {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            highlighter: LineHighlighter,
            validator: MatchingBracketValidator::new(),
        }
    }
}

impl Validator for RLHelper {
    fn validate(
        &self,
        context: &mut ValidationContext<'_>,
    ) -> Result<ValidationResult, ReadlineError> {
        self.validator.validate(context)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Highlighter for RLHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        hint.into()
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        self.highlighter.highlight(candidate, 0)
    }

    fn highlight_char(&self, line: &str, _: usize, _: bool) -> bool {
        !line.is_empty()
    }
}

struct LineHighlighter;

impl Highlighter for LineHighlighter {
    fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
        let mut line = line.to_string();
        let regex = Regex::new(
            r#"(?x)
            (?P<identifier>\b[$_\p{ID_Start}][$_\p{ID_Continue}\u{200C}\u{200D}]*\b) |
            (?P<string_double_quote>"([^"\\]|\\.)*") |
            (?P<string_single_quote>'([^'\\]|\\.)*') |
            (?P<template_literal>`([^`\\]|\\.)*`) |
            (?P<op>[+\-/*%~^!&|=<>;:]) |
            (?P<number>0[bB][01](_?[01])*n?|0[oO][0-7](_?[0-7])*n?|0[xX][0-9a-fA-F](_?[0-9a-fA-F])*n?|(([0-9](_?[0-9])*\.([0-9](_?[0-9])*)?)|(([0-9](_?[0-9])*)?\.[0-9](_?[0-9])*)|([0-9](_?[0-9])*))([eE][+-]?[0-9](_?[0-9])*)?n?)"#,
        ).unwrap();

        line = regex
            .replace_all(&line, |caps: &Captures<'_>| {
                // Colorize JavaScript built in primitives.
                if let Some(cap) = caps.name("identifier") {
                    // Check capture against the keywords list.
                    if KEYWORDS.contains(cap.as_str()) {
                        return cap.as_str().color(KEYWORD_COLOR).bold().to_string();
                    }

                    // Colorize special tokens.
                    return match cap.as_str() {
                        "true" | "false" | "null" | "Infinity" => {
                            cap.as_str().color(Color::Yellow).to_string()
                        }
                        "undefined" => cap.as_str().color(UNDEFINED_COLOR).to_string(),
                        _ => cap.as_str().to_string(),
                    };
                }

                // Colorize single quoted strings.
                if let Some(cap) = caps.name("string_single_quote") {
                    return cap.as_str().color(STRING_COLOR).to_string();
                }

                // Colorize double quoted strings.
                if let Some(cap) = caps.name("string_double_quote") {
                    return cap.as_str().color(STRING_COLOR).to_string();
                }

                // Colorize template literals.
                if let Some(cap) = caps.name("template_literal") {
                    return cap.as_str().color(STRING_COLOR).to_string();
                }

                // Colorize numbers.
                if let Some(cap) = caps.name("number") {
                    return cap.as_str().color(NUMBER_COLOR).to_string();
                }

                // Default.
                caps[0].to_string()
            })
            .to_string();

        line.into()
    }
}

/// Type of messages the Repl thread can send.
enum ReplMessage {
    // Evaluate a given JavaScript expression.
    Evaluate(String),
    // Terminate main process.
    Terminate,
}

/// CLI configuration for REPL.
static CLI_ROOT: &str = ".dune";
static CLI_HISTORY: &str = ".dune_history";

/// Starts the REPL server.
pub fn start(mut runtime: JsRuntime) {
    // Create a channel for thread communication.
    let (sender, receiver) = mpsc::channel::<ReplMessage>();
    let handle = runtime.event_loop.interrupt_handle();

    // Note: To prevent a busy loop, we schedule an empty repeatable
    // timer with a close to maximum timeout value.
    //
    // https://doc.rust-lang.org/std/time/struct.Instant.html#os-specific-behaviors

    runtime
        .event_loop
        .handle()
        .timer(u32::MAX as u64, true, |_| {});

    // Spawn the REPL thread.
    thread::spawn(move || {
        let mut editor = Editor::new().unwrap();
        let history_file_path = &dirs::home_dir().unwrap().join(CLI_ROOT).join(CLI_HISTORY);

        editor.set_helper(Some(RLHelper::new()));
        editor.load_history(history_file_path).unwrap_or_default();

        println!("Welcome to Dune v{}", env!("CARGO_PKG_VERSION"));
        let prompt = "> ".to_string();

        // Note: In order to wake-up the event-loop (so the main thread can evaluate the JS expression) in
        // case it's stack in the poll phase waiting for new I/O will call the `handle.interrupt()`
        // method that sends a wake-up signal across the main thread.

        loop {
            match editor.readline(&prompt) {
                Ok(line) if line == ".exit" => {
                    sender.send(ReplMessage::Terminate).unwrap();
                    handle.interrupt();
                    break;
                }
                Ok(line) => {
                    // Update REPL's history file.
                    editor.add_history_entry(&line).unwrap();
                    // Evaluate current expression.
                    let message = ReplMessage::Evaluate(line.trim_end().into());
                    sender.send(message).unwrap();
                    handle.interrupt();
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    sender.send(ReplMessage::Terminate).unwrap();
                    handle.interrupt();
                    break;
                }
                Err(e) => {
                    eprintln!("{e}");
                    sender.send(ReplMessage::Terminate).unwrap();
                    handle.interrupt();
                    break;
                }
            }
        }
        // Save REPL's history.
        fs::create_dir_all(history_file_path.parent().unwrap()).unwrap();
        editor.save_history(history_file_path).unwrap()
    });

    let context = runtime.context();

    loop {
        // Check for REPL messages.
        let maybe_message = receiver.try_recv();

        // Poll the event-loop.
        if maybe_message.is_err() {
            // Tick the event loop and report exceptions.
            runtime.tick_event_loop();
            runtime.report_exceptions();
            continue;
        }

        // Try execute the given expression, or exit the process.
        match maybe_message.unwrap() {
            ReplMessage::Evaluate(expression) => {
                match runtime.execute_script("<anonymous>", &expression) {
                    // Format the expression using console.log.
                    Ok(value) => {
                        let scope = &mut runtime.handle_scope();
                        let context = v8::Local::new(scope, context.clone());
                        let scope = &mut v8::ContextScope::new(scope, context);
                        let global = context.global(scope);
                        let console_name = v8::String::new(scope, "console").unwrap();
                        let console = global.get(scope, console_name.into()).unwrap();
                        let console = v8::Local::<v8::Object>::try_from(console).unwrap();
                        let log_name = v8::String::new(scope, "log").unwrap();
                        let log = console.get(scope, log_name.into()).unwrap();
                        let log = v8::Local::<v8::Function>::try_from(log).unwrap();
                        let value = v8::Local::new(scope, value);
                        log.call(scope, global.into(), &[value]);
                    }
                    Err(e) => eprintln!("{e}"),
                };
            }
            ReplMessage::Terminate => break,
        }
    }
}
