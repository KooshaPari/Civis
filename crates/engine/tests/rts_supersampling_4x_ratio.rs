//! Behavior tests for `4x supersampling dimension ratio`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-RENDER-002`.
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
mod rts_supersampling_4x_ratio {
    use civ_engine::rts_types::SsConfig;
    #[test]
    fn verify_fr_civ_rts_render_002_basic() {
        let ss = SsConfig::four_x(64, 64);
        assert_eq!(ss.factor, 4);
        // Internal render is 4x the output
        assert_eq!(ss.render_width(), 256);
        assert_eq!(ss.render_height(), 256);
        // Ratio is exactly 4:1
        assert_eq!(ss.render_width() / ss.output_width, 4);
        assert_eq!(ss.render_height() / ss.output_height, 4);
    }
}