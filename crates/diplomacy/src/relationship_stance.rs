//! Relationship-stance model — FR-CIV-DIPLO-STANCE.
//!
//! A focused, additive slice over the existing `Relation`/`Stance` substrate.
//! Faction pairs hold an **opinion scalar** that is shifted by named events
//! (`Trade = +`, `Raid = -`) and is then bucketed into one of three
//! relationship stances: `Ally`, `Neutral`, `Hostile`.
//!
//! ## Scope (additive — does not modify existing types)
//!
//! * [`FactionPair`] — canonical `(a, b)` pair key.
//! * [`Opinion`] — signed scalar shifted by [`RelationEvent`]s.
//! * [`RelationEvent`] — discrete event types (`Trade`, `Raid`) with signed
//!   deltas.
//! * [`RelationStance`] — three-bucket projection of opinion
//!   (`Ally` / `Neutral` / `Hostile`).
//! * [`StanceThresholds`] — tunable thresholds for the bucket boundaries.
//! * [`RelationshipStance`] — the aggregate: per-pair opinion + history,
//!   plus a method to project the current opinion to a [`RelationStance`].
//! * [`RelationshipStanceModel`] — the store of all [`RelationshipStance`]
//!   entries keyed by [`FactionPair`], with `apply_event` and `stance` APIs.
//!
//! ## Determinism
//!
//! All storage is `BTreeMap`-backed; opinions are `i32`; event deltas are
//! applied in input order. Two instances fed the same event stream produce
//! identical states and bucket projections.
//!
//! ## Independence from `DiplomacyState`
//!
//! This module does **not** depend on or modify the existing
//! [`crate::DiplomacyState`] / [`crate::Relation`] substrate. It is a
//! parallel, additive surface that callers may opt into. The two coexist:
//! existing game logic continues to use the substrate; new
//! relationship-stance consumers use this model.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Canonical ordered pair of [`crate::FactionId`]s.
///
/// Reuses [`crate::FactionId`] (alias for [`crate::PolityId`]) so consumers
/// can cross-reference the existing faction identifiers without an extra
/// mapping layer. Ordering matches [`crate::Pair::new`]: `lo <= hi`.
pub type FactionPair = crate::Pair;

/// Event that shifts the opinion scalar of a faction pair.
///
/// Each variant carries a **signed delta** that is added to the pair's
/// `Opinion` value (clamped to `[-opinion_max, +opinion_max]`). Deltas are
/// signed integers so callers can configure magnitude; the discriminator
/// keeps the *semantics* (trade warms, raid cools) explicit in event logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationEvent {
    /// A trade exchange. Positive delta warms opinion.
    Trade {
        /// Signed opinion delta (conventionally positive; clamped).
        delta: i32,
    },
    /// A raid / hostile act. Negative delta cools opinion.
    Raid {
        /// Signed opinion delta (conventionally negative; clamped).
        delta: i32,
    },
}

impl RelationEvent {
    /// The signed delta this event applies.
    pub fn delta(self) -> i32 {
        match self {
            RelationEvent::Trade { delta } => delta,
            RelationEvent::Raid { delta } => delta,
        }
    }

    /// `true` for warming events (`Trade`).
    pub fn is_warming(self) -> bool {
        matches!(self, RelationEvent::Trade { .. })
    }

    /// `true` for cooling events (`Raid`).
    pub fn is_cooling(self) -> bool {
        matches!(self, RelationEvent::Raid { .. })
    }
}

/// Bucket a pair's relationship falls into.
///
/// Distinct from [`crate::Stance`] by intent: this bucket is named after the
/// common player-facing relationship vocabulary (`Ally`/`Neutral`/`Hostile`)
/// rather than the substrate's `Allied/Neutral/Hostile`. The two enums
/// coexist because the relationship-stance model is additive and may evolve
/// independently from the substrate's internal projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationStance {
    /// Opinion ≥ `ally_threshold`.
    Ally,
    /// `hostile_threshold` < opinion < `ally_threshold`.
    Neutral,
    /// Opinion ≤ `hostile_threshold`.
    Hostile,
}

/// Tunable thresholds for bucketing opinion into [`RelationStance`].
///
/// Mirrors the structural invariants of [`crate::DiplomacyConfig`]:
/// `hostile_threshold < 0 < ally_threshold` and a non-negative
/// `opinion_max`. Defaults give sane symmetric buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StanceThresholds {
    /// Inclusive upper bound on `|opinion|`. Prevents griefing via giant
    /// deltas; also makes the projection deterministic.
    pub opinion_max: i32,
    /// Opinion ≤ this is [`RelationStance::Hostile`]. Must be < 0.
    pub hostile_threshold: i32,
    /// Opinion ≥ this is [`RelationStance::Ally`]. Must be > 0.
    pub ally_threshold: i32,
}

impl Default for StanceThresholds {
    fn default() -> Self {
        Self {
            opinion_max: 1_000,
            hostile_threshold: -50,
            ally_threshold: 50,
        }
    }
}

impl StanceThresholds {
    /// Validate the structural invariants. The relationship-stance model
    /// requires `hostile_threshold < 0 < ally_threshold` so the Neutral
    /// band is non-empty and the bucket boundaries are well-ordered.
    pub fn validate(&self) -> Result<(), StanceConfigError> {
        if self.opinion_max < 0 {
            return Err(StanceConfigError::NegativeOpinionMax(self.opinion_max));
        }
        if self.hostile_threshold >= 0 {
            return Err(StanceConfigError::HostileThresholdNotNegative(
                self.hostile_threshold,
            ));
        }
        if self.ally_threshold <= 0 {
            return Err(StanceConfigError::AllyThresholdNotPositive(
                self.ally_threshold,
            ));
        }
        if self.hostile_threshold >= self.ally_threshold {
            return Err(StanceConfigError::ThresholdsOverlap {
                hostile: self.hostile_threshold,
                ally: self.ally_threshold,
            });
        }
        Ok(())
    }
}

/// Configuration inconsistency in [`StanceThresholds`]. Reported at init,
/// never at runtime.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StanceConfigError {
    /// `opinion_max` is negative.
    #[error("opinion_max must be non-negative (got {0})")]
    NegativeOpinionMax(i32),
    /// `hostile_threshold` must be strictly negative.
    #[error("hostile_threshold must be < 0 (got {0})")]
    HostileThresholdNotNegative(i32),
    /// `ally_threshold` must be strictly positive.
    #[error("ally_threshold must be > 0 (got {0})")]
    AllyThresholdNotPositive(i32),
    /// `hostile_threshold >= ally_threshold` would leave no Neutral band.
    #[error("hostile_threshold ({hostile}) must be strictly less than ally_threshold ({ally})")]
    ThresholdsOverlap {
        /// `hostile_threshold` value.
        hostile: i32,
        /// `ally_threshold` value.
        ally: i32,
    },
}

/// Opinion state for one faction pair — a signed scalar plus a bounded
/// history of the events that produced it.
///
/// History is capped at [`HISTORY_LIMIT`] entries to keep memory bounded;
/// the projection to [`RelationStance`] only depends on `value`, so history
/// is audit-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opinion {
    /// Current opinion value, in `[-opinion_max, opinion_max]`.
    pub value: i32,
    /// Bounded history of the most recent events as `(delta)` (signed).
    pub history: Vec<i32>,
}

impl Default for Opinion {
    fn default() -> Self {
        Self {
            value: 0,
            history: Vec::new(),
        }
    }
}

impl Opinion {
    /// Push a delta onto the bounded history.
    fn push_history(&mut self, delta: i32) {
        self.history.push(delta);
        while self.history.len() > HISTORY_LIMIT {
            self.history.remove(0);
        }
    }
}

/// Upper bound on the per-pair history length (audit trail).
const HISTORY_LIMIT: usize = 20;

/// Per-pair relationship state: the opinion scalar plus its audit history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RelationshipStance {
    /// Current opinion scalar for this pair.
    pub opinion: Opinion,
}

impl RelationshipStance {
    /// Project the current opinion to a [`RelationStance`] using `thresholds`.
    pub fn stance(&self, thresholds: &StanceThresholds) -> RelationStance {
        stance_for(self.opinion.value, thresholds)
    }
}

/// Aggregate store: per-pair [`RelationshipStance`] keyed by [`FactionPair`].
///
/// All iteration order is stable (`BTreeMap`); `apply_event` is the only
/// mutator and it is deterministic given the event sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RelationshipStanceModel {
    thresholds: StanceThresholds,
    pairs: BTreeMap<FactionPair, RelationshipStance>,
}

impl RelationshipStanceModel {
    /// Construct a new model with the given thresholds. Validates the
    /// thresholds; rejects an invalid config at construction time.
    pub fn new(thresholds: StanceThresholds) -> Result<Self, StanceConfigError> {
        thresholds.validate()?;
        Ok(Self {
            thresholds,
            pairs: BTreeMap::new(),
        })
    }

    /// Borrow the current thresholds.
    pub fn thresholds(&self) -> &StanceThresholds {
        &self.thresholds
    }

    /// Number of tracked pairs.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// `true` if no pairs are tracked.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Look up the relationship state for `(a, b)`, if any. Symmetric:
    /// `(a, b)` and `(b, a)` resolve to the same entry.
    pub fn get(&self, a: crate::FactionId, b: crate::FactionId) -> Option<&RelationshipStance> {
        if a == b {
            return None;
        }
        self.pairs.get(&FactionPair::new(a, b))
    }

    /// Iterate all tracked pairs in stable order.
    pub fn pairs(&self) -> impl Iterator<Item = (&FactionPair, &RelationshipStance)> {
        self.pairs.iter()
    }

    /// Project the relationship between `a` and `b` to a [`RelationStance`].
    /// Unknown pairs default to [`RelationStance::Neutral`] (no recorded
    /// opinion ⇒ no relationship).
    pub fn stance(&self, a: crate::FactionId, b: crate::FactionId) -> RelationStance {
        match self.get(a, b) {
            Some(rs) => rs.stance(&self.thresholds),
            None => RelationStance::Neutral,
        }
    }

    /// Apply a [`RelationEvent`] between `a` and `b`. Shifts the opinion
    /// scalar by `event.delta()`, clamped to `[-opinion_max, opinion_max]`,
    /// and appends the delta to the bounded audit history. Returns the new
    /// [`RelationStance`] bucket after the shift.
    ///
    /// Self-targeted events (`a == b`) are no-ops and return `Neutral`.
    pub fn apply_event(
        &mut self,
        a: crate::FactionId,
        b: crate::FactionId,
        event: RelationEvent,
    ) -> RelationStance {
        if a == b {
            return RelationStance::Neutral;
        }
        let pair = FactionPair::new(a, b);
        let entry = self.pairs.entry(pair).or_default();
        let max = self.thresholds.opinion_max;
        let new_value = (i64::from(entry.opinion.value) + i64::from(event.delta()))
            .clamp(-i64::from(max), i64::from(max)) as i32;
        entry.opinion.value = new_value;
        entry.opinion.push_history(event.delta());
        entry.stance(&self.thresholds)
    }
}

/// Project an opinion value to a [`RelationStance`] using `thresholds`.
///
/// Pure function; same inputs always produce the same bucket.
fn stance_for(value: i32, thresholds: &StanceThresholds) -> RelationStance {
    if value <= thresholds.hostile_threshold {
        RelationStance::Hostile
    } else if value >= thresholds.ally_threshold {
        RelationStance::Ally
    } else {
        RelationStance::Neutral
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn f(id: u32) -> crate::FactionId {
        crate::FactionId::new(id)
    }

    fn thresholds() -> StanceThresholds {
        StanceThresholds {
            opinion_max: 1_000,
            hostile_threshold: -50,
            ally_threshold: 50,
        }
    }

    /// FR-CIV-DIPLO-STANCE: trade shifts warm opinion and a sequence of
    /// warming events crosses into the Ally bucket; raid shifts cool
    /// opinion and a sequence of cooling events crosses into the Hostile
    /// bucket. This proves the scalar → bucket projection responds to
    /// signed deltas at the configured thresholds.
    #[test]
    fn fr_civ_diplo_stance_opinion_shifts_move_across_buckets() {
        // Step 1: a single trade leaves the pair Neutral (small + delta).
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let a = f(1);
        let b = f(2);

        let after_one_trade =
            model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        assert_eq!(after_one_trade, RelationStance::Neutral);
        assert_eq!(model.stance(a, b), RelationStance::Neutral);

        // Step 2: enough additional trade events push the pair over the
        // ally threshold (+50). With 10/trade, five more brings the
        // running total to +60.
        for _ in 0..5 {
            model.apply_event(a, b, RelationEvent::Trade { delta: 10 });
        }
        assert_eq!(
            model.stance(a, b),
            RelationStance::Ally,
            "cumulative +60 trade should land in the Ally bucket"
        );

        // Step 3: a raid kicks the opinion downward. One raid of -40
        // leaves it Neutral (+20 after the prior +60, then -40 = +20).
        let after_raid = model.apply_event(a, b, RelationEvent::Raid { delta: -40 });
        assert_eq!(after_raid, RelationStance::Neutral);
        assert_eq!(model.stance(a, b), RelationStance::Neutral);

        // Step 4: more raids push the pair across the hostile threshold
        // (-50). Starting from +20, three -40 raids → -100 (clamped to
        // -1000), well below -50.
        for _ in 0..3 {
            model.apply_event(a, b, RelationEvent::Raid { delta: -40 });
        }
        assert_eq!(
            model.stance(a, b),
            RelationStance::Hostile,
            "cumulative cooling should land in the Hostile bucket"
        );

        // Step 5: the underlying opinion is clamped to `[-opinion_max, opinion_max]`.
        let rs = model.get(a, b).expect("pair tracked");
        assert!(rs.opinion.value >= -thresholds().opinion_max);
        assert!(rs.opinion.value <= thresholds().opinion_max);

        // Step 6: the audit history is bounded.
        assert!(rs.opinion.history.len() <= HISTORY_LIMIT);
    }

    /// Symmetry: applying the same event from either direction mutates
    /// the same canonical pair entry.
    #[test]
    fn relationship_stance_is_symmetric_in_storage() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        model.apply_event(f(3), f(4), RelationEvent::Trade { delta: 7 });
        let from_forward = model.get(f(3), f(4)).expect("forward");
        let from_reverse = model.get(f(4), f(3)).expect("reverse");
        assert_eq!(from_forward, from_reverse);
        assert_eq!(from_forward.opinion.value, 7);
    }

    /// Self-targeted events are no-ops.
    #[test]
    fn self_targeted_event_is_noop() {
        let mut model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        let result = model.apply_event(f(5), f(5), RelationEvent::Trade { delta: 999 });
        assert_eq!(result, RelationStance::Neutral);
        assert!(model.is_empty());
    }

    /// Unknown pairs default to Neutral without creating an entry.
    #[test]
    fn unknown_pair_is_neutral() {
        let model = RelationshipStanceModel::new(thresholds()).expect("valid config");
        assert_eq!(model.stance(f(1), f(2)), RelationStance::Neutral);
        assert!(model.is_empty());
    }

    /// Threshold validation rejects an overlapping config.
    #[test]
    fn invalid_thresholds_rejected() {
        let bad = StanceThresholds {
            opinion_max: 1_000,
            hostile_threshold: 10,
            ally_threshold: 5,
        };
        assert!(matches!(
            bad.validate(),
            Err(StanceConfigError::ThresholdsOverlap { .. })
        ));
    }

    /// Default thresholds validate.
    #[test]
    fn default_thresholds_validate() {
        assert!(StanceThresholds::default().validate().is_ok());
    }

    /// Bucket boundaries partition the integer line correctly.
    #[test]
    fn stance_for_partitions_thresholds() {
        let t = thresholds();
        assert_eq!(stance_for(-1_000, &t), RelationStance::Hostile);
        assert_eq!(stance_for(-50, &t), RelationStance::Hostile);
        assert_eq!(stance_for(-49, &t), RelationStance::Neutral);
        assert_eq!(stance_for(0, &t), RelationStance::Neutral);
        assert_eq!(stance_for(49, &t), RelationStance::Neutral);
        assert_eq!(stance_for(50, &t), RelationStance::Ally);
        assert_eq!(stance_for(1_000, &t), RelationStance::Ally);
    }
}