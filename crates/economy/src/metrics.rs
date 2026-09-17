//! FR-METRICS-001/002/003 — Economy metrics derived from budget + consumption.
//!
//! The [`EconomyMetrics`] struct provides four diagnostic fields:
//!
//! - `waste_joules` — Joules consumed but not productively allocated (lost as waste heat).
//! - `surplus_joules` — Joules remaining in the budget after consumption.
//! - `tyranny_index` — Proxy for over-taxation; rises when effective tax rate is high.
//! - `legitimacy_index` — Proxy for citizen consent; falls when tyranny rises.
//!
//! All values are derived from `energy_budget_joules` and `consumption_joules`
//! inputs. The computation is constant-time, allocation-free, and deterministic.
//!
//! Fixed-point variants use basis points (bp, 1 bp = 0.01 %) for replay
//! compatibility. Float variants are available for research export only.

use serde::{Deserialize, Serialize};

/// Waste-heat fraction of total consumption. At 10 %, every 10 Joules consumed
/// produce 1 Joule of waste heat.
const WASTE_FRACTION_BP: i64 = 1_000; // 10.00 % in basis points

/// Basis-point denominator (10_000 bp = 100 %).
const BP_DENOM: i64 = 10_000;

/// Maximum tyranny index (100 % tyranny = 10_000 bp).
const TYRANNY_MAX_BP: i64 = 10_000;

/// FR-METRICS-001 — economy metrics snapshot.
///
/// All fields are f64 for research-export convenience. The fixed-point
/// variants are computed internally and exposed via dedicated accessors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomyMetrics {
    /// Joules lost as waste heat (10 % of consumption).
    pub waste_joules: f64,
    /// Joules remaining in the budget after consumption (surplus = budget - consumption).
    pub surplus_joules: f64,
    /// Tyranny index in `[0.0, 1.0]`: rises with effective tax rate.
    pub tyranny_index: f64,
    /// Legitimacy index in `[0.0, 1.0]`: falls as tyranny rises.
    pub legitimacy_index: f64,
}

impl Default for EconomyMetrics {
    fn default() -> Self {
        Self {
            waste_joules: 0.0,
            surplus_joules: 0.0,
            tyranny_index: 0.0,
            legitimacy_index: 1.0,
        }
    }
}

/// FR-METRICS-003 — fixed-point economy metrics in basis points.
///
/// For replay-deterministic paths. Float variants are for research export only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EconomyMetricsFixed {
    /// Waste joules (integer).
    pub waste_joules: i64,
    /// Surplus joules (integer).
    pub surplus_joules: i64,
    /// Tyranny index in basis points `[0, 10_000]`.
    pub tyranny_index_bp: i64,
    /// Legitimacy index in basis points `[0, 10_000]`.
    pub legitimacy_index_bp: i64,
}

impl EconomyMetricsFixed {
    /// Convert to float metrics for research export.
    #[must_use]
    pub fn to_float(&self) -> EconomyMetrics {
        EconomyMetrics {
            waste_joules: self.waste_joules as f64,
            surplus_joules: self.surplus_joules as f64,
            tyranny_index: self.tyranny_index_bp as f64 / BP_DENOM as f64,
            legitimacy_index: self.legitimacy_index_bp as f64 / BP_DENOM as f64,
        }
    }
}

/// Compute tyranny index from effective tax rate.
///
/// Higher tax rates produce higher tyranny. The mapping is:
///
/// - rate ≤ 10 % ⇒ tyranny = 0.0 (low taxation, no tyranny)
/// - rate = 50 % ⇒ tyranny = 0.40
/// - rate = 100 % ⇒ tyranny = 1.0 (total confiscation)
///
/// Returns basis points `[0, 10_000]`.
fn tyranny_from_tax_rate_bp(tax_rate_bp: i64) -> i64 {
    const LOW_RATE_THRESHOLD: i64 = 1_000; // 10 %
    if tax_rate_bp <= LOW_RATE_THRESHOLD {
        return 0;
    }
    // Linear from 0 at 10 % to 10_000 at 100 %
    let effective = tax_rate_bp - LOW_RATE_THRESHOLD;
    let span = BP_DENOM - LOW_RATE_THRESHOLD; // 9_000
    (effective * TYRANNY_MAX_BP / span).min(TYRANNY_MAX_BP)
}

/// Compute legitimacy from tyranny.
///
/// Legitimacy decays as tyranny rises. The formula is:
///
/// `legitimacy = (1 - tyranny)^2`
///
/// This means moderate tyranny preserves most legitimacy, but high tyranny
/// causes a rapid collapse. Returns basis points `[0, 10_000]`.
fn legitimacy_from_tyranny_bp(tyranny_bp: i64) -> i64 {
    let one_minus_t = BP_DENOM - tyranny_bp.max(0).min(BP_DENOM);
    (one_minus_t * one_minus_t / BP_DENOM).max(0)
}

/// FR-METRICS-002 — compute economy metrics from budget and consumption.
///
/// `energy_budget_joules` is the macro budget at the start of the tick.
/// `consumption_joules` is the joules drained this tick.
/// `tax_rate_bp` is the effective tax rate in basis points.
///
/// Returns constant-time, allocation-free metrics. No I/O.
///
/// # Panics
///
/// This function does not panic; it clamps all inputs to valid ranges.
#[must_use]
pub fn compute_metrics(
    energy_budget_joules: i64,
    consumption_joules: i64,
    tax_rate_bp: i64,
) -> EconomyMetricsFixed {
    let budget = energy_budget_joules.max(0);
    let consumption = consumption_joules.max(0);

    // Waste = 10 % of consumption
    let waste = consumption * WASTE_FRACTION_BP / BP_DENOM;

    // Surplus = budget - consumption (clamped to 0)
    let surplus = (budget - consumption).max(0);

    // Tyranny from tax rate
    let tyranny_bp = tyranny_from_tax_rate_bp(tax_rate_bp);

    // Legitimacy from tyranny
    let legitimacy_bp = legitimacy_from_tyranny_bp(tyranny_bp);

    EconomyMetricsFixed {
        waste_joules: waste,
        surplus_joules: surplus,
        tyranny_index_bp: tyranny_bp,
        legitimacy_index_bp: legitimacy_bp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-METRICS-002: `compute(1000, 500, 1000)` returns waste=50, surplus=500.
    #[test]
    fn compute_metrics_basic() {
        let m = compute_metrics(1000, 500, 1_000); // 10 % tax
        assert_eq!(m.waste_joules, 50); // 10 % of 500
        assert_eq!(m.surplus_joules, 500); // 1000 - 500
    }

    /// FR-METRICS-003: zero tax rate → zero tyranny, full legitimacy.
    #[test]
    fn zero_tax_full_legitimacy() {
        let m = compute_metrics(1000, 500, 0);
        assert_eq!(m.tyranny_index_bp, 0);
        assert_eq!(m.legitimacy_index_bp, 10_000);
    }

    /// FR-METRICS-003: 100 % tax → max tyranny, zero legitimacy.
    #[test]
    fn max_tax_zero_legitimacy() {
        let m = compute_metrics(1000, 500, 10_000);
        assert_eq!(m.tyranny_index_bp, 10_000);
        assert_eq!(m.legitimacy_index_bp, 0);
    }

    /// FR-METRICS-001: consumption exceeds budget → surplus is 0, waste is 10 % of consumption.
    #[test]
    fn consumption_exceeds_budget() {
        let m = compute_metrics(100, 200, 0);
        assert_eq!(m.waste_joules, 20);
        assert_eq!(m.surplus_joules, 0);
    }

    /// FR-METRICS-002: zero consumption → zero waste, full surplus.
    #[test]
    fn zero_consumption() {
        let m = compute_metrics(500, 0, 0);
        assert_eq!(m.waste_joules, 0);
        assert_eq!(m.surplus_joules, 500);
    }

    /// FR-METRICS-003: float and fixed-point metrics agree to 6 decimal places.
    #[test]
    fn fixed_point_agrees_with_float() {
        let fixed = compute_metrics(1000, 500, 3_000); // 30 % tax
        let float = fixed.to_float();
        assert!((float.waste_joules - 50.0).abs() < 1e-6);
        assert!((float.surplus_joules - 500.0).abs() < 1e-6);
        // tyranny at 30 %: (3000 - 1000) * 10000 / 9000 ≈ 2222.22 bp → 0.222222
        let tyranny_f = fixed.tyranny_index_bp as f64 / 10_000.0;
        assert!((float.tyranny_index - tyranny_f).abs() < 1e-6);
    }

    /// 10 % tax rate → zero tyranny (below threshold).
    #[test]
    fn ten_percent_tax_no_tyranny() {
        let m = compute_metrics(1000, 500, 1_000);
        assert_eq!(m.tyranny_index_bp, 0);
    }

    /// 50 % tax rate → ~44.44 % tyranny.
    #[test]
    fn fifty_percent_tax_moderate_tyranny() {
        let m = compute_metrics(1000, 500, 5_000);
        // (5000 - 1000) * 10000 / 9000 = 4444 bp
        assert_eq!(m.tyranny_index_bp, 4_444);
        // legitimacy = (10000 - 4444)^2 / 10000 = 5556^2 / 10000 = 30869136 / 10000 = 3086
        assert_eq!(m.legitimacy_index_bp, 3_086);
    }

    /// Negative inputs are clamped to zero.
    #[test]
    fn negative_inputs_clamped() {
        let m = compute_metrics(-100, -50, -200);
        assert_eq!(m.waste_joules, 0);
        assert_eq!(m.surplus_joules, 0);
    }

    /// Tyranny is monotonic increasing with tax rate.
    #[test]
    fn tyranny_monotonic() {
        let mut prev = 0;
        for rate in (0..=10_000).step_by(500) {
            let m = compute_metrics(1000, 500, rate);
            assert!(
                m.tyranny_index_bp >= prev,
                "tyranny must be monotonic: {rate} gave {} < {prev}",
                m.tyranny_index_bp
            );
            prev = m.tyranny_index_bp;
        }
    }

    /// Legitimacy is monotonic decreasing with tax rate.
    #[test]
    fn legitimacy_monotonic() {
        let mut prev = i64::MAX;
        for rate in (0..=10_000).step_by(500) {
            let m = compute_metrics(1000, 500, rate);
            assert!(
                m.legitimacy_index_bp <= prev,
                "legitimacy must be monotonic: {rate} gave {} > {prev}",
                m.legitimacy_index_bp
            );
            prev = m.legitimacy_index_bp;
        }
    }
}
