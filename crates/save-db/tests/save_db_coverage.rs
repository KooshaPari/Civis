use std::path::PathBuf;

use civ_save_db::{format_session_saved_event_json, SaveDb, SessionSaveRecord};
use rusqlite::Connection;
use tempfile::tempdir;

fn temp_db_path() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("nested").join("saves.db");
    (dir, path)
}

#[test]
fn open_creates_parent_directory_and_schema() {
    let (_dir, path) = temp_db_path();

    let _db = SaveDb::open(&path).expect("open db");
    assert!(path.exists(), "database file should exist after open");

    let conn = Connection::open(&path).expect("reopen db");
    let save_slots: String = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'save_slots'",
            [],
            |row| row.get(0),
        )
        .expect("save_slots table");
    let autosaves: String = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'autosaves'",
            [],
            |row| row.get(0),
        )
        .expect("autosaves table");

    assert_eq!(save_slots, "save_slots");
    assert_eq!(autosaves, "autosaves");
}

#[test]
fn record_slot_save_upserts_and_list_for_session_reflects_latest_state() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    let first_id = db
        .record_slot_save("session-a", "slot-1", 100, "/saves/slot-1.civsave.zst", 1024)
        .expect("record first slot");
    let second_id = db
        .record_slot_save("session-a", "slot-1", 200, "/saves/slot-1.civsave.zst", 2048)
        .expect("record updated slot");

    assert_ne!(first_id, second_id);

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 1);

    let SessionSaveRecord::Slot(slot) = &records[0] else {
        panic!("expected slot record");
    };

    assert_eq!(slot.session_id, "session-a");
    assert_eq!(slot.slot_name, "slot-1");
    assert_eq!(slot.tick, 200);
    assert_eq!(slot.file_path, "/saves/slot-1.civsave.zst");
    assert_eq!(slot.byte_size, 2048);
}

#[test]
fn record_autosave_and_list_for_session_returns_newest_first() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    db.record_autosave("session-a", 5, "/saves/autosave-1.civsave.zst", 512)
        .expect("record autosave 1");
    db.record_autosave("session-a", 10, "/saves/autosave-2.civsave.zst", 1024)
        .expect("record autosave 2");

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 2);

    let autosaves: Vec<_> = records
        .into_iter()
        .map(|record| match record {
            SessionSaveRecord::Autosave(autosave) => autosave,
            other => panic!("expected autosave record, got {other:?}"),
        })
        .collect();

    assert_eq!(autosaves[0].tick, 10);
    assert_eq!(autosaves[0].file_path, "/saves/autosave-2.civsave.zst");
    assert_eq!(autosaves[1].tick, 5);
    assert_eq!(autosaves[1].file_path, "/saves/autosave-1.civsave.zst");
}

#[test]
fn list_for_session_groups_slots_before_autosaves_and_filters_other_sessions() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    db.record_slot_save("session-a", "slot-b", 20, "/saves/slot-b.civsave.zst", 111)
        .expect("slot b");
    db.record_slot_save("session-a", "slot-a", 10, "/saves/slot-a.civsave.zst", 222)
        .expect("slot a");
    db.record_autosave("session-a", 3, "/saves/autosave.civsave.zst", 333)
        .expect("autosave");
    db.record_slot_save("session-b", "slot-z", 99, "/saves/slot-z.civsave.zst", 444)
        .expect("other session");

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 3);

    let slot_names: Vec<_> = records
        .iter()
        .take(2)
        .map(|record| match record {
            SessionSaveRecord::Slot(slot) => slot.slot_name.as_str(),
            other => panic!("expected slot record, got {other:?}"),
        })
        .collect();
    assert_eq!(slot_names, vec!["slot-a", "slot-b"]);

    let SessionSaveRecord::Autosave(autosave) = &records[2] else {
        panic!("expected autosave record");
    };
    assert_eq!(autosave.session_id, "session-a");
    assert_eq!(autosave.tick, 3);
}

#[test]
fn evict_autosaves_keeps_newest_records_and_returns_deleted_paths() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    for tick in 1..=4 {
        db.record_autosave(
            "session-a",
            tick,
            &format!("/saves/autosave-{tick}.civsave.zst"),
            100 + tick,
        )
        .expect("record autosave");
    }

    let evicted = db.evict_autosaves("session-a", 2).expect("evict autosaves");
    assert_eq!(
        evicted,
        vec![
            "/saves/autosave-1.civsave.zst".to_string(),
            "/saves/autosave-2.civsave.zst".to_string(),
        ]
    );

    let records = db.list_for_session("session-a").expect("list session");
    let remaining_ticks: Vec<_> = records
        .into_iter()
        .map(|record| match record {
            SessionSaveRecord::Autosave(autosave) => autosave.tick,
            other => panic!("expected autosave record, got {other:?}"),
        })
        .collect();
    assert_eq!(remaining_ticks, vec![4, 3]);
}

#[test]
fn evict_autosaves_is_noop_when_under_limit() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    db.record_autosave("session-a", 1, "/saves/autosave-1.civsave.zst", 100)
        .expect("record autosave");

    let evicted = db.evict_autosaves("session-a", 4).expect("evict autosaves");
    assert!(evicted.is_empty());

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 1);
}

#[test]
fn format_session_saved_event_json_has_expected_shape() {
    let json = format_session_saved_event_json("session-a", "save-123", "slot-1", 42, 2048);
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse json");

    assert_eq!(value["event_type"], "session.saved.v1");
    assert_eq!(value["session_id"], "session-a");
    assert_eq!(value["save_id"], "save-123");
    assert_eq!(value["slot"], "slot-1");
    assert_eq!(value["tick"], 42);
    assert_eq!(value["byte_size"], 2048);
}
