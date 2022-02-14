mod bindings;
mod errors;
mod loaders;
mod modules;
mod process;
mod repl;
mod runtime;
mod stdio;

use runtime::JsRuntime;
use std::env;

fn main() {
    // Getting the filename from command-line arguments
    let args: Vec<String> = env::args().collect();
    // If filename is specified run it as a module, otherwise start the repl.
    if let Some(filename) = args.get(1) {
        let mut runtime = JsRuntime::new();
        let mod_result = runtime.execute_module(&filename);
        if let Err(e) = mod_result {
            eprintln!("{:#?}", e);
        }
    } else {
        repl::start();
    }
}
