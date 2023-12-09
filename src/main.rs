mod bindings;
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

use crate::errors::generic_error;
use clap::arg;
use clap::ArgMatches;
use clap::Command;
use colored::*;
use errors::unwrap_or_exit;
use modules::resolve_import;
use modules::ImportMap;
use path_absolutize::*;
use runtime::JsRuntime;
use runtime::JsRuntimeOptions;
use std::env;
use std::fs;
use std::net::SocketAddrV4;
use std::path::Path;
use std::path::PathBuf;
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

    let num_threads = args
        .remove_one::<String>("threadpool-size")
        .map(|val| val.parse::<usize>().unwrap_or_default());

    let import_map = load_import_map(import_map);

    // Load custom .env file if specified.
    if let Some(path) = args.remove_one::<String>("env-file") {
        // Try to parse the .env file.
        if let Err(e) = dotenv::load_env_file(path) {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }

    // NOTE: The following code tries to resolve the given filename
    // to an absolute path. If the first time fails we will append `./` to
    // it first, and retry the resolution in case the user forgot to specify it.
    let filename = unwrap_or_exit(
        resolve_import(None, &script, true, import_map.clone())
            .or_else(|_| resolve_import(None, &format!("./{script}"), true, import_map.clone())),
    );

    // Check if we need to enable the inspector.
    let inspect = args.remove_one::<String>("inspect").map(|address| {
        // Parse to IPv4 address.
        match address.parse::<SocketAddrV4>() {
            Ok(address) => (address, false),
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        }
    });
    // Check if we need to enable the inspector.
    let inspect_brk = args.remove_one::<String>("inspect-brk").map(|address| {
        // Parse to IPv4 address.
        match address.parse::<SocketAddrV4>() {
            Ok(address) => (address, true),
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        }
    });

    let inspect = inspect.or(inspect_brk);

    // Check if we have to run on `watch` mode.
    if args.contains_id("watch") {
        match watcher::start(&filename, args) {
            Ok(_) => return,
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        };
    }

    let options = JsRuntimeOptions {
        seed,
        reload,
        import_map,
        num_threads,
        inspect,
        test_mode: false,
    };

    // Create new JS runtime.
    let mut runtime = JsRuntime::with_options(options);
    let mod_result = runtime.execute_module(&filename, None);

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{e:?}"),
    };
}

fn test_command(mut args: ArgMatches) {
    // Get the path we need to import JavaScript tests from.
    let cwd = env::current_dir().unwrap();

    // Get the input path as an absolute location.
    let test_path = args.remove_one::<String>("FILES").map(PathBuf::from);
    let test_path = match test_path.as_ref().unwrap_or(&cwd).absolutize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{}", generic_error(e.to_string()));
            std::process::exit(1);
        }
    };

    // Load custom .env file if specified.
    if let Some(path) = args.remove_one::<String>("env-file") {
        // Try to parse the .env file.
        if let Err(e) = dotenv::load_env_file(path) {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }

    // Extract options from cli arguments.
    let fail_fast = args.remove_one::<bool>("fail-fast").unwrap_or_default();

    let filter = match args.remove_one::<String>("filter") {
        Some(value) => format!("new RegExp({})", value),
        None => "undefined".into(),
    };

    // Note: The env variable method is used to address an issue on Windows where
    // the test path entry is injected into the test script in a slightly
    // altered manner, leading to errors.

    env::set_var("TEST_ENTRY_PATH", String::from(test_path.to_string_lossy()));

    // Build JavaScript test script.
    let script = format!(
        "
        import {{ mainRunner }} from 'test';
        mainRunner.failFast = {};
        mainRunner.filter = {};
        await mainRunner.importTests(process.env.TEST_ENTRY_PATH);
        await mainRunner.run();
    ",
        fail_fast, filter,
    );

    // Extract runtime options.
    let reload = args.remove_one::<bool>("reload").unwrap_or_default();
    let import_map = args.remove_one::<String>("import-map");
    let import_map = load_import_map(import_map);

    let seed = args
        .remove_one::<String>("seed")
        .map(|val| val.parse::<i64>().unwrap_or_default());

    let num_threads = args
        .remove_one::<String>("threadpool-size")
        .map(|val| val.parse::<usize>().unwrap_or_default());

    // Build JS runtime options.
    let options = JsRuntimeOptions {
        seed,
        test_mode: true,
        import_map,
        num_threads,
        reload,
        ..Default::default()
    };

    // Create new JS runtime.
    let mut runtime = JsRuntime::with_options(options);
    let mod_result = runtime.execute_module("dune:environment/test", Some(&script));

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{e:?}"),
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
        None => println!("{source}"),
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

    let mut cli = Command::new("dune")
        .version(env!("CARGO_PKG_VERSION"))
        .about("A hobby runtime for JavaScript and TypeScript")
        .subcommand(
            Command::new("run")
                .about("Run a JavaScript or TypeScript program")
                .arg_required_else_help(true)
                .allow_external_subcommands(true)
                .arg(arg!(<SCRIPT> "The script that will run").required(true))
                .arg(arg!(-r --reload "Reload every URL import (cache is ignored)"))
                .arg(arg!(--seed <NUMBER> "Make the Math.random() method predictable"))
                .arg(arg!(--"env-file" <FILE> "Load configuration from local file"))
                .arg(arg!(--"import-map" <FILE> "Load import map from local file"))
                .arg(arg!(--"threadpool-size" <NUMBER> "Set the number of threads used for I/O"))
                .arg(arg!(--watch <FILES>... "Watch for file changes and restart process automatically").num_args(0..))
                .arg(arg!(--inspect <ADDRESS> "Enable inspector agent (127.0.0.1:9229)")
                .default_missing_value("127.0.0.1:9229").num_args(..=1))
                .arg(arg!(--"inspect-brk" <ADDRESS> "Enable inspector agent, break before user code starts")
                .default_missing_value("127.0.0.1:9229").num_args(..=1))
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
        .subcommand(
            Command::new("test").about("Execute tests using the built-in test runner")
                .arg(arg!(<FILES>... "List of file names or directories").required(false))
                .arg(arg!(--"fail-fast" "Stop after the first failure"))
                .arg(arg!(--filter <FILTER> "Run tests with this regex pattern in test description"))
                .arg(arg!(-r --reload "Reload every URL import (cache is ignored)"))
                .arg(arg!(--seed <NUMBER> "Make the Math.random() method predictable"))
                .arg(arg!(--"env-file" <FILE> "Load configuration from local file"))
                .arg(arg!(--"import-map" <FILE> "Load import map from local file"))
                .arg(arg!(--"threadpool-size" <NUMBER> "Set the number of threads used for I/O"))
        )
        .subcommand(Command::new("upgrade").about("Upgrade to the latest dune version"))
        .subcommand(Command::new("repl").about("Start the REPL (read, eval, print, loop)"))
        .get_matches();

    let (cmd, args) = cli.remove_subcommand().unwrap_or_default();

    match (cmd.as_str(), args) {
        ("run", args) => run_command(args),
        ("bundle", args) => bundle_command(args),
        ("compile", args) => compile_command(args),
        ("test", args) => test_command(args),
        ("upgrade", _) => upgrade_command(),
        _ => repl_command(),
    }
}
