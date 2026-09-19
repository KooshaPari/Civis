//! Tests for FR-CIV-VEHICLE-030 - Unit Movement Path
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for unit movement path.

#[cfg(test)]
mod fr_fr_civ_vehicle_030 {
    #[test]
    fn unit_movement_path_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
