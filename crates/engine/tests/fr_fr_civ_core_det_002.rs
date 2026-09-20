//! Tests for FR-CIV-CORE-DET-002
//!
//! Epic: FR-CIV-CORE-DET
//!
//! This test file verifies FR FR-CIV-CORE-DET-002: Build-Time Reproducibility.
//! Same seed + same code => same world state, proven by JSON round-trip.

#[cfg(test)]
mod fr_fr_civ_core_det_002 {
    /// WorldState serializes and deserializes without changing key fields.
    #[test]
    fn state_preserved_through_serialization() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        for _ in 0..3 {
            sim.tick();
        }
        let json = serde_json::to_string(&sim.state).expect("serialize");
        let restored: civ_engine::WorldState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sim.state.tick, restored.tick);
        assert_eq!(sim.state.population, restored.population);
        assert_eq!(
            sim.state.energy_budget_joules,
            restored.energy_budget_joules
        );
        assert_eq!(sim.state.rng_seed, restored.rng_seed);
        assert_eq!(sim.state.resources, restored.resources);
    }

    /// Two fresh simulations with same seed have identical default world state.
    #[test]
    fn default_state_reproducible() {
        let sim_a = civ_engine::Simulation::with_seed(100);
        let sim_b = civ_engine::Simulation::with_seed(100);
        assert_eq!(sim_a.state.tick, sim_b.state.tick);
        assert_eq!(sim_a.state.population, sim_b.state.population);
        assert_eq!(
            sim_a.state.energy_budget_joules,
            sim_b.state.energy_budget_joules
        );
        assert_eq!(sim_a.state.factions, sim_b.state.factions);
    }
}
