//! FR-UX-004 — LOD transitions SHALL be visually seamless within one
//! rendered frame.
//!
//! Matrix check: `render::lod_seamless_transition`.

use civ_render::lod::{LodLevel, LodTransition};

/// A level change is applied atomically in a single frame; no transition ever
/// spans more than one frame.
#[test]
fn lod_seamless_transition() {
    // The single-frame guarantee is a hard constraint.
    assert_eq!(LodTransition::max_frames(), 1);

    // Selecting a level from camera distance is monotonic in distance.
    assert_eq!(LodLevel::from_distance(8.0), LodLevel(0));
    assert_eq!(LodLevel::from_distance(48.0), LodLevel(1));
    assert_eq!(LodLevel::from_distance(96.0), LodLevel(2));
    assert_eq!(LodLevel::from_distance(512.0), LodLevel(3));
    assert!(LodLevel::from_distance(512.0) > LodLevel::from_distance(8.0));

    // A real swap completes within the very first frame it is advanced.
    let mut swap = LodTransition::begin(LodLevel(0), LodLevel(3));
    assert!(!swap.is_completed());
    let blend = swap.advance().expect("first frame performs the swap");
    assert!((blend - 1.0).abs() < f32::EPSILON, "single-frame full blend");
    assert!(swap.is_completed(), "swap finished inside one frame");
    assert_eq!(swap.advance(), None, "no second frame of blending");

    // Same-level updates never blend and are already complete.
    let noop = LodTransition::begin(LodLevel(2), LodLevel(2));
    assert!(noop.is_noop());
    assert!(noop.is_completed());
}
