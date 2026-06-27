//! CIV-1000 save layouts: uncompressed `.civsave/` folder (debug) and `.civsave.zst` archive (default).

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tar::{Archive, Builder};
use thiserror::Error;
use zstd::stream::{decode_all, encode_all};

use crate::{ModGuestStateSave, ReplayError, Simulation};

/// Sidecar metadata written beside replay + mod state.
pub const CIVSAVE_SPEC_ID: &str = "CIV-1000";
/// Folder format version for `metadata.json`.
pub const CIVSAVE_FORMAT_VERSION: u32 = 1;
/// Default on-disk save extension (zstd-compressed tar).
pub const CIVSAVE_ARCHIVE_EXTENSION: &str = "civsave.zst";

/// Zstd frame magic (little-endian `0xFD2FB528`).
const ZSTD_FRAME_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

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
    /// Tar archive read/write failure.
    #[error("archive: {0}")]
    Archive(String),
    /// Zstd compression failure.
    #[error("zstd: {0}")]
    Zstd(String),
}

fn io_err(path: impl AsRef<Path>, err: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Io {
        path: path.as_ref().to_path_buf(),
        message: err.to_string(),
    }
}

fn archive_err(message: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Archive(message.to_string())
}

fn zstd_err(message: impl std::fmt::Display) -> SaveBundleError {
    SaveBundleError::Zstd(message.to_string())
}

/// CIV-1000 save bundle: folder (debug) and `.civsave.zst` archive (default).
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

    /// Write a zstd-compressed tar archive at `path` (typically `*.civsave.zst`).
    pub fn save_archive(path: impl AsRef<Path>, sim: &Simulation) -> Result<(), SaveBundleError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
            }
        }

        let temp = tempfile::tempdir().map_err(|e| io_err(path, e))?;
        Self::save_dir(temp.path(), sim)?;
        let tar_bytes = tar_dir(temp.path())?;
        let compressed = encode_all(tar_bytes.as_slice(), 3).map_err(zstd_err)?;
        fs::write(path, compressed).map_err(|e| io_err(path, e))
    }

    /// Load simulation from a `.civsave.zst` archive.
    pub fn load_archive(path: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let path = path.as_ref();
        let compressed = fs::read(path).map_err(|e| io_err(path, e))?;
        let tar_bytes = decode_all(compressed.as_slice()).map_err(zstd_err)?;
        let temp = tempfile::tempdir().map_err(|e| io_err(path, e))?;
        extract_tar(tar_bytes.as_slice(), temp.path())?;
        Self::load_dir(temp.path())
    }

    /// Read `metadata.json` from a save folder or archive without loading the simulation.
    pub fn read_metadata(path: impl AsRef<Path>) -> Result<CivSaveMetadata, SaveBundleError> {
        let path = path.as_ref();
        if Self::is_save_dir(path) {
            let metadata_path = path.join("metadata.json");
            let json = fs::read_to_string(&metadata_path).map_err(|e| io_err(&metadata_path, e))?;
            Ok(serde_json::from_str(&json)?)
        } else if Self::is_save_archive(path) {
            let compressed = fs::read(path).map_err(|e| io_err(path, e))?;
            let tar_bytes = decode_all(compressed.as_slice()).map_err(zstd_err)?;
            read_metadata_from_tar(&tar_bytes)
        } else {
            Err(SaveBundleError::MissingComponent {
                dir: path.to_path_buf(),
                component: "metadata.json",
            })
        }
    }

    /// Load from either a `.civsave/` folder or a `.civsave.zst` archive.
    pub fn load(path: impl AsRef<Path>) -> Result<Simulation, SaveBundleError> {
        let path = path.as_ref();
        if Self::is_save_archive(path) {
            Self::load_archive(path)
        } else if Self::is_save_dir(path) {
            Self::load_dir(path)
        } else {
            Err(SaveBundleError::MissingComponent {
                dir: path.to_path_buf(),
                component: "replay.civreplay",
            })
        }
    }

    /// True when `path` is a directory containing `replay.civreplay`.
    #[must_use]
    pub fn is_save_dir(path: &Path) -> bool {
        path.is_dir() && path.join("replay.civreplay").is_file()
    }

    /// True when `path` is a `.civsave.zst` file with a zstd frame header.
    #[must_use]
    pub fn is_save_archive(path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        if path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zst"))
        {
            return true;
        }
        let Ok(mut file) = File::open(path) else {
            return false;
        };
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).is_ok() && magic == ZSTD_FRAME_MAGIC
    }
}

fn tar_dir(dir: &Path) -> Result<Vec<u8>, SaveBundleError> {
    let mut tar_buf = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_buf);
        for entry in fs::read_dir(dir).map_err(archive_err)? {
            let entry = entry.map_err(archive_err)?;
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| archive_err("non-utf8 file name in save dir"))?;
                builder
                    .append_path_with_name(&path, name)
                    .map_err(archive_err)?;
            }
        }
        builder.finish().map_err(archive_err)?;
    }
    Ok(tar_buf)
}

fn extract_tar(bytes: &[u8], dest: &Path) -> Result<(), SaveBundleError> {
    let mut archive = Archive::new(bytes);
    archive.unpack(dest).map_err(archive_err)
}

fn read_metadata_from_tar(bytes: &[u8]) -> Result<CivSaveMetadata, SaveBundleError> {
    use std::io::Read;
    let mut archive = Archive::new(bytes);
    for entry in archive.entries().map_err(archive_err)? {
        let mut entry = entry.map_err(archive_err)?;
        let entry_path = entry.path().map_err(archive_err)?;
        if entry_path.file_name().and_then(|s| s.to_str()) != Some("metadata.json") {
            continue;
        }
        let mut json = String::new();
        entry.read_to_string(&mut json).map_err(archive_err)?;
        return Ok(serde_json::from_str(&json)?);
    }
    Err(SaveBundleError::MissingComponent {
        dir: PathBuf::from("<archive>"),
        component: "metadata.json",
    })
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

    #[test]
    fn civsave_archive_round_trips_mod_guest_state() {
        let mut sim = Simulation::with_seed(11);
        for _ in 0..3 {
            sim.tick();
        }
        sim.mod_host_mut()
            .restore_guest_memory("archive-mod", vec![1, 2]);

        let dir = tempdir().expect("tempdir");
        let archive_path = dir.path().join("slot_test.civsave.zst");
        CivSaveBundle::save_archive(&archive_path, &sim).expect("save archive");
        assert!(CivSaveBundle::is_save_archive(&archive_path));

        let loaded = CivSaveBundle::load_archive(&archive_path).expect("load archive");
        assert_eq!(loaded.state.tick, sim.state.tick);
        assert_eq!(
            loaded.mod_host().guest_memory_snapshot("archive-mod"),
            vec![1, 2]
        );
    }
}
