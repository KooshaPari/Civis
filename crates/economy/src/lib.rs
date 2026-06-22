//! civ-economy — conservation-complete economy layer (CIV-0100 / CIV-0107).
//!
//! Target: double-entry ledger, allocation engines, district production, and
//! conservation invariants. `civ-engine::Simulation::phase_economy` syncs joule
//! budget into [`EconomyState`], calls [`drain_energy_budget`] and [`step`], then
//! writes back to `WorldState`.
//!
//! See `docs/specs/CIV-0100-economy-v1.md` and `docs/traceability/TRACEABILITY_MATRIX.md`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod allocation;
mod extraction;
mod institution;
mod market;
mod stocks;

pub use allocation::{AllocationEngine, CapitalistAllocator};
pub use extraction::{
    find_extraction_site, tick_extraction, ExtractionSite, Extractor, ResourceKind,
};
pub use institution::{
    collect_taxes, step_institutions, InstitutionAccount, InstitutionId, InstitutionKind,
    InstitutionLedger, InstitutionLedgerError, InstitutionPosting, LedgerSide, Taxation,
    INSTITUTION_MARKET, INSTITUTION_TREASURY,
};
pub use market::{GoodId, MarketState, MultiGoodMarket, Order, OrderBook, Side, Trade};
pub use stocks::Stocks;

use serde::{Deserialize, Serialize};

/// Schema version for `civ-economy`. Bumped on breaking snapshot / ledger changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Stub ledger account id (district / actor accounts land in CIV-0100 follow-up).
pub type AccountId = u32;

/// Global macro energy budget account.
pub const ACCOUNT_ENERGY_BUDGET: AccountId = 0;
/// Aggregate consumption / policy drain account.
pub const ACCOUNT_CONSUMPTION: AccountId = 1;

/// Bookkeeping row for a single ledger leg (stub; full double-entry pairs in CIV-0100 §3d).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Simulation tick when the entry was recorded.
    pub tick: u64,
    /// Debit amount (joules) for this leg.
    pub debit: i64,
    /// Credit amount (joules) for this leg.
    pub credit: i64,
    /// Account this leg posts to.
    pub account: AccountId,
}

/// Macro economy state (stub). District ledgers and allocation engines land in
/// follow-up work per CIV-0100 §Rust module layout.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EconomyState {
    /// Global joule balance in integer joules (no floating-point accumulation).
    pub energy_budget_joules: i64,
    /// Economy phase tick (advanced by [`step`]).
    pub tick: u64,
    /// Append-only bookkeeping log (stub).
    pub ledger: Vec<LedgerEntry>,
    /// Institution accounts and posting log (CIV-0100 §3d stub).
    #[serde(default)]
    pub institutions: InstitutionLedger,
    /// Budget at the previous [`step`] boundary (tick-close reconciliation).
    #[serde(default)]
    last_step_budget_joules: i64,
}

impl EconomyState {
    /// Create state with `energy_budget_joules` and an aligned tick-close baseline.
    pub fn with_energy_budget(energy_budget_joules: i64) -> Self {
        Self {
            energy_budget_joules,
            last_step_budget_joules: energy_budget_joules,
            ..Default::default()
        }
    }
}

fn push_ledger_entry(state: &mut EconomyState, debit: i64, credit: i64, account: AccountId) {
    debug_assert!(debit >= 0 && credit >= 0);
    state.ledger.push(LedgerEntry {
        tick: state.tick,
        debit,
        credit,
        account,
    });
}

/// Apply aggregate joule consumption (FR-ECON-001 engine path). Budget only decreases;
/// result is clamped to zero. Records a consumption ledger leg when joules are drained.
pub fn drain_energy_budget(state: &mut EconomyState, consumption_joules: i64) {
    if consumption_joules <= 0 {
        return;
    }
    let before = state.energy_budget_joules;
    let applied = consumption_joules.min(before);
    if applied == 0 {
        return;
    }
    state.energy_budget_joules = before - applied;
    push_ledger_entry(state, applied, applied, ACCOUNT_CONSUMPTION);
}

/// Ledger / budget invariant violation (CIV-0100 conservation checks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerInvariantError {
    /// Macro joule budget fell below zero.
    NegativeBudget {
        /// Observed budget (joules).
        budget: i64,
    },
    /// Ledger grew faster than the per-tick posting bound allows.
    LedgerTooLarge {
        /// Current ledger length.
        len: usize,
        /// Economy tick used for the bound (`tick * 2`).
        tick: u64,
        /// Maximum allowed length at this tick.
        max_len: usize,
    },
    /// A ledger leg has unequal debit and credit (stub double-entry must balance).
    UnbalancedEntry {
        /// Index of the offending entry in [`EconomyState::ledger`].
        index: usize,
        /// Debit amount on the leg.
        debit: i64,
        /// Credit amount on the leg.
        credit: i64,
    },
}

/// Verify macro budget and, when the ledger is non-empty, growth and leg balance.
///
/// Posting bound: at most two legs per economy tick (consumption drain + tick-close).
pub fn verify_ledger_conservation(state: &EconomyState) -> Result<(), LedgerInvariantError> {
    if state.energy_budget_joules < 0 {
        return Err(LedgerInvariantError::NegativeBudget {
            budget: state.energy_budget_joules,
        });
    }

    if state.ledger.is_empty() {
        return Ok(());
    }

    let max_len = state
        .tick
        .saturating_mul(2)
        .try_into()
        .unwrap_or(usize::MAX);
    let len = state.ledger.len();
    if len > max_len {
        return Err(LedgerInvariantError::LedgerTooLarge {
            len,
            tick: state.tick,
            max_len,
        });
    }

    for (index, entry) in state.ledger.iter().enumerate() {
        if entry.debit != entry.credit {
            return Err(LedgerInvariantError::UnbalancedEntry {
                index,
                debit: entry.debit,
                credit: entry.credit,
            });
        }
    }

    Ok(())
}

/// Advance one economy tick. Runs the institution stub pass, appends a tick-close
/// bookkeeping entry when the budget changed since the previous step, then advances
/// [`EconomyState::tick`].
pub fn step(state: &mut EconomyState) {
    step_institutions(state);

    if state.energy_budget_joules != state.last_step_budget_joules {
        let delta = state.last_step_budget_joules - state.energy_budget_joules;
        let amount = delta.abs();
        push_ledger_entry(state, amount, amount, ACCOUNT_ENERGY_BUDGET);
    }
    state.last_step_budget_joules = state.energy_budget_joules;
    state.tick = state.tick.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CIV-0100 — schema version is exposed for persistence / replay alignment.
    #[test]
    fn schema_version_present() {
        assert_eq!(SCHEMA_VERSION, 1);
    }

    #[test]
    fn drain_energy_budget_records_ledger_entry() {
        let mut state = EconomyState::with_energy_budget(100);
        drain_energy_budget(&mut state, 40);
        assert_eq!(state.energy_budget_joules, 60);
        assert_eq!(state.ledger.len(), 1);
        let entry = &state.ledger[0];
        assert_eq!(entry.tick, 0);
        assert_eq!(entry.debit, 40);
        assert_eq!(entry.credit, 40);
        assert_eq!(entry.account, ACCOUNT_CONSUMPTION);
    }

    /// Conservation: aggregate joule budget never goes negative after drain.
    #[test]
    fn drain_energy_budget_clamps_at_zero() {
        let mut state = EconomyState::with_energy_budget(50);
        drain_energy_budget(&mut state, 100);
        assert_eq!(state.energy_budget_joules, 0);
        assert!(state.energy_budget_joules >= 0);
        assert_eq!(state.ledger.len(), 1);
        assert_eq!(state.ledger[0].debit, 50);
    }

    #[test]
    fn step_appends_entry_when_budget_changed() {
        let mut state = EconomyState::with_energy_budget(100);
        drain_energy_budget(&mut state, 25);
        step(&mut state);
        assert_eq!(state.tick, 1);
        assert_eq!(state.ledger.len(), 2);
        let close = &state.ledger[1];
        assert_eq!(close.account, ACCOUNT_ENERGY_BUDGET);
        assert_eq!(close.debit, 25);
        assert_eq!(close.credit, 25);
        verify_ledger_conservation(&state).expect("conservation after drain + step");
    }

    #[test]
    fn verify_ledger_conservation_rejects_oversized_ledger() {
        let mut state = EconomyState::with_energy_budget(10);
        state.tick = 1;
        state.ledger = vec![
            LedgerEntry {
                tick: 0,
                debit: 1,
                credit: 1,
                account: ACCOUNT_CONSUMPTION,
            },
            LedgerEntry {
                tick: 0,
                debit: 1,
                credit: 1,
                account: ACCOUNT_CONSUMPTION,
            },
            LedgerEntry {
                tick: 0,
                debit: 1,
                credit: 1,
                account: ACCOUNT_CONSUMPTION,
            },
        ];
        assert_eq!(
            verify_ledger_conservation(&state),
            Err(LedgerInvariantError::LedgerTooLarge {
                len: 3,
                tick: 1,
                max_len: 2,
            })
        );
    }
}
