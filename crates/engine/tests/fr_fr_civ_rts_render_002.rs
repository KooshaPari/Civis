//! Tests for FR-CIV-RTS-RENDER-002
//!
//!
//! This test file verifies FR FR-CIV-RTS-RENDER-002.
//! Maps to CIV-0600 FR-CIV-ASSET-003: 4x Supersampling Required.

#[cfg(test)]
mod fr_fr_civ_rts_render_002 {
    use civ_engine::rts_types::SsConfig;

    /// FR-CIV-RTS-RENDER-002 -- 4x supersampling renders at 4x dimensions then downscales.
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
