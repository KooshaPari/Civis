//! Tests for FR-CIV-MOD-009 — Host capability imports
//!
//! Epic: FR-CIV-MOD
//! Verifies that HOST_CAPABILITY_IMPORTS is populated.

#[cfg(test)]
mod fr_fr_civ_mod_009 {
    /// FR-CIV-MOD-009: HOST_CAPABILITY_IMPORTS is non-empty.
    #[test]
    fn capability_imports_nonempty() {
        assert!(!civ_mod_host::HOST_CAPABILITY_IMPORTS.is_empty());
    }

    /// FR-CIV-MOD-009: HOST_GUEST_MEMORY_CAP is positive.
    #[test]
    fn guest_memory_cap_positive() {
        assert!(civ_mod_host::HOST_GUEST_MEMORY_CAP > 0);
    }
}
