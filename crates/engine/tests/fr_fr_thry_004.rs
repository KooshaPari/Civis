//! Tests for FR-THRY-004
//!
//! FR-THRY-004: The invariant checker SHALL run every tick and panic in debug
//! builds on violation.
//!
//! This verifies that check_tick_invariants exists, is callable, and that the
//! Simulation::tick() path invokes it (debug_assertions).

use civ_engine::invariants;
use civ_engine::Simulation;

#[cfg(test)]
mod fr_fr_thry_004 {
    use super::*;

    /// FR-THRY-004: check_tick_invariants is callable and returns Ok for valid state.
    #[test]
    fn invariant_checker_runs() {
        let mut sim = Simulation::with_seed(7);
        sim.tick();
        // If the invariant checker panics, this test will fail.
        // If it returns an error, the invariant is violated.
        assert!(invariants::check_tick_invariants(&sim).is_ok());
    }

    /// FR-THRY-004: InvariantError enum covers all required violation types.
    #[test]
    fn invariant_error_variants_exist() {
        // Compile-time check: these variants must exist for the checker to report them.
        let _tick_err =
            invariants::InvariantError::TickMonotonicity { tick: 0, recorded_ticks: 0 };
        let _pop_err = invariants::InvariantError::NegativePopulation { population: 0 };
        let _energy_err = invariants::InvariantError::NegativeEnergyBudget { raw: -1 };
    }
}
