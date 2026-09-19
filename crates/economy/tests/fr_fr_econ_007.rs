//! Tests for FR-ECON-007 — Trade Agreements
//!
//! Epic: FR-ECON
//! Trade agreements SHALL transfer Joules and MilliCredits between parties
//! each tick. Transfers are atomic: both legs succeed or neither is applied.

use civ_economy::{TradeAgreement, TradeAgreementError, Treasury};

#[cfg(test)]
mod fr_fr_econ_007 {
    use super::*;

    /// FR-ECON-007: Bilateral transfer debits sender and credits receiver.
    #[test]
    fn bilateral_transfer_balanced() {
        let mut from = Treasury::new(1000, 500);
        let mut to = Treasury::new(200, 100);
        let agreement = TradeAgreement::new(0, 1, 300, 150);
        let result = agreement.bilateral_transfer(&mut from, &mut to);
        assert!(result.is_ok());
        assert_eq!(from.balance_joules(), 700, "sender debited joules");
        assert_eq!(from.balance_milli_credits(), 350, "sender debited credits");
        assert_eq!(to.balance_joules(), 500, "receiver credited joules");
        assert_eq!(to.balance_milli_credits(), 250, "receiver credited credits");
    }

    /// FR-ECON-007: Insufficient joules returns error without mutating state.
    #[test]
    fn insufficient_joules_rejected_atomically() {
        let mut from = Treasury::new(100, 500);
        let mut to = Treasury::new(200, 100);
        let agreement = TradeAgreement::new(0, 1, 200, 0);
        let result = agreement.bilateral_transfer(&mut from, &mut to);
        assert!(result.is_err());
        match result.unwrap_err() {
            TradeAgreementError::InsufficientJoules { available, required } => {
                assert_eq!(available, 100);
                assert_eq!(required, 200);
            }
            other => panic!("expected InsufficientJoules, got {:?}", other),
        }
        // State must be unchanged (atomic rollback).
        assert_eq!(from.balance_joules(), 100);
        assert_eq!(to.balance_joules(), 200);
    }

    /// FR-ECON-007: Insufficient milli-credits returns error without mutating state.
    #[test]
    fn insufficient_milli_credits_rejected() {
        let mut from = Treasury::new(1000, 50);
        let mut to = Treasury::new(0, 0);
        let agreement = TradeAgreement::new(0, 1, 0, 200);
        let result = agreement.bilateral_transfer(&mut from, &mut to);
        assert!(result.is_err());
        match result.unwrap_err() {
            TradeAgreementError::InsufficientMilliCredits { available, required } => {
                assert_eq!(available, 50);
                assert_eq!(required, 200);
            }
            other => panic!("expected InsufficientMilliCredits, got {:?}", other),
        }
        // State unchanged.
        assert_eq!(from.balance_milli_credits(), 50);
        assert_eq!(to.balance_milli_credits(), 0);
    }

    /// FR-ECON-007: Zero-value transfer succeeds.
    #[test]
    fn zero_value_transfer_succeeds() {
        let mut from = Treasury::new(100, 50);
        let mut to = Treasury::new(200, 100);
        let agreement = TradeAgreement::new(0, 1, 0, 0);
        let result = agreement.bilateral_transfer(&mut from, &mut to);
        assert!(result.is_ok());
        assert_eq!(from.balance_joules(), 100);
        assert_eq!(to.balance_joules(), 200);
    }

    /// FR-ECON-007: Conservation — total joules across both treasuries is invariant.
    #[test]
    fn joule_conservation_across_transfer() {
        let mut from = Treasury::new(1000, 0);
        let mut to = Treasury::new(500, 0);
        let total_before = from.balance_joules() + to.balance_joules();
        let agreement = TradeAgreement::new(0, 1, 300, 0);
        agreement.bilateral_transfer(&mut from, &mut to).unwrap();
        let total_after = from.balance_joules() + to.balance_joules();
        assert_eq!(total_before, total_after, "total joules conserved across trade");
    }
}
