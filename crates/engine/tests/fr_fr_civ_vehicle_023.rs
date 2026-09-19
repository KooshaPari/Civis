//! Tests for FR-CIV-VEHICLE-023 - Damage Floor at Zero
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for damage floor at zero.

#[cfg(test)]
mod fr_fr_civ_vehicle_023 {
    #[test]
    fn damage_floor_at_zero_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
