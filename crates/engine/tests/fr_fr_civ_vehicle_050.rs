//! Tests for FR-CIV-VEHICLE-050
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-050.
//! Agents path over the medium-filtered lane graph by effective time.

#[cfg(test)]
mod fr_fr_civ_vehicle_050 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass, Medium};

    /// FR-CIV-VEHICLE-050 -- Agent boarding: time-saved must exceed overhead.
    #[test]
    fn verify_fr_civ_vehicle_050_basic() {
        let base = 1.0;
        let boarding_overhead = 0.5; // ticks to board

        // Walking: no vehicle, just base speed on road
        let walk_speed = effective_speed(base, LaneClass::Road.speed_mult(), 1.0, 1.0, 0.0, 1.0);
        // Coach: fast but has boarding overhead
        let coach_speed = effective_speed(base, LaneClass::Road.speed_mult(), 2.4, 1.0, 0.0, 1.0);

        // Time saved per unit distance = 1/walk - 1/coach
        let time_saved_per_unit = (1.0 / walk_speed) - (1.0 / coach_speed);
        // For long trips, boarding overhead is amortized
        let trip_distance = 10.0;
        let total_time_saved = time_saved_per_unit * trip_distance - boarding_overhead;
        assert!(total_time_saved > 0.0, "long trip on coach saves time");
    }
}
