//! Tests for FR-CIV-VEHICLE-042
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-042.
//! Bulk long-haul prefers high-capacity media emergently.

#[cfg(test)]
mod fr_fr_civ_vehicle_042 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-042 -- Rail has higher effective capacity-speed product than road.
    #[test]
    fn verify_fr_civ_vehicle_042_basic() {
        let base = 1.0;
        let vehicle = 1.0;
        let coupling = 1.0;
        let load = 0.0;
        let congestion = 1.0;

        // Rail: speed_mult = 2.0
        let rail_speed = effective_speed(base, LaneClass::Rail.speed_mult(), vehicle, coupling, load, congestion);
        // Road: speed_mult = 1.0
        let road_speed = effective_speed(base, LaneClass::Road.speed_mult(), vehicle, coupling, load, congestion);

        // Rail is strictly faster for bulk long-haul
        assert!(rail_speed > road_speed, "rail must be faster than road for bulk");
    }
}
