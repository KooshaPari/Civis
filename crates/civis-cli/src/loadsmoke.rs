//! Native load-and-continue smoke harness (`civis-loadsmoke`).
//!
//! Closes the parent-scorecard gap that flagged "no native acceptance
//! evidence". This module lets an agent (or CI) load a `.civsave.zst`
//! or `.civsave/` directory, advance the simulation by N ticks, and
//! emit a structured JSON receipt that downstream tools (or humans) can
//! diff against expectations.
//!
//! ## Why a separate bin
//!
//! The existing `civis-verify` exercises the windowed Bevy renderer;
//! `civis-dump` validates a CIVIS_DUMP JSON. Neither covers the
//! **engine-only load + continue** path, which is what "native
//! acceptance" means for the persistence/replay lane — the ability to
//! open a saved game on disk and keep playing it without a server, a
//! REPL, or a windowed renderer in the loop.
//!
//! ## Receipt shape
//!
//! `LoadSmokeReceipt` is a `Serialize` struct with:
//! - the loaded tick + format metadata,
//! - the tick range we advanced,
//! - the SHA-256 hex of the final state hash (so a regression diff is
//!   one line in the report),
//! - the size + path of the input file,
//! - and a `passed` boolean so CI can gate on the binary.
//!
//! Everything here is pure logic — `run_loadsmoke` takes owned paths
//! and returns a typed receipt; the bin handles argv + JSON printing.

use std::fs;
use std::path::{Path, PathBuf};

use civ_engine::save_bundle::{
    CivSaveBundle, CivSaveMetadata, SaveBundleError, CIVSAVE_FORMAT_VERSION,
};
use civ_engine::Simulation;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Structured receipt emitted by `civis-loadsmoke`.
///
/// Stable shape — agents and CI should diff on these fields by name,
/// not by position. Adding a field is backwards-compatible; removing
/// or renaming one is breaking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadSmokeReceipt {
    /// Harness library version (mirrors `civis_cli::HARNESS_VERSION`).
    pub harness_version: String,
    /// Path the load was attempted from (canonicalised).
    pub input_path: String,
    /// Detected input shape: `"archive"` (`.civsave.zst`) or `"dir"`.
    pub input_kind: String,
    /// File size in bytes (archive only — dirs report 0).
    pub input_byte_size: u64,
    /// Metadata parsed from `metadata.json` inside the bundle.
    pub metadata: CivSaveMetadata,
    /// Tick the simulation was at immediately after load.
    pub loaded_tick: u64,
    /// Tick after the post-load advance.
    pub final_tick: u64,
    /// Number of ticks advanced by the smoke (post-load).
    pub advanced_ticks: u32,
    /// Hex-encoded SHA-256 of the deterministic state hash at the
    /// final tick. Stable across runs with the same seed + actions.
    pub final_state_hash: String,
    /// Path of the saved archive (if `--save-out` was provided).
    pub output_archive_path: Option<String>,
    /// Byte size of the saved archive (if `--save-out` was provided).
    pub output_byte_size: Option<u64>,
    /// Engine state fields captured post-load + post-advance for
    /// machine-readable cross-version compatibility checks.
    pub state_summary: StateSummary,
    /// Pass/fail summary — `false` on any non-recoverable error.
    pub passed: bool,
    /// Human-readable error message when `passed == false`.
    pub error: Option<String>,
}

/// Aggregate simulation scalars — written to the receipt so a regression
/// diff is human-readable without unpacking the bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateSummary {
    /// `Simulation::state.tick`.
    pub tick: u64,
    /// `Simulation::state.population`.
    pub population: u64,
    /// `Simulation::state.belief`.
    pub belief: u64,
    /// `Simulation::state.cohesion`.
    pub cohesion: u64,
    /// `Simulation::state.unrest`.
    pub unrest: u64,
}

/// User-facing options to `run_loadsmoke`.
///
/// Defaults match a no-op smoke (`--ticks 0`); the bin surfaces
/// `--ticks` and `--save-out` via clap.
#[derive(Debug, Clone)]
pub struct LoadSmokeOptions {
    /// Path to the `.civsave.zst` archive or `.civsave/` folder.
    pub input_path: PathBuf,
    /// Number of ticks to advance after loading.
    pub advanced_ticks: u32,
    /// Optional output path — if set, the post-load simulation is
    /// written to this `.civsave.zst` archive as well.
    pub save_out: Option<PathBuf>,
    /// Harness version string written to the receipt.
    pub harness_version: String,
}

/// Run the load-and-continue smoke and produce a receipt.
///
/// Pure logic — no CLI parsing, no JSON printing. The caller is free
/// to serialize the receipt however they like. On error, the returned
/// receipt has `passed: false` and an `error` message; this is the
/// contract CI gates on.
pub fn run_loadsmoke(options: &LoadSmokeOptions) -> LoadSmokeReceipt {
    let input = &options.input_path;
    let metadata = match read_metadata_or_default(input) {
        Ok(meta) => meta,
        Err(err) => {
            return LoadSmokeReceipt::failure(
                options,
                0,
                format!("metadata read failed: {err}"),
            );
        }
    };

    let byte_size = if metadata_is_archive_kind(input) {
        fs::metadata(input).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let loaded_tick = metadata.tick;

    let mut sim = match CivSaveBundle::load(input) {
        Ok(sim) => sim,
        Err(err) => {
            return LoadSmokeReceipt::failure(
                options,
                loaded_tick,
                format!("load failed: {err}"),
            );
        }
    };

    // The smoke's whole point is "after load, the engine still ticks
    // deterministically". Tick the requested number of times and
    // capture the deterministic state hash + a few stable scalars.
    advance(&mut sim, options.advanced_ticks);

    let final_tick = sim.state.tick;
    let state_hash = compute_state_hash(&sim);
    let state_summary = StateSummary::from(&sim);

    // If --save-out was provided, re-archive the loaded+advanced
    // simulation. This is the "save game still works after a load"
    // path that the parent chat flagged as unproven.
    let (output_archive_path, output_byte_size) = match options.save_out.as_ref() {
        Some(path) => match save_archive(path, &sim) {
            Ok(()) => {
                let size = fs::metadata(path).map(|m| m.len()).ok();
                (Some(path.display().to_string()), size)
            }
            Err(err) => {
                return LoadSmokeReceipt::failure(
                    options,
                    final_tick,
                    format!("save-out failed: {err}"),
                );
            }
        },
        None => (None, None),
    };

    LoadSmokeReceipt {
        harness_version: options.harness_version.clone(),
        input_path: input.display().to_string(),
        input_kind: if metadata_is_archive_kind(input) {
            "archive".to_string()
        } else {
            "dir".to_string()
        },
        input_byte_size: byte_size,
        metadata,
        loaded_tick,
        final_tick,
        advanced_ticks: options.advanced_ticks,
        final_state_hash: state_hash,
        output_archive_path,
        output_byte_size,
        state_summary,
        passed: true,
        error: None,
    }
}

/// Advance the simulation by `n` ticks. Exposed so tests can drive the
/// smoke without touching the filesystem.
pub fn advance(sim: &mut Simulation, n: u32) {
    for _ in 0..n {
        sim.tick();
    }
}

/// Compute a SHA-256 hex of the deterministic per-tick scalars. This is
/// the "stable fingerprint" the receipt publishes for cross-run diffs.
pub fn compute_state_hash(sim: &Simulation) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sim.state.tick.to_le_bytes());
    hasher.update(sim.state.population.to_le_bytes());
    hasher.update(sim.state.belief.to_le_bytes());
    hasher.update(sim.state.cohesion.to_le_bytes());
    hasher.update(sim.state.unrest.to_le_bytes());
    hasher.update(sim.state.energy_budget_joules.to_bits().to_le_bytes());
    hex::encode(hasher.finalize())
}

/// Resolve `metadata.json` from either an archive or directory input.
/// Returns a sentinel-filled `CivSaveMetadata` on read failure so the
/// caller can still build a `passed: false` receipt.
fn read_metadata_or_default(path: &Path) -> Result<CivSaveMetadata, SaveBundleError> {
    CivSaveBundle::read_metadata(path)
}

/// Detect whether the input is an archive or directory bundle.
fn metadata_is_archive_kind(path: &Path) -> bool {
    CivSaveBundle::is_save_archive(path)
}

/// Persist the simulation to a `.civsave.zst` archive. Returns the
/// canonical error type so callers can format it cleanly.
fn save_archive(path: &Path, sim: &Simulation) -> Result<(), SaveBundleError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| SaveBundleError::Io {
                path: parent.to_path_buf(),
                message: err.to_string(),
            })?;
        }
    }
    CivSaveBundle::save_archive(path, sim)
}

impl LoadSmokeReceipt {
    /// Build a `passed: false` receipt with an error message.
    ///
    /// Used by `run_loadsmoke` when load or save-out fails partway
    /// through. The metadata is filled with a sentinel so the receipt
    /// shape stays stable for CI diffs.
    fn failure(options: &LoadSmokeOptions, loaded_tick: u64, error: String) -> Self {
        let input = &options.input_path;
        let kind = if metadata_is_archive_kind(input) {
            "archive"
        } else {
            "dir"
        };
        let byte_size = if kind == "archive" {
            fs::metadata(input).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        Self {
            harness_version: options.harness_version.clone(),
            input_path: input.display().to_string(),
            input_kind: kind.to_string(),
            input_byte_size: byte_size,
            metadata: CivSaveMetadata {
                spec_id: "UNKNOWN".to_string(),
                format_version: CIVSAVE_FORMAT_VERSION,
                tick: loaded_tick,
                scenario_name: None,
            },
            loaded_tick,
            final_tick: loaded_tick,
            advanced_ticks: options.advanced_ticks,
            final_state_hash: String::new(),
            output_archive_path: None,
            output_byte_size: None,
            state_summary: StateSummary {
                tick: loaded_tick,
                population: 0,
                belief: 0,
                cohesion: 0,
                unrest: 0,
            },
            passed: false,
            error: Some(error),
        }
    }
}

impl StateSummary {
    /// Snapshot the deterministic scalars from a `Simulation`.
    ///
    /// Must be the only place these fields are read in the receipt —
    /// any future field addition goes here.
    fn from(sim: &Simulation) -> Self {
        Self {
            tick: sim.state.tick,
            population: sim.state.population,
            belief: sim.state.belief,
            cohesion: sim.state.cohesion,
            unrest: sim.state.unrest,
        }
    }
}

// -- Error helper --------------------------------------------------------

/// Convenience wrapper that turns a `SaveBundleError` into a string the
/// bin can write to stderr.
#[must_use]
pub fn describe_save_error(err: &SaveBundleError) -> String {
    format!("{err}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn options(input: &Path, ticks: u32) -> LoadSmokeOptions {
        LoadSmokeOptions {
            input_path: input.to_path_buf(),
            advanced_ticks: ticks,
            save_out: None,
            harness_version: "test".to_string(),
        }
    }

    /// Build a deterministic archive fixture: tick `n0` times, save.
    fn build_fixture(dir: &Path, n0: u32) -> PathBuf {
        let mut sim = Simulation::with_seed(7);
        for _ in 0..n0 {
            sim.tick();
        }
        let path = dir.join("fixture.civsave.zst");
        CivSaveBundle::save_archive(&path, &sim).expect("save fixture");
        path
    }

    #[test]
    fn receipt_advances_deterministically_from_archive() {
        let dir = tempdir().expect("tempdir");
        let fixture = build_fixture(dir.path(), 5);
        let opts = options(&fixture, 3);

        let receipt = run_loadsmoke(&opts);

        assert!(receipt.passed, "smoke should pass: {:?}", receipt.error);
        assert_eq!(receipt.input_kind, "archive");
        assert!(receipt.input_byte_size > 0);
        assert_eq!(receipt.loaded_tick, 5);
        assert_eq!(receipt.advanced_ticks, 3);
        assert_eq!(receipt.final_tick, 8);
        assert_eq!(receipt.state_summary.tick, 8);
        // The hash is non-empty and 64 hex chars (SHA-256).
        assert_eq!(receipt.final_state_hash.len(), 64);
        assert!(receipt.error.is_none());
        assert!(receipt.output_archive_path.is_none());
    }

    #[test]
    fn receipt_is_deterministic_across_runs_same_seed() {
        let dir = tempdir().expect("tempdir");
        let fixture_a = build_fixture(dir.path(), 4);
        let fixture_b = build_fixture(dir.path(), 4);

        // The two fixtures are saved at the same tick — they should
        // hash to the same state after the same number of post-load
        // ticks because `Simulation::with_seed(7)` is deterministic.
        let opts_a = options(&fixture_a, 2);
        let opts_b = options(&fixture_b, 2);
        let receipt_a = run_loadsmoke(&opts_a);
        let receipt_b = run_loadsmoke(&opts_b);

        assert!(receipt_a.passed);
        assert!(receipt_b.passed);
        assert_eq!(receipt_a.final_state_hash, receipt_b.final_state_hash);
        assert_eq!(receipt_a.state_summary.population, receipt_b.state_summary.population);
    }

    #[test]
    fn receipt_save_out_writes_a_new_archive() {
        let dir = tempdir().expect("tempdir");
        let fixture = build_fixture(dir.path(), 2);
        let save_out = dir.path().join("out.civsave.zst");
        let opts = LoadSmokeOptions {
            input_path: fixture,
            advanced_ticks: 7,
            save_out: Some(save_out.clone()),
            harness_version: "test".to_string(),
        };

        let receipt = run_loadsmoke(&opts);

        assert!(receipt.passed, "smoke should pass: {:?}", receipt.error);
        assert!(save_out.exists(), "save_out archive should exist");
        assert_eq!(
            receipt.output_archive_path.as_deref(),
            Some(save_out.display().to_string().as_str())
        );
        assert!(receipt.output_byte_size.unwrap_or(0) > 0);

        // Roundtrip: the new archive must also load successfully.
        let reloaded = CivSaveBundle::load(&save_out).expect("reload save_out");
        assert_eq!(reloaded.state.tick, receipt.final_tick);
    }

    #[test]
    fn receipt_failure_on_missing_input() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist.civsave.zst");
        let opts = options(&missing, 1);

        let receipt = run_loadsmoke(&opts);

        assert!(!receipt.passed);
        assert!(receipt.error.is_some());
        assert_eq!(receipt.advanced_ticks, 1);
    }

    #[test]
    fn receipt_zero_ticks_is_a_valid_no_op_smoke() {
        let dir = tempdir().expect("tempdir");
        let fixture = build_fixture(dir.path(), 6);
        let opts = options(&fixture, 0);

        let receipt = run_loadsmoke(&opts);

        assert!(receipt.passed);
        assert_eq!(receipt.advanced_ticks, 0);
        assert_eq!(receipt.loaded_tick, 6);
        assert_eq!(receipt.final_tick, 6);
    }

    #[test]
    fn compute_state_hash_changes_between_ticks() {
        let dir = tempdir().expect("tempdir");
        let fixture = build_fixture(dir.path(), 10);
        let opts_zero = options(&fixture, 0);
        let opts_one = options(&fixture, 1);

        let r_zero = run_loadsmoke(&opts_zero);
        let r_one = run_loadsmoke(&opts_one);

        assert_ne!(
            r_zero.final_state_hash, r_one.final_state_hash,
            "state hash must change when the simulation advances"
        );
    }

    #[test]
    fn advance_helper_increments_tick() {
        let mut sim = Simulation::with_seed(2);
        assert_eq!(sim.state.tick, 0);
        advance(&mut sim, 5);
        assert_eq!(sim.state.tick, 5);
    }
}
