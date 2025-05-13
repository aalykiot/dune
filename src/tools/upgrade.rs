use anyhow::Result;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use std::fs;
use std::process::Command;
use tempdir::TempDir;
use ureq::ResponseExt;

const RELEASE_URL: &str = "https://github.com/aalykiot/dune/releases/latest/";
const RELEASE_TAG_PREFIX: &str = "https://github.com/aalykiot/dune/releases/tag/v";

pub fn run_upgrade() -> Result<()> {
    // Start the upgrade.
    println!("Looking up latest version");

    // Find dune's latest version.
    let response = ureq::get(RELEASE_URL).call()?;
    let version = response
        .get_uri()
        .to_string()
        .replace(RELEASE_TAG_PREFIX, "");

    // Check if latest version is already installed.
    if env!("CARGO_PKG_VERSION") == version {
        println!("Latest version {version} is already installed");
        std::process::exit(0);
    }

    println!("Found latest version {version}");

    let archive = format!("dune-{}.zip", env!("TARGET"));
    let download_url =
        format!("https://github.com/aalykiot/dune/releases/download/v{version}/{archive}");

    println!("Downloading {download_url}");

    let mut response = ureq::get(&download_url).call()?;

    let total_bytes = response
        .headers()
        .get("content-length")
        .and_then(|len| len.to_str().ok())
        .and_then(|len| len.parse::<u64>().ok())
        .unwrap_or(0);

    // Display a progress bar when downloading for better UX.
    let pb = ProgressBar::new(total_bytes);
    let pb_template =
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes}";

    pb.set_length(total_bytes);
    pb.set_style(
        ProgressStyle::with_template(pb_template)
            .unwrap()
            .progress_chars("#>-"),
    );

    // Create temp folder to download new archive.
    let temp_dir = TempDir::new("dune_zip_binary")?;
    let archive = format!("dune-{}.zip", env!("TARGET"));

    // Download new binary.
    let reader = response.body_mut().as_reader();
    let mut binary = fs::File::create_new(temp_dir.path().join(&archive))?;
    std::io::copy(&mut pb.wrap_read(reader), &mut binary)?;

    pb.finish_and_clear();

    let exe_name = "dune";
    let exe_extension = if cfg!(windows) { "exe" } else { "" };

    println!("Dune is upgrading to version {version}");

    // Unzip archive based on specific platform.
    if cfg!(windows) {
        Command::new("powershell.exe")
            .arg("-NoLogo")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg("Expand-Archive")
            .arg("-Path")
            .arg(format!("'{}'", temp_dir.path().join(archive).display()))
            .arg("-DestinationPath")
            .arg(format!("'{}'", temp_dir.path().display()))
            .output()?;
    } else {
        Command::new("unzip")
            .current_dir(temp_dir.path())
            .arg(temp_dir.path().join(archive))
            .output()?;
    }

    let next = temp_dir.path().join(exe_name).with_extension(exe_extension);
    let current = std::env::current_exe()?;

    if cfg!(windows) {
        // On windows you cannot replace the currently running executable.
        // so first we rename it to dune.outdated.exe
        fs::rename(&current, current.with_extension("outdated.exe"))?;
    }

    // Copy new binary to dune's execution path.
    fs::copy(next, &current)?;

    Ok(())
}
