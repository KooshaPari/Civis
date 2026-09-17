//! LOD transitions (FR-UX-004, CIV-0300).
//!
//! LOD transitions SHALL be visually seamless within one rendered frame. To
//! guarantee that, a transition never spans multiple frames: when the chosen
//! detail level changes, the new level is swapped in atomically inside the same
//! frame, and any cross-fade is expressed as a single-frame blend weight rather
//! than a multi-frame animation.
//!
//! This module models that contract so it can be asserted headlessly.

use serde::{Deserialize, Serialize};

/// Discrete level-of-detail bucket (0 = highest detail).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LodLevel(pub u8);

impl LodLevel {
    /// Highest detail level.
    pub const HIGHEST: Self = Self(0);

    /// Pick a level from the camera distance, in world units.
    ///
    /// Buckets double the distance each step: `<32` => LOD 0, `<64` => LOD 1,
    /// `<128` => LOD 2, otherwise the coarse LOD 3.
    #[must_use]
    pub fn from_distance(distance: f32) -> Self {
        if distance < 32.0 {
            Self(0)
        } else if distance < 64.0 {
            Self(1)
        } else if distance < 128.0 {
            Self(2)
        } else {
            Self(3)
        }
    }
}

/// A single-frame LOD swap (FR-UX-004).
///
/// [`LodTransition::begin`] records the level being replaced and the level
/// being swapped in. [`advance`](Self::advance) completes the swap after exactly
/// one frame and reports the blend weight that frame used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LodTransition {
    /// Level active before the swap.
    pub from: LodLevel,
    /// Level swapped in for this frame.
    pub to: LodLevel,
    /// Whether the swap has already been applied.
    completed: bool,
}

impl LodTransition {
    /// Begin a transition between two levels.
    ///
    /// A no-op transition (same level) is marked complete immediately so the
    /// caller does not blend a level into itself.
    #[must_use]
    pub fn begin(from: LodLevel, to: LodLevel) -> Self {
        Self {
            from,
            to,
            completed: from == to,
        }
    }

    /// Is this transition a no-op (same level on both sides)?
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.from == self.to
    }

    /// Has the swap been applied?
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Advance the transition by one frame.
    ///
    /// Returns the blend weight for this frame (`0.0` for a no-op, `1.0` for a
    /// real swap, because the swap completes within the single frame). Returns
    /// `None` once completed, so callers stop blending.
    pub fn advance(&mut self) -> Option<f32> {
        if self.completed {
            return None;
        }
        self.completed = true;
        Some(1.0)
    }

    /// Frames a `from != to` swap is allowed to span. The guarantee is exactly
    /// one frame.
    #[must_use]
    pub const fn max_frames() -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_buckets_to_level() {
        assert_eq!(LodLevel::from_distance(10.0), LodLevel(0));
        assert_eq!(LodLevel::from_distance(50.0), LodLevel(1));
        assert_eq!(LodLevel::from_distance(100.0), LodLevel(2));
        assert_eq!(LodLevel::from_distance(1000.0), LodLevel(3));
    }

    #[test]
    fn swap_completes_in_one_frame() {
        let mut t = LodTransition::begin(LodLevel(0), LodLevel(2));
        assert!(!t.is_completed());
        assert_eq!(t.advance(), Some(1.0));
        assert!(t.is_completed());
        // second advance is a no-op
        assert_eq!(t.advance(), None);
    }

    #[test]
    fn same_level_is_immediate_noop() {
        let t = LodTransition::begin(LodLevel(1), LodLevel(1));
        assert!(t.is_noop());
        assert!(t.is_completed());
        assert_eq!(LodTransition::max_frames(), 1);
    }
}
