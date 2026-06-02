use std::process::Command;

use crate::{CliError, CliResult};

const PROCESS_NAMES: [&str; 5] = ["civ-standalone", "civ-bevy-ref", "civis", "cargo", "rustc"];

pub fn kill_competing_processes() -> CliResult<()> {
    for name in PROCESS_NAMES {
        if let Err(err) = kill_by_image_name(name) {
            return Err(err);
        }
    }
    Ok(())
}

#[cfg(windows)]
fn kill_by_image_name(name: &str) -> CliResult<()> {
    let image = format!("{name}.exe");
    let output = Command::new("taskkill")
        .args(["/F", "/IM", &image])
        .output()?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    if stderr.contains("could not find") || stderr.contains("not found") || output.status.code() == Some(128) {
        return Ok(());
    }

    Err(CliError::new(
        output.status.code().unwrap_or(1),
        format!("failed to kill {name}"),
    ))
}

#[cfg(not(windows))]
fn kill_by_image_name(name: &str) -> CliResult<()> {
    let _ = name;
    Ok(())
}

