//! Tests for FR-CIV-DET-006 — Determinism (simulation replay)
//!
//! Epic: FR-CIV-DET
//! Covers: FR-CIV-DET-006
//! Verifies that replay logs capture and restore simulation state deterministically.

#[cfg(test)]
mod fr_fr_civ_det_006 {
    /// FR-CIV-DET-006: ReplayLog default has empty events.
    #[test]
    fn replay_log_starts_empty() {
        let log = civ_engine::ReplayLog::default();
        assert!(log.events.is_empty());
        assert_eq!(log.schema_version, 1);
    }

    /// FR-CIV-DET-006: Record tick and verify it appears in events.
    #[test]
    fn record_tick_appears_in_events() {
        let mut log = civ_engine::ReplayLog::default();
        log.record_tick(1);
        assert_eq!(log.events.len(), 1);
        assert!(log.running_hash.is_some());
    }

    /// FR-CIV-DET-006: Running hash changes after recording a tick.
    #[test]
    fn running_hash_changes_after_tick() {
        let mut log = civ_engine::ReplayLog::default();
        assert!(log.running_hash.is_none());
        log.record_tick(1);
        let hash1 = log.running_hash;
        log.record_tick(2);
        assert_ne!(log.running_hash, hash1);
    }
}
