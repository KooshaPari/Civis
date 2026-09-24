//! FR-NFR-S-06 — WebSocket frame size (< 20 KB average per frame).
//!
//! Spec: `docs/traceability/fr-nfr-s-06/fr-nfr-s-06-intent.md`, sourced
//! from `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3. The average
//! binary WebSocket frame carrying a 1k-citizen snapshot must stay under
//! 20 KB so large frames do not dominate bandwidth and inflate broadcast
//! lag for observers.
//!
//! The acceptance gate is `civ_server::perf_budgets::ws_frame_budget_met`.
//! These tests exercise its behavioral contract on real encoded-frame
//! sizes: the budget constant, acceptance of compact frames, rejection
//! at/over budget (strict <), and the end-to-end report plumbing a
//! broadcast load test uses to publish the average frame size.

use civ_server::perf_budgets::{
    ws_frame_budget_met, BudgetMeasurement, BudgetReport, WS_FRAME_BUDGET_BYTES,
};

/// FR-NFR-S-06: the budget constant is the spec's 20 KB figure.
#[test]
fn nfr_s_06_budget_constant_is_twenty_kb() {
    assert_eq!(
        WS_FRAME_BUDGET_BYTES,
        20 * 1024,
        "spec §10.3 bounds the average 1k-snapshot frame at 20 KB"
    );
}

/// FR-NFR-S-06 happy path: compact encodings stay within the budget —
/// an empty frame, a small delta frame, and a full-budget-tight frame.
#[test]
fn nfr_s_06_frames_within_budget_accepted() {
    assert!(ws_frame_budget_met(0));
    // A typical delta snapshot frame (a few KB of changed citizens).
    assert!(ws_frame_budget_met(4 * 1024));
    // Tightest legal frame: 1 byte under the budget.
    assert!(ws_frame_budget_met(WS_FRAME_BUDGET_BYTES - 1));
}

/// FR-NFR-S-06 edge case: the requirement is strict "< 20 KB", so a
/// frame exactly at the budget or a raw un-compacted snapshot blob
/// violates the NFR.
#[test]
fn nfr_s_06_frames_at_or_over_budget_rejected() {
    assert!(
        !ws_frame_budget_met(WS_FRAME_BUDGET_BYTES),
        "exactly 20 KB violates the strict < 20 KB requirement"
    );
    // A raw JSON snapshot that escaped the compact encoder.
    assert!(!ws_frame_budget_met(64 * 1024));
    // Saturating edge: usize::MAX must fail, not wrap.
    assert!(!ws_frame_budget_met(usize::MAX));
}

/// FR-NFR-S-06 end-to-end: the measured average frame size flows
/// through `BudgetMeasurement`; only the frame flag reacts when the
/// encoder blows the budget, and the aggregate report fails.
#[test]
fn nfr_s_06_budget_report_surfaces_oversized_frames() {
    let base = BudgetMeasurement {
        concurrent_clients: 100,
        handshake_ms: 2.0,
        tick_1k_ms: 10.0,
        tick_10k_ms: 50.0,
        commands_per_sec: 1_500,
        event_log_bytes_per_min: 1_000_000,
        ws_frame_bytes: 16 * 1024,
    };
    let ok = BudgetReport::from_measurement(base);
    assert!(ok.ws_frame, "16 KiB average satisfies FR-NFR-S-06");
    assert!(ok.all_passed());

    let bloated = BudgetMeasurement {
        ws_frame_bytes: 32 * 1024,
        ..base
    };
    let bad = BudgetReport::from_measurement(bloated);
    assert!(
        !bad.ws_frame,
        "32 KiB average violates FR-NFR-S-06 and must flip the flag"
    );
    assert!(!bad.all_passed());
    // Only the frame-size flag reacts to the frame sample.
    assert!(bad.ws_handshake);
    assert!(bad.tick_scale);
    assert!(bad.command_rate);
    assert!(bad.event_log);
}
