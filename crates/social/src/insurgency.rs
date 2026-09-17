//! FR-SOCI-003 / FR-SOCI-004 — Insurgency tracking.
//!
//! Insurgency starts when aggregate stress exceeds a configured threshold
//! and ends when it drops below a lower threshold (hysteresis).
//! Lifecycle events are emitted on transitions.

use crate::events::{Event, EventType};
use serde::{Deserialize, Serialize};

/// Default stress threshold for insurgency start (basis points).
pub const DEFAULT_START_THRESHOLD_BP: i64 = 800;

/// Default stress threshold for insurgency end (basis points, hysteresis).
pub const DEFAULT_END_THRESHOLD_BP: i64 = 400;

/// Configuration for insurgency thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsurgencyConfig {
    /// Aggregate stress at or above which insurgency starts (bp).
    pub start_threshold_bp: i64,
    /// Aggregate stress at or below which insurgency ends (bp).
    pub end_threshold_bp: i64,
}

impl Default for InsurgencyConfig {
    fn default() -> Self {
        Self {
            start_threshold_bp: DEFAULT_START_THRESHOLD_BP,
            end_threshold_bp: DEFAULT_END_THRESHOLD_BP,
        }
    }
}

/// Tracks insurgency state for a civilization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsurgencyTracker {
    /// Whether insurgency is currently active.
    pub active: bool,
}

impl InsurgencyTracker {
    /// Create a new tracker (no insurgency).
    pub fn new() -> Self {
        Self { active: false }
    }

    /// Evaluate aggregate stress and return any events to emit.
    pub fn tick(
        &mut self,
        aggregate_stress_bp: i64,
        config: &InsurgencyConfig,
        tick: u64,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        if !self.active && aggregate_stress_bp >= config.start_threshold_bp {
            self.active = true;
            events.push(Event {
                event_type: EventType::InsurgencyStarted,
                tick,
            });
        } else if self.active && aggregate_stress_bp <= config.end_threshold_bp {
            self.active = false;
            events.push(Event {
                event_type: EventType::InsurgencyEnded,
                tick,
            });
        }
        events
    }
}

impl Default for InsurgencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_above_threshold() {
        let mut tracker = InsurgencyTracker::new();
        let cfg = InsurgencyConfig::default();
        let events = tracker.tick(900, &cfg, 10);
        assert!(tracker.active);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::InsurgencyStarted);
        assert_eq!(events[0].tick, 10);
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
        let mut tracker = InsurgencyTracker {
            active: true,
        };
        let cfg = InsurgencyConfig::default();
        let events = tracker.tick(300, &cfg, 20);
        assert!(!tracker.active);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::InsurgencyEnded);
    }

    #[test]
    fn stays_active_in_hysteresis_band() {
        let mut tracker = InsurgencyTracker {
            active: true,
        };
        let cfg = InsurgencyConfig::default();
        // 600 is between end (400) and start (800) => stays active
        let events = tracker.tick(600, &cfg, 5);
        assert!(tracker.active);
        assert!(events.is_empty());
    }
}
