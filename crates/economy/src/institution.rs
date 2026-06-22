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
    /// A posting has the same account on both sides (debit == credit is a no-op).
    ///
    /// Each `InstitutionPosting` is recorded as a single balanced pair, so the
    /// conservation contract is per-posting: `debit != credit`. This variant
    /// replaces the previous broken cross-posting sum (which summed
    /// `postings[i].amount` for both sides, making the check trivially pass).
    SelfPosting {
        /// The account that appeared on both legs of the posting.
        side: LedgerSide,
        /// Amount that would have been transferred (no-op).
        amount: i64,
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
    ///
    /// Each posting is intrinsically balanced: `apply_debit` and `apply_credit`
    /// are called with the same `amount`, and the resulting `InstitutionPosting`
    /// row records the pair. Conservation across postings is therefore
    /// enforced per-posting (`debit != credit`, `amount > 0`) rather than via
    /// cross-posting sums — see [`Self::verify_conservation`].
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

    /// Verify per-posting conservation and non-negative institution balances.
    ///
    /// Each `InstitutionPosting` is intrinsically balanced, so the conservation
    /// contract is per-posting (`debit != credit`, `amount > 0`). The previous
    /// cross-posting sum (`postings.iter().map(|p| p.amount).sum()` for both
    /// sides) was a tautology — both sides summed the same field — and was
    /// removed in favour of the meaningful per-posting check.
    pub fn verify_conservation(&self) -> Result<(), InstitutionLedgerError> {
        for posting in &self.postings {
            if posting.amount <= 0 {
                return Err(InstitutionLedgerError::NonPositiveAmount {
                    amount: posting.amount,
                });
            }
            if posting.debit == posting.credit {
                return Err(InstitutionLedgerError::SelfPosting {
                    side: posting.debit,
                    amount: posting.amount,
                });
            }
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

    /// `post` rejects a posting whose debit and credit are the same account —
    /// a self-posting is a no-op and would corrupt the conservation
    /// invariants if recorded.
    #[test]
    fn post_rejects_self_posting_macro() {
        let mut economy = EconomyState::with_energy_budget(100);
        let mut ledger = InstitutionLedger::with_defaults();
        let err = ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                10,
            )
            .expect_err("self-posting must be rejected");
        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                amount: 10,
            }
        );
        // No mutation must have happened.
        assert_eq!(economy.energy_budget_joules, 100);
        assert!(ledger.postings.is_empty());
    }

    #[test]
    fn post_rejects_self_posting_institution() {
        let mut economy = EconomyState::with_energy_budget(0);
        let mut ledger = InstitutionLedger::with_defaults();
        let err = ledger
            .post(
                &mut economy,
                LedgerSide::Institution(INSTITUTION_MARKET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                5,
            )
            .expect_err("self-posting must be rejected");
        assert_eq!(
            err,
            InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Institution(INSTITUTION_MARKET),
                amount: 5,
            }
        );
        assert!(ledger.postings.is_empty());
    }

    /// `verify_conservation` catches a corrupted posting (debit == credit)
    /// that snuck past `post`. The previous sum-based check summed the same
    /// field for both sides and was a tautology; this regression test would
    /// have silently passed under the old code.
    #[test]
    fn verify_conservation_catches_self_posting_in_log() {
        let mut economy = EconomyState::with_energy_budget(0);
        let mut ledger = InstitutionLedger::with_defaults();
        // Construct a corrupted posting directly (bypass `post`'s guard, which
        // is what the engine would do if it ever serialised a bad row from
        // disk).
        ledger.postings.push(InstitutionPosting {
            tick: 0,
            debit: LedgerSide::Institution(INSTITUTION_TREASURY),
            credit: LedgerSide::Institution(INSTITUTION_TREASURY),
            amount: 1,
        });
        assert_eq!(
            ledger.verify_conservation(),
            Err(InstitutionLedgerError::SelfPosting {
                side: LedgerSide::Institution(INSTITUTION_TREASURY),
                amount: 1,
            })
        );
        // Suppress unused mutability warning when other consumers borrow
        // `economy` for the next test.
        let _ = &mut economy;
    }

    /// End-to-end conservation under a sequence of random institution↔institution
    /// transfers: the sum of all institution balances plus the macro joule
    /// budget must remain invariant (no joules created or destroyed).
    #[test]
    fn institution_transfers_are_conservative_end_to_end() {
        let mut economy = EconomyState::with_energy_budget(80);
        let mut ledger = InstitutionLedger::with_defaults();
        // Fund both institutions from the macro budget so transfers can flow.
        ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_TREASURY),
                50,
            )
            .expect("seed treasury");
        ledger
            .post(
                &mut economy,
                LedgerSide::Macro(ACCOUNT_ENERGY_BUDGET),
                LedgerSide::Institution(INSTITUTION_MARKET),
                30,
            )
            .expect("seed market");
        assert_eq!(economy.energy_budget_joules, 0);

        let total_before: i64 = economy.energy_budget_joules
            + ledger.institution_balance(INSTITUTION_TREASURY)
            + ledger.institution_balance(INSTITUTION_MARKET);
        assert_eq!(total_before, 80);

        // Six alternating transfers; the only flow direction is institution→institution,
        // so the macro joule budget is untouched and conservation must hold at the
        // institution layer.
        let flows = [
            (INSTITUTION_TREASURY, INSTITUTION_MARKET, 5),
            (INSTITUTION_MARKET, INSTITUTION_TREASURY, 3),
            (INSTITUTION_TREASURY, INSTITUTION_MARKET, 7),
            (INSTITUTION_MARKET, INSTITUTION_TREASURY, 2),
            (INSTITUTION_TREASURY, INSTITUTION_MARKET, 4),
            (INSTITUTION_MARKET, INSTITUTION_TREASURY, 1),
        ];
        for (from, to, amount) in flows {
            ledger
                .post(
                    &mut economy,
                    LedgerSide::Institution(from),
                    LedgerSide::Institution(to),
                    amount,
                )
                .expect("transfer");
            let total: i64 = economy.energy_budget_joules
                + ledger.institution_balance(INSTITUTION_TREASURY)
                + ledger.institution_balance(INSTITUTION_MARKET);
            assert_eq!(total, total_before, "joules leaked across transfers");
            assert!(economy.energy_budget_joules >= 0);
            assert!(ledger.institution_balance(INSTITUTION_TREASURY) >= 0);
            assert!(ledger.institution_balance(INSTITUTION_MARKET) >= 0);
            ledger.verify_conservation().expect("conservation");
        }
        assert_eq!(ledger.postings.len(), 2 + flows.len());
    }
}
