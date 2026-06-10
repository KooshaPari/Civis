use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::{CliError, CliResult};

/// Wall-clock grace to wait for the autoshot PNG to land on disk after the game
/// process returns. The standalone now uses a wall-clock warmup (not Time::delta)
/// and self-exits ~1.5s after capture, but `save_to_disk` flushes asynchronously,
/// so poll for the file instead of assuming it exists the instant the process
/// exits.
const SHOT_FLUSH_TIMEOUT: Duration = Duration::from_secs(10);

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

    // Poll for the PNG: the capture save is async, so the file may lag the
    // process exit by a few frames. Wait on file-exists (with a nonzero size)
    // up to SHOT_FLUSH_TIMEOUT before declaring failure.
    let deadline = Instant::now() + SHOT_FLUSH_TIMEOUT;
    loop {
        if let Ok(meta) = fs::metadata(&out) {
            if meta.len() > 0 {
                break;
            }
        }
        if Instant::now() >= deadline {
            return Err(CliError::new(
                1,
                format!(
                    "screenshot file not created within {}s: {}",
                    SHOT_FLUSH_TIMEOUT.as_secs(),
                    out.display()
                ),
            ));
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    let bytes = fs::metadata(&out)?.len();
    Ok(ScreenshotResult {
        path: out,
        bytes,
        exit_code: output.status.code().unwrap_or(1),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
