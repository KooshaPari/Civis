//! Institution accounts and double-entry posting (CIV-0100 §3d stub).
//!
//! Posts balanced pairs between [`InstitutionId`] accounts and macro [`AccountId`]s.
//! Full district / state-actor wiring lands in follow-up work.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{AccountId, EconomyState, ACCOUNT_ENERGY_BUDGET};

/// Institution identifier (district / state actor account).
pub type InstitutionId = u32;

/// Well-known institution ids for the default stub ledger.
pub const INSTITUTION_MARKET: InstitutionId = 1;
/// Treasury institution (fiscal pool stub).
pub const INSTITUTION_TREASURY: InstitutionId = 2;

/// Institution role in the economy layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// Market clearing / exchange institution.
    Market,
    /// State fiscal treasury.
    Treasury,
}

/// One leg of an institution ↔ macro posting (debit side or credit side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LedgerSide {
    /// Macro ledger account ([`AccountId`]).
    Macro(AccountId),
    /// Institution account ([`InstitutionId`]).
    Institution(InstitutionId),
}

/// Completed double-entry posting between institution and macro accounts.
///
/// Carries independent debit-side and credit-side amounts (both in joules).
/// In a healthy ledger they MUST be equal: every joule debited from one
/// account is credited to another. Separating the two amounts lets
/// [`InstitutionLedger::verify_conservation`] detect tampered postings
/// (e.g. replayed from a bad disk row) that bypass [`InstitutionLedger::post`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionPosting {
    /// Simulation tick when the posting was recorded.
    pub tick: u64,
    /// Account debited (balance decreases).
    pub debit: LedgerSide,
    /// Joules debited from `debit`.
    #[serde(default)]
    pub debit_amount: i64,
    /// Account credited (balance increases).
    pub credit: LedgerSide,
    /// Joules credited to `credit`.
    #[serde(default)]
    pub credit_amount: i64,
    /// Caller-supplied amount (kept for diagnostics; not used in conservation math).
    /// Legacy postings serialized before the split-amount fields landed default to 0 —
    /// the conservation invariants treat those as "amount unspecified" and reconstruct
    /// a balanced pair from the legacy `amount` field.
    #[serde(default)]
    pub amount: i64,
}

/// Per-institution joule balance (non-negative unless explicitly permitted later).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionAccount {
    /// Institution id.
    pub id: InstitutionId,
    /// Role of this institution.
    pub kind: InstitutionKind,
    /// Joule balance held by the institution.
    pub balance_joules: i64,
}

/// Institution-layer ledger: balances plus append-only posting log.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InstitutionLedger {
    /// Institution id → account state.
    pub accounts: BTreeMap<InstitutionId, InstitutionAccount>,
    /// Append-only double-entry postings for this layer.
    pub postings: Vec<InstitutionPosting>,
}

/// Per-institution tax rate in basis points (1 bp = 0.01 %).
///
/// Stored in a dedicated struct so the engine / scenario layer can wire it
/// into the simulation without leaking it into the [`InstitutionLedger`]
/// invariants (which deal in joule balances).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Taxation {
    /// Institution id → tax rate in basis points applied to the macro joule budget
    /// at every `phase_economy` collection pass.
    pub rates_bp: BTreeMap<InstitutionId, u32>,
    /// Maximum joules collectable per institution per pass (saturating guard).
    /// `None` means unbounded.
    #[serde(default)]
    pub per_institution_cap: Option<i64>,
}

/// Summary of a tax collection pass (FR-ECON-004).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TaxCollection {
    /// Total joules debited from the macro budget and credited to receiving institutions.
    pub total_collected_joules: i64,
    /// Per-institution joules collected in this pass.
    pub per_institution_joules: BTreeMap<InstitutionId, i64>,
    /// Number of institutions whose demand was capped by `per_institution_cap`.
    pub capped_institutions: u32,
}

/// Institution posting or balance invariant violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstitutionLedgerError {
    /// Transfer amount must be strictly positive.
    NonPositiveAmount {
        /// Attempted amount (joules).
        amount: i64,
    },
    /// Unknown institution id.
    UnknownInstitution {
        /// Missing institution id.
        id: InstitutionId,
    },
    /// Macro account has insufficient joules for a debit.
    InsufficientMacroBalance {
        /// Macro account id.
        account: AccountId,
        /// Available joules.
        available: i64,
        /// Requested debit (joules).
        requested: i64,
    },
    /// Institution balance would go negative.
    NegativeInstitutionBalance {
        /// Institution id.
        id: InstitutionId,
        /// Balance before posting (joules).
        before: i64,
        /// Requested debit (joules).
        requested: i64,
    },
    /// Aggregate debits ≠ aggregate credits across postings.
    UnbalancedPostings {
        /// Sum of debited amounts.
        debits: i64,
        /// Sum of credited amounts.
        credits: i64,
    },
    /// A posting credits the same account it debits (no joules actually moved).
    SelfPosting {
        /// The side that was both debited and credited.
        side: LedgerSide,
        /// Amount that would have moved (always > 0 for a real self-posting).
        amount: i64,
    },
    /// Per-posting amount integrity violation: debit-side amount does not equal
    /// credit-side amount (a posting was constructed or replayed with mismatched legs).
    TamperedAmount {
        /// Tick the offending posting was recorded on.
        tick: u64,
        /// Index of the posting in [`InstitutionLedger::postings`].
        index: usize,
        /// Debit-side joules recorded on the posting.
        debit_amount: i64,
        /// Credit-side joules recorded on the posting.
        credit_amount: i64,
    },
    /// A posting has a non-positive debit-side or credit-side amount.
    NonPositivePostingAmount {
        /// Tick the offending posting was recorded on.
        tick: u64,
        /// Index of the posting in [`InstitutionLedger::postings`].
        index: usize,
        /// Debit-side joules recorded on the posting.
        debit_amount: i64,
        /// Credit-side joules recorded on the posting.
        credit_amount: i64,
    },
}

impl InstitutionLedger {
    /// Default stub ledger: Market + Treasury at zero balance.
    pub fn with_defaults() -> Self {
        let mut accounts = BTreeMap::new();
        accounts.insert(
            INSTITUTION_MARKET,
            InstitutionAccount {
                id: INSTITUTION_MARKET,
                kind: InstitutionKind::Market,
                balance_joules: 0,
            },
        );
        accounts.insert(
            INSTITUTION_TREASURY,
            InstitutionAccount {
                id: INSTITUTION_TREASURY,
                kind: InstitutionKind::Treasury,
                balance_joules: 0,
            },
        );
        Self {
            accounts,
            postings: Vec::new(),
        }
    }

    /// Current joule balance for an institution (0 if unknown).
    pub fn institution_balance(&self, id: InstitutionId) -> i64 {
        self.accounts
            .get(&id)
            .map(|a| a.balance_joules)
            .unwrap_or(0)
    }

    /// Post a balanced transfer: debit side pays, credit side receives.
    pub fn post(
        &mut self,
        economy: &mut EconomyState,
        debit: LedgerSide,
        credit: LedgerSide,
        amount: i64,
    ) -> Result<(), InstitutionLedgerError> {
        if amount <= 0 {
            return Err(InstitutionLedgerError::NonPositiveAmount { amount });
        }
        if debit == credit {
            return Err(InstitutionLedgerError::SelfPosting { side: debit, amount });
        }

        self.apply_debit(economy, debit, amount)?;
        self.apply_credit(economy, credit, amount)?;

        self.postings.push(InstitutionPosting {
            tick: economy.tick,
            debit,
            debit_amount: amount,
            credit,
            credit_amount: amount,
            amount,
        });

        Ok(())
    }

    fn apply_debit(
        &mut self,
        economy: &mut EconomyState,
        side: LedgerSide,
        amount: i64,
    ) -> Result<(), InstitutionLedgerError> {
        match side {
            LedgerSide::Macro(account) => {
                let available = macro_balance(economy, account);
                if available < amount {
                    return Err(InstitutionLedgerError::InsufficientMacroBalance {
                        account,
                        available,
                        requested: amount,
                    });
                }
                set_macro_balance(economy, account, available - amount);
            }
            LedgerSide::Institution(id) => {
                let account = self
                    .accounts
                    .get_mut(&id)
                    .ok_or(InstitutionLedgerError::UnknownInstitution { id })?;
                if account.balance_joules < amount {
                    return Err(InstitutionLedgerError::NegativeInstitutionBalance {
                        id,
                        before: account.balance_joules,
                        requested: amount,
                    });
                }
                account.balance_joules -= amount;
            }
        }
        Ok(())
    }

    fn apply_credit(
        &mut self,
        economy: &mut EconomyState,
        side: LedgerSide,
        amount: i64,
    ) -> Result<(), InstitutionLedgerError> {
        match side {
            LedgerSide::Macro(account) => {
                let balance = macro_balance(economy, account);
                set_macro_balance(economy, account, balance.saturating_add(amount));
            }
            LedgerSide::Institution(id) => {
                let account = self
                    .accounts
                    .get_mut(&id)
                    .ok_or(InstitutionLedgerError::UnknownInstitution { id })?;
                account.balance_joules = account.balance_joules.saturating_add(amount);
            }
        }
        Ok(())
    }

    /// Verify aggregate debits equal aggregate credits and institution balances are non-negative.
    ///
    /// Walks the posting log and rejects:
    /// - postings with mismatched debit-side / credit-side amounts
    ///   ([`InstitutionLedgerError::TamperedAmount`]),
    /// - postings with non-positive debit-side or credit-side amounts
    ///   ([`InstitutionLedgerError::NonPositivePostingAmount`]),
    /// - postings where the same account is on both sides
    ///   ([`InstitutionLedgerError::SelfPosting`]),
    /// - aggregate debit/credit totals that drift apart
    ///   ([`InstitutionLedgerError::UnbalancedPostings`]),
    /// - any institution balance that has gone negative
    ///   ([`InstitutionLedgerError::NegativeInstitutionBalance`]).
    pub fn verify_conservation(&self) -> Result<(), InstitutionLedgerError> {
        let mut debits: i64 = 0;
        let mut credits: i64 = 0;

        for (index, posting) in self.postings.iter().enumerate() {
            // Per-posting amount integrity: a tampered posting has debit_amount != credit_amount.
            // Legacy postings (serialized before the split-amount fields landed) have both = 0
            // and `amount` as the canonical figure. Treat those as balanced by construction.
            let (debit_amount, credit_amount) = if posting.debit_amount == 0
                && posting.credit_amount == 0
                && posting.amount > 0
            {
                (posting.amount, posting.amount)
            } else {
                (posting.debit_amount, posting.credit_amount)
            };

            if debit_amount != credit_amount {
                return Err(InstitutionLedgerError::TamperedAmount {
                    tick: posting.tick,
                    index,
                    debit_amount,
                    credit_amount,
                });
            }
            if debit_amount <= 0 || credit_amount <= 0 {
                return Err(InstitutionLedgerError::NonPositivePostingAmount {
                    tick: posting.tick,
                    index,
                    debit_amount,
                    credit_amount,
                });
            }
            if posting.debit == posting.credit {
                return Err(InstitutionLedgerError::SelfPosting {
                    side: posting.debit,
                    amount: debit_amount,
                });
            }

            debits = debits.saturating_add(debit_amount);
            credits = credits.saturating_add(credit_amount);
        }

        if debits != credits {
            return Err(InstitutionLedgerError::UnbalancedPostings { debits, credits });
        }

        for account in self.accounts.values() {
            if account.balance_joules < 0 {
                return Err(InstitutionLedgerError::NegativeInstitutionBalance {
                    id: account.id,
                    before: account.balance_joules,
                    requested: 0,
                });
            }
        }

        Ok(())
    }
}

/// Collect taxes for one economy tick (FR-ECON-004).
///
/// For each institution with a rate in `taxation`, compute
/// `joules = (budget * rate_bp) / 10_000` and post a balanced transfer
/// from the macro energy budget to that institution's account.
///
/// Returns a [`TaxCollection`] summary regardless of how many institutions
/// actually had a rate configured (no rate ⇒ no joules moved).
pub fn collect_taxes(
    state: &mut EconomyState,
    taxation: &Taxation,
) -> Result<TaxCollection, InstitutionLedgerError> {
    // Lazy-seed defaults so callers don't have to run `step_institutions` first.
    if state.institutions.accounts.is_empty() {
        state.institutions = InstitutionLedger::with_defaults();
    }

    let mut summary = TaxCollection::default();
    let mut grand_total: i64 = 0;
    let mut capped: u32 = 0;

    // Snapshot the rates so we can mutate the ledger inside the loop without
    // borrowing `taxation` for the whole duration.
    let rates: Vec<(InstitutionId, u32)> = taxation
        .rates_bp
        .iter()
        .map(|(id, rate)| (*id, *rate))
        .collect();

    for (institution_id, rate_bp) in rates {
        if rate_bp == 0 {
            continue;
        }
        let budget = state.energy_budget_joules;
        if budget <= 0 {
            break; // No joules to collect; stop early.
        }
        // Integer-only arithmetic: floor at 0, no overflow on u32.
        let mut joules = budget.saturating_mul(rate_bp as i64) / 10_000;
        if let Some(cap) = taxation.per_institution_cap {
            if joules > cap {
                joules = cap;
                capped = capped.saturating_add(1);
            }
        }
        if joules <= 0 {
            continue;
        }
        // We can't call `state.institutions.post(&mut state, ...)` here because
        // the borrow checker would see two simultaneous `&mut` borrows of `state`.
        // Re-implement the post inline using the public + private helpers, which
        // is safe because we already validated `joules > 0` and have a fresh
        // `budget` snapshot above.
        let debit = LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET);
        let credit = LedgerSide::Institution(institution_id);
        if debit == credit {
            // Defensive: same-side debit/credit should never happen for tax
            // collection (institution_id ≠ ACCOUNT_ENERGY_BUDGET), but keep the
            // invariant tight.
            return Err(InstitutionLedgerError::SelfPosting {
                side: debit,
                amount: joules,
            });
        }
        // Debit macro account.
        let available = state.energy_budget_joules;
        if available < joules {
            return Err(InstitutionLedgerError::InsufficientMacroBalance {
                account: ACCOUNT_ENERGY_BUDGET,
                available,
                requested: joules,
            });
        }
        state.energy_budget_joules = available - joules;
        // Credit institution account.
        let account = state
            .institutions
            .accounts
            .get_mut(&institution_id)
            .ok_or(InstitutionLedgerError::UnknownInstitution { id: institution_id })?;
        account.balance_joules = account.balance_joules.saturating_add(joules);
        // Record the posting.
        state.institutions.postings.push(InstitutionPosting {
            tick: state.tick,
            debit,
            debit_amount: joules,
            credit,
            credit_amount: joules,
            amount: joules,
        });

        summary
            .per_institution_joules
            .insert(institution_id, joules);
        grand_total = grand_total.saturating_add(joules);
        // Stop iterating if the budget is now empty.
        if state.energy_budget_joules <= 0 {
            break;
        }
    }

    summary.total_collected_joules = grand_total;
    summary.capped_institutions = capped;
    Ok(summary)
}

fn macro_balance(economy: &EconomyState, account: AccountId) -> i64 {
    match account {
        ACCOUNT_ENERGY_BUDGET => economy.energy_budget_joules,
        _ => 0,
    }
}

fn set_macro_balance(economy: &mut EconomyState, account: AccountId, balance: i64) {
    if account == ACCOUNT_ENERGY_BUDGET {
        economy.energy_budget_joules = balance;
    }
}

/// Institution phase hook (CIV-0100 §3d stub). Called from [`crate::step`].
pub fn step_institutions(state: &mut EconomyState) {
    if state.institutions.accounts.is_empty() {
        state.institutions = InstitutionLedger::with_defaults();
    }
    // Future: baseline provision, treasury flows, market settlement.
    let _ = state.institutions.verify_conservation();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EconomyState;

    #[test]
    fn post_macro_to_institution_conserves_debits_and_credits() {
        let mut economy = EconomyState::with_energy_budget(100);
        let mut ledger = InstitutionLedger::with_defaults();

        ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                40,
            )
            .expect("post");

        assert_eq!(economy.energy_budget_joules, 60);
        assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 40);
        assert_eq!(ledger.postings.len(), 1);
        ledger.verify_conservation().expect("conservation");
    }

    #[test]
    fn post_institution_to_institution_conserves_and_non_negative() {
        let mut economy = EconomyState::with_energy_budget(50);
        let mut ledger = InstitutionLedger::with_defaults();

        ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                30,
            )
            .expect("fund treasury");
        ledger
            .post(
                &mut economy,
                LedgerSide::Institution(INSTITUTION_TREASURY),
                LedgerSide::Institution(INSTITUTION_MARKET),
                10,
            )
            .expect("treasury to market");

        assert_eq!(ledger.institution_balance(INSTITUTION_TREASURY), 20);
        assert_eq!(ledger.institution_balance(INSTITUTION_MARKET), 10);
        ledger.verify_conservation().expect("conservation");
    }

    #[test]
    fn post_rejects_negative_institution_balance() {
        let mut economy = EconomyState::with_energy_budget(0);
        let mut ledger = InstitutionLedger::with_defaults();

        let err = ledger
            .post(
                &mut economy,
                LedgerSide::Institution(INSTITUTION_MARKET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                1,
            )
            .expect_err("empty market cannot debit");

        assert_eq!(
            err,
            InstitutionLedgerError::NegativeInstitutionBalance {
                id: INSTITUTION_MARKET,
                before: 0,
                requested: 1,
            }
        );
    }

    #[test]
    fn step_institutions_seeds_defaults() {
        let mut economy = EconomyState::with_energy_budget(10);
        step_institutions(&mut economy);
        assert!(economy
            .institutions
            .accounts
            .contains_key(&INSTITUTION_MARKET));
        assert!(economy
            .institutions
            .accounts
            .contains_key(&INSTITUTION_TREASURY));
        economy
            .institutions
            .verify_conservation()
            .expect("conservation");
    }

    // ---- Conservation hardening regression tests (L5-110) --------------------

    #[test]
    fn post_records_split_debit_and_credit_amounts() {
        let mut economy = EconomyState::with_energy_budget(100);
        let mut ledger = InstitutionLedger::with_defaults();

        ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                25,
            )
            .expect("post");

        let posting = &ledger.postings[0];
        assert_eq!(posting.debit_amount, 25);
        assert_eq!(posting.credit_amount, 25);
        assert_eq!(posting.amount, 25);
    }

    #[test]
    fn post_rejects_self_posting_to_same_institution() {
        let mut economy = EconomyState::with_energy_budget(100);
        let mut ledger = InstitutionLedger::with_defaults();

        let err = ledger
            .post(
                &mut economy,
                LedgerSide::Institution(INSTITUTION_TREASURY),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                10,
            )
            .expect_err("treasury cannot post to itself");

        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Institution(INSTITUTION_TREASURY),
                amount: 10,
            }
        );
    }

    #[test]
    fn post_rejects_self_posting_to_same_macro_account() {
        let mut economy = EconomyState::with_energy_budget(100);
        let mut ledger = InstitutionLedger::with_defaults();

        let err = ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                5,
            )
            .expect_err("macro cannot post to itself");

        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                amount: 5,
            }
        );
    }

    /// Regression for the L5-110 audit: the prior `verify_conservation` summed the
    /// same `posting.amount` field for both sides, so the check was tautological.
    /// This test bypasses `post()` (the gateway guard) and writes a corrupted posting
    /// directly into the log; a real conservation check must reject it.
    #[test]
    fn verify_conservation_rejects_tampered_posting_amount() {
        let mut ledger = InstitutionLedger::with_defaults();
        ledger.accounts.get_mut(&INSTITUTION_TREASURY).unwrap().balance_joules = 0;

        ledger.postings.push(InstitutionPosting {
            tick: 0,
            debit: LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            debit_amount: 10, // tampered: doesn't match credit
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit_amount: 7,
            amount: 10,
        });

        let err = ledger
            .verify_conservation()
            .expect_err("tampered posting must be rejected");

        assert_eq!(
            err,
            InstitutionLedgerError::TamperedAmount {
                tick: 0,
                index: 0,
                debit_amount: 10,
                credit_amount: 7,
            }
        );
    }

    /// Regression: a posting where the same account is on both sides (e.g. forged
    /// from a bad disk row) must be detected by `verify_conservation` even though
    /// the amounts match.
    #[test]
    fn verify_conservation_rejects_self_posting_in_log() {
        let mut ledger = InstitutionLedger::with_defaults();
        ledger.accounts.get_mut(&INSTITUTION_TREASURY).unwrap().balance_joules = 0;

        ledger.postings.push(InstitutionPosting {
            tick: 5,
            debit: LedgerSide::Institution(INSTITUTION_TREASURY),
            debit_amount: 10,
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit_amount: 10,
            amount: 10,
        });

        let err = ledger
            .verify_conservation()
            .expect_err("self-posting in log must be rejected");

        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Institution(INSTITUTION_TREASURY),
                amount: 10,
            }
        );
    }

    #[test]
    fn verify_conservation_rejects_non_positive_posting_amount() {
        let mut ledger = InstitutionLedger::with_defaults();

        ledger.postings.push(InstitutionPosting {
            tick: 0,
            debit: LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            debit_amount: 0,
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit_amount: 0,
            amount: 0,
        });

        let err = ledger
            .verify_conservation()
            .expect_err("non-positive posting amount must be rejected");

        assert_eq!(
            err,
            InstitutionLedgerError::NonPositivePostingAmount {
                tick: 0,
                index: 0,
                debit_amount: 0,
                credit_amount: 0,
            }
        );
    }

    /// Backwards compat: postings serialized before the split-amount fields landed
    /// have `debit_amount = 0`, `credit_amount = 0`, and the canonical `amount > 0`.
    /// `verify_conservation` must reconstruct a balanced pair from the legacy field.
    #[test]
    fn verify_conservation_accepts_legacy_posting_without_split_amounts() {
        let mut ledger = InstitutionLedger::with_defaults();
        ledger.accounts.get_mut(&INSTITUTION_TREASURY).unwrap().balance_joules = 0;

        ledger.postings.push(InstitutionPosting {
            tick: 0,
            debit: LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
            debit_amount: 0,
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit_amount: 0,
            amount: 42,
        });

        ledger
            .verify_conservation()
            .expect("legacy posting must be accepted via legacy amount field");
    }

    // ---- Taxation tests (L5-110, FR-ECON-004 partial) -------------------------

    #[test]
    fn collect_taxes_posts_balanced_transfer_at_basis_points_rate() {
        let mut economy = EconomyState::with_energy_budget(100_000);
        let mut taxation = Taxation::default();
        taxation
            .rates_bp
            .insert(INSTITUTION_TREASURY, 500); // 5.00 %

        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        assert_eq!(summary.total_collected_joules, 5_000);
        assert_eq!(
            summary.per_institution_joules[&INSTITUTION_TREASURY],
            5_000
        );
        assert_eq!(economy.energy_budget_joules, 95_000);
        assert_eq!(
            economy.institutions.institution_balance(INSTITUTION_TREASURY),
            5_000
        );
        economy
            .institutions
            .verify_conservation()
            .expect("tax collection conserves joules");
    }

    #[test]
    fn collect_taxes_with_no_rates_is_no_op() {
        let mut economy = EconomyState::with_energy_budget(1_000);
        let taxation = Taxation::default();
        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        assert_eq!(summary.total_collected_joules, 0);
        assert_eq!(economy.energy_budget_joules, 1_000);
        assert_eq!(economy.institutions.postings.len(), 0);
    }

    #[test]
    fn collect_taxes_zero_rate_skips_institution() {
        let mut economy = EconomyState::with_energy_budget(1_000);
        let mut taxation = Taxation::default();
        taxation.rates_bp.insert(INSTITUTION_TREASURY, 0);

        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        assert_eq!(summary.total_collected_joules, 0);
        assert_eq!(economy.energy_budget_joules, 1_000);
    }

    #[test]
    fn collect_taxes_distributes_to_multiple_institutions() {
        let mut economy = EconomyState::with_energy_budget(200_000);
        // Seed treasury + market so the ledger knows them.
        economy.institutions = InstitutionLedger::with_defaults();
        let mut taxation = Taxation::default();
        taxation.rates_bp.insert(INSTITUTION_TREASURY, 250); // 2.5 %
        taxation.rates_bp.insert(INSTITUTION_MARKET, 100); // 1.0 %

        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        // First iteration: 200_000 * 250 / 10_000 = 5_000 (treasury),
        // then 195_000 * 100 / 10_000 = 1_950 (market).
        assert_eq!(summary.total_collected_joules, 6_950);
        assert_eq!(economy.energy_budget_joules, 200_000 - 6_950);
        economy
            .institutions
            .verify_conservation()
            .expect("multi-institution tax conserves joules");
    }

    #[test]
    fn collect_taxes_respects_per_institution_cap() {
        let mut economy = EconomyState::with_energy_budget(1_000_000);
        let mut taxation = Taxation::default();
        taxation.rates_bp.insert(INSTITUTION_TREASURY, 5_000); // 50 %
        taxation.per_institution_cap = Some(100); // very tight cap

        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        assert_eq!(summary.total_collected_joules, 100);
        assert_eq!(summary.capped_institutions, 1);
        assert_eq!(
            summary.per_institution_joules[&INSTITUTION_TREASURY],
            100
        );
    }

    #[test]
    fn collect_taxes_floors_at_zero_for_tiny_rates() {
        let mut economy = EconomyState::with_energy_budget(50);
        let mut taxation = Taxation::default();
        taxation.rates_bp.insert(INSTITUTION_TREASURY, 1); // 0.01 % of 50 = 0
        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        assert_eq!(summary.total_collected_joules, 0);
        assert_eq!(economy.energy_budget_joules, 50);
    }

    #[test]
    fn collect_taxes_stops_when_budget_depleted() {
        let mut economy = EconomyState::with_energy_budget(1);
        economy.institutions = InstitutionLedger::with_defaults();
        let mut taxation = Taxation::default();
        taxation.rates_bp.insert(INSTITUTION_TREASURY, 5_000);
        taxation.rates_bp.insert(INSTITUTION_MARKET, 5_000);
        let summary = collect_taxes(&mut economy, &taxation).expect("collect");
        // First institution: 1 * 5000 / 10000 = 0 ⇒ no posting ⇒ continues.
        // Second: same. Summary still reports 0 joules collected, but both
        // were inspected (we don't break early when joules == 0).
        assert_eq!(summary.total_collected_joules, 0);
    }
}
