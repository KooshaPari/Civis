//! Behavior tests for `atlas config dimensions are pow2`.
//!
//! These assertions are real and were previously filed under `FR-CIV-RTS-RENDER-004`.
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
mod rts_atlas_power_of_two {
    use civ_engine::rts_types::{all_atlases, AtlasConfig};
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