// Bindings to the V8 Inspector API
// http://hyperandroid.com/2020/02/12/v8-inspector-from-an-embedder-standpoint/

use std::net::SocketAddrV4;
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;
use warp::Filter;

pub struct Inspector {
    address: SocketAddrV4,
    handle: Option<JoinHandle<()>>,
    inbound: mpsc::Sender<()>,
    outbound: mpsc::Receiver<()>,
}

impl Inspector {
    pub fn new(address: SocketAddrV4) -> Inspector {
        let (inbound, outbound) = mpsc::channel();
        Self {
            address,
            handle: None,
            inbound,
            outbound,
        }
    }

    pub fn start(&mut self) {
        let address = self.address.clone();
        let inbound_rc = self.inbound.clone();
        let handle = thread::spawn(move || executor().block_on(serve(address, inbound_rc)));
        self.handle = Some(handle);
    }
}

fn executor() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
}

pub async fn serve(address: SocketAddrV4, _inbound: mpsc::Sender<()>) {
    let root = warp::path("json").map(|| format!("OK"));
    println!("Debugger listening on {}", address);
    warp::serve(root).run(address).await;
}
