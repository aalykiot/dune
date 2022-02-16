mod bindings;
mod errors;
mod hooks;
mod loaders;
mod modules;
mod process;
mod repl;
mod runtime;
mod stdio;

use errors::unwrap_or_exit;
use modules::resolve_import;
use runtime::JsRuntime;
use std::env;

fn main() {
    // Getting the filename from command-line arguments
    let args: Vec<String> = env::args().collect();
    // If filename is specified run it as a module, otherwise start the repl.
    if let Some(filename) = args.get(1) {
        // The following code tries to resolve the given filename to an
        // absolute path. If the first time fails we will append `./` to
        // it first, and retry the resolution in case the user forgot to specify it.
        let filename = unwrap_or_exit(
            resolve_import(None, filename)
                .or_else(|_| resolve_import(None, &format!("./{}", filename))),
        );
        // Create a new runtime instance and run the module.
        let mut runtime = JsRuntime::new();
        let mod_result = runtime.execute_module(&filename, None);

        if let Err(e) = mod_result {
            eprintln!("{:#?}", e);
        }
        return;
    }

    repl::start();
}
