//! Era transition history and chronicle (FR-ERA).
//!
//! Records emergent age advances per faction when threshold evaluation
//! detects a strictly higher [`super::era::CivAge`].

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::era::CivAge;

/// Maximum chronicle lines retained in memory.
pub const ERA_CHRONICLE_MAX_LEN: usize = 200;

/// A single emergent era transition for one faction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraTransition {
    pub tick: u64,
    pub faction_id: u32,
    pub from: CivAge,
    pub to: CivAge,
}

/// Bounded chronicle of emergent era advances.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EraHistory {
    transitions: VecDeque<EraTransition>,
    chronicle: VecDeque<String>,
}

impl EraHistory {
    /// Record an era advance when `to` is strictly after `from`.
    pub fn record_advance(&mut self, tick: u64, faction_id: u32, from: CivAge, to: CivAge) {
        if to <= from {
            return;
        }
        self.transitions.push_back(EraTransition {
            tick,
            faction_id,
            from,
            to,
        });
        while self.transitions.len() > ERA_CHRONICLE_MAX_LEN {
            self.transitions.pop_front();
        }

        let line = format!(
            "tick {tick}: faction {faction_id} entered the {} age (from {})",
            to.as_str(),
            from.as_str()
        );
        self.chronicle.push_back(line);
        while self.chronicle.len() > ERA_CHRONICLE_MAX_LEN {
            self.chronicle.pop_front();
        }
    }

    /// Recorded transitions (oldest first).
    #[must_use]
    pub fn transitions(&self) -> Vec<EraTransition> {
        self.transitions.iter().cloned().collect()
    }

    /// Chronicle lines for HUD / replay surfaces.
    #[must_use]
    pub fn chronicle(&self) -> Vec<String> {
        self.chronicle.iter().cloned().collect()
    }

    /// Number of era transitions recorded.
    #[must_use]
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_only_forward_transitions_and_chronicle_lines() {
        let mut history = EraHistory::default();
        history.record_advance(3, 7, CivAge::Stone, CivAge::Bronze);
        history.record_advance(4, 7, CivAge::Bronze, CivAge::Bronze);

        assert_eq!(history.transition_count(), 1);
        assert_eq!(history.transitions()[0].faction_id, 7);
        assert_eq!(
            history.chronicle(),
            vec!["tick 3: faction 7 entered the Bronze age (from Stone)".to_string()]
        );
    }

    #[test]
    fn retains_only_the_bounded_tail() {
        let mut history = EraHistory::default();
        for tick in 0..=ERA_CHRONICLE_MAX_LEN {
            history.record_advance(tick as u64, 1, CivAge::Stone, CivAge::Bronze);
        }

        assert_eq!(history.transition_count(), ERA_CHRONICLE_MAX_LEN);
        assert_eq!(history.transitions()[0].tick, 1);
        assert!(history.chronicle()[0].starts_with("tick 1:"));
    }

    #[test]
    fn round_trips_through_serde() {
        let mut history = EraHistory::default();
        history.record_advance(9, 2, CivAge::Bronze, CivAge::Iron);
        let encoded = serde_json::to_vec(&history).expect("serialize history");
        let decoded: EraHistory = serde_json::from_slice(&encoded).expect("deserialize history");
        assert_eq!(decoded, history);
    }
}

use std::collections::HashMap;

/// Type of historical event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    War,
    Trade,
    Religion,
    Culture,
    Disaster,
    Political,
    Other,
}

/// A single historical event with tick, type, description, participants, consequences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalEvent {
    pub tick: u64,
    pub event_type: EventType,
    pub description: String,
    pub participants: Vec<u32>,
    pub consequences: Vec<String>,
}

impl Default for HistoricalEvent {
    fn default() -> Self {
        Self {
            tick: 0,
            event_type: EventType::Other,
            description: String::new(),
            participants: Vec::new(),
            consequences: Vec::new(),
        }
    }
}

/// Append-only log of historical events with summaries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryLog {
    pub events: Vec<HistoricalEvent>,
    pub summaries: HashMap<String, String>,
    pub max_events: usize,
}

impl HistoryLog {
    /// Create a new HistoryLog with a capacity limit.
    #[must_use]
    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            summaries: HashMap::new(),
            max_events,
        }
    }

    /// Record a historical event, evicting oldest if over capacity.
    pub fn record_event(&mut self, event: HistoricalEvent) {
        self.events.push(event);
        while self.events.len() > self.max_events && self.max_events > 0 {
            self.events.remove(0);
        }
    }

    /// Summarize all events within a tick range into a single string.
    #[must_use]
    pub fn summarize_era(&self, era_name: &str, start_tick: u64, end_tick: u64) -> String {
        let count = self
            .events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .count();
        format!(
            "Era '{}' (ticks {}-{}): {} events recorded",
            era_name, start_tick, end_tick, count
        )
    }

    /// Query events by type within a tick range.
    #[must_use]
    pub fn query_events(
        &self,
        event_type: EventType,
        start_tick: u64,
        end_tick: u64,
    ) -> Vec<&HistoricalEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type && e.tick >= start_tick && e.tick <= end_tick)
            .collect()
    }

    /// Compute a normalized impact score for events in a tick range.
    /// Each event contributes based on its consequence count.
    #[must_use]
    pub fn compute_historical_impact(&self, start_tick: u64, end_tick: u64) -> f32 {
        let total_consequences: usize = self
            .events
            .iter()
            .filter(|e| e.tick >= start_tick && e.tick <= end_tick)
            .map(|e| e.consequences.len())
            .sum();
        (total_consequences as f32 / 100.0).min(1.0)
    }
}

/// Advance a history log by one tick (currently a no-op placeholder).
#[must_use]
pub fn tick_history(log: &HistoryLog, _tick: u64) -> HistoryLog {
    log.clone()
}

#[cfg(test)]
mod history_extended_tests {
    use super::*;

    #[test]
    fn history_log_with_capacity() {
        let log = HistoryLog::with_capacity(5);
        assert_eq!(log.max_events, 5);
        assert!(log.events.is_empty());
    }

    #[test]
    fn history_log_record_event() {
        let mut log = HistoryLog::with_capacity(10);
        let event = HistoricalEvent {
            tick: 1,
            event_type: EventType::War,
            description: "Battle of X".into(),
            participants: vec![1, 2],
            consequences: vec!["territory change".into()],
        };
        log.record_event(event);
        assert_eq!(log.events.len(), 1);
    }

    #[test]
    fn history_log_evicts_oldest() {
        let mut log = HistoryLog::with_capacity(3);
        for i in 0..5 {
            log.record_event(HistoricalEvent {
                tick: i,
                ..HistoricalEvent::default()
            });
        }
        assert_eq!(log.events.len(), 3);
        assert_eq!(log.events[0].tick, 2);
    }

    #[test]
    fn summarize_era_count() {
        let mut log = HistoryLog::with_capacity(100);
        for i in 0..5 {
            log.record_event(HistoricalEvent {
                tick: i,
                ..HistoricalEvent::default()
            });
        }
        let summary = log.summarize_era("Test Era", 0, 4);
        assert!(summary.contains("5 events"));
    }

    #[test]
    fn query_events_by_type() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 1,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 2,
            event_type: EventType::Trade,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 3,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        let wars = log.query_events(EventType::War, 0, 10);
        assert_eq!(wars.len(), 2);
    }

    #[test]
    fn query_events_tick_range() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 5,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        log.record_event(HistoricalEvent {
            tick: 15,
            event_type: EventType::War,
            ..HistoricalEvent::default()
        });
        let wars = log.query_events(EventType::War, 0, 10);
        assert_eq!(wars.len(), 1);
    }

    #[test]
    fn compute_historical_impact_zero() {
        let log = HistoryLog::with_capacity(100);
        assert!((log.compute_historical_impact(0, 100) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_historical_impact_with_events() {
        let mut log = HistoryLog::with_capacity(100);
        log.record_event(HistoricalEvent {
            tick: 1,
            consequences: vec!["a".into(), "b".into(), "c".into()],
            ..HistoricalEvent::default()
        });
        let impact = log.compute_historical_impact(0, 10);
        assert!(impact > 0.0);
    }

    #[test]
    fn tick_history_clones() {
        let log = HistoryLog::with_capacity(10);
        let ticked = tick_history(&log, 100);
        assert_eq!(ticked.max_events, log.max_events);
    }

    #[test]
    fn historical_event_default() {
        let e = HistoricalEvent::default();
        assert_eq!(e.event_type, EventType::Other);
        assert!(e.description.is_empty());
    }
}
