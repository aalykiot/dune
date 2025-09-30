// Bindings to the V8 Inspector API.
//
// http://hyperandroid.com/2020/02/12/v8-inspector-from-an-embedder-standpoint/
// https://github.com/ahmadov/v8_inspector_example/tree/master/

use crate::errors::generic_error;
use crate::errors::unwrap_or_exit;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::extract::ws::WebSocketUpgrade;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use dune_event_loop::LoopInterruptHandle;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::Serialize;
use std::cell::Cell;
use std::cell::RefCell;
use std::net::SocketAddrV4;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tokio::sync::broadcast;
use uuid::Uuid;
use v8::inspector::ChannelImpl;
use v8::inspector::V8InspectorClientImpl;

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

#[derive(Debug)]
enum FrontendMessage {
    /// A new debugger session has been successfully connected.
    Connected,
    /// The current debugger session has been disconnected.
    Disconnected,
    /// A command message received from devtools.
    Command(String),
}

/// This structure is responsible for providing inspector
/// interface to the `JsRuntime`.
pub struct JsRuntimeInspector {
    v8_inspector: Rc<v8::inspector::V8Inspector>,
    handshake_rx: mpsc::Receiver<()>,
    pub state: Rc<InspectorState>,
    break_on_start: bool,
    handle: LoopInterruptHandle,
    root: Option<String>,
    outbound_tx: broadcast::Sender<InspectorMessage>,
    inbound_tx: mpsc::Sender<FrontendMessage>,
    handshake_tx: mpsc::Sender<()>,
}

impl JsRuntimeInspector {
    // Creates a JsRuntimeInspector instance.
    pub fn new(
        isolate: &mut v8::Isolate,
        context: v8::Global<v8::Context>,
        handle: LoopInterruptHandle,
        should_wait: bool,
        root: Option<String>,
    ) -> Rc<Self> {
        let (inbound_tx, inbound_rx) = mpsc::channel::<FrontendMessage>();
        let (handshake_tx, handshake_rx) = mpsc::channel::<()>();
        let (outbound_tx, _outbound_rx) = broadcast::channel::<InspectorMessage>(64);
        let state = Rc::new(InspectorState {
            v8_inspector: RefCell::new(None),
            inbound_rx,
            outbound_tx: outbound_tx.clone(),
            on_pause: Cell::new(false),
            waiting_for_session: Cell::new(should_wait),
            session: RefCell::new(None),
        });

        v8::scope_with_context!(scope, isolate, context.clone());

        let client = Box::new(InspectorClient(state.clone()));
        let v8_inspector_client = v8::inspector::V8InspectorClient::new(client);
        let v8_inspector = Rc::new(v8::inspector::V8Inspector::create(
            scope,
            v8_inspector_client,
        ));

        *state.v8_inspector.borrow_mut() = Some(v8_inspector.clone());

        // Tell the inspector about the global context.
        let context_name = v8::inspector::StringView::from("global context".as_bytes());
        let aux_data = v8::inspector::StringView::from(r#"{"isDefault": true}"#.as_bytes());
        let context = v8::Local::new(scope, context);

        v8_inspector.context_created(context, CONTEXT_GROUP_ID, context_name, aux_data);

        Rc::new(Self {
            v8_inspector,
            handshake_rx,
            state,
            break_on_start: should_wait,
            inbound_tx,
            handshake_tx,
            outbound_tx,
            handle,
            root,
        })
    }

    /// This function "blocks" the thread until at least one inspector client has
    /// established a handshake with the inspector. After that, it instructs V8
    /// to pause at the next statement.
    pub fn wait_for_session_and_break_on_next_statement(&self) {
        // We need to periodically wake up to allow V8 to respond
        // to incoming messages (before the handshake).
        let timeout = Duration::from_millis(200);

        loop {
            // We don't want a busy loop thus the timeout on channel recv.
            match self.handshake_rx.recv_timeout(timeout) {
                // Handshake established, pause execution.
                Ok(_) => {
                    self.state.poll_session();
                    self.state.break_on_next_statement();
                    break;
                }
                Err(_) => {
                    // Continue polling session for CDP messages.
                    self.state.poll_session();
                }
            }
        }
    }

    // Notify the inspector that the context is about to destroyed.
    pub fn context_destroyed(&self, scope: &mut v8::PinScope, ctx: v8::Global<v8::Context>) {
        // Get a local context reference.
        let context = v8::Local::new(scope, ctx);
        self.v8_inspector.context_destroyed(context);
    }

    // Starts listening for ws connections.
    pub fn start_agent(&self, address: SocketAddrV4) {
        // Build the shared state for axum.
        let state = AppState {
            id: Uuid::new_v4(),
            address,
            outbound_tx: self.outbound_tx.clone(),
            inbound_tx: self.inbound_tx.clone(),
            handle: self.handle.clone(),
            handshake_tx: self.handshake_tx.clone(),
            root: self.root.clone(),
        };

        // Build a single threaded tokio runtime.
        let executor = Builder::new_current_thread()
            .thread_name("dune-inspector-thread")
            .worker_threads(2)
            .enable_io()
            .build()
            .unwrap();

        // Spawn the web-socket server thread.
        thread::spawn(move || executor.block_on(serve(state)));

        if self.break_on_start {
            self.wait_for_session_and_break_on_next_statement();
        }
    }
}

pub struct InspectorState {
    v8_inspector: RefCell<Option<Rc<v8::inspector::V8Inspector>>>,
    inbound_rx: mpsc::Receiver<FrontendMessage>,
    outbound_tx: broadcast::Sender<InspectorMessage>,
    on_pause: Cell<bool>,
    waiting_for_session: Cell<bool>,
    session: RefCell<Option<InspectorSession>>,
}

impl InspectorState {
    /// Polls the debugger session for incoming messages from the frontend (devtools).
    pub fn poll_session(&self) {
        // Block the thread until a devtools message is received.
        if self.on_pause.get() || self.waiting_for_session.get() {
            let message = self.inbound_rx.recv().unwrap();
            self.process_incoming_message(message);
            return;
        }
        // Check for and process any pending devtools messages.
        while let Some(message) = self.inbound_rx.try_iter().next() {
            self.process_incoming_message(message);
        }
    }

    /// Processes the received messages, such as establishing or disconnecting a session
    /// and dispatching commands to the active session.
    fn process_incoming_message(&self, message: FrontendMessage) {
        match message {
            // Establish a new InspectorSession upon frontend connection.
            FrontendMessage::Connected => {
                self.waiting_for_session.set(false);
                *self.session.borrow_mut() = Some(InspectorSession::new(
                    self.v8_inspector.borrow().as_ref().unwrap().clone(),
                    self.outbound_tx.clone(),
                ));
            }
            // Drop the current session and perform clean-ups.
            FrontendMessage::Disconnected => {
                self.session.borrow_mut().take();
            }
            // Dispatch the received command to the active session.
            FrontendMessage::Command(data) => {
                if let Some(session) = self.session.borrow().as_ref() {
                    session.dispatch_message(data);
                }
            }
        }
    }

    /// Instructs V8 to pause at the next statement.
    fn break_on_next_statement(&self) {
        if let Some(session) = self.session.borrow_mut().as_ref() {
            session.break_on_next_statement();
        }
    }
}

struct InspectorClient(Rc<InspectorState>);

impl V8InspectorClientImpl for InspectorClient {
    fn run_message_loop_on_pause(&self, context_group_id: i32) {
        // Context id should always be the same.
        assert_eq!(context_group_id, CONTEXT_GROUP_ID);
        self.0.on_pause.set(true);
        // Poll session while we're on the "pause" state.
        while self.0.on_pause.get() {
            self.0.poll_session();
        }
    }

    fn quit_message_loop_on_pause(&self) {
        self.0.on_pause.set(false);
    }

    fn run_if_waiting_for_debugger(&self, context_group_id: i32) {
        assert_eq!(context_group_id, CONTEXT_GROUP_ID);
        self.0.waiting_for_session.set(false);
    }
}

/// An inspector session that proxies messages to concrete "transport layer",
/// like a websocket connection.
struct InspectorSession {
    v8_session: v8::inspector::V8InspectorSession,
}

impl InspectorSession {
    pub fn new(
        v8_inspector: Rc<v8::inspector::V8Inspector>,
        outbound_tx: broadcast::Sender<InspectorMessage>,
    ) -> Self {
        let state = InspectorSessionState {
            outbound_tx: outbound_tx.clone(),
        };
        let v8_session = v8_inspector.connect(
            CONTEXT_GROUP_ID,
            v8::inspector::Channel::new(Box::new(state)),
            v8::inspector::StringView::empty(),
            v8::inspector::V8InspectorClientTrustLevel::FullyTrusted,
        );

        Self { v8_session }
    }

    // Schedule a v8 break on next statement.
    pub fn break_on_next_statement(&self) {
        let reason = v8::inspector::StringView::from(&b"debugCommand"[..]);
        let details = v8::inspector::StringView::empty();
        self.v8_session
            .schedule_pause_on_next_statement(reason, details);
    }

    // Dispatch message to V8 session.
    pub fn dispatch_message(&self, msg: String) {
        let bytes = msg.as_bytes();
        let v8_message = v8::inspector::StringView::from(bytes);
        self.v8_session.dispatch_protocol_message(v8_message);
    }
}

struct InspectorSessionState {
    outbound_tx: broadcast::Sender<InspectorMessage>,
}

impl InspectorSessionState {
    // Dispatch message to outbound channel.
    fn send_message(&self, msg: v8::UniquePtr<v8::inspector::StringBuffer>) {
        let message = msg.unwrap().string().to_string();
        self.outbound_tx.send(message).unwrap();
    }
}

impl ChannelImpl for InspectorSessionState {
    fn send_response(&self, _call_id: i32, message: v8::UniquePtr<v8::inspector::StringBuffer>) {
        self.send_message(message);
    }

    fn send_notification(&self, message: v8::UniquePtr<v8::inspector::StringBuffer>) {
        self.send_message(message);
    }

    fn flush_protocol_notifications(&self) {}
}

#[derive(Clone)]
struct AppState {
    pub id: Uuid,
    pub address: SocketAddrV4,
    pub outbound_tx: broadcast::Sender<InspectorMessage>,
    pub inbound_tx: mpsc::Sender<FrontendMessage>,
    pub handshake_tx: mpsc::Sender<()>,
    pub handle: LoopInterruptHandle,
    pub root: Option<String>,
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
    // Bind to specified address, handle errors gracefully.
    let listener = TcpListener::bind(state.address).await;
    let listener = unwrap_or_exit(listener.map_err(|e| generic_error(e.to_string())));

    println!("Debugger listening on ws://{}/{}", state.address, state.id);
    println!("Visit chrome://inspect to connect to the debugger.");

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
        url: state.root.to_owned().unwrap_or_default(),
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

    let handle = state.handle.clone();
    let inbound_tx = state.inbound_tx.clone();
    let mut outbound_tx = state.outbound_tx.subscribe();

    // Notify that a debugger was attached.
    state.inbound_tx.send(FrontendMessage::Connected).unwrap();

    // Spawn the task that listens for devtools frontend messages.
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(data))) = receiver.next().await {
            // Wake up the event-loop if necessary.
            let data = data.to_string();
            let _ = inbound_tx.send(FrontendMessage::Command(data.clone()));
            handle.interrupt();

            // Notify main thread that a debugger is attached and ready.
            if data.contains("Runtime.runIfWaitingForDebugger") {
                state.handshake_tx.send(()).unwrap();
            }
        }
    });

    // Spawn the task that sends messages to devtools frontend.
    let mut send_task = tokio::spawn(async move {
        while let Ok(message) = outbound_tx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    });

    // If any one of the tasks completes, abort the other.
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    // Notify that the debugger detached.
    let _ = state.inbound_tx.send(FrontendMessage::Disconnected);
    state.handle.interrupt();
}
