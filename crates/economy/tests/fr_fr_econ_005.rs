//! Tests for FR-ECON-005 — Waste Heat Computation
//!
//! Epic: FR-ECON
//! The waste heat SHALL be computed as a percentage of total Joules consumed
//! per tick. Default fraction is 10%. All math is integer-saturating.

use civ_economy::{compute_waste_heat, WasteHeatConfig};

#[cfg(test)]
mod fr_fr_econ_005 {
    use super::*;

    /// FR-ECON-005: Waste heat is exactly 10% of consumption at default config.
    #[test]
    fn waste_heat_10_percent_of_consumption() {
        let config = WasteHeatConfig::default();
        let result = compute_waste_heat(1000, &config);
        assert_eq!(result.waste_joules, 100, "10% of 1000 = 100 waste joules");
        assert_eq!(
            result.productive_joules, 900,
            "productive = consumption - waste"
        );
        assert_eq!(result.consumption_joules, 1000);
    }

    /// FR-ECON-005: Zero consumption produces zero waste heat.
    #[test]
    fn zero_consumption_zero_waste() {
        let config = WasteHeatConfig::default();
        let result = compute_waste_heat(0, &config);
        assert_eq!(result.waste_joules, 0);
        assert_eq!(result.productive_joules, 0);
    }

    /// FR-ECON-005: Negative consumption is clamped to zero.
    #[test]
    fn negative_consumption_clamped_to_zero() {
        let config = WasteHeatConfig::default();
        let result = compute_waste_heat(-500, &config);
        assert_eq!(result.consumption_joules, 0);
        assert_eq!(result.waste_joules, 0);
    }

    /// FR-ECON-005: Custom waste fraction (25%) applies correctly.
    #[test]
    fn custom_waste_fraction() {
        let config = WasteHeatConfig { fraction_bp: 2_500 }; // 25%
        let result = compute_waste_heat(400, &config);
        assert_eq!(result.waste_joules, 100, "25% of 400 = 100");
        assert_eq!(result.productive_joules, 300);
    }

    /// FR-ECON-005: Zero waste fraction means no energy lost.
    #[test]
    fn zero_fraction_no_waste() {
        let config = WasteHeatConfig { fraction_bp: 0 };
        let result = compute_waste_heat(1000, &config);
        assert_eq!(result.waste_joules, 0);
        assert_eq!(result.productive_joules, 1000);
    }

    /// FR-ECON-005: Conservation invariant — waste + productive = consumption.
    #[test]
    fn conservation_invariant() {
        let config = WasteHeatConfig { fraction_bp: 3_500 }; // 35%
        let result = compute_waste_heat(10_000, &config);
        assert_eq!(
            result.waste_joules + result.productive_joules,
            result.consumption_joules,
            "waste + productive must equal consumption"
        );
    }

    /// FR-ECON-005: Large consumption value does not overflow (integer saturating).
    #[test]
    fn large_consumption_no_overflow() {
        let config = WasteHeatConfig::default();
        let result = compute_waste_heat(i64::MAX / 2, &config);
        assert!(result.waste_joules > 0);
        assert!(result.productive_joules > 0);
    }

    /// FR-ECON-005: an out-of-range fraction is clamped, not honoured.
    ///
    /// `fraction_bp` is documented as living in `[0, 10_000]`. Nothing enforces
    /// that on the public struct, so `compute_waste_heat` has to. Without the
    /// clamp, `fraction_bp = 50_000` returns waste five times consumption, which
    /// makes `productive_joules` negative and breaks the conservation invariant
    /// the test above checks. Mutation testing showed this clamp was untested:
    /// deleting it left the whole suite green.
    #[test]
    fn out_of_range_fraction_is_clamped() {
        for fraction_bp in [10_001, 50_000, i64::MAX] {
            let config = WasteHeatConfig { fraction_bp };
            let result = compute_waste_heat(1_000, &config);
            assert_eq!(
                result.waste_joules, 1_000,
                "fraction_bp {fraction_bp} must clamp to 100% waste, not scale with the input"
            );
            assert_eq!(
                result.productive_joules, 0,
                "productive joules must not go negative at fraction_bp {fraction_bp}"
            );
            assert_eq!(
                result.waste_joules + result.productive_joules,
                result.consumption_joules,
                "conservation must hold at fraction_bp {fraction_bp}"
            );
        }
    }

    /// FR-ECON-005: a negative fraction is clamped to zero waste, not negative waste.
    #[test]
    fn negative_fraction_is_clamped_to_zero() {
        for fraction_bp in [-1, -10_000, i64::MIN] {
            let config = WasteHeatConfig { fraction_bp };
            let result = compute_waste_heat(1_000, &config);
            assert_eq!(
                result.waste_joules, 0,
                "fraction_bp {fraction_bp} must clamp to zero waste, not negative waste"
            );
            assert_eq!(
                result.productive_joules, 1_000,
                "all consumption stays productive at fraction_bp {fraction_bp}"
            );
        }
    }

    /// FR-ECON-005: the exact clamp boundaries behave as documented.
    #[test]
    fn clamp_boundaries_are_exact() {
        let zero = compute_waste_heat(1_000, &WasteHeatConfig { fraction_bp: 0 });
        assert_eq!((zero.waste_joules, zero.productive_joules), (0, 1_000));

        let full = compute_waste_heat(1_000, &WasteHeatConfig { fraction_bp: 10_000 });
        assert_eq!((full.waste_joules, full.productive_joules), (1_000, 0));
    }
}
