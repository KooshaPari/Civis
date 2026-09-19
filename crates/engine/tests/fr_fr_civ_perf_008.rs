//! Tests for FR-CIV-PERF-008
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-008: Hash chain determinism.

#[cfg(test)]
mod fr_fr_civ_perf_008 {
    /// Verify FR-CIV-PERF-008: Hash chain is deterministic for identical inputs.
    #[test]
    fn verify_fr_civ_perf_008_basic() {
        use civ_engine::hash_chain::{tick_hash, tick_event_bytes, GENESIS};
        let event = tick_event_bytes(42);
        let first = tick_hash(&GENESIS, &event);
        let second = tick_hash(&GENESIS, &event);
        assert_eq!(first, second, "hash chain must be deterministic");
    }

    /// Verify different ticks produce different hashes.
    #[test]
    fn different_ticks_different_hashes() {
        use civ_engine::hash_chain::{tick_hash, tick_event_bytes, GENESIS};
        let hash_a = tick_hash(&GENESIS, &tick_event_bytes(1));
        let hash_b = tick_hash(&GENESIS, &tick_event_bytes(2));
        assert_ne!(hash_a, hash_b, "different inputs must produce different hashes");
    }

    /// Verify chain advances correctly.
    #[test]
    fn chain_advances() {
        use civ_engine::hash_chain::{tick_hash, tick_event_bytes, GENESIS};
        let first = tick_hash(&GENESIS, &tick_event_bytes(0));
        let second = tick_hash(&first, &tick_event_bytes(1));
        assert_ne!(first, GENESIS, "first step must differ from genesis");
        assert_ne!(second, first, "second step must differ from first");
    }
}
