//! Tests for FR-CIV-CORE-007
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-007: Snapshot Serialization.
//! State can be serialized to JSON snapshot without loss.

#[cfg(test)]
mod fr_fr_civ_core_007 {
    /// Serialize WorldState to JSON and deserialize; key fields must round-trip.
    #[test]
    fn world_state_json_round_trip() {
        let mut sim = civ_engine::Simulation::with_seed(7);
        sim.tick();
        let original = sim.state.clone();
        let json = serde_json::to_string(&original).expect("serialize");
        let restored: civ_engine::WorldState =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original.tick, restored.tick);
        assert_eq!(original.population, restored.population);
        assert_eq!(
            original.energy_budget_joules,
            restored.energy_budget_joules
        );
        assert_eq!(original.factions, restored.factions);
    }

    /// SimulationSnapshot must be serializable to JSON without error.
    #[test]
    fn simulation_snapshot_json_serializable() {
        let mut sim = civ_engine::Simulation::with_seed(10);
        sim.tick();
        let snap = sim.snapshot();
        let json = serde_json::to_string(&snap).expect("snapshot to JSON");
        assert!(json.len() > 10, "snapshot JSON should be non-trivial");
    }
}
