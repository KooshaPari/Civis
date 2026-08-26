//! Diplomacy stance model (FR-CIV-DIPLOMACY-004, issue #958).
//!
//! Extends the coarse [`crate::EmergentStance`] (Rival/Neutral/Ally) with a
//! per-pair **opinion vector** `{ trust, fear, respect }` that evolves through
//! interactions. The stance falls out of the opinion vector's quadrant, giving
//! a richer basis for concession bias, alliance eligibility, and AI decisions.
//!
//! # Design
//!
//! Each `(PolityId, PolityId)` pair has a [`DiplomacyStance`] with three
//! independent axes:
//!
//! - **Trust** `[-1, 1]`: built by trade, broken by betrayal, decays toward 0.
//! - **Fear** `[0, 1]`: built by military superiority, border friction, shared enemies.
//! - **Respect** `[-1, 1]`: built by cultural similarity, broken by defection.
//!
//! These axes are updated via [`InteractionEvent`]s (trade, treaty, betrayal,
//! military contact, cultural exchange) and decay linearly each tick.
//!
//! The coarse [`crate::EmergentStance`] is derived from the opinion vector:
//! - Ally: trust > 0.5 AND respect > 0.3
//! - Rival: trust < -0.3 OR fear > 0.7
//! - Neutral: everything else
//!
//! # Determinism
//!
//! All math is f32 clamped. BTreeMap iteration is sorted. No RNG.

use crate::{Pair, PolityId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Per-pair opinion vector. The three axes evolve independently through
/// interactions and decay toward zero each tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyStance {
    /// Trust axis `[-1, 1]`. Positive = cooperative, negative = suspicious.
    pub trust: f32,
    /// Fear axis `[0, 1]`. 0 = no threat, 1 = existential dread.
    pub fear: f32,
    /// Respect axis `[-1, 1]`. Positive = admire, negative = despise.
    pub respect: f32,
}

impl Default for DiplomacyStance {
    fn default() -> Self {
        Self {
            trust: 0.0,
            fear: 0.0,
            respect: 0.0,
        }
    }
}

impl DiplomacyStance {
    /// New zero stance (unknown relationship).
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluate trust/respect into a coarse [`crate::EmergentStance`].
    pub fn coarse_stance(&self) -> crate::EmergentStance {
        if self.trust > 0.5 && self.respect > 0.3 {
            crate::EmergentStance::Ally
        } else if self.trust < -0.3 || self.fear > 0.7 {
            crate::EmergentStance::Rival
        } else {
            crate::EmergentStance::Neutral
        }
    }

    /// A weighted composite score in `[-1, 1]` for concession bias.
    /// Positive = favorable, negative = hostile.
    pub fn concession_bias(&self) -> f32 {
        (0.5 * self.trust + 0.3 * self.respect - 0.4 * (self.fear * 2.0 - 1.0)).clamp(-1.0, 1.0)
    }

    /// Classify the relationship into one of 6 named quadrants.
    pub fn quadrant(&self) -> RelationQuadrant {
        if self.trust > 0.5 && self.fear < 0.3 {
            RelationQuadrant::TrustedAlly
        } else if self.fear > 0.7 && self.trust < 0.0 {
            RelationQuadrant::FearedEnemy
        } else if self.respect > 0.5 && self.trust > 0.0 {
            RelationQuadrant::HonoredPartner
        } else if self.fear > 0.5 && self.respect < 0.0 {
            RelationQuadrant::ResentedThreat
        } else if self.trust < -0.3 && self.fear < 0.3 {
            RelationQuadrant::DistrustedOutsider
        } else {
            RelationQuadrant::Indifferent
        }
    }

    /// Apply an interaction event, adjusting the opinion vector.
    pub fn apply_event(&mut self, event: &InteractionEvent) {
        match event {
            InteractionEvent::TradeVolume { volume } => {
                // Trade builds trust, capped at diminishing returns.
                let delta = 0.1 * volume.min(1.0);
                self.trust = (self.trust + delta).clamp(-1.0, 1.0);
            }
            InteractionEvent::TreatyFormed => {
                // Treaty formation boosts trust and respect.
                self.trust = (self.trust + 0.3).clamp(-1.0, 1.0);
                self.respect = (self.respect + 0.2).clamp(-1.0, 1.0);
                self.fear = (self.fear - 0.1).clamp(0.0, 1.0);
            }
            InteractionEvent::TreatyBroken => {
                // Breaking a treaty devastates trust.
                self.trust = (self.trust - 0.5).clamp(-1.0, 1.0);
                self.respect = (self.respect - 0.3).clamp(-1.0, 1.0);
            }
            InteractionEvent::Betrayal => {
                // Betrayal is worse than a simple break.
                self.trust = (self.trust - 0.8).clamp(-1.0, 1.0);
                self.respect = (self.respect - 0.5).clamp(-1.0, 1.0);
                self.fear = (self.fear + 0.2).clamp(0.0, 1.0);
            }
            InteractionEvent::MilitaryContact { superiority } => {
                // Military superiority builds fear, weak respect if extreme.
                let fear_delta = 0.15 * superiority.clamp(0.0, 1.0);
                self.fear = (self.fear + fear_delta).clamp(0.0, 1.0);
                if *superiority > 0.8 {
                    // Overwhelming force erodes respect.
                    self.respect = (self.respect - 0.1).clamp(-1.0, 1.0);
                }
            }
            InteractionEvent::CulturalExchange { similarity } => {
                // Cultural similarity builds respect and trust.
                let delta = 0.1 * similarity.clamp(-1.0, 1.0);
                self.respect = (self.respect + delta).clamp(-1.0, 1.0);
                self.trust = (self.trust + delta * 0.5).clamp(-1.0, 1.0);
            }
            InteractionEvent::BorderFriction { intensity } => {
                // Border friction increases fear and decreases trust.
                let i = intensity.clamp(0.0, 1.0);
                self.fear = (self.fear + 0.05 * i).clamp(0.0, 1.0);
                self.trust = (self.trust - 0.05 * i).clamp(-1.0, 1.0);
            }
            InteractionEvent::SharedEnemy { strength } => {
                // Shared enemy builds trust ("enemy of my enemy").
                let delta = 0.15 * strength.clamp(0.0, 1.0);
                self.trust = (self.trust + delta).clamp(-1.0, 1.0);
                self.fear = (self.fear - delta * 0.3).clamp(0.0, 1.0);
            }
        }
    }

    /// Decay all axes toward their neutral values (trust→0, fear→0, respect→0).
    /// `rate` is the per-tick decay fraction (e.g. 0.02 = 2% per tick).
    pub fn decay(&mut self, rate: f32) {
        let r = rate.clamp(0.0, 1.0);
        self.trust *= 1.0 - r;
        self.fear *= 1.0 - r;
        self.respect *= 1.0 - r;
    }
}

/// Named relationship quadrants derived from the opinion vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationQuadrant {
    /// High trust, low fear — reliable partner.
    TrustedAlly,
    /// High fear, low trust — existential threat.
    FearedEnemy,
    /// High respect, positive trust — valued collaborator.
    HonoredPartner,
    /// High fear, negative respect — dangerous and despised.
    ResentedThreat,
    /// Low trust, low fear — distant outsider.
    DistrustedOutsider,
    /// Everything else.
    Indifferent,
}

/// An interaction event that updates the opinion vector between two factions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InteractionEvent {
    /// Trade happened; volume is `[0, 1]` normalized bilateral trade.
    TradeVolume {
        /// Normalized trade volume.
        volume: f32,
    },
    /// A treaty was formed between the pair.
    TreatyFormed,
    /// A treaty was broken (not necessarily a betrayal).
    TreatyBroken,
    /// A betrayal occurred (surprise attack while treaty active).
    Betrayal,
    /// Military contact; superiority `[-1, 1]` (negative = we are weaker).
    MilitaryContact {
        /// Relative military superiority.
        superiority: f32,
    },
    /// Cultural exchange; similarity `[-1, 1]` (negative = cultures clash).
    CulturalExchange {
        /// Cultural similarity.
        similarity: f32,
    },
    /// Border friction; intensity `[0, 1]`.
    BorderFriction {
        /// Friction intensity.
        intensity: f32,
    },
    /// Shared enemy detected; strength `[0, 1]`.
    SharedEnemy {
        /// Strength of the shared-enemy pull.
        strength: f32,
    },
}

/// The diplomacy stance engine. Tracks per-pair opinion vectors and derives
/// coarse stances for downstream systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyStanceEngine {
    /// Per-pair opinion vectors. BTreeMap for deterministic iteration.
    stances: BTreeMap<Pair, DiplomacyStance>,
    /// Per-tick decay rate (fraction).
    decay_rate: f32,
}

impl Default for DiplomacyStanceEngine {
    fn default() -> Self {
        Self::new(0.02)
    }
}

impl DiplomacyStanceEngine {
    /// New engine with the given per-tick decay rate (e.g. 0.02 = 2%).
    pub fn new(decay_rate: f32) -> Self {
        Self {
            stances: BTreeMap::new(),
            decay_rate,
        }
    }

    /// Get or create the stance for pair `(a, b)`.
    pub fn get_or_create(&mut self, a: PolityId, b: PolityId) -> &mut DiplomacyStance {
        let pair = Pair::new(a, b);
        self.stances.entry(pair).or_default()
    }

    /// Get the stance for `(a, b)` (immutable).
    pub fn get(&self, a: PolityId, b: PolityId) -> DiplomacyStance {
        self.stances
            .get(&Pair::new(a, b))
            .copied()
            .unwrap_or_default()
    }

    /// Get the coarse stance for `(a, b)`.
    pub fn coarse_stance(&self, a: PolityId, b: PolityId) -> crate::EmergentStance {
        self.get(a, b).coarse_stance()
    }

    /// Get the concession bias for `(a, b)`.
    pub fn concession_bias(&self, a: PolityId, b: PolityId) -> f32 {
        self.get(a, b).concession_bias()
    }

    /// Apply an interaction event to pair `(a, b)`.
    pub fn apply(&mut self, a: PolityId, b: PolityId, event: &InteractionEvent) {
        self.get_or_create(a, b).apply_event(event);
    }

    /// Decay all stances toward neutral. Call once per tick.
    pub fn tick_decay(&mut self) {
        for stance in self.stances.values_mut() {
            stance.decay(self.decay_rate);
        }
    }

    /// Number of tracked pairs.
    pub fn len(&self) -> usize {
        self.stances.len()
    }

    /// True if no pairs are tracked.
    pub fn is_empty(&self) -> bool {
        self.stances.is_empty()
    }

    /// All tracked pairs and their stances (sorted by pair for determinism).
    pub fn pairs(&self) -> &BTreeMap<Pair, DiplomacyStance> {
        &self.stances
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a() -> PolityId {
        PolityId::new(1)
    }
    fn b() -> PolityId {
        PolityId::new(2)
    }
    fn c() -> PolityId {
        PolityId::new(3)
    }

    #[test]
    fn default_stance_is_indifferent() {
        let s = DiplomacyStance::new();
        assert_eq!(s.trust, 0.0);
        assert_eq!(s.fear, 0.0);
        assert_eq!(s.respect, 0.0);
        assert_eq!(s.quadrant(), RelationQuadrant::Indifferent);
    }

    #[test]
    fn trade_builds_trust() {
        let mut s = DiplomacyStance::new();
        s.apply_event(&InteractionEvent::TradeVolume { volume: 0.8 });
        assert!(s.trust > 0.0, "trust should increase from trade");
    }

    #[test]
    fn treaty_boosts_trust_and_respect() {
        let mut s = DiplomacyStance::new();
        s.apply_event(&InteractionEvent::TreatyFormed);
        assert!(s.trust > 0.2);
        assert!(s.respect > 0.1);
        assert!(s.fear < 0.01);
    }

    #[test]
    fn betrayal_devastates_trust() {
        let mut s = DiplomacyStance::new();
        s.apply_event(&InteractionEvent::TreatyFormed);
        let trust_before = s.trust;
        s.apply_event(&InteractionEvent::Betrayal);
        assert!(
            s.trust < trust_before - 0.5,
            "betrayal should destroy trust"
        );
    }

    #[test]
    fn military_superiority_builds_fear() {
        let mut s = DiplomacyStance::new();
        s.apply_event(&InteractionEvent::MilitaryContact {
            superiority: 0.9,
        });
        assert!(s.fear > 0.1);
    }

    #[test]
    fn cultural_exchange_builds_respect() {
        let mut s = DiplomacyStance::new();
        s.apply_event(&InteractionEvent::CulturalExchange {
            similarity: 0.7,
        });
        assert!(s.respect > 0.0);
        assert!(s.trust > 0.0);
    }

    #[test]
    fn decay_reduces_all_axes() {
        let mut s = DiplomacyStance {
            trust: 0.5,
            fear: 0.5,
            respect: 0.5,
        };
        s.decay(0.1);
        assert!(s.trust < 0.5);
        assert!(s.fear < 0.5);
        assert!(s.respect < 0.5);
    }

    #[test]
    fn ally_quadrant_requires_high_trust_and_respect() {
        let s = DiplomacyStance {
            trust: 0.7,
            fear: 0.1,
            respect: 0.5,
        };
        assert_eq!(s.quadrant(), RelationQuadrant::TrustedAlly);
    }

    #[test]
    fn enemy_quadrant_requires_high_fear() {
        let s = DiplomacyStance {
            trust: -0.5,
            fear: 0.9,
            respect: -0.3,
        };
        assert_eq!(s.quadrant(), RelationQuadrant::FearedEnemy);
    }

    #[test]
    fn concession_bias_positive_for_allies() {
        let s = DiplomacyStance {
            trust: 0.7,
            fear: 0.1,
            respect: 0.5,
        };
        assert!(s.concession_bias() > 0.0);
    }

    #[test]
    fn concession_bias_negative_for_enemies() {
        let s = DiplomacyStance {
            trust: -0.7,
            fear: 0.8,
            respect: -0.5,
        };
        assert!(s.concession_bias() < 0.0);
    }

    #[test]
    fn coarse_stance_ally() {
        let s = DiplomacyStance {
            trust: 0.6,
            fear: 0.1,
            respect: 0.4,
        };
        assert_eq!(s.coarse_stance(), crate::EmergentStance::Ally);
    }

    #[test]
    fn coarse_stance_rival() {
        let s = DiplomacyStance {
            trust: -0.5,
            fear: 0.8,
            respect: -0.2,
        };
        assert_eq!(s.coarse_stance(), crate::EmergentStance::Rival);
    }

    #[test]
    fn engine_applies_event_to_pair() {
        let mut engine = DiplomacyStanceEngine::new(0.02);
        engine.apply(a(), b(), &InteractionEvent::TradeVolume { volume: 0.5 });
        assert!(engine.get(a(), b).trust > 0.0);
        assert_eq!(engine.get(a(), c()).trust, 0.0); // other pair unaffected
    }

    #[test]
    fn engine_decay_reduces_all() {
        let mut engine = DiplomacyStanceEngine::new(0.1);
        engine.apply(a(), b(), &InteractionEvent::TreatyFormed);
        let before = engine.get(a(), b);
        engine.tick_decay();
        let after = engine.get(a(), b);
        assert!(after.trust < before.trust);
    }

    #[test]
    fn engine_tracks_len() {
        let mut engine = DiplomacyStanceEngine::new(0.02);
        assert!(engine.is_empty());
        engine.apply(a(), b(), &InteractionEvent::TreatyFormed);
        engine.apply(a(), c(), &InteractionEvent::TreatyFormed);
        assert_eq!(engine.len(), 2);
    }
}
