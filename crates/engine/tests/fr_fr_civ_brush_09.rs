//! Tests for FR-CIV-BRUSH-09
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-09.

#[cfg(test)]
mod fr_fr_civ_brush_09 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-09 -- Infrastructure cluster has 5 placement modes.
    #[test]
    fn verify_fr_civ_brush_09_basic() {
        let modes = BrushCluster::Infrastructure.modes();
        assert_eq!(modes.len(), 5);
        assert_eq!(modes[0].label, "Road");
        assert_eq!(modes[4].label, "Canal");
    }
}
