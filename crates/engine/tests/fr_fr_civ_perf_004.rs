//! Tests for FR-CIV-PERF-004
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-004: TickProfile phases_over_budget.

#[cfg(test)]
mod fr_fr_civ_perf_004 {
    /// Verify FR-CIV-PERF-004: TickProfile records and queries phase timings.
    #[test]
    fn verify_fr_civ_perf_004_basic() {
        let mut profile = civ_engine::perf::TickProfile::default();
        profile.record("economy", 500);
        profile.record("social", 200);
        profile.record("military", 100);
        assert_eq!(profile.total_micros, 800);
        assert_eq!(profile.phases.len(), 3);
    }

    /// Verify phases_over_budget filters correctly.
    #[test]
    fn phases_over_budget_filter() {
        let timings: Vec<civ_engine::perf::PhaseTiming> = vec![
            ("economy", 500),
            ("social", 200),
            ("military", 100),
        ];
        let over = civ_engine::perf::phases_over_budget(&timings, 300);
        assert_eq!(over.len(), 1);
        assert_eq!(over[0], ("economy", 500));
    }

    /// Verify tick_over_budget comparison.
    #[test]
    fn tick_over_budget_boundary() {
        let mut profile = civ_engine::perf::TickProfile::default();
        profile.record("phase", 100);
        assert!(civ_engine::perf::tick_over_budget(&profile, 100));
        assert!(!civ_engine::perf::tick_over_budget(&profile, 101));
    }
}
