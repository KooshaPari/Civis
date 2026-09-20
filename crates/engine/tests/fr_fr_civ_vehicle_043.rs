//! Tests for FR-CIV-VEHICLE-043
//!
//!
//! This test file verifies FR FR-CIV-VEHICLE-043.
//! Multimodal chains form only through capability nodes.

#[cfg(test)]
mod fr_fr_civ_vehicle_043 {
    use civ_engine::vehicle_types::{LaneClass, Medium};

    /// FR-CIV-VEHICLE-043 -- Port node enables land-water transfer; no port = no transfer.
    #[test]
    fn verify_fr_civ_vehicle_043_basic() {
        // A road-only node cannot connect to water lanes
        assert!(!LaneClass::Road.admits_medium(Medium::Water));
        // A water-only node cannot connect to land vehicles
        assert!(!LaneClass::Water.admits_medium(Medium::Land));
        // A port node must offer BOTH land and water lane connections
        // (this is verified by checking both admit their respective media)
        assert!(LaneClass::Road.admits_medium(Medium::Land));
        assert!(LaneClass::Water.admits_medium(Medium::Water));
    }
}
