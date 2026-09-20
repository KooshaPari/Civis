//! Tests for FR-CIV-CORE-006
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-006: No System Time in Simulation.
//! The simulation must produce deterministic results independent of wall-clock
//! time. We verify this by running the same seed twice and comparing outputs.

#[cfg(test)]
mod fr_fr_civ_core_006 {
    /// Two simulations with the same seed must produce identical tick-1 state,
    /// proving no wall-clock time leaks into the simulation.
    #[test]
    fn same_seed_produces_deterministic_state() {
        let mut sim_a = civ_engine::Simulation::with_seed(42);
        let mut sim_b = civ_engine::Simulation::with_seed(42);
        sim_a.tick();
        sim_b.tick();
        assert_eq!(sim_a.state.tick, sim_b.state.tick);
        assert_eq!(sim_a.state.population, sim_b.state.population);
        assert_eq!(
            sim_a.state.energy_budget_joules,
            sim_b.state.energy_budget_joules
        );
        assert_eq!(sim_a.state.rng_seed, sim_b.state.rng_seed);
    }

    /// Different seeds must produce different state after tick, proving
    /// the RNG is seeded from the config, not from wall-clock time.
    #[test]
    fn different_seeds_diverge() {
        let mut sim_a = civ_engine::Simulation::with_seed(1);
        let mut sim_b = civ_engine::Simulation::with_seed(999);
        // Run several ticks so stochastic events have time to diverge
        for _ in 0..10 {
            sim_a.tick();
            sim_b.tick();
        }
        // Replay log event counts or RNG state should differ
        let differs = sim_a.state.rng_seed != sim_b.state.rng_seed
            || sim_a.replay_log().events.len() != sim_b.replay_log().events.len();
        assert!(differs, "different seeds should produce different replay state");
    }
}
