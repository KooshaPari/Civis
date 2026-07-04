//! Session file persistence for the Bevy reference client.
//!
//! Stores lightweight per-slot session state under the user's Civis data
//! directory, e.g. `%APPDATA%\civis\sessions\slot-1.ron` on Windows.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current session file format version.
pub const SESSION_FORMAT_VERSION: u32 = 1;

/// Per-world setup parameters mirrored from the main menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSetupParams {
    /// World seed selected for the current run.
    pub seed: u64,
    /// World-size preset index.
    pub world_size: usize,
}

impl Default for WorldSetupParams {
    fn default() -> Self {
        Self {
            seed: 0xC1F1_5EED_D3AD_BEEF,
            world_size: 1,
        }
    }
}

/// Minimal session state persisted per save slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionData {
    /// Format version written to disk.
    pub version: u32,
    /// Simulation seed for the saved session.
    pub seed: u64,
    /// Simulation tick at the time of save.
    pub tick: u64,
    /// World setup parameters used to generate the session.
    pub world_setup: WorldSetupParams,
    /// Save timestamp as Unix milliseconds.
    pub save_timestamp_unix_ms: u64,
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            version: SESSION_FORMAT_VERSION,
            seed: 0,
            tick: 0,
            world_setup: WorldSetupParams::default(),
            save_timestamp_unix_ms: 0,
        }
    }
}

/// Persist a session to the given slot.
pub fn save(slot: u8, data: &SessionData) -> io::Result<()> {
    let path = session_path(slot);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, text)
}

/// Load a session from the given slot.
pub fn load(slot: u8) -> io::Result<SessionData> {
    let text = fs::read_to_string(session_path(slot))?;
    ron::from_str(&text).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

/// Build the on-disk file path for a slot.
#[must_use]
pub fn session_path(slot: u8) -> PathBuf {
    session_dir().join(format!("slot-{slot}.ron"))
}

fn session_dir() -> PathBuf {
    user_data_dir().join("civis").join("sessions")
}

fn user_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(dir) = std::env::var_os("APPDATA") {
            return PathBuf::from(dir);
        }
        if let Some(dir) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(dir);
        }
    }

    if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(dir);
    }

    let home = std::env::var_os("HOME").unwrap_or_default();
    Path::new(&home).join(".local").join("share")
}

/// Return the current Unix timestamp in milliseconds.
#[must_use]
pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::UNIX_EPOCH;

    #[test]
    fn path_uses_slot_name() {
        let path = session_path(3);
        assert!(path.ends_with(Path::new("civis").join("sessions").join("slot-3.ron")));
    }

    #[test]
    fn ron_round_trip() {
        let data = SessionData {
            version: SESSION_FORMAT_VERSION,
            seed: 42,
            tick: 99,
            world_setup: WorldSetupParams {
                seed: 7,
                world_size: 4,
            },
            save_timestamp_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_millis() as u64,
        };
        let text = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())
            .expect("serialize");
        let back: SessionData = ron::from_str(&text).expect("deserialize");
        assert_eq!(back, data);
    }
}
