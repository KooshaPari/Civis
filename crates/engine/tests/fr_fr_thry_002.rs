//! Tests for FR-THRY-002
//!
//! FR-THRY-002: Total MilliCredit supply SHALL remain constant absent explicit
//! treasury mint/burn operations.
//!
//! Verifies economy ledger conservation includes credit balance checks.

use civ_engine::invariants;
use civ_engine::Simulation;

#[cfg(test)]
mod fr_fr_thry_002 {
    use super::*;

    /// FR-THRY-002: Economy ledger conservation passes after tick (credits balanced).
    #[test]
    fn credit_supply_conserved_after_tick() {
        let mut sim = Simulation::with_seed(55);
        sim.tick();
        let result = invariants::check_tick_invariants(&sim);
        assert!(
            result.is_ok(),
            "FR-THRY-002 credit supply conservation violated: {:?}",
            result.err()
        );
    }

    /// FR-THRY-002: Credit conservation holds across multiple ticks.
    #[test]
    fn credit_supply_conserved_across_ticks() {
        let mut sim = Simulation::with_seed(88);
        for _ in 0..20 {
            sim.tick();
            let result = invariants::check_tick_invariants(&sim);
            assert!(
                result.is_ok(),
                "FR-THRY-002 violated at tick {}: {:?}",
                sim.state.tick,
                result.err()
            );
        }
    }
}
