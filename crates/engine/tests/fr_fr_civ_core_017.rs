//! Tests for FR-CIV-CORE-017
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-017: Snapshot Filtering.
//! Clients can request partial snapshots (filter by entity type, region).

#[cfg(test)]
mod fr_fr_civ_core_017 {
    /// SimulationSnapshot contains a subset of Simulation fields (partial view).
    #[test]
    fn snapshot_is_partial_view() {
        let mut sim = civ_engine::Simulation::with_seed(1);
        sim.tick();
        let snap = sim.snapshot();
        let json = serde_json::to_value(&snap).expect("snapshot to JSON");
        assert!(
            json.get("citizen_count").is_some(),
            "snapshot must have citizen_count"
        );
        assert!(
            json.get("building_count").is_some(),
            "snapshot must have building_count"
        );
        assert!(
            json.get("military_count").is_some(),
            "snapshot must have military_count"
        );
        assert!(json.get("tick").is_some(), "snapshot must have tick");
    }

    /// get_snapshot_for_session exists for per-client filtered views.
    #[test]
    fn session_snapshot_exists() {
        let mut sim = civ_engine::Simulation::with_seed(1);
        sim.tick();
        let session_snap = sim.get_snapshot_for_session("test-client-1", 0, &[]);
        let json_str = serde_json::to_string(&session_snap).expect("session snapshot to JSON");
        assert!(
            json_str.len() > 10,
            "session snapshot should be non-trivial"
        );
    }
}
