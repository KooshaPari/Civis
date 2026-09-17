//! FR-SOCI-001 — Ideological alignment per-citizen cohort.
//!
//! Ideology is tracked as a continuous score in `[-1_000, 1_000]` (basis
//! points) where `-1_000` is extreme dissent and `1_000` is full alignment.
//! Each cohort holds its own score; updates are driven externally by the
//! engine tick.

use serde::{Deserialize, Serialize};

/// Basis-point denominator for ideology range `[-1_000, 1_000]`.
const IDEOLOGY_HALF_RANGE: i64 = 1_000;

/// A cohort identifier (opaque, engine-assigned).
pub type CohortId = u64;

/// Per-cohort ideological alignment score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdeologyScore {
    /// Which cohort this score belongs to.
    pub cohort: CohortId,
    /// Continuous alignment in `[-1_000, 1_000]` basis points.
    pub alignment_bp: i64,
}

impl IdeologyScore {
    /// Create a new ideology score clamped to the valid range.
    pub fn new(cohort: CohortId, alignment_bp: i64) -> Self {
        Self {
            cohort,
            alignment_bp: alignment_bp.clamp(-IDEOLOGY_HALF_RANGE, IDEOLOGY_HALF_RANGE),
        }
    }

    /// Apply a delta (in basis points) to the alignment, clamping to range.
    pub fn apply_delta(&mut self, delta_bp: i64) {
        self.alignment_bp = (self.alignment_bp + delta_bp)
            .clamp(-IDEOLOGY_HALF_RANGE, IDEOLOGY_HALF_RANGE);
    }

    /// Returns `true` when alignment is positive (aligned with regime).
    pub fn is_aligned(&self) -> bool {
        self.alignment_bp > 0
    }

    /// Returns `true` when alignment is strongly negative (extreme dissent).
    pub fn is_dissenting(&self) -> bool {
        self.alignment_bp < -IDEOLOGY_HALF_RANGE / 2
    }
}

impl Default for IdeologyScore {
    fn default() -> Self {
        Self {
            cohort: 0,
            alignment_bp: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_cohort_continuous() {
        let mut s = IdeologyScore::new(42, 500);
        assert_eq!(s.cohort, 42);
        assert_eq!(s.alignment_bp, 500);
        assert!(s.is_aligned());
        s.apply_delta(-200);
        assert_eq!(s.alignment_bp, 300);
    }

    #[test]
    fn clamps_to_valid_range() {
        let s = IdeologyScore::new(1, 9999);
        assert_eq!(s.alignment_bp, 1_000);
        let s = IdeologyScore::new(1, -9999);
        assert_eq!(s.alignment_bp, -1_000);
    }

    #[test]
    fn dissent_detected() {
        let s = IdeologyScore::new(0, -600);
        assert!(s.is_dissenting());
        assert!(!s.is_aligned());
    }
}
