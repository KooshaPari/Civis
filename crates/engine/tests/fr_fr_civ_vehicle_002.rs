//! Tests for FR-CIV-VEHICLE-002 - Unit Position Tracking
//!
//! Epic: FR-CIV-VEHICLE
//! Military units SHALL track grid positions for movement.

#[cfg(test)]
mod fr_fr_civ_vehicle_002 {
    /// FR-CIV-VEHICLE-002: Position struct has x,y coordinates.
    #[test]
    fn position_has_coordinates() {
        let pos = civ_engine::Position { x: 10, y: 20 };
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
    }

    /// FR-CIV-VEHICLE-002: Norm-to-grid conversion is deterministic.
    #[test]
    fn norm_to_grid_deterministic() {
        let p1 = civ_engine::norm_to_grid(0.5, 0.5);
        let p2 = civ_engine::norm_to_grid(0.5, 0.5);
        assert_eq!(p1.x, p2.x);
        assert_eq!(p1.y, p2.y);
    }
}