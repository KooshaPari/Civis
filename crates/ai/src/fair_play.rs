//! FR-AI-005 — Fair-play resource expenditure cap.
//!
//! AI SHALL never exceed a configurable MilliCredit/Joule expenditure
//! per tick (fair-play cap). This module enforces the cap and tracks
//! consumption.

use serde::{Deserialize, Serialize};

/// Default fair-play cap: 10 000 MilliCredits per tick.
pub const DEFAULT_MC_CAP_PER_TICK: i64 = 10_000;

/// Default fair-play cap: 5 000 Joules per tick.
pub const DEFAULT_J_PER_TICK: i64 = 5_000;

/// Configuration for the fair-play cap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FairPlayCap {
    /// Maximum MilliCredits the AI may spend this tick.
    pub mc_limit: i64,
    /// Maximum Joules the AI may consume this tick.
    pub joule_limit: i64,
    /// MilliCredits already spent this tick (mutable accumulator).
    pub mc_spent: i64,
    /// Joules already consumed this tick (mutable accumulator).
    pub joules_used: i64,
}

impl FairPlayCap {
    /// Create a cap with default limits and zero consumption.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mc_limit: DEFAULT_MC_CAP_PER_TICK,
            joule_limit: DEFAULT_J_PER_TICK,
            mc_spent: 0,
            joules_used: 0,
        }
    }

    /// Create a cap with custom limits.
    #[must_use]
    pub fn with_limits(mc_limit: i64, joule_limit: i64) -> Self {
        Self {
            mc_limit,
            joule_limit,
            mc_spent: 0,
            joules_used: 0,
        }
    }

    /// Try to spend `mc` MilliCredits. Returns `true` if allowed, `false` if it
    /// would exceed the cap (and does NOT modify state).
    pub fn try_spend_mc(&mut self, mc: i64) -> bool {
        if mc <= 0 {
            return true;
        }
        if self.mc_spent + mc > self.mc_limit {
            return false;
        }
        self.mc_spent += mc;
        true
    }

    /// Try to consume `j` Joules. Returns `true` if allowed.
    pub fn try_use_joules(&mut self, j: i64) -> bool {
        if j <= 0 {
            return true;
        }
        if self.joules_used + j > self.joule_limit {
            return false;
        }
        self.joules_used += j;
        true
    }

    /// Remaining MilliCredit budget this tick.
    #[must_use]
    pub fn mc_remaining(&self) -> i64 {
        (self.mc_limit - self.mc_spent).max(0)
    }

    /// Remaining Joule budget this tick.
    #[must_use]
    pub fn joule_remaining(&self) -> i64 {
        (self.joule_limit - self.joules_used).max(0)
    }

    /// Reset accumulators for a new tick.
    pub fn reset(&mut self) {
        self.mc_spent = 0;
        self.joules_used = 0;
    }
}

impl Default for FairPlayCap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spend_within_limit_succeeds() {
        let mut cap = FairPlayCap::with_limits(100, 200);
        assert!(cap.try_spend_mc(50));
        assert_eq!(cap.mc_spent, 50);
    }

    #[test]
    fn spend_exceeding_limit_fails() {
        let mut cap = FairPlayCap::with_limits(100, 200);
        assert!(!cap.try_spend_mc(101));
        assert_eq!(cap.mc_spent, 0);
    }
}
