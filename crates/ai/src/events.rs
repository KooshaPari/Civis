//! FR-AI-006 — AI decision event emission.
//!
//! AI decision events SHALL be emitted for post-run analysis and replay.
//! Each event records the leader, chosen action, tick, and the full
//! utility score breakdown.

use serde::{Deserialize, Serialize};

/// A single AI decision event emitted each time an AI leader picks an action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionEvent {
    /// Simulation tick when the decision was made.
    pub tick: u64,
    /// Leader identifier.
    pub leader_id: String,
    /// The action that was chosen.
    pub chosen_action: String,
    /// Utility score of the chosen action.
    pub score: f64,
    /// Total number of candidate actions considered.
    pub candidates_count: usize,
}

/// Log of all decision events for a single tick.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DecisionLog {
    /// All events emitted this tick.
    pub events: Vec<DecisionEvent>,
}

impl DecisionLog {
    /// Record a new decision event.
    pub fn emit(&mut self, event: DecisionEvent) {
        self.events.push(event);
    }

    /// Number of events in the log.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the log is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Filter events for a specific leader.
    #[must_use]
    pub fn for_leader(&self, leader_id: &str) -> Vec<&DecisionEvent> {
        self.events.iter().filter(|e| e.leader_id == leader_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emit_adds_event() {
        let mut log = DecisionLog::default();
        log.emit(DecisionEvent {
            tick: 1,
            leader_id: "leader_1".into(),
            chosen_action: "build".into(),
            score: 0.85,
            candidates_count: 4,
        });
        assert_eq!(log.len(), 1);
        assert!(!log.is_empty());
    }

    #[test]
    fn for_leader_filters() {
        let mut log = DecisionLog::default();
        log.emit(DecisionEvent { tick: 1, leader_id: "a".into(), chosen_action: "x".into(), score: 1.0, candidates_count: 1 });
        log.emit(DecisionEvent { tick: 1, leader_id: "b".into(), chosen_action: "y".into(), score: 0.5, candidates_count: 1 });
        assert_eq!(log.for_leader("a").len(), 1);
        assert_eq!(log.for_leader("c").len(), 0);
    }
}
