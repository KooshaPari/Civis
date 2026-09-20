//! Tests for FR-CIV-VEHICLE-010
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-010.
//! Vehicle routes only over lanes admitting its medium.

#[cfg(test)]
mod fr_fr_civ_vehicle_010 {
    use civ_engine::vehicle_types::{LaneClass, Medium};

    /// FR-CIV-VEHICLE-010 -- Lane medium compatibility gate.
    #[test]
    fn verify_fr_civ_vehicle_010_basic() {
        // Land vehicle on land lanes
        assert!(LaneClass::Trail.admits_medium(Medium::Land));
        assert!(LaneClass::Road.admits_medium(Medium::Land));
        assert!(LaneClass::Highway.admits_medium(Medium::Land));
        // Land vehicle on water lane: no
        assert!(!LaneClass::Water.admits_medium(Medium::Land));
        // Water vehicle on water lane: yes
        assert!(LaneClass::Water.admits_medium(Medium::Water));
        // Rail vehicle on rail lane: yes
        assert!(LaneClass::Rail.admits_medium(Medium::Rail));
        // Air vehicle on air lane: yes
        assert!(LaneClass::Air.admits_medium(Medium::Air));
        // Cross-medium rejection
        assert!(!LaneClass::Rail.admits_medium(Medium::Land));
        assert!(!LaneClass::Air.admits_medium(Medium::Water));
    }
}
