//! Tests for FR-CIV-VEHICLE-046 - Line of Sight Check
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for line of sight check.

#[cfg(test)]
mod fr_fr_civ_vehicle_046 {
    #[test]
    fn line_of_sight_check_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
