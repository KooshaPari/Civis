//! Tests for FR-CIV-MOD-002 — Permission checking
//!
//! Epic: FR-CIV-MOD
//! Verifies that ModPermissions controls access correctly.

#[cfg(test)]
mod fr_fr_civ_mod_002 {
    /// FR-CIV-MOD-002: Default permissions are all denied.
    #[test]
    fn default_permissions_deny_all() {
        let perms = civ_mod_host::ModPermissions::default();
        assert!(!perms.read_economy);
        assert!(!perms.read_climate);
        assert!(!perms.read_military);
        assert!(!perms.write_policy);
        assert!(!perms.write_economy);
        assert!(!perms.write_events);
        assert!(!perms.transfer_funds);
    }

    /// FR-CIV-MOD-002: PolicyActionKind roundtrips through emit_type.
    #[test]
    fn policy_action_roundtrip() {
        use civ_mod_host::{PolicyActionKind, policy_action_to_emit_type};
        for kind in [PolicyActionKind::SetTaxRate, PolicyActionKind::SetSubsidyRate] {
            let emit = kind.to_emit_type();
            let back = policy_action_to_emit_type(emit);
            assert!(back.is_some(), "roundtrip failed for {emit}");
        }
    }
}
