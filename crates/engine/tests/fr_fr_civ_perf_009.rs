//! Tests for FR-CIV-PERF-009
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-009: Hash chain state machine.

#[cfg(test)]
mod fr_fr_civ_perf_009 {
    /// Verify FR-CIV-PERF-009: HashChainState advances correctly.
    #[test]
    fn verify_fr_civ_perf_009_basic() {
        use civ_engine::hash_chain::{HashChainState, tick_event_bytes, GENESIS};
        let mut chain = HashChainState::new();
        assert_eq!(chain.running_hash, GENESIS, "starts at genesis");
        let h1 = chain.advance(&tick_event_bytes(1));
        assert_ne!(chain.running_hash, GENESIS, "advanced from genesis");
        assert_eq!(chain.running_hash, h1);
    }

    /// Verify chain_root_from_ticks computes correctly.
    #[test]
    fn chain_root_from_ticks_nonempty() {
        use civ_engine::hash_chain::{chain_root_from_ticks, GENESIS};
        let root = chain_root_from_ticks(0..5);
        assert!(root.is_some(), "non-empty iterator should produce a root");
        assert_ne!(root.unwrap(), GENESIS, "root must differ from genesis");
    }

    /// Verify chain_root_from_ticks is None for empty iterator.
    #[test]
    fn chain_root_from_ticks_empty() {
        use civ_engine::hash_chain::chain_root_from_ticks;
        assert!(chain_root_from_ticks(std::iter::empty::<u64>()).is_none());
    }
}
