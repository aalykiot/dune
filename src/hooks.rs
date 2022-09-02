use crate::bindings::throw_type_error;
use crate::errors::unwrap_or_exit;
use crate::modules::resolve_import;
use crate::runtime::JsRuntime;

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

    let referrer = v8::Global::new(scope, referrer);

    // The following should never fail (that's why we use unwrap) since any errors should
    // have been caught at the `fetch_module_tree` step.
    let dependant = state
        .modules
        .iter()
        .find(|(_, module)| **module == referrer)
        .map(|(path, _)| path.clone())
        .unwrap();

    let specifier = specifier.to_rust_string_lossy(scope);
    let specifier = unwrap_or_exit(resolve_import(Some(&dependant), &specifier));

    // This call should always give us back the module. Any errors will be caught
    // on the `fetch_module_tree` step.
    let module = state.modules.get(&specifier).unwrap().clone();

    Some(v8::Local::new(scope, module))
}

/// Called the first time import.meta is accessed for a module.
/// https://docs.rs/v8/0.49.0/v8/type.HostInitializeImportMetaObjectCallback.html
pub extern "C" fn host_initialize_import_meta_object_cb(
    context: v8::Local<v8::Context>,
    module: v8::Local<v8::Module>,
    meta: v8::Local<v8::Object>,
) {
    // Get `CallbackScope` from context.
    let scope = &mut unsafe { v8::CallbackScope::new(context) };
    let state = JsRuntime::state(scope);
    let state = state.borrow();

    // Make the module global.
    let module = v8::Global::new(scope, module);

    let url = state
        .modules
        .iter()
        .find(|(_, m)| **m == module)
        .map(|(p, _)| p.clone())
        .unwrap();

    let is_main = state.modules.main() == Some(url.to_owned());

    // Setup import.url property.
    let key = v8::String::new(scope, "url").unwrap();
    let value = v8::String::new(scope, &url).unwrap();
    meta.create_data_property(scope, key.into(), value.into());

    // Setup import.main property.
    let key = v8::String::new(scope, "main").unwrap();
    let value = v8::Boolean::new(scope, is_main);
    meta.create_data_property(scope, key.into(), value.into());

    let url = v8::String::new(scope, &url).unwrap();
    let builder = v8::FunctionBuilder::new(import_meta_resolve).data(url.into());

    // Setup import.resolve() method.
    let key = v8::String::new(scope, "resolve").unwrap();
    let value = v8::FunctionBuilder::<v8::Function>::build(builder, scope).unwrap();
    meta.set(scope, key.into(), value.into());
}

fn import_meta_resolve(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
    mut rv: v8::ReturnValue,
) {
    // Check for provided arguments.
    if args.length() == 0 {
        throw_type_error(scope, "Not enough arguments specified.");
        return;
    }

    let base = args.data().unwrap().to_rust_string_lossy(scope);
    let specifier = args.get(0).to_rust_string_lossy(scope);

    match resolve_import(Some(&base), &specifier) {
        Ok(path) => rv.set(v8::String::new(scope, &path).unwrap().into()),
        Err(e) => throw_type_error(scope, &e.to_string()),
    };
}

/// Called when a promise rejects with no rejection handler specified.
/// https://docs.rs/v8/0.49.0/v8/type.PromiseRejectCallback.html
pub extern "C" fn promise_reject_cb(message: v8::PromiseRejectMessage) {
    // Create a v8 callback-scope.
    let scope = &mut unsafe { v8::CallbackScope::new(&message) };
    let event = message.get_event();

    let reason = match event {
        v8::PromiseRejectEvent::PromiseHandlerAddedAfterReject
        | v8::PromiseRejectEvent::PromiseRejectAfterResolved
        | v8::PromiseRejectEvent::PromiseResolveAfterResolved => return,
        v8::PromiseRejectEvent::PromiseRejectWithNoHandler => message.get_value().unwrap(),
    };

    // Get access to the runtime's state.
    let state_rc = JsRuntime::state(scope);
    let mut state = state_rc.borrow_mut();

    let reason = v8::Global::new(scope, reason);

    // Register this promise rejection to the runtime.
    state.promise_exceptions.push(reason);
}
