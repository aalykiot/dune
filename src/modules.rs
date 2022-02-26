use crate::errors::unwrap_or_exit;
use crate::loaders::CoreModuleLoader;
use crate::loaders::FsModuleLoader;
use crate::loaders::ModuleLoader;
use crate::loaders::UrlModuleLoader;
use crate::runtime::JsRuntime;
use anyhow::Result;
use lazy_static::lazy_static;
use rusty_v8 as v8;
use std::collections::HashMap;
use url::Url;

// Registering core modules into a vector.
lazy_static! {
    pub static ref CORE_MODULES: Vec<CoreModule> = {
        vec![
            CoreModule::new(
                "dune:modules/console",
                "console",
                include_str!("../lib/console.js"),
            ),
            CoreModule::new(
                "dune:modules/events",
                "events",
                include_str!("../lib/events.js"),
            ),
            CoreModule::new(
                "dune:modules/process",
                "process",
                include_str!("../lib/process.js"),
            ),
            CoreModule::new(
                "dune:internal/btree",
                "_internal/btree",
                include_str!("../lib/_btree.js"),
            ),
        ]
    };
}

#[derive(Debug)]
pub struct CoreModule {
    pub path: String,
    pub alias: String,
    pub code: String,
    pub internal: bool,
}

impl CoreModule {
    fn new(path: &str, alias: &str, code: &str) -> CoreModule {
        CoreModule {
            path: path.into(),
            alias: alias.into(),
            code: code.into(),
            internal: false,
        }
    }

    pub fn match_against(&self, key: &str) -> bool {
        self.path == key || self.alias == key
    }
}

// Utility to easily create v8 script origins.
pub fn create_origin<'s>(
    scope: &mut v8::HandleScope<'s, ()>,
    name: &str,
    is_module: bool,
) -> v8::ScriptOrigin<'s> {
    let name = v8::String::new(scope, name).unwrap();
    let source_map = v8::undefined(scope);
    v8::ScriptOrigin::new(
        scope,
        name.into(),
        0,
        0,
        false,
        0,
        source_map.into(),
        false,
        false,
        is_module,
    )
}

pub type ModulePath = String;
pub type ModuleSource = String;

// Points to a module that lives inside v8.
pub type ModuleReference = v8::Global<v8::Module>;

// Holds information about resolved ES modules.
#[derive(Default)]
pub struct ModuleMap {
    // main: Option<ModulePath>,
    map: HashMap<ModulePath, ModuleReference>,
}

impl ModuleMap {
    pub fn new_es_module<'a>(
        &mut self,
        scope: &mut v8::HandleScope<'a>,
        path: &str,
        module: v8::Local<'a, v8::Module>,
    ) {
        let module_ref = v8::Global::new(scope, module);
        self.map.insert(path.into(), module_ref);
    }
}

impl std::ops::Deref for ModuleMap {
    type Target = HashMap<ModulePath, ModuleReference>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for ModuleMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

// Finds the right loader, and resolves the import.
pub fn resolve_import(base: Option<&str>, specifier: &str) -> Result<ModulePath> {
    // Looking at the params to decide the loader.
    let loader: Box<dyn ModuleLoader> = {
        let is_core_module_import = CORE_MODULES.iter().any(|cm| cm.match_against(specifier));
        let is_url_import =
            Url::parse(specifier).is_ok() || (base.is_some() && Url::parse(base.unwrap()).is_ok());

        match (is_core_module_import, is_url_import) {
            (true, _) => Box::new(CoreModuleLoader),
            (_, true) => Box::new(UrlModuleLoader),
            _ => Box::new(FsModuleLoader),
        }
    };
    // Resolve module.
    loader.resolve(base, specifier)
}

// Finds the right loader, and loads the import.
pub fn load_import(specifier: &str) -> Result<ModuleSource> {
    // Looking at the params to decide the loader.
    let loader: Box<dyn ModuleLoader> = match (
        CORE_MODULES.iter().any(|cm| cm.match_against(specifier)),
        Url::parse(specifier).is_ok(),
    ) {
        (true, _) => Box::new(CoreModuleLoader),
        (_, true) => Box::new(UrlModuleLoader),
        _ => Box::new(FsModuleLoader),
    };
    // Load module.
    loader.load(specifier)
}

// It resolves module imports ahead of time (useful for async).
// https://source.chromium.org/chromium/v8/v8.git/+/51e736ca62bd5c7bfd82488a5587fed31dbf45d5:src/d8.cc;l=741
pub fn fetch_module_tree<'a>(
    scope: &mut v8::HandleScope<'a>,
    filename: &str,
    source: Option<&str>,
) -> Option<v8::Local<'a, v8::Module>> {
    // Create a script origin for the import.
    let origin = create_origin(scope, filename, true);
    let module_map_rc = JsRuntime::module_map(scope);

    // Check if source is specified from caller, if not, use a loader.
    let source = match source {
        Some(source) => source.into(),
        None => unwrap_or_exit(load_import(filename)),
    };
    let source = v8::String::new(scope, &source).unwrap();
    let source = v8::script_compiler::Source::new(source, Some(&origin));

    let module = match v8::script_compiler::compile_module(scope, source) {
        Some(module) => module,
        None => return None,
    };

    // Add new es module to runtime's module map.
    module_map_rc
        .borrow_mut()
        .new_es_module(scope, filename, module);

    let requests = module.get_module_requests();

    for i in 0..requests.length() {
        // Getting the import request from the `module_requests` array.
        let request = requests.get(scope, i).unwrap();
        let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

        // Transforming v8's ModuleRequest into a Rust string.
        let specifier = request.get_specifier().to_rust_string_lossy(scope);
        let specifier = unwrap_or_exit(resolve_import(Some(filename), &specifier));

        // Using recursion resolve the rest sub-tree of modules.
        if !module_map_rc.borrow().contains_key(&specifier) {
            fetch_module_tree(scope, &specifier, None)?;
        }
    }

    Some(module)
}
