//! FR-INST-005 — Institution time-series data SHALL be stored in the
//! metrics DB for post-run analysis.

use civ_institutions::{CivilizationState, GovernanceType, InstitutionTimeSeries};

/// Time-series data is stored per tick.
#[test]
fn db_institution_series_stored() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    state.tick(0, 0, 100);
    state.tick(1, 0, 110);
    state.tick(2, 0, 120);
    assert_eq!(state.time_series.len(), 3);
}

/// Time-series data points carry tick and governance type.
#[test]
fn series_point_carry_metadata() {
    let mut state = CivilizationState::new(42, GovernanceType::Autocracy);
    state.tick(10, 5_000, 200);
    let latest = state.time_series.latest().unwrap();
    assert_eq!(latest.tick, 10);
    assert_eq!(latest.civilization_id, 42);
    assert_eq!(latest.governance_type, GovernanceType::Autocracy);
    assert_eq!(latest.capture_bp, state.capture.value_bp);
    assert_eq!(latest.population, 200);
}

/// Time-series is append-only.
#[test]
fn series_is_append_only() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    state.tick(0, 0, 100);
    state.tick(1, 0, 100);
    let first_tick = state.time_series.points[0].tick;
    let second_tick = state.time_series.points[1].tick;
    assert!(second_tick > first_tick, "series must be chronological");
}

/// Empty series is empty.
#[test]
fn empty_series_is_empty() {
    let ts = InstitutionTimeSeries::new(1);
    assert!(ts.is_empty());
    assert_eq!(ts.len(), 0);
    assert!(ts.latest().is_none());
}

/// Governance type transition is reflected in time-series.
#[test]
fn governance_transition_recorded_in_series() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    state.tick(0, 0, 100);
    // Force collapse.
    state.legitimacy.apply_outcome(civ_institutions::GovernanceOutcome::poor(0.9));
    state.tick(1, 0, 100);
    // The second point should reflect the collapsed type.
    let point = &state.time_series.points[1];
    assert_eq!(point.governance_type, state.governance_type);
}

/// Series persists across many ticks.
#[test]
fn series_scales_to_many_ticks() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    for t in 0..1000 {
        state.tick(t, 1_000, 100);
    }
    assert_eq!(state.time_series.len(), 1000);
}
