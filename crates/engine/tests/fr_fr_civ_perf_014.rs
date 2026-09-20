//! Tests for FR-CIV-PERF-014
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-014: Spectator view construction.

#[cfg(test)]
mod fr_fr_civ_perf_014 {
    /// Verify FR-CIV-PERF-014: SpectatorView can be constructed from simulation.
    #[test]
    fn verify_fr_civ_perf_014_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let view = sim.spectator_view();
        assert!(
            view.civ_pins.len() > 0,
            "spectator view should have civilian pins"
        );
        assert_eq!(view.factions.len(), 4, "should have 4 factions");
    }

    /// Verify spectator view faction ids are sequential.
    #[test]
    fn spectator_faction_ids_sequential() {
        let sim = civ_engine::Simulation::with_seed(1u64);
        let view = sim.spectator_view();
        for (idx, faction) in view.factions.iter().enumerate() {
            assert_eq!(faction.id, idx as u32);
        }
    }
}
