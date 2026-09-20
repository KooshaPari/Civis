//! Tests for FR-CIV-VEHICLE-014
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-014.
//! Adding a LaneClass variant does not change land-only outcomes.

#[cfg(test)]
mod fr_fr_civ_vehicle_014 {
    use civ_engine::vehicle_types::{LaneClass, Medium};

    /// FR-CIV-VEHICLE-014 -- Additive LaneClass proof: land outcomes stable.
    #[test]
    fn verify_fr_civ_vehicle_014_basic() {
        // Land admission is stable across all lane classes
        let land_lanes = [LaneClass::Trail, LaneClass::Road, LaneClass::Highway];
        for lane in land_lanes {
            assert!(lane.admits_medium(Medium::Land));
        }
        // Non-land classes still reject land
        assert!(!LaneClass::Water.admits_medium(Medium::Land));
        assert!(!LaneClass::Rail.admits_medium(Medium::Land));
        assert!(!LaneClass::Air.admits_medium(Medium::Land));
    }
}
