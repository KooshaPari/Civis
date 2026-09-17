//! Tests for FR-THRY-001
//!
//! FR-THRY-001: Total Joule energy in a closed system SHALL be conserved each tick
//! (production - consumption - waste = 0).
//!
//! This verifies the invariant checker catches negative energy budgets and that
//! the economy ledger conservation check passes after a tick.

use civ_engine::invariants;
use civ_engine::Simulation;

#[cfg(test)]
mod fr_fr_thry_001 {
    use super::*;

    /// FR-THRY-001: After a tick, invariants pass (joule conservation holds).
    #[test]
    fn joule_conservation_after_tick() {
        let mut sim = Simulation::with_seed(42);
        sim.tick();
        // The invariant checker validates energy budget >= 0 and ledger conservation.
        // If joule conservation is violated, check_tick_invariants returns an error.
        let result = invariants::check_tick_invariants(&sim);
        assert!(
            result.is_ok(),
            "FR-THRY-001 joule conservation violated: {:?}",
            result.err()
        );
    }

    /// FR-THRY-001: Invariants hold across multiple consecutive ticks.
    #[test]
    fn joule_conservation_across_ticks() {
        let mut sim = Simulation::with_seed(100);
        for _ in 0..10 {
            sim.tick();
            let result = invariants::check_tick_invariants(&sim);
            assert!(
                result.is_ok(),
                "FR-THRY-001 violated at tick {}: {:?}",
                sim.state.tick,
                result.err()
            );
        }
    }
}
