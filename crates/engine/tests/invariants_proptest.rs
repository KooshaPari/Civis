//! Property-based economy invariant checks after simulation ticks.

use civ_economy::verify_ledger_conservation;
use civ_engine::{Fixed, Simulation};
use proptest::prelude::*;

proptest! {
    /// After 1..20 ticks, macro ledger conservation holds and energy budget stays non-negative.
    #[test]
    fn ledger_conservation_and_energy_budget_after_ticks(
        seed in any::<u64>(),
        n in 1usize..=20,
    ) {
        let mut sim = Simulation::with_seed(seed);
        for _ in 0..n {
            sim.tick();

            prop_assert!(
                sim.state.energy_budget_joules.raw >= 0,
                "world energy budget negative after tick {}: raw={}",
                sim.state.tick,
                sim.state.energy_budget_joules.raw,
            );
            prop_assert!(
                sim.economy_state.energy_budget_joules >= 0,
                "economy energy budget negative after tick {}: {}",
                sim.state.tick,
                sim.economy_state.energy_budget_joules,
            );
            prop_assert_eq!(
                sim.state.energy_budget_joules,
                Fixed::from_num(sim.economy_state.energy_budget_joules),
                "world and economy energy budgets diverged after tick {}",
                sim.state.tick,
            );

            verify_ledger_conservation(&sim.economy_state).map_err(|e| {
                TestCaseError::fail(format!(
                    "verify_ledger_conservation failed after tick {}: {e:?}",
                    sim.state.tick,
                ))
            })?;
        }
    }
}
