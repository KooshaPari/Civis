//! FR-CIV-SEASON-CYCLE: deterministic additive seasonal climate oscillation.

/// Tunable additive seasonal oscillation over one simulated year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeasonCycleParams {
    /// Number of ticks in one simulated year.
    pub year_length_ticks: u64,
    /// Peak additive temperature deviation in degrees Celsius.
    pub temperature_amplitude_c: f64,
    /// Peak additive precipitation deviation in model units.
    pub precipitation_amplitude: f64,
}

impl Default for SeasonCycleParams {
    fn default() -> Self {
        Self {
            year_length_ticks: 365,
            temperature_amplitude_c: 10.0,
            precipitation_amplitude: 0.25,
        }
    }
}

/// Additive seasonal climate offsets for a simulation tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeasonCycleSample {
    /// Add this value to baseline or simulated temperature.
    pub temperature_delta_c: f64,
    /// Add this value to baseline or simulated precipitation.
    pub precipitation_delta: f64,
}

/// Return additive temperature and precipitation offsets for `tick`.
///
/// The cycle trough is at the start/end of the year and peaks halfway through
/// the period. `year_length_ticks == 0` is treated as a disabled cycle.
pub fn seasonal_cycle(tick: u64, params: SeasonCycleParams) -> SeasonCycleSample {
    if params.year_length_ticks == 0 {
        return SeasonCycleSample {
            temperature_delta_c: 0.0,
            precipitation_delta: 0.0,
        };
    }

    let phase =
        (tick % params.year_length_ticks) as f64 / params.year_length_ticks as f64;
    let wave = -(std::f64::consts::TAU * phase).cos();

    SeasonCycleSample {
        temperature_delta_c: params.temperature_amplitude_c * wave,
        precipitation_delta: params.precipitation_amplitude * wave,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_peaks_mid_cycle_and_troughs_at_cycle_ends() {
        let params = SeasonCycleParams {
            year_length_ticks: 100,
            temperature_amplitude_c: 12.0,
            precipitation_amplitude: 0.5,
        };

        let start = seasonal_cycle(0, params);
        let middle = seasonal_cycle(50, params);
        let end = seasonal_cycle(100, params);

        assert!(
            middle.temperature_delta_c > start.temperature_delta_c,
            "temperature should peak above start-of-cycle trough"
        );
        assert!(
            middle.temperature_delta_c > end.temperature_delta_c,
            "temperature should peak above end-of-cycle trough"
        );
        approx_eq(start.temperature_delta_c, -12.0);
        approx_eq(middle.temperature_delta_c, 12.0);
        approx_eq(end.temperature_delta_c, start.temperature_delta_c);
    }

    fn approx_eq(a: f64, b: f64) {
        let diff = (a - b).abs();
        assert!(diff < 1e-10, "expected {b}, got {a} (diff = {diff})");
    }
}
