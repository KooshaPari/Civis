//! Tests for FR-CIV-VEHICLE-020
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-020.
//! With all couplings at 1.0 and no load, effective_speed reduces to base*lane*vehicle.

#[cfg(test)]
mod fr_fr_civ_vehicle_020 {
    use civ_engine::vehicle_types::{effective_speed, LaneClass};

    /// FR-CIV-VEHICLE-020 -- Orthogonality proof: base*lane*vehicle when no coupling.
    #[test]
    fn verify_fr_civ_vehicle_020_basic() {
        let base = 1.0;
        let lane = LaneClass::Road.speed_mult(); // 1.0
        let vehicle = 1.7; // wagon
        let expected = base * lane * vehicle;

        let actual = effective_speed(base, lane, vehicle, 1.0, 0.0, 1.0);
        assert!((actual - expected).abs() < 0.001, "expected {}, got {}", expected, actual);
    }
}
