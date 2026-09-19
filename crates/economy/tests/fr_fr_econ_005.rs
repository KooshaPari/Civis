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
}
