//! Tests for FR-CIV-RTS-RENDER-001
//!
//!
//! This test file verifies FR FR-CIV-RTS-RENDER-001.
//! Maps to CIV-0600 FR-CIV-ASSET-001: SVG Template Rendering.

#[cfg(test)]
mod fr_fr_civ_rts_render_001 {
    use civ_engine::rts_types::SsConfig;

    /// FR-CIV-RTS-RENDER-001 -- SsConfig computes correct render dimensions.
    #[test]
    fn verify_fr_civ_rts_render_001_basic() {
        let ss = SsConfig::four_x(128, 128);
        assert_eq!(ss.factor, 4);
        assert_eq!(ss.render_width(), 512);
        assert_eq!(ss.render_height(), 512);
        assert_eq!(ss.output_width, 128);
        assert_eq!(ss.output_height, 128);
    }
}
