use crate::errors::unwrap_or_exit;
use crate::loaders::CoreModuleLoader;
use crate::loaders::FsModuleLoader;
use crate::loaders::ModuleLoader;
use crate::loaders::UrlModuleLoader;
use crate::runtime::JsRuntime;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use url::Url;

lazy_static! {
    pub static ref CORE_MODULES: HashMap<&'static str, &'static str> = {
        let modules = vec![
            ("console", include_str!("./js/console.js")),
            ("events", include_str!("./js/events.js")),
            ("process", include_str!("./js/process.js")),
            ("timers", include_str!("./js/timers.js")),
            ("assert", include_str!("./js/assert.js")),
            ("util", include_str!("./js/util.js")),
            ("text-encoding", include_str!("./js/text-encoding.js")),
            ("fs", include_str!("./js/fs.js")),
        ];
        HashMap::from_iter(modules.into_iter())
    };
}

/// Creates v8 script origins.
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

/// Points to a module that lives inside v8.
pub type ModuleReference = v8::Global<v8::Module>;

#[derive(Default)]
#[allow(dead_code)]
/// Holds information about resolved ES modules.
pub struct ModuleMap {
    main: Option<ModulePath>,
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

/// Resolves an import using the appropriate loader.
pub fn resolve_import(base: Option<&str>, specifier: &str) -> Result<ModulePath> {
    // Look the params and choose a loader.
    let loader: Box<dyn ModuleLoader> = {
        // Regex to match valid URLs based on VB.NET's URL validation.
        // http://urlregex.com/
        lazy_static! {
            static ref URL_REGEX: Regex =
                Regex::new(r"(http(s)?://)?([\w-]+\.)+[\w-]+[.com]+(/[/?%&=]*)?").unwrap();
        }

        let is_core_module_import = CORE_MODULES.contains_key(specifier);
        let is_url_import =
            URL_REGEX.is_match(specifier) || (base.is_some() && URL_REGEX.is_match(base.unwrap()));

        match (is_core_module_import, is_url_import) {
            (true, _) => Box::new(CoreModuleLoader),
            (_, true) => Box::new(UrlModuleLoader),
            _ => Box::new(FsModuleLoader),
        }
    };

    // Resolve module.
    loader.resolve(base, specifier)
}

/// Loads an import using the appropriate loader.
pub fn load_import(specifier: &str) -> Result<ModuleSource> {
    // Windows absolute path regex validator.
    lazy_static! {
        static ref WINDOWS_REGEX: Regex = Regex::new(r"^[a-zA-Z]:\\").unwrap();
    }

    // Look the params and choose a loader.
    let loader: Box<dyn ModuleLoader> = match (
        CORE_MODULES.contains_key(specifier),
        WINDOWS_REGEX.is_match(specifier),
        Url::parse(specifier).is_ok(),
    ) {
        (true, _, _) => Box::new(CoreModuleLoader),
        (_, true, _) => Box::new(FsModuleLoader),
        (_, _, true) => Box::new(UrlModuleLoader),
        _ => Box::new(FsModuleLoader),
    };

    // Load module.
    loader.load(specifier)
}

/// Resolves module imports ahead of time (useful for async).
/// https://source.chromium.org/chromium/v8/v8.git/+/51e736ca62bd5c7bfd82488a5587fed31dbf45d5:src/d8.cc;l=741
pub fn fetch_module_tree<'a>(
    scope: &mut v8::HandleScope<'a>,
    filename: &str,
    source: Option<&str>,
) -> Option<v8::Local<'a, v8::Module>> {
    // Create a script origin.
    let origin = create_origin(scope, filename, true);
    let state = JsRuntime::state(scope);

    // Find appropriate loader if source is empty.
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

    // Add ES module to map.
    state
        .borrow_mut()
        .modules
        .new_es_module(scope, filename, module);

    let requests = module.get_module_requests();

    for i in 0..requests.length() {
        // Get import request from the `module_requests` array.
        let request = requests.get(scope, i).unwrap();
        let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

        // Transform v8's ModuleRequest into Rust string.
        let specifier = request.get_specifier().to_rust_string_lossy(scope);
        let specifier = unwrap_or_exit(resolve_import(Some(filename), &specifier));

        // Resolve subtree of modules.
        if !state.borrow().modules.contains_key(&specifier) {
            fetch_module_tree(scope, &specifier, None)?;
        }
    }

    Some(module)
}
