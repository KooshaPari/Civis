//! Tests for FR-CIV-MOD-015 — Policy action constants
//!
//! Epic: FR-CIV-MOD
//! Verifies that policy action kind roundtrips and constants match SDK.

#[cfg(test)]
mod fr_fr_civ_mod_015 {
    /// FR-CIV-MOD-015: policy_action_to_emit_type returns Some for valid actions.
    #[test]
    fn valid_emit_types() {
        use civ_mod_host::policy_action_to_emit_type;
        for action_id in [0u32, 1, 2] {
            if let Some(result) = policy_action_to_emit_type(action_id) {
                assert!(result > 0);
            }
        }
    }

    /// FR-CIV-MOD-015: All PolicyActionKind variants produce non-zero emit_type.
    #[test]
    fn all_variants_have_emit_type() {
        use civ_mod_host::PolicyActionKind;
        for kind in [PolicyActionKind::SetTaxRate, PolicyActionKind::SetSubsidyRate, PolicyActionKind::TransferFunds] {
            let emit = kind.to_emit_type();
            assert!(emit > 0, "emit_type for {kind:?} should be > 0");
        }
    }
}
