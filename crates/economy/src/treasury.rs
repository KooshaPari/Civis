//! FR-ECON-010 — Treasury in MilliCredits (i64, no float).
//!
//! Tracks balance as i64 to prevent floating-point accumulation errors.
//! All monetary operations use integer MilliCredits.

use serde::{Deserialize, Serialize};

/// Treasury holding balances in Joules (i64) and MilliCredits (i64).
/// No floating-point types are used to prevent accumulation errors (FR-ECON-010).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Treasury {
    /// Energy balance in Joules (integer, no float).
    balance_joules: i64,
    /// Monetary balance in MilliCredits (1 credit = 1000 milli-credits).
    balance_milli_credits: i64,
}

impl Treasury {
    /// Create a new treasury with initial balances.
    pub fn new(joules: i64, milli_credits: i64) -> Self {
        Self {
            balance_joules: joules,
            balance_milli_credits: milli_credits,
        }
    }

    /// Current Joule balance.
    pub fn balance_joules(&self) -> i64 {
        self.balance_joules
    }

    /// Current MilliCredit balance.
    pub fn balance_milli_credits(&self) -> i64 {
        self.balance_milli_credits
    }

    /// Credit Joules (add).
    pub fn credit_joules(&mut self, amount: i64) {
        self.balance_joules += amount;
    }

    /// Debit Joules (subtract). Caller must ensure sufficient balance.
    pub fn debit_joules(&mut self, amount: i64) {
        self.balance_joules -= amount;
    }

    /// Credit MilliCredits (add).
    pub fn credit_milli_credits(&mut self, amount: i64) {
        self.balance_milli_credits += amount;
    }

    /// Debit MilliCredits (subtract). Caller must ensure sufficient balance.
    pub fn debit_milli_credits(&mut self, amount: i64) {
        self.balance_milli_credits -= amount;
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self {
            balance_joules: 0,
            balance_milli_credits: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treasury_initial_balance() {
        let t = Treasury::new(1000, 500);
        assert_eq!(t.balance_joules(), 1000);
        assert_eq!(t.balance_milli_credits(), 500);
    }

    #[test]
    fn treasury_default_is_zero() {
        let t = Treasury::default();
        assert_eq!(t.balance_joules(), 0);
        assert_eq!(t.balance_milli_credits(), 0);
    }

    #[test]
    fn treasury_credit_debit_joules() {
        let mut t = Treasury::new(100, 0);
        t.credit_joules(50);
        assert_eq!(t.balance_joules(), 150);
        t.debit_joules(30);
        assert_eq!(t.balance_joules(), 120);
    }

    #[test]
    fn treasury_credit_debit_milli_credits() {
        let mut t = Treasury::new(0, 200);
        t.credit_milli_credits(100);
        assert_eq!(t.balance_milli_credits(), 300);
        t.debit_milli_credits(50);
        assert_eq!(t.balance_milli_credits(), 250);
    }

    /// Verify that balances are i64 — no float accumulation.
    #[test]
    fn treasury_no_float_accumulation() {
        let mut t = Treasury::default();
        // Accumulate 10,000 small integer credits.
        for _ in 0..10_000 {
            t.credit_milli_credits(1);
        }
        assert_eq!(t.balance_milli_credits(), 10_000);
        // Subtraction also stays exact.
        for _ in 0..5_000 {
            t.debit_milli_credits(1);
        }
        assert_eq!(t.balance_milli_credits(), 5_000);
    }
}
