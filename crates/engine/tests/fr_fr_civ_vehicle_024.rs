//! Tests for FR-CIV-VEHICLE-024
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-024.
//! Scalar speed path is unchanged when no vehicle is involved.

#[cfg(test)]
mod fr_fr_civ_vehicle_024 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-024 -- Foot baseline preserved (vehicle_mult = 1.0).
    #[test]
    fn verify_fr_civ_vehicle_024_basic() {
        let base = 1.0;
        let lane = LaneClass::Trail.speed_mult();
        // Foot baseline: vehicle speed mult = 1.0 (not a vehicle row)
        let foot_speed = effective_speed(base, lane, 1.0, 1.0, 0.0, 1.0);
        assert!(foot_speed >= base, "foot speed must be at least walk speed");
        // Trail is slightly slower than road
        let road_foot = effective_speed(base, LaneClass::Road.speed_mult(), 1.0, 1.0, 0.0, 1.0);
        assert!(road_foot >= foot_speed, "road foot >= trail foot");
    }
}
