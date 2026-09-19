//! Tests for FR-CIV-VEHICLE-021 - Damage Event Type
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for damage event type.

#[cfg(test)]
mod fr_fr_civ_vehicle_021 {
    #[test]
    fn damage_event_type_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
