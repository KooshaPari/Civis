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
