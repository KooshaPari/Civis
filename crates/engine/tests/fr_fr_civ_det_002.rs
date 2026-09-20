//! Tests for FR-CIV-DET-002 — Determinism (hash chain)
//!
//! Epic: FR-CIV-DET
//! Covers: FR-CIV-DET-002
//! Verifies the per-tick BLAKE3 hash chain provides tamper-evident determinism.

#[cfg(test)]
mod fr_fr_civ_det_002 {
    /// FR-CIV-DET-002: Identical inputs produce identical chain links.
    #[test]
    fn identical_inputs_same_chain_link() {
        use civ_engine::hash_chain::{tick_event_bytes, tick_hash, GENESIS};
        let event = tick_event_bytes(7);
        let first = tick_hash(&GENESIS, &event);
        let second = tick_hash(&GENESIS, &event);
        assert_eq!(first, second);
    }

    /// FR-CIV-DET-002: Tampering with tick bytes changes the hash.
    #[test]
    fn tamper_changes_hash() {
        use civ_engine::hash_chain::{tick_event_bytes, tick_hash, GENESIS};
        let mut event = tick_event_bytes(42);
        let intact = tick_hash(&GENESIS, &event);
        event[0] ^= 0x01;
        let tampered = tick_hash(&GENESIS, &event);
        assert_ne!(intact, tampered);
    }

    /// FR-CIV-DET-002: Chain root from tick sequence matches incremental advance.
    #[test]
    fn chain_root_matches_incremental() {
        use civ_engine::hash_chain::{chain_root_from_ticks, tick_event_bytes, HashChainState};
        let ticks = [1u64, 2, 3];
        let mut state = HashChainState::new();
        for tick in ticks {
            state.advance(&tick_event_bytes(tick));
        }
        assert_eq!(chain_root_from_ticks(ticks), Some(state.running_hash));
    }

    /// FR-CIV-DET-002: Empty tick sequence returns None.
    #[test]
    fn empty_sequence_returns_none() {
        use civ_engine::hash_chain::chain_root_from_ticks;
        assert_eq!(chain_root_from_ticks([]), None);
    }

    /// FR-CIV-DET-002: hash_hex produces lowercase 64-char string.
    #[test]
    fn hash_hex_format() {
        use civ_engine::hash_chain::hash_hex;
        let hash = [0xAB_u8; 32];
        let hex = hash_hex(&hash);
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
