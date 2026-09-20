//! Tests for FR-CIV-BRUSH-06
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-06.

#[cfg(test)]
mod fr_fr_civ_brush_06 {
    use civ_engine::brush_types::{BrushCluster, ModeGroup};

    /// FR-CIV-BRUSH-06 -- Terraform cluster has Precise and God mode groups.
    #[test]
    fn verify_fr_civ_brush_06_basic() {
        let modes = BrushCluster::Terraform.modes();
        assert_eq!(modes.len(), 10);
        // First 6 are Precise group
        for m in &modes[..6] {
            assert_eq!(m.group, ModeGroup::Precise);
        }
        // Last 4 are God group
        for m in &modes[6..] {
            assert_eq!(m.group, ModeGroup::God);
        }
    }
}
