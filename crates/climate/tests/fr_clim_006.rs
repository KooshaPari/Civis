//! FR-CLIM-006 — Integration test: adaptation investment reduces climate damage.
//!
//! Verifies that MilliCredit investment in adaptation reduces the damage
//! fraction with diminishing returns.

use civ_climate::adaptation::{AdaptationConfig, AdaptationTracker};
use civ_climate::damage::{compute_damage, DamageConfig};

/// FR-CLIM-006: Investment reduces climate damage.
#[test]
fn investment_reduces_damage() {
    let damage_cfg = DamageConfig::default();
    let adapt_cfg = AdaptationConfig::default();

    // Baseline damage without adaptation
    let no_adapt = compute_damage(4.0, &damage_cfg, 0);

    // With adaptation investment
    let mut tracker = AdaptationTracker::new();
    let result = tracker.invest(50_000, &adapt_cfg);
    let with_adapt = compute_damage(4.0, &damage_cfg, result.reduction_bp);

    assert!(
        with_adapt.damage_bp < no_adapt.damage_bp,
        "Adaptation should reduce damage: {} vs {}",
        with_adapt.damage_bp,
        no_adapt.damage_bp
    );
}

/// FR-CLIM-006: More investment provides more reduction (with diminishing returns).
#[test]
fn more_investment_more_reduction() {
    let cfg = AdaptationConfig {
        scale: 5, // Small scale so 10k doesn't max out immediately
        max_reduction_bp: 5_000,
    };
    let mut t = AdaptationTracker::new();

    let r1 = t.invest(10_000, &cfg);
    let r2 = t.invest(10_000, &cfg);
    let r3 = t.invest(10_000, &cfg);

    assert!(r2.reduction_bp > r1.reduction_bp, "Second investment should add reduction");
    assert!(r3.reduction_bp > r2.reduction_bp, "Third investment should add reduction");
}

/// FR-CLIM-006: Diminishing returns — marginal effectiveness decreases.
#[test]
fn diminishing_returns() {
    let cfg = AdaptationConfig::default();
    let mut tracker = AdaptationTracker::new();

    let r1 = tracker.invest(10_000, &cfg);
    let r2 = tracker.invest(10_000, &cfg);

    assert!(
        r2.marginal_effectiveness < r1.marginal_effectiveness,
        "Marginal effectiveness should decrease: first={}, second={}",
        r1.marginal_effectiveness,
        r2.marginal_effectiveness,
    );
}

/// FR-CLIM-006: Adaptation reduction is capped.
#[test]
fn reduction_capped() {
    let cfg = AdaptationConfig::default();
    let mut tracker = AdaptationTracker::new();

    tracker.invest(1_000_000_000, &cfg);
    assert!(
        tracker.reduction_bp <= cfg.max_reduction_bp,
        "Reduction should not exceed maximum: got {} vs max {}",
        tracker.reduction_bp,
        cfg.max_reduction_bp,
    );
}

/// FR-CLIM-006: Combined adaptation + damage pipeline test.
#[test]
fn combined_pipeline() {
    let damage_cfg = DamageConfig::default();
    let adapt_cfg = AdaptationConfig::default();

    // Full pipeline: invest, compute reduction, apply to damage
    let mut tracker = AdaptationTracker::new();
    let adapt_result = tracker.invest(100_000, &adapt_cfg);

    let damage = compute_damage(5.0, &damage_cfg, adapt_result.reduction_bp);
    let no_adapt_damage = compute_damage(5.0, &damage_cfg, 0);

    assert!(
        damage.production_multiplier_bp > no_adapt_damage.production_multiplier_bp,
        "Adaptation should increase production multiplier"
    );
}

/// FR-CLIM-006: Zero investment provides zero reduction.
#[test]
fn zero_investment_zero_reduction() {
    let cfg = AdaptationConfig::default();
    let mut tracker = AdaptationTracker::new();

    let result = tracker.invest(0, &cfg);
    assert_eq!(result.reduction_bp, 0);
}
