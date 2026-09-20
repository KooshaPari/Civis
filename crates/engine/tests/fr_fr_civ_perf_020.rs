//! Tests for FR-CIV-PERF-020
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-020: Invariant checking after ticks.

#[cfg(test)]
mod fr_fr_civ_perf_020 {
    /// Verify FR-CIV-PERF-020: Invariants pass after a tick.
    #[test]
    fn verify_fr_civ_perf_020_basic() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        sim.tick();
        civ_engine::invariants::check_tick_invariants(&sim)
            .expect("invariants should pass after tick");
    }

    /// Verify invariants hold for multiple ticks.
    #[test]
    fn invariants_across_multiple_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(7u64);
        for _ in 0..5 {
            sim.tick();
            civ_engine::invariants::check_tick_invariants(&sim)
                .expect("invariants must hold each tick");
        }
    }
}
