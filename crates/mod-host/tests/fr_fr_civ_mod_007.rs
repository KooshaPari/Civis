//! Tests for FR-CIV-MOD-007 — HookResult merging
//!
//! Epic: FR-CIV-MOD
//! Verifies that HookResult values can be merged.

#[cfg(test)]
mod fr_fr_civ_mod_007 {
    /// FR-CIV-MOD-007: merge_results Continue + Continue = Continue.
    #[test]
    fn merge_continue_continue() {
        use civ_mod_host::hooks::{HookResult, merge_results};
        assert!(matches!(merge_results(HookResult::Continue, HookResult::Continue), HookResult::Continue));
    }

    /// FR-CIV-MOD-007: HookResult enum variants exist.
    #[test]
    fn hook_result_variants() {
        use civ_mod_host::hooks::HookResult;
        let _ = HookResult::Continue;
        let _ = HookResult::Modify("test".into());
        let _ = HookResult::Cancel;
        let _ = HookResult::Replace("test".into());
    }
}
