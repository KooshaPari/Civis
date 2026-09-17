//! FR-ECON-008 — district collapse after consecutive deficit ticks.
//!
//! A district in Joule deficit for N consecutive ticks emits a collapse event.
//! The configurable threshold is the number of consecutive deficit ticks.
//!
//! All math is integer-saturating. No floats accumulate across calls.

use serde::{Deserialize, Serialize};

/// Default consecutive deficit ticks before collapse.
pub const DEFAULT_DEFICIT_TICKS: u32 = 3;

/// A district's energy state for collapse tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictEnergyState {
    /// District identifier.
    pub district_id: u32,
    /// Current Joule balance.
    pub balance_joules: i64,
    /// Number of consecutive ticks in deficit.
    pub consecutive_deficit_ticks: u32,
}

impl DistrictEnergyState {
    /// Create a new district with given balance.
    #[must_use]
    pub fn new(district_id: u32, balance_joules: i64) -> Self {
        Self {
            district_id,
            balance_joules,
            consecutive_deficit_ticks: 0,
        }
    }
}

/// Result of a collapse check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollapseCheck {
    /// Whether the district has collapsed.
    pub collapsed: bool,
    /// Current consecutive deficit count.
    pub consecutive_deficit_ticks: u32,
    /// Configured threshold.
    pub threshold: u32,
}

/// Collapse event emitted when a district collapses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistrictCollapseEvent {
    /// District that collapsed.
    pub district_id: u32,
    /// Tick at which collapse occurred.
    pub tick: u64,
    /// Number of consecutive deficit ticks at collapse.
    pub deficit_ticks: u32,
}

/// Check and advance district deficit state. Returns a collapse event if the
/// district crosses the threshold this tick.
///
/// If `balance >= 0`, the deficit counter resets to 0.
/// If `balance < 0`, the counter increments. When it reaches `threshold`,
/// a [`DistrictCollapseEvent`] is returned and the counter resets.
#[must_use]
pub fn tick_district_collapse(
    state: &mut DistrictEnergyState,
    tick: u64,
    threshold: u32,
) -> Option<DistrictCollapseEvent> {
    if state.balance_joules >= 0 {
        state.consecutive_deficit_ticks = 0;
        return None;
    }

    state.consecutive_deficit_ticks = state.consecutive_deficit_ticks.saturating_add(1);

    if state.consecutive_deficit_ticks >= threshold {
        let event = DistrictCollapseEvent {
            district_id: state.district_id,
            tick,
            deficit_ticks: state.consecutive_deficit_ticks,
        };
        state.consecutive_deficit_ticks = 0; // reset after collapse
        Some(event)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_collapse_when_positive_balance() {
        let mut state = DistrictEnergyState::new(0, 100);
        let event = tick_district_collapse(&mut state, 1, DEFAULT_DEFICIT_TICKS);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }

    #[test]
    fn deficit_increments_counter() {
        let mut state = DistrictEnergyState::new(0, -10);
        let _ = tick_district_collapse(&mut state, 1, DEFAULT_DEFICIT_TICKS);
        assert_eq!(state.consecutive_deficit_ticks, 1);
    }

    #[test]
    fn collapse_after_threshold_ticks() {
        let mut state = DistrictEnergyState::new(42, -10);
        for tick in 1..=3 {
            let event = tick_district_collapse(&mut state, tick, 3);
            if tick < 3 {
                assert_eq!(event, None);
            } else {
                assert!(event.is_some());
                let e = event.unwrap();
                assert_eq!(e.district_id, 42);
                assert_eq!(e.deficit_ticks, 3);
            }
        }
        // Counter resets after collapse
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }

    #[test]
    fn positive_balance_resets_counter() {
        let mut state = DistrictEnergyState::new(0, -10);
        let _ = tick_district_collapse(&mut state, 1, 3);
        let _ = tick_district_collapse(&mut state, 2, 3);
        assert_eq!(state.consecutive_deficit_ticks, 2);
        // Now positive
        state.balance_joules = 50;
        let event = tick_district_collapse(&mut state, 3, 3);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }

    #[test]
    fn custom_threshold() {
        let mut state = DistrictEnergyState::new(0, -10);
        let _ = tick_district_collapse(&mut state, 1, 1); // threshold=1
        // Should collapse immediately
        state.consecutive_deficit_ticks = 1; // manually set since tick_district already reset
        // Re-test with threshold 5
        let mut state2 = DistrictEnergyState::new(0, -10);
        let e = tick_district_collapse(&mut state2, 1, 5);
        assert_eq!(e, None); // only 1 tick, threshold is 5
    }
}
