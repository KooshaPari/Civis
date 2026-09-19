//! Tests for FR-CIV-PERF-003
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-003: JSON serialization timing.

#[cfg(test)]
mod fr_fr_civ_perf_003 {
    /// Verify FR-CIV-PERF-003: JSON serialization completes within budget.
    #[test]
    fn verify_fr_civ_perf_003_basic() {
        let ws = civ_engine::WorldState::default();
        let rounds = 100;
        let start = std::time::Instant::now();
        for _ in 0..rounds {
            let _ = serde_json::to_string(&ws).expect("serialize");
        }
        let elapsed = start.elapsed().as_millis();
        assert!(
            elapsed < 1000,
            "{rounds} serializations took {elapsed}ms, exceeds 1s budget"
        );
    }

    /// Verify deserialization roundtrip is also fast.
    #[test]
    fn deserialization_roundtrip_timing() {
        let ws = civ_engine::WorldState::default();
        let json = serde_json::to_string(&ws).expect("serialize");
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _: civ_engine::WorldState = serde_json::from_str(&json).expect("deserialize");
        }
        let elapsed = start.elapsed().as_millis();
        assert!(
            elapsed < 1000,
            "100 deserializations took {elapsed}ms, exceeds 1s budget"
        );
    }
}
