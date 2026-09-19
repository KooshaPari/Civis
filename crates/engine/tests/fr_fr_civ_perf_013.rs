//! Tests for FR-CIV-PERF-013
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-013: Replay log Tick event.

#[cfg(test)]
mod fr_fr_civ_perf_013 {
    /// Verify FR-CIV-PERF-013: Replay log has events after tick.
    #[test]
    fn verify_fr_civ_perf_013_basic() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        sim.tick();
        let log = sim.replay_log();
        // Replay log should have events after a tick
        assert!(!log.events.is_empty(), "replay log must have events after tick");
    }

    /// Verify replay log event count increases with ticks.
    #[test]
    fn replay_log_grows_with_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(7u64);
        sim.tick();
        let len1 = sim.replay_log().events.len();
        sim.tick();
        let len2 = sim.replay_log().events.len();
        assert!(len2 > len1, "replay log should grow with each tick");
    }
}
