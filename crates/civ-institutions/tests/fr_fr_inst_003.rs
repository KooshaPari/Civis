//! FR-INST-003 — The engine SHALL emit `institution.capture.threshold.v1`
//! when capture crosses 0.75.

use civ_institutions::{check_capture_threshold, CAPTURE_THRESHOLD_BP};

/// Event emitted when threshold is crossed.
#[test]
fn inst_events_capture_threshold_event() {
    let event = check_capture_threshold(100, 1, CAPTURE_THRESHOLD_BP, CAPTURE_THRESHOLD_BP, false);
    assert!(event.is_some(), "event must be emitted at threshold");
    let e = event.unwrap();
    assert_eq!(e.event_type, "institution.capture.threshold.v1");
}

/// Event not emitted below threshold.
#[test]
fn no_event_below_threshold() {
    let event = check_capture_threshold(100, 1, 5_000, CAPTURE_THRESHOLD_BP, false);
    assert!(event.is_none());
}

/// One-shot semantics: no duplicate event.
#[test]
fn one_shot_no_duplicate() {
    let event1 = check_capture_threshold(100, 1, 8_000, CAPTURE_THRESHOLD_BP, false);
    assert!(event1.is_some());
    let event2 = check_capture_threshold(101, 1, 8_000, CAPTURE_THRESHOLD_BP, true);
    assert!(event2.is_none(), "must not emit when already triggered");
}

/// Event carries correct tick and civilization id.
#[test]
fn event_carries_metadata() {
    let event = check_capture_threshold(42, 7, 9_000, CAPTURE_THRESHOLD_BP, false).unwrap();
    assert_eq!(event.tick, 42);
    assert_eq!(event.civilization_id, 7);
    assert_eq!(event.capture_score_bp, 9_000);
}

/// Event emitted above threshold (not just at).
#[test]
fn event_above_threshold() {
    let event = check_capture_threshold(10, 1, 10_000, CAPTURE_THRESHOLD_BP, false);
    assert!(event.is_some());
}

/// Default threshold is 7500 bp (0.75).
#[test]
fn default_threshold_is_075() {
    assert_eq!(CAPTURE_THRESHOLD_BP, 7_500);
}
