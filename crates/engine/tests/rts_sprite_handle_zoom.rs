//! Behavior tests for `SpriteHandle::set_zoom state update`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-ZOOM-001`.
//! That ID is not a requirement: the only place it appears in the repository
//! is the middle column of CIV-0600's §14 traceability table
//! (`docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3205-3224`), which uses it
//! as a "verification owner" label for a test file that does not exist.
//! CIV-0300 §12.1 owns `FR-CIV-RTS-001..015` and never mentions the
//! RENDER/ZOOM/NATION sub-namespaces.
//!
//! So the ID was dropped rather than satisfied: there is no requirement text to
//! implement, and inventing one would be the same defect in a new place. The
//! behavior is still worth a test, so the test stays under a name that says
//! what it checks.

#[cfg(test)]
mod rts_sprite_handle_zoom {
    use civ_engine::rts_types::{SpriteHandle, ZoomTier};
    #[test]
    fn verify_fr_civ_rts_zoom_001_basic() {
        let mut sprite = SpriteHandle::new("terrain_plains", ZoomTier::Strategic);
        assert_eq!(sprite.texture_key(), "terrain_plains_z1");
        assert!(!sprite.swapped_this_frame);

        // Synchronous swap — no await, immediate texture key change
        sprite.set_zoom(ZoomTier::Tactical);
        assert_eq!(sprite.texture_key(), "terrain_plains_z2");
        assert!(sprite.swapped_this_frame);

        sprite.set_zoom(ZoomTier::Citizen);
        assert_eq!(sprite.texture_key(), "terrain_plains_z3");
    }
}