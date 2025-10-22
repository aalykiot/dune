use crate::runtime::check_exceptions;
use crate::runtime::JsRuntime;
use anyhow::bail;
use anyhow::Result;
use colored::*;
use phf::phf_set;
use phf::Set;
use regex::Captures;
use regex::Regex;
use rustyline::error::ReadlineError;
use rustyline::highlight::CmdKind;
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
use swc_common::sync::Lrc;
use swc_common::FileName;
use swc_common::SourceMap;
use swc_ecma_ast::ImportDecl;
use swc_ecma_ast::ImportSpecifier;
use swc_ecma_ast::Module;
use swc_ecma_ast::ModuleDecl;
use swc_ecma_ast::ModuleItem;
use swc_ecma_ast::Script;
use swc_ecma_ast::Stmt;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::EsSyntax;
use swc_ecma_parser::Parser;
use swc_ecma_parser::StringInput;
use swc_ecma_parser::Syntax;
use swc_ecma_visit::Visit;

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

    fn highlight_char(&self, line: &str, _: usize, _: CmdKind) -> bool {
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
        let prompt = "> ";

        editor.set_helper(Some(RLHelper::new()));
        editor.load_history(history_file_path).unwrap_or_default();

        println!("Welcome to Dune v{}", env!("CARGO_PKG_VERSION"));
        println!("exit using ctrl+d, ctrl+c or .close");

        // Note: In order to wake-up the event-loop (so the main thread can evaluate the JS expression) in
        // case it's stack in the poll phase waiting for new I/O will call the `handle.interrupt()`
        // method that sends a wake-up signal across the main thread.

        loop {
            match editor.readline(prompt) {
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

    loop {
        // Check for REPL messages.
        let maybe_message = receiver.try_recv();

        // Poll the event-loop.
        if maybe_message.is_err() {
            // Tick the event loop and report exceptions.
            runtime.tick_event_loop();
            // Check for exceptions.
            runtime.with_scope(|scope| {
                if let Some(error) = check_exceptions(scope) {
                    eprintln!("{error}");
                }
            });

            continue;
        }

        // Try execute the given expression, or exit the process.
        match maybe_message.unwrap() {
            ReplMessage::Evaluate(expression) => match EsModuleParts::parse(&expression) {
                Ok(module) => {
                    println!("DEBUG(imports): {}", module.imports());
                    println!("DEBUG(script): {}", module.script());
                }
                Err(e) => eprintln!("{e}"),
            },
            ReplMessage::Terminate => break,
        }
    }
}

#[derive(Default)]
pub struct EsModuleParts {
    /// All the imports in AST format.
    imports: Vec<ImportDecl>,
    /// The rest of the code in AST format.
    statements: Vec<Stmt>,
    /// SWC internal source-map.
    cm: Lrc<SourceMap>,
}

impl EsModuleParts {
    /// Parsed a source string into a module.
    pub fn parse(source: &str) -> Result<Self> {
        // Initialize the JavaScript lexer.
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());
        let lexer = Lexer::new(
            Syntax::Es(EsSyntax::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = match parser.parse_module().map_err(|e| e.into_kind().msg()) {
            Ok(module) => module,
            Err(e) => bail!(e),
        };

        // Visit nodes and extract imports and statements.
        let mut this = Self {
            cm,
            ..Default::default()
        };
        this.visit_module(&module);

        Ok(this)
    }

    /// Returns only the module imports as source code.
    pub fn imports(&self) -> String {
        self.imports
            .iter()
            .map(|import_ast| {
                // Find the import's source (meaning the "from" part).
                let source = import_ast.src.raw.clone().unwrap().to_string();

                // No specifiers means an import like "import x".
                if import_ast.specifiers.is_empty() {
                    return format!("import {};", source);
                }

                // Collect formatted import statements.
                let mut named = vec![];
                for specifier in &import_ast.specifiers {
                    match specifier {
                        ImportSpecifier::Default(s) => {
                            let specifier = s.local.sym.to_string();
                            return format!("import {} from {};", specifier, source);
                        }
                        ImportSpecifier::Namespace(s) => {
                            let specifier = s.local.sym.to_string();
                            return format!("import * as {} from {};", specifier, source);
                        }
                        ImportSpecifier::Named(s) => {
                            let specifier = s.local.sym.to_string();
                            named.push(specifier);
                        }
                    };
                }

                // Reaching this point means we have a named import (import { x, y } from z)
                // so, we will collect all the names and build the import.
                format!("import {{ {} }} from {};", named.join(", "), source)
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Returns the rest of the JS expressions.
    pub fn script(&self) -> String {
        // We have to convert the vec of AST statements that we have
        // into a Script so the emitter can emit the JS code.
        let script = Script {
            body: self.statements.clone(),
            ..Default::default()
        };

        // This is where we're gonna store the JavaScript output.
        let mut output = vec![];
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: self.cm.clone(),
            comments: None,
            wr: JsWriter::new(self.cm.clone(), "\n", &mut output, None),
        };

        emitter.emit_script(&script).unwrap();

        String::from_utf8_lossy(&output).to_string()
    }
}

impl Visit for EsModuleParts {
    fn visit_module(&mut self, node: &Module) {
        for item in &node.body {
            match item {
                // Parse ES module imports.
                ModuleItem::ModuleDecl(ModuleDecl::Import(value)) => {
                    self.imports.push(value.clone());
                }
                // Rest of expressions.
                ModuleItem::Stmt(statement) => {
                    self.statements.push(statement.clone());
                }
                _ => {}
            }
        }
    }
}
