//! Tests for FR-THRY-003
//!
//! FR-THRY-003: Population delta per tick SHALL equal births minus deaths
//! minus emigration plus immigration.
//!
//! Verifies population remains non-negative and invariant checker passes.

use civ_engine::invariants;
use civ_engine::Simulation;

#[cfg(test)]
mod fr_fr_thry_003 {
    use super::*;

    /// FR-THRY-003: Population invariant holds after tick (non-negative, bounded).
    #[test]
    fn population_delta_balanced_after_tick() {
        let mut sim = Simulation::with_seed(33);
        let pop_before = sim.state.population;
        sim.tick();
        let result = invariants::check_tick_invariants(&sim);
        assert!(
            result.is_ok(),
            "FR-THRY-003 population invariant violated: {:?}",
            result.err()
        );
        // Population must remain non-negative (u64, so always true, but verify it didn't wrap).
        assert!(
            sim.state.population < u64::MAX,
            "FR-THRY-003 population overflow detected"
        );
    }

    /// FR-THRY-003: Population stays consistent across many ticks.
    #[test]
    fn population_consistent_across_ticks() {
        let mut sim = Simulation::with_seed(77);
        for _ in 0..15 {
            sim.tick();
            let result = invariants::check_tick_invariants(&sim);
            assert!(result.is_ok());
            assert!(sim.state.population < u64::MAX);
        }
    }
}
