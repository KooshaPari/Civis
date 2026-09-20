//! Tests for FR-CIV-MOD-012 — DeterminismError variants
//!
//! Epic: FR-CIV-MOD
//! Verifies that DeterminismError has the expected variants.

#[cfg(test)]
mod fr_fr_civ_mod_012 {
    /// FR-CIV-MOD-012: DeterminismError::FloatContamination variant works.
    #[test]
    fn float_contamination_error() {
        use civ_mod_host::DeterminismError;
        let err = DeterminismError::FloatContamination { count: 5 };
        let display = format!("{err}");
        assert!(display.contains("5"));
    }

    /// FR-CIV-MOD-012: DeterminismScanReport has correct fields.
    #[test]
    fn scan_report_has_correct_fields() {
        use civ_mod_host::DeterminismScanReport;
        let report = DeterminismScanReport::default();
        assert_eq!(report.float_instruction_count, 0);
        assert!(report.hard_rejections.is_empty());
        assert_eq!(report.float_contamination_site_count, 0);
    }
}
