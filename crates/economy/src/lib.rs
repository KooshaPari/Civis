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

pub use allocation::{AllocationEngine, CapitalistAllocator};
pub use extraction::{
    find_extraction_site, tick_extraction, ExtractionSite, Extractor, ResourceKind,
};
pub use institution::{
    step_institutions, InstitutionAccount, InstitutionId, InstitutionKind, InstitutionLedger,
    InstitutionLedgerError, InstitutionPosting, LedgerSide, INSTITUTION_MARKET,
    INSTITUTION_TREASURY,
};
pub use market::MarketState;

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

/// Post an institution↔institution joule transfer.
///
/// Thin convenience wrapper over [`InstitutionLedger::post`] that takes
/// [`InstitutionId`]s directly (no [`LedgerSide`] plumbing) and returns the
/// resulting [`InstitutionPosting`] on success. The macro joule budget is
/// untouched — for macro↔institution transfers, call [`InstitutionLedger::post`]
/// directly with a [`LedgerSide::Macro`] leg.
///
/// Use this from the engine to exercise the institution ledger end-to-end
/// (e.g. `phase_economy` posting a per-tick treasury→market fee). The
/// institution ledger is otherwise dormant — [`step`] only seeds defaults.
pub fn transfer_joules(
    state: &mut EconomyState,
    from: InstitutionId,
    to: InstitutionId,
    joules: i64,
) -> Result<InstitutionPosting, InstitutionLedgerError> {
    if state.institutions.accounts.is_empty() {
        state.institutions = institution::InstitutionLedger::with_defaults();
    }
    // Move the ledger out so we can pass `&mut state` and `&mut self` together
    // (the underlying `post` borrows both). Same pattern as `step` above.
    let mut institutions = std::mem::take(&mut state.institutions);
    let result = institutions.post(
        state,
        LedgerSide::Institution(from),
        LedgerSide::Institution(to),
        joules,
    );
    state.institutions = institutions;
    result?;
    Ok(state
        .institutions
        .postings
        .last()
        .expect("posting was just pushed")
        .clone())
}

/// Verify both layers of the economy (macro joule budget + institution ledger)
/// in a single call. Returns the first violation encountered, macro first.
pub fn verify_economy_invariants(
    state: &EconomyState,
) -> Result<(), EconomyInvariantError> {
    if let Err(err) = verify_ledger_conservation(state) {
        return Err(EconomyInvariantError::Macro(err));
    }
    if let Err(err) = state.institutions.verify_conservation() {
        return Err(EconomyInvariantError::Institution(err));
    }
    Ok(())
}

/// Combined macro + institution invariant violation (see [`verify_economy_invariants`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EconomyInvariantError {
    /// Macro joule budget or macro ledger violation.
    Macro(LedgerInvariantError),
    /// Institution ledger violation.
    Institution(InstitutionLedgerError),
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
    /// CIV-002 P1 allocator slice: `step` runs the auction, posts balanced
    /// institution transfers, and conserves total joule budget end-to-end.
    /// Covers FR-ECON-001.
    #[test]
    fn step_integrates_allocator_with_conservation() {
        use crate::{Bid, Offer, INSTITUTION_MARKET, INSTITUTION_TREASURY};

        let mut state = EconomyState::with_energy_budget(2_000);
        // Seed the institution ledger (market + treasury accounts) so we can
        // post to it. Mirrors what `step` would do lazily on the first tick.
        crate::institution::step_institutions(&mut state);
        // Fund both institutions so trades can clear.
        let mut ledger = state.institutions.clone();
        ledger
            .post(
                &mut state,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                1_000,
            )
            .unwrap();
        state.institutions = ledger;
        let mut ledger = state.institutions.clone();
        ledger
            .post(
                &mut state,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                1_000,
            )
            .unwrap();
        state.institutions = ledger;

        // Post a crossing pair: treasury bids 5 units of food at 120, market
        // offers 5 at 80 → mid-point 100.
        let mut allocator = std::mem::take(&mut state.allocator);
        allocator
            .post_bid(Bid {
                id: 0,
                bidder: INSTITUTION_TREASURY,
                good: "food".to_string(),
                quantity: 5,
                price: 120,
            })
            .unwrap();
        allocator
            .post_offer(Offer {
                id: 0,
                offerer: INSTITUTION_MARKET,
                good: "food".to_string(),
                quantity: 5,
                price: 80,
            })
            .unwrap();
        state.allocator = allocator;

        let macro_before = state.energy_budget_joules;
        step(&mut state);

        // After one step: tick=1, the trade posted a 500-unit transfer from
        // treasury → market, both institution balances are non-negative, the
        // macro joule budget is unchanged by the auction itself.
        assert_eq!(state.tick, 1);
        assert!(state.institutions.institution_balance(INSTITUTION_TREASURY) >= 0);
        assert!(state.institutions.institution_balance(INSTITUTION_MARKET) >= 0);
        assert_eq!(state.energy_budget_joules, macro_before);
        state
            .institutions
            .verify_conservation()
            .expect("conservation after allocator integration");
    }

    /// Property: across N consecutive `step` calls, the macro joule budget
    /// plus the sum of institution joule balances is invariant (no joules
    /// created or destroyed by the allocator). This is the heart of
    /// FR-ECON-002.
    #[test]
    fn step_conserves_total_joules_across_many_ticks() {
        use crate::{Bid, Offer, INSTITUTION_MARKET, INSTITUTION_TREASURY};

        let mut state = EconomyState::with_energy_budget(20_000);
        // Seed the institution ledger and distribute starting joules across
        // both institutions and the macro budget in a known ratio so we can
        // detect any leak.
        crate::institution::step_institutions(&mut state);
        let mut ledger = state.institutions.clone();
        ledger
            .post(
                &mut state,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                5_000,
            )
            .unwrap();
        state.institutions = ledger;
        let mut ledger = state.institutions.clone();
        ledger
            .post(
                &mut state,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                5_000,
            )
            .unwrap();
        state.institutions = ledger;
        // Macro budget now 10_000, treasury 5_000, market 5_000 → 20_000 total.
        let total_before: i64 = state.energy_budget_joules
            + state.institutions.institution_balance(INSTITUTION_TREASURY)
            + state.institutions.institution_balance(INSTITUTION_MARKET);

        // Post a small order book that will partially clear and partially
        // ration across multiple ticks.
        let mut allocator = std::mem::take(&mut state.allocator);
        for i in 0..4 {
            allocator
                .post_bid(Bid {
                    id: 0,
                    bidder: INSTITUTION_TREASURY,
                    good: "food".to_string(),
                    quantity: 3 + i,
                    price: 150 - i * 20,
                })
                .unwrap();
            allocator
                .post_offer(Offer {
                    id: 0,
                    offerer: INSTITUTION_MARKET,
                    good: "food".to_string(),
                    quantity: 2 + i,
                    price: 80 + i * 10,
                })
                .unwrap();
        }
        state.allocator = allocator;

        for _ in 0..10 {
            step(&mut state);
            let total: i64 = state.energy_budget_joules
                + state.institutions.institution_balance(INSTITUTION_TREASURY)
                + state.institutions.institution_balance(INSTITUTION_MARKET);
            assert_eq!(total, total_before, "joules leaked during economy step");
            assert!(
                state.energy_budget_joules >= 0,
                "macro budget went negative"
            );
            assert!(
                state.institutions.institution_balance(INSTITUTION_TREASURY) >= 0,
                "treasury went negative"
            );
            assert!(
                state.institutions.institution_balance(INSTITUTION_MARKET) >= 0,
                "market went negative"
            );
        }
    }

    /// `transfer_joules` is the public institution↔institution transfer API.
    /// It seeds defaults lazily, debits the source, credits the destination,
    /// and returns the resulting posting — without touching the macro joule
    /// budget.
    #[test]
    fn transfer_joules_moves_joules_between_institutions() {
        use crate::institution::InstitutionLedger;
        let mut state = EconomyState::with_energy_budget(100);
        // Seed and fund Treasury from the macro budget. Market is lazy-seeded
        // by `transfer_joules` itself, so we don't pre-fund it.
        let mut ledger = InstitutionLedger::with_defaults();
        ledger
            .post(
                &mut state,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                100,
            )
            .unwrap();
        state.institutions = ledger;

        // First `transfer_joules` call lazy-seeds defaults. We drain to
        // verify the path.
        let macro_before = state.energy_budget_joules;
        let posting = transfer_joules(&mut state, INSTITUTION_TREASURY, INSTITUTION_MARKET, 40)
            .expect("transfer");
        assert_eq!(posting.amount, 40);
        assert_eq!(
            posting.debit,
            LedgerSide::Institution(INSTITUTION_TREASURY)
        );
        assert_eq!(posting.credit, LedgerSide::Institution(INSTITUTION_MARKET));
        assert_eq!(posting.tick, 0);
        assert_eq!(state.energy_budget_joules, macro_before);
        assert_eq!(
            state.institutions.institution_balance(INSTITUTION_TREASURY),
            60
        );
        assert_eq!(
            state.institutions.institution_balance(INSTITUTION_MARKET),
            40
        );
        verify_economy_invariants(&state).expect("end-to-end conservation");
    }

    /// `transfer_joules` rejects self-transfers (same institution on both sides).
    #[test]
    fn transfer_joules_rejects_self_transfer() {
        let mut state = EconomyState::with_energy_budget(0);
        let err = transfer_joules(&mut state, INSTITUTION_TREASURY, INSTITUTION_TREASURY, 10)
            .expect_err("self-transfer rejected");
        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Institution(INSTITUTION_TREASURY),
                amount: 10,
            }
        );
        assert!(state.institutions.postings.is_empty());
    }

    /// `transfer_joules` rejects over-debit (source balance too small).
    #[test]
    fn transfer_joules_rejects_overdebit() {
        let mut state = EconomyState::with_energy_budget(0);
        // Treasury defaults to 0 — a 10-joule debit must fail.
        let err = transfer_joules(&mut state, INSTITUTION_TREASURY, INSTITUTION_MARKET, 10)
            .expect_err("insufficient treasury");
        assert_eq!(
            err,
            InstitutionLedgerError::NegativeInstitutionBalance {
                id: INSTITUTION_TREASURY,
                before: 0,
                requested: 10,
            }
        );
    }

    /// `verify_economy_invariants` reports macro-side violations first, then
    /// institution-side ones, when both layers are dirty.
    #[test]
    fn verify_economy_invariants_reports_macro_violation_first() {
        let mut state = EconomyState::with_energy_budget(-1);
        // Hand-craft a self-posting in the institution layer too.
        state.institutions.postings.push(InstitutionPosting {
            tick: 0,
            debit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            amount: 1,
        });
        match verify_economy_invariants(&state) {
            Err(EconomyInvariantError::Macro(LedgerInvariantError::NegativeBudget { budget })) => {
                assert_eq!(budget, -1);
            }
            other => panic!("expected macro violation, got {other:?}"),
        }
    }
}
