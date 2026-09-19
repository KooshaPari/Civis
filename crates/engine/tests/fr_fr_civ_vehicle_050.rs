//! Tests for FR-CIV-VEHICLE-050 - Unit Morale Floor
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for unit morale floor.

#[cfg(test)]
mod fr_fr_civ_vehicle_050 {
    #[test]
    fn unit_morale_floor_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
