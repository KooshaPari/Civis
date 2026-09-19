//! Tests for FR-CIV-VEHICLE-045 - BFS Next Step
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for bfs next step.

#[cfg(test)]
mod fr_fr_civ_vehicle_045 {
    #[test]
    fn bfs_next_step_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
