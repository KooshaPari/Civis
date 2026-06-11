//! Production save slots (CIV-1000 §13 partial) — mirrors civ-watch `saves/` layout.

use std::path::{Path, PathBuf};

use civ_engine::CivSaveBundle;
use serde::Serialize;
use serde_json::Value;

use crate::jsonrpc::{error_code, JsonRpcError};

/// Named production slots accepted by `save.slot` / `save.load`.
pub const PRODUCTION_SLOTS: [&str; 5] = ["slot-1", "slot-2", "slot-3", "slot-4", "slot-5"];

/// One row returned by `save.list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveListEntry {
    /// Save stem name (e.g. `slot-1`, `autosave-20260526`).
    pub name: String,
    /// Engine tick at save time (from `metadata.json` when available).
    pub tick: u64,
    /// `slot`, `auto`, or `manual`.
    pub save_type: &'static str,
}

/// Validate `slot_name` against [`PRODUCTION_SLOTS`].
pub fn validate_production_slot(slot: &str) -> Result<(), String> {
    if PRODUCTION_SLOTS.contains(&slot) {
        Ok(())
    } else {
        Err(format!(
            "invalid slot {slot:?}; expected one of {}",
            PRODUCTION_SLOTS.join(", ")
        ))
    }
}

/// Classify a save stem for `save.list`.
pub fn save_type_for_name(name: &str) -> &'static str {
    if PRODUCTION_SLOTS.contains(&name) {
        "slot"
    } else if name == "autosave" || name.starts_with("autosave-") {
        "auto"
    } else {
        "manual"
    }
}

fn sanitize_save_filename(filename: &str) -> Result<String, String> {
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        return Err("filename cannot be empty".into());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("filename must be a simple name".into());
    }
    Ok(trimmed
        .trim_end_matches(".civreplay")
        .trim_end_matches(".civsave.zst")
        .trim_end_matches(".civsave")
        .to_string())
}

/// Path to `{dir}/{name}.civsave.zst`.
pub fn save_archive_path(dir: &Path, filename: &str) -> Result<PathBuf, String> {
    let name = sanitize_save_filename(filename)?;
    Ok(dir.join(format!("{name}.civsave.zst")))
}

/// Parse `{ "slot_name": "slot-1" }` and validate against production slots.
pub fn parse_slot_name_params(params: Option<&Value>) -> Result<String, JsonRpcError> {
    let slot_name = params
        .and_then(|p| p.get("slot_name"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or(JsonRpcError {
            code: error_code::INVALID_PARAMS,
            message: "Invalid params: expected non-empty string \"slot_name\"".to_owned(),
            data: None,
        })?;
    validate_production_slot(slot_name).map_err(|message| JsonRpcError {
        code: error_code::INVALID_PARAMS,
        message,
        data: None,
    })?;
    Ok(slot_name.to_owned())
}

fn save_stem_from_path(path: &Path) -> Option<String> {
    if CivSaveBundle::is_save_archive(path) {
        path.file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".civsave.zst").to_string())
    } else if CivSaveBundle::is_save_dir(path) {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".civsave").to_string())
    } else if path.extension().and_then(|s| s.to_str()) == Some("civreplay") {
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}

fn read_save_tick(path: &Path) -> u64 {
    if CivSaveBundle::is_save_archive(path) || CivSaveBundle::is_save_dir(path) {
        CivSaveBundle::read_metadata(path)
            .map(|meta| meta.tick)
            .unwrap_or(0)
    } else {
        0
    }
}

/// List save files under `dir` (archives, folders, legacy `.civreplay`).
pub fn list_saves(dir: &Path) -> Result<Vec<SaveListEntry>, JsonRpcError> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| JsonRpcError {
        code: error_code::INTERNAL_ERROR,
        message: err.to_string(),
        data: None,
    })?;
    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = save_stem_from_path(&path) else {
            continue;
        };
        entries.push(SaveListEntry {
            name: name.clone(),
            tick: read_save_tick(&path),
            save_type: save_type_for_name(&name),
        });
    }
    entries.sort_by(|a, b| b.tick.cmp(&a.tick).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

/// Find the freshest on-disk save to use for load-on-launch (P5 / CIV-1000 §13.5).
///
/// Slots are preferred over autosaves; within each tier, the freshest `mtime`
/// wins. Returns `Ok(None)` if `dir` is empty or unreadable so the caller can
/// fall back to a fresh `Simulation::default()`.
pub fn most_recent_save_path(dir: &Path) -> Result<Option<PathBuf>, std::io::Error> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(read) => read,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };

    let mut best: Option<(PathBuf, std::time::SystemTime, u8)> = None;
    // tier 0 = production slot, tier 1 = autosave, tier 2 = manual / replay
    let tier_for = |name: &str| -> u8 {
        if PRODUCTION_SLOTS.contains(&name) {
            0
        } else if name == "autosave" || name.starts_with("autosave-") {
            1
        } else {
            2
        }
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = save_stem_from_path(&path) else {
            continue;
        };
        let mtime = entry
            .metadata()
            .and_then(|meta| meta.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        let tier = tier_for(&name);
        let candidate = (path, mtime, tier);
        best = match best {
            None => Some(candidate),
            Some(current) => {
                if candidate.2 < current.2
                    || (candidate.2 == current.2 && candidate.1 > current.1)
                {
                    Some(candidate)
                } else {
                    Some(current)
                }
            }
        };
    }
    Ok(best.map(|(path, _, _)| path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_engine::Simulation;
    use tempfile::tempdir;

    #[test]
    fn validate_production_slot_accepts_five_slots() {
        for slot in PRODUCTION_SLOTS {
            validate_production_slot(slot).expect(slot);
        }
        assert!(validate_production_slot("slot-6").is_err());
    }

    #[test]
    fn fr_save_025_list_saves_sorts_by_tick_descending() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        let older_path = dir.path().join("slot-1.civsave.zst");
        CivSaveBundle::save_archive(&older_path, &sim).expect("save older");
        sim.tick();
        sim.tick();
        let newer_path = dir.path().join("slot-2.civsave.zst");
        CivSaveBundle::save_archive(&newer_path, &sim).expect("save newer");

        let entries = list_saves(dir.path()).expect("list");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "slot-2");
        assert_eq!(entries[1].name, "slot-1");
        assert!(entries[0].tick > entries[1].tick);
    }

    #[test]
    fn list_saves_includes_saved_archive() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        let path = dir.path().join("slot-1.civsave.zst");
        CivSaveBundle::save_archive(&path, &sim).expect("save");
        let entries = list_saves(dir.path()).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "slot-1");
        assert_eq!(entries[0].tick, sim.state.tick);
        assert_eq!(entries[0].save_type, "slot");
    }

    #[test]
    fn most_recent_save_path_prefers_slot_over_autosave() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        for _ in 0..3 {
            sim.tick();
        }
        let slot_path = dir.path().join("slot-2.civsave.zst");
        CivSaveBundle::save_archive(&slot_path, &sim).expect("save slot");
        // Sleep briefly so the autosave mtime is strictly newer, ensuring
        // tier preference (not mtime) drives the choice.
        std::thread::sleep(std::time::Duration::from_millis(20));
        let autosave_path = dir.path().join("autosave-00000000000000000003.civsave.zst");
        CivSaveBundle::save_archive(&autosave_path, &sim).expect("save auto");

        let chosen = most_recent_save_path(dir.path())
            .expect("most recent")
            .expect("found");
        assert_eq!(chosen, slot_path, "slot tier should win even when older");
    }

    #[test]
    fn most_recent_save_path_picks_freshest_in_tier() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        let old = dir.path().join("autosave-00000000000000000001.civsave.zst");
        CivSaveBundle::save_archive(&old, &sim).expect("save old");
        std::thread::sleep(std::time::Duration::from_millis(20));
        sim.tick();
        let newer = dir.path().join("autosave-00000000000000000002.civsave.zst");
        CivSaveBundle::save_archive(&newer, &sim).expect("save new");

        let chosen = most_recent_save_path(dir.path())
            .expect("most recent")
            .expect("found");
        assert_eq!(chosen, newer);
    }

    #[test]
    fn most_recent_save_path_returns_none_on_missing_dir() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist");
        let chosen = most_recent_save_path(&missing).expect("none on missing");
        assert!(chosen.is_none());
    }
}
