//! Tests for FR-CIV-PERF-011
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-011: Simulation tick determinism.

#[cfg(test)]
mod fr_fr_civ_perf_011 {
    /// Verify FR-CIV-PERF-011: Same seed produces identical tick outcomes.
    #[test]
    fn verify_fr_civ_perf_011_basic() {
        let mut a = civ_engine::Simulation::with_seed(42u64);
        let mut b = civ_engine::Simulation::with_seed(42u64);
        a.tick();
        b.tick();
        assert_eq!(a.state.tick, b.state.tick, "tick must match");
        assert_eq!(a.state.population, b.state.population, "population must match");
        assert_eq!(
            a.state.energy_budget_joules, b.state.energy_budget_joules,
            "energy budget must match"
        );
    }

    /// Verify determinism holds across multiple ticks.
    #[test]
    fn determinism_across_multiple_ticks() {
        let mut a = civ_engine::Simulation::with_seed(99u64);
        let mut b = civ_engine::Simulation::with_seed(99u64);
        for _ in 0..10 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.state.tick, b.state.tick);
        assert_eq!(a.state.population, b.state.population);
    }
}
