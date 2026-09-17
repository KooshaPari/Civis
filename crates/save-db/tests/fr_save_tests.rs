//! FR traceability tests for the save-db crate.
//!
//! Covers: FR-SAVE-001, FR-SAVE-003, FR-SAVE-004, FR-SAVE-005, FR-SAVE-010

use civ_save_db::{SaveDb, SessionSaveRecord};

fn temp_db() -> SaveDb {
    SaveDb::open_in_memory().expect("failed to open in-memory SaveDb")
}

/// FR-SAVE-001 — Save/load persistence: slot save records can be written and read back.
#[test]
fn fr_save_001_slot_save_roundtrip() {
    let db = temp_db();
    let id = db
        .record_slot_save("sess-1", "slot-a", 100, "/tmp/save1.bin", 4096)
        .expect("record_slot_save failed");
    assert!(!id.is_empty(), "record_slot_save must return a non-empty id");

    let records = db.list_for_session("sess-1").expect("list_for_session failed");
    assert_eq!(records.len(), 1, "expected exactly 1 record for session");
    match &records[0] {
        SessionSaveRecord::Slot(slot) => {
            assert_eq!(slot.session_id, "sess-1");
            assert_eq!(slot.slot_name, "slot-a");
            assert_eq!(slot.tick, 100);
            assert_eq!(slot.file_path, "/tmp/save1.bin");
            assert_eq!(slot.byte_size, 4096);
        }
        _ => panic!("expected Slot record"),
    }
}

/// FR-SAVE-003 — Save metadata: record contains all required metadata fields.
#[test]
fn fr_save_003_save_metadata_completeness() {
    let db = temp_db();
    let id = db
        .record_slot_save("sess-meta", "meta-slot", 500, "/data/save.bin", 8192)
        .expect("record failed");

    let records = db.list_for_session("sess-meta").expect("list failed");
    assert_eq!(records.len(), 1);
    if let SessionSaveRecord::Slot(slot) = &records[0] {
        assert_eq!(slot.id, id, "id must match returned id");
        assert!(!slot.created_at.is_empty(), "created_at must be populated");
        assert!(slot.byte_size > 0, "byte_size must be positive");
    } else {
        panic!("expected Slot");
    }
}

/// FR-SAVE-004 — Save versioning: overwriting the same slot updates the record.
#[test]
fn fr_save_004_slot_overwrite_updates_tick() {
    let db = temp_db();
    db.record_slot_save("sess-ver", "v-slot", 10, "/v1.bin", 100)
        .expect("first save");
    db.record_slot_save("sess-ver", "v-slot", 20, "/v2.bin", 200)
        .expect("second save (overwrite)");

    let records = db.list_for_session("sess-ver").expect("list");
    assert_eq!(records.len(), 1, "overwrite should produce exactly 1 record");
    if let SessionSaveRecord::Slot(slot) = &records[0] {
        assert_eq!(slot.tick, 20, "tick should reflect latest save");
        assert_eq!(slot.file_path, "/v2.bin", "file_path should reflect latest save");
        assert_eq!(slot.byte_size, 200, "byte_size should reflect latest save");
    } else {
        panic!("expected Slot");
    }
}

/// FR-SAVE-005 — Save integrity: autosave records are stored independently from slots.
#[test]
fn fr_save_005_autosave_independent_of_slots() {
    let db = temp_db();
    db.record_slot_save("sess-integ", "slot-1", 10, "/s.bin", 100)
        .expect("slot save");
    db.record_autosave("sess-integ", 15, "/auto.bin", 200)
        .expect("autosave");

    let records = db.list_for_session("sess-integ").expect("list");
    assert_eq!(records.len(), 2, "should have 1 slot + 1 autosave");

    let has_slot = records.iter().any(|r| matches!(r, SessionSaveRecord::Slot(_)));
    let has_auto = records
        .iter()
        .any(|r| matches!(r, SessionSaveRecord::Autosave(_)));
    assert!(has_slot, "must contain a Slot record");
    assert!(has_auto, "must contain an Autosave record");
}

/// FR-SAVE-010 — Save recovery: evict_autosaves removes oldest while keeping max_slots.
#[test]
fn fr_save_010_evict_autosaves_keeps_newest() {
    let db = temp_db();
    for tick in 0..5 {
        db.record_autosave("sess-evict", tick, &format!("/auto-{tick}.bin"), 100)
            .expect("autosave");
    }

    let evicted = db
        .evict_autosaves("sess-evict", 2)
        .expect("evict failed");
    assert_eq!(evicted.len(), 3, "should evict 5 - 2 = 3 autosaves");

    // Evicted paths should be the oldest (tick 0, 1, 2)
    assert!(evicted.contains(&"/auto-0.bin".to_string()));
    assert!(evicted.contains(&"/auto-1.bin".to_string()));
    assert!(evicted.contains(&"/auto-2.bin".to_string()));

    // Remaining should be the newest (tick 3, 4)
    let records = db.list_for_session("sess-evict").expect("list");
    let auto_count = records
        .iter()
        .filter(|r| matches!(r, SessionSaveRecord::Autosave(_)))
        .count();
    assert_eq!(auto_count, 2, "should have 2 autosaves remaining");
}

/// FR-SAVE-001 — Save/load persistence: empty session returns empty list.
#[test]
fn fr_save_001_empty_session_returns_no_records() {
    let db = temp_db();
    let records = db
        .list_for_session("nonexistent")
        .expect("list failed");
    assert!(records.is_empty(), "empty session should return no records");
}
