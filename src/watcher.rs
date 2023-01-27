use crate::modules::load_import;
use anyhow::bail;
use anyhow::Result;
use clap::ArgMatches;
use colored::*;
use notify::event::DataChange;
use notify::event::ModifyKind;
use notify::Config;
use notify::Event;
use notify::EventHandler;
use notify::EventKind;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

const WATCH_EXTENSIONS: [&str; 4] = ["js", "jsx", "ts", "tsx"];

struct WatcherHandler {
    // The sending-half of Rust’s asynchronous channel type.
    tx: mpsc::Sender<PathBuf>,
    // A hashmap that keeps track of event timestamps.
    records: HashMap<PathBuf, Instant>,
}

impl EventHandler for WatcherHandler {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        let event = event.unwrap();
        let path = event.paths.get(0).unwrap().to_owned();
        let path_ext = path.extension().unwrap_or_default().to_str().unwrap();

        // We only care to monitor files with specific extensions.
        let extension = WATCH_EXTENSIONS.iter().any(|ext| &path_ext == ext);

        // Filter out uninterested events and files.
        if !(event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Content))
            || event.kind == EventKind::Modify(ModifyKind::Any)
            || extension)
        {
            return;
        }

        // HACKY: Some times we receive duplicate events. To counter that we'll accept
        // change events with more than 250ms time difference.
        match self.records.get_mut(&path) {
            Some(instant) => {
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
pub fn start(script: &str, arguments: ArgMatches) -> Result<()> {
    // Check if entry point is a local file.
    let windows_regex = Regex::new(r"^[a-zA-Z]:\\").unwrap();

    if !(script.starts_with('/') || windows_regex.is_match(script)) {
        bail!("Watch mode is only available for local files as entry point.");
    }

    // Check if the script exists in the file-system.
    if let Err(e) = load_import(script, true) {
        bail!(e.to_string());
    }

    // Get the paths we need to add a watcher on.
    let watch_paths: Vec<_> = arguments.get_many::<String>("watch").unwrap().collect();

    // Remove the `--watch` CLI arguments.
    let mut args = env::args()
        .skip(3)
        .filter(|arg| !arg.starts_with("--watch"))
        .filter(|arg| !arg.starts_with("--watch="))
        .filter(|arg| !watch_paths.iter().any(|path| *path == arg))
        .collect::<Vec<String>>();

    args.insert(0, "run".into());
    args.insert(1, script.into());

    let (sender, receiver) = mpsc::channel::<PathBuf>();

    // Create an appropriate watcher for the current system.
    let mut watcher = RecommendedWatcher::new(
        WatcherHandler {
            tx: sender,
            records: HashMap::default(),
        },
        Config::default().with_compare_contents(true),
    )
    .unwrap();

    if watch_paths.is_empty() {
        // Start watching the current working dir.
        println!("{}", "[dune] watching dir(s): *.*".yellow());
        match watcher.watch(Path::new("."), RecursiveMode::Recursive) {
            Ok(_) => {}
            Err(e) => bail!(e),
        }
    } else {
        // Start watching requested paths.
        for path in watch_paths.clone() {
            match watcher.watch(Path::new(path), RecursiveMode::Recursive) {
                Ok(_) => {}
                Err(e) => bail!(e),
            }
        }

        println!(
            "{}",
            format!(
                "[dune] watching dir(s): {}",
                watch_paths
                    .iter()
                    .map(|path| (*path).to_owned())
                    .collect::<Vec<String>>()
                    .join(", ")
            )
            .yellow()
        );
    }

    let exe = std::env::current_exe().unwrap();
    let extension = if cfg!(windows) { "exe" } else { "" };

    'outer: loop {
        // Run the main script as a child process.
        let mut process = match Command::new(exe.with_extension(extension))
            .args(&args)
            .spawn()
        {
            Ok(process) => process,
            Err(e) => bail!(e),
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

            // Check if the child process has been terminated.
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

    Ok(())
}
