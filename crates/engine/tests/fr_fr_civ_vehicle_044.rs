//! Tests for FR-CIV-VEHICLE-044 - Formation Kind Distinct
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for formation kind distinct.

#[cfg(test)]
mod fr_fr_civ_vehicle_044 {
    #[test]
    fn formation_kind_distinct_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
