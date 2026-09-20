//! Tests for FR-CIV-PERF-015
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-015: Spectator view is_day field.

#[cfg(test)]
mod fr_fr_civ_perf_015 {
    /// Verify FR-CIV-PERF-015: SpectatorView.is_day reflects climate day_phase.
    #[test]
    fn verify_fr_civ_perf_015_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let view = sim.spectator_view();
        // is_day is determined by climate day_phase in [0.25, 0.75)
        let day_phase = sim.climate().day_phase;
        let expected_is_day = (0.25..0.75).contains(&day_phase);
        assert_eq!(view.is_day, expected_is_day);
    }

    /// Verify spectator view buildings are populated.
    #[test]
    fn spectator_buildings_present() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let view = sim.spectator_view();
        // With 4 factions x 3 buildings each = 12 deterministic pins
        assert!(
            view.buildings.len() >= 12,
            "expected at least 12 building pins, got {}",
            view.buildings.len()
        );
    }
}
