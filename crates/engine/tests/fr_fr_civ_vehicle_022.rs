//! Tests for FR-CIV-VEHICLE-022
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-022.
//! Sailing ship speed varies with wind direction.

#[cfg(test)]
mod fr_fr_civ_vehicle_022 {
    use civ_engine::vehicle_types::effective_speed;

    /// FR-CIV-VEHICLE-022 -- Wind coupling affects sailing ship speed.
    #[test]
    fn verify_fr_civ_vehicle_022_basic() {
        let base = 1.0;
        let lane = 1.2; // water lane
        let ship = 2.2; // sailing ship
        let empty = 0.0;
        let congestion = 1.0;

        let downwind = effective_speed(base, lane, ship, 1.5, empty, congestion);
        let crosswind = effective_speed(base, lane, ship, 1.0, empty, congestion);
        let upwind = effective_speed(base, lane, ship, 0.5, empty, congestion);

        assert!(downwind > crosswind, "downwind > crosswind");
        assert!(crosswind > upwind, "crosswind > upwind");
    }
}
