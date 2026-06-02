use std::path::PathBuf;

use civ_engine::{CivSaveBundle, Simulation};

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug, Clone)]
pub struct CliError {
    pub message: String,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(exit_code: i32, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(value: std::io::Error) -> Self {
        Self::new(1, value.to_string())
    }
}

pub fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map_or(manifest_dir, |p| p.to_path_buf())
}

pub mod build;
pub mod census;
pub mod proc;
pub mod screenshot;
pub mod verify;

pub use build::run_build;
pub use census::{census_to_json, find_latest_run_log, parse_census_text, read_census_from_log, CensusData};
pub use screenshot::{run_screenshot, ScreenshotResult};
pub use verify::{run_verify, VerifyResult};

pub fn command_new_world(seed: u64, out: &std::path::Path) -> CliResult<serde_json::Value> {
    let sim = Simulation::with_seed(seed);
    CivSaveBundle::save_archive(out, &sim)
        .map_err(|err| CliError::new(1, format!("failed to write {}: {err}", out.display())))?;
    Ok(serde_json::json!({
        "command": "new-world",
        "seed": seed,
        "output": out,
        "tick": sim.state.tick,
    }))
}

pub fn command_inspect_save(path: &std::path::Path) -> CliResult<serde_json::Value> {
    let metadata = CivSaveBundle::read_metadata(path)
        .map_err(|err| CliError::new(1, format!("failed to read {}: {err}", path.display())))?;
    let sim = CivSaveBundle::load(path)
        .map_err(|err| CliError::new(1, format!("failed to load {}: {err}", path.display())))?;
    Ok(serde_json::json!({
        "command": "inspect-save",
        "path": path,
        "metadata": metadata,
        "state": {
            "tick": sim.state.tick,
            "population": sim.snapshot().population,
            "energy_budget_joules": sim.state.energy_budget_joules,
        },
    }))
}

pub fn command_bench(seed: u64, ticks: u64) -> CliResult<serde_json::Value> {
    let mut sim = Simulation::with_seed(seed);
    let start = std::time::Instant::now();
    for _ in 0..ticks {
        sim.tick();
    }
    let elapsed = start.elapsed();
    let per_tick_ns = elapsed.as_nanos() / u128::from(ticks.max(1));
    Ok(serde_json::json!({
        "command": "bench",
        "seed": seed,
        "ticks": ticks,
        "elapsed_ms": elapsed.as_secs_f64() * 1000.0,
        "per_tick_ns": per_tick_ns,
        "final_tick": sim.state.tick,
    }))
}
