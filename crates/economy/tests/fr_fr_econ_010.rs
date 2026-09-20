//! Tests for FR-ECON-010 — Treasury Balance Tracking
//!
//! Epic: FR-ECON
//! Treasury balance SHALL be tracked in MilliCredits (i64) with no
//! floating-point accumulation. All monetary operations use integer math.

use civ_economy::Treasury;

#[cfg(test)]
mod fr_fr_econ_010 {
    use super::*;

    /// FR-ECON-010: Treasury initializes with specified balances.
    #[test]
    fn treasury_initial_balance() {
        let t = Treasury::new(5000, 2500);
        assert_eq!(t.balance_joules(), 5000);
        assert_eq!(t.balance_milli_credits(), 2500);
    }

    /// FR-ECON-010: Default treasury starts at zero for both currencies.
    #[test]
    fn treasury_default_is_zero() {
        let t = Treasury::default();
        assert_eq!(t.balance_joules(), 0);
        assert_eq!(t.balance_milli_credits(), 0);
    }

    /// FR-ECON-010: Credit increases balance, debit decreases it.
    #[test]
    fn credit_and_debit_joules() {
        let mut t = Treasury::new(1000, 0);
        t.credit_joules(500);
        assert_eq!(t.balance_joules(), 1500);
        t.debit_joules(300);
        assert_eq!(t.balance_joules(), 1200);
    }

    /// FR-ECON-010: Credit and debit work for MilliCredits.
    #[test]
    fn credit_and_debit_milli_credits() {
        let mut t = Treasury::new(0, 1000);
        t.credit_milli_credits(250);
        assert_eq!(t.balance_milli_credits(), 1250);
        t.debit_milli_credits(400);
        assert_eq!(t.balance_milli_credits(), 850);
    }

    /// FR-ECON-010: Large i64 values do not overflow (integer-only, no float).
    #[test]
    fn large_i64_values_no_overflow() {
        let mut t = Treasury::new(i64::MAX / 2, i64::MAX / 3);
        t.credit_joules(1000);
        assert!(t.balance_joules() > i64::MAX / 2);
        t.credit_milli_credits(1000);
        assert!(t.balance_milli_credits() > i64::MAX / 3);
    }

    /// FR-ECON-010: Multiple round-trip credit/debit cycles remain consistent.
    #[test]
    fn round_trip_consistency() {
        let mut t = Treasury::new(1000, 1000);
        for _ in 0..100 {
            t.credit_joules(10);
            t.debit_joules(10);
            t.credit_milli_credits(5);
            t.debit_milli_credits(5);
        }
        assert_eq!(t.balance_joules(), 1000);
        assert_eq!(t.balance_milli_credits(), 1000);
    }
}
