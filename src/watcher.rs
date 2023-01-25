use colored::*;
use notify::event::DataChange;
use notify::event::ModifyKind;
use notify::Config;
use notify::Event;
use notify::EventHandler;
use notify::EventKind;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Result;
use notify::Watcher;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

const WATCH_EXTENSIONS: [&str; 4] = ["js", "jsx", "ts", "tsx"];

struct WatcherHandler {
    // The sending-half of Rust’s asynchronous channel type.
    tx: mpsc::Sender<PathBuf>,
    // A hashmap that keep tracks event timestamps.
    records: HashMap<PathBuf, Instant>,
}

impl EventHandler for WatcherHandler {
    fn handle_event(&mut self, event: Result<Event>) {
        let event = event.unwrap();
        let path = event.paths.get(0).unwrap().to_owned();
        let path_ext = path.extension().unwrap_or_default().to_str().unwrap();

        // We only care to monitor files with specific extensions.
        let extension = WATCH_EXTENSIONS.iter().any(|ext| &path_ext == ext);

        // Filter out uninterested events and files.
        if event.kind != EventKind::Modify(ModifyKind::Data(DataChange::Content)) || !extension {
            return;
        }

        match self.records.get_mut(&path) {
            Some(instant) => {
                // HACKY: Some times we receive duplicate events. To counter that we'll accept
                // change events with more than 250ms time difference.
                if Instant::now() - *instant > Duration::from_millis(250) {
                    *instant = Instant::now();
                    self.tx.send(path).unwrap();
                }
            }
            None => {
                self.records.insert(path.clone(), Instant::now());
                self.tx.send(path).unwrap();
            }
        }
    }
}

/// Starts the file-system watcher.
pub fn start() {
    // Remove the `--watch` CLI argument.
    let args: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| *arg != "--watch")
        .collect();

    let (sender, receiver) = mpsc::channel::<PathBuf>();

    // Create an FS watcher recommended for the system.
    let mut watcher = RecommendedWatcher::new(
        WatcherHandler {
            tx: sender,
            records: HashMap::default(),
        },
        Config::default().with_compare_contents(true),
    )
    .unwrap();

    println!("{}", "[dune] watching dir(s): *.*".yellow());

    // Start watching the current working dir.
    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .unwrap();

    'outer: loop {
        // Spawn the child process.
        let mut process = match std::process::Command::new("dune").args(&args).spawn() {
            Ok(process) => process,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };

        loop {
            // Check if we have a file change to handle.
            if let Ok(path) = receiver.recv_timeout(Duration::from_millis(250)) {
                println!("{}", "[dune] file change detected!".green());
                println!("[dune] {}", path.display());
                println!("{}", "[dune] restarting...".green());
                process.kill().unwrap();
                continue 'outer;
            }

            // Check if the process has been terminated.
            if let Ok(Some(status)) = process.try_wait() {
                match status.code() {
                    // Process exited with error.
                    Some(1) => {
                        println!(
                            "{}",
                            "[dune] process exited with error, restarting on file change...".red()
                        );
                        receiver.recv().unwrap();
                        continue 'outer;
                    }
                    // Process exited (probably) successfully.
                    _ => break 'outer,
                }
            }
        }
    }
}
