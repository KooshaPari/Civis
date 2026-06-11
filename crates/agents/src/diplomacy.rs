//! Emergent inter-cluster diplomacy.
//!
//! This module tracks pairwise relations between emergent [`ClusterId`]s.
//! Relations are not hardcoded to factions; instead, they drift from observed
//! interactions such as resource competition, trade, and proximity pressure.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cluster::ClusterId;

/// Qualitative relation state derived from a pair score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationKind {
    /// Strong cooperative relation.
    Alliance,
    /// Positive but not fully allied relation.
    Trade,
    /// Neither cooperative nor hostile.
    Neutral,
    /// Negative relation with tension.
    Rivalry,
    /// Severe hostility.
    War,
}

/// Inputs that drive relation updates.
///
/// Six micro-driver signals per the CIV-007 spec. All fields are non-negative
/// magnitudes; direction is baked into [`DiplomacyMatrix::apply_signal`] via
/// the weight signs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct DiplomacySignal {
    /// Shared-resource pressure between clusters. Higher values push relations negative.
    pub resource_competition: f32,
    /// Exchange intensity between clusters. Higher values push relations positive.
    pub trade_volume: f32,
    /// Proximity pressure between clusters. Higher values mildly push relations negative.
    pub proximity: f32,
    /// Accumulated combat grievance this tick (from `last_tick_engagements`).
    /// Decays via [`GriefAccumulator`]; caller passes the current decayed value.
    /// Pushes relations strongly negative.
    pub combat_grievance: f32,
    /// How much faction A's surplus matches faction B's deficit across goods.
    /// High complementarity implies latent trade benefit; pushes relations positive.
    pub need_complementarity: f32,
    /// Energy-scarcity pressure derived from `energy_budget_joules` vs
    /// Subsistence demand. Positive when both factions are energy-scarce
    /// (sharpens competition). Negative (represented as a negative value here)
    /// when one faction has surplus that the other needs (pull toward cooperation).
    pub scarcity_pressure: f32,
}

/// Per-pair grievance accumulator with exponential decay (CIV-007 §2.2).
///
/// Decay rate is in units of "fraction lost per tick", i.e.
/// `grievance(t+1) = grievance(t) * (1 - decay_rate) + new_damage`.
/// Suggested range: 0.005–0.03 per tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GriefAccumulator {
    /// Per-pair grievance scores keyed by canonical `(min_id, max_id)` pair.
    pub pairs: std::collections::HashMap<(u32, u32), f32>,
    /// Exponential decay rate applied every tick (suggested: 0.01).
    pub decay_rate: f32,
    /// Weight per engagement added to the accumulator (suggested: 0.05).
    pub engagement_weight: f32,
}

impl GriefAccumulator {
    /// Create a default accumulator with criticality-safe defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pairs: std::collections::HashMap::new(),
            decay_rate: 0.01,
            engagement_weight: 0.05,
        }
    }

    fn key(a: u32, b: u32) -> (u32, u32) {
        if a <= b { (a, b) } else { (b, a) }
    }

    /// Advance decay for all pairs without adding new engagements (called at
    /// the top of every `phase_diplomacy` tick).
    pub fn tick_decay(&mut self) {
        let decay = 1.0 - self.decay_rate;
        for v in self.pairs.values_mut() {
            *v *= decay;
            if *v < 1e-5 {
                *v = 0.0;
            }
        }
    }

    /// Record engagements for the tick.  `shooter` and `target` are faction ids.
    pub fn add_engagement(&mut self, shooter: u32, target: u32) {
        let key = Self::key(shooter, target);
        let entry = self.pairs.entry(key).or_insert(0.0);
        *entry += self.engagement_weight;
    }

    /// Current grievance for a faction pair (symmetric).
    #[must_use]
    pub fn get(&self, a: u32, b: u32) -> f32 {
        self.pairs.get(&Self::key(a, b)).copied().unwrap_or(0.0)
    }
}

/// Stored relation record for a cluster pair.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RelationRecord {
    /// Continuous score in the range roughly `[-1.0, 1.0]`.
    pub score: f32,
    /// Number of samples folded into this record.
    pub samples: u32,
}

impl RelationRecord {
    fn new() -> Self {
        Self {
            score: 0.0,
            samples: 0,
        }
    }
}

/// Result of applying one diplomacy update.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyOutcome {
    /// Relation kind before the update.
    pub before: RelationKind,
    /// Relation kind after the update.
    pub after: RelationKind,
    /// Updated relation score.
    pub score: f32,
}

/// Symmetric relation matrix over emergent clusters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DiplomacyMatrix {
    relations: HashMap<(ClusterId, ClusterId), RelationRecord>,
}

impl DiplomacyMatrix {
    /// Creates an empty relation matrix.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn key(a: ClusterId, b: ClusterId) -> (ClusterId, ClusterId) {
        if a.0 <= b.0 {
            (a, b)
        } else {
            (b, a)
        }
    }

    fn relation_kind(score: f32) -> RelationKind {
        if score >= 0.60 {
            RelationKind::Alliance
        } else if score >= 0.20 {
            RelationKind::Trade
        } else if score <= -0.60 {
            RelationKind::War
        } else if score <= -0.20 {
            RelationKind::Rivalry
        } else {
            RelationKind::Neutral
        }
    }

    /// Returns the stored relation record for a pair, if any.
    #[must_use]
    pub fn record(&self, a: ClusterId, b: ClusterId) -> Option<RelationRecord> {
        self.relations.get(&Self::key(a, b)).copied()
    }

    /// Returns the current qualitative relation between two clusters.
    #[must_use]
    pub fn relation(&self, a: ClusterId, b: ClusterId) -> RelationKind {
        self.record(a, b)
            .map(|record| Self::relation_kind(record.score))
            .unwrap_or(RelationKind::Neutral)
    }

    /// Applies a new interaction signal to a pair and returns the outcome.
    ///
    /// Drift equation (CIV-007 §1.3):
    /// ```text
    /// drift = trade_volume    × W_trade      (0.08)
    ///       - competition     × W_compete    (0.12)
    ///       - proximity       × W_border     (0.04)
    ///       - combat_grievance× W_grievance  (0.18)
    ///       + complementarity × W_complement (0.06)
    ///       - scarcity_pressure × W_scarcity (0.10)
    /// ```
    /// `scarcity_pressure` may be negative (one-surplus-one-deficit case),
    /// which would then add to the score rather than subtract — correct by
    /// design, representing a pull toward cooperation.
    pub fn apply_signal(
        &mut self,
        a: ClusterId,
        b: ClusterId,
        signal: DiplomacySignal,
    ) -> DiplomacyOutcome {
        const W_TRADE: f32 = 0.08;
        const W_COMPETE: f32 = 0.12;
        const W_BORDER: f32 = 0.04;
        const W_GRIEVANCE: f32 = 0.18;
        const W_COMPLEMENT: f32 = 0.06;
        const W_SCARCITY: f32 = 0.10;

        let key = Self::key(a, b);
        let entry = self
            .relations
            .entry(key)
            .or_insert_with(RelationRecord::new);
        let before = Self::relation_kind(entry.score);

        let drift = signal.trade_volume * W_TRADE
            - signal.resource_competition * W_COMPETE
            - signal.proximity * W_BORDER
            - signal.combat_grievance * W_GRIEVANCE
            + signal.need_complementarity * W_COMPLEMENT
            - signal.scarcity_pressure * W_SCARCITY;
        entry.score = (entry.score + drift).clamp(-1.0, 1.0);
        entry.samples = entry.samples.saturating_add(1);

        DiplomacyOutcome {
            before,
            after: Self::relation_kind(entry.score),
            score: entry.score,
        }
    }

    /// Shannon entropy over the five [`RelationKind`] buckets across all pairs
    /// (CIV-007 §4.3).  `H = 0` means all pairs are in the same state;
    /// `H ≈ 2.32` is maximum diversity.  Target operating range: `[1.5, 2.1]`.
    #[must_use]
    pub fn trust_entropy(&self) -> f32 {
        if self.relations.is_empty() {
            return 0.0;
        }
        let total = self.relations.len() as f32;
        let mut counts = [0u32; 5]; // Alliance, Trade, Neutral, Rivalry, War
        for record in self.relations.values() {
            let idx = match Self::relation_kind(record.score) {
                RelationKind::Alliance => 0,
                RelationKind::Trade => 1,
                RelationKind::Neutral => 2,
                RelationKind::Rivalry => 3,
                RelationKind::War => 4,
            };
            counts[idx] += 1;
        }
        counts
            .iter()
            .filter(|&&c| c > 0)
            .map(|&c| {
                let p = c as f32 / total;
                -p * p.log2()
            })
            .sum()
    }

    /// Returns a sorted snapshot of all stored relations.
    #[must_use]
    pub fn snapshot(&self) -> Vec<(ClusterId, ClusterId, RelationRecord)> {
        let mut rows: Vec<(ClusterId, ClusterId, RelationRecord)> = self
            .relations
            .iter()
            .map(|(&(a, b), &record)| (a, b, record))
            .collect();
        rows.sort_by_key(|(a, b, _)| (a.0, b.0));
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_is_symmetric_and_empty_defaults_neutral() {
        let matrix = DiplomacyMatrix::new();
        let a = ClusterId(10);
        let b = ClusterId(20);

        assert_eq!(matrix.relation(a, b), RelationKind::Neutral);
        assert_eq!(matrix.relation(b, a), RelationKind::Neutral);
        assert_eq!(matrix.record(a, b), None);
    }

    #[test]
    fn trade_emerges_toward_alliance() {
        let mut matrix = DiplomacyMatrix::new();
        let a = ClusterId(1);
        let b = ClusterId(2);

        let mut last = None;
        for _ in 0..20 {
            last = Some(matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    trade_volume: 1.0,
                    ..Default::default()
                },
            ));
        }

        let outcome = last.expect("outcome present");
        assert!(outcome.score > 0.6);
        assert_eq!(outcome.after, RelationKind::Alliance);
        assert_eq!(matrix.relation(a, b), RelationKind::Alliance);
        assert_eq!(matrix.relation(b, a), RelationKind::Alliance);
    }

    #[test]
    fn competition_and_proximity_can_drive_war() {
        let mut matrix = DiplomacyMatrix::new();
        let a = ClusterId(7);
        let b = ClusterId(11);

        for _ in 0..25 {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    resource_competition: 1.0,
                    proximity: 0.5,
                    ..Default::default()
                },
            );
        }

        assert_eq!(matrix.relation(a, b), RelationKind::War);
        let record = matrix.record(a, b).expect("record present");
        assert!(record.score <= -0.6);
        assert!(record.samples >= 25);
    }

    #[test]
    fn mixed_interactions_remain_stable_and_clamped() {
        let mut matrix = DiplomacyMatrix::new();
        let a = ClusterId(3);
        let b = ClusterId(4);

        for _ in 0..100 {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    trade_volume: 2.0,
                    resource_competition: 0.5,
                    proximity: 0.5,
                },
            );
        }

        let record = matrix.record(a, b).expect("record present");
        assert!((-1.0..=1.0).contains(&record.score));
        assert_eq!(matrix.relation(a, b), RelationKind::Alliance);

        let snapshot = matrix.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].0, a);
        assert_eq!(snapshot[0].1, b);
    }
}
