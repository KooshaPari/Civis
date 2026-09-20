//! Tests for FR-CIV-RTS-RENDER-005
//!
//!
//! This test file verifies FR FR-CIV-RTS-RENDER-005.
//! Maps to CIV-0600 FR-CIV-ASSET-007: UV Coordinate Validity.

#[cfg(test)]
mod fr_fr_civ_rts_render_005 {
    use civ_engine::rts_types::UvRect;

    /// FR-CIV-RTS-RENDER-005 -- UV rects fit inside atlas bounds and don't overlap.
    #[test]
    fn verify_fr_civ_rts_render_005_basic() {
        let rect_a = UvRect { x: 0, y: 0, w: 64, h: 64 };
        let rect_b = UvRect { x: 64, y: 0, w: 64, h: 64 };
        let rect_c = UvRect { x: 0, y: 64, w: 64, h: 64 };

        // Both fit in a 2048x2048 atlas
        assert!(rect_a.fits_in(2048, 2048));
        assert!(rect_b.fits_in(2048, 2048));

        // No overlap between non-overlapping rects
        assert!(!rect_a.overlaps(&rect_b));
        assert!(!rect_a.overlaps(&rect_c));

        // Overlapping rects
        let rect_overlap = UvRect { x: 32, y: 32, w: 64, h: 64 };
        assert!(rect_a.overlaps(&rect_overlap));

        // Area computation
        assert_eq!(rect_a.area(), 64 * 64);
    }
}
