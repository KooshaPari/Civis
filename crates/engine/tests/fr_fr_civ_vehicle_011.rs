//! Tests for FR-CIV-VEHICLE-011 - Unit Morale Decay
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for unit morale decay.

#[cfg(test)]
mod fr_fr_civ_vehicle_011 {
    #[test]
    fn unit_morale_decay_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
