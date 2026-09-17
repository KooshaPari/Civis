//! FR-DIPL-005 tests — Espionage detection probability.
//!
//! Espionage operations SHALL have a configurable detection probability per tick.

use civ_diplomacy::{EspionageAction, EspionageConfig, EspionageEngine};

/// FR-DIPL-005: detection_probability_applied — probability is computed from config.
#[test]
fn espionage::detection_probability_applied() {
    let config = EspionageConfig {
        base_detection_chance: 0.20,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    eng.deploy(1, 2, 0.5).expect("deploy");

    // Cover is 1.0 by default, so (1.0 - 1.0) = 0.0 -> detection = 0.0
    let prob = eng
        .detection_probability(EspionageAction::GatherIntel, 0)
        .expect("valid network");
    assert!(prob.abs() < f32::EPSILON, "full cover = zero detection");
}

/// FR-DIPL-005: Detection probability increases as cover decreases.
#[test]
fn espionage::detection_scales_with_cover() {
    let config = EspionageConfig {
        base_detection_chance: 0.50,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    eng.deploy(1, 2, 0.5).expect("deploy");

    // Full cover -> zero detection
    let prob_full = eng
        .detection_probability(EspionageAction::GatherIntel, 0)
        .unwrap();
    assert!(prob_full.abs() < f32::EPSILON);

    // Reduce cover -> detection increases
    eng.networks[0].cover = 0.5;
    // detection = 0.50 * (1.0 - 0.5) * 0.3 = 0.075
    let prob_half = eng
        .detection_probability(EspionageAction::GatherIntel, 0)
        .unwrap();
    assert!(prob_half > 0.0, "reduced cover increases detection");
    assert!((prob_half - 0.075).abs() < 1e-6);
}

/// FR-DIPL-005: Detection probability varies by action risk factor.
#[test]
fn espionage::detection_scales_with_risk() {
    let config = EspionageConfig {
        base_detection_chance: 0.50,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    eng.deploy(1, 2, 0.5).expect("deploy");
    eng.networks[0].cover = 0.0; // no cover

    // GatherIntel risk=0.3 => 0.5 * 1.0 * 0.3 = 0.15
    let prob_intel = eng
        .detection_probability(EspionageAction::GatherIntel, 0)
        .unwrap();
    // AssassinateLeader risk=1.0 => 0.5 * 1.0 * 1.0 = 0.5
    let prob_assassinate = eng
        .detection_probability(EspionageAction::AssassinateLeader, 0)
        .unwrap();

    assert!(prob_assassinate > prob_intel, "high risk = higher detection");
    assert!((prob_intel - 0.15).abs() < 1e-6);
    assert!((prob_assassinate - 0.50).abs() < 1e-6);
}

/// FR-DIPL-005: Detection probability is clamped to [0.0, 1.0].
#[test]
fn espionage::detection_probability_clamped() {
    let config = EspionageConfig {
        base_detection_chance: 1.0,
        cover_decay: 0.0,
        strength_growth: 0.0,
        ..Default::default()
    };
    let mut eng = EspionageEngine::new(config).expect("valid config");
    eng.deploy(1, 2, 0.5).expect("deploy");
    eng.networks[0].cover = 0.0;

    let prob = eng
        .detection_probability(EspionageAction::AssassinateLeader, 0)
        .unwrap();
    assert!(prob <= 1.0, "probability clamped at 1.0");
    assert!(prob >= 0.0, "probability clamped at 0.0");
}

/// FR-DIPL-005: Configurable base_detection_chance affects all operations.
#[test]
fn espionage::configurable_detection_chance() {
    let mut eng_low = EspionageEngine::new(EspionageConfig {
        base_detection_chance: 0.10,
        ..Default::default()
    })
    .expect("valid");
    eng_low.deploy(1, 2, 0.5).unwrap();
    eng_low.networks[0].cover = 0.0;

    let mut eng_high = EspionageEngine::new(EspionageConfig {
        base_detection_chance: 0.90,
        ..Default::default()
    })
    .expect("valid");
    eng_high.deploy(1, 2, 0.5).unwrap();
    eng_high.networks[0].cover = 0.0;

    let prob_low = eng_low
        .detection_probability(EspionageAction::GatherIntel, 0)
        .unwrap();
    let prob_high = eng_high
        .detection_probability(EspionageAction::GatherIntel, 0)
        .unwrap();
    assert!(prob_high > prob_low, "higher config = higher detection");
}
