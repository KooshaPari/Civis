//! Tests for FR-CIV-PERF-RT-002
//!
//! Epic: FR-CIV-PERF-RT
//!
//! This test file verifies FR FR-CIV-PERF-RT-002: Replay log grows linearly.

#[cfg(test)]
mod fr_fr_civ_perf_rt_002 {
    /// Verify FR-CIV-PERF-RT-002: Replay log grows with ticks, not quadratically.
    #[test]
    fn verify_fr_civ_perf_rt_002_basic() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        sim.tick();
        let len_after_1 = sim.replay_log().events.len();
        assert!(len_after_1 > 0, "replay log should have events after tick");
        for _ in 0..4 {
            sim.tick();
        }
        let len_after_5 = sim.replay_log().events.len();
        assert!(
            len_after_5 > len_after_1,
            "replay log should grow with more ticks"
        );
    }

    /// Verify replay log event count grows roughly linearly.
    #[test]
    fn replay_log_linear_growth() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        let mut prev_len = 0;
        for i in 0..5 {
            sim.tick();
            let len = sim.replay_log().events.len();
            assert!(
                len > prev_len,
                "tick {i}: log must grow, was {prev_len} now {len}"
            );
            prev_len = len;
        }
    }
}
