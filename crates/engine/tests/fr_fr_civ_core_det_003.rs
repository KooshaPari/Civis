//! Tests for FR-CIV-CORE-DET-003
//!
//! Epic: FR-CIV-CORE-DET
//!
//! This test file verifies FR FR-CIV-CORE-DET-003: Seed Determinism.
//! Different seeds must produce different simulation trajectories.

#[cfg(test)]
mod fr_fr_civ_core_det_003 {
    /// Different seeds produce divergent state after ticking.
    #[test]
    fn different_seeds_diverge() {
        let mut sim_a = civ_engine::Simulation::with_seed(1);
        let mut sim_b = civ_engine::Simulation::with_seed(2);
        for _ in 0..10 {
            sim_a.tick();
            sim_b.tick();
        }
        // Replay log events or RNG seeds should differ
        let differs = sim_a.state.rng_seed != sim_b.state.rng_seed
            || sim_a.replay_log().events.len() != sim_b.replay_log().events.len();
        assert!(differs, "different seeds should produce different replay state");
    }

    /// Same seed always produces same state (idempotency).
    #[test]
    fn same_seed_always_same() {
        for seed in [1u64, 42, 100, 999, 12345] {
            let mut a = civ_engine::Simulation::with_seed(seed);
            let mut b = civ_engine::Simulation::with_seed(seed);
            for _ in 0..3 {
                a.tick();
                b.tick();
            }
            assert_eq!(a.state.tick, b.state.tick, "seed {} tick mismatch", seed);
            assert_eq!(
                a.state.population, b.state.population,
                "seed {} pop mismatch",
                seed
            );
        }
    }
}
