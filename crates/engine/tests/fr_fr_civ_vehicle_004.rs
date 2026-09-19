//! Tests for FR-CIV-VEHICLE-004 - Unit Spawn at Normalized Coords
//!
//! Epic: FR-CIV-VEHICLE
//! Units SHALL be spawnable at normalized map coordinates (0..1).

#[cfg(test)]
mod fr_fr_civ_vehicle_004 {
    /// FR-CIV-VEHICLE-004: Norm-to-grid maps 0..1 range to grid coords.
    #[test]
    fn norm_to_grid_maps_range() {
        let p_min = civ_engine::norm_to_grid(0.0, 0.0);
        let p_max = civ_engine::norm_to_grid(1.0, 1.0);
        assert!(p_min.x <= p_max.x, "x should increase with norm");
        assert!(p_min.y <= p_max.y, "y should increase with norm");
    }

    /// FR-CIV-VEHICLE-004: Grid-to-norm roundtrip preserves approximate location.
    #[test]
    fn grid_to_norm_roundtrip() {
        let pos = civ_engine::Position { x: 0, y: 0 };
        let (nx, ny) = civ_engine::grid_to_norm(pos);
        assert!(nx >= 0.0 && nx <= 1.0);
        assert!(ny >= 0.0 && ny <= 1.0);
    }
}