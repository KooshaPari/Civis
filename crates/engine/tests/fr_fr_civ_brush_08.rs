//! Tests for FR-CIV-BRUSH-08
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-BRUSH-08.

#[cfg(test)]
mod fr_fr_civ_brush_08 {
    use civ_engine::brush_types::BrushCluster;

    /// FR-CIV-BRUSH-08 -- Structure cluster has 7 building placement modes.
    #[test]
    fn verify_fr_civ_brush_08_basic() {
        let modes = BrushCluster::Structure.modes();
        assert_eq!(modes.len(), 7);
        let labels: Vec<_> = modes.iter().map(|m| m.label).collect();
        assert!(labels.contains(&"House"));
        assert!(labels.contains(&"Farm"));
        assert!(labels.contains(&"Monument"));
    }
}
