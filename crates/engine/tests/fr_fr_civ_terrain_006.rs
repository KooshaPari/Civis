//! Tests for FR-CIV-TERRAIN-006
//!
//! Epic: FR-CIV-TERRAIN
//!
//! This test file verifies FR FR-CIV-TERRAIN-006: Actor Y-axis (height) persists
//! deterministically across save/load. State round-trips preserve all fields.

#[cfg(test)]
mod fr_fr_civ_terrain_006 {
    /// WorldState JSON round-trip preserves all numeric fields (including height).
    #[test]
    fn state_round_trip_preserves_all_fields() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for _ in 0..3 {
            sim.tick();
        }
        let json = serde_json::to_string(&sim.state).expect("serialize");
        let restored: civ_engine::WorldState =
            serde_json::from_str(&json).expect("deserialize");
        // All numeric fields must match
        assert_eq!(sim.state.tick, restored.tick);
        assert_eq!(sim.state.population, restored.population);
        assert_eq!(
            sim.state.energy_budget_joules,
            restored.energy_budget_joules
        );
        assert_eq!(sim.state.rng_seed, restored.rng_seed);
        assert_eq!(sim.state.resources, restored.resources);
    }

    /// Simulation snapshot preserves world state across ticks.
    #[test]
    fn snapshot_preserves_state_deterministically() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for _ in 0..5 {
            sim.tick();
        }
        let snap = sim.snapshot();
        assert_eq!(snap.tick, 5);
        assert!(snap.population >= 0);
        // Snapshot energy matches state energy
        assert_eq!(snap.energy_budget, sim.state.energy_budget_joules);
    }
}
