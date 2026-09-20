//! Tests for FR-CIV-RTS-ZOOM-001
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-RTS-ZOOM-001.
//! Maps to CIV-0600 FR-CIV-ASSET-018: Zoom Transition Synchronous Swap.

#[cfg(test)]
mod fr_fr_civ_rts_zoom_001 {
    use civ_engine::rts_types::{SpriteHandle, ZoomTier};

    /// FR-CIV-RTS-ZOOM-001 -- Zoom swap is synchronous: set_zoom updates texture key immediately.
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
