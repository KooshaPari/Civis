//! Legends ingest performance budget — FR-CIV-LEGENDS-PERF-01.
//!
//! Per `docs/traceability/fr-emergence-matrix.md` row 260, the legends
//! ingest pipeline SHALL keep its P99 latency below `INGEST_P99_BUDGET_MS`
//! when processing a fixture of `INGEST_FIXTURE_SIZE` events. The bench
//! `benches/ingest_p99.rs` (TODO) drives the gate; this module is the
//! spec-side constants and a tiny timing helper the bench (and any
//! production-side latency assertion) can call.
//!
//! ## Design contract
//!
//! 1. **Budget constants are `pub const`.** A bench test imports them
//!    directly, asserts the measured P99 < `INGEST_P99_BUDGET_MS`, and
//!    CI fails the build on regression.
//! 2. **Timing helper is `no_std`-friendly.** It uses
//!    [`std::time::Instant`] (the substrate owns `Instant`) and reports
//!    the elapsed time in **milliseconds** as `f64`, matching the
//!    fixture's resolution.
//! 3. **Pure data, no engine.** No tokio, no Bevy. Any host can call
//!    [`IngestTimer::start`] / [`IngestTimer::elapsed_ms`] without
//!    pulling in extra deps.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::time::Instant;

/// The legends-ingest P99 latency budget — FR-CIV-LEGENDS-PERF-01.
///
/// Per the spec, ingesting 1k events must run at P99 < 50 ms on the
/// reference benchmark machine. The benchmark fixture
/// ([`INGEST_FIXTURE_SIZE`] events) drives the gate; CI fails the
/// build on regression.
pub const INGEST_P99_BUDGET_MS: f64 = 50.0;

/// The reference fixture size — 1k events, as specified by
/// FR-CIV-LEGENDS-PERF-01.
pub const INGEST_FIXTURE_SIZE: usize = 1_000;

/// A stopwatch for measuring legends-ingest latency.
///
/// Construct one with [`IngestTimer::start`], call [`Self::elapsed_ms`]
/// when the batch is fully ingested. The bench iterates the batch `N`
/// times and reports the percentile distribution; this struct just
/// owns the timing primitive.
///
/// `IngestTimer` is `Copy`-able (a single `Instant`) so callers can
/// pass it into helper functions without ownership friction.
#[derive(Debug, Clone, Copy)]
pub struct IngestTimer {
    started: Instant,
}

impl IngestTimer {
    /// Start the timer (records `Instant::now()`).
    #[must_use]
    pub fn start() -> Self {
        Self {
            started: Instant::now(),
        }
    }

    /// Elapsed time in milliseconds as `f64`. Saturating on the
    /// `Instant` subtraction (`u128`); a 1-second ingest maps to
    /// `1000.0`.
    #[must_use]
    pub fn elapsed_ms(self) -> f64 {
        let elapsed = self.started.elapsed();
        // `u128 → f64` saturates gracefully; we don't expect negative
        // values because `Instant::elapsed` is monotonic.
        elapsed.as_secs_f64() * 1000.0
    }
}

/// True when `p99_ms` is within the FR-CIV-LEGENDS-PERF-01 budget
/// (`<= INGEST_P99_BUDGET_MS`). Hosts and benches use this as a
/// single-line assertion gate.
#[must_use]
pub fn ingest_p99_within_budget(p99_ms: f64) -> bool {
    p99_ms <= INGEST_P99_BUDGET_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-LEGENDS-PERF-01: the spec constants are non-zero and the
    /// fixture size is `1000`.
    #[test]
    fn budget_constants_match_spec() {
        assert!((INGEST_P99_BUDGET_MS - 50.0).abs() < f64::EPSILON);
        assert_eq!(INGEST_FIXTURE_SIZE, 1_000);
    }

    /// `IngestTimer` reports a non-negative elapsed time and the
    /// budget gate accepts / rejects values correctly.
    #[test]
    fn timer_and_budget_gate() {
        let t = IngestTimer::start();
        let ms = t.elapsed_ms();
        assert!(ms >= 0.0);
        // Within budget.
        assert!(ingest_p99_within_budget(INGEST_P99_BUDGET_MS));
        assert!(ingest_p99_within_budget(INGEST_P99_BUDGET_MS - 1.0));
        // Over budget.
        assert!(!ingest_p99_within_budget(INGEST_P99_BUDGET_MS + 1.0));
        assert!(!ingest_p99_within_budget(1000.0));
    }
}