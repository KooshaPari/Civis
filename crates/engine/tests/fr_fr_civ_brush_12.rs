//! Tests for FR-CIV-BRUSH-12
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-BRUSH-12.

#[cfg(test)]
mod fr_fr_civ_brush_12 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-12 -- Policy cluster has 3 effect modes.
    #[test]
    fn verify_fr_civ_brush_12_basic() {
        let modes = BrushCluster::Policy.modes();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0].label, "Tax");
        assert_eq!(modes[1].label, "Edict");
        assert_eq!(modes[2].label, "Religion");
    }
}
