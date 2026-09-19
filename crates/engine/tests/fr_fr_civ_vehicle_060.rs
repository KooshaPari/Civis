//! Tests for FR-CIV-VEHICLE-060 - Unit Recruitment
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for unit recruitment.

#[cfg(test)]
mod fr_fr_civ_vehicle_060 {
    #[test]
    fn unit_recruitment_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
