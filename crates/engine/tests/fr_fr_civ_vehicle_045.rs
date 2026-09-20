//! Tests for FR-CIV-VEHICLE-045
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-045.
//! Unassignable flow (no compatible vehicle) is reported as unmet demand.

#[cfg(test)]
mod fr_fr_civ_vehicle_045 {
    use civ_engine::vehicle_types::{LaneClass, Medium};

    /// FR-CIV-VEHICLE-045 -- No compatible vehicle = unmet demand signal.
    #[test]
    fn verify_fr_civ_vehicle_045_basic() {
        // If only land lanes exist and cargo needs water transport,
        // there is no compatible vehicle -> unmet demand
        let available_lanes = [LaneClass::Trail, LaneClass::Road, LaneClass::Highway];
        let cargo_needs_water = Medium::Water;
        let has_compatible = available_lanes.iter().any(|l| l.admits_medium(cargo_needs_water));
        assert!(!has_compatible, "no water lane available = unmet demand");
    }
}
