use civ_engine::{CivSaveBundle, Simulation};
use rand::RngCore;

#[test]
fn saved_rng_resumes_at_next_draw() {
    let saves = tempfile::tempdir().unwrap();
    let path = saves.path().join("rng.civsave.zst");
    let mut source = Simulation::with_seed(42);
    source.rng_mut().set_stream(7);
    let _ = source.rng_mut().next_u64();
    CivSaveBundle::save_archive(&path, &source).unwrap();
    let expected: Vec<_> = (0..40).map(|_| source.rng_mut().next_u64()).collect();
    let mut restored = CivSaveBundle::load(&path).unwrap();
    assert_eq!(
        (0..40)
            .map(|_| restored.rng_mut().next_u64())
            .collect::<Vec<_>>(),
        expected,
        "saved RNG stream must resume across block boundaries"
    );
}

#[test]
fn version_and_tick_must_match_present_manifest() {
    for (field, value) in [("format_version", 4), ("format_version", 5), ("tick", 99)] {
        let saves = tempfile::tempdir().unwrap();
        let path = saves.path().join("rng.civsave");
        CivSaveBundle::save_dir(&path, &Simulation::with_seed(42)).unwrap();
        let metadata_path = path.join("metadata.json");
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&metadata_path).unwrap()).unwrap();
        metadata[field] = serde_json::json!(value);
        std::fs::write(metadata_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
        assert!(
            CivSaveBundle::load_dir(&path).is_err(),
            "mismatched {field} must fail"
        );
    }
}

#[test]
fn current_metadata_requires_matching_spec_id() {
    let saves = tempfile::tempdir().unwrap();
    let path = saves.path().join("rng.civsave");
    CivSaveBundle::save_dir(&path, &Simulation::with_seed(42)).unwrap();
    let metadata_path = path.join("metadata.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&metadata_path).unwrap()).unwrap();
    metadata["spec_id"] = serde_json::json!("wrong");
    std::fs::write(metadata_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
    assert!(CivSaveBundle::load_dir(&path).is_err());
}

#[test]
fn missing_rng_sidecar_is_rejected() {
    let saves = tempfile::tempdir().unwrap();
    let path = saves.path().join("rng.civsave");
    CivSaveBundle::save_dir(&path, &Simulation::with_seed(42)).unwrap();
    std::fs::remove_file(path.join("rng.json")).unwrap();
    assert!(CivSaveBundle::load_dir(&path).is_err());
}

#[test]
fn tampered_rng_sidecar_is_rejected() {
    let saves = tempfile::tempdir().unwrap();
    let path = saves.path().join("rng.civsave");
    CivSaveBundle::save_dir(&path, &Simulation::with_seed(42)).unwrap();
    std::fs::write(path.join("rng.json"), "{}").unwrap();
    assert!(CivSaveBundle::load_dir(&path).is_err());
}

#[test]
fn legacy_v4_without_rng_sidecar_still_loads() {
    let saves = tempfile::tempdir().unwrap();
    let path = saves.path().join("rng.civsave");
    CivSaveBundle::save_dir(&path, &Simulation::with_seed(42)).unwrap();
    std::fs::remove_file(path.join("rng.json")).unwrap();
    std::fs::remove_file(path.join("integrity.json")).unwrap();
    let metadata_path = path.join("metadata.json");
    let mut metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&metadata_path).unwrap()).unwrap();
    metadata["format_version"] = serde_json::json!(4);
    std::fs::write(metadata_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
    assert!(CivSaveBundle::load_dir(&path).is_ok());
}
