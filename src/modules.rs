use crate::loaders::{FsModuleLoader, ModuleLoader};
use crate::runtime::JsRuntime;

use std::collections::HashMap;

use rusty_v8 as v8;

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
pub type ModuleReference = v8::Global<v8::Module>;

// Holds information about resolved ES modules.
#[derive(Default)]
pub struct ModuleMap(HashMap<ModulePath, ModuleReference>);

impl std::ops::Deref for ModuleMap {
    type Target = HashMap<String, v8::Global<v8::Module>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ModuleMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// It resolves module imports ahead of time (useful for async).
// https://source.chromium.org/chromium/v8/v8.git/+/51e736ca62bd5c7bfd82488a5587fed31dbf45d5:src/d8.cc;l=741
pub fn fetch_module_tree<'a>(
    scope: &mut v8::HandleScope<'a>,
    filename: &str,
) -> Option<v8::Local<'a, v8::Module>> {
    let loader = FsModuleLoader::default();
    let origin = create_origin(scope, filename, true);

    let source = loader.load(filename).unwrap();
    let source = v8::String::new(scope, &source).unwrap();
    let source = v8::script_compiler::Source::new(source, Some(&origin));

    let module = match v8::script_compiler::compile_module(scope, source) {
        Some(value) => value,
        None => return None,
    };

    let state = JsRuntime::state(scope);

    state
        .borrow_mut()
        .module_map
        .insert(filename.to_string(), v8::Global::new(scope, module));

    let requests = module.get_module_requests();

    for i in 0..requests.length() {
        // Getting the import request from the `module_requests` array.
        let request = requests.get(scope, i).unwrap();
        let request = v8::Local::<v8::ModuleRequest>::try_from(request).unwrap();

        // Transforming v8's ModuleRequest into a Rust string.
        let specifier = request.get_specifier().to_rust_string_lossy(scope);

        let target = match loader.resolve(filename, &specifier) {
            Ok(value) => value,
            Err(_) => return None,
        };
        // Using recursion resolve the rest sub-tree of modules.
        if !state.borrow().module_map.contains_key(&target) {
            fetch_module_tree(scope, &target)?;
        }
    }

    Some(module)
}

// Called during Module::instantiate_module.
// https://docs.rs/rusty_v8/latest/rusty_v8/type.ResolveModuleCallback.html
pub fn module_resolve_cb<'a>(
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _import_assertions: v8::Local<'a, v8::FixedArray>,
    referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    // Getting a CallbackScope from the given context.
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let state = JsRuntime::state(scope);

    // The following should never fail (that's why we use unwrap) since any errors should
    // have been caught at the `fetch_module_tree` step.
    let dependant = state
        .borrow()
        .module_map
        .iter()
        .find(|(_, module)| **module == v8::Global::new(scope, referrer))
        .map(|(target, _)| target.clone())
        .unwrap();

    let specifier = specifier.to_rust_string_lossy(scope);

    let loader = FsModuleLoader::default();

    let import = match loader.resolve(&dependant, &specifier) {
        Ok(value) => value,
        Err(_) => return None,
    };

    let module = match state.borrow_mut().module_map.get(&import) {
        Some(value) => value.clone(),
        None => return None,
    };

    Some(v8::Local::new(scope, module))
}
