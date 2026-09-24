//! FR-NFR-S-05 — event log growth rate (< 5 MB/minute at 1k citizens).
//!
//! Spec: `docs/traceability/fr-nfr-s-05/fr-nfr-s-05-intent.md`, sourced
//! from `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3. The append-only
//! replay/event log must grow by less than 5 MB per minute at the
//! canonical 1k-citizen, 10 ticks/sec scenario (~7 GB/day archival
//! footprint for a multi-day research run).
//!
//! The acceptance gate is `civ_server::perf_budgets::event_log_budget_met`.
//! These tests exercise its behavioral contract on real byte-rate
//! samples: the budget constant, acceptance of a within-budget rate,
//! rejection at/over budget (strict <), and the end-to-end report
//! plumbing a Prometheus recording rule would feed.

use civ_server::perf_budgets::{
    event_log_budget_met, BudgetMeasurement, BudgetReport, EVENT_LOG_BUDGET_BYTES_MIN,
};

/// FR-NFR-S-05: the budget constant is the spec's 5 MB/minute figure.
#[test]
fn nfr_s_05_budget_constant_is_five_mb_per_min() {
    assert_eq!(
        EVENT_LOG_BUDGET_BYTES_MIN,
        5 * 1024 * 1024,
        "spec §10.3 budgets event-log growth at 5 MB/minute"
    );
}

/// FR-NFR-S-05 happy path: growth rates strictly under the budget
/// pass — an idle/lightly-loaded session and a full-rate recording
/// run that lands just inside the envelope.
#[test]
fn nfr_s_05_growth_within_budget_accepted() {
    assert!(event_log_budget_met(0), "no growth is trivially within budget");
    // A lightly-loaded session: ~1 MB/min.
    assert!(event_log_budget_met(1_000_000));
    // Tightest legal rate: 1 byte/minute under the budget.
    assert!(event_log_budget_met(EVENT_LOG_BUDGET_BYTES_MIN - 1));
}

/// FR-NFR-S-05 edge case: the requirement is strict "< 5 MB/min", so a
/// rate exactly at the budget or beyond it violates the NFR — this is
/// the case a recording rule alarms on.
#[test]
fn nfr_s_05_growth_at_or_over_budget_rejected() {
    assert!(
        !event_log_budget_met(EVENT_LOG_BUDGET_BYTES_MIN),
        "exactly 5 MB/min violates the strict < 5 MB/min requirement"
    );
    // Hot session with unbounded log growth.
    assert!(!event_log_budget_met(EVENT_LOG_BUDGET_BYTES_MIN + 1));
    assert!(!event_log_budget_met(50_000_000));
    // Saturating edge: u64::MAX must fail, not wrap.
    assert!(!event_log_budget_met(u64::MAX));
}

/// FR-NFR-S-05 end-to-end: the recorded byte rate flows through
/// `BudgetMeasurement`; only the event-log flag reacts when the
/// 60-second rate crosses the budget.
#[test]
fn nfr_s_05_budget_report_surfaces_runaway_log_growth() {
    let base = BudgetMeasurement {
        concurrent_clients: 100,
        handshake_ms: 2.0,
        tick_1k_ms: 10.0,
        tick_10k_ms: 50.0,
        commands_per_sec: 1_500,
        event_log_bytes_per_min: 4_000_000,
        ws_frame_bytes: 10_000,
    };
    let ok = BudgetReport::from_measurement(base);
    assert!(ok.event_log, "4 MB/min satisfies FR-NFR-S-05");
    assert!(ok.all_passed());

    let runaway = BudgetMeasurement {
        event_log_bytes_per_min: 8_000_000,
        ..base
    };
    let bad = BudgetReport::from_measurement(runaway);
    assert!(
        !bad.event_log,
        "8 MB/min violates FR-NFR-S-05 and must flip the flag"
    );
    assert!(!bad.all_passed());
    // Only the event-log flag reacts to the byte-rate sample.
    assert!(bad.ws_handshake);
    assert!(bad.tick_scale);
    assert!(bad.command_rate);
    assert!(bad.ws_frame);
}
