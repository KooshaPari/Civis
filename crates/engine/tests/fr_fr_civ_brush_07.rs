//! Tests for FR-CIV-BRUSH-07
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-07.

#[cfg(test)]
mod fr_fr_civ_brush_07 {
    use civ_engine::brush_types::{BrushCluster, ModeGroup};

    /// FR-CIV-BRUSH-07 -- Life cluster has Spawn and Effect mode groups.
    #[test]
    fn verify_fr_civ_brush_07_basic() {
        let modes = BrushCluster::Life.modes();
        assert_eq!(modes.len(), 6);
        assert_eq!(modes[0].group, ModeGroup::Spawn);
        assert_eq!(modes[1].group, ModeGroup::Spawn);
        assert_eq!(modes[2].group, ModeGroup::Effect);
        assert_eq!(modes[5].label, "Extinct");
    }
}
