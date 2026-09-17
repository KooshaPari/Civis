//! FR-CLIM-002 — Global mean temperature derived from CO₂ concentration.
//!
//! Uses a parameterised logarithmic formula calibrated to IPCC AR6:
//!   ΔT = sensitivity * ln(CO₂ / CO₂_pre) / ln(2)
//!
//! where `sensitivity` is the equilibrium climate sensitivity (°C per doubling
//! of CO₂). The formula returns the temperature anomaly; the absolute mean
//! temperature adds the pre-industrial baseline (14 °C).

use serde::{Deserialize, Serialize};

use crate::co2::PRE_INDUSTRIAL_CO2_PPM;

/// Pre-industrial baseline temperature (°C).
pub const BASELINE_TEMP_C: f64 = 14.0;

/// Default equilibrium climate sensitivity (°C per CO₂ doubling).
/// IPCC AR6 likely range: 2.5–4.0 °C; central estimate ≈ 3.0 °C.
pub const DEFAULT_ECS_C: f64 = 3.0;

/// Configuration for the temperature derivation model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureConfig {
    /// Equilibrium climate sensitivity (°C per doubling of CO₂).
    pub ecs_c: f64,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            ecs_c: DEFAULT_ECS_C,
        }
    }
}

/// Compute the global mean temperature anomaly (°C) from CO₂ concentration.
///
/// Uses the logarithmic forcing relationship:
///   ΔT = ECS × ln(CO₂ / CO₂_pre) / ln(2)
///
/// Returns 0.0 if CO₂ is at or below pre-industrial levels.
#[must_use]
pub fn temperature_anomaly_from_co2(co2_ppm: f64, config: &TemperatureConfig) -> f64 {
    if co2_ppm <= PRE_INDUSTRIAL_CO2_PPM {
        return 0.0;
    }
    let ratio = co2_ppm / PRE_INDUSTRIAL_CO2_PPM;
    config.ecs_c * ratio.ln() / 2.0_f64.ln()
}

/// Compute global mean surface temperature (°C) from CO₂ concentration.
///
/// Returns `BASELINE_TEMP_C + anomaly`.
#[must_use]
pub fn temperature_from_co2(co2_ppm: f64, config: &TemperatureConfig) -> f64 {
    BASELINE_TEMP_C + temperature_anomaly_from_co2(co2_ppm, config)
}

/// Tracks temperature derived from CO₂ and thermal inertia.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureTracker {
    /// Current global mean temperature (°C).
    pub mean_temp_c: f64,
    /// Current temperature anomaly above baseline (°C).
    pub anomaly_c: f64,
}

impl TemperatureTracker {
    /// Create at pre-industrial equilibrium.
    pub fn new() -> Self {
        Self {
            mean_temp_c: BASELINE_TEMP_C,
            anomaly_c: 0.0,
        }
    }

    /// Update temperature from current CO₂ level.
    pub fn update_from_co2(&mut self, co2_ppm: f64, config: &TemperatureConfig) {
        self.anomaly_c = temperature_anomaly_from_co2(co2_ppm, config);
        self.mean_temp_c = BASELINE_TEMP_C + self.anomaly_c;
    }
}

impl Default for TemperatureTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preindustrial_co2_gives_zero_anomaly() {
        let cfg = TemperatureConfig::default();
        let anomaly = temperature_anomaly_from_co2(PRE_INDUSTRIAL_CO2_PPM, &cfg);
        assert!((anomaly).abs() < 1e-10);
    }

    #[test]
    fn doubling_co2_gives_ecs_anomaly() {
        let cfg = TemperatureConfig {
            ecs_c: 3.0,
            ..Default::default()
        };
        let doubled = PRE_INDUSTRIAL_CO2_PPM * 2.0;
        let anomaly = temperature_anomaly_from_co2(doubled, &cfg);
        // At doubling, anomaly should equal ECS exactly.
        assert!((anomaly - 3.0).abs() < 1e-10);
    }

    #[test]
    fn temperature_increases_with_co2() {
        let cfg = TemperatureConfig::default();
        let t_low = temperature_from_co2(350.0, &cfg);
        let t_high = temperature_from_co2(500.0, &cfg);
        assert!(t_high > t_low);
    }

    #[test]
    fn below_preindustrial_returns_baseline() {
        let cfg = TemperatureConfig::default();
        let t = temperature_from_co2(200.0, &cfg);
        assert!((t - BASELINE_TEMP_C).abs() < 1e-10);
    }

    #[test]
    fn tracker_update_reflects_co2() {
        let mut tracker = TemperatureTracker::new();
        let cfg = TemperatureConfig::default();
        tracker.update_from_co2(450.0, &cfg);
        assert!(tracker.mean_temp_c > BASELINE_TEMP_C);
        assert!(tracker.anomaly_c > 0.0);
    }

    #[test]
    fn custom_ecs_scales_anomaly() {
        let cfg_high = TemperatureConfig { ecs_c: 6.0 };
        let cfg_low = TemperatureConfig { ecs_c: 2.0 };
        let co2 = 560.0; // double pre-industrial
        let a_high = temperature_anomaly_from_co2(co2, &cfg_high);
        let a_low = temperature_anomaly_from_co2(co2, &cfg_low);
        // High ECS should give triple the anomaly of low ECS.
        assert!((a_high / a_low - 3.0).abs() < 1e-10);
    }
}
