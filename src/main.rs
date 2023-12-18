mod bindings;
mod cli;
mod dns;
mod dotenv;
mod errors;
mod event_loop;
mod file;
mod hooks;
mod http_parser;
mod inspector;
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
mod watcher;

use crate::cli::process_cli_arguments;
use crate::errors::generic_error;
use runtime::JsRuntime;
use std::env;
use tools::bundle;
use tools::compile;
use tools::upgrade;

fn run_standalone(source: String) {
    // Create a new JS runtime.
    let tag = "dune:standalone/main";
    let mut runtime = JsRuntime::new();
    let mod_result = runtime.execute_module(tag, Some(&source));

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{e:?}"),
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

    #[cfg(target_family = "windows")]
    {
        // In shitty platforms like Windows we have to manually enable ANSI colors. ¯\_(ツ)_/¯
        let _ = enable_ansi_support::enable_ansi_support();
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

    process_cli_arguments();
}
