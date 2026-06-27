//! Integration tests for uncovered pub fns in crates/server (FR-CIV-TEST-007).
//!
//! Covers: validate_production_slot, save_type_for_name, save_category_for_name,
//!         mtime_unix_seconds, age_label_for_mtime, parse_replay_path,
//!         parse_sim_command_action.
use std::time::{Duration, SystemTime};

use civ_server::jsonrpc::{parse_replay_path, parse_sim_command_action, SimCommandAction};
use civ_server::saves::{
    age_label_for_mtime, mtime_unix_seconds, save_category_for_name, save_type_for_name,
    validate_production_slot, SaveSlotCategory, PRODUCTION_SLOTS,
};
use serde_json::json;

// ── validate_production_slot ─────────────────────────────────────────────────

#[test]
fn validate_production_slot_accepts_all_known_slots() {
    for slot in &PRODUCTION_SLOTS {
        assert!(validate_production_slot(slot).is_ok(), "expected Ok for {slot}");
    }
}

#[test]
fn validate_production_slot_rejects_unknown() {
    assert!(validate_production_slot("slot-9").is_err());
    assert!(validate_production_slot("").is_err());
    assert!(validate_production_slot("autosave").is_err());
}

// ── save_type_for_name / save_category_for_name ──────────────────────────────

#[test]
fn save_type_for_name_classifies_correctly() {
    assert_eq!(save_type_for_name("slot-1"), "slot");
    assert_eq!(save_type_for_name("slot-5"), "slot");
    assert_eq!(save_type_for_name("autosave"), "auto");
    assert_eq!(save_type_for_name("autosave-20260601"), "auto");
    assert_eq!(save_type_for_name("my-save"), "manual");
}

#[test]
fn save_category_for_name_matches_type_classification() {
    assert_eq!(save_category_for_name("slot-1"), SaveSlotCategory::Slot);
    assert_eq!(save_category_for_name("autosave"), SaveSlotCategory::Auto);
    assert_eq!(save_category_for_name("autosave-001"), SaveSlotCategory::Auto);
    assert_eq!(save_category_for_name("manual-save"), SaveSlotCategory::Manual);
}

// ── mtime_unix_seconds / age_label_for_mtime ────────────────────────────────

#[test]
fn mtime_unix_seconds_converts_epoch_correctly() {
    let epoch = SystemTime::UNIX_EPOCH;
    assert_eq!(mtime_unix_seconds(epoch), 0);
    assert_eq!(mtime_unix_seconds(epoch + Duration::from_secs(42)), 42);
}

#[test]
fn age_label_for_mtime_does_not_panic() {
    let now = SystemTime::now();
    // Recent file -- no panic
    let _ = age_label_for_mtime(now - Duration::from_secs(30), now);
    // Future mtime (mtime > now) -- clamps to 0, no panic
    let _ = age_label_for_mtime(now + Duration::from_secs(3600), now);
}

// ── parse_replay_path ────────────────────────────────────────────────────────

#[test]
fn parse_replay_path_accepts_relative_path() {
    // Only relative paths without parent segments are accepted.
    let params = json!({ "path": "saves/my.civreplay" });
    assert_eq!(
        parse_replay_path(Some(&params)).unwrap(),
        "saves/my.civreplay"
    );
}

#[test]
fn parse_replay_path_rejects_absolute_and_parent_segments() {
    // Absolute paths must be rejected (security: arbitrary-file-read).
    assert!(
        parse_replay_path(Some(&json!({ "path": "/saves/my.civreplay" }))).is_err(),
        "absolute paths must be rejected"
    );
    // Parent segments must be rejected (path traversal).
    assert!(
        parse_replay_path(Some(&json!({ "path": "../etc/passwd" }))).is_err(),
        "parent segments must be rejected"
    );
}

#[test]
fn parse_replay_path_rejects_missing_or_empty() {
    assert!(parse_replay_path(None).is_err());
    assert!(parse_replay_path(Some(&json!({}))).is_err());
    assert!(parse_replay_path(Some(&json!({ "path": "" }))).is_err());
}

// ── parse_sim_command_action ─────────────────────────────────────────────────

#[test]
fn parse_sim_command_action_known_and_unknown() {
    assert_eq!(
        parse_sim_command_action(Some(&json!({ "action": "noop" }))),
        Some(SimCommandAction::Noop)
    );
    assert_eq!(
        parse_sim_command_action(Some(&json!({ "action": "tick" }))),
        Some(SimCommandAction::Tick)
    );
    assert_eq!(
        parse_sim_command_action(Some(&json!({ "action": "explode" }))),
        None
    );
    assert_eq!(parse_sim_command_action(None), None);
}