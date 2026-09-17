//! FR-SOCI-002 — Citizen stress accumulation.
//!
//! Citizen stress SHALL accumulate when Joule access falls below
//! subsistence level.

use civ_social::{StressAccumulator, StressConfig};

#[test]
fn accumulates_below_subsistence() {
    let mut acc = StressAccumulator::new();
    let cfg = StressConfig::default(); // subsistence = 500 J, increment = 50 bp

    // Below subsistence: stress goes up.
    acc.tick(100, &cfg);
    assert_eq!(acc.stress_bp, 50);

    acc.tick(200, &cfg);
    assert_eq!(acc.stress_bp, 100);
}

#[test]
fn decays_above_subsistence() {
    let mut acc = StressAccumulator { stress_bp: 200 };
    let cfg = StressConfig::default(); // decay = 25 bp

    // Above subsistence: stress goes down.
    acc.tick(600, &cfg);
    assert_eq!(acc.stress_bp, 175);
}

#[test]
fn stress_clamped_to_max() {
    let mut acc = StressAccumulator { stress_bp: 990 };
    let cfg = StressConfig::default();
    acc.tick(0, &cfg);
    assert_eq!(acc.stress_bp, 1_000);
    assert!(acc.is_maxed());
}

#[test]
fn stress_clamped_to_zero() {
    let mut acc = StressAccumulator { stress_bp: 10 };
    let cfg = StressConfig::default();
    acc.tick(1_000, &cfg);
    assert_eq!(acc.stress_bp, 0);
    assert!(!acc.is_stressed());
}

#[test]
fn custom_subsistence_threshold() {
    let mut acc = StressAccumulator::new();
    let cfg = StressConfig {
        subsistence_joules: 200,
        increment_bp: 100,
        decay_bp: 50,
    };

    // 150 < 200 => stress up
    acc.tick(150, &cfg);
    assert_eq!(acc.stress_bp, 100);

    // 250 >= 200 => decay
    acc.tick(250, &cfg);
    assert_eq!(acc.stress_bp, 50);
}

#[test]
fn stress_never_negative() {
    let mut acc = StressAccumulator { stress_bp: 0 };
    let cfg = StressConfig::default();
    acc.tick(10_000, &cfg);
    assert_eq!(acc.stress_bp, 0);
}
