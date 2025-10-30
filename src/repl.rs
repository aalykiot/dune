use crate::runtime::check_exceptions;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
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
use swc_common::Span;
use swc_common::Spanned;
use swc_common::SyntaxContext;
use swc_ecma_ast::*;
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

/// Type of command the Repl thread can send.
enum ReplCommand {
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
    let (sender, receiver) = mpsc::channel::<ReplCommand>();
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
                    sender.send(ReplCommand::Terminate).unwrap();
                    handle.interrupt();
                    break;
                }
                Ok(line) => {
                    // Update REPL's history file.
                    editor.add_history_entry(&line).unwrap();
                    // Evaluate current expression.
                    let message = ReplCommand::Evaluate(line.trim_end().into());
                    sender.send(message).unwrap();
                    handle.interrupt();
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    sender.send(ReplCommand::Terminate).unwrap();
                    handle.interrupt();
                    break;
                }
                Err(e) => {
                    eprintln!("{e}");
                    sender.send(ReplCommand::Terminate).unwrap();
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
        let repl_command = receiver.try_recv();

        // Poll the event-loop.
        if repl_command.is_err() {
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

        let mut input = match repl_command.unwrap() {
            ReplCommand::Terminate => break,
            ReplCommand::Evaluate(prompt) => match ParsedInput::parse(&prompt) {
                Ok(input) => input,
                Err(e) => {
                    eprintln!("{}: {e}", "Parse Error".red().bold());
                    continue;
                }
            },
        };

        println!("DEBUG: {:?}", input.transform());
    }
}

#[derive(Default, Debug)]
pub struct ParsedInput {
    /// Any esm imports in AST format.
    imports: Vec<ImportDecl>,
    /// Rest of the code in AST format.
    statements: Vec<Stmt>,
}

impl ParsedInput {
    /// Parses a given module into parts.
    pub fn parse(source: &str) -> Result<Self> {
        // Initialize the JavaScript lexer.
        let mut this = ParsedInput::default();
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Anon.into(), source.to_string());

        let lexer = Lexer::new(
            Syntax::Es(EsSyntax::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().map_err(|e| {
            let span = e.span();
            let loc = cm.lookup_char_pos(span.lo());
            anyhow!("{} at {}:{}", e.kind().msg(), loc.line, loc.col_display + 1)
        })?;

        this.visit_module(&module);
        Ok(this)
    }

    /// Modifies the AST so that the given prompt runs as an ES module.
    pub fn transform(&mut self) -> String {
        // We need to find all the declerations (classes, fucntions and variables)
        // that we will need to assign to globalThis later on.
        let mut declarations = vec![];

        for specifier in self.imports.iter().flat_map(|import| &import.specifiers) {
            collect_import_declarations(specifier, &mut declarations);
        }

        for statement in self.statements.iter() {
            if let Stmt::Decl(decleration) = statement {
                match decleration {
                    Decl::Class(class) => declarations.push(class.ident.sym.to_string()),
                    Decl::Fn(function) => declarations.push(function.ident.sym.to_string()),
                    Decl::Var(variable) => variable.decls.iter().for_each(|decl| {
                        collect_var_declarations(&decl.name, &mut declarations);
                    }),
                    _ => {}
                }
            }
        }

        // If the last statement if an expression then we should default export it
        // so we can capture and show its value on the REPL output.
        let default_export = match self.statements.last() {
            Some(Stmt::Expr(expression)) => Some(ExportDefaultExpr {
                span: expression.span(),
                expr: expression.expr.clone(),
            }),
            _ => None,
        };

        // In case we default export an expression we should remove the actual
        // node from the statements to avoid duplicate code.
        if default_export.is_some() {
            self.statements.pop();
        }

        // Appened auto generated statements as AST nodes.
        for dec_name in declarations.iter() {
            self.statements.push(assign_to_global_this(dec_name));
        }

        self.emit_module(default_export)
    }

    /// Emits a JavaScript module from the AST statements.
    fn emit_module(&self, mut default_export: Option<ExportDefaultExpr>) -> String {
        // Combine imports and statements into the same AST.
        let mut module_items: Vec<ModuleItem> = self
            .imports
            .iter()
            .cloned()
            .map(|i| ModuleItem::ModuleDecl(ModuleDecl::Import(i)))
            .chain(self.statements.iter().cloned().map(ModuleItem::Stmt))
            .collect();

        if let Some(value) = default_export.take() {
            module_items.push(ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(value)));
        }

        // We have to convert the vec of AST statements that we have
        // into a Script so the emitter can emit the code.
        let module = Module {
            body: module_items,
            span: Default::default(),
            shebang: None,
        };

        // Buffer to collect the emitted JavaScript bytes.
        let mut output = vec![];

        let cm: Lrc<SourceMap> = Default::default();
        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm, "\n", &mut output, None),
        };
        emitter.emit_module(&module).unwrap();

        String::from_utf8_lossy(&output).to_string()
    }
}

impl Visit for ParsedInput {
    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
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

/// Traverses a variable declaration pattern and collects all declared names.
fn collect_var_declarations(pattern: &Pat, locals: &mut Vec<String>) {
    // This function is recursive because JavaScript allows complex
    // and deeply nested assignment patterns.
    match pattern {
        Pat::Ident(binding) => locals.push(binding.sym.to_string()),
        Pat::Rest(rest) => collect_var_declarations(&rest.arg, locals),
        Pat::Assign(assign) => collect_var_declarations(&assign.left, locals),
        Pat::Array(array) => array.elems.iter().flatten().for_each(|pat| {
            collect_var_declarations(pat, locals);
        }),
        Pat::Object(object) => {
            for prop in object.props.iter() {
                match prop {
                    ObjectPatProp::KeyValue(kv) => collect_var_declarations(&kv.value, locals),
                    ObjectPatProp::Rest(rest) => collect_var_declarations(&rest.arg, locals),
                    ObjectPatProp::Assign(assign) => locals.push(assign.key.id.to_string()),
                }
            }
        }
        _ => {}
    };
}

#[inline]
fn collect_import_declarations(specifier: &ImportSpecifier, locals: &mut Vec<String>) {
    locals.push(match specifier {
        ImportSpecifier::Default(value) => value.local.sym.to_string(),
        ImportSpecifier::Namespace(value) => value.local.sym.to_string(),
        ImportSpecifier::Named(value) => value.local.sym.to_string(),
    });
}

// Creates a statement that assigns a variable to globalThis.
fn assign_to_global_this(name: &str) -> Stmt {
    let span: Span = Default::default();
    let property = IdentName::new(name.into(), span);
    let global_this = Expr::Ident(Ident::new(
        "globalThis".into(),
        span,
        SyntaxContext::empty(),
    ));

    // The left-hand side: 'globalThis.<name>'.
    let left_target = AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
        span,
        obj: Box::new(global_this),
        prop: MemberProp::Ident(property),
    }));

    // The right-hand side: '<name>'.
    let right_expr = Expr::Ident(Ident::new(name.into(), span, SyntaxContext::empty()));
    let assign_expr = Expr::Assign(AssignExpr {
        span,
        op: op!("="),
        left: left_target,
        right: Box::new(right_expr),
    });

    Stmt::Expr(ExprStmt {
        span,
        expr: Box::new(assign_expr),
    })
}
