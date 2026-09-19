//! Tests for FR-CIV-TERRAIN-001
//!
//! Epic: FR-CIV-TERRAIN
//!
//! This test file verifies FR FR-CIV-TERRAIN-001: Terrain playability smoke.
//! Simulation can be created from cold boot and tick successfully.

#[cfg(test)]
mod fr_fr_civ_terrain_001 {
    /// Simulation can be created from a seed (cold boot) and tick.
    #[test]
    fn cold_boot_simulation_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        assert_eq!(sim.state.tick, 0, "fresh sim starts at tick 0");
        sim.tick();
        assert_eq!(sim.state.tick, 1, "after tick, tick == 1");
    }

    /// Simulation has valid world state after creation.
    #[test]
    fn world_state_valid_at_creation() {
        let sim = civ_engine::Simulation::with_seed(42);
        assert!(sim.state.population >= 0);
        assert!(sim.state.energy_budget_joules >= civ_engine::Fixed::ZERO);
        assert!(!sim.state.factions.is_empty(), "should have factions");
    }
}
