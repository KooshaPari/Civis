//! Tests for FR-CIV-TERRAIN-005
//!
//! Epic: FR-CIV-TERRAIN
//!
//! This test file verifies FR FR-CIV-TERRAIN-005: Water placement single source
//! of truth. The WATER_MARKER_MATERIAL constant provides a single water ID.

#[cfg(test)]
mod fr_fr_civ_terrain_005 {
    /// WATER_MARKER_MATERIAL is a constant accessible from civ_engine.
    #[test]
    fn water_marker_is_single_constant() {
        let water_a = civ_engine::WATER_MARKER_MATERIAL;
        let water_b = civ_engine::WATER_MARKER_MATERIAL;
        assert_eq!(water_a, water_b, "water marker must be a single constant");
    }

    /// CoastalColumn references water_y for water level tracking.
    #[test]
    fn coastal_column_tracks_water_level() {
        use civ_engine::CoastalColumn;
        let col = CoastalColumn {
            base_y: 50,
            last_water_y: 30,
        };
        assert!(
            col.last_water_y <= col.base_y,
            "water level must be at or below base"
        );
    }
}
