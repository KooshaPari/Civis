use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{workspace_root, CliError, CliResult};
use crate::proc::kill_competing_processes;

pub fn run_build(target_dir: &Path) -> CliResult<PathBuf> {
    kill_competing_processes()?;

    let cargo_root = workspace_root();
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "civ-bevy-ref",
            "--features",
            "voxel,models,egui",
            "--bin",
            "civ-standalone",
            "--release",
        ])
        .current_dir(cargo_root)
        .env("CARGO_TARGET_DIR", target_dir)
        .status()?;

    let code = status.code().unwrap_or(1);
    if code != 0 {
        return Err(CliError::new(code, "cargo build failed"));
    }

    Ok(target_dir.join("release").join("civ-standalone.exe"))
}

