//! Tests for FR-CIV-PERF-RT-001
//!
//! Epic: FR-CIV-PERF-RT
//!
//! This test file verifies FR FR-CIV-PERF-RT-001: Runtime simulation tick timing.

#[cfg(test)]
mod fr_fr_civ_perf_rt_001 {
    /// Verify FR-CIV-PERF-RT-001: Simulation tick completes within time budget.
    #[test]
    fn verify_fr_civ_perf_rt_001_basic() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        let start = std::time::Instant::now();
        sim.tick();
        let elapsed = start.elapsed().as_millis();
        assert!(
            elapsed < 5000,
            "single tick took {elapsed}ms, exceeds 5s budget"
        );
    }

    /// Verify multiple ticks stay within budget.
    #[test]
    fn multiple_ticks_within_budget() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        let start = std::time::Instant::now();
        for _ in 0..10 {
            sim.tick();
        }
        let elapsed = start.elapsed().as_millis();
        assert!(
            elapsed < 30_000,
            "10 ticks took {elapsed}ms, exceeds 30s budget"
        );
        assert_eq!(sim.state.tick, 10);
    }
}
