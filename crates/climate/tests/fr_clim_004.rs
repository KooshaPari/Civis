//! FR-CLIM-004 — Integration test: climate damage reduces district Joule production.
//!
//! Verifies that production capacity is reduced when temperature exceeds the damage threshold.

use civ_climate::damage::{
    apply_damage_to_production, compute_damage, DamageConfig, DamageTracker,
};

/// FR-CLIM-004: Climate damage reduces production above threshold.
#[test]
fn reduces_production_above_threshold() {
    let config = DamageConfig::default();
    let damage = compute_damage(4.0, &config, 0);

    // 2 °C above threshold × 500 bp/°C = 1000 bp damage
    assert!(damage.active);
    assert_eq!(damage.damage_bp, 1000);
    assert_eq!(damage.production_multiplier_bp, 9000);

    let base = 10_000;
    let effective = apply_damage_to_production(base, &damage);
    assert_eq!(effective, 9000, "Production should be reduced by 10%");
}

/// FR-CLIM-004: No damage below threshold.
#[test]
fn no_damage_below_threshold() {
    let config = DamageConfig::default();
    let damage = compute_damage(1.0, &config, 0);

    assert!(!damage.active);
    assert_eq!(damage.damage_bp, 0);
    assert_eq!(damage.production_multiplier_bp, 10_000);

    let base = 5000;
    let effective = apply_damage_to_production(base, &damage);
    assert_eq!(effective, 5000, "No damage below threshold");
}

/// FR-CLIM-004: Damage scales linearly with temperature above threshold.
#[test]
fn damage_scales_with_anomaly() {
    let config = DamageConfig::default();
    let d1 = compute_damage(3.0, &config, 0);
    let d2 = compute_damage(4.0, &config, 0);
    let d3 = compute_damage(5.0, &config, 0);

    assert!(d3.damage_bp > d2.damage_bp);
    assert!(d2.damage_bp > d1.damage_bp);
}

/// FR-CLIM-004: Damage is capped at maximum.
#[test]
fn damage_capped() {
    let config = DamageConfig::default();
    let damage = compute_damage(100.0, &config, 0);

    assert_eq!(damage.damage_bp, config.max_damage_bp);
    assert!(damage.production_multiplier_bp >= 1000, "At least 10% capacity remains");
}

/// FR-CLIM-004: Tracker accumulates damage over ticks.
#[test]
fn tracker_accumulates() {
    let config = DamageConfig::default();
    let mut tracker = DamageTracker::new();

    tracker.update(1.0, &config, 0);
    assert!(!tracker.current.active);

    tracker.update(3.0, &config, 0);
    assert!(tracker.current.active);
    assert_eq!(tracker.active_ticks, 1);
    assert!(tracker.cumulative_damage_bp > 0);
}

/// FR-CLIM-004: Large base production produces proportionally large loss.
#[test]
fn large_production_scales_loss() {
    let config = DamageConfig::default();
    let damage = compute_damage(5.0, &config, 0);

    let small = apply_damage_to_production(1000, &damage);
    let large = apply_damage_to_production(10_000, &damage);

    // Both should be reduced by the same percentage.
    assert_eq!(small * 10, large, "Loss should scale proportionally");
}
