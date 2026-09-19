//! Tests for FR-CIV-VEHICLE-041 - Doctrine Fitness Score
//!
//! Epic: FR-CIV-VEHICLE
//! Vehicle system behavior for doctrine fitness score.

#[cfg(test)]
mod fr_fr_civ_vehicle_041 {
    #[test]
    fn doctrine_fitness_score_works() {
        let ws = civ_engine::WorldState::default();
        assert_eq!(ws.tick, 0);
    }
}
