use crate::errors::unwrap_or_exit;
use crate::modules::resolve_import;
use crate::runtime::JsRuntime;
use v8;

/// Called during Module::instantiate_module.
/// https://docs.rs/rusty_v8/latest/rusty_v8/type.ResolveModuleCallback.html
pub fn module_resolve_cb<'a>(
    context: v8::Local<'a, v8::Context>,
    specifier: v8::Local<'a, v8::String>,
    _: v8::Local<'a, v8::FixedArray>,
    referrer: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    // Get `CallbackScope` from context.
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let state = JsRuntime::state(scope);
    let state = state.borrow();

    // The following should never fail (that's why we use unwrap) since any errors should
    // have been caught at the `fetch_module_tree` step.
    let dependant = state
        .modules
        .iter()
        .find(|(_, module)| **module == v8::Global::new(scope, referrer))
        .map(|(path, _)| path.clone())
        .unwrap();

    let specifier = specifier.to_rust_string_lossy(scope);
    let specifier = unwrap_or_exit(resolve_import(Some(&dependant), &specifier));

    // This call should always give us back the module. Any errors will be caught
    // on the `fetch_module_tree` step.
    let module = state.modules.get(&specifier).unwrap().clone();

    Some(v8::Local::new(scope, module))
}
