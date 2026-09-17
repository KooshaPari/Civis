//! FR-SOCI-004 — Insurgency lifecycle events.
//!
//! The engine SHALL emit `social.insurgency.started.v1` and
//! `social.insurgency.ended.v1`.

use civ_social::{InsurgencyConfig, InsurgencyTracker};
use civ_social::events::{Event, EventType};

#[test]
fn insurgency_lifecycle_events() {
    let mut tracker = InsurgencyTracker::new();
    let cfg = InsurgencyConfig::default();

    // Phase 1: No insurgency => no events
    let events = tracker.tick(500, &cfg, 1);
    assert!(events.is_empty());

    // Phase 2: Stress spikes => insurgency starts
    let events = tracker.tick(900, &cfg, 5);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::InsurgencyStarted);
    assert_eq!(events[0].tick, 5);
    assert!(tracker.active);

    // Phase 3: Stress in hysteresis band => no new events
    let events = tracker.tick(600, &cfg, 10);
    assert!(events.is_empty());
    assert!(tracker.active);

    // Phase 4: Stress drops => insurgency ends
    let events = tracker.tick(300, &cfg, 15);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::InsurgencyEnded);
    assert_eq!(events[0].tick, 15);
    assert!(!tracker.active);
}

#[test]
fn event_serializes() {
    let event = Event {
        event_type: EventType::InsurgencyStarted,
        tick: 42,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("InsurgencyStarted"));
    assert!(json.contains("42"));
}
