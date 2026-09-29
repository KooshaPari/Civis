//! Behavior tests for `UvRect fit and overlap predicates`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-RENDER-005`.
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
mod rts_uv_rect_containment {
    use civ_engine::rts_types::UvRect;
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