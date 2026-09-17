//! FR-INST-004 — Institutional collapse and governance transition.
//!
//! When institutional legitimacy falls below the collapse threshold,
//! or when the capture score is sufficiently high, the current
//! governance type SHALL transition to a new type. The transition
//! is deterministic based on the current type and collapse cause.

use serde::{Deserialize, Serialize};

use crate::capture::CaptureScore;
use crate::governance::GovernanceType;
use crate::legitimacy::InstitutionLegitimacy;

/// Cause of institutional collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollapseCause {
    /// Legitimacy fell below collapse threshold.
    LegitimacyLoss,
    /// Capture score exceeded threshold, elites seized control.
    EliteCapture,
    /// Combined legitimacy loss and elite capture.
    Combined,
}

/// Event emitted when institutional collapse triggers a governance
/// type transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollapseTransitionEvent {
    /// Tick at which collapse occurred.
    pub tick: u64,
    /// Civilization identifier.
    pub civilization_id: u32,
    /// Governance type before collapse.
    pub from_type: GovernanceType,
    /// Governance type after collapse.
    pub to_type: GovernanceType,
    /// What caused the collapse.
    pub cause: CollapseCause,
    /// Legitimacy at collapse.
    pub legitimacy: f32,
    /// Capture score at collapse (basis points).
    pub capture_bp: i64,
}

/// Evaluates whether the civilization has collapsed and determines the
/// transition target.
///
/// Returns `Some(CollapseTransitionEvent)` when the institution should
/// collapse. The governance transition is deterministic based on the
/// current type.
pub fn check_collapse(
    tick: u64,
    civilization_id: u32,
    current_type: GovernanceType,
    legitimacy: &InstitutionLegitimacy,
    capture: &CaptureScore,
) -> Option<CollapseTransitionEvent> {
    let legitimacy_collapsed = legitimacy.is_collapsed();
    let capture_exceeded = capture.is_captured();

    let cause = match (legitimacy_collapsed, capture_exceeded) {
        (true, true) => CollapseCause::Combined,
        (true, false) => CollapseCause::LegitimacyLoss,
        (false, true) => CollapseCause::EliteCapture,
        (false, false) => return None,
    };

    let to_type = transition_target(current_type, cause);

    Some(CollapseTransitionEvent {
        tick,
        civilization_id,
        from_type: current_type,
        to_type,
        cause,
        legitimacy: legitimacy.value,
        capture_bp: capture.value_bp,
    })
}

/// Determines the governance transition target given the current type
/// and collapse cause.
///
/// Transitions follow a "power diffusion" model:
/// - Elite capture pushes toward Autocracy (elites seize direct control).
/// - Legitimacy loss pushes toward Anarchy (power vacuum).
/// - Combined push toward Oligarchy (elites fill the vacuum).
pub fn transition_target(current: GovernanceType, cause: CollapseCause) -> GovernanceType {
    match cause {
        CollapseCause::EliteCapture => GovernanceType::Autocracy,
        CollapseCause::LegitimacyLoss => GovernanceType::Anarchy,
        CollapseCause::Combined => match current {
            GovernanceType::Democracy | GovernanceType::Technocracy => GovernanceType::Oligarchy,
            GovernanceType::Autocracy | GovernanceType::Oligarchy => GovernanceType::Anarchy,
            GovernanceType::Theocracy => GovernanceType::Autocracy,
            GovernanceType::Anarchy => GovernanceType::Anarchy,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CAPTURE_THRESHOLD_BP;
    use crate::legitimacy::LEGITIMACY_COLLAPSE_THRESHOLD;

    #[test]
    fn no_collapse_when_healthy() {
        let leg = InstitutionLegitimacy::default();
        let cap = CaptureScore::new();
        let result = check_collapse(0, 1, GovernanceType::Democracy, &leg, &cap);
        assert!(result.is_none());
    }

    #[test]
    fn collapse_from_legitimacy_loss() {
        let leg = InstitutionLegitimacy::new(LEGITIMACY_COLLAPSE_THRESHOLD - 0.01);
        let cap = CaptureScore::new();
        let result = check_collapse(10, 1, GovernanceType::Democracy, &leg, &cap);
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.cause, CollapseCause::LegitimacyLoss);
        assert_eq!(event.to_type, GovernanceType::Anarchy);
    }

    #[test]
    fn collapse_from_elite_capture() {
        let leg = InstitutionLegitimacy::default();
        let cap = CaptureScore::with_value(CAPTURE_THRESHOLD_BP);
        let result = check_collapse(20, 1, GovernanceType::Democracy, &leg, &cap);
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.cause, CollapseCause::EliteCapture);
        assert_eq!(event.to_type, GovernanceType::Autocracy);
    }

    #[test]
    fn transition_target_matches_cause() {
        assert_eq!(
            transition_target(GovernanceType::Democracy, CollapseCause::EliteCapture),
            GovernanceType::Autocracy
        );
        assert_eq!(
            transition_target(GovernanceType::Autocracy, CollapseCause::LegitimacyLoss),
            GovernanceType::Anarchy
        );
    }
}
