//! Tests for FR-CIV-BRUSH-10
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-BRUSH-10.

#[cfg(test)]
mod fr_fr_civ_brush_10 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-10 -- Disaster cluster has 6 effect modes.
    #[test]
    fn verify_fr_civ_brush_10_basic() {
        let modes = BrushCluster::Disaster.modes();
        assert_eq!(modes.len(), 6);
        assert_eq!(modes[0].label, "Meteor");
        assert_eq!(modes[5].label, "Plague");
    }
}
