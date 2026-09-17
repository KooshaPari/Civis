//! FR-SOCI-003 — Insurgency starts above threshold.
//!
//! Insurgency SHALL start when aggregate stress exceeds the configured
//! threshold.

use civ_social::{InsurgencyConfig, InsurgencyTracker};
use civ_social::events::EventType;

#[test]
fn starts_above_threshold() {
    let mut tracker = InsurgencyTracker::new();
    let cfg = InsurgencyConfig::default(); // start = 800 bp

    let events = tracker.tick(900, &cfg, 10);
    assert!(tracker.active);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::InsurgencyStarted);
}

#[test]
fn no_start_below_threshold() {
    let mut tracker = InsurgencyTracker::new();
    let cfg = InsurgencyConfig::default();

    let events = tracker.tick(500, &cfg, 1);
    assert!(!tracker.active);
    assert!(events.is_empty());
}

#[test]
fn ends_below_hysteresis() {
    let mut tracker = InsurgencyTracker { active: true };
    let cfg = InsurgencyConfig::default(); // end = 400 bp

    let events = tracker.tick(300, &cfg, 20);
    assert!(!tracker.active);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::InsurgencyEnded);
}

#[test]
fn stays_active_in_hysteresis_band() {
    let mut tracker = InsurgencyTracker { active: true };
    let cfg = InsurgencyConfig::default();

    // 600 is between end (400) and start (800) => stays active, no events.
    let events = tracker.tick(600, &cfg, 5);
    assert!(tracker.active);
    assert!(events.is_empty());
}

#[test]
fn custom_thresholds() {
    let mut tracker = InsurgencyTracker::new();
    let cfg = InsurgencyConfig {
        start_threshold_bp: 500,
        end_threshold_bp: 200,
    };

    // 550 >= 500 => starts
    let events = tracker.tick(550, &cfg, 1);
    assert!(tracker.active);
    assert_eq!(events.len(), 1);

    // 250 is between 200 and 500 => stays active
    let events = tracker.tick(250, &cfg, 2);
    assert!(tracker.active);
    assert!(events.is_empty());

    // 150 <= 200 => ends
    let events = tracker.tick(150, &cfg, 3);
    assert!(!tracker.active);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, EventType::InsurgencyEnded);
}
