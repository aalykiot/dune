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
use runtime::JsRuntimeOptions;
use std::env;

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
    #[clap(about = "Bundle everything into a single file (WIP)")]
    Bundle,
    #[clap(about = "Upgrade to the latest dune version (WIP)")]
    Upgrade,
    #[clap(about = "Start the REPL (read, eval, print, loop)")]
    Repl,
}

fn main() {
    // If no arguments specified, start the REPL.
    if env::args().count() == 1 {
        // Start REPL.
        repl::start(JsRuntime::new());
        return;
    }

    // Otherwise, use clap to parse the arguments.
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            script,
            reload,
            seed,
            unstable,
        } => {
            // NOTE: The following code tries to resolve the given filename
            // to an absolute path. If the first time fails we will append `./` to
            // it first, and retry the resolution in case the user forgot to specify it.
            let filename = unwrap_or_exit(
                resolve_import(None, &script)
                    .or_else(|_| resolve_import(None, &format!("./{}", script))),
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
        Commands::Repl => repl::start(JsRuntime::new()),
        Commands::Upgrade | Commands::Bundle => println!("This command is not available :("),
    }
}
