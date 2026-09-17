//! FR-INST-001 — Governance type for each civilization.
//!
//! Each civilization SHALL have an institutional type (democracy, autocracy,
//! technocracy, etc.) that defines how power is distributed and decisions
//! are made. The type is assigned at initialization and can transition
//! during collapse events (see `collapse` module).

use serde::{Deserialize, Serialize};

/// Institutional governance type for a civilization.
///
/// Each civilization SHALL have exactly one active [`GovernanceType`] at any
/// given tick. The type is assigned at initialization and may transition
/// when institutional collapse triggers a regime change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernanceType {
    /// Power concentrated in a single ruler or ruling party.
    Autocracy,
    /// Rule by a small, privileged class.
    Oligarchy,
    /// Representative government with elected officials.
    Democracy,
    /// Rule by a technical elite or expert class.
    Technocracy,
    /// Rule by a religious authority or priestly class.
    Theocracy,
    /// Transitional or unstable governance with no clear structure.
    Anarchy,
}

impl GovernanceType {
    /// Total number of governance types.
    pub const COUNT: usize = 6;

    /// Returns a human-readable name for this governance type.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Autocracy => "Autocracy",
            Self::Oligarchy => "Oligarchy",
            Self::Democracy => "Democracy",
            Self::Technocracy => "Technocracy",
            Self::Theocracy => "Theocracy",
            Self::Anarchy => "Anarchy",
        }
    }

    /// Returns the stable index of this governance type (sorted by
    /// typical power concentration, most to least concentrated).
    pub fn index(self) -> usize {
        match self {
            Self::Autocracy => 0,
            Self::Oligarchy => 1,
            Self::Theocracy => 2,
            Self::Technocracy => 3,
            Self::Democracy => 4,
            Self::Anarchy => 5,
        }
    }
}

impl Default for GovernanceType {
    fn default() -> Self {
        Self::Democracy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_type_index_is_stable() {
        assert_eq!(GovernanceType::Autocracy.index(), 0);
        assert_eq!(GovernanceType::Democracy.index(), 4);
        assert_eq!(GovernanceType::Anarchy.index(), 5);
    }

    #[test]
    fn governance_type_count_matches_variants() {
        assert_eq!(GovernanceType::COUNT, 6);
    }

    #[test]
    fn default_is_democracy() {
        assert_eq!(GovernanceType::default(), GovernanceType::Democracy);
    }

    #[test]
    fn as_str_matches_variant() {
        assert_eq!(GovernanceType::Autocracy.as_str(), "Autocracy");
        assert_eq!(GovernanceType::Technocracy.as_str(), "Technocracy");
    }
}
