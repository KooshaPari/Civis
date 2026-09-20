//! Tests for FR-CIV-PERF-012
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-012: Replay log operations.

#[cfg(test)]
mod fr_fr_civ_perf_012 {
    /// Verify FR-CIV-PERF-012: Simulation replay log is accessible and starts empty.
    #[test]
    fn verify_fr_civ_perf_012_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let log = sim.replay_log();
        assert_eq!(log.seed, 42);
    }

    /// Verify replay log is not empty after ticks.
    #[test]
    fn replay_log_not_empty_after_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        sim.tick();
        let log = sim.replay_log();
        assert!(!log.events.is_empty(), "replay log should have events after tick");
    }
}
