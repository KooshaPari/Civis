//! FR-SOCI-005 — Health index computation.
//!
//! Health index SHALL be computed from food Joules, clean water, and
//! medical infrastructure.

use civ_social::{compute_health, HealthConfig, HealthInputs, MAX_HEALTH_BP};

#[test]
fn computed_from_inputs() {
    let inputs = HealthInputs {
        food_bp: 1000,
        water_bp: 1000,
        medical_bp: 1000,
    };
    let cfg = HealthConfig::default();
    let result = compute_health(inputs, &cfg, false, 1);

    // All inputs at max => index at max
    assert_eq!(result.index_bp, MAX_HEALTH_BP);
    assert!(!result.crisis);
    assert_eq!(result.labor_factor_bp, MAX_HEALTH_BP);
}

#[test]
fn weighted_average() {
    // food=1000 (w=40), water=0 (w=30), medical=0 (w=30)
    // index = (1000*40 + 0*30 + 0*30) / 100 = 400
    let inputs = HealthInputs {
        food_bp: 1000,
        water_bp: 0,
        medical_bp: 0,
    };
    let cfg = HealthConfig::default();
    let result = compute_health(inputs, &cfg, false, 1);

    assert_eq!(result.index_bp, 400);
}

#[test]
fn zero_inputs_zero_health() {
    let inputs = HealthInputs {
        food_bp: 0,
        water_bp: 0,
        medical_bp: 0,
    };
    let cfg = HealthConfig::default();
    let result = compute_health(inputs, &cfg, false, 1);

    assert_eq!(result.index_bp, 0);
    assert!(result.crisis);
    assert_eq!(result.labor_factor_bp, cfg.crisis_labor_factor_bp);
}

#[test]
fn inputs_clamped_to_valid_range() {
    let inputs = HealthInputs {
        food_bp: 5000,
        water_bp: -100,
        medical_bp: 200,
    };
    let cfg = HealthConfig::default();
    let result = compute_health(inputs, &cfg, false, 1);

    // food clamped to 1000, water clamped to 0, medical = 200
    // index = (1000*40 + 0*30 + 200*30) / 100 = (40000 + 0 + 6000) / 100 = 460
    assert_eq!(result.index_bp, 460);
}

#[test]
fn crisis_detected_when_below_threshold() {
    let inputs = HealthInputs {
        food_bp: 200,
        water_bp: 200,
        medical_bp: 200,
    };
    let cfg = HealthConfig::default();
    // index = (200*40 + 200*30 + 200*30) / 100 = 20000/100 = 200
    // threshold = 300 => crisis
    let result = compute_health(inputs, &cfg, false, 10);

    assert_eq!(result.index_bp, 200);
    assert!(result.crisis);
    assert_eq!(result.labor_factor_bp, cfg.crisis_labor_factor_bp);
}

#[test]
fn no_crisis_when_above_threshold() {
    let inputs = HealthInputs {
        food_bp: 1000,
        water_bp: 500,
        medical_bp: 500,
    };
    let cfg = HealthConfig::default();
    // index = (1000*40 + 500*30 + 500*30) / 100 = 70000/100 = 700
    let result = compute_health(inputs, &cfg, false, 1);

    assert_eq!(result.index_bp, 700);
    assert!(!result.crisis);
    assert_eq!(result.labor_factor_bp, MAX_HEALTH_BP);
}
