//! FR-AI-003 — Personality profile affecting utility weights.
//!
//! Each AI leader SHALL have a personality profile that modifies the
//! base utility weights used during move scoring.

use serde::{Deserialize, Serialize};

use crate::utility::UtilityWeights;

/// Named personality archetypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalityKind {
    /// Balanced leader.
    Balanced,
    /// Expansionist: prioritises resources and strategy.
    Expansionist,
    /// Diplomat: prioritises diplomatic value.
    Diplomat,
    /// Militant: prioritises strategic value.
    Militant,
    /// Isolationist: prioritises safety.
    Isolationist,
}

/// Per-leader personality profile with weight modifiers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonalityProfile {
    /// The archetype kind.
    pub kind: PersonalityKind,
    /// Multipliers applied to base `UtilityWeights`.
    /// E.g. 1.2 means 20 % boost to that dimension.
    pub resource_mult: f64,
    /// Strategic weight multiplier.
    pub strategic_mult: f64,
    /// Safety weight multiplier.
    pub safety_mult: f64,
    /// Diplomatic weight multiplier.
    pub diplomatic_mult: f64,
}

impl PersonalityProfile {
    /// Return the pre-configured profile for a given archetype.
    #[must_use]
    pub fn from_kind(kind: PersonalityKind) -> Self {
        match kind {
            PersonalityKind::Balanced => Self {
                kind,
                resource_mult: 1.0,
                strategic_mult: 1.0,
                safety_mult: 1.0,
                diplomatic_mult: 1.0,
            },
            PersonalityKind::Expansionist => Self {
                kind,
                resource_mult: 1.4,
                strategic_mult: 1.2,
                safety_mult: 0.7,
                diplomatic_mult: 0.8,
            },
            PersonalityKind::Diplomat => Self {
                kind,
                resource_mult: 0.9,
                strategic_mult: 0.8,
                safety_mult: 1.0,
                diplomatic_mult: 1.5,
            },
            PersonalityKind::Militant => Self {
                kind,
                resource_mult: 0.8,
                strategic_mult: 1.5,
                safety_mult: 0.6,
                diplomatic_mult: 0.5,
            },
            PersonalityKind::Isolationist => Self {
                kind,
                resource_mult: 0.7,
                strategic_mult: 0.6,
                safety_mult: 1.6,
                diplomatic_mult: 0.4,
            },
        }
    }

    /// Apply personality multipliers to base weights, returning modified weights.
    #[must_use]
    pub fn apply_to(&self, base: &UtilityWeights) -> UtilityWeights {
        UtilityWeights {
            resource_value: base.resource_value * self.resource_mult,
            strategic_value: base.strategic_value * self.strategic_mult,
            safety_value: base.safety_value * self.safety_mult,
            diplomatic_value: base.diplomatic_value * self.diplomatic_mult,
        }
    }
}

/// Configuration for stochastic personality drift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersonalityDrift {
    /// Ticks between drift events.
    pub interval_ticks: u64,
    /// Maximum absolute change per drift event per weight.
    pub magnitude: f64,
}

impl Default for PersonalityDrift {
    fn default() -> Self {
        Self {
            interval_ticks: 50,
            magnitude: 0.05,
        }
    }
}

impl PersonalityDrift {
    /// Create a drift config with given interval and magnitude.
    #[must_use]
    pub fn new(interval_ticks: u64, magnitude: f64) -> Self {
        Self {
            interval_ticks,
            magnitude,
        }
    }

    /// Apply stochastic drift to a profile. `tick` is the current tick;
    /// `rng_value` is a caller-provided pseudo-random f64 in [-1.0, 1.0].
    /// Returns a new profile with drifted multipliers.
    #[must_use]
    pub fn apply(
        &self,
        profile: &PersonalityProfile,
        tick: u64,
        rng_value: f64,
    ) -> PersonalityProfile {
        if self.interval_ticks == 0 || tick % self.interval_ticks != 0 {
            return profile.clone();
        }
        let drift = self.magnitude * rng_value.clamp(-1.0, 1.0);
        PersonalityProfile {
            kind: profile.kind,
            resource_mult: (profile.resource_mult + drift).clamp(0.1, 3.0),
            strategic_mult: (profile.strategic_mult + drift).clamp(0.1, 3.0),
            safety_mult: (profile.safety_mult + drift).clamp(0.1, 3.0),
            diplomatic_mult: (profile.diplomatic_mult + drift).clamp(0.1, 3.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balanced_is_identity() {
        let base = UtilityWeights::default();
        let profile = PersonalityProfile::from_kind(PersonalityKind::Balanced);
        let modified = profile.apply_to(&base);
        assert_eq!(modified, base);
    }

    #[test]
    fn expansionist_boosts_resource() {
        let base = UtilityWeights::default();
        let profile = PersonalityProfile::from_kind(PersonalityKind::Expansionist);
        let modified = profile.apply_to(&base);
        assert!(modified.resource_value > base.resource_value);
    }
}
