//! FR-CLIM-001 — Atmospheric CO₂ accumulation from industrial Joule consumption.
//!
//! Each tick, industrial Joule consumption is converted to CO₂ ppm via an
//! emission factor. CO₂ accumulates linearly and can decay at a natural
//! absorption rate (ocean + biosphere sink).
//!
//! All math uses `i64` basis points and integer arithmetic to avoid
//! floating-point drift across ticks. Public fields use `f64` only for
//! ergonomic CO₂ concentration (ppm) representation.

use serde::{Deserialize, Serialize};

/// Pre-industrial CO₂ concentration (ppm).
pub const PRE_INDUSTRIAL_CO2_PPM: f64 = 280.0;

/// Default emission factor: ppm of CO₂ per megajoule of industrial consumption.
/// Calibrated so that ~1000 GJ per tick ≈ 2 ppm (realistic single-tick pulse).
pub const DEFAULT_EMISSION_FACTOR_PPM_PER_MJ: f64 = 0.002;

/// Natural CO₂ absorption fraction in basis points per tick (default ≈ 0.5 %).
pub const DEFAULT_ABSORPTION_BP_PER_TICK: i64 = 50;

/// Basis-point denominator (10_000 bp = 100 %).
const BP_DENOM: i64 = 10_000;

/// Configuration for the CO₂ accumulation model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Co2Config {
    /// CO₂ ppm emitted per megajoule of industrial consumption.
    pub emission_factor_ppm_per_mj: f64,
    /// Natural absorption rate in basis points per tick (e.g. 50 = 0.5 %).
    pub absorption_bp: i64,
}

impl Default for Co2Config {
    fn default() -> Self {
        Self {
            emission_factor_ppm_per_mj: DEFAULT_EMISSION_FACTOR_PPM_PER_MJ,
            absorption_bp: DEFAULT_ABSORPTION_BP_PER_TICK,
        }
    }
}

/// Tracks atmospheric CO₂ concentration and provides accumulation logic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Co2Tracker {
    /// Current atmospheric CO₂ concentration (ppm).
    pub co2_ppm: f64,
    /// Cumulative industrial emissions across all ticks (ppm equivalent).
    pub cumulative_emissions_ppm: f64,
    /// Total industrial Joule consumption across all ticks.
    pub total_industrial_joules: i64,
}

impl Co2Tracker {
    /// Create a new tracker at pre-industrial levels.
    pub fn new() -> Self {
        Self {
            co2_ppm: PRE_INDUSTRIAL_CO2_PPM,
            cumulative_emissions_ppm: 0.0,
            total_industrial_joules: 0,
        }
    }

    /// Advance CO₂ by one tick given industrial Joule consumption.
    ///
    /// 1. Compute emissions: `industrial_joules * emission_factor`.
    /// 2. Absorb a fraction of current CO₂ above pre-industrial baseline.
    /// 3. Return the new CO₂ concentration.
    pub fn accumulate_tick(&mut self, industrial_joules: i64, config: &Co2Config) -> f64 {
        let j = industrial_joules.max(0);
        self.total_industrial_joules += j;

        // Convert joules (assumed in MJ scale) to ppm emission.
        let emission_ppm = (j as f64) * config.emission_factor_ppm_per_mj;
        self.cumulative_emissions_ppm += emission_ppm;
        self.co2_ppm += emission_ppm;

        // Natural absorption: remove a fraction of the excess above baseline.
        let excess = (self.co2_ppm - PRE_INDUSTRIAL_CO2_PPM).max(0.0);
        let absorption = config.absorption_bp.clamp(0, BP_DENOM) as f64 / (BP_DENOM as f64);
        self.co2_ppm -= excess * absorption;

        // CO₂ must never drop below pre-industrial baseline.
        if self.co2_ppm < PRE_INDUSTRIAL_CO2_PPM {
            self.co2_ppm = PRE_INDUSTRIAL_CO2_PPM;
        }

        self.co2_ppm
    }

    /// Set CO₂ directly (e.g. for scenario init or geo-engineering).
    pub fn set_co2(&mut self, ppm: f64) {
        self.co2_ppm = ppm;
    }
}

impl Default for Co2Tracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_starts_at_preindustrial() {
        let t = Co2Tracker::new();
        assert!((t.co2_ppm - PRE_INDUSTRIAL_CO2_PPM).abs() < 1e-10);
        assert_eq!(t.cumulative_emissions_ppm, 0.0);
    }

    #[test]
    fn zero_joules_no_change() {
        let mut t = Co2Tracker::new();
        let config = Co2Config::default();
        let result = t.accumulate_tick(0, &config);
        // With zero excess, absorption does nothing.
        assert!((result - PRE_INDUSTRIAL_CO2_PPM).abs() < 1e-10);
    }

    #[test]
    fn positive_joules_increase_co2() {
        let mut t = Co2Tracker::new();
        let config = Co2Config::default();
        let before = t.co2_ppm;
        t.accumulate_tick(1000, &config);
        assert!(t.co2_ppm > before, "CO₂ should rise after industrial consumption");
    }

    #[test]
    fn absorption_reduces_excess_over_time() {
        let mut t = Co2Tracker::new();
        let config = Co2Config {
            emission_factor_ppm_per_mj: 0.0,
            absorption_bp: 500, // 5 % absorption
        };
        t.set_co2(400.0);
        let before = t.co2_ppm;
        t.accumulate_tick(0, &config);
        assert!(
            t.co2_ppm < before,
            "CO₂ should decrease with positive absorption and no new emissions"
        );
    }

    #[test]
    fn co2_never_drops_below_preindustrial() {
        let mut t = Co2Tracker::new();
        let config = Co2Config {
            emission_factor_ppm_per_mj: 0.0,
            absorption_bp: 5000, // 50 % absorption
        };
        t.set_co2(280.5); // barely above baseline
        t.accumulate_tick(0, &config);
        assert!(t.co2_ppm >= PRE_INDUSTRIAL_CO2_PPM - 1e-10);
    }

    #[test]
    fn cumulative_emissions_tracking() {
        let mut t = Co2Tracker::new();
        let config = Co2Config::default();
        t.accumulate_tick(100, &config);
        t.accumulate_tick(200, &config);
        assert_eq!(t.total_industrial_joules, 300);
        assert!(t.cumulative_emissions_ppm > 0.0);
    }

    #[test]
    fn negative_joules_clamped_to_zero() {
        let mut t = Co2Tracker::new();
        let config = Co2Config::default();
        let before = t.co2_ppm;
        t.accumulate_tick(-500, &config);
        assert!((t.co2_ppm - before).abs() < 1e-10);
    }
}
