//! CIV-0104 minimal tick invariant checks.
//!
//! Validates core world-state constraints after each tick. Full constraint-set
//! enforcement lives in the spec; this module starts with tick monotonicity,
//! non-negative population, non-negative energy budget, and economy ledger bounds.

use crate::engine::Simulation;
use crate::Fixed;

/// Invariant violation detected after a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvariantError {
    /// `state.tick` does not match the number of recorded tick markers.
    TickMonotonicity { tick: u64, recorded_ticks: usize },
    /// Population counter is negative (should be impossible for `u64`).
    NegativePopulation { population: u64 },
    /// Energy budget raw fixed-point value fell below zero.
    NegativeEnergyBudget { raw: i64 },
    /// `civ-economy` ledger conservation check failed.
    EconomyLedger(civ_economy::LedgerInvariantError),
}

/// Check minimal post-tick invariants on [`Simulation`].
///
/// - **Tick monotonicity:** `state.tick` equals the count of `ReplayEvent::Tick`
///   entries in the replay log (one marker per completed tick).
/// - **Population non-negative:** always true for `u64`; kept as an explicit guard.
/// - **Energy budget non-negative:** `energy_budget_joules.raw >= 0`.
/// - **Economy ledger:** when non-empty, macro budget ≥ 0, leg balance, and
///   `ledger.len() <= economy_state.tick * 2` via [`civ_economy::verify_ledger_conservation`].
pub fn check_tick_invariants(sim: &Simulation) -> Result<(), InvariantError> {
    use crate::replay::ReplayEvent;

    let recorded_ticks = sim
        .replay_log()
        .events
        .iter()
        .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
        .count();

    if recorded_ticks as u64 != sim.state.tick {
        return Err(InvariantError::TickMonotonicity {
            tick: sim.state.tick,
            recorded_ticks,
        });
    }

    // Population is `u64`; non-negativity is enforced by the type system.
    let _population = sim.state.population;

    if sim.state.energy_budget_joules.raw < Fixed::ZERO.raw {
        return Err(InvariantError::NegativeEnergyBudget {
            raw: sim.state.energy_budget_joules.raw,
        });
    }

    if !sim.economy_state.ledger.is_empty() {
        civ_economy::verify_ledger_conservation(&sim.economy_state)
            .map_err(InvariantError::EconomyLedger)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_economy::{EconomyState, LedgerEntry, LedgerInvariantError, ACCOUNT_CONSUMPTION};

    #[test]
    fn check_tick_invariants_accepts_simulation_with_economy_ledger() {
        let mut sim = Simulation::with_seed(104);
        sim.tick();
        assert!(!sim.economy_state.ledger.is_empty());
        check_tick_invariants(&sim).expect("ledger invariants after tick");
    }

    #[test]
    fn check_tick_invariants_rejects_oversized_ledger() {
        let mut sim = Simulation::with_seed(1);
        sim.tick();
        sim.economy_state.tick = 1;
        sim.economy_state.ledger.push(LedgerEntry {
            tick: 0,
            debit: 1,
            credit: 1,
            account: ACCOUNT_CONSUMPTION,
        });
        let err = check_tick_invariants(&sim).unwrap_err();
        assert_eq!(
            err,
            InvariantError::EconomyLedger(LedgerInvariantError::LedgerTooLarge {
                len: sim.economy_state.ledger.len(),
                tick: 1,
                max_len: 2,
            })
        );
    }

    #[test]
    fn empty_ledger_skips_growth_check() {
        let sim = Simulation::with_seed(2);
        assert!(sim.economy_state.ledger.is_empty());
        check_tick_invariants(&sim).expect("no ledger growth check when empty");
    }

    #[test]
    fn negative_economy_budget_rejected_when_ledger_nonempty() {
        let mut sim = Simulation::with_seed(3);
        sim.tick();
        sim.economy_state.energy_budget_joules = -1;
        let err = check_tick_invariants(&sim).unwrap_err();
        assert_eq!(
            err,
            InvariantError::EconomyLedger(LedgerInvariantError::NegativeBudget { budget: -1 })
        );
    }

    #[test]
    fn unbalanced_ledger_entry_rejected() {
        let mut state = EconomyState::with_energy_budget(10);
        state.tick = 1;
        state.ledger.push(LedgerEntry {
            tick: 0,
            debit: 5,
            credit: 3,
            account: ACCOUNT_CONSUMPTION,
        });
        assert_eq!(
            civ_economy::verify_ledger_conservation(&state),
            Err(LedgerInvariantError::UnbalancedEntry {
                index: 0,
                debit: 5,
                credit: 3,
            })
        );
    }
}
