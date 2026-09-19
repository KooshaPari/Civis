//! Tests for FR-REP-001 — Replay Export Format
//!
//! Epic: FR-REP
//! The engine SHALL export complete simulation runs to a replay format
//! containing a header, event log, and integrity verification.

use civ_engine::{Fixed, WorldState};

#[cfg(test)]
mod fr_fr_rep_001 {
    use super::*;

    /// FR-REP-001: Hash chain module produces deterministic hashes.
    #[test]
    fn hash_chain_deterministic() {
        let bytes = civ_engine::hash_chain::tick_event_bytes(42);
        let h1 = civ_engine::hash_chain::tick_hash(&civ_engine::hash_chain::GENESIS, &bytes);
        let h2 = civ_engine::hash_chain::tick_hash(&civ_engine::hash_chain::GENESIS, &bytes);
        assert_eq!(h1, h2, "same input must produce same hash");
    }

    /// FR-REP-001: Chain advances by incorporating prior hash.
    #[test]
    fn chain_includes_prior_hash() {
        let bytes1 = civ_engine::hash_chain::tick_event_bytes(1);
        let bytes2 = civ_engine::hash_chain::tick_event_bytes(2);
        let h1 = civ_engine::hash_chain::tick_hash(&civ_engine::hash_chain::GENESIS, &bytes1);
        let h2 = civ_engine::hash_chain::tick_hash(&h1, &bytes2);
        assert_ne!(h1, h2, "chain must advance");
        assert_ne!(h2, civ_engine::hash_chain::GENESIS, "chain must not revert");
    }

    /// FR-REP-001: Replay format version constant exists.
    #[test]
    fn replay_format_version_exists() {
        let _version = civ_engine::FORMAT_VERSION;
        let _magic = civ_engine::MAGIC;
    }

    /// FR-REP-001: SaveBundle metadata has spec ID.
    #[test]
    fn save_bundle_has_spec_id() {
        assert_eq!(civ_engine::CIVSAVE_SPEC_ID, "CIV-1000");
        assert!(civ_engine::CIVSAVE_FORMAT_VERSION > 0);
    }
}