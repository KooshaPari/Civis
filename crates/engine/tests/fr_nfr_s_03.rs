//! FR-NFR-S-03 — citizen count scaling, 1k → 10k sub-linear (< 8×).
//!
//! Spec: `docs/traceability/fr-nfr-s-03/fr-nfr-s-03-intent.md`, sourced
//! from `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3. A 10× population
//! increase must not produce a 10× tick-time blowup; the measurable
//! target is `tick_time_10k / tick_time_1k < 8` (expected ~5 with
//! rayon data-parallel phases and the SoA layout).
//!
//! The acceptance gate is `civ_server::perf_budgets::tick_scale_budget_met`.
//! These tests exercise its behavioral contract on real measured-pair
//! inputs: the budget constant, the rayon-expected ~5× pass, the strict
//! sub-8 boundary, rejection of linear (10×) and degenerate baselines,
//! and the end-to-end report plumbing.

use civ_server::perf_budgets::{
    tick_scale_budget_met, BudgetMeasurement, BudgetReport, TICK_SCALE_BUDGET,
};

/// FR-NFR-S-03: the budget constant is the spec's 8× bound.
#[test]
fn nfr_s_03_budget_constant_is_eight_x() {
    assert!(
        (TICK_SCALE_BUDGET - 8.0).abs() < f64::EPSILON,
        "spec §10.3 bounds tick_time_10k/tick_time_1k at 8, got {}",
        TICK_SCALE_BUDGET
    );
}

/// FR-NFR-S-03 happy path: sub-linear scaling passes. The spec's
/// expected rayon result (~5×) and the tightest legal ratio just under
/// 8× are both accepted; perfect O(1) scaling trivially passes.
#[test]
fn nfr_s_03_sublinear_scaling_accepted() {
    // Identical tick time at 1k and 10k citizens (best case).
    assert!(tick_scale_budget_met(10.0, 10.0));
    // The spec's rayon expectation: ~5× for a 10× population.
    assert!(tick_scale_budget_met(10.0, 50.0), "~5x is the expected result");
    // Tightest legal ratio: 7.99× must pass.
    assert!(tick_scale_budget_met(10.0, 79.9));
}

/// FR-NFR-S-03 edge case: linear scaling (10×) or worse violates the
/// requirement, as does a degenerate (non-positive) 1k baseline.
#[test]
fn nfr_s_03_linear_or_degenerate_scaling_rejected() {
    // At the bound: ratio exactly 8.0 must fail (strict <).
    assert!(!tick_scale_budget_met(10.0, 80.0));
    // Full linear blowup: 10x citizens -> 10x tick time.
    assert!(
        !tick_scale_budget_met(10.0, 100.0),
        "linear scaling violates FR-NFR-S-03"
    );
    // Degenerate baselines are failed measurements, not passes.
    assert!(!tick_scale_budget_met(0.0, 1.0), "zero baseline");
    assert!(!tick_scale_budget_met(-1.0, 1.0), "negative baseline");
}

/// FR-NFR-S-03 end-to-end: measured tick-time pairs flow through
/// `BudgetMeasurement` and only the tick-scale flag reacts when the
/// 10k measurement crosses the budget.
#[test]
fn nfr_s_03_budget_report_surfaces_scaling_regression() {
    let base = BudgetMeasurement {
        concurrent_clients: 100,
        handshake_ms: 2.0,
        tick_1k_ms: 10.0,
        tick_10k_ms: 50.0,
        commands_per_sec: 1_500,
        event_log_bytes_per_min: 1_000_000,
        ws_frame_bytes: 10_000,
    };
    let ok = BudgetReport::from_measurement(base);
    assert!(ok.tick_scale, "5x scaling satisfies FR-NFR-S-03");
    assert!(ok.all_passed());

    let regressed = BudgetMeasurement {
        tick_10k_ms: 100.0,
        ..base
    };
    let bad = BudgetReport::from_measurement(regressed);
    assert!(
        !bad.tick_scale,
        "10x scaling violates FR-NFR-S-03 and must flip the flag"
    );
    assert!(!bad.all_passed());
    // Only the tick-scale flag reacts to the tick-time pair.
    assert!(bad.ws_handshake);
    assert!(bad.command_rate);
    assert!(bad.event_log);
    assert!(bad.ws_frame);
}
