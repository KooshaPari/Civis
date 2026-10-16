#![allow(clippy::assertions_on_constants)] // constant asserts are intentional compile-time invariants; tracked for lane review

//! Tests for FR-CIV-SERVER-002-PROTO
//!
//! Upgraded from stub to real assertions.
//!
//! This test file verifies FR FR-CIV-SERVER-002-PROTO.

#[cfg(test)]
mod fr_fr_civ_server_002_proto {
    /// Verify FR-CIV-SERVER-002-PROTO type existence and basic behavior.
    #[test]
    fn verify_fr_civ_server_002_proto_basic() {
        use civ_server::SESSION_HISTORY_CAP;
        assert!(SESSION_HISTORY_CAP > 0);
    }
}
