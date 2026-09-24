//! FR-NFR-S-02 — WebSocket connection overhead (< 5 ms per client).
//!
//! Spec: `docs/traceability/fr-nfr-s-02/fr-nfr-s-02-intent.md`, sourced
//! from `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3. A viewer's join
//! path (WS upgrade + handshake + initial snapshot) must complete in
//! under 5 ms so the first frame lands well inside one 100 ms tick.
//!
//! The acceptance gate implemented by the server is
//! `civ_server::perf_budgets::ws_handshake_budget_met`, aggregated
//! through `BudgetReport`. These tests exercise the gate's behavioral
//! contract: the budget constant, strict sub-5 ms acceptance, rejection
//! at/over budget, and the end-to-end report plumbing a load test uses
//! to publish the p95 measurement.

use civ_server::perf_budgets::{
    ws_handshake_budget_met, BudgetMeasurement, BudgetReport, WS_HANDSHAKE_BUDGET_MS,
};

/// FR-NFR-S-02: the budget constant itself is the spec's 5 ms figure.
#[test]
fn nfr_s_02_budget_constant_is_five_millis() {
    assert!(
        (WS_HANDSHAKE_BUDGET_MS - 5.0).abs() < f64::EPSILON,
        "spec §10.3 requires a 5 ms handshake budget, got {}",
        WS_HANDSHAKE_BUDGET_MS
    );
}

/// FR-NFR-S-02 happy path: join latencies strictly under 5 ms pass the
/// gate, including a join that completes instantaneously.
#[test]
fn nfr_s_02_handshake_under_budget_accepted() {
    assert!(ws_handshake_budget_met(0.0), "0 ms is within budget");
    assert!(ws_handshake_budget_met(1.25), "typical local join passes");
    // The tightest practical legal measurement: 1 µs below the budget must
    // pass (5.0 - f64::EPSILON would round back to exactly 5.0 at this
    // magnitude, since one ulp near 5.0 is ~8.9e-16 > f64::EPSILON).
    assert!(ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS - 0.001));
}

/// FR-NFR-S-02 edge case: the requirement is strict "< 5 ms", so a
/// measurement exactly at the budget or beyond it is a violation.
#[test]
fn nfr_s_02_handshake_at_or_over_budget_rejected() {
    assert!(
        !ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS),
        "exactly 5 ms violates the strict < 5 ms requirement"
    );
    assert!(!ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS + 1.0));
    assert!(!ws_handshake_budget_met(f64::INFINITY));
}

/// FR-NFR-S-02 end-to-end: a p95 measurement reported through
/// `BudgetMeasurement` flips only the handshake flag, and a failing
/// handshake alone is enough to fail the aggregate report.
#[test]
fn nfr_s_02_budget_report_surfaces_handshake_violation() {
    let passing = BudgetMeasurement {
        concurrent_clients: 100,
        handshake_ms: 4.5,
        tick_1k_ms: 10.0,
        tick_10k_ms: 50.0,
        commands_per_sec: 1_500,
        event_log_bytes_per_min: 1_000_000,
        ws_frame_bytes: 10_000,
    };
    let ok = BudgetReport::from_measurement(passing);
    assert!(ok.ws_handshake, "4.5 ms join must satisfy FR-NFR-S-02");
    assert!(ok.all_passed());

    let slow = BudgetMeasurement {
        handshake_ms: 5.5,
        ..passing
    };
    let bad = BudgetReport::from_measurement(slow);
    assert!(!bad.ws_handshake, "5.5 ms join violates FR-NFR-S-02");
    assert!(
        !bad.all_passed(),
        "slow handshake must fail the aggregate budget report"
    );
    // Only the handshake flag reacts to the handshake sample.
    assert!(bad.ws_client);
    assert!(bad.tick_scale);
    assert!(bad.command_rate);
}

#[cfg(test)]
mod fr_nfr_s_02 {
    use civ_engine::WorldState;

    // NFR-S-02 — fresh world state starts at tick 0 (handshake snapshot baseline).
    #[test]
    fn verify_nfr_s_02_basic() {
        let ws = WorldState::default();
        assert_eq!(ws.tick, 0);
        let ws2 = WorldState::default();
        assert_eq!(ws.tick, ws2.tick, "fresh instances must be stable");
    }
}
