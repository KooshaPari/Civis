//! Tests for FR-CIV-DET-001 — Determinism
//!
//! Epic: FR-CIV-DET
//! Verifies that the simulation engine produces deterministic results:
//! identical seeds yield identical state, and the step function is reproducible.

#[cfg(test)]
mod fr_fr_civ_det_001 {
    /// FR-CIV-DET-001: Same seed produces same simulation state after tick.
    #[test]
    fn determinism_same_seed_same_output() {
        let mut sim1 = civ_engine::Simulation::with_seed(12345);
        let mut sim2 = civ_engine::Simulation::with_seed(12345);
        sim1.tick();
        sim2.tick();
        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(
            sim1.state.energy_budget_joules,
            sim2.state.energy_budget_joules
        );
        assert_eq!(sim1.state.population, sim2.state.population);
    }

    /// FR-CIV-DET-001: Different seeds produce different RNG output.
    #[test]
    fn different_seeds_produce_different_rng() {
        use rand::Rng;
        let mut rng1 = civ_engine::create_rng(1);
        let mut rng2 = civ_engine::create_rng(99999);
        let values1: Vec<u64> = (0..5).map(|_| rng1.gen()).collect();
        let values2: Vec<u64> = (0..5).map(|_| rng2.gen()).collect();
        assert_ne!(values1, values2);
    }

    /// FR-CIV-DET-001: Step function is deterministic with fixed-point arithmetic.
    #[test]
    fn step_function_is_deterministic() {
        let consumption = civ_engine::Fixed::from_num(200);
        let make_ws = || civ_engine::WorldState {
            tick: 5,
            energy_budget_joules: civ_engine::Fixed::from_num(1000),
            ..civ_engine::WorldState::default()
        };
        let r1 = civ_engine::step(make_ws(), consumption);
        let r2 = civ_engine::step(make_ws(), consumption);
        assert_eq!(r1.tick, r2.tick);
        assert_eq!(r1.energy_budget_joules, r2.energy_budget_joules);
    }

    /// FR-CIV-DET-001: Hash chain is deterministic for same inputs.
    #[test]
    fn hash_chain_is_deterministic() {
        use civ_engine::hash_hex;
        let bytes1 = [0xAB_u8; 32];
        let bytes2 = [0xAB_u8; 32];
        assert_eq!(hash_hex(&bytes1), hash_hex(&bytes2));
    }
}
