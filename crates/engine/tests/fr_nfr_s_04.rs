//! FR-NFR-S-04 — command throughput (> 1,000 commands/sec).
//!
//! Spec: `docs/traceability/fr-nfr-s-04/fr-nfr-s-04-intent.md`, sourced
//! from `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3. The server must
//! absorb a 1 kHz command flood without the tick interval lengthening:
//! commands are queued asynchronously and the acceptance budget is
//! > 1,000 commands/sec.
//!
//! The acceptance gate is `civ_server::perf_budgets::command_rate_budget_met`.
//! These tests exercise its behavioral contract on real command-rate
//! samples: the budget constant, acceptance at/above 1 kHz, rejection
//! below it, and the end-to-end report plumbing a stress test uses to
//! publish the measured rate.

use civ_server::perf_budgets::{
    command_rate_budget_met, BudgetMeasurement, BudgetReport, COMMAND_RATE_BUDGET_PER_SEC,
};

/// FR-NFR-S-04: the budget constant is the spec's 1,000 cmd/s figure.
#[test]
fn nfr_s_04_budget_constant_is_one_khz() {
    assert_eq!(
        COMMAND_RATE_BUDGET_PER_SEC, 1_000,
        "spec §10.3 requires absorbing > 1,000 commands/sec"
    );
}

/// FR-NFR-S-04 happy path: a command rate at or above the budget
/// passes — the tick loop is not blocked by the flood.
#[test]
fn nfr_s_04_command_rate_at_or_above_budget_accepted() {
    assert!(
        command_rate_budget_met(COMMAND_RATE_BUDGET_PER_SEC),
        "exactly 1,000 cmd/s meets the budget"
    );
    assert!(command_rate_budget_met(COMMAND_RATE_BUDGET_PER_SEC + 1));
    // A scripted-agent burst well beyond the requirement.
    assert!(command_rate_budget_met(5_000));
    // Saturating edge: u64::MAX must pass (rate cannot be "too high").
    assert!(command_rate_budget_met(u64::MAX));
}

/// FR-NFR-S-04 edge case: rates below the budget violate the
/// requirement — including zero (the tick loop fully blocked).
#[test]
fn nfr_s_04_command_rate_below_budget_rejected() {
    assert!(
        !command_rate_budget_met(COMMAND_RATE_BUDGET_PER_SEC - 1),
        "999 cmd/s is below the FR-NFR-S-04 budget"
    );
    assert!(!command_rate_budget_met(0), "blocked input queue reads 0");
    assert!(!command_rate_budget_met(100));
}

/// FR-NFR-S-04 end-to-end: a stress test's measured rate flows through
/// `BudgetMeasurement`; only the command-rate flag reacts when the
/// flood is absorbed slower than the budget.
#[test]
fn nfr_s_04_budget_report_surfaces_throughput_drop() {
    let base = BudgetMeasurement {
        concurrent_clients: 100,
        handshake_ms: 2.0,
        tick_1k_ms: 10.0,
        tick_10k_ms: 50.0,
        commands_per_sec: 1_200,
        event_log_bytes_per_min: 1_000_000,
        ws_frame_bytes: 10_000,
    };
    let ok = BudgetReport::from_measurement(base);
    assert!(ok.command_rate, "1,200 cmd/s satisfies FR-NFR-S-04");
    assert!(ok.all_passed());

    let stalled = BudgetMeasurement {
        commands_per_sec: 250,
        ..base
    };
    let bad = BudgetReport::from_measurement(stalled);
    assert!(
        !bad.command_rate,
        "250 cmd/s violates FR-NFR-S-04 and must flip the flag"
    );
    assert!(!bad.all_passed());
    // Only the command-rate flag reacts to the rate sample.
    assert!(bad.ws_handshake);
    assert!(bad.tick_scale);
    assert!(bad.event_log);
    assert!(bad.ws_frame);
}
