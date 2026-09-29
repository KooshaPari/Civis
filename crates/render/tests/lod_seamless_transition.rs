//!
//! Provenance: this test previously carried a `FR-UX-004` tag. That id is
//! defined in `docs/models/civ-sim/USER_SPEC.md` as an unrelated
//! requirement and is NOT implemented by this code. The only document
//! claiming otherwise was `docs/traceability/TRACEABILITY_MATRIX.md`,
//! which cited a nonexistent spec file. The assertions below are real and
//! are retained as a behavioral test of single-frame LOD transition; the false id was removed
//! rather than rebound. See `docs/audits/id-provenance-corrections.md`.
//! rendered frame.
//!

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
