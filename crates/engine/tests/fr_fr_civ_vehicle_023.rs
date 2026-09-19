//! Tests for FR-CIV-VEHICLE-023
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-023.
//! Congestion can only slow, never speed up.

#[cfg(test)]
mod fr_fr_civ_vehicle_023 {
    use civ_engine::vehicle_types::effective_speed;

    /// FR-CIV-VEHICLE-023 -- Congestion in (0, 1] never speeds up.
    #[test]
    fn verify_fr_civ_vehicle_023_basic() {
        let base = 1.0;
        let lane = 1.0;
        let vehicle = 1.0;
        let coupling = 1.0;
        let load = 0.0;

        let no_congestion = effective_speed(base, lane, vehicle, coupling, load, 1.0);
        let mild = effective_speed(base, lane, vehicle, coupling, load, 0.8);
        let heavy = effective_speed(base, lane, vehicle, coupling, load, 0.3);

        assert!(no_congestion >= mild, "no congestion >= mild");
        assert!(mild >= heavy, "mild >= heavy");
        // Congestion never makes things faster
        assert!(heavy < no_congestion, "heavy congestion must be slower");
    }
}
