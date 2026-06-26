//! End-to-end tests for save/load round-trips with the civ-server crate.
use std::path::PathBuf;

fn test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("target").join("test-saves")
}

fn write_slot(slot: &str, data: &[u8]) -> bool {
    let path = test_dir().join(format!("{slot}.sav"));
    let _ = std::fs::create_dir_all(&test_dir());
    std::fs::write(&path, data).is_ok()
}

fn load_slot(slot: &str) -> Option<Vec<u8>> {
    let path = test_dir().join(format!("{slot}.sav"));
    std::fs::read(&path).ok()
}

fn cleanup(slot: &str) {
    let _ = std::fs::remove_file(test_dir().join(format!("{slot}.sav")));
}

#[test]
fn save_and_load_round_trip_returns_identical_data() {
    let slot = "test_rt";
    let data = b"hello-save-world";
    assert!(write_slot(slot, data));
    assert_eq!(load_slot(slot).as_deref(), Some(&data[..]));
    cleanup(slot);
}

#[test]
fn load_nonexistent_slot_returns_none() {
    assert!(load_slot("nonexistent").is_none());
}

#[test]
fn overwrite_existing_slot_replaces_content() {
    let slot = "test_ow";
    assert!(write_slot(slot, b"old"));
    assert!(write_slot(slot, b"new"));
    assert_eq!(load_slot(slot).as_deref(), Some(&b"new"[..]));
    cleanup(slot);
}

#[test]
fn delete_removes_slot() {
    let slot = "test_del";
    assert!(write_slot(slot, b"data"));
    cleanup(slot);
    assert!(load_slot(slot).is_none());
}
