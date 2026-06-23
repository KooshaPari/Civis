/// A simplified climate model simulating CO₂ concentration, mean temperature,
/// and sea-level response to anthropogenic emissions.
///
/// # Design
///
/// The model uses a single-box energy-balance approach:
///
/// - **Direct forcing**: each `step()` converts `emissions_ppm` into a
///   temperature increment via `ClimateParams::co2_sensitivity`.
/// - **Thermal inertia** (`thermal_inertia`) damps the instantaneous forcing,
///   representing ocean heat uptake.
/// - **Feedback** (`feedback_factor`) amplifies the temperature change
///   (ice-albedo, water-vapour, cloud feedbacks).
/// - **Sea level** rises linearly with the temperature anomaly relative to a
///   14 °C pre-industrial baseline.
///
/// # Example
///
/// ```rust
/// use civ_climate::{ClimateParams, ClimateState};
///
/// let mut state = ClimateState::new();
/// let params = ClimateParams::default();
/// state.step(10.0, &params);
/// assert!(state.mean_temp_c > 14.0);
/// ```

/// Tunable parameters that characterise the climate system's response.
#[derive(Debug, Clone)]
pub struct ClimateParams {
    /// Temperature increase (°C) per ppm of CO₂ emitted in a single step.
    pub co2_sensitivity: f64,
    /// Fraction of the forcing that is absorbed by the system (ocean / ice)
    /// and thus *delayed* from appearing as surface warming in the current
    /// tick. 0.0 = no inertia (full forcing instant), 1.0 = infinite inertia.
    pub thermal_inertia: f64,
    /// Positive-feedback multiplier applied *after* inertia damping.
    /// 0.3 = 30% amplification of the damped forcing.
    pub feedback_factor: f64,
}

impl Default for ClimateParams {
    fn default() -> Self {
        Self {
            co2_sensitivity: 0.008,
            thermal_inertia: 0.95,
            feedback_factor: 0.3,
        }
    }
}

/// The observable state of the climate at a given simulation tick.
#[derive(Debug, Clone)]
pub struct ClimateState {
    /// Atmospheric CO₂ concentration (parts per million).
    pub co2_ppm: f64,
    /// Global mean surface temperature (°C).
    pub mean_temp_c: f64,
    /// Global mean sea level (metres) relative to a baseline datum.
    pub sea_level_m: f64,
    /// Number of simulation steps elapsed.
    pub tick: u64,
}

/// Pre-industrial baseline temperature used for anomaly calculations.
const BASELINE_TEMP_C: f64 = 14.0;

/// Sea-level sensitivity (metres of rise per °C of anomaly per tick).
const SEA_LEVEL_SENSITIVITY_M_PER_C: f64 = 0.002;

impl ClimateState {
    /// Create a new `ClimateState` initialised to pre-industrial conditions.
    pub fn new() -> Self {
        Self {
            co2_ppm: 280.0,         // pre-industrial ≈ 280 ppm
            mean_temp_c: BASELINE_TEMP_C,
            sea_level_m: 0.0,       // baseline datum
            tick: 0,
        }
    }

    /// Advance the climate state by one tick, given `emissions_ppm` of CO₂
    /// released during this tick and the system `params`.
    pub fn step(&mut self, emissions_ppm: f64, params: &ClimateParams) {
        // Accumulate CO₂
        self.co2_ppm += emissions_ppm;

        // Direct temperature forcing
        let mut temp_change = emissions_ppm * params.co2_sensitivity;

        // Thermal inertia damps the instantaneous response
        temp_change *= 1.0 - params.thermal_inertia;

        // Positive feedback amplifies the damped response
        temp_change *= 1.0 + params.feedback_factor;

        self.mean_temp_c += temp_change;

        // Sea level rises proportionally to the temperature anomaly
        let anomaly = self.temp_anomaly_c();
        self.sea_level_m += anomaly.abs() * SEA_LEVEL_SENSITIVITY_M_PER_C;

        self.tick += 1;
    }

    /// Overwrite the CO₂ concentration directly (e.g. for scenario
    /// initialisation or geo-engineering interventions).
    pub fn set_co2(&mut self, ppm: f64) {
        self.co2_ppm = ppm;
    }

    /// Temperature anomaly (°C) relative to the pre-industrial baseline
    /// (14 °C).  Positive values indicate warming.
    pub fn temp_anomaly_c(&self) -> f64 {
        self.mean_temp_c - BASELINE_TEMP_C
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// 1.  `ClimateState::new()` returns sensible pre-industrial defaults.
    #[test]
    fn new_creates_defaults() {
        let s = ClimateState::new();
        approx_eq(s.co2_ppm, 280.0);
        approx_eq(s.mean_temp_c, 14.0);
        approx_eq(s.sea_level_m, 0.0);
        assert_eq!(s.tick, 0);
    }

    /// 2.  Stepping with zero emissions does not change temperature.
    #[test]
    fn step_zero_emissions() {
        let mut s = ClimateState::new();
        let p = ClimateParams::default();
        s.step(0.0, &p);
        approx_eq(s.mean_temp_c, 14.0);
        assert_eq!(s.tick, 1);
    }

    /// 3.  Stepping with positive emissions raises the mean temperature.
    #[test]
    fn step_with_emissions_raises_temp() {
        let mut s = ClimateState::new();
        let p = ClimateParams::default();
        let before = s.mean_temp_c;
        s.step(100.0, &p);
        assert!(
            s.mean_temp_c > before,
            "temperature should increase after positive emissions"
        );
    }

    /// 4.  `set_co2` directly overwrites the CO₂ value.
    #[test]
    fn set_co2_changes_state() {
        let mut s = ClimateState::new();
        s.set_co2(415.0);
        approx_eq(s.co2_ppm, 415.0);
        // other fields unchanged
        approx_eq(s.mean_temp_c, 14.0);
    }

    /// 5.  Sea level rises when temperature anomaly is positive.
    #[test]
    fn sea_level_rises_with_temp() {
        let mut s = ClimateState::new();
        let p = ClimateParams::default();

        let sea_before = s.sea_level_m;
        // Large emission pulse to produce a measurable sea-level response
        s.step(500.0, &p);

        assert!(
            s.sea_level_m > sea_before,
            "sea level should rise after warming: {:.6} -> {:.6}",
            sea_before,
            s.sea_level_m
        );
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Assert two `f64` values are equal to within 1e-10.
    fn approx_eq(a: f64, b: f64) {
        let diff = (a - b).abs();
        assert!(
            diff < 1e-10,
            "expected {b}, got {a} (diff = {diff})"
        );
    }
}
