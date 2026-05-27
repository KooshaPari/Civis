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
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
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
}
