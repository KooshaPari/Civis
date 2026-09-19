//! Tests for FR-ECON-008 — District Collapse After Consecutive Deficit Ticks
//!
//! Epic: FR-ECON
//! A district in Joule deficit for N consecutive ticks SHALL emit a collapse
//! event. Default threshold is 3 ticks. Counter resets on recovery.

use civ_economy::{tick_district_collapse, DistrictEnergyState};

#[cfg(test)]
mod fr_fr_econ_008 {
    use super::*;

    /// FR-ECON-008: Positive balance resets deficit counter, no collapse.
    #[test]
    fn positive_balance_no_collapse() {
        let mut state = DistrictEnergyState::new(1, 500);
        let event = tick_district_collapse(&mut state, 1, 3);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }

    /// FR-ECON-008: Three consecutive deficit ticks triggers collapse.
    #[test]
    fn collapse_after_three_deficit_ticks() {
        let mut state = DistrictEnergyState::new(1, -100);
        // Tick 1: deficit begins
        let event = tick_district_collapse(&mut state, 1, 3);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 1);
        // Tick 2: still in deficit
        let event = tick_district_collapse(&mut state, 2, 3);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 2);
        // Tick 3: threshold reached — collapse!
        let event = tick_district_collapse(&mut state, 3, 3);
        assert!(event.is_some());
        let collapse = event.unwrap();
        assert_eq!(collapse.district_id, 1);
        assert_eq!(collapse.tick, 3);
        assert_eq!(collapse.deficit_ticks, 3);
        // Counter resets after collapse.
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }

    /// FR-ECON-008: Recovery (positive balance) resets counter before threshold.
    #[test]
    fn recovery_resets_counter() {
        let mut state = DistrictEnergyState::new(2, -50);
        let _ = tick_district_collapse(&mut state, 1, 3);
        let _ = tick_district_collapse(&mut state, 2, 3);
        assert_eq!(state.consecutive_deficit_ticks, 2);
        // Positive balance resets.
        state.balance_joules = 100;
        let event = tick_district_collapse(&mut state, 3, 3);
        assert_eq!(event, None);
        assert_eq!(state.consecutive_deficit_ticks, 0, "counter reset on recovery");
    }

    /// FR-ECON-008: Configurable threshold — 1 tick collapse.
    #[test]
    fn threshold_one_immediate_collapse() {
        let mut state = DistrictEnergyState::new(3, -10);
        let event = tick_district_collapse(&mut state, 1, 1);
        assert!(event.is_some());
        assert_eq!(event.unwrap().deficit_ticks, 1);
    }

    /// FR-ECON-008: DistrictEnergyState tracks balance correctly.
    #[test]
    fn district_energy_state_tracks_balance() {
        let state = DistrictEnergyState::new(5, 1000);
        assert_eq!(state.district_id, 5);
        assert_eq!(state.balance_joules, 1000);
        assert_eq!(state.consecutive_deficit_ticks, 0);
    }
}
