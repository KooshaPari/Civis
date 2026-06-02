use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{
    run_build,
    run_screenshot,
    CensusData,
    CliError,
    CliResult,
    find_latest_run_log,
    parse_census_text,
    workspace_root,
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

fn run_robocopy_with_mirror_assets(root: &Path) -> CliResult<i32> {
    let status = Command::new("robocopy")
        .arg("assets")
        .arg(root.join("dist/assets"))
        .current_dir(root)
        .args(["/MIR", "/XF", "*.orig"])
        .status()?;
    let code = status.code().unwrap_or(1);
    if code >= 8 {
        return Err(CliError::new(
            code,
            "robocopy exited with failure code (>=8)",
        ));
    }
    Ok(code)
}

fn has_compiler_error(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("error:") || lower.contains("error[")
}

pub fn run_verify(target_dir: &Path, out: &Path) -> CliResult<VerifyResult> {
    let root = workspace_root();
    let built = run_build(target_dir)?;

    let dist = root.join("dist");
    let exe = dist.join("Civis.exe");
    fs::copy(&built, &exe)?;

    run_robocopy_with_mirror_assets(&root)?;

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
        return Err(CliError::new(
            1,
            format!("civ-panic.log detected:\n{tail}"),
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
