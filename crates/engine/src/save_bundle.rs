//! Uncompressed `.civsave/` folder stub (CIV-1000 §2.2 debug layout).

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ModGuestStateSave, ReplayError, Simulation};

/// Sidecar metadata written beside replay + mod state.
pub const CIVSAVE_SPEC_ID: &str = "CIV-1000";
/// Folder format version for `metadata.json`.
pub const CIVSAVE_FORMAT_VERSION: u32 = 1;

/// `metadata.json` in a `.civsave/` directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivSaveMetadata {
    /// Spec identifier (`CIV-1000`).
    pub spec_id: String,
    /// Folder format version.
    pub format_version: u32,
    /// Engine tick at save time.
    pub tick: u64,
    /// Optional scenario label for UI.
    pub scenario_name: Option<String>,
}

/// Errors reading or writing save folders.
#[derive(Debug, Error)]
pub enum SaveBundleError {
    /// Filesystem failure.
    #[error("io at {path}: {message}")]
    Io {
        /// Path involved.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// JSON metadata or mod state failure.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Guest mod state import failure.
    #[error("mod state: {0}")]
    ModState(#[from] civ_mod_host::GuestStateError),
    /// Replay encode/decode failure.
    #[error("replay: {0}")]
    Replay(#[from] ReplayError),
    /// Missing required component in the folder.
    #[error("missing {component} in {dir}")]
    MissingComponent {
        /// Folder path.
        dir: PathBuf,
        /// Expected file name.
        component: &'static str,
    },
}

fn io_err(path: impl AsRef<Path>, err: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Io {
        path: path.as_ref().to_path_buf(),
        message: err.to_string(),
    }
}

/// CIV-1000 debug folder: `metadata.json`, `mod_state.json`, `replay.civreplay`.
pub struct CivSaveBundle;

impl CivSaveBundle {
    /// Write an uncompressed save folder at `dir` (typically `*.civsave/`).
    pub fn save_dir(dir: impl AsRef<Path>, sim: &Simulation) -> Result<(), SaveBundleError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;

        let metadata = CivSaveMetadata {
            spec_id: CIVSAVE_SPEC_ID.to_owned(),
            format_version: CIVSAVE_FORMAT_VERSION,
            tick: sim.state.tick,
            scenario_name: None,
        };
        let metadata_path = dir.join("metadata.json");
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .map_err(|e| io_err(&metadata_path, e))?;

        let mod_state_path = dir.join("mod_state.json");
        fs::write(&mod_state_path, sim.export_mod_guest_state().to_json()?)
            .map_err(|e| io_err(&mod_state_path, e))?;

        let replay_path = dir.join("replay.civreplay");
        sim.save_replay(&replay_path)?;
        Ok(())
    }

    /// Load simulation from a `.civsave/` folder.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let dir = dir.as_ref();
        let replay_path = dir.join("replay.civreplay");
        if !replay_path.is_file() {
            return Err(SaveBundleError::MissingComponent {
                dir: dir.to_path_buf(),
                component: "replay.civreplay",
            });
        }
        let mut sim = Simulation::load_replay_from_file(&replay_path)?;

        let mod_state_path = dir.join("mod_state.json");
        if mod_state_path.is_file() {
            let json =
                fs::read_to_string(&mod_state_path).map_err(|e| io_err(&mod_state_path, e))?;
            let save = ModGuestStateSave::from_json(&json)?;
            sim.restore_mod_guest_state(&save)?;
        }

        Ok(sim)
    }

    /// True when `path` is a directory containing `replay.civreplay`.
    #[must_use]
    pub fn is_save_dir(path: &Path) -> bool {
        path.is_dir() && path.join("replay.civreplay").is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn civsave_folder_round_trips_mod_guest_state() {
        let mut sim = Simulation::with_seed(9);
        for _ in 0..5 {
            sim.tick();
        }
        sim.mod_host_mut()
            .restore_guest_memory("test-mod", vec![4, 5, 6]);

        let dir = tempdir().expect("tempdir");
        let save_path = dir.path().join("slot_test");
        CivSaveBundle::save_dir(&save_path, &sim).expect("save");

        let loaded = CivSaveBundle::load_dir(&save_path).expect("load");
        assert_eq!(loaded.state.tick, sim.state.tick);
        assert_eq!(
            loaded.mod_host().guest_memory_snapshot("test-mod"),
            vec![4, 5, 6]
        );
    }
}
