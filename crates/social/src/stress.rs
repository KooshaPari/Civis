//! FR-SOCI-002 — Citizen stress accumulation.
//!
//! Stress accumulates when a citizen cohort's Joule access falls below
//! the subsistence level. Stress is tracked as a continuous value in
//! `[0, 1_000]` basis points where 1_000 means maximum stress.

use serde::{Deserialize, Serialize};

/// Basis-point maximum stress level.
pub const MAX_STRESS_BP: i64 = 1_000;

/// Default subsistence Joule threshold per tick.
pub const DEFAULT_SUBSISTENCE_JOULES: i64 = 500;

/// Default stress increase per tick when below subsistence (basis points).
pub const DEFAULT_STRESS_INCREMENT_BP: i64 = 50;

/// Default stress decay per tick when at or above subsistence (basis points).
pub const DEFAULT_STRESS_DECAY_BP: i64 = 25;

/// Configuration for stress accumulation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressConfig {
    /// Joule level below which stress accumulates.
    pub subsistence_joules: i64,
    /// Stress increase per tick when below subsistence (bp).
    pub increment_bp: i64,
    /// Stress decay per tick when above subsistence (bp).
    pub decay_bp: i64,
}

impl Default for StressConfig {
    fn default() -> Self {
        Self {
            subsistence_joules: DEFAULT_SUBSISTENCE_JOULES,
            increment_bp: DEFAULT_STRESS_INCREMENT_BP,
            decay_bp: DEFAULT_STRESS_DECAY_BP,
        }
    }
}

/// Stress state for a single citizen cohort.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressAccumulator {
    /// Current stress in `[0, 1_000]` basis points.
    pub stress_bp: i64,
}

impl StressAccumulator {
    /// Create a new accumulator at zero stress.
    pub fn new() -> Self {
        Self { stress_bp: 0 }
    }

    /// Tick the accumulator: accumulate or decay stress based on Joule access.
    pub fn tick(&mut self, joules_available: i64, config: &StressConfig) {
        if joules_available < config.subsistence_joules {
            self.stress_bp = (self.stress_bp + config.increment_bp).min(MAX_STRESS_BP);
        } else {
            self.stress_bp = (self.stress_bp - config.decay_bp).max(0);
        }
    }

    /// Returns `true` when stress has reached the maximum.
    pub fn is_maxed(&self) -> bool {
        self.stress_bp >= MAX_STRESS_BP
    }

    /// Returns `true` when stress is above zero.
    pub fn is_stressed(&self) -> bool {
        self.stress_bp > 0
    }
}

impl Default for StressAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_below_subsistence() {
        let mut acc = StressAccumulator::new();
        let cfg = StressConfig::default();
        acc.tick(100, &cfg); // 100 < 500 => stress up
        assert_eq!(acc.stress_bp, 50);
        assert!(acc.is_stressed());
    }

    #[test]
    fn decays_above_subsistence() {
        let mut acc = StressAccumulator { stress_bp: 200 };
        let cfg = StressConfig::default();
        acc.tick(600, &cfg); // 600 >= 500 => decay
        assert_eq!(acc.stress_bp, 175);
    }

    #[test]
    fn stress_cannot_exceed_max() {
        let mut acc = StressAccumulator { stress_bp: 990 };
        let cfg = StressConfig::default();
        acc.tick(0, &cfg);
        assert_eq!(acc.stress_bp, MAX_STRESS_BP);
        assert!(acc.is_maxed());
    }

    #[test]
    fn stress_cannot_go_negative() {
        let mut acc = StressAccumulator { stress_bp: 10 };
        let cfg = StressConfig::default();
        acc.tick(1000, &cfg);
        assert_eq!(acc.stress_bp, 0);
        assert!(!acc.is_stressed());
    }

    #[test]
    fn zero_stress_not_stressed() {
        let acc = StressAccumulator::new();
        assert!(!acc.is_stressed());
        assert!(!acc.is_maxed());
    }
}
