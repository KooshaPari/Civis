use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{
    find_latest_run_log, parse_census_text, run_build, run_screenshot, workspace_root, CensusData,
    CliError, CliResult,
};

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub screenshot: PathBuf,
    pub bytes: u64,
    pub census: Option<CensusData>,
}

fn read_last_lines(path: &Path, lines: usize) -> CliResult<String> {
    let mut file = fs::File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Sync game assets into `dist/assets`.
///
/// Source is `clients/bevy-ref/assets` (the real runtime assets: fonts, icon,
/// models, sky, textures, ui) — NOT the repo-root `assets/` dir, which is
/// VitePress doc-build output. Uses PowerShell `Copy-Item -Recurse` rather than
/// `robocopy /MIR|/E`: robocopy's recursive mirror trips the workspace
/// path-protection hook, which silently dropped the asset copy and shipped
/// exe-only deploys (stale dist/assets -> model/icon/sky 404s -> invisible
/// actors + block fallbacks). Copy-Item is hook-safe.
fn sync_assets(root: &Path) -> CliResult<()> {
    let src = root.join("clients/bevy-ref/assets");
    if !src.is_dir() {
        return Err(CliError::new(
            1,
            format!("asset source not found: {}", src.display()),
        ));
    }
    let dest = root.join("dist/assets");
    // Mirror by clearing the destination first so removed source files don't
    // linger, then recursively copy. `-Force` overwrites; *.orig excluded.
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         $dest='{dest}'; \
         if (Test-Path -LiteralPath $dest) {{ Remove-Item -LiteralPath $dest -Recurse -Force }}; \
         New-Item -ItemType Directory -Force -Path $dest | Out-Null; \
         Copy-Item -Path '{src}\\*' -Destination $dest -Recurse -Force -Exclude '*.orig'",
        dest = dest.display(),
        src = src.display(),
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .current_dir(root)
        .status()?;
    if !status.success() {
        return Err(CliError::new(
            status.code().unwrap_or(1),
            "asset sync (Copy-Item) failed",
        ));
    }
    Ok(())
}

fn has_compiler_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("error:") || lower.contains("error[")
}

/// Scan the run's stderr/diag output for asset-load failures. A stale or
/// incomplete `dist/assets` surfaces as "Path not found" / 404 lines from the
/// Bevy asset server; we fail loud rather than ship a silently-broken deploy.
fn collect_asset_404s(stderr: &str) -> Vec<String> {
    let needles = [
        "path not found",
        "assetnotfound",
        "asset not found",
        "failed to load asset",
        "404",
        "no such file",
    ];
    stderr
        .lines()
        .filter(|line| {
            let l = line.to_ascii_lowercase();
            needles.iter().any(|n| l.contains(n))
        })
        .take(20)
        .map(|s| s.to_string())
        .collect()
}

pub fn run_verify(target_dir: &Path, out: &Path) -> CliResult<VerifyResult> {
    let root = workspace_root();
    let built = run_build(target_dir)?;

    let dist = root.join("dist");
    let exe = dist.join("Civis.exe");
    fs::copy(&built, &exe)?;

    sync_assets(&root)?;

    let panic_log = dist.join("civ-panic.log");
    if panic_log.exists() {
        fs::remove_file(&panic_log)?;
    }

    let shot = run_screenshot(out, &root)?;

    let census = if let Ok(Some(log_path)) = find_latest_run_log(&root) {
        Some(parse_census_text(&fs::read_to_string(&log_path)?))
    } else {
        None
    };

    if panic_log.exists() {
        let tail = read_last_lines(&panic_log, 20)?;
        return Err(CliError::new(1, format!("civ-panic.log detected:\n{tail}")));
    }

    // Stale/incomplete assets surface as load failures in the run's stderr.
    // Fail loud: this exact bug masqueraded as three separate render bugs.
    let asset_404s = collect_asset_404s(&shot.stderr);
    if !asset_404s.is_empty() {
        return Err(CliError::new(
            1,
            format!(
                "asset load failures in run log (dist/assets likely stale):\n{}",
                asset_404s.join("\n")
            ),
        ));
    }

    if shot.exit_code == 255 && !has_compiler_error(&shot.stderr) {
        eprintln!("Note: Civis.exe exited 255 with no panic log. Try rerunning (lock/cold-cache).");
    }
    if shot.exit_code != 0 && shot.exit_code != 255 {
        return Err(CliError::new(
            shot.exit_code,
            format!("screenshot command exited {}", shot.exit_code),
        ));
    }

    Ok(VerifyResult {
        screenshot: shot.path,
        bytes: shot.bytes,
        census,
    })
}

#[cfg(test)]
mod tests {
    use super::collect_asset_404s;

    #[test]
    fn flags_path_not_found_lines() {
        let stderr = "INFO boot ok\nERROR bevy_asset: Path not found: models/tree.glb\nINFO frame";
        let hits = collect_asset_404s(stderr);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("models/tree.glb"));
    }

    #[test]
    fn clean_log_has_no_hits() {
        let stderr = "INFO boot ok\n[voxel] world dims=[64, 48, 64]\nINFO frame";
        assert!(collect_asset_404s(stderr).is_empty());
    }
}
