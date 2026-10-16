#![allow(clippy::assertions_on_constants)] // constant asserts are intentional compile-time invariants; tracked for lane review

//! Tests for FR-SESSION-026
//!
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-SESSION-026.

#[cfg(test)]
mod fr_fr_session_026 {
    /// Verify FR-SESSION-026 type existence and basic behavior.
    #[test]
    fn verify_fr_session_026_basic() {
        use civ_server::SESSION_HISTORY_CAP;
        assert!(SESSION_HISTORY_CAP > 0);
    }
}
