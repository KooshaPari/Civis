//! Tests for FR-CIV-VEHICLE-011
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-011.
//! Water-lane traversal speed uses medium coupling.

#[cfg(test)]
mod fr_fr_civ_vehicle_011 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-011 -- Water lane speed with current coupling.
    #[test]
    fn verify_fr_civ_vehicle_011_basic() {
        let base = 1.0;
        let lane = LaneClass::Water.speed_mult();
        let vehicle = 1.5; // sailing ship
        let load = 0.0;
        let congestion = 1.0;

        // Downstream (positive coupling > 1.0)
        let downstream = effective_speed(base, lane, vehicle, 1.3, load, congestion);
        // Upstream (negative coupling < 1.0)
        let upstream = effective_speed(base, lane, vehicle, 0.7, load, congestion);
        // No current (1.0)
        let neutral = effective_speed(base, lane, vehicle, 1.0, load, congestion);

        assert!(downstream > neutral, "downstream must be faster");
        assert!(upstream < neutral, "upstream must be slower");
    }
}
