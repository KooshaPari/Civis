//! Behavior tests for `color_matches tolerance`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-NATION-002`.
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
mod rts_nation_shader_tolerance {
    use civ_engine::rts_types::{color_distance, color_matches, SHADER_TOLERANCE};
    #[test]
    fn verify_fr_civ_rts_nation_002_basic() {
        // Identical colors should match
        assert!(color_matches([1.0, 0.0, 0.0], [1.0, 0.0, 0.0]));
        // Very close colors within tolerance should match (dithering artifact)
        let close = [0.5, 0.5, 0.5];
        let perturbed = [0.52, 0.5, 0.5];
        assert!(color_distance(close, perturbed) < SHADER_TOLERANCE);
        assert!(color_matches(close, perturbed));
        // Distinct colors should not match
        assert!(!color_matches([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]));
    }
}