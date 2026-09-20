//! Tests for FR-CIV-BRUSH-11
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-11.

#[cfg(test)]
mod fr_fr_civ_brush_11 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-11 -- Diplomacy cluster has 3 relation modes.
    #[test]
    fn verify_fr_civ_brush_11_basic() {
        let modes = BrushCluster::Diplomacy.modes();
        assert_eq!(modes.len(), 3);
        assert_eq!(modes[0].label, "Alliance");
        assert_eq!(modes[1].label, "War");
        assert_eq!(modes[2].label, "Trade");
    }
}
