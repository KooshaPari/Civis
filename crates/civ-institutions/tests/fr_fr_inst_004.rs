//! FR-INST-004 — Institutional collapse SHALL trigger a governance type
//! transition.

use civ_institutions::*;

/// Collapse triggers type transition.
#[test]
fn collapse_triggers_type_transition() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    // Drive legitimacy below threshold through poor governance.
    state.legitimacy.apply_outcome(GovernanceOutcome::poor(0.4));
    state.legitimacy.apply_outcome(GovernanceOutcome::poor(0.4));
    assert!(state.legitimacy.is_collapsed());
    // Tick should trigger collapse.
    let result = state.tick(10, 0, 100);
    assert!(result.collapse_transition.is_some(), "must emit collapse event");
    let event = result.collapse_transition.unwrap();
    assert_eq!(event.from_type, GovernanceType::Democracy);
    assert_eq!(event.cause, CollapseCause::LegitimacyLoss);
}

/// Legitimacy loss transitions to Anarchy.
#[test]
fn legitimacy_loss_to_anarchy() {
    let target = transition_target(GovernanceType::Democracy, CollapseCause::LegitimacyLoss);
    assert_eq!(target, GovernanceType::Anarchy);
}

/// Elite capture transitions to Autocracy.
#[test]
fn elite_capture_to_autocracy() {
    let target = transition_target(GovernanceType::Democracy, CollapseCause::EliteCapture);
    assert_eq!(target, GovernanceType::Autocracy);
}

/// No collapse when institution is healthy.
#[test]
fn no_collapse_healthy() {
    let mut state = CivilizationState::new(1, GovernanceType::Democracy);
    let result = state.tick(10, 0, 100);
    assert!(result.collapse_transition.is_none());
}

/// Combined collapse from Democracy goes to Oligarchy.
#[test]
fn combined_democracy_to_oligarchy() {
    let target = transition_target(GovernanceType::Democracy, CollapseCause::Combined);
    assert_eq!(target, GovernanceType::Oligarchy);
}

/// Collapse event records the pre-collapse governance type.
#[test]
fn collapse_records_from_type() {
    let mut state = CivilizationState::new(1, GovernanceType::Technocracy);
    state.legitimacy.apply_outcome(GovernanceOutcome::poor(0.9));
    let result = state.tick(5, 0, 50);
    let event = result.collapse_transition.unwrap();
    assert_eq!(event.from_type, GovernanceType::Technocracy);
}
