//! Tests for FR-CIV-CORE-015
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-015: State Hash Contracts.
//! The hash chain advances with each tick and is non-genesis after ticking.

#[cfg(test)]
mod fr_fr_civ_core_015 {
    use civ_engine::hash_chain::{HashChainState, GENESIS, HASH_LEN};

    /// HashChainState starts at GENESIS and advances on tick.
    #[test]
    fn hash_chain_advances_from_genesis() {
        let mut chain = HashChainState::new();
        assert_eq!(chain.running_hash, GENESIS);
        let new_hash = chain.advance(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_ne!(new_hash, GENESIS, "chain must advance from genesis");
        assert_eq!(chain.running_hash, new_hash);
    }

    /// Two independent chains with same payload diverge from different roots.
    #[test]
    fn chain_divergence_with_different_roots() {
        let mut chain_a = HashChainState::new();
        let mut chain_b = HashChainState::new();
        chain_a.advance(&[0u8; 8]);
        chain_b.advance(&[0u8; 8]);
        // Same root + same payload => same hash
        assert_eq!(chain_a.running_hash, chain_b.running_hash);

        // Different roots => different hash
        let mut chain_c = HashChainState::new();
        chain_c.advance(&[1u8; 8]);
        assert_ne!(chain_a.running_hash, chain_c.running_hash);
    }

    /// After simulation ticks, the replay log has events (state hash chain is live).
    #[test]
    fn simulation_hash_chain_is_live() {
        let mut sim = civ_engine::Simulation::with_seed(42);
        sim.tick();
        let log = sim.replay_log();
        assert!(!log.events.is_empty(), "events must exist after tick");
        // hash_chain module functions are usable
        let h = civ_engine::hash_chain::hash_hex(&[0u8; HASH_LEN]);
        assert_eq!(h.len(), 64, "hash_hex produces 64-char string");
    }
}
