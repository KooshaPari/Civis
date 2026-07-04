//! Institution legitimacy state for **FR-CIV-INST-LEGIT**.
//!
//! Legitimacy is a normalized scalar that moves with governance outcomes.
//! Repeated poor outcomes can push an institution below the collapse
//! threshold, allowing the owning simulation to remove or disable it.

use serde::{Deserialize, Serialize};

/// Default legitimacy for a newly created institution.
pub const DEFAULT_LEGITIMACY: f32 = 1.0;

/// Minimum allowed legitimacy value.
pub const MIN_LEGITIMACY: f32 = 0.0;

/// Maximum allowed legitimacy value.
pub const MAX_LEGITIMACY: f32 = 1.0;

/// Collapse threshold for institution legitimacy.
///
/// A legitimacy value strictly below this threshold is considered collapsed.
pub const LEGITIMACY_COLLAPSE_THRESHOLD: f32 = 0.25;

/// Governance outcome signal applied to an institution's legitimacy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GovernanceOutcome {
    /// Legitimacy delta produced by the outcome.
    ///
    /// Positive values increase legitimacy; negative values reduce it.
    pub legitimacy_delta: f32,
}

impl GovernanceOutcome {
    /// Creates an outcome from a raw legitimacy delta.
    pub const fn new(legitimacy_delta: f32) -> Self {
        Self { legitimacy_delta }
    }

    /// Creates a poor governance outcome that reduces legitimacy.
    pub const fn poor(amount: f32) -> Self {
        Self {
            legitimacy_delta: -amount,
        }
    }

    /// Creates a good governance outcome that increases legitimacy.
    pub const fn good(amount: f32) -> Self {
        Self {
            legitimacy_delta: amount,
        }
    }
}

/// Mutable legitimacy state for a civic institution.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InstitutionLegitimacy {
    /// Current normalized legitimacy scalar.
    pub value: f32,
    /// Threshold below which the institution has collapsed.
    pub collapse_threshold: f32,
}

impl Default for InstitutionLegitimacy {
    fn default() -> Self {
        Self {
            value: DEFAULT_LEGITIMACY,
            collapse_threshold: LEGITIMACY_COLLAPSE_THRESHOLD,
        }
    }
}

impl InstitutionLegitimacy {
    /// Creates legitimacy state with a clamped normalized value.
    pub fn new(value: f32) -> Self {
        Self {
            value: clamp_legitimacy(value),
            collapse_threshold: LEGITIMACY_COLLAPSE_THRESHOLD,
        }
    }

    /// Applies a governance outcome and returns the updated legitimacy value.
    pub fn apply_outcome(&mut self, outcome: GovernanceOutcome) -> f32 {
        self.value = clamp_legitimacy(self.value + outcome.legitimacy_delta);
        self.value
    }

    /// Returns true when legitimacy has fallen below the collapse threshold.
    pub fn is_collapsed(self) -> bool {
        self.value < self.collapse_threshold
    }
}

fn clamp_legitimacy(value: f32) -> f32 {
    value.clamp(MIN_LEGITIMACY, MAX_LEGITIMACY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poor_outcomes_drop_legitimacy_below_collapse_threshold() {
        let mut legitimacy = InstitutionLegitimacy::default();

        legitimacy.apply_outcome(GovernanceOutcome::poor(0.4));
        legitimacy.apply_outcome(GovernanceOutcome::poor(0.4));

        assert!(legitimacy.value < LEGITIMACY_COLLAPSE_THRESHOLD);
        assert!(legitimacy.is_collapsed());
    }
}
