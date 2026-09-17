//! FR-CLIM-004 — Climate damage reduces district Joule production capacity.
//!
//! When the global mean temperature exceeds a damage threshold, a fraction
//! of district production capacity is lost. The damage fraction increases
//! with temperature anomaly and can be reduced by adaptation investment
//! (see `adaptation.rs`).
//!
//! Damage is expressed as a production multiplier in basis points:
//!   - 10 000 bp = no damage (100 % capacity)
//!   - 0 bp = total loss (0 % capacity)

use serde::{Deserialize, Serialize};

/// Basis-point denominator (10 000 bp = 100 %).
const BP_DENOM: i64 = 10_000;

/// Default damage threshold (°C anomaly above which damage begins).
pub const DEFAULT_DAMAGE_THRESHOLD_C: f64 = 2.0;

/// Damage scaling factor in basis points per °C above threshold.
/// At 500 bp/°C, 2 °C above threshold → 1000 bp (10 %) damage.
pub const DEFAULT_DAMAGE_SCALE_BP_PER_C: i64 = 500;

/// Maximum damage fraction in basis points (cap at 90 % to allow recovery).
pub const MAX_DAMAGE_BP: i64 = 9_000;

/// Configuration for the climate damage model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageConfig {
    /// Temperature anomaly (°C) above which damage begins.
    pub threshold_c: f64,
    /// Damage scaling in basis points per °C above threshold.
    pub scale_bp_per_c: i64,
    /// Maximum damage cap in basis points.
    pub max_damage_bp: i64,
}

impl Default for DamageConfig {
    fn default() -> Self {
        Self {
            threshold_c: DEFAULT_DAMAGE_THRESHOLD_C,
            scale_bp_per_c: DEFAULT_DAMAGE_SCALE_BP_PER_C,
            max_damage_bp: MAX_DAMAGE_BP,
        }
    }
}

/// Result of a damage computation for one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageResult {
    /// Production multiplier in basis points (10 000 = full capacity).
    pub production_multiplier_bp: i64,
    /// Damage fraction in basis points (10 000 = total loss).
    pub damage_bp: i64,
    /// Whether damage is active (anomaly exceeds threshold).
    pub active: bool,
}

/// Compute climate damage from temperature anomaly.
///
/// `damage_bp = min(max_damage, scale * max(0, anomaly - threshold))`
/// `multiplier_bp = 10_000 - damage_bp`
///
/// The `adaptation_reduction_bp` parameter (from FR-CLIM-006) reduces
/// the raw damage before capping.
#[must_use]
pub fn compute_damage(
    anomaly_c: f64,
    config: &DamageConfig,
    adaptation_reduction_bp: i64,
) -> DamageResult {
    if anomaly_c <= config.threshold_c {
        return DamageResult {
            production_multiplier_bp: BP_DENOM,
            damage_bp: 0,
            active: false,
        };
    }

    let excess = anomaly_c - config.threshold_c;
    let raw_damage = (excess * config.scale_bp_per_c as f64) as i64;
    let reduced_damage = (raw_damage - adaptation_reduction_bp).max(0);
    let capped_damage = reduced_damage.min(config.max_damage_bp);

    DamageResult {
        production_multiplier_bp: BP_DENOM - capped_damage,
        damage_bp: capped_damage,
        active: true,
    }
}

/// Apply climate damage to a district's Joule production capacity.
///
/// Returns the effective production after damage reduction.
#[must_use]
pub fn apply_damage_to_production(
    base_production_joules: i64,
    damage: &DamageResult,
) -> i64 {
    (base_production_joules * damage.production_multiplier_bp / BP_DENOM).max(0)
}

/// Tracks accumulated damage state across ticks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageTracker {
    /// Current damage result.
    pub current: DamageResult,
    /// Cumulative damage across all ticks (sum of damage_bp values).
    pub cumulative_damage_bp: i64,
    /// Number of ticks damage has been active.
    pub active_ticks: u64,
}

impl DamageTracker {
    pub fn new() -> Self {
        Self {
            current: DamageResult {
                production_multiplier_bp: BP_DENOM,
                damage_bp: 0,
                active: false,
            },
            cumulative_damage_bp: 0,
            active_ticks: 0,
        }
    }

    /// Update damage based on current anomaly and adaptation reduction.
    pub fn update(
        &mut self,
        anomaly_c: f64,
        config: &DamageConfig,
        adaptation_reduction_bp: i64,
    ) -> DamageResult {
        self.current = compute_damage(anomaly_c, config, adaptation_reduction_bp);
        self.cumulative_damage_bp += self.current.damage_bp;
        if self.current.active {
            self.active_ticks += 1;
        }
        self.current
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_damage_below_threshold() {
        let cfg = DamageConfig::default();
        let result = compute_damage(1.0, &cfg, 0);
        assert_eq!(result.damage_bp, 0);
        assert_eq!(result.production_multiplier_bp, BP_DENOM);
        assert!(!result.active);
    }

    #[test]
    fn damage_at_threshold() {
        let cfg = DamageConfig::default();
        let result = compute_damage(2.0, &cfg, 0);
        assert!(!result.active, "exactly at threshold should not trigger damage");
    }

    #[test]
    fn damage_above_threshold() {
        let cfg = DamageConfig::default();
        let result = compute_damage(3.0, &cfg, 0);
        // 1 °C above threshold × 500 bp/°C = 500 bp damage
        assert_eq!(result.damage_bp, 500);
        assert_eq!(result.production_multiplier_bp, 9500);
        assert!(result.active);
    }

    #[test]
    fn damage_capped_at_max() {
        let cfg = DamageConfig::default();
        let result = compute_damage(25.0, &cfg, 0);
        assert_eq!(result.damage_bp, MAX_DAMAGE_BP);
    }

    #[test]
    fn adaptation_reduces_damage() {
        let cfg = DamageConfig::default();
        let no_adapt = compute_damage(3.0, &cfg, 0);
        let with_adapt = compute_damage(3.0, &cfg, 300);
        assert!(with_adapt.damage_bp < no_adapt.damage_bp);
        assert_eq!(no_adapt.damage_bp - with_adapt.damage_bp, 300);
    }

    #[test]
    fn adaptation_cannot_make_negative_damage() {
        let cfg = DamageConfig::default();
        let result = compute_damage(2.5, &cfg, 9999);
        assert_eq!(result.damage_bp, 0);
    }

    #[test]
    fn apply_damage_reduces_production() {
        let damage = DamageResult {
            production_multiplier_bp: 8000, // 80 % capacity
            damage_bp: 2000,
            active: true,
        };
        let effective = apply_damage_to_production(1000, &damage);
        assert_eq!(effective, 800);
    }

    #[test]
    fn tracker_accumulates() {
        let cfg = DamageConfig::default();
        let mut tracker = DamageTracker::new();
        tracker.update(1.0, &cfg, 0);
        assert!(!tracker.current.active);
        tracker.update(3.0, &cfg, 0);
        assert!(tracker.current.active);
        assert_eq!(tracker.active_ticks, 1);
        assert!(tracker.cumulative_damage_bp > 0);
    }
}
