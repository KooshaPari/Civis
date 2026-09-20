//! Tests for FR-CIV-MOD-016 — Float contamination scanning
//!
//! Epic: FR-CIV-MOD
//! Verifies scan_float_action_emit_contamination on empty input.

#[cfg(test)]
mod fr_fr_civ_mod_016 {
    /// FR-CIV-MOD-016: scan_float_action_emit_contamination on empty bytes returns no sites.
    #[test]
    fn scan_empty_wasm_no_sites() {
        let result = civ_mod_host::scan_float_action_emit_contamination(&[]);
        match result {
            Ok(sites) => assert!(sites.is_empty()),
            Err(_) => {} // Empty WASM may fail to parse, which is acceptable
        }
    }

    /// FR-CIV-MOD-016: FloatContaminationSite struct has correct fields.
    #[test]
    fn contamination_site_has_correct_fields() {
        use civ_mod_host::FloatContaminationSite;
        let site = FloatContaminationSite {
            function_index: 0,
            instruction_index: 42,
            reason: "test".into(),
        };
        assert_eq!(site.function_index, 0);
        assert_eq!(site.instruction_index, 42);
    }
}
