//! Tests for FR-CIV-LLM-003
//!
//! Epic: FR-CIV-LLM
//!
//! This test file verifies FR FR-CIV-LLM-003: Simulation seeded determinism.

#[cfg(test)]
mod fr_fr_civ_llm_003 {
    /// Verify FR-CIV-LLM-003: Two simulations with same seed produce same initial state.
    #[test]
    fn verify_fr_civ_llm_003_basic() {
        let a = civ_engine::Simulation::with_seed(42u64);
        let b = civ_engine::Simulation::with_seed(42u64);
        assert_eq!(a.state.tick, b.state.tick, "tick must match");
        assert_eq!(
            a.state.population, b.state.population,
            "population must match"
        );
        assert_eq!(
            a.state.energy_budget_joules, b.state.energy_budget_joules,
            "energy budget must match"
        );
        assert_eq!(
            a.state.rng_seed, b.state.rng_seed,
            "rng seed must match"
        );
    }

    /// Verify different seeds produce different RNG seeds.
    #[test]
    fn different_seeds_different_state() {
        let a = civ_engine::Simulation::with_seed(1u64);
        let b = civ_engine::Simulation::with_seed(2u64);
        assert_ne!(a.state.rng_seed, b.state.rng_seed);
    }
}
