//! FR-CLIM-003 — Integration test: threshold crossing events.
//!
//! Verifies `climate.threshold.crossed.v1` is emitted exactly once per level crossing.

use civ_climate::events::{ThresholdLevel, ThresholdTracker};

/// FR-CLIM-003: Threshold event is emitted when temperature crosses a defined level.
#[test]
fn threshold_event_emitted() {
    let mut tracker = ThresholdTracker::new();
    let events = tracker.check(1.0, 15.0, 10);

    assert_eq!(events.len(), 1, "Should emit exactly one event for Paris crossing");
    assert_eq!(events[0].event_type, "climate.threshold.crossed.v1");
    assert_eq!(events[0].level, ThresholdLevel::Paris);
    assert_eq!(events[0].tick, 10);
    assert!((events[0].anomaly_c - 1.0).abs() < 1e-10);
}

/// FR-CLIM-003: Events are only emitted once per threshold level.
#[test]
fn no_duplicate_events() {
    let mut tracker = ThresholdTracker::new();
    tracker.check(1.5, 15.5, 10);
    let events2 = tracker.check(1.6, 15.6, 11);
    let events3 = tracker.check(2.0, 16.0, 12);

    assert!(events2.is_empty(), "Paris and ParisTarget already crossed");
    assert_eq!(events3.len(), 1, "Only ParisUpper is newly crossed");
    assert_eq!(events3[0].level, ThresholdLevel::ParisUpper);
}

/// FR-CLIM-003: Multiple thresholds crossed simultaneously emit multiple events.
#[test]
fn simultaneous_crossings() {
    let mut tracker = ThresholdTracker::new();
    let events = tracker.check(3.5, 17.5, 50);

    // Should cross: Paris (1.0), ParisTarget (1.5), ParisUpper (2.0), Severe (3.0)
    assert_eq!(events.len(), 4, "Should emit 4 events for a jump to +3.5 °C");
    assert!(events.iter().any(|e| e.level == ThresholdLevel::Paris));
    assert!(events.iter().any(|e| e.level == ThresholdLevel::Severe));
    assert_eq!(tracker.crossed_count(), 4);
}

/// FR-CLIM-003: Below threshold produces no events.
#[test]
fn no_event_below_threshold() {
    let mut tracker = ThresholdTracker::new();
    let events = tracker.check(0.5, 14.5, 1);
    assert!(events.is_empty());
}

/// FR-CLIM-003: Sequential threshold crossing emits events in order.
#[test]
fn sequential_crossing_order() {
    let mut tracker = ThresholdTracker::new();

    let e1 = tracker.check(1.1, 15.1, 1);
    assert_eq!(e1.len(), 1);
    assert_eq!(e1[0].level, ThresholdLevel::Paris);

    let e2 = tracker.check(1.6, 15.6, 2);
    assert_eq!(e2.len(), 1);
    assert_eq!(e2[0].level, ThresholdLevel::ParisTarget);

    let e3 = tracker.check(2.1, 16.1, 3);
    assert_eq!(e3.len(), 1);
    assert_eq!(e3[0].level, ThresholdLevel::ParisUpper);
}

/// FR-CLIM-003: Custom thresholds can be tracked.
#[test]
fn custom_threshold() {
    let mut tracker = ThresholdTracker::new();
    let event = tracker.check_custom(2.5, 2.5, 16.5, 20);
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.level, ThresholdLevel::Custom(2.5));
    assert_eq!(event.event_type, "climate.threshold.crossed.v1");
}
