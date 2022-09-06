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
mod typescript;

use clap::{Parser, Subcommand};
use errors::unwrap_or_exit;
use modules::resolve_import;
use runtime::JsRuntime;
use std::env;

#[derive(Parser)]
#[clap(
    name = "dune",
    about = "A hobby runtime for JavaScript and TypeScript",
    version,
    author
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Run a JavaScript or TypeScript program")]
    Run {
        #[clap(forbid_empty_values = true, help = "The script that will run")]
        script: String,
        #[clap(short, long, help = "Reload every URL import (cache is ignored)")]
        reload: bool,
        #[clap(long, help = "Suppress messages when downloading dependencies")]
        quite: bool,
        #[clap(long, help = "Make the Math.random() method predictable")]
        seed: Option<f64>,
        #[clap(long, help = "Enable unstable features and APIs")]
        unstable: bool,
    },
    #[clap(about = "Upgrade dune to the latest version")]
    Upgrade,
    #[clap(about = "Start the REPL (read, eval, print, loop)")]
    Repl,
}

/// Prints a message to console and exits with non-zero code.
fn report_and_exit(message: &str) {
    eprintln!("{}", message);
    std::process::exit(1);
}

fn main() {
    // Parse command line arguments.
    let args: Vec<String> = env::args().collect();

    // If no arguments specified, start the REPL.
    if args.is_empty() {
        // Start REPL.
        repl::start(JsRuntime::new());
        return;
    }

    // Otherwise, use clap to parse the arguments.
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { script, .. } => {
            // NOTE: The following code tries to resolve the given filename
            // to an absolute path. If the first time fails we will append `./` to
            // it first, and retry the resolution in case the user forgot to specify it.
            let filename = unwrap_or_exit(
                resolve_import(None, script)
                    .or_else(|_| resolve_import(None, &format!("./{}", script))),
            );

            // Create new JS runtime.
            let mut runtime = JsRuntime::new();
            let mod_result = runtime.execute_module(&filename, None);

            match mod_result {
                Ok(_) => runtime.run_event_loop(),
                Err(e) => report_and_exit(&format!("{:?}", e)),
            };
        }
        Commands::Repl => repl::start(JsRuntime::new()),
        Commands::Upgrade => report_and_exit("The `upgrade` command it's not implemented :("),
    }
}
