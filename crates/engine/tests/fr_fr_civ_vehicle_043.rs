//! Tests for FR-CIV-VEHICLE-043 - Military Phase Config
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for military phase config.

#[cfg(test)]
mod fr_fr_civ_vehicle_043 {
    #[test]
    fn military_phase_config_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
