//! Tests for FR-ECON-009 — Subsistence Mode
//!
//! Epic: FR-ECON
//! Subsistence mode SHALL activate when a civilization's total Joule balance
//! drops below a configurable threshold. Deactivation occurs when reserves
//! recover above the threshold.

use civ_economy::{SubsistenceMode, DEFAULT_SUBSISTENCE_THRESHOLD};

#[cfg(test)]
mod fr_fr_econ_009 {
    use super::*;

    /// FR-ECON-009: Subsistence activates when reserves fall below threshold.
    #[test]
    fn subsistence_activates_below_threshold() {
        let mut mode = SubsistenceMode::new();
        assert!(!mode.update(200), "above threshold: not active");
        assert!(!mode.is_active());
        assert!(mode.update(50), "below threshold: active");
        assert!(mode.is_active());
    }

    /// FR-ECON-009: Subsistence deactivates when reserves recover.
    #[test]
    fn subsistence_deactivates_above_threshold() {
        let mut mode = SubsistenceMode::new();
        mode.update(50); // activate
        assert!(mode.is_active());
        mode.update(200); // deactivate
        assert!(!mode.is_active());
        assert_eq!(mode.consecutive_ticks(), 0, "counter resets on recovery");
    }

    /// FR-ECON-009: Consecutive ticks in subsistence mode are tracked.
    #[test]
    fn consecutive_ticks_tracked() {
        let mut mode = SubsistenceMode::new();
        mode.update(50);
        assert_eq!(mode.consecutive_ticks(), 1);
        mode.update(30);
        assert_eq!(mode.consecutive_ticks(), 2);
        mode.update(10);
        assert_eq!(mode.consecutive_ticks(), 3);
    }

    /// FR-ECON-009: Default threshold matches documented constant.
    #[test]
    fn default_threshold_is_100() {
        let mode = SubsistenceMode::new();
        assert_eq!(mode.threshold(), DEFAULT_SUBSISTENCE_THRESHOLD);
        assert_eq!(mode.threshold(), 100);
    }

    /// FR-ECON-009: Custom threshold is honored.
    #[test]
    fn custom_threshold_honored() {
        let mut mode = SubsistenceMode::with_threshold(500);
        assert_eq!(mode.threshold(), 500);
        assert!(!mode.update(600), "above custom threshold");
        assert!(mode.update(400), "below custom threshold");
    }

    /// FR-ECON-009: Exact boundary value (equal to threshold) does NOT activate.
    #[test]
    fn exact_threshold_does_not_activate() {
        let mut mode = SubsistenceMode::with_threshold(100);
        assert!(!mode.update(100), "equal to threshold should not activate");
        assert!(!mode.is_active());
    }
}
