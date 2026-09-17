//! FR-CLIM-002 — Integration test: global mean temperature derived from CO₂ concentration.
//!
//! Verifies the logarithmic forcing formula produces physically plausible temperatures.

use civ_climate::co2::PRE_INDUSTRIAL_CO2_PPM;
use civ_climate::temperature::{temperature_anomaly_from_co2, temperature_from_co2, TemperatureConfig, TemperatureTracker, BASELINE_TEMP_C};

/// FR-CLIM-002: Temperature is derived from CO₂ via parameterised formula.
#[test]
fn temperature_derived_from_co2() {
    let config = TemperatureConfig::default();

    // Pre-industrial: 280 ppm → 14 °C, 0 anomaly
    let t = temperature_from_co2(PRE_INDUSTRIAL_CO2_PPM, &config);
    assert!((t - BASELINE_TEMP_C).abs() < 1e-10);
    assert!((temperature_anomaly_from_co2(PRE_INDUSTRIAL_CO2_PPM, &config)).abs() < 1e-10);
}

/// FR-CLIM-002: Doubling CO₂ gives the configured ECS.
#[test]
fn doubling_gives_ecs() {
    let config = TemperatureConfig { ecs_c: 3.0 };
    let doubled = PRE_INDUSTRIAL_CO2_PPM * 2.0;
    let anomaly = temperature_anomaly_from_co2(doubled, &config);
    assert!(
        (anomaly - 3.0).abs() < 1e-10,
        "Doubling CO₂ should produce anomaly equal to ECS: got {anomaly}"
    );
}

/// FR-CLIM-002: Temperature increases monotonically with CO₂.
#[test]
fn monotonic_increase() {
    let config = TemperatureConfig::default();
    let co2_levels = [300.0, 350.0, 400.0, 500.0, 600.0, 800.0];
    let mut prev_t = 0.0;
    for &co2 in &co2_levels {
        let t = temperature_from_co2(co2, &config);
        assert!(t > prev_t, "Temperature should increase: {co2} ppm → {t} °C");
        prev_t = t;
    }
}

/// FR-CLIM-002: Different ECS values scale the anomaly linearly.
#[test]
fn ecs_scales_anomaly() {
    let co2 = 560.0; // exactly 2x pre-industrial
    let a_2 = temperature_anomaly_from_co2(co2, &TemperatureConfig { ecs_c: 2.0 });
    let a_4 = temperature_anomaly_from_co2(co2, &TemperatureConfig { ecs_c: 4.0 });
    assert!((a_4 / a_2 - 2.0).abs() < 1e-10);
}

/// FR-CLIM-002: TemperatureTracker updates correctly.
#[test]
fn tracker_updates() {
    let mut tracker = TemperatureTracker::new();
    let config = TemperatureConfig::default();

    assert!((tracker.mean_temp_c - BASELINE_TEMP_C).abs() < 1e-10);

    tracker.update_from_co2(450.0, &config);
    assert!(tracker.mean_temp_c > BASELINE_TEMP_C);
    assert!((tracker.mean_temp_c - (BASELINE_TEMP_C + tracker.anomaly_c)).abs() < 1e-10);
}

/// FR-CLIM-002: Below-preindustrial CO₂ gives baseline temperature.
#[test]
fn below_preindustrial_gives_baseline() {
    let config = TemperatureConfig::default();
    let t = temperature_from_co2(200.0, &config);
    assert!((t - BASELINE_TEMP_C).abs() < 1e-10);
}
