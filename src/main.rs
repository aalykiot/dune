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
mod repl;
mod runtime;
mod stdio;
mod timers;
mod tools;
mod typescript;

use crate::errors::generic_error;
use clap::{Parser, Subcommand};
use errors::unwrap_or_exit;
use modules::resolve_import;
use runtime::JsRuntime;
use runtime::JsRuntimeOptions;
use std::env;
use std::fs;
use std::path::Path;
use tools::bundle;
use tools::compile;
use tools::upgrade;

#[derive(Parser)]
#[clap(
    name = "dune",
    about = "A hobby runtime for JavaScript and TypeScript",
    version
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Clone, Subcommand)]
enum Commands {
    #[clap(about = "Run a JavaScript or TypeScript program")]
    Run {
        #[clap(forbid_empty_values = true, help = "The script that will run")]
        script: String,
        #[clap(short, long, help = "Reload every URL import (cache is ignored)")]
        reload: bool,
        #[clap(long, help = "Make the Math.random() method predictable")]
        seed: Option<i64>,
        #[clap(long, help = "Enable unstable features and APIs")]
        unstable: bool,
    },
    #[clap(about = "Bundle everything into a single file")]
    Bundle {
        #[clap(forbid_empty_values = true, help = "The entry point script")]
        entry: String,
        #[clap(short, long, help = "The filename of the generated bundle")]
        output: Option<String>,
        #[clap(short, long, help = "Reload every URL import (cache is ignored)")]
        reload: bool,
        #[clap(long, help = "Minify the generated bundle")]
        minify: bool,
    },
    #[clap(about = "Compile script to standalone executable")]
    Compile {
        #[clap(forbid_empty_values = true, help = "The entry point script")]
        entry: String,
        #[clap(
            short,
            long,
            help = "The filename of the generated standalone executable"
        )]
        output: Option<String>,
        #[clap(short, long, help = "Reload every URL import (cache is ignored)")]
        reload: bool,
    },
    #[clap(about = "Upgrade to the latest dune version")]
    Upgrade,
    #[clap(about = "Start the REPL (read, eval, print, loop)")]
    Repl,
}

fn run_command(script: String, reload: bool, seed: Option<i64>, unstable: bool) {
    // NOTE: The following code tries to resolve the given filename
    // to an absolute path. If the first time fails we will append `./` to
    // it first, and retry the resolution in case the user forgot to specify it.
    let filename = unwrap_or_exit(
        resolve_import(None, &script).or_else(|_| resolve_import(None, &format!("./{}", script))),
    );

    let options = JsRuntimeOptions {
        seed,
        reload,
        unstable,
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

fn bundle_command(entry: String, output: Option<String>, reload: bool, minify: bool) {
    match bundle::run_bundle(&entry, reload, minify) {
        Ok(source) => output_bundle(source, output),
        Err(e) => eprintln!("{:?}", generic_error(e.to_string())),
    }
}

fn compile_command(entry: String, output: Option<String>, reload: bool) {
    match compile::run_compile(&entry, output, reload) {
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

    let standalone = compile::extract_standalone();

    // Check for errors during standalone extraction.
    if let Err(e) = standalone {
        eprintln!("{:?}", generic_error(e.to_string()));
        std::process::exit(1);
    };

    match standalone.unwrap() {
        Some(source) => run_standalone(source),
        None if env::args().count() == 1 => repl::start(JsRuntime::new()),
        None => {
            // Use clap to parse the arguments.
            let cli = Cli::parse();

            match cli.command {
                Commands::Run {
                    script,
                    reload,
                    seed,
                    unstable,
                } => run_command(script, reload, seed, unstable),
                Commands::Bundle {
                    entry,
                    output,
                    reload,
                    minify,
                } => bundle_command(entry, output, reload, minify),
                Commands::Compile {
                    entry,
                    output,
                    reload,
                } => compile_command(entry, output, reload),
                Commands::Upgrade => upgrade_command(),
                Commands::Repl => repl_command(),
            }
        }
    };
}
