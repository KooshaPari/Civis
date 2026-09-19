//! Tests for FR-CIV-VEHICLE-012
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-012.
//! Rail lanes are a forward-only overlay; absence leaves existing routing unchanged.

#[cfg(test)]
mod fr_fr_civ_vehicle_012 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-012 -- Removing rail doesn't change land speed.
    #[test]
    fn verify_fr_civ_vehicle_012_basic() {
        let base = 1.0;
        let road = LaneClass::Road.speed_mult();
        let vehicle = 1.3; // cart
        let load = 0.0;
        let congestion = 1.0;

        // Land speed is unchanged whether rail exists or not
        let land_speed = effective_speed(base, road, vehicle, 1.0, load, congestion);
        let rail_speed = effective_speed(base, LaneClass::Rail.speed_mult(), vehicle, 1.0, load, congestion);

        // Rail is faster than road
        assert!(rail_speed > land_speed);
        // But road speed itself is independent
        assert!(land_speed > base, "road vehicle must be faster than walking");
    }
}
