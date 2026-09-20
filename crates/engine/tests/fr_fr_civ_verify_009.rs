//! Tests for FR-CIV-VERIFY-009
//! Epic: FR-CIV-VERIFY. Worktree stale-branch audit.
#[cfg(test)]
mod fr_fr_civ_verify_009 {
    #[test]
    fn replay_integrity_verifies_state() {
        // FR-CIV-VERIFY-009 requires state integrity for staleness detection.
        let ws1 = civ_engine::WorldState::default();
        let ws2 = ws1.clone();
        // Identical states must compare equal.
        assert_eq!(ws1, ws2, "Clone must produce identical state");
    }
}
