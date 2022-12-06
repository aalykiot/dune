mod bindings;
mod dns;
mod errors;
mod event_loop;
mod file;
mod hooks;
mod loaders;
mod modules;
mod net;
mod perf_hooks;
mod process;
mod promise;
mod repl;
mod runtime;
mod stdio;
mod timers;
mod tools;
mod transpilers;

use crate::errors::generic_error;
use clap::arg;
use clap::ArgMatches;
use clap::Command;
use errors::unwrap_or_exit;
use modules::resolve_import;
use modules::ImportMap;
use runtime::JsRuntime;
use runtime::JsRuntimeOptions;
use std::env;
use std::fs;
use std::path::Path;
use tools::bundle;
use tools::compile;
use tools::upgrade;

fn load_import_map(filename: Option<String>) -> Option<ImportMap> {
    let filename = filename.unwrap_or_else(|| "import-map.json".into());
    match fs::read_to_string(filename) {
        Ok(contents) => Some(unwrap_or_exit(ImportMap::parse_from_json(&contents))),
        Err(_) => None,
    }
}

fn run_command(mut args: ArgMatches) {
    // Extract options from cli arguments.
    let script = args.remove_one::<String>("SCRIPT").unwrap();
    let reload = args.remove_one::<bool>("reload").unwrap_or_default();
    let import_map = args.remove_one::<String>("import-map");
    let seed = args
        .remove_one::<String>("seed")
        .map(|val| val.parse::<i64>().unwrap_or_default());

    let import_map = load_import_map(import_map);

    // NOTE: The following code tries to resolve the given filename
    // to an absolute path. If the first time fails we will append `./` to
    // it first, and retry the resolution in case the user forgot to specify it.
    let filename = unwrap_or_exit(
        resolve_import(None, &script, import_map.clone())
            .or_else(|_| resolve_import(None, &format!("./{}", script), import_map.clone())),
    );

    let options = JsRuntimeOptions {
        seed,
        reload,
        import_map,
    };

    // Create new JS runtime.
    let mut runtime = JsRuntime::with_options(options);
    let mod_result = runtime.execute_module(&filename, None);

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{:?}", e),
    };
}

fn repl_command() {
    // Start REPL.
    repl::start(JsRuntime::new());
}

fn upgrade_command() {
    match upgrade::run_upgrade() {
        Ok(_) => println!("Upgraded successfully"),
        Err(e) => eprintln!("{}", generic_error(e.to_string())),
    }
}

fn output_bundle(source: String, output: Option<String>) {
    // If output is specified write source there, otherwise print it to screen.
    match output {
        Some(output) => {
            // Make sure output has a .js extension.
            let path = Path::new(&output).with_extension("js");
            // Write source to output.
            match fs::create_dir_all(path.parent().unwrap()).map(|_| fs::write(path, source)) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", generic_error(e.to_string())),
            };
        }
        None => println!("{}", source),
    };
}

fn bundle_command(mut args: ArgMatches) {
    // Extract options from cli arguments.
    let entry = args.remove_one::<String>("ENTRY").unwrap();
    let output = args.remove_one::<String>("output");
    let skip_cache = args.remove_one::<bool>("reload").unwrap_or_default();
    let minify = args.remove_one::<bool>("minify").unwrap_or_default();

    let import_map = args.remove_one::<String>("import-map");
    let import_map = load_import_map(import_map);

    let options = bundle::Options {
        skip_cache,
        minify,
        import_map,
    };

    match bundle::run_bundle(&entry, &options) {
        Ok(source) => output_bundle(source, output),
        Err(e) => eprintln!("{:?}", generic_error(e.to_string())),
    }
}

fn compile_command(mut args: ArgMatches) {
    // Extract options from cli arguments.
    let entry = args.remove_one::<String>("ENTRY").unwrap();
    let output = args.remove_one::<String>("output");
    let skip_cache = args.remove_one::<bool>("reload").unwrap_or_default();

    let import_map = args.remove_one::<String>("import-map");
    let import_map = load_import_map(import_map);

    let options = compile::Options {
        skip_cache,
        minify: true,
        import_map,
    };

    match compile::run_compile(&entry, output, &options) {
        Ok(_) => {}
        Err(e) => eprintln!("{:?}", generic_error(e.to_string())),
    }
}

fn run_standalone(source: String) {
    // Create a new JS runtime.
    let mut runtime = JsRuntime::new();
    let mod_result = runtime.execute_module("dune:standalone/main", Some(&source));

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{:?}", e),
    };
    std::process::exit(0);
}

/// Custom hook on panics (copied from Deno).
fn setup_panic_hook() {
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        eprintln!("\n============================================================");
        eprintln!("Dune has panicked. This is a bug in Dune. Please report this");
        eprintln!("at https://github.com/aalykiot/dune/issues");
        eprintln!("If you can reliably reproduce this panic, include the");
        eprintln!("reproduction steps and re-run with the RUST_BACKTRACE=1 env");
        eprintln!("var set and include the backtrace in your report.");
        eprintln!();
        eprintln!("Platform: {} {}", env::consts::OS, env::consts::ARCH);
        eprintln!("Version: {}", env!("CARGO_PKG_VERSION"));
        eprintln!("Args: {:?}", env::args().collect::<Vec<_>>());
        eprintln!();
        orig_hook(panic_info);
        std::process::exit(1);
    }));
}

fn main() {
    // Set custom panic hook on release builds.
    if !cfg!(debug_assertions) {
        setup_panic_hook();
    }

    // Try run dune as a compiled standalone program.
    match compile::extract_standalone() {
        Ok(Some(source)) => run_standalone(source),
        Err(e) => {
            eprintln!("{:?}", generic_error(e.to_string()));
            std::process::exit(1);
        }
        _ => {}
    };

    let mut cli = Command::new("dune")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A hobby runtime for JavaScript and TypeScript")
        .subcommand(
            Command::new("run")
                .about("Run a JavaScript or TypeScript program")
                .arg_required_else_help(true)
                .arg(arg!(<SCRIPT> "The script that will run").required(true))
                .arg(arg!(-r --reload "Reload every URL import (cache is ignored)"))
                .arg(arg!(--seed <NUMBER> "Make the Math.random() method predictable"))
                .arg(arg!(--"import-map" <FILE> "Load import map file from local file")),
        )
        .subcommand(
            Command::new("bundle")
                .about("Bundle everything into a single file")
                .arg_required_else_help(true)
                .arg(arg!(<ENTRY> "The entry point script").required(true))
                .arg(arg!(-o --output <FILE> "The filename of the generated bundle"))
                .arg(arg!(-r --reload "Reload every URL import (cache is ignored)"))
                .arg(arg!(--minify "Minify the generated bundle"))
                .arg(arg!(--"import-map" <FILE> "Load import map file from local file")),
        )
        .subcommand(
            Command::new("compile")
                .about("Compile script to standalone executable")
                .arg_required_else_help(true)
                .arg(arg!(<ENTRY> "The entry point script").required(true))
                .arg(arg!(-o --output <FILE> "The filename of the generated standalone executable"))
                .arg(arg!(-r --reload "Reload every URL import (cache is ignored)"))
                .arg(arg!(--"import-map" <FILE> "Load import map file from local file")),
        )
        .subcommand(Command::new("upgrade").about("Upgrade to the latest dune version"))
        .subcommand(Command::new("repl").about("Start the REPL (read, eval, print, loop)"))
        .get_matches();

    let (cmd, args) = cli.remove_subcommand().unwrap_or_default();

    match (cmd.as_str(), args) {
        ("run", args) => run_command(args),
        ("bundle", args) => bundle_command(args),
        ("compile", args) => compile_command(args),
        ("upgrade", _) => upgrade_command(),
        _ => repl_command(),
    }
}
