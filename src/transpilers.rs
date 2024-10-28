use anyhow::bail;
use anyhow::Result;
use base64::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use swc_common::comments::SingleThreadedComments;
use swc_common::errors::ColorConfig;
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_common::FileName;
use swc_common::FilePathMapping;
use swc_common::Globals;
use swc_common::Mark;
use swc_common::SourceMap;
use swc_common::GLOBALS;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::Parser;
use swc_ecma_parser::StringInput;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::TsSyntax;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_react::react;
use swc_ecma_transforms_react::Options;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

lazy_static! {
    static ref PRAGMA_REGEX: Regex = Regex::new(r"@jsx\s+([^\s]+)").unwrap();
}

pub struct TypeScript;

impl TypeScript {
    /// Compiles TypeScript code into JavaScript.
    pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
        let globals = Globals::default();
        let cm: Lrc<SourceMap> = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let filename = match filename {
            Some(filename) => FileName::Custom(filename.into()),
            None => FileName::Anon,
        };

        let fm = cm.new_source_file(filename.into(), source.into());

        // Initialize the TypeScript lexer.
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                no_early_errors: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        let program = match parser
            .parse_program()
            .map_err(|e| e.into_diagnostic(&handler).emit())
        {
            Ok(module) => module,
            Err(_) => bail!("TypeScript compilation failed."),
        };

        // This is where we're gonna store the JavaScript output.
        let mut output = vec![];
        let mut source_map = vec![];

        GLOBALS.set(&globals, || {
            // Apply the rest SWC transforms to generated code.
            let program = program
                .fold_with(&mut resolver(Mark::new(), Mark::new(), true))
                .fold_with(&mut strip(Mark::new(), Mark::new()))
                .fold_with(&mut hygiene())
                .fold_with(&mut fixer(None));

            {
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default(),
                    cm: cm.clone(),
                    comments: None,
                    wr: JsWriter::new(cm.clone(), "\n", &mut output, Some(&mut source_map)),
                };

                emitter.emit_program(&program).unwrap();
            }
        });

        // Turn source-map struct into a string representation.
        let mut source_map_buf = Vec::new();
        let source_map = cm.build_source_map(&source_map);
        source_map.to_writer(&mut source_map_buf).unwrap();

        // Prepare the inline source map comment.
        let source_map = String::from_utf8_lossy(&source_map_buf).to_string();
        let source_map = BASE64_STANDARD.encode(source_map.as_bytes());
        let source_map = format!(
            "//# sourceMappingURL=data:application/json;base64,{}",
            source_map
        );

        let code = String::from_utf8_lossy(&output).to_string();
        let output = format!("{}\n{}", code, source_map);

        Ok(output)
    }
}

pub struct Jsx;

impl Jsx {
    /// Compiles JSX code into JavaScript.
    pub fn compile(filename: Option<&str>, source: &str) -> Result<String> {
        let globals = Globals::default();
        let cm: Lrc<SourceMap> = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let filename = match filename {
            Some(filename) => FileName::Custom(filename.into()),
            None => FileName::Anon,
        };

        let fm = cm.new_source_file(filename.into(), source.into());

        // NOTE: We're using a TypeScript lexer to parse JSX because it's a super-set
        // of JavaScript and we also want to support .tsx files.

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: true,
                decorators: true,
                no_early_errors: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        let module = match parser
            .parse_module()
            .map_err(|e| e.into_diagnostic(&handler).emit())
        {
            Ok(module) => module,
            Err(_) => bail!("JSX compilation failed."),
        };

        // This is where we're gonna store the JavaScript output.
        let mut output = vec![];
        let mut source_map = vec![];

        // Look for the JSX pragma in the source code.
        // https://www.gatsbyjs.com/blog/2019-08-02-what-is-jsx-pragma/

        let pragma = PRAGMA_REGEX
            .find_iter(source)
            .next()
            .map(|m| m.as_str().to_string().replace("@jsx ", ""));

        GLOBALS.set(&globals, || {
            // Apply SWC transforms to given code.
            let module = module.fold_with(&mut react::<SingleThreadedComments>(
                cm.clone(),
                None,
                Options {
                    pragma,
                    ..Default::default()
                },
                Mark::new(),
                Mark::new(),
            ));

            {
                let mut emitter = Emitter {
                    cfg: swc_ecma_codegen::Config::default(),
                    cm: cm.clone(),
                    comments: None,
                    wr: JsWriter::new(cm.clone(), "\n", &mut output, Some(&mut source_map)),
                };

                emitter.emit_module(&module).unwrap();
            }
        });

        // Turn source-map struct into a string representation.
        let mut source_map_buf = Vec::new();
        let source_map = cm.build_source_map(&source_map);
        source_map.to_writer(&mut source_map_buf).unwrap();

        // Prepare the inline source map comment.
        let source_map = String::from_utf8_lossy(&source_map_buf).to_string();
        let source_map = BASE64_STANDARD.encode(source_map.as_bytes());
        let source_map = format!(
            "//# sourceMappingURL=data:application/json;base64,{}",
            source_map
        );

        let code = String::from_utf8_lossy(&output).to_string();
        let output = format!("{}\n{}", code, source_map);

        Ok(output)
    }
}

pub struct Wasm;

impl Wasm {
    // Converts a wasm binary into an ES module template.
    pub fn parse(source: &str) -> String {
        format!(
            "
        const wasmCode = new Uint8Array({:?});
        const wasmModule = new WebAssembly.Module(wasmCode);
        const wasmInstance = new WebAssembly.Instance(wasmModule);
        export default wasmInstance.exports;
        ",
            source.as_bytes()
        )
    }
}
