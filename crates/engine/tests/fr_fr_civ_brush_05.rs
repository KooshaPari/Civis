//! Tests for FR-CIV-BRUSH-05
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-05.

#[cfg(test)]
mod fr_fr_civ_brush_05 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-05 -- Material cluster has 4 modes: Replace, Drop, Erase, Surface.
    #[test]
    fn verify_fr_civ_brush_05_basic() {
        let modes = BrushCluster::Material.modes();
        assert_eq!(modes.len(), 4);
        assert_eq!(modes[0].label, "Replace");
        assert_eq!(modes[1].label, "Drop");
        assert_eq!(modes[2].label, "Erase");
        assert_eq!(modes[3].label, "Surface");
    }
}
