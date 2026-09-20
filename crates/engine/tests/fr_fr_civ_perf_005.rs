//! Tests for FR-CIV-PERF-005
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-005: Serialization timing.

#[cfg(test)]
mod fr_fr_civ_perf_005 {
    /// Verify FR-CIV-PERF-005: Multiple serialization rounds stay fast.
    #[test]
    fn verify_fr_civ_perf_005_basic() {
        let mut ws = civ_engine::WorldState::default();
        ws.tick = 42;
        ws.population = 10_000;
        const ITERS: u32 = 100;
        const ROUNDS: u32 = 5;
        let mut best_ms = u128::MAX;
        for _ in 0..ROUNDS {
            let start = std::time::Instant::now();
            for _ in 0..ITERS {
                let _ = serde_json::to_string(&ws).expect("serialize");
            }
            best_ms = best_ms.min(start.elapsed().as_millis());
        }
        assert!(
            best_ms < 200,
            "{ITERS} serializations took {best_ms}ms best-of-{ROUNDS}, exceeds 200ms"
        );
    }

    /// Verify serialization output is stable (deterministic).
    #[test]
    fn serialization_is_deterministic() {
        let mut ws = civ_engine::WorldState::default();
        ws.tick = 100;
        ws.population = 5000;
        let json1 = serde_json::to_string(&ws).expect("serialize");
        let json2 = serde_json::to_string(&ws).expect("serialize");
        assert_eq!(json1, json2, "serialization must be deterministic");
    }
}
