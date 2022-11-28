use crate::errors::unwrap_or_exit;
use crate::event_loop::TaskResult;
use crate::hooks::module_resolve_cb;
use crate::loaders::CoreModuleLoader;
use crate::loaders::FsModuleLoader;
use crate::loaders::ModuleLoader;
use crate::loaders::UrlModuleLoader;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
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
            ("perf_hooks", include_str!("./js/perf_hooks.js")),
            ("colors", include_str!("./js/colors.js")),
            ("dns", include_str!("./js/dns.js")),
            ("net", include_str!("./js/net.js")),
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

#[derive(Default)]
#[allow(dead_code)]
/// Holds information about resolved ES modules.
pub struct ModuleMap {
    main: Option<ModulePath>,
    modules: HashMap<ModulePath, v8::Global<v8::Module>>,
    pub dynamic_imports_seen: HashSet<ModulePath>,
    pub dynamic_imports: Vec<(ModulePath, v8::Global<v8::PromiseResolver>)>,
}

impl ModuleMap {
    /// Registers a new ES module to the map.
    pub fn new_es_module<'a>(
        &mut self,
        scope: &mut v8::HandleScope<'a>,
        path: &str,
        module: v8::Local<'a, v8::Module>,
    ) {
        // Make a global ref.
        let module = v8::Global::new(scope, module);
        let should_update_main =
            self.main.is_none() && (fs::metadata(path).is_ok() || path.starts_with("http"));

        // No main module has been set, so let's update it's value.
        if should_update_main {
            self.main = Some(path.into());
        }

        self.modules.insert(path.into(), module);
    }

    /// Registers a new dynamic import.
    pub fn new_dynamic_import<'s>(
        &mut self,
        scope: &mut v8::HandleScope<'s>,
        base: Option<&str>,
        specifier: &str,
        promise: v8::Global<v8::PromiseResolver>,
    ) {
        let import_map = JsRuntime::state(scope).borrow().options.import_map.clone();
        let specifier = match base {
            Some(base) => match resolve_import(Some(base), specifier, import_map) {
                Ok(specifier) => specifier,
                Err(e) => {
                    let exception = v8::String::new(scope, &e.to_string()).unwrap();
                    let exception = v8::Exception::error(scope, exception);
                    promise.open(scope).reject(scope, exception);
                    return;
                }
            },
            None => specifier.into(),
        };

        // Check if we have the requested module to our cache.
        if let Some(module) = self.modules.get(&specifier) {
            let module = v8::Local::new(scope, module);
            let namespace = module.get_module_namespace();
            promise.open(scope).resolve(scope, namespace);
            return;
        }

        self.dynamic_imports.push((specifier.clone(), promise));
    }

    /// Returns the main module.
    pub fn main(&self) -> Option<ModulePath> {
        self.main.clone()
    }
}

pub struct DynamicImportFuture {
    pub specifier: String,
    pub maybe_result: TaskResult,
    pub promise: v8::Global<v8::PromiseResolver>,
}

impl JsFuture for DynamicImportFuture {
    fn run(&mut self, scope: &mut v8::HandleScope) {
        // Extract the result.
        let result = self.maybe_result.take().unwrap();

        // Handle when something goes wrong with loading the import.
        if let Err(e) = result {
            let message = v8::String::new(scope, &e.to_string()).unwrap();
            let exception = v8::Exception::error(scope, message);
            // Reject the promise on failure.
            self.promise.open(scope).reject(scope, exception);
            return;
        }

        // Create module's origin.
        let origin = create_origin(scope, &self.specifier, true);

        // Otherwise, get the result and deserialize it.
        let source = result.unwrap();
        let source: String = bincode::deserialize(&source).unwrap();
        let source = v8::String::new(scope, &source).unwrap();
        let source = v8::script_compiler::Source::new(source, Some(&origin));

        // Create a try-catch scope.
        let tc_scope = &mut v8::TryCatch::new(scope);

        // Compile source to a v8 module.
        let module = match v8::script_compiler::compile_module(tc_scope, source) {
            Some(module) => module,
            None => {
                let exception = tc_scope.exception().unwrap();
                self.promise.open(tc_scope).reject(tc_scope, exception);
                return;
            }
        };

        // Instantiate ES module.
        if module
            .instantiate_module(tc_scope, module_resolve_cb)
            .is_none()
        {
            assert!(tc_scope.has_caught());
            let exception = tc_scope.exception().unwrap();
            self.promise.open(tc_scope).reject(tc_scope, exception);
            return;
        }

        let _ = module.evaluate(tc_scope);

        // Check for module evaluation errors.
        if module.get_status() == v8::ModuleStatus::Errored {
            let exception = module.get_exception();
            self.promise.open(tc_scope).reject(tc_scope, exception);
            return;
        }

        // Update the ES modules map (for future requests).
        let state_rc = JsRuntime::state(tc_scope);
        let mut state = state_rc.borrow_mut();

        state
            .modules
            .new_es_module(tc_scope, &self.specifier, module);

        // Note: Since this is a dynamic import will resolve the promise
        // with the module's namespace object instead of it's evaluation result.
        self.promise
            .open(tc_scope)
            .resolve(tc_scope, module.get_module_namespace());
    }
}

impl std::ops::Deref for ModuleMap {
    type Target = HashMap<ModulePath, v8::Global<v8::Module>>;
    fn deref(&self) -> &Self::Target {
        &self.modules
    }
}

impl std::ops::DerefMut for ModuleMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.modules
    }
}

/// Resolves an import using the appropriate loader.
pub fn resolve_import(
    base: Option<&str>,
    specifier: &str,
    import_map: Option<ImportMap>,
) -> Result<ModulePath> {
    // Use import-maps if available.
    let specifier = match import_map {
        Some(map) => map.lookup(specifier).unwrap_or(specifier.into()),
        None => specifier.into(),
    };

    // Look the params and choose a loader.
    let loader: Box<dyn ModuleLoader> = {
        let is_core_module_import = CORE_MODULES.contains_key(specifier.as_str());
        let is_url_import = Url::parse(&specifier).is_ok();
        let is_url_import = is_url_import || (base.is_some() && Url::parse(base.unwrap()).is_ok());

        match (is_core_module_import, is_url_import) {
            (true, _) => Box::new(CoreModuleLoader),
            (_, true) => Box::new(UrlModuleLoader::default()),
            _ => Box::new(FsModuleLoader),
        }
    };

    // Resolve module.
    loader.resolve(base, &specifier)
}

/// Loads an import using the appropriate loader.
pub fn load_import(specifier: &str, skip_cache: bool) -> Result<ModuleSource> {
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
        (_, _, true) => Box::new(UrlModuleLoader { skip_cache }),
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

    // This options is used only when loading URL imports.
    let skip_cache = state.borrow().options.reload;

    // Find appropriate loader if source is empty.
    let source = match source {
        Some(source) => source.into(),
        None => unwrap_or_exit(load_import(filename, skip_cache)),
    };
    let source = v8::String::new(scope, &source).unwrap();
    let source = v8::script_compiler::Source::new(source, Some(&origin));

    let module = match v8::script_compiler::compile_module(scope, source) {
        Some(module) => module,
        None => return None,
    };

    let import_map = state.borrow().options.import_map.clone();

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
        let specifier = unwrap_or_exit(resolve_import(
            Some(filename),
            &specifier,
            import_map.clone(),
        ));

        // Resolve subtree of modules.
        if !state.borrow().modules.contains_key(&specifier) {
            fetch_module_tree(scope, &specifier, None)?;
        }
    }

    Some(module)
}

/// A single import mapping (specifier, target).
type ImportMapEntry = (String, String);

/// Key-Value entries representing WICG import-maps.
#[derive(Debug, Clone)]
pub struct ImportMap {
    map: Vec<ImportMapEntry>,
}

impl ImportMap {
    /// Creates an ImportMap from JSON text.
    pub fn parse_from_json(text: &str) -> Result<ImportMap> {
        // Parse JSON string into serde value.
        let json: Value = serde_json::from_str(&text)?;
        let imports = json["imports"].to_owned();

        if imports.is_null() || !imports.is_object() {
            return Err(anyhow!("Import map's 'imports' must be an object"));
        }

        let map: HashMap<String, String> = serde_json::from_value(imports)?;
        let mut map: Vec<ImportMapEntry> = Vec::from_iter(map.into_iter());

        // Note: We're sorting the imports because we need to support "Packages"
        // via trailing slashes, so the lengthier mapping should always be selected.
        //
        // https://github.com/WICG/import-maps#packages-via-trailing-slashes

        map.sort_by(|a, b| b.0.cmp(&a.0));

        Ok(ImportMap { map })
    }

    /// Tries to match a specifier against an import-map entry.
    pub fn lookup(&self, specifier: &str) -> Option<String> {
        // Find a mapping if exists.
        let (base, target) = match self.map.iter().find(|(k, _)| specifier.starts_with(k)) {
            Some(mapping) => mapping,
            None => return None,
        };

        // Note: The reason we need this additional check below with the specifier's
        // extension (if exists) is to be able to support extension-less imports.
        //
        // https://github.com/WICG/import-maps#extension-less-imports

        match Path::new(specifier).extension() {
            Some(ext) => match Path::new(specifier) == Path::new(base).with_extension(ext) {
                true => None,
                false => Some(specifier.replacen(base, target, 1)),
            },
            None => Some(specifier.replacen(base, target, 1)),
        }
    }
}
