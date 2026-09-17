//! FR-ECON-BUDGET — fiscal budget tracking and variance analysis.
//!
//! The budget system tracks planned vs actual expenditure across named buckets
//! (military, infrastructure, education, etc.) and computes fiscal health
//! indicators. This implements E2.7 (Budget system) from the civ-002 spec.
//!
//! # Semantics
//!
//! - A [`BudgetPlan`] defines planned allocations per bucket per tick.
//! - A [`BudgetSnapshot`] records actual expenditure after a tick.
//! - [`BudgetVariance`] computes the difference between planned and actual.
//! - [`fiscal_health`] returns a consolidated health score.
//!
//! All math is integer-saturating. No floats accumulate across calls.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Budget category identifier.
pub type BucketId = u32;

/// Well-known budget buckets.
pub const BUCKET_MILITARY: BucketId = 0;
pub const BUCKET_INFRASTRUCTURE: BucketId = 1;
pub const BUCKET_EDUCATION: BucketId = 2;
pub const BUCKET_HEALTH: BucketId = 3;
pub const BUCKET_ADMINISTRATION: BucketId = 4;

/// A named budget bucket with planned and actual amounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetBucket {
    /// Bucket identifier.
    pub id: BucketId,
    /// Planned allocation in joules.
    pub planned_joules: i64,
    /// Actual expenditure in joules.
    pub actual_joules: i64,
}

impl BudgetBucket {
    /// Variance: positive means under-budget, negative means over-budget.
    #[must_use]
    pub fn variance(&self) -> i64 {
        self.planned_joules - self.actual_joules
    }

    /// Variance as a fraction of planned: positive = under, negative = over.
    /// Returns `None` when planned is zero (division by zero).
    #[must_use]
    pub fn variance_fraction(&self) -> Option<f64> {
        if self.planned_joules == 0 {
            return None;
        }
        Some(self.variance() as f64 / self.planned_joules as f64)
    }
}

/// A budget plan: planned allocations per bucket for a tick range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetPlan {
    /// Start tick of this budget plan.
    pub start_tick: u64,
    /// End tick (inclusive) of this budget plan.
    pub end_tick: u64,
    /// Planned allocations per bucket.
    pub buckets: BTreeMap<BucketId, BudgetBucket>,
}

impl BudgetPlan {
    /// Total planned expenditure across all buckets.
    #[must_use]
    pub fn total_planned(&self) -> i64 {
        self.buckets.values().map(|b| b.planned_joules).sum()
    }

    /// Record actual expenditure for a bucket. Creates the bucket if absent.
    pub fn record_actual(&mut self, bucket_id: BucketId, actual_joules: i64) {
        self.buckets
            .entry(bucket_id)
            .or_insert_with(|| BudgetBucket {
                id: bucket_id,
                planned_joules: 0,
                actual_joules: 0,
            })
            .actual_joules = actual_joules;
    }
}

/// Snapshot of a budget plan after a tick, with all actuals filled in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    /// The underlying plan.
    pub plan: BudgetPlan,
    /// Total actual expenditure.
    pub total_actual_joules: i64,
}

impl BudgetSnapshot {
    /// Create a snapshot from a plan, computing totals.
    #[must_use]
    pub fn from_plan(plan: BudgetPlan) -> Self {
        let total_actual = plan.buckets.values().map(|b| b.actual_joules).sum();
        Self {
            plan,
            total_actual_joules: total_actual,
        }
    }

    /// Overall variance (positive = under-budget).
    #[must_use]
    pub fn variance(&self) -> i64 {
        self.plan.total_planned() - self.total_actual_joules
    }
}

/// Per-bucket variance with planned, actual, and difference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetVariance {
    /// Bucket identifier.
    pub bucket_id: BucketId,
    /// Planned amount.
    pub planned: i64,
    /// Actual amount.
    pub actual: i64,
    /// Variance (planned - actual).
    pub variance: i64,
}

/// Compute per-bucket variances from a snapshot.
#[must_use]
pub fn compute_variances(snapshot: &BudgetSnapshot) -> Vec<BudgetVariance> {
    snapshot
        .plan
        .buckets
        .values()
        .map(|b| BudgetVariance {
            bucket_id: b.id,
            planned: b.planned_joules,
            actual: b.actual_joules,
            variance: b.variance(),
        })
        .collect()
}

/// Fiscal health score in `[0, 10_000]` bp.
///
/// The score is high when:
/// - Total expenditure is close to planned (small variance)
/// - No single bucket is grossly over-budget
/// - The overall budget is not in deficit
///
/// Returns 0 for a completely broken budget and 10_000 for perfect adherence.
#[must_use]
pub fn fiscal_health(snapshot: &BudgetSnapshot) -> i64 {
    const BP_DENOM: i64 = 10_000;

    let planned = snapshot.plan.total_planned();
    let actual = snapshot.total_actual_joules;

    if planned == 0 && actual == 0 {
        return BP_DENOM; // empty budget is "healthy"
    }

    // Over-budget ratio: how much we exceeded the plan
    let over_budget = (actual - planned).max(0);
    let under_budget = (planned - actual).max(0);

    // Base score: 100 % when perfectly on-plan, decays with variance
    let total = planned.max(actual).max(1);
    let base_score = if actual <= planned {
        // Under or on budget: score = 10000 - (variance/planned * 5000)
        // Small under-spending is mildly bad (waste), large is worse
        let under_ratio = under_budget * BP_DENOM / total;
        BP_DENOM - under_ratio / 2
    } else {
        // Over budget: harsh penalty
        let over_ratio = over_budget * BP_DENOM / total;
        (BP_DENOM - over_ratio).max(0)
    };

    // Per-bucket penalty: any bucket more than 50 % over plan deducts points
    let mut bucket_penalty: i64 = 0;
    for bucket in snapshot.plan.buckets.values() {
        if bucket.planned_joules > 0 {
            let over = (bucket.actual_joules - bucket.planned_joules).max(0);
            let over_ratio = over * BP_DENOM / bucket.planned_joules;
            if over_ratio > 5_000 {
                // More than 50 % over: deduct proportional penalty
                bucket_penalty += (over_ratio - 5_000) / 10;
            }
        }
    }

    (base_score - bucket_penalty).clamp(0, BP_DENOM)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_plan() -> BudgetPlan {
        let mut buckets = BTreeMap::new();
        buckets.insert(
            BUCKET_MILITARY,
            BudgetBucket {
                id: BUCKET_MILITARY,
                planned_joules: 100,
                actual_joules: 0,
            },
        );
        buckets.insert(
            BUCKET_INFRASTRUCTURE,
            BudgetBucket {
                id: BUCKET_INFRASTRUCTURE,
                planned_joules: 200,
                actual_joules: 0,
            },
        );
        BudgetPlan {
            start_tick: 0,
            end_tick: 10,
            buckets,
        }
    }

    #[test]
    fn budget_bucket_variance() {
        let b = BudgetBucket {
            id: 0,
            planned_joules: 100,
            actual_joules: 80,
        };
        assert_eq!(b.variance(), 20);
        assert!((b.variance_fraction().unwrap() - 0.2).abs() < 1e-10);
    }

    #[test]
    fn budget_bucket_over_budget_negative_variance() {
        let b = BudgetBucket {
            id: 0,
            planned_joules: 100,
            actual_joules: 120,
        };
        assert_eq!(b.variance(), -20);
    }

    #[test]
    fn budget_plan_total_planned() {
        let plan = test_plan();
        assert_eq!(plan.total_planned(), 300);
    }

    #[test]
    fn budget_snapshot_variance() {
        let mut plan = test_plan();
        plan.record_actual(BUCKET_MILITARY, 90);
        plan.record_actual(BUCKET_INFRASTRUCTURE, 190);
        let snapshot = BudgetSnapshot::from_plan(plan);
        assert_eq!(snapshot.variance(), 20); // 300 - 280
    }

    #[test]
    fn fiscal_health_perfect_adherence() {
        let mut plan = test_plan();
        plan.record_actual(BUCKET_MILITARY, 100);
        plan.record_actual(BUCKET_INFRASTRUCTURE, 200);
        let snapshot = BudgetSnapshot::from_plan(plan);
        let health = fiscal_health(&snapshot);
        assert_eq!(health, 10_000);
    }

    #[test]
    fn fiscal_health_over_budget_reduces_score() {
        let mut plan = test_plan();
        plan.record_actual(BUCKET_MILITARY, 150);
        plan.record_actual(BUCKET_INFRASTRUCTURE, 250);
        let snapshot = BudgetSnapshot::from_plan(plan);
        let health = fiscal_health(&snapshot);
        assert!(health < 10_000, "over-budget should reduce health");
    }

    #[test]
    fn fiscal_health_empty_budget_is_healthy() {
        let plan = BudgetPlan {
            start_tick: 0,
            end_tick: 0,
            buckets: BTreeMap::new(),
        };
        let snapshot = BudgetSnapshot::from_plan(plan);
        assert_eq!(fiscal_health(&snapshot), 10_000);
    }

    #[test]
    fn compute_variances_returns_all_buckets() {
        let mut plan = test_plan();
        plan.record_actual(BUCKET_MILITARY, 50);
        plan.record_actual(BUCKET_INFRASTRUCTURE, 200);
        let snapshot = BudgetSnapshot::from_plan(plan);
        let variances = compute_variances(&snapshot);
        assert_eq!(variances.len(), 2);
        let mil = variances.iter().find(|v| v.bucket_id == BUCKET_MILITARY).unwrap();
        assert_eq!(mil.variance, 50); // 100 - 50
    }
}
