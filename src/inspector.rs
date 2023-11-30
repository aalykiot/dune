// Bindings to the V8 Inspector API
// http://hyperandroid.com/2020/02/12/v8-inspector-from-an-embedder-standpoint/

use v8::inspector::V8InspectorClientBase;
use v8::inspector::V8InspectorClientImpl;

/// Currently Dune supports only a single context in `JsRuntime`.
const CONTEXT_GROUP_ID: i32 = 1;

struct InspectorClient {
    base: V8InspectorClientBase,
    paused: bool,
}

impl InspectorClient {
    /// Creates a new inspector instance.
    fn new() -> InspectorClient {
        Self {
            base: V8InspectorClientBase::new::<Self>(),
            paused: false,
        }
    }
}

impl V8InspectorClientImpl for InspectorClient {
    /// Returns a reference to v8 inspector client instance.
    fn base(&self) -> &V8InspectorClientBase {
        &self.base
    }

    /// Returns a mut reference to v8 inspector client instance.
    fn base_mut(&mut self) -> &mut V8InspectorClientBase {
        &mut self.base
    }

    /// Returns a raw pointer to the v8 inspector client instance.
    unsafe fn base_ptr(this: *const Self) -> *const V8InspectorClientBase
    where
        Self: Sized,
    {
        // SAFETY: This pointer is valid for the whole lifetime of inspector.
        unsafe { std::ptr::addr_of!((*this).base) }
    }

    /// Called by V8 debugging internals when you are breaking into js code from Dev Tools.
    fn run_message_loop_on_pause(&mut self, context_group_id: i32) {
        assert_eq!(context_group_id, CONTEXT_GROUP_ID);
        assert!(!self.paused);
        self.paused = true;

        // TODO: Consume all front end (Dev Tools) debugging messages.
        todo!()
    }

    /// Called Once V8 knows it has no more inspector messages pending.
    fn quit_message_loop_on_pause(&mut self) {
        self.paused = false;
    }
}
