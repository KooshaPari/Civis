//! Tests for FR-CIV-RTS-RENDER-004
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-RTS-RENDER-004.
//! Maps to CIV-0600 FR-CIV-ASSET-006: Power-of-Two Atlas Dimensions.

#[cfg(test)]
mod fr_fr_civ_rts_render_004 {
    use civ_engine::rts_types::{all_atlases, AtlasConfig};

    /// FR-CIV-RTS-RENDER-004 -- All atlas configs have power-of-two dimensions.
    #[test]
    fn verify_fr_civ_rts_render_004_basic() {
        for atlas in all_atlases() {
            assert!(
                atlas.is_pow2(),
                "Atlas '{}' has non-power-of-two dimensions: {}x{}",
                atlas.name,
                atlas.width,
                atlas.height
            );
            assert!(atlas.width.is_power_of_two());
            assert!(atlas.height.is_power_of_two());
        }
        // Verify specific sizes from CIV-0600 §7.2
        let terrain = AtlasConfig::terrain();
        assert_eq!(terrain.width, 2048);
        assert_eq!(terrain.height, 2048);
        let buildings = AtlasConfig::buildings();
        assert_eq!(buildings.width, 1024);
        assert_eq!(buildings.height, 1024);
        let citizens = AtlasConfig::citizens();
        assert_eq!(citizens.width, 512);
        assert_eq!(citizens.height, 512);
    }
}
