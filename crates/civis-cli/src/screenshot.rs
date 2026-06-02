use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{CliError, CliResult};

#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    pub path: PathBuf,
    pub bytes: u64,
    pub exit_code: i32,
    pub stderr: String,
}

pub fn run_screenshot(relative_out: &Path, root: &Path) -> CliResult<ScreenshotResult> {
    let dist = root.join("dist");
    let exe = dist.join("Civis.exe");
    if !exe.exists() {
        return Err(CliError::new(
            1,
            format!("missing executable {}", exe.display()),
        ));
    }

    let out = dist.join(relative_out);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| CliError::new(1, err.to_string()))?;
    }

    let output = Command::new(&exe)
        .current_dir(&dist)
        .env("CIVIS_AUTOSTART", "1")
        .env("CIVIS_AUTOSHOT", relative_out)
        .env("CIVIS_AUTOSHOT_WARMUP", "12")
        .output()?;

    if !out.exists() {
        return Err(CliError::new(
            1,
            format!("screenshot file not created {}", out.display()),
        ));
    }

    let bytes = fs::metadata(&out)?.len();
    Ok(ScreenshotResult {
        path: out,
        bytes,
        exit_code: output.status.code().unwrap_or(1),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
