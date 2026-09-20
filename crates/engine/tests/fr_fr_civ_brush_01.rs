//! Tests for FR-CIV-BRUSH-01
//!
//!
//! This test file verifies FR FR-CIV-BRUSH-01.

#[cfg(test)]
mod fr_fr_civ_brush_01 {
    use civ_engine::brush_types::{BrushCluster, BrushSettings};

    /// FR-CIV-BRUSH-01 — BrushSettings struct exists with required fields
    /// and default values for radius, strength, falloff, and shape.
    #[test]
    fn verify_fr_civ_brush_01_basic() {
        let bs = BrushSettings::default();
        assert!(bs.radius > 0, "default radius must be > 0");
        assert!(bs.strength > 0, "default strength must be > 0");
        assert!(bs.falloff == civ_engine::brush_types::BrushFalloff::Linear);
        assert!(bs.shape == civ_engine::brush_types::BrushShape::Disc);
    }
}
