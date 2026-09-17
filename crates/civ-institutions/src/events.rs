//! FR-INST-003 — Threshold events for institutional capture.
//!
//! The engine SHALL emit `institution.capture.threshold.v1` when the
//! institutional capture score crosses 0.75 (7_500 basis points).
//!
//! Events are modeled with one-shot semantics: each threshold crossing
//! emits exactly once. The owning simulation tracks which thresholds
//! have already fired.

use serde::{Deserialize, Serialize};

/// Event type identifier for institutional capture threshold crossing.
pub const CAPTURE_THRESHOLD_EVENT: &str = "institution.capture.threshold.v1";

/// Event emitted when the institutional capture score crosses the
/// configured threshold (default 0.75).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureThresholdEvent {
    /// Tick at which the threshold was crossed.
    pub tick: u64,
    /// Civilization identifier.
    pub civilization_id: u32,
    /// Capture score at the moment of the event (basis points).
    pub capture_score_bp: i64,
    /// The threshold that was crossed (basis points).
    pub threshold_bp: i64,
    /// Event type identifier string.
    pub event_type: String,
}

impl CaptureThresholdEvent {
    /// Creates a new threshold event.
    pub fn new(tick: u64, civilization_id: u32, capture_score_bp: i64, threshold_bp: i64) -> Self {
        Self {
            tick,
            civilization_id,
            capture_score_bp,
            threshold_bp,
            event_type: CAPTURE_THRESHOLD_EVENT.to_string(),
        }
    }
}

/// Evaluates whether a capture threshold crossing should emit an event.
///
/// Returns `Some(CaptureThresholdEvent)` when `capture_bp >= threshold_bp`
/// and the threshold has not already been triggered (checked via
/// `already_triggered`). Returns `None` otherwise.
pub fn check_capture_threshold(
    tick: u64,
    civilization_id: u32,
    capture_bp: i64,
    threshold_bp: i64,
    already_triggered: bool,
) -> Option<CaptureThresholdEvent> {
    if already_triggered {
        return None;
    }
    if capture_bp >= threshold_bp {
        Some(CaptureThresholdEvent::new(
            tick,
            civilization_id,
            capture_bp,
            threshold_bp,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_emitted_when_threshold_crossed() {
        let event = check_capture_threshold(100, 1, 8_000, 7_500, false);
        assert!(event.is_some());
        let e = event.unwrap();
        assert_eq!(e.event_type, CAPTURE_THRESHOLD_EVENT);
        assert_eq!(e.tick, 100);
    }

    #[test]
    fn no_event_below_threshold() {
        let event = check_capture_threshold(100, 1, 5_000, 7_500, false);
        assert!(event.is_none());
    }

    #[test]
    fn no_event_when_already_triggered() {
        let event = check_capture_threshold(100, 1, 8_000, 7_500, true);
        assert!(event.is_none());
    }

    #[test]
    fn event_emitted_at_exact_threshold() {
        let event = check_capture_threshold(50, 2, 7_500, 7_500, false);
        assert!(event.is_some());
    }
}
