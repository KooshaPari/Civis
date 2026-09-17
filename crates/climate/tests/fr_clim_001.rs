//! FR-CLIM-001 — Integration test: atmospheric CO₂ accumulation from industrial Joule consumption.
//!
//! Verifies the full pipeline: industrial joules → CO₂ ppm accumulation → tracking.

use civ_climate::co2::{Co2Config, Co2Tracker, PRE_INDUSTRIAL_CO2_PPM};

/// FR-CLIM-001: CO₂ accumulates each tick based on industrial Joule consumption.
#[test]
fn co2_accumulates_with_consumption() {
    let mut tracker = Co2Tracker::new();
    let config = Co2Config::default();

    let co2_before = tracker.co2_ppm;

    // Simulate 100 ticks of industrial consumption at 500 MJ per tick.
    for _ in 0..100 {
        tracker.accumulate_tick(500, &config);
    }

    assert!(
        tracker.co2_ppm > co2_before,
        "CO₂ should increase after sustained industrial consumption"
    );
    assert!(
        tracker.co2_ppm > PRE_INDUSTRIAL_CO2_PPM,
        "CO₂ should exceed pre-industrial baseline"
    );
}

/// FR-CLIM-001: Higher industrial consumption produces more CO₂.
#[test]
fn higher_consumption_more_co2() {
    let config = Co2Config::default();

    let mut low = Co2Tracker::new();
    let mut high = Co2Tracker::new();

    for _ in 0..50 {
        low.accumulate_tick(100, &config);
        high.accumulate_tick(1000, &config);
    }

    assert!(
        high.co2_ppm > low.co2_ppm,
        "Higher consumption should produce higher CO₂"
    );
}

/// FR-CLIM-001: Zero consumption does not increase CO₂ beyond absorption.
#[test]
fn zero_consumption_no_accumulation() {
    let mut tracker = Co2Tracker::new();
    let config = Co2Config::default();

    let co2_before = tracker.co2_ppm;
    for _ in 0..10 {
        tracker.accumulate_tick(0, &config);
    }

    // With default config, zero excess means absorption doesn't change anything.
    assert!((tracker.co2_ppm - co2_before).abs() < 1e-10);
}

/// FR-CLIM-001: Cumulative tracking is correct.
#[test]
fn cumulative_tracking_is_accurate() {
    let mut tracker = Co2Tracker::new();
    let config = Co2Config::default();

    tracker.accumulate_tick(100, &config);
    tracker.accumulate_tick(200, &config);
    tracker.accumulate_tick(300, &config);

    assert_eq!(tracker.total_industrial_joules, 600);
    assert!(tracker.cumulative_emissions_ppm > 0.0);
}

/// FR-CLIM-001: CO₂ never drops below pre-industrial even with strong absorption.
#[test]
fn co2_floor_at_preindustrial() {
    let mut tracker = Co2Tracker::new();
    let config = Co2Config {
        emission_factor_ppm_per_mj: 0.0,
        absorption_bp: 9000, // 90 % absorption — very aggressive
    };

    for _ in 0..100 {
        tracker.accumulate_tick(0, &config);
    }

    assert!(
        tracker.co2_ppm >= PRE_INDUSTRIAL_CO2_PPM - 1e-10,
        "CO₂ must never drop below pre-industrial: got {}",
        tracker.co2_ppm
    );
}
