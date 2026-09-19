//! Tests for FR-CIV-PERF-006
//!
//! Epic: FR-CIV-PERF
//!
//! This test file verifies FR FR-CIV-PERF-006: TickProfile slowest phase detection.

#[cfg(test)]
mod fr_fr_civ_perf_006 {
    /// Verify FR-CIV-PERF-006: TickProfile::slowest() returns the slowest phase.
    #[test]
    fn verify_fr_civ_perf_006_basic() {
        let mut profile = civ_engine::perf::TickProfile::default();
        profile.record("fast_phase", 50);
        profile.record("slow_phase", 900);
        profile.record("medium_phase", 200);
        let slowest = profile.slowest().expect("should have a slowest phase");
        assert_eq!(slowest.0, "slow_phase");
        assert_eq!(slowest.1, 900);
    }

    /// Verify slowest returns None for empty profile.
    #[test]
    fn slowest_empty_profile() {
        let profile = civ_engine::perf::TickProfile::default();
        assert!(profile.slowest().is_none());
    }

    /// Verify clear resets the profile.
    #[test]
    fn clear_resets_profile() {
        let mut profile = civ_engine::perf::TickProfile::default();
        profile.record("phase", 100);
        assert_eq!(profile.total_micros, 100);
        profile.clear();
        assert_eq!(profile.total_micros, 0);
        assert!(profile.phases.is_empty());
    }
}
