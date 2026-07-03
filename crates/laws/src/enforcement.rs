//! Law-enforcement state for tracked violations and penalties.
//!
//! This is intentionally small and deterministic: callers record violations,
//! and once the running count reaches the configured threshold, the configured
//! penalty is applied.

/// A penalty applied when violation pressure crosses the enforcement threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LawPenalty {
    /// Severity multiplier or step amount associated with the penalty.
    pub severity: u32,
}

/// Tracks violation accrual and threshold-triggered penalties.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LawEnforcement {
    threshold: u32,
    violation_count: u32,
    applied_penalties: u32,
    penalty: LawPenalty,
}

impl LawEnforcement {
    /// Create a new enforcement tracker.
    pub fn new(threshold: u32, penalty: LawPenalty) -> Self {
        Self {
            threshold: threshold.max(1),
            violation_count: 0,
            applied_penalties: 0,
            penalty,
        }
    }

    /// Current number of accrued violations.
    #[must_use]
    pub fn violation_count(&self) -> u32 {
        self.violation_count
    }

    /// Number of penalties already applied.
    #[must_use]
    pub fn applied_penalties(&self) -> u32 {
        self.applied_penalties
    }

    /// Record a violation. Returns `Some(penalty)` when the threshold is met.
    pub fn record_violation(&mut self) -> Option<LawPenalty> {
        self.violation_count = self.violation_count.saturating_add(1);
        if self.violation_count >= self.threshold {
            self.violation_count = 0;
            self.applied_penalties = self.applied_penalties.saturating_add(1);
            Some(self.penalty)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_violations_cross_threshold_and_apply_penalty() {
        let mut enforcement = LawEnforcement::new(3, LawPenalty { severity: 7 });

        assert_eq!(enforcement.record_violation(), None);
        assert_eq!(enforcement.violation_count(), 1);
        assert_eq!(enforcement.record_violation(), None);
        assert_eq!(enforcement.violation_count(), 2);

        let penalty = enforcement.record_violation();
        assert_eq!(penalty, Some(LawPenalty { severity: 7 }));
        assert_eq!(enforcement.violation_count(), 0);
        assert_eq!(enforcement.applied_penalties(), 1);
    }
}
