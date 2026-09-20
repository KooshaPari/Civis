//! Tests for FR-CIV-VEHICLE-021
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-021.
//! Fully-laden vehicle is strictly slower than empty.

#[cfg(test)]
mod fr_fr_civ_vehicle_021 {
    use civ_engine::vehicle_types::effective_speed;

    /// FR-CIV-VEHICLE-021 -- Load factor monotone decreasing.
    #[test]
    fn verify_fr_civ_vehicle_021_basic() {
        let base = 1.0;
        let lane = 1.0;
        let vehicle = 1.5;
        let coupling = 1.0;
        let congestion = 1.0;

        let empty = effective_speed(base, lane, vehicle, coupling, 0.0, congestion);
        let half = effective_speed(base, lane, vehicle, coupling, 0.5, congestion);
        let full = effective_speed(base, lane, vehicle, coupling, 1.0, congestion);

        assert!(empty > half, "empty ({}) must be faster than half-laden ({})", empty, half);
        assert!(half > full, "half-laden ({}) must be faster than full ({})", half, full);
    }
}
