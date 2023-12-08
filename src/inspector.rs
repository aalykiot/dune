// Bindings to the V8 Inspector API.
//
// http://hyperandroid.com/2020/02/12/v8-inspector-from-an-embedder-standpoint/
// https://github.com/ahmadov/v8_inspector_example/tree/master/

use crate::event_loop::LoopInterruptHandle;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::Serialize;
use std::cell::Cell;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::net::SocketAddrV4;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use uuid::Uuid;

// Dune supports only a single context in `JsRuntime`.
const CONTEXT_GROUP_ID: i32 = 1;

#[derive(Serialize)]
struct Details {
    description: String,
    #[serde(rename = "devtoolsFrontendUrl")]
    devtools_frontend_url: String,
    id: String,
    title: String,
    #[serde(rename = "type")]
    type_: String,
    url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
}

#[derive(Serialize)]
struct Versions {
    #[serde(rename = "Browser")]
    browser: String,
    #[serde(rename = "Protocol-Version")]
    protocol: String,
    #[serde(rename = "V8-Version")]
    v8: String,
}

// Messages sent by the connected frontend devtools.
type InspectorMessage = String;
// Messages send by the ws server to inspector.
type FrontendMessage = String;

/// This structure is used responsible for providing inspector
/// interface to the `JsRuntime`.
pub struct JsRuntimeInspector {
    v8_inspector_client: v8::inspector::V8InspectorClientBase,
    v8_inspector: Rc<RefCell<v8::UniquePtr<v8::inspector::V8Inspector>>>,
    session: Option<Box<InspectorSession>>,
    inbound_rx: mpsc::Receiver<FrontendMessage>,
    inbound_tx: mpsc::Sender<FrontendMessage>,
    handle: LoopInterruptHandle,
    sessions_tx: mpsc::Sender<()>,
    sessions_rx: mpsc::Receiver<()>,
    on_pause: Rc<Cell<bool>>,
    waiting_for_session: Rc<Cell<bool>>,
    outbound_tx: broadcast::Sender<InspectorMessage>,
}

impl JsRuntimeInspector {
    pub fn new(
        isolate: &mut v8::Isolate,
        context: v8::Global<v8::Context>,
        handle: LoopInterruptHandle,
        wait_and_break: bool,
    ) -> Rc<RefCell<Self>> {
        // Create a JsRuntimeInspector instance.
        let v8_inspector_client = v8::inspector::V8InspectorClientBase::new::<Self>();
        let (inbound_tx, inbound_rx) = mpsc::channel::<FrontendMessage>();
        let (sessions_tx, sessions_rx) = mpsc::channel::<()>();
        let (outbound_tx, _outbound_rx) = broadcast::channel::<InspectorMessage>(16);

        let inspector = Rc::new(RefCell::new(Self {
            v8_inspector_client,
            v8_inspector: Default::default(),
            session: None,
            inbound_tx,
            inbound_rx,
            handle,
            sessions_tx,
            sessions_rx,
            on_pause: Rc::new(Cell::new(false)),
            waiting_for_session: Rc::new(Cell::new(wait_and_break)),
            outbound_tx,
        }));

        let scope = &mut v8::HandleScope::new(isolate);
        let context = v8::Local::new(scope, context);

        let mut this = inspector.borrow_mut();
        this.v8_inspector = Rc::new(RefCell::new(
            v8::inspector::V8Inspector::create(scope, &mut *this).into(),
        ));
        this.session = Some(InspectorSession::new(
            this.v8_inspector.clone(),
            this.outbound_tx.clone(),
        ));

        // Tell the inspector about the global context.
        this.context_created(context, "global context", r#"{"isDefault": true}"#);

        // Note: In order to return the `JsRuntimeInspector` we need to release
        // the borrow we have with `this`.
        drop(this);

        inspector
    }

    // Starts listening for ws connections.
    pub fn start_agent(&mut self, address: SocketAddrV4) {
        // Build the shared state for axum.
        let state = AppState {
            id: Uuid::new_v4(),
            address: address.clone(),
            outbound_tx: self.outbound_tx.clone(),
            inbound_tx: self.inbound_tx.clone(),
            handle: self.handle.clone(),
            sessions_tx: self.sessions_tx.clone(),
        };

        let executor = tokio::runtime::Runtime::new().unwrap();
        println!("Debugger listening on ws://{}/{}", address, state.id);

        // Spawn the web-socket server thread.
        thread::spawn(move || executor.block_on(serve(state)));

        // Wait for session to connect if requested.
        self.wait_for_session_and_break_on_next_statement();
    }

    // Polls the inbound channel for available CDP messages.
    pub fn poll_sessions(&mut self) {
        for message in self.inbound_rx.try_iter() {
            self.session.as_mut().unwrap().dispatch_message(message);
        }
    }

    /// This function blocks the thread until at least one inspector client has
    /// established a websocket connection. After that, it instructs V8
    /// to pause at the next statement.
    fn wait_for_session_and_break_on_next_statement(&mut self) {
        // Wait until a ws connection is established.
        let _ = self.sessions_rx.recv().unwrap();
        // Poll sessions for CDP messages.
        self.poll_sessions();
        self.session.as_mut().unwrap().break_on_next_statement();
    }

    // Notify the inspector about the newly created context.
    fn context_created(&mut self, context: v8::Local<v8::Context>, name: &str, aux_data: &str) {
        // Build v8 compatible values.
        let context_name = v8::inspector::StringView::from(name.as_bytes());
        let aux_data_view = v8::inspector::StringView::from(aux_data.as_bytes());

        // Get a mut reference to v8 inspector.
        let mut inspector_rc = self.v8_inspector.borrow_mut();
        let inspector = inspector_rc.as_mut().unwrap();

        inspector.context_created(context, CONTEXT_GROUP_ID, context_name, aux_data_view);
    }
}

impl v8::inspector::V8InspectorClientImpl for JsRuntimeInspector {
    fn base(&self) -> &v8::inspector::V8InspectorClientBase {
        &self.v8_inspector_client
    }

    fn base_mut(&mut self) -> &mut v8::inspector::V8InspectorClientBase {
        &mut self.v8_inspector_client
    }

    unsafe fn base_ptr(this: *const Self) -> *const v8::inspector::V8InspectorClientBase
    where
        Self: Sized,
    {
        // SAFETY: this pointer is valid for the whole lifetime of inspector
        unsafe { std::ptr::addr_of!((*this).v8_inspector_client) }
    }

    fn run_message_loop_on_pause(&mut self, context_group_id: i32) {
        // Context id should always be the same.
        assert_eq!(context_group_id, CONTEXT_GROUP_ID);
        self.on_pause.set(true);
        // Poll session while we're on the "pause" state.
        while self.on_pause.get() {
            self.poll_sessions();
        }
    }

    fn quit_message_loop_on_pause(&mut self) {
        self.on_pause.set(false);
    }

    fn run_if_waiting_for_debugger(&mut self, context_group_id: i32) {
        assert_eq!(context_group_id, CONTEXT_GROUP_ID);
        self.waiting_for_session.set(false);
    }
}

/// An inspector session that proxies messages to concrete "transport layer",
/// like a websocket connection.
struct InspectorSession {
    v8_channel: v8::inspector::ChannelBase,
    v8_session: v8::UniqueRef<v8::inspector::V8InspectorSession>,
    outbound_tx: broadcast::Sender<InspectorMessage>,
}

impl InspectorSession {
    pub fn new(
        v8_inspector: Rc<RefCell<v8::UniquePtr<v8::inspector::V8Inspector>>>,
        outbound_tx: broadcast::Sender<InspectorMessage>,
    ) -> Box<InspectorSession> {
        new_box_with(move |self_ptr| {
            let v8_channel = v8::inspector::ChannelBase::new::<Self>();
            let mut v8_inspector = v8_inspector.borrow_mut();
            let v8_inspector_ptr = v8_inspector.as_mut().unwrap();

            #[allow(clippy::undocumented_unsafe_blocks)]
            let v8_session = v8_inspector_ptr.connect(
                CONTEXT_GROUP_ID,
                // Note: V8Inspector::connect() should require that the 'v8_channel'
                // argument cannot move.
                unsafe { &mut *self_ptr },
                v8::inspector::StringView::empty(),
                v8::inspector::V8InspectorClientTrustLevel::FullyTrusted,
            );

            Self {
                v8_channel,
                v8_session,
                outbound_tx,
            }
        })
    }

    // Dispatch message to V8 session.
    fn dispatch_message(&mut self, msg: String) {
        let bytes = msg.as_bytes();
        let v8_message = v8::inspector::StringView::from(bytes);
        self.v8_session.dispatch_protocol_message(v8_message);
    }

    // Dispatch message to outbound channel.
    fn send_message(&self, msg: v8::UniquePtr<v8::inspector::StringBuffer>) {
        let message = msg.unwrap().string().to_string();
        let _ = self.outbound_tx.send(message);
    }

    // Schedule a v8 break on next statement.
    pub fn break_on_next_statement(&mut self) {
        let reason = v8::inspector::StringView::from(&b"debugCommand"[..]);
        let details = v8::inspector::StringView::empty();
        (*self.v8_session).schedule_pause_on_next_statement(reason, details);
    }
}

impl v8::inspector::ChannelImpl for InspectorSession {
    fn base(&self) -> &v8::inspector::ChannelBase {
        &self.v8_channel
    }

    fn base_mut(&mut self) -> &mut v8::inspector::ChannelBase {
        &mut self.v8_channel
    }

    unsafe fn base_ptr(this: *const Self) -> *const v8::inspector::ChannelBase
    where
        Self: Sized,
    {
        // SAFETY: This pointer is valid for the whole lifetime of inspector.
        unsafe { std::ptr::addr_of!((*this).v8_channel) }
    }

    fn send_response(
        &mut self,
        _call_id: i32,
        message: v8::UniquePtr<v8::inspector::StringBuffer>,
    ) {
        self.send_message(message);
    }

    fn send_notification(&mut self, message: v8::UniquePtr<v8::inspector::StringBuffer>) {
        self.send_message(message);
    }

    fn flush_protocol_notifications(&mut self) {}
}

#[derive(Clone)]
struct AppState {
    pub id: Uuid,
    pub address: SocketAddrV4,
    pub outbound_tx: broadcast::Sender<InspectorMessage>,
    pub inbound_tx: mpsc::Sender<FrontendMessage>,
    pub sessions_tx: mpsc::Sender<()>,
    pub handle: LoopInterruptHandle,
}

impl AppState {
    // Returns the websocket URL for the debugger.
    pub fn url(&self) -> String {
        format!("{}/{}", self.address, self.id)
    }

    // Returns the URL of the Chrome DevTools UI.
    pub fn devtools_url(&self) -> String {
        format!(
            "devtools://devtools/bundled/js_app.html?experiments=true&v8only=true&ws={}",
            self.url()
        )
    }
}

async fn serve(state: AppState) {
    // Bind to specified address using hyper.
    let address = state.address.clone();
    let listener = TcpListener::bind(address).await.unwrap();

    // Build our application with some routes.
    let app = Router::new()
        .route(&format!("/{}", &state.id), get(root))
        .route("/json", get(json))
        .route("/json/list", get(json))
        .route("/json/version", get(json_version))
        .with_state(state);

    // Start listening for connections.
    axum::serve(listener, app).await.unwrap();
}

/// Route to attach the CDP websocket debugger.
async fn root(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    // Finalize the upgrade process by returning upgrade callback.
    ws.on_upgrade(move |socket| websocket(socket, state))
}

/// A list of all available websocket targets.
async fn json(State(state): State<AppState>) -> Json<Vec<Details>> {
    Json(vec![Details {
        description: "dune".into(),
        devtools_frontend_url: state.devtools_url(),
        id: state.id.to_string(),
        title: format!("dune [pid: {}]", std::process::id()),
        type_: "node".into(),
        url: "TBD".into(),
        web_socket_debugger_url: format!("ws://{}", state.url()),
    }])
}

/// Browser version metadata.
async fn json_version() -> Json<Versions> {
    Json(Versions {
        browser: format!("Dune/{}", env!("CARGO_PKG_VERSION")),
        protocol: "1.3".into(),
        v8: v8::V8::get_version().into(),
    })
}

// This function deals with a single websocket connection, for which we will
// spawn two independent tasks (for receiving / sending CDP messages).
async fn websocket(socket: WebSocket, state: AppState) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = socket.split();

    let inbound_tx = state.inbound_tx.clone();
    let mut outbound_tx = state.outbound_tx.subscribe();

    // Spawn the task that listens for devtools frontend messages.
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(data))) = receiver.next().await {
            // Wake up the event-loop if necessary.
            inbound_tx.send(data.clone()).unwrap();
            state.handle.interrupt();
            // Notify main thread that a debugger is attached and ready.
            if data.contains("Runtime.runIfWaitingForDebugger") {
                state.sessions_tx.send(()).unwrap();
            }
        }
    });

    // Spawn the task that sends messages to devtools frontend.
    let mut send_task = tokio::spawn(async move {
        while let Ok(message) = outbound_tx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(message)).await.is_err() {
                break;
            }
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }
}

fn new_box_with<T>(new_fn: impl FnOnce(*mut T) -> T) -> Box<T> {
    let b = Box::new(MaybeUninit::<T>::uninit());
    let p = Box::into_raw(b) as *mut T;
    // SAFETY: memory layout for `T` is ensured on first line of this function
    unsafe {
        std::ptr::write(p, new_fn(p));
        Box::from_raw(p)
    }
}
