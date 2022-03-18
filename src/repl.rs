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

    fn highlight_char(&self, line: &str, _: usize) -> bool {
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
                    // Check if capture is a JavaScript keyword.
                    if KEYWORDS.contains(cap.as_str()) {
                        return cap.as_str().color(KEYWORD_COLOR).bold().to_string();
                    }
                    // Special token colorization.
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

    // Spawn the REPL thread.
    thread::spawn(move || {
        let mut editor = Editor::new();
        let history_file_path = &dirs::home_dir().unwrap().join(CLI_ROOT).join(CLI_HISTORY);

        editor.set_helper(Some(RLHelper::new()));
        editor.load_history(history_file_path).unwrap_or_default();

        println!("Welcome to Dune v{}", env!("CARGO_PKG_VERSION"));
        let prompt = "> ".to_string();

        loop {
            match editor.readline(&prompt) {
                Ok(line) if line == ".exit" => {
                    sender.send(ReplMessage::Terminate).unwrap();
                    break;
                }
                Ok(line) => {
                    // Update REPL's history file.
                    editor.add_history_entry(&line);
                    // Evaluate current expression.
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
        // Saving REPL's history before exiting.
        fs::create_dir_all(history_file_path.parent().unwrap()).unwrap();
        editor.save_history(history_file_path).unwrap()
    });

    loop {
        // Check if the REPL sent any new messages.
        let maybe_message = receiver.try_recv();
        // If not, poll the event-loop again.
        if maybe_message.is_err() {
            runtime.poll_event_loop();
            continue;
        }
        // If it did, try execute the given expression, or exit the process.
        match maybe_message.unwrap() {
            ReplMessage::Evaluate(expression) => {
                match runtime.execute_script("<anonymous>", &expression) {
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
