use crate::bundle;
use crate::compile;
use crate::dotenv;
use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::modules::resolve_import;
use crate::modules::ImportMap;
use crate::repl;
use crate::runtime::JsRuntime;
use crate::runtime::JsRuntimeOptions;
use crate::upgrade;
use crate::watcher;
use anyhow::bail;
use anyhow::Result;
use clap::ArgAction;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueHint;
use colored::*;
use path_absolutize::*;
use std::env;
use std::fs;
use std::net::SocketAddrV4;
use std::ops::RangeInclusive;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[command(flatten)]
    global_args: GlobalArgs,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        about = "Run a JavaScript or TypeScript program",
        arg_required_else_help = true
    )]
    Run(RunArgs),
    #[command(
        about = "Bundle everything into a single file",
        arg_required_else_help = true
    )]
    Bundle(BundleArgs),
    #[command(
        about = "Compile script to standalone executable",
        arg_required_else_help = true
    )]
    Compile(CompileArgs),
    #[command(
        about = "Execute tests using the built-in test runner",
        arg_required_else_help = true
    )]
    Test(TestArgs),
    #[command(about = "Upgrade to the latest dune version")]
    Upgrade,
    #[command(about = "Start the REPL (read, eval, print, loop)")]
    Repl,
}

#[derive(Debug, Args)]
struct GlobalArgs {
    #[arg(
        help = "Reload every URL import (cache is ignored)",
        action = ArgAction::SetTrue,
        short,
        long,
        global = true
    )]
    reload: Option<bool>,
    #[arg(
        help = "Make the Math.random() method predictable",
        long = "seed",
        value_name = "NUMBER",
        global = true
    )]
    seed: Option<i64>,
    #[arg(
        help = "Load configuration from local file",
        long = "env-file",
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        global = true
    )]
    env_file: Option<PathBuf>,
    #[arg(
        help = "Load import map from local file",
        long = "import-map",
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        require_equals = true,
        default_missing_value = "import-map.json",
        num_args = ..=1,
        global = true
    )]
    import_map: Option<PathBuf>,
    #[arg(
        help = "Set the number of threads used for I/O",
        long = "threadpool-size",
        value_name = "NUMBER",
        global = true
    )]
    thread_pool_size: Option<usize>,
    #[arg(
        help = "Enable inspector agent (default: 127.0.0.1:9229)",
        value_name = "ADDRESS",
        long = "inspect",
        require_equals = true,
        default_missing_value = "127.0.0.1:9229",
        num_args = ..=1,
        value_parser = parse_inspect_address,
        global = true
    )]
    inspect: Option<SocketAddrV4>,
    #[arg(
        help = "Enable inspector agent, break before user code starts",
        value_name = "ADDRESS",
        long = "inspect-brk",
        require_equals = true,
        default_missing_value = "127.0.0.1:9229",
        num_args = ..=1,
        value_parser = parse_inspect_address,
        global = true
    )]
    inspect_brk: Option<SocketAddrV4>,
    #[arg(
        help = "Expose the garbage collector",
        action = ArgAction::SetTrue,
        long = "expose-gc",
        global = true
    )]
    expose_gc: Option<bool>,
}

#[derive(Debug, Parser)]
struct RunArgs {
    #[arg(help = "The script that will run", required = true)]
    script: String,
    #[arg(
        help = "Watch for file changes and restart process automatically",
        value_name = "FILES",
        num_args = 0..,
        short,
        long = "watch",
        require_equals = true,
        value_delimiter = ','
    )]
    watch: Option<Vec<String>>,
}

#[derive(Debug, Parser)]
struct BundleArgs {
    #[arg(help = "The entry point script", required = true)]
    entry: String,
    #[arg(
        help = "The filename of the generated bundle",
        short,
        long,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
    )]
    output: Option<PathBuf>,
    #[arg(
        help = "Minify the generated bundle",
        action = ArgAction::SetTrue,
        long,
    )]
    minify: Option<bool>,
}

type CompileArgs = BundleArgs;

#[derive(Debug, Parser)]
struct TestArgs {
    #[arg(
        help = "Path to a test file or directory",
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        value_delimiter = ',',
        required = false
    )]
    path: Option<PathBuf>,
    #[arg(
        help = "Stop after the first failure",
        default_value = "false",
        action = ArgAction::SetTrue,
        long
    )]
    fail_fast: bool,
    #[arg(
        help = "Run tests with this regex pattern in test description",
        value_name = "FILTER",
        require_equals = true,
        long
    )]
    filter: Option<String>,
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn parse_inspect_address(s: &str) -> Result<SocketAddrV4> {
    // Try to parse full string as IPv4 address.
    if let Ok(address) = s.parse::<SocketAddrV4>() {
        return Ok(address);
    }
    // Check if only the port is defined.
    match s.parse::<usize>() {
        Ok(port) if PORT_RANGE.contains(&port) => {
            let address = format!("127.0.0.1:{port}");
            let address = address.parse::<SocketAddrV4>().unwrap();
            return Ok(address);
        }
        _ => {}
    }
    bail!("Value can't be parsed into an IPv4 address")
}

fn load_import_map(filename: Option<&PathBuf>) -> Option<ImportMap> {
    filename.map(|file| {
        let contents = fs::read_to_string(file).map_err(|e| e.into());
        let contents = unwrap_or_exit(contents);
        unwrap_or_exit(ImportMap::parse_from_json(&contents))
    })
}

fn run_command(args: &RunArgs, globals: &GlobalArgs) {
    // Try load the requested import-map.
    let import_map = load_import_map(globals.import_map.as_ref());

    // NOTE: The following code tries to resolve the given filename
    // to an absolute path. If the first time fails we will append `./` to
    // it first, and retry the resolution in case the user forgot to specify it.
    let filename = unwrap_or_exit(
        resolve_import(None, &args.script, true, import_map.clone()).or_else(|_| {
            resolve_import(
                None,
                &format!("./{}", args.script),
                true,
                import_map.clone(),
            )
        }),
    );

    // Check if we have to run on `watch` mode.
    if args.watch.is_some() {
        let watch_paths = args.watch.to_owned().unwrap();
        match watcher::start(&filename, watch_paths) {
            Ok(_) => return,
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        };
    }

    // Load custom .env file if specified.
    if let Some(path) = globals.env_file.as_ref() {
        // Try to parse the .env file.
        if let Err(e) = dotenv::load_env_file(path) {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }

    // Check if we need to enable the inspector.
    let inspect = globals
        .inspect
        .map(|address| (address, false))
        .or(globals.inspect_brk.map(|address| (address, true)));

    // Local files must start with `file://`.
    let root = match filename.starts_with("http") {
        true => Some(filename.clone()),
        false => Some(format!("file://{}", filename.clone())),
    };

    let options = JsRuntimeOptions {
        seed: globals.seed.to_owned(),
        reload: globals.reload.unwrap_or_default(),
        num_threads: globals.thread_pool_size.to_owned(),
        import_map,
        inspect,
        root,
        test_mode: false,
        expose_gc: globals.expose_gc.unwrap_or_default(),
    };

    // Create new JS runtime.
    let mut runtime = JsRuntime::with_options(options);
    let mod_result = runtime.execute_module(&filename, None);

    match mod_result {
        Ok(_) => runtime.run_event_loop(),
        Err(e) => eprintln!("{e:?}"),
    };
}

fn test_command(args: &TestArgs, globals: &GlobalArgs) {
    // Get the path we need to import JavaScript tests from.
    let cwd = env::current_dir().unwrap();

    // Try load the requested import-map.
    let import_map = load_import_map(globals.import_map.as_ref());

    // Get the input path as an absolute location.
    let test_path = match args.path.as_ref().unwrap_or(&cwd).absolutize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("{}", generic_error(e.to_string()));
            std::process::exit(1);
        }
    };

    // Load custom .env file if specified.
    if let Some(path) = globals.env_file.as_ref() {
        // Try to parse the .env file.
        if let Err(e) = dotenv::load_env_file(path) {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }

    let filter = match args.filter.as_ref() {
        Some(value) => format!("new RegExp({value})"),
        None => "undefined".into(),
    };

    // Check if we need to enable the inspector.
    let inspect = globals
        .inspect
        .map(|address| (address, false))
        .or(globals.inspect_brk.map(|address| (address, true)));

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
        args.fail_fast, filter,
    );

    // Build JS runtime options.
    let options = JsRuntimeOptions {
        seed: globals.seed.to_owned(),
        reload: globals.reload.unwrap_or_default(),
        num_threads: globals.thread_pool_size.to_owned(),
        test_mode: true,
        import_map,
        inspect,
        expose_gc: globals.expose_gc.unwrap_or_default(),
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

fn repl_command(globals: &GlobalArgs) {
    // Build a JS runtime based on CLI arguments.
    if let Some(path) = globals.env_file.as_ref() {
        // Try to parse the .env file.
        if let Err(e) = dotenv::load_env_file(path) {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    }

    let options = JsRuntimeOptions {
        num_threads: globals.thread_pool_size.to_owned(),
        expose_gc: globals.expose_gc.unwrap_or_default(),
        seed: globals.seed.to_owned(),
        ..Default::default()
    };

    // Start REPL.
    let runtime = JsRuntime::with_options(options);
    repl::start(runtime);
}

fn upgrade_command() {
    match upgrade::run_upgrade() {
        Ok(_) => println!("Upgraded successfully"),
        Err(e) => eprintln!("{}", generic_error(e.to_string())),
    }
}

fn output_bundle(source: &str, output: Option<&PathBuf>) {
    // If output is specified write source there, otherwise print it to screen.
    match output {
        Some(output) => {
            // Make sure output has a .js extension.
            let path = output.with_extension("js");
            // Write source to output.
            match fs::create_dir_all(path.parent().unwrap()).map(|_| fs::write(path, source)) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", generic_error(e.to_string())),
            };
        }
        None => println!("{source}"),
    };
}

fn bundle_command(args: &BundleArgs, globals: &GlobalArgs) {
    // Try load the requested import-map.
    let import_map = load_import_map(globals.import_map.as_ref());
    let skip_cache = globals.reload.unwrap_or_default();
    let minify = args.minify.unwrap_or_default();

    let options = bundle::Options {
        skip_cache,
        minify,
        import_map,
    };

    match bundle::run_bundle(&args.entry, &options) {
        Ok(source) => output_bundle(&source, args.output.as_ref()),
        Err(e) => eprintln!("{:?}", generic_error(e.to_string())),
    }
}

fn compile_command(args: &CompileArgs, globals: &GlobalArgs) {
    // Try load the requested import-map.
    let import_map = load_import_map(globals.import_map.as_ref());
    let skip_cache = globals.reload.unwrap_or_default();

    let options = compile::Options {
        skip_cache,
        minify: true,
        import_map,
    };

    if let Err(e) = compile::run_compile(&args.entry, args.output.as_ref(), &options) {
        eprintln!("{:?}", generic_error(e.to_string()));
    }
}

pub fn process_cli_arguments() {
    let cli = Cli::parse();
    let globals = &cli.global_args;

    match cli.command {
        Some(Command::Run(args)) => run_command(&args, globals),
        Some(Command::Bundle(args)) => bundle_command(&args, globals),
        Some(Command::Compile(args)) => compile_command(&args, globals),
        Some(Command::Test(args)) => test_command(&args, globals),
        Some(Command::Repl) => repl_command(globals),
        Some(Command::Upgrade) => upgrade_command(),
        None => repl_command(globals),
    };
}
