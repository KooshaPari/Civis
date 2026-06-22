use std::path::PathBuf;

use civ_save_db::{SaveDb, SessionSaveRecord};
use tempfile::tempdir;

fn temp_db_path() -> (tempfile::TempDir, PathBuf) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("saves.db");
    (dir, path)
}

#[test]
fn record_slot_save_creates_entry_and_appears_in_list_for_session() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    let save_id = db
        .record_slot_save("session-a", "slot-1", 100, "/saves/slot-1.civsave.zst", 1024)
        .expect("record slot");

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 1);

    let SessionSaveRecord::Slot(slot) = &records[0] else {
        panic!("expected slot record");
    };

    assert_eq!(slot.id, save_id);
    assert_eq!(slot.session_id, "session-a");
    assert_eq!(slot.slot_name, "slot-1");
    assert_eq!(slot.tick, 100);
    assert_eq!(slot.file_path, "/saves/slot-1.civsave.zst");
    assert_eq!(slot.byte_size, 1024);
}

#[test]
fn record_autosave_creates_autosave_entry() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    let save_id = db
        .record_autosave("session-a", 5, "/saves/autosave-1.civsave.zst", 512)
        .expect("record autosave");

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 1);

    let SessionSaveRecord::Autosave(autosave) = &records[0] else {
        panic!("expected autosave record");
    };

    assert_eq!(autosave.id, save_id);
    assert_eq!(autosave.session_id, "session-a");
    assert_eq!(autosave.tick, 5);
    assert_eq!(autosave.file_path, "/saves/autosave-1.civsave.zst");
    assert_eq!(autosave.byte_size, 512);
}

#[test]
fn evict_autosaves_removes_oldest_when_over_limit() {
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
    let ticks: Vec<_> = records
        .into_iter()
        .map(|record| match record {
            SessionSaveRecord::Autosave(autosave) => autosave.tick,
            other => panic!("expected autosave record, got {other:?}"),
        })
        .collect();

    assert_eq!(ticks, vec![4, 3]);
}

#[test]
fn list_for_session_returns_empty_for_unknown_session_id() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    let records = db.list_for_session("session-missing").expect("list session");
    assert!(records.is_empty());
}

#[test]
fn two_saves_in_same_session_both_appear() {
    let (_dir, path) = temp_db_path();
    let db = SaveDb::open(&path).expect("open db");

    db.record_slot_save("session-a", "slot-1", 10, "/saves/slot-1.civsave.zst", 100)
        .expect("record slot 1");
    db.record_slot_save("session-a", "slot-2", 20, "/saves/slot-2.civsave.zst", 200)
        .expect("record slot 2");

    let records = db.list_for_session("session-a").expect("list session");
    assert_eq!(records.len(), 2);

    let slot_names: Vec<_> = records
        .into_iter()
        .map(|record| match record {
            SessionSaveRecord::Slot(slot) => slot.slot_name,
            other => panic!("expected slot record, got {other:?}"),
        })
        .collect();

    assert_eq!(slot_names, vec!["slot-1".to_string(), "slot-2".to_string()]);
}
