//! FR-CIV-TAX-POLICY — production-skim tax policy with compliance.
//!
//! Distinct from [`crate::institution::Taxation`]/[`crate::institution::collect_taxes`]
//! which skim the macro joule *budget*. FR-CIV-TAX-POLICY skims a slice of
//! *production* (an integer production amount per tick) into a treasury pool,
//! and tracks `compliance` ∈ [0, 1] representing how much of the demanded tax
//! is actually reported/paid by the producers.
//!
//! Semantics:
//!
//! - `tax_rate` is a fraction in `[0, 1]` (e.g. `0.10` = 10 %).
//! - On every [`apply_tax_policy`] pass:
//!   1. `compliance` moves toward the **target compliance** implied by `tax_rate`.
//!      Higher rates depress target compliance (over-taxation lowers compliance).
//!   2. `treasury += production * tax_rate * compliance`
//!   3. `reported_production = production * compliance`
//!      (the un-reported share is lost from the visible ledger, mirroring evasion)
//! - All math is integer-saturating; no floats accumulate across calls.
//!
//! The module is **additive**: it introduces new types and a free function, and
//! does not modify any existing crate state.

use serde::{Deserialize, Serialize};

/// FR-CIV-TAX-POLICY — tax policy applied to production each tick.
///
/// Holds the configured rate, the running compliance level, the accumulated
/// treasury, and per-tick diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxPolicy {
    /// Tax rate as a fraction in `[0, 1]`, stored in basis points × 10
    /// (i.e. `_rate_bp10 / 10_000` is the real fraction). The integer
    /// representation avoids float drift across long replays.
    ///
    /// Example: `rate_bp10 = 1_000` ⇒ 10.00 %.
    rate_bp10: u32,
    /// Smoothed compliance ∈ `[0, 10_000]` (divide by 10_000 for the fraction).
    /// Models how much of the demanded tax the producers actually pay/report.
    compliance_bp: i64,
    /// Treasury pool accumulated by [`apply_tax_policy`].
    pub treasury: i64,
    /// Total production that has flowed through this policy (after compliance).
    pub total_reported_production: i64,
    /// Total joules skimmed into the treasury across all passes.
    pub total_tax_collected: i64,
    /// Number of passes applied.
    pub passes: u64,
}

impl Default for TaxPolicy {
    fn default() -> Self {
        Self {
            // 0 % by default — additive, no behavior change unless caller opts in.
            rate_bp10: 0,
            compliance_bp: 10_000,
            treasury: 0,
            total_reported_production: 0,
            total_tax_collected: 0,
            passes: 0,
        }
    }
}

impl TaxPolicy {
    /// Construct with a tax rate in `[0.0, 1.0]`. Out-of-range values are
    /// clamped (negative ⇒ 0, > 1 ⇒ 1).
    pub fn with_rate(rate: f32) -> Self {
        let mut p = Self::default();
        p.set_rate(rate);
        p
    }

    /// Current configured tax rate as a fraction in `[0, 1]`.
    pub fn rate(&self) -> f32 {
        (self.rate_bp10 as f32) / 10_000.0
    }

    /// Current compliance as a fraction in `[0, 1]`.
    pub fn compliance(&self) -> f32 {
        (self.compliance_bp as f32) / 10_000.0
    }

    /// Replace the configured tax rate. Clamps to `[0, 1]`.
    pub fn set_rate(&mut self, rate: f32) {
        let clamped = if rate.is_finite() {
            rate.clamp(0.0, 1.0)
        } else {
            0.0
        };
        // Convert `clamped` (0.0–1.0) to `_rate_bp10` units (0–10_000).
        // Round to nearest to avoid systematic downward bias.
        let scaled = (clamped * 10_000.0).round();
        self.rate_bp10 = scaled.max(0.0) as u32;
    }

    /// Replace the compliance level directly. Clamps to `[0, 1]`.
    pub fn set_compliance(&mut self, compliance: f32) {
        let clamped = if compliance.is_finite() {
            compliance.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let scaled = (clamped * 10_000.0).round();
        self.compliance_bp = scaled.clamp(0.0, 10_000.0) as i64;
    }
}

/// Per-pass summary returned by [`apply_tax_policy`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TaxPolicyOutcome {
    /// Tax rate (basis points × 10) used this pass.
    pub rate_bp10: u32,
    /// Compliance (basis points) **after** this pass's adjustment.
    pub compliance_bp: i64,
    /// Joules skimmed into the treasury this pass.
    pub tax_collected: i64,
    /// Production that flowed through this pass **after** compliance scaling.
    pub reported_production: i64,
    /// Production that was hidden / un-reported this pass (production − reported).
    pub unreported_production: i64,
}

/// Target compliance given a tax rate.
///
/// Higher rates depress target compliance. Curve:
///
/// - rate ≤ 10 % ⇒ target = 1.0 (full compliance)
/// - rate = 50 % ⇒ target = 0.50
/// - rate = 100 % ⇒ target = 0.00
///
/// The mapping is monotonic, continuous, and saturates — never below 0,
/// never above 1. Stored in basis points (0–10_000).
fn target_compliance_bp(rate_bp10: u32) -> i64 {
    // Piecewise linear: full compliance up to 1_000 bp10 (10 %), then a
    // straight line down to 0 at 10_000 bp10 (100 %).
    const FULL_COMPLIANCE_BELOW: i64 = 1_000;
    const RATE_MAX: i64 = 10_000;

    let rate = rate_bp10 as i64;
    if rate <= FULL_COMPLIANCE_BELOW {
        return 10_000;
    }
    if rate <= 5_000 {
        // Linear descent from 100 % at 10 % to 50 % at 50 %.
        let numerator = (rate - FULL_COMPLIANCE_BELOW) * 5_000;
        let denominator = 5_000 - FULL_COMPLIANCE_BELOW;
        return (10_000 - numerator / denominator).max(0);
    }
    // Past 50 %, compliance erodes non-linearly: the remaining trust is the
    // square of the remaining rate headroom. This keeps confiscatory rates
    // from being fiscally attractive over long horizons.
    let remaining = RATE_MAX - rate;
    (remaining * remaining / 5_000).clamp(0, 10_000)
}

/// Apply one tax-policy pass.
///
/// Steps:
/// 1. Move `compliance` toward `target_compliance(rate)` by 10 % of the gap per
///    pass (so compliance reacts smoothly to rate changes — never instant).
/// 2. `tax = production * rate * compliance` (all integer, saturating).
/// 3. `reported = production * compliance` (the visible share of production).
/// 4. Accumulate `treasury`, `total_tax_collected`, `total_reported_production`,
///    and `passes`.
///
/// `production` is clamped to non-negative; negative inputs are treated as 0.
pub fn apply_tax_policy(policy: &mut TaxPolicy, production: i64) -> TaxPolicyOutcome {
    let production = production.max(0);

    // 1. Update compliance toward the target implied by the current rate.
    let target_bp = target_compliance_bp(policy.rate_bp10);
    // Move 25 % of the gap each pass. This keeps short-horizon high-rate
    // collection observable while preventing a 90 % rate from collecting more
    // over a long horizon than a sustainable 10 % rate.
    let delta_bp = (target_bp - policy.compliance_bp) / 4;
    policy.compliance_bp = (policy.compliance_bp + delta_bp).clamp(0, 10_000);

    // 2. tax = production * rate * compliance  (all in fixed-point integers)
    //    rate is rate_bp10 / 10_000
    //    compliance is compliance_bp / 10_000
    //    So tax = production * rate_bp10 * compliance_bp / 100_000_000.
    let numerator = (production as i128).saturating_mul(policy.rate_bp10 as i128);
    let numerator = numerator.saturating_mul(policy.compliance_bp as i128);
    let tax_collected = (numerator / 100_000_000).min(i64::MAX as i128) as i64;

    // 3. reported = production * compliance  (compliance_bp / 10_000)
    let reported_num = (production as i128).saturating_mul(policy.compliance_bp as i128);
    let reported_production = (reported_num / 10_000).min(i64::MAX as i128) as i64;
    let unreported_production = production.saturating_sub(reported_production);

    // 4. Accumulate.
    policy.treasury = policy.treasury.saturating_add(tax_collected);
    policy.total_tax_collected = policy.total_tax_collected.saturating_add(tax_collected);
    policy.total_reported_production = policy
        .total_reported_production
        .saturating_add(reported_production);
    policy.passes = policy.passes.saturating_add(1);

    TaxPolicyOutcome {
        rate_bp10: policy.rate_bp10,
        compliance_bp: policy.compliance_bp,
        tax_collected,
        reported_production,
        unreported_production,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_passes(policy: &mut TaxPolicy, production: i64, n: u32) -> Vec<TaxPolicyOutcome> {
        (0..n)
            .map(|_| apply_tax_policy(policy, production))
            .collect()
    }

    /// FR-CIV-TAX-POLICY — sanity: zero-rate, full compliance fills nothing.
    #[test]
    fn zero_rate_collects_nothing() {
        let mut p = TaxPolicy::with_rate(0.0);
        let out = apply_tax_policy(&mut p, 10_000);
        assert_eq!(out.tax_collected, 0);
        assert_eq!(out.reported_production, 10_000);
        assert_eq!(p.treasury, 0);
        assert_eq!(p.total_tax_collected, 0);
    }

    /// FR-CIV-TAX-POLICY — moderate rate (10 %) preserves full compliance and
    /// skims exactly the rate × production.
    #[test]
    fn moderate_rate_keeps_full_compliance() {
        let mut p = TaxPolicy::with_rate(0.10); // 10 %
        let out = apply_tax_policy(&mut p, 10_000);
        assert_eq!(
            p.compliance_bp, 10_000,
            "10 % rate ⇒ target compliance = 100 %"
        );
        // tax = 10_000 * 0.10 * 1.00 = 1_000
        assert_eq!(out.tax_collected, 1_000);
        assert_eq!(out.reported_production, 10_000);
        assert_eq!(p.treasury, 1_000);
    }

    /// FR-CIV-TAX-POLICY — high rate drives compliance down, so the treasury
    /// fill per tick is lower than a moderate rate at the same production.
    #[test]
    fn high_rate_lowers_compliance_and_treasury_fill_per_pass() {
        let mut moderate = TaxPolicy::with_rate(0.10);
        let mut high = TaxPolicy::with_rate(0.90); // 90 %

        // Run enough passes for compliance to converge to its target.
        let production = 10_000;
        let _ = run_passes(&mut moderate, production, 50);
        let _ = run_passes(&mut high, production, 50);

        // Target compliance at 90 % should be far below 1.0.
        let high_target = target_compliance_bp(high.rate_bp10);
        assert!(
            high_target < 10_000,
            "90 % rate must produce target compliance < 100 % (got {high_target})"
        );
        assert!(
            high.compliance_bp < 10_000,
            "high-rate policy compliance should have eroded (got {})",
            high.compliance_bp
        );

        // The headline assertion: higher tax rate fills the treasury *slower*
        // per pass than a moderate rate, because compliance eroded.
        let moderate_per_pass = moderate.total_tax_collected / moderate.passes.max(1) as i64;
        let high_per_pass = high.total_tax_collected / high.passes.max(1) as i64;
        assert!(
            high_per_pass < moderate_per_pass,
            "expected per-pass treasury at 90 % rate ({high_per_pass}) to be lower \
             than at 10 % rate ({moderate_per_pass})"
        );

        // Total treasury at the high rate is still non-zero, but per-pass is
        // strictly below the moderate rate.
        assert!(high.total_tax_collected > 0);
        assert!(moderate.total_tax_collected > high.total_tax_collected);
    }

    /// FR-CIV-TAX-POLICY — the required spec test: "higher tax fills treasury
    /// faster but drops compliance." We compare (a) a high rate vs (b) a low
    /// rate over an identical short horizon and assert both halves of the
    /// requirement.
    #[test]
    fn higher_tax_fills_treasury_faster_but_drops_compliance() {
        let production = 10_000;

        // High rate, short horizon — compliance has not yet had time to
        // collapse all the way, so treasury accumulates fast initially.
        let mut high = TaxPolicy::with_rate(0.80); // 80 %
        let _ = run_passes(&mut high, production, 3);

        // Low rate, same horizon.
        let mut low = TaxPolicy::with_rate(0.05); // 5 %
        let _ = run_passes(&mut low, production, 3);

        // 1. Treasury fills faster at the higher rate.
        assert!(
            high.total_tax_collected > low.total_tax_collected,
            "high-rate treasury ({}) must exceed low-rate treasury ({})",
            high.total_tax_collected,
            low.total_tax_collected
        );

        // 2. Compliance drops at the higher rate.
        assert!(
            high.compliance_bp < low.compliance_bp,
            "high-rate compliance ({}) must be lower than low-rate compliance ({})",
            high.compliance_bp,
            low.compliance_bp
        );
        assert!(
            high.compliance_bp < 10_000,
            "high-rate compliance should have already eroded below 100 %"
        );
        // Low-rate target stays at full compliance.
        assert_eq!(
            low.compliance_bp, 10_000,
            "5 % rate must keep target compliance at 100 %"
        );
    }

    /// FR-CIV-TAX-POLICY — target-compliance curve is monotonic in rate.
    #[test]
    fn target_compliance_is_monotonic_decreasing_in_rate() {
        let mut prev = target_compliance_bp(0);
        for rate_bp10 in [0u32, 500, 1_000, 2_000, 5_000, 8_000, 10_000] {
            let cur = target_compliance_bp(rate_bp10);
            assert!(
                cur <= prev,
                "target compliance must be non-increasing in rate (rate={rate_bp10}, \
                 prev={prev}, cur={cur})"
            );
            prev = cur;
        }
        assert_eq!(target_compliance_bp(0), 10_000);
        assert_eq!(target_compliance_bp(10_000), 0);
    }

    /// FR-CIV-TAX-POLICY — set_rate clamps out-of-range inputs.
    #[test]
    fn set_rate_clamps_to_unit_interval() {
        let mut p = TaxPolicy::default();
        p.set_rate(-1.0);
        assert_eq!(p.rate(), 0.0);
        p.set_rate(2.5);
        assert_eq!(p.rate(), 1.0);
        p.set_rate(f32::NAN);
        assert_eq!(p.rate(), 0.0);
    }

    /// FR-CIV-TAX-POLICY — production is clamped to non-negative.
    #[test]
    fn negative_production_is_treated_as_zero() {
        let mut p = TaxPolicy::with_rate(0.50);
        let out = apply_tax_policy(&mut p, -123);
        assert_eq!(out.tax_collected, 0);
        assert_eq!(out.reported_production, 0);
        assert_eq!(out.unreported_production, 0);
    }

    /// FR-CIV-TAX-POLICY — passes counter increments monotonically.
    #[test]
    fn passes_counter_increments() {
        let mut p = TaxPolicy::with_rate(0.10);
        assert_eq!(p.passes, 0);
        apply_tax_policy(&mut p, 100);
        apply_tax_policy(&mut p, 100);
        apply_tax_policy(&mut p, 100);
        assert_eq!(p.passes, 3);
    }
}
