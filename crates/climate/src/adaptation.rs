//! FR-CLIM-006 — Adaptation investment reduces climate damage.
//!
//! Civilizations invest MilliCredits into adaptation to reduce climate damage.
//! The effectiveness follows diminishing returns: each unit of investment
//! provides less marginal reduction than the previous.
//!
//! Adaptation level is cumulative (does not decay) and reduces the damage
//! fraction computed by `damage.rs`.
//!
//! The damage reduction formula (in basis points):
//!   `reduction_bp = min(max_reduction, floor(sqrt(investment_milli_credits) * scale))`

use serde::{Deserialize, Serialize};

/// Default scaling factor: sqrt(investment) * scale → reduction_bp.
/// At 1M MilliCredits: sqrt(1_000_000) * 50 = 50 000 → capped.
pub const DEFAULT_ADAPTATION_SCALE: i64 = 50;

/// Maximum damage reduction from adaptation (basis points).
/// 5 000 bp = at most 50 % damage reduction.
pub const DEFAULT_MAX_REDUCTION_BP: i64 = 5_000;

/// Configuration for the adaptation investment model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptationConfig {
    /// Scaling factor for the diminishing-returns formula.
    pub scale: i64,
    /// Maximum reduction in basis points.
    pub max_reduction_bp: i64,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            scale: DEFAULT_ADAPTATION_SCALE,
            max_reduction_bp: DEFAULT_MAX_REDUCTION_BP,
        }
    }
}

/// Result of an adaptation computation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdaptationResult {
    /// Cumulative investment in MilliCredits.
    pub total_investment_mc: i64,
    /// Damage reduction in basis points (applied to damage before capping).
    pub reduction_bp: i64,
    /// Marginal effectiveness of the last investment (reduction_bp per MC).
    pub marginal_effectiveness: f64,
}

/// Tracks adaptation investment for a civilization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptationTracker {
    /// Cumulative MilliCredits invested in adaptation.
    pub investment_mc: i64,
    /// Current damage reduction from adaptation (basis points).
    pub reduction_bp: i64,
    /// Number of investment actions taken.
    pub investment_count: u64,
}

impl AdaptationTracker {
    /// Create with no adaptation investment.
    pub fn new() -> Self {
        Self {
            investment_mc: 0,
            reduction_bp: 0,
            investment_count: 0,
        }
    }

    /// Invest MilliCredits into adaptation.
    ///
    /// Returns the updated adaptation result.
    pub fn invest(
        &mut self,
        milli_credits: i64,
        config: &AdaptationConfig,
    ) -> AdaptationResult {
        let investment = milli_credits.max(0);
        let _old_mc = self.investment_mc;
        self.investment_mc += investment;
        self.investment_count += 1;

        let new_reduction = Self::compute_reduction(self.investment_mc, config);
        let old_reduction = self.reduction_bp;
        self.reduction_bp = new_reduction;

        let marginal = if investment > 0 {
            (new_reduction - old_reduction) as f64 / investment as f64
        } else {
            0.0
        };

        AdaptationResult {
            total_investment_mc: self.investment_mc,
            reduction_bp: self.reduction_bp,
            marginal_effectiveness: marginal,
        }
    }

    /// Compute the reduction_bp from total investment using the diminishing
    /// returns formula.
    fn compute_reduction(investment_mc: i64, config: &AdaptationConfig) -> i64 {
        if investment_mc <= 0 {
            return 0;
        }
        let raw = ((investment_mc as f64).sqrt() * config.scale as f64) as i64;
        raw.min(config.max_reduction_bp).max(0)
    }

    /// Compute adaptation result without mutating state (read-only).
    pub fn compute(&self, config: &AdaptationConfig) -> AdaptationResult {
        let reduction = Self::compute_reduction(self.investment_mc, config);
        AdaptationResult {
            total_investment_mc: self.investment_mc,
            reduction_bp: reduction,
            marginal_effectiveness: 0.0,
        }
    }
}

impl Default for AdaptationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_investment_no_reduction() {
        let tracker = AdaptationTracker::new();
        let cfg = AdaptationConfig::default();
        let result = tracker.compute(&cfg);
        assert_eq!(result.reduction_bp, 0);
    }

    #[test]
    fn investment_provides_reduction() {
        let mut tracker = AdaptationTracker::new();
        let cfg = AdaptationConfig::default();
        let result = tracker.invest(10_000, &cfg);
        assert!(result.reduction_bp > 0, "investment should provide some reduction");
        assert_eq!(result.total_investment_mc, 10_000);
    }

    #[test]
    fn diminishing_returns() {
        let cfg = AdaptationConfig::default();
        let mut t1 = AdaptationTracker::new();
        let mut t2 = AdaptationTracker::new();

        let r1 = t1.invest(10_000, &cfg);
        let _r2 = t2.invest(10_000, &cfg);
        let r3 = t2.invest(10_000, &cfg);

        // First 10k gives more marginal benefit than second 10k
        assert!(r1.reduction_bp > 0);
        assert!(r3.marginal_effectiveness < r1.marginal_effectiveness);
    }

    #[test]
    fn reduction_capped_at_max() {
        let cfg = AdaptationConfig::default();
        let mut tracker = AdaptationTracker::new();
        // Massive investment
        tracker.invest(1_000_000_000, &cfg);
        assert!(tracker.reduction_bp <= cfg.max_reduction_bp);
    }

    #[test]
    fn negative_investment_clamped() {
        let mut tracker = AdaptationTracker::new();
        let cfg = AdaptationConfig::default();
        let result = tracker.invest(-500, &cfg);
        assert_eq!(result.total_investment_mc, 0);
        assert_eq!(tracker.reduction_bp, 0);
    }

    #[test]
    fn cumulative_investment() {
        let mut tracker = AdaptationTracker::new();
        let cfg = AdaptationConfig::default();
        tracker.invest(1_000, &cfg);
        tracker.invest(2_000, &cfg);
        tracker.invest(3_000, &cfg);
        assert_eq!(tracker.investment_mc, 6_000);
        assert_eq!(tracker.investment_count, 3);
    }

    #[test]
    fn zero_investment_no_change() {
        let mut tracker = AdaptationTracker::new();
        let cfg = AdaptationConfig::default();
        tracker.invest(5_000, &cfg);
        let before = tracker.reduction_bp;
        tracker.invest(0, &cfg);
        assert_eq!(tracker.reduction_bp, before);
    }
}
