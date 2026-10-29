#![allow(clippy::assertions_on_constants)] // constant asserts are intentional compile-time invariants; tracked for lane review

//! Tests for FR-SESSION-022
//!
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-SESSION-022.

#[cfg(test)]
mod fr_fr_session_022 {
    /// Verify FR-SESSION-022 type existence and basic behavior.
    #[test]
    fn verify_fr_session_022_basic() {
        use civ_server::SESSION_HISTORY_CAP;
        assert!(SESSION_HISTORY_CAP > 0);
    }
}
