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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionPosting {
    /// Simulation tick when the posting was recorded.
    pub tick: u64,
    /// Account debited (balance decreases).
    pub debit: LedgerSide,
    /// Account credited (balance increases).
    pub credit: LedgerSide,
    /// Transfer amount (joules).
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

        self.apply_debit(economy, debit, amount)?;
        self.apply_credit(economy, credit, amount)?;

        self.postings.push(InstitutionPosting {
            tick: economy.tick,
            debit,
            credit,
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

    /// Verify every institution's stored balance reconciles against its posting
    /// history, and that no balance is negative.
    ///
    /// Each institution opens at zero (see [`with_defaults`](Self::with_defaults)),
    /// so its current balance must equal the net of postings that credited it
    /// minus those that debited it. A drift between the stored balance and the
    /// replayed posting net signals a bookkeeping bug and is reported as
    /// [`UnbalancedPostings`](InstitutionLedgerError::UnbalancedPostings)
    /// (`debits` = stored balance, `credits` = replayed net). If non-zero opening
    /// balances are introduced later, add them to `net` here.
    pub fn verify_conservation(&self) -> Result<(), InstitutionLedgerError> {
        for account in self.accounts.values() {
            if account.balance_joules < 0 {
                return Err(InstitutionLedgerError::NegativeInstitutionBalance {
                    id: account.id,
                    before: account.balance_joules,
                    requested: 0,
                });
            }

            let net: i64 = self
                .postings
                .iter()
                .map(|p| {
                    let credited = matches!(p.credit, LedgerSide::Institution(i) if i == account.id);
                    let debited = matches!(p.debit, LedgerSide::Institution(i) if i == account.id);
                    match (credited, debited) {
                        (true, false) => p.amount,
                        (false, true) => -p.amount,
                        // A self-posting (debit==credit==this institution) or an
                        // unrelated posting nets to zero for this account.
                        _ => 0,
                    }
                })
                .sum();

            if net != account.balance_joules {
                return Err(InstitutionLedgerError::UnbalancedPostings {
                    debits: account.balance_joules,
                    credits: net,
                });
            }
        }

        Ok(())
    }
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
    //
    // Surface conservation violations loudly rather than swallowing them: a
    // ledger that fails to reconcile is a bookkeeping bug, not a recoverable
    // condition. We log at error level every tick it holds and trip a
    // debug_assert so it fails fast in dev/test builds.
    if let Err(err) = state.institutions.verify_conservation() {
        tracing::error!(
            tick = state.tick,
            ?err,
            "institution ledger failed conservation reconciliation"
        );
        debug_assert!(false, "institution ledger conservation violated: {err:?}");
    }
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
    fn verify_conservation_catches_balance_drift() {
        // Regression: the old check summed `p.amount` into both sides, so it was
        // a tautology that could never fail. Corrupt a stored balance away from
        // its posting history and confirm reconciliation now reports the drift.
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
        ledger.verify_conservation().expect("clean ledger reconciles");

        // Inject drift: treasury claims 99 but postings only credited 40.
        ledger
            .accounts
            .get_mut(&INSTITUTION_TREASURY)
            .expect("treasury")
            .balance_joules = 99;

        let err = ledger
            .verify_conservation()
            .expect_err("drift must be caught");
        assert_eq!(
            err,
            InstitutionLedgerError::UnbalancedPostings {
                debits: 99,
                credits: 40,
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
}
