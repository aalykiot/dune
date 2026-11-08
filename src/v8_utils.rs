use crate::errors::extract_error_code;
use crate::errors::IoError;
use anyhow::Error;
use std::cell::Cell;
use std::ffi::c_void;
use std::rc::Rc;

/// Adds a property with the given name and value, into the given object.
pub fn set_property_to(
    scope: &mut v8::PinScope,
    target: v8::Local<v8::Object>,
    name: &'static str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.set(scope, key.into(), value);
}

/// Adds a read-only property with the given name and value, into the given object.
pub fn set_constant_to(
    scope: &mut v8::PinScope,
    target: v8::Local<v8::Object>,
    name: &str,
    value: v8::Local<v8::Value>,
) {
    let key = v8::String::new(scope, name).unwrap();
    target.define_own_property(scope, key.into(), value, v8::PropertyAttribute::READ_ONLY);
}

/// Adds a `Function` object which calls the given Rust function
pub fn set_function_to(
    scope: &mut v8::PinScope,
    target: v8::Local<v8::Object>,
    name: &'static str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) {
    let key = v8::String::new(scope, name).unwrap();
    let template = v8::FunctionTemplate::new(scope, callback);
    let val = template.get_function(scope).unwrap();

    target.set(scope, key.into(), val.into());
}

/// Creates an object with a given name under a `target` object.
pub fn create_object_under<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<v8::Object>,
    name: &'static str,
) -> v8::Local<'s, v8::Object> {
    let template = v8::ObjectTemplate::new(scope);
    let key = v8::String::new(scope, name).unwrap();
    let value = template.new_instance(scope).unwrap();

    target.set(scope, key.into(), value.into());
    value
}

/// Stores a Rust type inside a v8 object.
pub fn set_internal_ref<T>(
    scope: &mut v8::PinScope,
    target: v8::Local<v8::Object>,
    index: usize,
    data: T,
) -> *mut T {
    let boxed_ref = Box::new(data);
    let address = Box::leak(boxed_ref) as *mut T;
    let v8_ext = v8::External::new(scope, address as *mut c_void);

    target.set_internal_field(index, v8_ext.into());
    address
}

/// Gets a previously stored Rust type from a v8 object.
pub fn get_internal_ref<'s, T>(
    scope: &mut v8::PinScope<'s, '_>,
    source: v8::Local<v8::Object>,
    index: usize,
) -> &'s mut T {
    let v8_ref = source.get_internal_field(scope, index).unwrap();
    let external = v8_ref.cast::<v8::External>();
    let value = external.value() as *mut T;

    unsafe { &mut *value }
}

/// Sets error code to exception if possible.
pub fn set_exception_code(
    scope: &mut v8::PinScope,
    exception: v8::Local<v8::Value>,
    error: &Error,
) {
    let exception = exception.to_object(scope).unwrap();
    if let Some(error) = error.downcast_ref::<IoError>() {
        if let Some(code) = extract_error_code(error) {
            let key = v8::String::new(scope, "code").unwrap();
            let value = v8::String::new(scope, &format!("ERR_{code}")).unwrap();
            exception.set(scope, key.into(), value.into());
        }
    }
}

/// Useful utility to throw v8 exceptions.
pub fn throw_exception(scope: &mut v8::PinScope, error: &Error) {
    let message = error.to_string().to_owned();
    let message = v8::String::new(scope, &message).unwrap();
    let exception = v8::Exception::error(scope, message);
    set_exception_code(scope, exception, error);
    scope.throw_exception(exception);
}

/// Useful utility to throw v8 type errors.
pub fn throw_type_error(scope: &mut v8::PinScope, message: &str) {
    let message = v8::String::new(scope, message).unwrap();
    let exception = v8::Exception::type_error(scope, message);
    scope.throw_exception(exception);
}

/// Returns a v8 object that will drop the internal Rust instance
/// when the garbage collector runs the clean-up.
pub fn wrap_gc_dropped<'s, T: 'static>(
    scope: &mut v8::PinScope<'s, '_>,
    value: T,
) -> v8::Local<'s, v8::Object> {
    // Allocate space for the Rust value.
    let wrapper = v8::ObjectTemplate::new(scope);
    wrapper.set_internal_field_count(1);

    let wrapper = wrapper.new_instance(scope).unwrap();
    let value_ptr = set_internal_ref(scope, wrapper, 0, value);
    let weak_rc = Rc::new(Cell::new(None));

    // Note: To automatically release the resource when the object that
    // owns the underlying Rust instance is garbage collected, we use
    // a Weak reference with a finalizer callback.
    let value_weak = v8::Weak::with_finalizer(
        scope,
        wrapper,
        Box::new({
            let weak_rc = weak_rc.clone();
            move |isolate| unsafe {
                drop(Box::from_raw(value_ptr));
                drop(v8::Weak::from_raw(isolate, weak_rc.get()));
            }
        }),
    );

    weak_rc.set(value_weak.into_raw());
    wrapper
}

/// Looks up `object[key]` and downcasts it to T.
pub fn get_property<'s, T>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    key: &str,
) -> Option<v8::Local<'s, T>>
where
    v8::Local<'s, v8::Value>: TryInto<v8::Local<'s, T>>,
{
    let key = v8::String::new(scope, key)?;
    object.get(scope, key.into())?.try_into().ok()
}
