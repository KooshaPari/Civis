//! FR-ECON-009 — Subsistence mode.
//!
//! When a district's energy reserves fall below a configurable threshold,
//! it enters subsistence mode: non-essential production is halted and only
//! core consumption (food, shelter) is maintained.

use serde::{Deserialize, Serialize};

/// Default subsistence threshold in Joules (100 J).
pub const DEFAULT_SUBSISTENCE_THRESHOLD: i64 = 100;

/// Subsistence mode state for a district (FR-ECON-009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubsistenceMode {
    /// Whether the district is currently in subsistence mode.
    active: bool,
    /// Energy threshold below which subsistence activates.
    threshold: i64,
    /// Number of consecutive ticks spent in subsistence mode.
    consecutive_ticks: u64,
}

impl SubsistenceMode {
    /// Create with default threshold.
    pub fn new() -> Self {
        Self {
            active: false,
            threshold: DEFAULT_SUBSISTENCE_THRESHOLD,
            consecutive_ticks: 0,
        }
    }

    /// Create with a custom threshold.
    pub fn with_threshold(threshold: i64) -> Self {
        Self {
            active: false,
            threshold,
            consecutive_ticks: 0,
        }
    }

    /// Check and update the subsistence state based on current energy reserves.
    /// Returns `true` if subsistence mode is active after the update.
    pub fn update(&mut self, energy_reserves: i64) -> bool {
        if energy_reserves < self.threshold {
            self.active = true;
            self.consecutive_ticks += 1;
        } else {
            self.active = false;
            self.consecutive_ticks = 0;
        }
        self.active
    }

    /// Whether subsistence mode is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Number of consecutive ticks in subsistence mode.
    pub fn consecutive_ticks(&self) -> u64 {
        self.consecutive_ticks
    }

    /// The current threshold.
    pub fn threshold(&self) -> i64 {
        self.threshold
    }
}

impl Default for SubsistenceMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsistence_activates_below_threshold() {
        let mut mode = SubsistenceMode::new();
        assert!(!mode.update(200));
        assert!(!mode.is_active());
        assert!(mode.update(50));
        assert!(mode.is_active());
    }

    #[test]
    fn subsistence_deactivates_above_threshold() {
        let mut mode = SubsistenceMode::new();
        mode.update(50);
        assert!(mode.is_active());
        assert!(!mode.update(200));
        assert!(!mode.is_active());
    }

    #[test]
    fn subsistence_tracks_consecutive_ticks() {
        let mut mode = SubsistenceMode::new();
        mode.update(50);
        mode.update(30);
        mode.update(10);
        assert_eq!(mode.consecutive_ticks(), 3);
    }

    #[test]
    fn subsistence_resets_consecutive_on_recovery() {
        let mut mode = SubsistenceMode::new();
        mode.update(50);
        mode.update(30);
        assert_eq!(mode.consecutive_ticks(), 2);
        mode.update(500);
        assert_eq!(mode.consecutive_ticks(), 0);
    }

    #[test]
    fn subsistence_custom_threshold() {
        let mut mode = SubsistenceMode::with_threshold(500);
        assert!(!mode.update(600));
        assert!(mode.update(400));
        assert_eq!(mode.threshold(), 500);
    }
}
