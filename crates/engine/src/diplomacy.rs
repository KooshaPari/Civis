//! Diplomacy subsystem for the simulation engine.
//!
//! This module contains types and helpers related to faction relations,
//! diplomacy events, and inter-faction signals. The actual `phase_diplomacy`
//! method and player diplomacy actions remain in `engine.rs` as they require
//! `&mut Simulation` access.

use civ_agents::{DiplomacyOutcome, DiplomacySignal, RelationKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Snapshot of one pairwise faction-relation row (FR-CIV-DIPLOMACY).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionRelationSnapshot {
    pub faction_a: u32,
    pub faction_b: u32,
    pub score: f32,
    pub kind: String,
    pub samples: u32,
}

/// Stub per-pair faction-relation record (FR-CIV-DIPLOMACY).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionRelationRecord {
    pub score: f32,
    pub samples: u32,
}

/// Stub faction-relation matrix. Wraps a `BTreeMap<(u32, u32), f32>` with the
/// `apply_signal` / `record` / `relation` methods the diplomacy phase calls.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionRelations {
    rows: BTreeMap<(u32, u32), FactionRelationRecord>,
}

impl FactionRelations {
    /// Apply a [`DiplomacySignal`] to the `(a, b)` pair and
    /// return a deterministic [`DiplomacyOutcome`].
    pub fn apply_signal<A, B>(
        &mut self,
        a: A,
        b: B,
        signal: DiplomacySignal,
    ) -> DiplomacyOutcome
    where
        A: Into<u32>,
        B: Into<u32>,
    {
        let (a, b) = (a.into(), b.into());
        let entry = self.rows.entry((a, b)).or_default();
        entry.score =
            (entry.score + signal.trade_volume - signal.combat_grievance).clamp(-1.0, 1.0);
        entry.samples = entry.samples.saturating_add(1);
        DiplomacyOutcome {
            before: RelationKind::Neutral,
            after: RelationKind::Neutral,
            score: entry.score,
        }
    }

    /// Read-only access to the relation record for `(a, b)`.
    pub fn record<A, B>(&self, a: A, b: B) -> Option<&FactionRelationRecord>
    where
        A: Into<u32>,
        B: Into<u32>,
    {
        self.rows.get(&(a.into(), b.into()))
    }

    /// Map a relation score to a coarse string kind for snapshotting.
    #[must_use]
    pub fn relation<A, B>(&self, a: A, b: B) -> String
    where
        A: Into<u32>,
        B: Into<u32>,
    {
        let score = self
            .rows
            .get(&(a.into(), b.into()))
            .map(|r| r.score)
            .unwrap_or(0.0);
        if score > 0.5 {
            "allied".to_string()
        } else if score < -0.5 {
            "hostile".to_string()
        } else {
            "neutral".to_string()
        }
    }

    /// Iterate directed relation rows `(a, b) → record`.
    pub fn iter_rows(&self) -> impl Iterator<Item = (&(u32, u32), &FactionRelationRecord)> {
        self.rows.iter()
    }

    /// Mean score across every directed row that mentions `faction`.
    #[must_use]
    pub fn mean_score_involving(&self, faction: u32) -> Option<f32> {
        let mut total = 0.0_f32;
        let mut count = 0_u32;
        for ((a, b), record) in &self.rows {
            if *a == faction || *b == faction {
                total += record.score;
                count += 1;
            }
        }
        (count > 0).then_some(total / count as f32)
    }
}

/// Diplomacy event kind (FR-CIV-DIPLOMACY).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplomacyKind {
    TradeAgreement,
    Conflict,
    Peace,
}

/// A diplomacy event between two factions (FR-CIV-DIPLOMACY).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyEvent {
    pub tick: u64,
    pub faction_a: u32,
    pub faction_b: u32,
    pub kind: DiplomacyKind,
}
