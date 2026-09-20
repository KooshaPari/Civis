//! FR-CIV-SOCIAL-002-IDEOLOGY — Per-citizen ideology vector with social drift.
//!
//! Ideology is tracked as a continuous score in `[-1_000, 1_000]` (basis
//! points) where `-1_000` is extreme dissent and `1_000` is full alignment.
//! Each cohort holds its own score; updates are driven externally by the
//! engine tick.
//
// FR-CIV-SOCIAL-002-IDEOLOGY also calls for a `Citizen { ideology: f32 }`
// shape with a `ideology_shift()` function based on institution policy drift.
// That signature lives on `Citizen` in `civ_engine`; this module exposes the
// bounded per-cohort primitives it consumes.

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

    /// **FR-CIV-SOCIAL-002-IDEOLOGY** — Apply a delta (in basis points) to the
    /// alignment, clamping to range. This is the primitive the engine tick uses
    /// when an institution policy drift or other social event shifts a cohort's
    /// ideological stance.
    pub fn apply_delta(&mut self, delta_bp: i64) {
        self.alignment_bp = (self.alignment_bp + delta_bp)
            .clamp(-IDEOLOGY_HALF_RANGE, IDEOLOGY_HALF_RANGE);
    }

    /// **FR-CIV-SOCIAL-002-IDEOLOGY** — Linear-blend a score toward a target by
    /// `weight` in `[0, 1]`. Useful when an institution nudges a cohort's
    /// ideology toward (or away from) a target value rather than applying a
    /// single delta. The result is clamped to the valid `[-1_000, 1_000]` range.
    pub fn blend_toward(&mut self, target: IdeologyScore, weight: f32) {
        let w = weight.clamp(0.0, 1.0);
        let tgt = target.alignment_bp as f32;
        let cur = self.alignment_bp as f32;
        let next = cur + (tgt - cur) * w;
        let rounded = next.round() as i64;
        self.alignment_bp = rounded.clamp(-IDEOLOGY_HALF_RANGE, IDEOLOGY_HALF_RANGE);
    }

    /// **FR-CIV-SOCIAL-002-IDEOLOGY** — Convenience accessor that returns the
    /// alignment as a normalized `f32` in `[-1.0, 1.0]` for callers that want a
    /// plain float (the spec asks for `Citizen { ideology: f32 }` in that
    /// range). One basis point ⇒ `1e-3`.
    #[must_use]
    pub fn alignment_f32(&self) -> f32 {
        self.alignment_bp as f32 / IDEOLOGY_HALF_RANGE as f32
    }

    /// **FR-CIV-SOCIAL-002-IDEOLOGY** — Returns `true` when alignment is
    /// positive (aligned with regime).
    pub fn is_aligned(&self) -> bool {
        self.alignment_bp > 0
    }

    /// **FR-CIV-SOCIAL-002-IDEOLOGY** — Returns `true` when alignment is
    /// strongly negative (extreme dissent).
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

    /// FR-CIV-SOCIAL-002-IDEOLOGY — `alignment_f32()` projects basis points into
    /// the normalized `[-1.0, 1.0]` range the spec asks for.
    #[test]
    fn alignment_f32_round_trip() {
        let full = IdeologyScore::new(1, 1_000);
        assert!((full.alignment_f32() - 1.0).abs() < 1e-6);
        let full_neg = IdeologyScore::new(1, -1_000);
        assert!((full_neg.alignment_f32() + 1.0).abs() < 1e-6);
        let zero = IdeologyScore::new(1, 0);
        assert_eq!(zero.alignment_f32(), 0.0);
    }

    /// FR-CIV-SOCIAL-002-IDEOLOGY — `blend_toward` interpolates and clamps.
    #[test]
    fn blend_toward_moves_score_and_clamps() {
        let mut s = IdeologyScore::new(1, 0);
        let target = IdeologyScore::new(1, 800);
        s.blend_toward(target, 0.5);
        // (0 + (800 - 0) * 0.5).round() == 400
        assert_eq!(s.alignment_bp, 400);

        // Out-of-range target should still clamp to the valid window.
        s.alignment_bp = 500;
        let far_target = IdeologyScore::new(1, 999_999);
        s.blend_toward(far_target, 1.0);
        assert_eq!(s.alignment_bp, 1_000);
    }
}
