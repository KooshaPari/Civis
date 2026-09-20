//! Tests for FR-CIV-PERF-RT-003
//!
//! Epic: FR-CIV-PERF-RT
//!
//! This test file verifies FR FR-CIV-PERF-RT-003: Simulation memory footprint.

#[cfg(test)]
mod fr_fr_civ_perf_rt_003 {
    /// Verify FR-CIV-PERF-RT-003: Simulation serializes to a reasonable size.
    #[test]
    fn verify_fr_civ_perf_rt_003_basic() {
        let sim = civ_engine::Simulation::with_seed(42u64);
        let json = serde_json::to_string(&sim.state).expect("serialize simulation state");
        let bytes = json.len();
        // A fresh simulation's WorldState should serialize under 1 MiB
        assert!(
            bytes < 1_048_576,
            "Simulation state serialized to {bytes} bytes, exceeds 1 MiB"
        );
    }

    /// Verify simulation state size after some ticks stays bounded.
    #[test]
    fn simulation_state_bounded_after_ticks() {
        let mut sim = civ_engine::Simulation::with_seed(42u64);
        for _ in 0..5 {
            sim.tick();
        }
        let json = serde_json::to_string(&sim.state).expect("serialize after ticks");
        assert!(
            json.len() < 1_048_576,
            "simulation state after 5 ticks serialized to {} bytes",
            json.len()
        );
    }
}
