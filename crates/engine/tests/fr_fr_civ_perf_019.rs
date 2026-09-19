//! Tests for FR-CIV-PERF-019
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-019: Simulation with different seeds.

#[cfg(test)]
mod fr_fr_civ_perf_019 {
    /// Verify FR-CIV-PERF-019: Different seeds produce different RNG seeds.
    #[test]
    fn verify_fr_civ_perf_019_basic() {
        let sim_a = civ_engine::Simulation::with_seed(1u64);
        let sim_b = civ_engine::Simulation::with_seed(2u64);
        assert_ne!(sim_a.state.rng_seed, sim_b.state.rng_seed);
    }

    /// Verify SimSeed wrapper works with Simulation::with_seed.
    #[test]
    fn simseed_wrapper_works() {
        let seed = civ_engine::SimSeed::from_u64(42);
        let sim = civ_engine::Simulation::with_seed(seed);
        assert_eq!(sim.state.rng_seed, 42);
    }
}
