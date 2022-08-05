use anyhow::bail;
use anyhow::Result;
use swc_common::errors::ColorConfig;
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_common::FileName;
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
use swc_ecma_parser::TsConfig;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

/// Transpile TypeScript code into JavaScript.
pub fn transpile(filename: Option<&str>, source: &str) -> Result<String> {
    let globals = Globals::default();
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let filename = match filename {
        Some(filename) => FileName::Custom(filename.into()),
        None => FileName::Anon,
    };

    let fm = cm.new_source_file(filename.clone(), source.into());

    // Initialize the TypeScript lexer.
    let lexer = Lexer::new(
        Syntax::Typescript(TsConfig {
            tsx: filename.to_string().ends_with(".tsx"),
            decorators: true,
            dts: false,
            no_early_errors: true,
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
        Err(_) => bail!("TypeScript compilation failed."),
    };

    // This is where we're gonna store the JavaScript output.
    let mut buffer = vec![];

    GLOBALS.set(&globals, || {
        // Conduct identifier scope analysis
        let module = module.fold_with(&mut resolver(Mark::new(), Mark::new(), true));

        // Remove typescript types
        let module = module.fold_with(&mut strip(Mark::new()));

        // Fix up any identifiers with the same name, but different contexts
        let module = module.fold_with(&mut hygiene());

        // Ensure that we have enough parenthesis.
        let module = module.fold_with(&mut fixer(None));

        {
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config {
                    ..Default::default()
                },
                cm: cm.clone(),
                comments: None,
                wr: JsWriter::new(cm, "\n", &mut buffer, None),
            };

            emitter.emit_module(&module).unwrap();
        }
    });

    Ok(String::from_utf8_lossy(&buffer).to_string())
}
