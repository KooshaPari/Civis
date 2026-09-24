//! Scalability performance budgets — NFR-S-01, NFR-S-02, NFR-S-03,
//! NFR-S-04, NFR-S-05, NFR-S-06.
//!
//! Per `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 Scalability
//! and `docs/traceability/index.md` rows 1231..1236, the workspace
//! SHALL enforce the following budgets on the headless server:
//!
//! | NFR ID  | Metric                       | Budget                          |
//! |---------|------------------------------|---------------------------------|
//! | NFR-S-01 | Max simultaneous WS clients | `>= WS_CLIENT_BUDGET`           |
//! | NFR-S-02 | Connection overhead          | `< WS_HANDSHAKE_BUDGET_MS`      |
//! | NFR-S-03 | Tick scaling 1k→10k citizens | ratio `< TICK_SCALE_BUDGET`     |
//! | NFR-S-04 | Command throughput           | `>= COMMAND_RATE_BUDGET_PER_SEC`|
//! | NFR-S-05 | Event-log growth rate        | `< EVENT_LOG_BUDGET_BYTES_MIN`  |
//! | NFR-S-06 | WS frame size (1k snapshot)  | `< WS_FRAME_BUDGET_BYTES`       |
//!
//! This module is the spec-side constants and tiny gate functions the
//! load tests and CI step `tests/load/100_clients.rs` import.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// NFR-S-01 — minimum simultaneous WebSocket clients the headless
/// server MUST accept at 10 ticks/sec.
pub const WS_CLIENT_BUDGET: usize = 100;

/// NFR-S-02 — per-client connection overhead budget (handshake +
/// initial snapshot, in milliseconds).
pub const WS_HANDSHAKE_BUDGET_MS: f64 = 5.0;

/// NFR-S-03 — `tick_time_10k / tick_time_1k` budget. The spec allows
/// at most ~8× with rayon (expect ~5×); we pin the budget at 8.0 so a
/// regression to linear scaling trips the gate.
pub const TICK_SCALE_BUDGET: f64 = 8.0;

/// NFR-S-04 — minimum command throughput (commands/sec) the server
/// MUST accept without tick delay.
pub const COMMAND_RATE_BUDGET_PER_SEC: u64 = 1_000;

/// NFR-S-05 — event-log growth budget at 1k citizens at 10 ticks/sec,
/// in bytes/minute.
pub const EVENT_LOG_BUDGET_BYTES_MIN: u64 = 5 * 1024 * 1024;

/// NFR-S-06 — WebSocket binary frame budget (average) for a 1k-citizen
/// snapshot, in bytes.
pub const WS_FRAME_BUDGET_BYTES: usize = 20 * 1024;

/// NFR-S-01 acceptance gate. Returns `true` when `concurrent_clients`
/// meets the budget. Hosts call this at startup after wiring the WS
/// listener.
#[must_use]
pub fn ws_client_budget_met(concurrent_clients: usize) -> bool {
    concurrent_clients >= WS_CLIENT_BUDGET
}

/// NFR-S-02 acceptance gate. Returns `true` when the per-client
/// handshake latency is within the budget.
#[must_use]
pub fn ws_handshake_budget_met(handshake_ms: f64) -> bool {
    handshake_ms < WS_HANDSHAKE_BUDGET_MS
}

/// NFR-S-03 acceptance gate. Returns `true` when the tick-time ratio
/// (10k / 1k) is sub-linear at the spec bound.
#[must_use]
pub fn tick_scale_budget_met(tick_1k_ms: f64, tick_10k_ms: f64) -> bool {
    if tick_1k_ms <= 0.0 {
        return false;
    }
    let ratio = tick_10k_ms / tick_1k_ms;
    ratio < TICK_SCALE_BUDGET
}

/// NFR-S-04 acceptance gate. Returns `true` when the command rate
/// meets the throughput budget.
#[must_use]
pub fn command_rate_budget_met(commands_per_sec: u64) -> bool {
    commands_per_sec >= COMMAND_RATE_BUDGET_PER_SEC
}

/// NFR-S-05 acceptance gate. Returns `true` when the event-log growth
/// rate is within the byte budget.
#[must_use]
pub fn event_log_budget_met(bytes_per_min: u64) -> bool {
    bytes_per_min < EVENT_LOG_BUDGET_BYTES_MIN
}

/// NFR-S-06 acceptance gate. Returns `true` when the average WS frame
/// size is within the byte budget.
#[must_use]
pub fn ws_frame_budget_met(frame_bytes: usize) -> bool {
    frame_bytes < WS_FRAME_BUDGET_BYTES
}

/// Pack every acceptance result into a single summary struct — useful
/// for the load-test bench to log one line per run with all budgets'
/// pass/fail status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetReport {
    /// NFR-S-01.
    pub ws_client: bool,
    /// NFR-S-02.
    pub ws_handshake: bool,
    /// NFR-S-03.
    pub tick_scale: bool,
    /// NFR-S-04.
    pub command_rate: bool,
    /// NFR-S-05.
    pub event_log: bool,
    /// NFR-S-06.
    pub ws_frame: bool,
}

impl BudgetReport {
    /// True iff every NFR-S-01..06 gate passed.
    #[must_use]
    pub fn all_passed(self) -> bool {
        self.ws_client
            && self.ws_handshake
            && self.tick_scale
            && self.command_rate
            && self.event_log
            && self.ws_frame
    }

    /// Build a report from a [`BudgetMeasurement`].
    #[must_use]
    pub fn from_measurement(m: BudgetMeasurement) -> Self {
        Self {
            ws_client: ws_client_budget_met(m.concurrent_clients),
            ws_handshake: ws_handshake_budget_met(m.handshake_ms),
            tick_scale: tick_scale_budget_met(m.tick_1k_ms, m.tick_10k_ms),
            command_rate: command_rate_budget_met(m.commands_per_sec),
            event_log: event_log_budget_met(m.event_log_bytes_per_min),
            ws_frame: ws_frame_budget_met(m.ws_frame_bytes),
        }
    }
}

/// Observed values used to compute a [`BudgetReport`].
#[derive(Debug, Clone, Copy, Default)]
pub struct BudgetMeasurement {
    /// NFR-S-01 sample.
    pub concurrent_clients: usize,
    /// NFR-S-02 sample.
    pub handshake_ms: f64,
    /// NFR-S-03 — 1k-citizen tick time.
    pub tick_1k_ms: f64,
    /// NFR-S-03 — 10k-citizen tick time.
    pub tick_10k_ms: f64,
    /// NFR-S-04 sample.
    pub commands_per_sec: u64,
    /// NFR-S-05 sample.
    pub event_log_bytes_per_min: u64,
    /// NFR-S-06 sample.
    pub ws_frame_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// NFR-S-01..06 budgets match the spec table values.
    #[test]
    fn budget_constants_match_spec() {
        assert_eq!(WS_CLIENT_BUDGET, 100);
        assert!((WS_HANDSHAKE_BUDGET_MS - 5.0).abs() < f64::EPSILON);
        assert!((TICK_SCALE_BUDGET - 8.0).abs() < f64::EPSILON);
        assert_eq!(COMMAND_RATE_BUDGET_PER_SEC, 1_000);
        assert_eq!(EVENT_LOG_BUDGET_BYTES_MIN, 5 * 1024 * 1024);
        assert_eq!(WS_FRAME_BUDGET_BYTES, 20 * 1024);
    }

    /// Each gate accepts a within-budget sample and rejects an
    /// over-budget sample.
    #[test]
    fn gates_accept_within_budget_reject_over_budget() {
        // S-01.
        assert!(ws_client_budget_met(WS_CLIENT_BUDGET));
        assert!(ws_client_budget_met(WS_CLIENT_BUDGET + 50));
        assert!(!ws_client_budget_met(WS_CLIENT_BUDGET - 1));
        // S-02.
        assert!(ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS - 0.1));
        assert!(!ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS));
        assert!(!ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS + 1.0));
        // S-03.
        assert!(tick_scale_budget_met(10.0, 50.0));
        assert!(!tick_scale_budget_met(10.0, 100.0));
        assert!(!tick_scale_budget_met(0.0, 1.0));
        // S-04.
        assert!(command_rate_budget_met(COMMAND_RATE_BUDGET_PER_SEC));
        assert!(!command_rate_budget_met(COMMAND_RATE_BUDGET_PER_SEC - 1));
        // S-05.
        assert!(event_log_budget_met(1_000_000));
        assert!(!event_log_budget_met(EVENT_LOG_BUDGET_BYTES_MIN));
        // S-06.
        assert!(ws_frame_budget_met(10_000));
        assert!(!ws_frame_budget_met(WS_FRAME_BUDGET_BYTES));
    }

    /// `BudgetReport` aggregates every gate.
    #[test]
    fn budget_report_aggregates_gates() {
        let passing = BudgetMeasurement {
            concurrent_clients: 150,
            handshake_ms: 2.5,
            tick_1k_ms: 10.0,
            tick_10k_ms: 50.0,
            commands_per_sec: 2_000,
            event_log_bytes_per_min: 1_000_000,
            ws_frame_bytes: 10_000,
        };
        let r = BudgetReport::from_measurement(passing);
        assert!(r.all_passed());

        // Failure on S-06.
        let mut bad = passing;
        bad.ws_frame_bytes = WS_FRAME_BUDGET_BYTES + 1;
        let r = BudgetReport::from_measurement(bad);
        assert!(!r.all_passed());
        assert!(!r.ws_frame);
        assert!(r.ws_client);
    }

    // NFR-S-03 — tick scaling 1k→10k citizens stays sub-linear at the spec
    // budget (ratio < 8.0). Gate accepts ~5× (rayon expectation), rejects at
    // the 8× budget bound, and rejects degenerate zero/negative baselines.
    #[test]
    fn nfr_s_03_tick_scale_budget_enforces_sublinear_scaling() {
        assert_eq!(TICK_SCALE_BUDGET, 8.0);
        // Expected rayon scaling (~5×) passes.
        assert!(tick_scale_budget_met(10.0, 50.0));
        // Just under the budget passes.
        assert!(tick_scale_budget_met(10.0, 79.9));
        // At or over the budget fails.
        assert!(!tick_scale_budget_met(10.0, 80.0));
        assert!(!tick_scale_budget_met(10.0, 100.0));
        // Non-positive baseline is treated as a failed measurement.
        assert!(!tick_scale_budget_met(0.0, 1.0));
        assert!(!tick_scale_budget_met(-1.0, 1.0));

        // End-to-end through the aggregate report: S-03 flag flips when the
        // 10k tick time crosses the budget.
        let mut m = BudgetMeasurement {
            concurrent_clients: 100,
            handshake_ms: 1.0,
            tick_1k_ms: 10.0,
            tick_10k_ms: 50.0,
            commands_per_sec: 1_500,
            event_log_bytes_per_min: 1_000_000,
            ws_frame_bytes: 10_000,
        };
        assert!(BudgetReport::from_measurement(m).tick_scale);
        m.tick_10k_ms = 80.0;
        assert!(!BudgetReport::from_measurement(m).tick_scale);
    }

    // NFR-S-06 — WS binary frame budget: a 1k-citizen snapshot frame must
    // stay under `WS_FRAME_BUDGET_BYTES` (20 KiB), the gate is strict (<,
    // not <=), and the report aggregates the S-06 gate honestly.
    #[test]
    fn ws_frame_budget_gate_bounds_one_k_snapshot_frames() {
        assert_eq!(WS_FRAME_BUDGET_BYTES, 20 * 1024);
        // Representative encoded frame sizes for a 1k-citizen snapshot.
        assert!(ws_frame_budget_met(12_288), "12 KiB frame within budget");
        assert!(!ws_frame_budget_met(WS_FRAME_BUDGET_BYTES), "strict < gate");
        assert!(!ws_frame_budget_met(usize::MAX));

        // S-06 failure is surfaced through the aggregate report.
        let measurement = BudgetMeasurement {
            ws_frame_bytes: WS_FRAME_BUDGET_BYTES + 1,
            ..BudgetMeasurement::default()
        };
        let report = BudgetReport::from_measurement(measurement);
        assert!(!report.ws_frame);
        assert!(!report.all_passed());

        let ok = BudgetMeasurement {
            ws_frame_bytes: WS_FRAME_BUDGET_BYTES - 1,
            ..BudgetMeasurement::default()
        };
        let report = BudgetReport::from_measurement(ok);
        assert!(report.ws_frame);
    }

    // NFR-S-02 — per-client connection overhead (WebSocket handshake +
    // initial snapshot) must stay strictly under WS_HANDSHAKE_BUDGET_MS.
    #[test]
    fn nfr_s_02_handshake_connection_overhead_budget_gate() {
        assert_eq!(WS_HANDSHAKE_BUDGET_MS, 5.0);
        // Strict less-than: just under passes, exactly at the budget fails.
        assert!(ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS - 0.001));
        assert!(!ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS));
        assert!(!ws_handshake_budget_met(WS_HANDSHAKE_BUDGET_MS + 4.5));

        // A within-budget handshake sample surfaces through the report.
        let within = BudgetMeasurement {
            handshake_ms: 2.5,
            ..BudgetMeasurement::default()
        };
        let report = BudgetReport::from_measurement(within);
        assert!(report.ws_handshake);
        assert!(
            !report.all_passed(),
            "other budgets are unmeasured defaults, so all_passed stays false"
        );

        // An over-budget handshake fails only the S-02 gate.
        let over = BudgetMeasurement {
            handshake_ms: WS_HANDSHAKE_BUDGET_MS,
            ..BudgetMeasurement::default()
        };
        let bad = BudgetReport::from_measurement(over);
        assert!(!bad.ws_handshake);
        assert!(bad.ws_client, "S-02 failure must not bleed into S-01");
    }
}