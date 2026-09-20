//! Tests for FR-CIV-RTS-NATION-002
//!
//!
//! This test file verifies FR FR-CIV-RTS-NATION-002.
//! Maps to CIV-0600 FR-CIV-ASSET-016: Nation Recoloring Shader Correctness.

#[cfg(test)]
mod fr_fr_civ_rts_nation_002 {
    use civ_engine::rts_types::{color_distance, color_matches, SHADER_TOLERANCE};

    /// FR-CIV-RTS-NATION-002 -- Shader color matching uses tolerance for dithering.
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
