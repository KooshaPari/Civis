//! FR-CLIM-003 — Threshold crossing events.
//!
//! The engine SHALL emit `climate.threshold.crossed.v1` when temperature
//! crosses a defined level. Thresholds are checked every tick and events
//! are emitted exactly once per crossing (hysteresis: only upward crossings).

use serde::{Deserialize, Serialize};

/// Named temperature threshold levels (°C above baseline = anomaly).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThresholdLevel {
    /// +1.0 °C — Paris Agreement aspirational guardrail.
    Paris,
    /// +1.5 °C — Paris Agreement primary target.
    ParisTarget,
    /// +2.0 °C — Paris Agreement upper bound.
    ParisUpper,
    /// +3.0 °C — Severe impacts zone.
    Severe,
    /// +4.0 °C — Catastrophic / civilisational risk.
    Catastrophic,
    /// Custom threshold with a specific anomaly value.
    Custom(f64),
}

impl ThresholdLevel {
    /// The anomaly value in °C for this threshold.
    pub fn anomaly_c(&self) -> f64 {
        match self {
            Self::Paris => 1.0,
            Self::ParisTarget => 1.5,
            Self::ParisUpper => 2.0,
            Self::Severe => 3.0,
            Self::Catastrophic => 4.0,
            Self::Custom(v) => *v,
        }
    }

    /// All standard thresholds in ascending order.
    pub fn standard_levels() -> &'static [ThresholdLevel] {
        &[
            ThresholdLevel::Paris,
            ThresholdLevel::ParisTarget,
            ThresholdLevel::ParisUpper,
            ThresholdLevel::Severe,
            ThresholdLevel::Catastrophic,
        ]
    }
}

/// An emitted event when a threshold is crossed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdEvent {
    /// The event type identifier (matches spec `climate.threshold.crossed.v1`).
    pub event_type: String,
    /// Which threshold was crossed.
    pub level: ThresholdLevel,
    /// The tick at which the crossing occurred.
    pub tick: u64,
    /// Temperature anomaly at the moment of crossing (°C).
    pub anomaly_c: f64,
    /// Absolute mean temperature at crossing (°C).
    pub mean_temp_c: f64,
}

/// Tracks which thresholds have been crossed and emits events on new crossings.
#[derive(Debug, Clone)]
pub struct ThresholdTracker {
    /// Set of standard levels already crossed (indexed by anomaly value).
    crossed_anomalies: Vec<f64>,
    /// Pending events produced during the last `check` call.
    pending_events: Vec<ThresholdEvent>,
}

impl ThresholdTracker {
    /// Create an empty tracker (no thresholds crossed yet).
    pub fn new() -> Self {
        Self {
            crossed_anomalies: Vec::new(),
            pending_events: Vec::new(),
        }
    }

    /// Check temperature against all standard thresholds and emit events
    /// for any newly crossed levels.
    ///
    /// `anomaly_c`: current temperature anomaly above baseline.
    /// `mean_temp_c`: current absolute temperature.
    /// `tick`: current simulation tick.
    pub fn check(
        &mut self,
        anomaly_c: f64,
        mean_temp_c: f64,
        tick: u64,
    ) -> Vec<ThresholdEvent> {
        self.pending_events.clear();

        for level in ThresholdLevel::standard_levels() {
            let threshold = level.anomaly_c();
            let already_crossed = self.crossed_anomalies.iter().any(|a| (*a - threshold).abs() < 1e-10);
            if !already_crossed && anomaly_c >= threshold {
                self.crossed_anomalies.push(threshold);
                self.pending_events.push(ThresholdEvent {
                    event_type: "climate.threshold.crossed.v1".to_string(),
                    level: *level,
                    tick,
                    anomaly_c,
                    mean_temp_c,
                });
            }
        }

        self.pending_events.clone()
    }

    /// Check a custom threshold.
    pub fn check_custom(
        &mut self,
        custom_threshold: f64,
        anomaly_c: f64,
        mean_temp_c: f64,
        tick: u64,
    ) -> Option<ThresholdEvent> {
        let already_crossed = self
            .crossed_anomalies
            .iter()
            .any(|a| (*a - custom_threshold).abs() < 1e-10);
        if !already_crossed && anomaly_c >= custom_threshold {
            self.crossed_anomalies.push(custom_threshold);
            let event = ThresholdEvent {
                event_type: "climate.threshold.crossed.v1".to_string(),
                level: ThresholdLevel::Custom(custom_threshold),
                tick,
                anomaly_c,
                mean_temp_c,
            };
            self.pending_events.push(event.clone());
            Some(event)
        } else {
            None
        }
    }

    /// Check an individual level and emit if newly crossed.
    pub fn check_level(
        &mut self,
        level: ThresholdLevel,
        anomaly_c: f64,
        mean_temp_c: f64,
        tick: u64,
    ) -> Option<ThresholdEvent> {
        let threshold = level.anomaly_c();
        let already_crossed = self
            .crossed_anomalies
            .iter()
            .any(|a| (*a - threshold).abs() < 1e-10);
        if !already_crossed && anomaly_c >= threshold {
            self.crossed_anomalies.push(threshold);
            let event = ThresholdEvent {
                event_type: "climate.threshold.crossed.v1".to_string(),
                level,
                tick,
                anomaly_c,
                mean_temp_c,
            };
            self.pending_events.push(event.clone());
            Some(event)
        } else {
            None
        }
    }

    /// How many thresholds have been crossed so far.
    pub fn crossed_count(&self) -> usize {
        self.crossed_anomalies.len()
    }

    /// Whether a specific level has been crossed.
    pub fn is_crossed(&self, level: ThresholdLevel) -> bool {
        let threshold = level.anomaly_c();
        self.crossed_anomalies
            .iter()
            .any(|a| (*a - threshold).abs() < 1e-10)
    }

    /// Access pending events from the last `check` call.
    pub fn pending_events(&self) -> &[ThresholdEvent] {
        &self.pending_events
    }
}

impl Default for ThresholdTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_event_below_threshold() {
        let mut tracker = ThresholdTracker::new();
        let events = tracker.check(0.5, 14.5, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn paris_event_on_first_crossing() {
        let mut tracker = ThresholdTracker::new();
        let events = tracker.check(1.0, 15.0, 10);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, ThresholdLevel::Paris);
        assert_eq!(events[0].event_type, "climate.threshold.crossed.v1");
        assert_eq!(events[0].tick, 10);
    }

    #[test]
    fn no_duplicate_event_on_same_level() {
        let mut tracker = ThresholdTracker::new();
        tracker.check(1.0, 15.0, 10);
        let events = tracker.check(1.2, 15.2, 11);
        assert!(events.is_empty(), "should not re-emit Paris event");
    }

    #[test]
    fn multiple_levels_crossed_at_once() {
        let mut tracker = ThresholdTracker::new();
        let events = tracker.check(3.5, 17.5, 50);
        // Should cross Paris (1.0), ParisTarget (1.5), ParisUpper (2.0), Severe (3.0)
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn sequential_crossing() {
        let mut tracker = ThresholdTracker::new();
        tracker.check(0.8, 14.8, 1);
        let e1 = tracker.check(1.1, 15.1, 2);
        assert_eq!(e1.len(), 1);
        assert_eq!(e1[0].level, ThresholdLevel::Paris);
        let e2 = tracker.check(1.6, 15.6, 3);
        assert_eq!(e2.len(), 1);
        assert_eq!(e2[0].level, ThresholdLevel::ParisTarget);
        assert_eq!(tracker.crossed_count(), 2);
    }

    #[test]
    fn custom_threshold() {
        let mut tracker = ThresholdTracker::new();
        let event = tracker.check_custom(2.5, 2.5, 16.5, 20);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.level, ThresholdLevel::Custom(2.5));
    }

    #[test]
    fn is_crossed_reflects_state() {
        let mut tracker = ThresholdTracker::new();
        assert!(!tracker.is_crossed(ThresholdLevel::Paris));
        tracker.check(1.0, 15.0, 1);
        assert!(tracker.is_crossed(ThresholdLevel::Paris));
        assert!(!tracker.is_crossed(ThresholdLevel::Catastrophic));
    }
}
