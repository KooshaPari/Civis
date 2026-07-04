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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct DiplomacySignal {
    /// Shared-resource pressure between clusters. Higher values push relations negative.
    pub resource_competition: f32,
    /// Exchange intensity between clusters. Higher values push relations positive.
    pub trade_volume: f32,
    /// Proximity pressure between clusters. Higher values mildly push relations negative.
    pub proximity: f32,
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
    /// Positive trade pushes the score upward. Competition and proximity push
    /// it downward. The record is clamped to keep the matrix stable.
    pub fn apply_signal(
        &mut self,
        a: ClusterId,
        b: ClusterId,
        signal: DiplomacySignal,
    ) -> DiplomacyOutcome {
        let key = Self::key(a, b);
        let entry = self.relations.entry(key).or_insert_with(RelationRecord::new);
        let before = Self::relation_kind(entry.score);

        let drift =
            signal.trade_volume * 0.08 - signal.resource_competition * 0.12 - signal.proximity * 0.04;
        entry.score = (entry.score + drift).clamp(-1.0, 1.0);
        entry.samples = entry.samples.saturating_add(1);

        DiplomacyOutcome {
            before,
            after: Self::relation_kind(entry.score),
            score: entry.score,
        }
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
