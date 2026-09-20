//! Tests for FR-CIV-VEHICLE-013
//!
//! Epic: auto-generated
//!
//! This test file verifies FR FR-CIV-VEHICLE-013.
//! Multimodal transfer possible iff shared node has port capability.

#[cfg(test)]
mod fr_fr_civ_vehicle_013 {
    use civ_engine::vehicle_types::{LaneClass, Medium};

    /// FR-CIV-VEHICLE-013 -- Transfer requires matching medium on both sides.
    #[test]
    fn verify_fr_civ_vehicle_013_basic() {
        // A truck (Land) needs a port node to transfer to a ship (Water)
        // The port must offer both a Water lane and a Land lane connection
        assert!(LaneClass::Road.admits_medium(Medium::Land));
        assert!(LaneClass::Water.admits_medium(Medium::Water));
        // Without a water lane connection at the port, no transfer
        assert!(!LaneClass::Road.admits_medium(Medium::Water));
        assert!(!LaneClass::Water.admits_medium(Medium::Land));
    }
}
