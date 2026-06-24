//! Production save slots (CIV-1000 §13 partial) — mirrors civ-watch `saves/` layout.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

/// Categorical slot bucket used by the save browser UI (`FR-CIV-SAVE-001`).
///
/// Three buckets align with the production-slot / autosave / manual naming
/// convention. The strings are stable across clients (web dashboard, Bevy,
/// Godot, Unreal) so the UI can switch on them without parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveSlotCategory {
    /// One of the five production slots (`slot-1`..`slot-5`).
    Slot,
    /// Autosave (`autosave`, `autosave-<n>`).
    Auto,
    /// Manual save (anything that isn't a production slot or autosave).
    Manual,
}

impl SaveSlotCategory {
    /// String form used in serialized JSON and the public `save.list`
    /// response. Stable across clients.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Slot => "slot",
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Coarse age label derived from a save's `mtime`.
///
/// The browser UI uses this to render "just now", "5 min ago", "yesterday",
/// etc. without forcing the client to format a `SystemTime` in five
/// different timezones. Labels are stable strings; the underlying
/// `mtime_unix` is included so a future client can re-format with full
/// precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveAgeLabel {
    /// Less than 60 seconds old.
    JustNow,
    /// 1–59 minutes old.
    MinutesAgo,
    /// 1–23 hours old.
    HoursAgo,
    /// 1–6 days old.
    DaysAgo,
    /// More than 7 days old.
    WeeksAgo,
    /// `mtime` could not be read; the row still appears in the browser
    /// but cannot be age-classified.
    Unknown,
}

impl SaveAgeLabel {
    /// Classify `age_seconds` (always non-negative) into a stable label.
    /// The thresholds are chosen to give the UI 5 readable buckets.
    #[must_use]
    pub const fn from_seconds(age_seconds: u64) -> Self {
        if age_seconds < 60 {
            Self::JustNow
        } else if age_seconds < 60 * 60 {
            Self::MinutesAgo
        } else if age_seconds < 24 * 60 * 60 {
            Self::HoursAgo
        } else if age_seconds < 7 * 24 * 60 * 60 {
            Self::DaysAgo
        } else {
            Self::WeeksAgo
        }
    }
}

/// Richer row returned by the save browser (CIV-1000 §13 + `FR-CIV-SAVE-001`).
///
/// `SaveBrowserEntry` extends `SaveListEntry` with the fields the in-game
/// save-slot browser needs to render a panel: byte size for "show me how
/// much disk this uses", mtime for client-side re-sorting, a categorical
/// slot bucket, and a coarse age label so the UI can write
/// "Saved 5 minutes ago" without touching `chrono` on the client.
///
/// The shape is deliberately a superset of `SaveListEntry`: existing
/// `save.list` consumers continue to work because every field of
/// `SaveListEntry` is present in `SaveBrowserEntry` with the same name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveBrowserEntry {
    /// Save stem name (e.g. `slot-1`, `autosave-20260526`).
    pub name: String,
    /// Engine tick at save time (from `metadata.json` when available).
    pub tick: u64,
    /// `slot`, `auto`, or `manual` — mirrors `SaveListEntry::save_type`.
    pub save_type: &'static str,
    /// Categorical bucket (the typed form of `save_type`).
    pub category: SaveSlotCategory,
    /// File size in bytes, or `None` if the file was unreadable.
    pub byte_size: Option<u64>,
    /// File mtime as Unix seconds, or `None` if the file was unreadable.
    pub mtime_unix: Option<u64>,
    /// Coarse age classification, or `Unknown` if mtime is missing.
    pub age_label: SaveAgeLabel,
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

/// Classify a save stem into the typed [`SaveSlotCategory`].
#[must_use]
pub fn save_category_for_name(name: &str) -> SaveSlotCategory {
    if PRODUCTION_SLOTS.contains(&name) {
        SaveSlotCategory::Slot
    } else if name == "autosave" || name.starts_with("autosave-") {
        SaveSlotCategory::Auto
    } else {
        SaveSlotCategory::Manual
    }
}

/// Convert a `SystemTime` mtime to Unix seconds (clamped to 0 on the
/// unlikely path where the platform reports a pre-epoch mtime).
#[must_use]
pub fn mtime_unix_seconds(mtime: SystemTime) -> u64 {
    mtime
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Convert `now - mtime` (clamped to 0) into a [`SaveAgeLabel`].
///
/// `now` is taken as a parameter so tests can pin time without sleeping.
#[must_use]
pub fn age_label_for_mtime(mtime: SystemTime, now: SystemTime) -> SaveAgeLabel {
    let age = now.duration_since(mtime).map(|d| d.as_secs()).unwrap_or(0);
    SaveAgeLabel::from_seconds(age)
}

/// List save files under `dir` as [`SaveBrowserEntry`] rows for the
/// in-game save-slot browser UI (`FR-CIV-SAVE-001`).
///
/// Each row carries a typed category, byte size, mtime, and a coarse
/// age label so the UI can render the panel without re-walking the
/// directory or touching `chrono` on the client. Rows are sorted
/// freshest-first by `(tick desc, name asc)` to match [`list_saves`].
pub fn list_saves_browser(dir: &Path) -> Result<Vec<SaveBrowserEntry>, JsonRpcError> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| JsonRpcError {
        code: error_code::INTERNAL_ERROR,
        message: err.to_string(),
        data: None,
    })?;
    let now = SystemTime::now();
    let mut entries = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        let Some(name) = save_stem_from_path(&path) else {
            continue;
        };
        let metadata = entry.metadata().ok();
        let byte_size = metadata.as_ref().map(|m| m.len());
        let mtime = metadata.and_then(|m| m.modified().ok());
        let mtime_unix = mtime.map(mtime_unix_seconds);
        let age_label = match mtime {
            Some(t) => age_label_for_mtime(t, now),
            None => SaveAgeLabel::Unknown,
        };
        let category = save_category_for_name(&name);
        entries.push(SaveBrowserEntry {
            name: name.clone(),
            tick: read_save_tick(&path),
            save_type: category.as_str(),
            category,
            byte_size,
            mtime_unix,
            age_label,
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
                if candidate.2 < current.2 || (candidate.2 == current.2 && candidate.1 > current.1)
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

    // -- FR-CIV-SAVE-001 ---------------------------------------------------
    //
    // FR-CIV-SAVE-001 — Save-slot UI + browser. The substrate slice here
    // adds a `SaveBrowserEntry` view that the in-game UI (web dashboard,
    // Bevy/Godot/Unreal panels) can render without re-walking the save
    // directory. Tests below cover the typed category mapping, the age
    // bucketing, the byte_size/mtime propagation, and the sort ordering
    // that the browser relies on.

    /// FR-CIV-SAVE-001.
    ///
    /// Production slot stems classify to `SaveSlotCategory::Slot`.
    #[test]
    fn fr_civ_save_001_save_category_for_name_classifies_three_buckets() {
        for slot in PRODUCTION_SLOTS {
            assert_eq!(save_category_for_name(slot), SaveSlotCategory::Slot);
        }
        assert_eq!(save_category_for_name("autosave"), SaveSlotCategory::Auto);
        assert_eq!(
            save_category_for_name("autosave-20260610"),
            SaveSlotCategory::Auto
        );
        assert_eq!(
            save_category_for_name("autosave-00000000000000000042"),
            SaveSlotCategory::Auto
        );
        assert_eq!(
            save_category_for_name("quick-save"),
            SaveSlotCategory::Manual
        );
        assert_eq!(
            save_category_for_name("replay-20260610-183045"),
            SaveSlotCategory::Manual
        );
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `SaveAgeLabel::from_seconds` partitions `age_seconds` into 5
    /// readable buckets. The boundary cases are the contract — clients
    /// rely on them for stable UI rendering.
    #[test]
    fn fr_civ_save_001_age_label_partitions_five_buckets() {
        // < 60s → JustNow
        assert_eq!(SaveAgeLabel::from_seconds(0), SaveAgeLabel::JustNow);
        assert_eq!(SaveAgeLabel::from_seconds(59), SaveAgeLabel::JustNow);
        // 60s..3600s → MinutesAgo
        assert_eq!(SaveAgeLabel::from_seconds(60), SaveAgeLabel::MinutesAgo);
        assert_eq!(
            SaveAgeLabel::from_seconds(60 * 60 - 1),
            SaveAgeLabel::MinutesAgo
        );
        // 1h..24h → HoursAgo
        assert_eq!(SaveAgeLabel::from_seconds(60 * 60), SaveAgeLabel::HoursAgo);
        assert_eq!(
            SaveAgeLabel::from_seconds(24 * 60 * 60 - 1),
            SaveAgeLabel::HoursAgo
        );
        // 1d..7d → DaysAgo
        assert_eq!(
            SaveAgeLabel::from_seconds(24 * 60 * 60),
            SaveAgeLabel::DaysAgo
        );
        assert_eq!(
            SaveAgeLabel::from_seconds(7 * 24 * 60 * 60 - 1),
            SaveAgeLabel::DaysAgo
        );
        // 7d+ → WeeksAgo
        assert_eq!(
            SaveAgeLabel::from_seconds(7 * 24 * 60 * 60),
            SaveAgeLabel::WeeksAgo
        );
        assert_eq!(SaveAgeLabel::from_seconds(u64::MAX), SaveAgeLabel::WeeksAgo);
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `mtime_unix_seconds` clamps a pre-epoch mtime to 0 instead of
    /// overflowing or panicking.
    #[test]
    fn fr_civ_save_001_mtime_unix_seconds_clamps_pre_epoch() {
        let post_epoch = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        assert_eq!(mtime_unix_seconds(post_epoch), 1_700_000_000);
        let pre_epoch = SystemTime::UNIX_EPOCH - std::time::Duration::from_secs(1);
        assert_eq!(mtime_unix_seconds(pre_epoch), 0);
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `age_label_for_mtime` returns `JustNow` for the same mtime and
    /// `now`, and `Unknown` is reachable only when mtime is `None`
    /// (handled by the list helper, not this function).
    #[test]
    fn fr_civ_save_001_age_label_for_mtime_classifies_zero_age_as_just_now() {
        let t = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
        assert_eq!(age_label_for_mtime(t, t), SaveAgeLabel::JustNow);
        // 5 minutes later.
        let now = t + std::time::Duration::from_secs(300);
        assert_eq!(age_label_for_mtime(t, now), SaveAgeLabel::MinutesAgo);
        // 2 hours later.
        let now = t + std::time::Duration::from_secs(2 * 60 * 60);
        assert_eq!(age_label_for_mtime(t, now), SaveAgeLabel::HoursAgo);
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `list_saves_browser` returns one row per save with the typed
    /// category, byte_size, mtime_unix, and an age_label consistent with
    /// the file's mtime.
    #[test]
    fn fr_civ_save_001_list_saves_browser_returns_rich_rows() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        // Production slot
        let slot_path = dir.path().join("slot-1.civsave.zst");
        CivSaveBundle::save_archive(&slot_path, &sim).expect("save slot");
        // Autosave
        let auto_path = dir.path().join("autosave-00000000000000000001.civsave.zst");
        CivSaveBundle::save_archive(&auto_path, &sim).expect("save auto");
        // Manual save
        let manual_path = dir.path().join("quick-save.civsave.zst");
        CivSaveBundle::save_archive(&manual_path, &sim).expect("save manual");

        let entries = list_saves_browser(dir.path()).expect("browser list");
        assert_eq!(entries.len(), 3);

        // All three rows must have a byte size and mtime — the browser UI
        // depends on them for the panel rendering.
        for entry in &entries {
            assert!(
                entry.byte_size.is_some(),
                "byte_size missing for {}",
                entry.name
            );
            assert!(
                entry.mtime_unix.is_some(),
                "mtime_unix missing for {}",
                entry.name
            );
            // `now` is the test's wall clock, the save was just written,
            // so the age_label must be JustNow.
            assert_eq!(entry.age_label, SaveAgeLabel::JustNow);
        }

        // Category mapping.
        let slot = entries
            .iter()
            .find(|e| e.name == "slot-1")
            .expect("slot row");
        assert_eq!(slot.category, SaveSlotCategory::Slot);
        assert_eq!(slot.save_type, "slot");
        let auto = entries
            .iter()
            .find(|e| e.name.starts_with("autosave"))
            .expect("auto row");
        assert_eq!(auto.category, SaveSlotCategory::Auto);
        assert_eq!(auto.save_type, "auto");
        let manual = entries
            .iter()
            .find(|e| e.name == "quick-save")
            .expect("manual row");
        assert_eq!(manual.category, SaveSlotCategory::Manual);
        assert_eq!(manual.save_type, "manual");
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `list_saves_browser` and `list_saves` agree on the row set and the
    /// (tick desc, name asc) sort ordering. The browser is a strict
    /// superset of the list view.
    #[test]
    fn fr_civ_save_001_browser_agrees_with_list_saves() {
        let dir = tempdir().expect("tempdir");
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        let a = dir.path().join("slot-1.civsave.zst");
        CivSaveBundle::save_archive(&a, &sim).expect("save a");
        sim.tick();
        sim.tick();
        let b = dir.path().join("slot-2.civsave.zst");
        CivSaveBundle::save_archive(&b, &sim).expect("save b");
        sim.tick();
        let c = dir.path().join("autosave-00000000000000000003.civsave.zst");
        CivSaveBundle::save_archive(&c, &sim).expect("save c");

        let list = list_saves(dir.path()).expect("list");
        let browser = list_saves_browser(dir.path()).expect("browser");
        assert_eq!(list.len(), browser.len());
        for (l, b) in list.iter().zip(browser.iter()) {
            assert_eq!(l.name, b.name);
            assert_eq!(l.tick, b.tick);
            assert_eq!(l.save_type, b.save_type);
        }
    }

    /// FR-CIV-SAVE-001.
    ///
    /// `list_saves_browser` on a missing directory returns an error that
    /// JSON-RPC can surface as `INTERNAL_ERROR`. This matches
    /// `list_saves` behavior so consumers have a single error path.
    #[test]
    fn fr_civ_save_001_list_saves_browser_errors_on_missing_dir() {
        let dir = tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist");
        let err = list_saves_browser(&missing).expect_err("missing dir should error");
        assert_eq!(err.code, error_code::INTERNAL_ERROR);
    }

    #[test]
    fn save_type_for_name_classifies() {
        assert_eq!(save_type_for_name("slot-1"), "slot");
        assert_eq!(save_type_for_name("autosave"), "auto");
        assert_eq!(save_type_for_name("autosave-3"), "auto");
        assert_eq!(save_type_for_name("my-game"), "manual");
    }

    #[test]
    fn save_archive_path_builds_and_rejects() {
        use std::path::Path;
        let dir = Path::new("/tmp/saves");
        let p = save_archive_path(dir, "mygame").expect("ok");
        assert!(p.to_string_lossy().ends_with("mygame.civsave.zst"));
        assert!(save_archive_path(dir, "").is_err());
        assert!(save_archive_path(dir, "../escape").is_err());
        assert!(save_archive_path(dir, "a/b").is_err());
    }

    #[test]
    fn parse_slot_name_params_extracts_and_validates() {
        use serde_json::json;
        let ok = parse_slot_name_params(Some(&json!({"slot_name":"slot-1"})));
        assert!(ok.is_ok());
        assert_eq!(ok.unwrap(), "slot-1");
        assert!(parse_slot_name_params(None).is_err());
        assert!(parse_slot_name_params(Some(&json!({}))).is_err());
        assert!(parse_slot_name_params(Some(&json!({"slot_name":""}))).is_err());
        assert!(parse_slot_name_params(Some(&json!({"slot_name":"bogus-slot"}))).is_err());
    }
}
