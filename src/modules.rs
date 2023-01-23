use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use crate::errors::JsError;
use crate::event_loop::LoopHandle;
use crate::event_loop::TaskResult;
use crate::loaders::CoreModuleLoader;
use crate::loaders::FsModuleLoader;
use crate::loaders::ModuleLoader;
use crate::loaders::UrlModuleLoader;
use crate::runtime::JsFuture;
use crate::runtime::JsRuntime;
use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::env;
use std::fs;
use std::path::Path;
use std::rc::Rc;
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
            ("stream", include_str!("./js/stream.js")),
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

pub struct ModuleMap {
    pub main: Option<ModulePath>,
    pub index: HashMap<ModulePath, v8::Global<v8::Module>>,
    pub seen: HashSet<ModulePath>,
    pub pending: Vec<Rc<RefCell<ModuleGraph>>>,
}

impl ModuleMap {
    // Creates a new module-map instance.
    pub fn new() -> ModuleMap {
        Self {
            main: None,
            index: HashMap::new(),
            seen: HashSet::new(),
            pending: vec![],
        }
    }

    // Inserts a compiled ES module to the map.
    pub fn insert(&mut self, path: &str, module: v8::Global<v8::Module>) {
        // No main module has been set, so let's update the value.
        if self.main.is_none() && (fs::metadata(path).is_ok() || path.starts_with("http")) {
            self.main = Some(path.into());
        }
        self.index.insert(path.into(), module);
    }

    // Returns if there are still pending imports to be loaded.
    pub fn has_pending_imports(&self) -> bool {
        !self.pending.is_empty()
    }

    // Returns a v8 module reference from me module-map.
    pub fn get(&self, key: &str) -> Option<v8::Global<v8::Module>> {
        self.index.get(key).cloned()
    }

    // Returns a specifier given a v8 module.
    pub fn get_path(&self, module: v8::Global<v8::Module>) -> Option<ModulePath> {
        self.index
            .iter()
            .find(|(_, m)| **m == module)
            .map(|(p, _)| p.clone())
    }

    // Returns the main entry point.
    pub fn main(&self) -> Option<ModulePath> {
        self.main.clone()
    }
}

#[derive(Debug, Clone)]
pub enum ImportKind {
    // Loading static imports.
    Static,
    // Loading a dynamic import.
    Dynamic(v8::Global<v8::PromiseResolver>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStatus {
    // Indicates the module is being fetched.
    Fetching,
    // Indicates the dependencies are being fetched.
    Resolving,
    // Indicates the modules is resolved.
    Ready,
}

#[derive(Debug)]
pub struct EsModule {
    pub path: ModulePath,
    pub status: ModuleStatus,
    pub dependencies: Vec<Rc<RefCell<EsModule>>>,
    pub exception: Rc<RefCell<Option<String>>>,
    pub is_dynamic_import: bool,
}

impl EsModule {
    // Traverses the dependency tree to check if the module is ready.
    pub fn fast_forward(&mut self) {
        // If the module is ready, no need to check the sub-tree.
        if self.status == ModuleStatus::Ready {
            return;
        }

        // Fast-forward all dependencies.
        self.dependencies
            .iter_mut()
            .for_each(|dep| dep.borrow_mut().fast_forward());

        // The module is compiled and has 0 dependencies.
        if self.dependencies.is_empty() && self.status == ModuleStatus::Resolving {
            self.status = ModuleStatus::Ready;
            return;
        }

        // At this point, the module is still being fetched...
        if self.dependencies.is_empty() {
            return;
        }

        if !self
            .dependencies
            .iter_mut()
            .map(|m| m.borrow().status.clone())
            .any(|status| status != ModuleStatus::Ready)
        {
            self.status = ModuleStatus::Ready;
        }
    }
}

#[derive(Debug)]
pub struct ModuleGraph {
    pub kind: ImportKind,
    pub root_rc: Rc<RefCell<EsModule>>,
    pub same_origin: LinkedList<v8::Global<v8::PromiseResolver>>,
}

impl ModuleGraph {
    // Initializes a new graph resolving a static import.
    pub fn static_import(path: &str) -> ModuleGraph {
        // Create an ES module instance.
        let module = Rc::new(RefCell::new(EsModule {
            path: path.into(),
            status: ModuleStatus::Fetching,
            dependencies: vec![],
            exception: Rc::new(RefCell::new(None)),
            is_dynamic_import: false,
        }));

        Self {
            kind: ImportKind::Static,
            root_rc: module,
            same_origin: LinkedList::new(),
        }
    }

    // Initializes a new graph resolving a dynamic import.
    pub fn dynamic_import(path: &str, promise: v8::Global<v8::PromiseResolver>) -> ModuleGraph {
        // Create an ES module instance.
        let module = Rc::new(RefCell::new(EsModule {
            path: path.into(),
            status: ModuleStatus::Fetching,
            dependencies: vec![],
            exception: Rc::new(RefCell::new(None)),
            is_dynamic_import: true,
        }));

        Self {
            kind: ImportKind::Dynamic(promise),
            root_rc: module,
            same_origin: LinkedList::new(),
        }
    }
}

pub struct EsModuleFuture {
    pub path: ModulePath,
    pub module: Rc<RefCell<EsModule>>,
    pub maybe_result: TaskResult,
}

impl EsModuleFuture {
    // Handles an error based on the import type.
    fn handle_failure(&mut self, e: anyhow::Error) {
        let module = self.module.borrow();
        // In dynamic imports we reject the promise(s).
        if module.is_dynamic_import {
            module.exception.borrow_mut().replace(e.to_string());
            return;
        }
        // In static imports we exit the process.
        eprintln!("{}", generic_error(e.to_string()));
        std::process::exit(1);
    }
}

impl JsFuture for EsModuleFuture {
    /// Drives the future to completion.
    fn run(&mut self, scope: &mut v8::HandleScope) {
        let state_rc = JsRuntime::state(scope);
        let mut state = state_rc.borrow_mut();

        // If the graph has exceptions, stop resolving the current sub-tree (dynamic imports).
        if self.module.borrow().exception.borrow().is_some() {
            state.module_map.seen.remove(&self.path);
            return;
        }

        // Extract module's source code.
        let source = self.maybe_result.take().unwrap();
        let source = match source {
            Ok(source) => bincode::deserialize::<String>(&source).unwrap(),
            Err(e) => {
                self.handle_failure(Error::msg(e.to_string()));
                return;
            }
        };

        let tc_scope = &mut v8::TryCatch::new(scope);
        let origin = create_origin(tc_scope, &self.path, true);

        // Compile source and get it's dependencies.
        let source = v8::String::new(tc_scope, &source).unwrap();
        let source = v8::script_compiler::Source::new(source, Some(&origin));

        let module = match v8::script_compiler::compile_module(tc_scope, source) {
            Some(module) => module,
            None => {
                assert!(tc_scope.has_caught());
                let exception = tc_scope.exception().unwrap();
                let exception = JsError::from_v8_exception(tc_scope, exception, None);
                let exception = format!("{} ({})", exception.message, exception.resource_name);

                self.handle_failure(Error::msg(exception));
                return;
            }
        };

        state
            .module_map
            .insert(&self.path, v8::Global::new(tc_scope, module));

        let import_map = state.options.import_map.clone();
        let skip_cache = match self.module.borrow().is_dynamic_import {
            true => true,
            false => state.options.reload,
        };

        let mut dependencies = vec![];

        let requests = module.get_module_requests();
        let base = self.path.clone();

        for i in 0..requests.length() {
            // Get import request from the `module_requests` array.
            let request = requests.get(tc_scope, i).unwrap();
            let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

            // Transform v8's ModuleRequest into Rust string.
            let base = Some(base.as_str());
            let specifier = request.get_specifier().to_rust_string_lossy(tc_scope);
            let specifier = match resolve_import(base, &specifier, import_map.clone()) {
                Ok(specifier) => specifier,
                Err(e) => {
                    self.handle_failure(Error::msg(e.to_string()));
                    return;
                }
            };

            // Requested module has been seen already.
            if state.module_map.seen.contains(&specifier) {
                continue;
            }

            state.module_map.seen.insert(specifier.clone());

            // Create a new ES module instance.
            let module = Rc::new(RefCell::new(EsModule {
                path: specifier.clone(),
                status: ModuleStatus::Fetching,
                dependencies: vec![],
                exception: Rc::clone(&self.module.borrow().exception),
                is_dynamic_import: self.module.borrow().is_dynamic_import,
            }));

            dependencies.push(Rc::clone(&module));

            // Use the event-loop to asynchronously load the requested module.
            let task = {
                let specifier = specifier.clone();
                move || match load_import(&specifier, skip_cache) {
                    Ok(source) => Some(Ok(bincode::serialize(&source).unwrap())),
                    Err(e) => Some(Result::Err(e)),
                }
            };

            let task_cb = {
                let state_rc = state_rc.clone();
                move |_: LoopHandle, maybe_result: TaskResult| {
                    let mut state = state_rc.borrow_mut();
                    let future = EsModuleFuture {
                        path: specifier,
                        module: Rc::clone(&module),
                        maybe_result,
                    };
                    state.pending_futures.push(Box::new(future));
                }
            };

            state.handle.spawn(task, Some(task_cb));
        }

        self.module.borrow_mut().status = ModuleStatus::Resolving;
        self.module.borrow_mut().dependencies = dependencies;
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
        Some(map) => map.lookup(specifier).unwrap_or_else(|| specifier.into()),
        None => specifier.into(),
    };

    // Look the params and choose a loader.
    let loader: Box<dyn ModuleLoader> = {
        let is_core_module_import = CORE_MODULES.contains_key(specifier.as_str());
        let is_url_import = Url::parse(&specifier).is_ok();
        let is_url_import = is_url_import || (base.is_some() && Url::parse(base.unwrap()).is_ok());

        match (is_core_module_import, is_url_import) {
            (true, _) => Box::new(CoreModuleLoader),
            (_, true) => Box::<UrlModuleLoader>::default(),
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
        let json: Value = serde_json::from_str(text)?;
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
        let (base, mut target) = match self.map.iter().find(|(k, _)| specifier.starts_with(k)) {
            Some(mapping) => mapping.to_owned(),
            None => return None,
        };

        // The following code treats "./" as an alias for the CWD.
        if target.starts_with("./") {
            let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
            target = target.replacen('.', &cwd, 1);
        }

        // Note: The reason we need this additional check below with the specifier's
        // extension (if exists) is to be able to support extension-less imports.
        //
        // https://github.com/WICG/import-maps#extension-less-imports

        match Path::new(specifier).extension() {
            Some(ext) => match Path::new(specifier) == Path::new(&base).with_extension(ext) {
                false => Some(specifier.replacen(&base, &target, 1)),
                _ => None,
            },
            None => Some(specifier.replacen(&base, &target, 1)),
        }
    }
}

/// Resolves module imports synchronously.
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
        None => unwrap_or_exit(load_import(filename, true)),
    };
    let source = v8::String::new(scope, &source).unwrap();
    let source = v8::script_compiler::Source::new(source, Some(&origin));

    let module = match v8::script_compiler::compile_module(scope, source) {
        Some(module) => module,
        None => return None,
    };

    // Subscribe module to the module-map.
    let module_ref = v8::Global::new(scope, module);
    state.borrow_mut().module_map.insert(filename, module_ref);

    let requests = module.get_module_requests();

    for i in 0..requests.length() {
        // Get import request from the `module_requests` array.
        let request = requests.get(scope, i).unwrap();
        let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

        // Transform v8's ModuleRequest into Rust string.
        let specifier = request.get_specifier().to_rust_string_lossy(scope);
        let specifier = unwrap_or_exit(resolve_import(Some(filename), &specifier, None));

        // Resolve subtree of modules.
        if !state.borrow().module_map.index.contains_key(&specifier) {
            fetch_module_tree(scope, &specifier, None)?;
        }
    }

    Some(module)
}
