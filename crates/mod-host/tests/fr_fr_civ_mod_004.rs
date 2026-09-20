//! Tests for FR-CIV-MOD-004 — ModStatus enum
//!
//! Epic: FR-CIV-MOD
//! Verifies the ModStatus enum variants.

#[cfg(test)]
mod fr_fr_civ_mod_004 {
    /// FR-CIV-MOD-004: ModStatus variants exist and are distinct.
    #[test]
    fn mod_status_variants() {
        use civ_mod_host::ModStatus;
        let a = ModStatus::Active;
        let b = ModStatus::Suspended;
        let c = ModStatus::Faulted;
        assert_ne!(a, b);
        assert_ne!(b, c);
    }

    /// FR-CIV-MOD-004: ModHost defaults to Active status for unknown mods.
    #[test]
    fn unknown_mod_defaults_to_active() {
        let host = civ_mod_host::ModHost::new();
        assert_eq!(host.mod_status("nonexistent"), civ_mod_host::ModStatus::Active);
    }
}
