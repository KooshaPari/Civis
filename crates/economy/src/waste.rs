//! FR-ECON-005 — waste heat computation.
//!
//! Waste heat is computed as a percentage of total Joules consumed per tick.
//! The waste fraction is configurable (default 10 %) and represents energy
//! lost to inefficiency, entropy, and non-productive use.
//!
//! All math is integer-saturating. No floats accumulate across calls.

use serde::{Deserialize, Serialize};

/// Default waste-heat fraction in basis points (10 %).
pub const DEFAULT_WASTE_FRACTION_BP: i64 = 1_000;

/// Basis-point denominator (10_000 bp = 100 %).
const BP_DENOM: i64 = 10_000;

/// Configuration for waste-heat computation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasteHeatConfig {
    /// Waste-heat fraction in basis points `[0, 10_000]`.
    pub fraction_bp: i64,
}

impl Default for WasteHeatConfig {
    fn default() -> Self {
        Self {
            fraction_bp: DEFAULT_WASTE_FRACTION_BP,
        }
    }
}

/// Result of a waste-heat computation for one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasteHeatResult {
    /// Total joules consumed this tick.
    pub consumption_joules: i64,
    /// Joules lost as waste heat.
    pub waste_joules: i64,
    /// Joules productively used (consumption - waste).
    pub productive_joules: i64,
}

/// Compute waste heat from consumption.
///
/// `waste = consumption * fraction_bp / 10_000`
///
/// Returns zero waste for zero or negative consumption.
#[must_use]
pub fn compute_waste_heat(consumption_joules: i64, config: &WasteHeatConfig) -> WasteHeatResult {
    let consumption = consumption_joules.max(0);
    let fraction = config.fraction_bp.clamp(0, BP_DENOM);
    let waste = consumption * fraction / BP_DENOM;
    WasteHeatResult {
        consumption_joules: consumption,
        waste_joules: waste,
        productive_joules: consumption - waste,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waste_is_10_percent_by_default() {
        let r = compute_waste_heat(1000, &WasteHeatConfig::default());
        assert_eq!(r.waste_joules, 100);
        assert_eq!(r.productive_joules, 900);
    }

    #[test]
    fn zero_consumption_zero_waste() {
        let r = compute_waste_heat(0, &WasteHeatConfig::default());
        assert_eq!(r.waste_joules, 0);
        assert_eq!(r.productive_joules, 0);
    }

    #[test]
    fn negative_consumption_clamped() {
        let r = compute_waste_heat(-100, &WasteHeatConfig::default());
        assert_eq!(r.waste_joules, 0);
    }

    #[test]
    fn custom_fraction() {
        let config = WasteHeatConfig { fraction_bp: 2_500 }; // 25 %
        let r = compute_waste_heat(400, &config);
        assert_eq!(r.waste_joules, 100);
        assert_eq!(r.productive_joules, 300);
    }

    #[test]
    fn zero_fraction_no_waste() {
        let config = WasteHeatConfig { fraction_bp: 0 };
        let r = compute_waste_heat(500, &config);
        assert_eq!(r.waste_joules, 0);
        assert_eq!(r.productive_joules, 500);
    }

    #[test]
    fn full_fraction_all_waste() {
        let config = WasteHeatConfig { fraction_bp: 10_000 };
        let r = compute_waste_heat(500, &config);
        assert_eq!(r.waste_joules, 500);
        assert_eq!(r.productive_joules, 0);
    }

    #[test]
    fn conservation_waste_plus_productive_equals_consumption() {
        let r = compute_waste_heat(777, &WasteHeatConfig::default());
        assert_eq!(r.waste_joules + r.productive_joules, r.consumption_joules);
    }
}
