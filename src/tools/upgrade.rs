use anyhow::Result;
use std::fs;
use std::io::Read;
use std::process::Command;
use tempdir::TempDir;

const RELEASE_URL: &str = "https://github.com/aalykiot/dune/releases/latest/";

pub fn run_upgrade() -> Result<()> {
    // Start the upgrade.
    println!("Looking up latest version");

    // Find dune's latest version.
    let response = ureq::get(RELEASE_URL).call()?;
    let version = response
        .get_url()
        .replace("https://github.com/aalykiot/dune/releases/tag/v", "");

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

    // Download the new binary.
    let response = ureq::get(&download_url).call()?;

    let mut binary = vec![];
    response.into_reader().read_to_end(&mut binary)?;

    // Get handles to temp and home directories.
    let temp_dir = TempDir::new("dune_zip_binary")?;

    // Write binary to disk.
    fs::write(temp_dir.path().join(&archive), binary)?;

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
