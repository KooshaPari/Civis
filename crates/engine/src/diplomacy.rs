//! Diplomacy subsystem for the simulation engine.
//!
//! This module contains types and helpers related to faction relations,
//! diplomacy events, and inter-faction signals.

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

/// Stub faction-relation matrix.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionRelations {
    rows: BTreeMap<(u32, u32), FactionRelationRecord>,
}

impl FactionRelations {
    pub fn apply_signal<A, B>(&mut self, a: A, b: B, signal: DiplomacySignal) -> DiplomacyOutcome
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

    pub fn record<A, B>(&self, a: A, b: B) -> Option<&FactionRelationRecord>
    where
        A: Into<u32>,
        B: Into<u32>,
    {
        self.rows.get(&(a.into(), b.into()))
    }

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

    pub fn iter_rows(&self) -> impl Iterator<Item = (&(u32, u32), &FactionRelationRecord)> {
        self.rows.iter()
    }

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

// ---- Simulation diplomacy methods (extracted from engine.rs) ----

use crate::engine::Fixed;
use crate::engine::Simulation;
use crate::engine::{faction_cluster_id, rollup_cluster_member_counts};
use rand::Rng;

impl Simulation {
    /// Phase hook for macro-level diplomacy events (FR-CIV-DIPLOMACY).
    /// Stub: full implementation pending faction_relations field.
    pub fn run_macro_diplomacy_event(&mut self) {}

    /// Emit a relation-threshold-crossing event (FR-CIV-DIPLOMACY).
    /// Stub: full implementation pending faction_relations field.
    pub fn emit_relation_threshold_event(
        &mut self,
        _faction_a: u32,
        _faction_b: u32,
        _outcome: civ_agents::DiplomacyOutcome,
    ) {
    }

    pub(crate) fn phase_diplomacy(&mut self) {
        if self.state.tick % 500 != 0 {
            return;
        }
        self.run_macro_diplomacy_event();
    }

    /// Apply an explicit player diplomacy command to the emergent relation substrate.
    #[must_use]
    pub fn apply_player_diplomacy_action(
        &mut self,
        source_faction: u32,
        target_faction: u32,
        kind: DiplomacyKind,
    ) -> Option<FactionRelationSnapshot> {
        if source_faction == target_faction
            || !self.state.factions.contains_key(&source_faction)
            || !self.state.factions.contains_key(&target_faction)
        {
            return None;
        }

        let signal = match kind {
            DiplomacyKind::TradeAgreement => DiplomacySignal {
                trade_volume: 1.0,
                need_complementarity: 0.5,
                ..DiplomacySignal::default()
            },
            DiplomacyKind::Conflict => DiplomacySignal {
                resource_competition: 0.5,
                combat_grievance: 1.0,
                ..DiplomacySignal::default()
            },
            DiplomacyKind::Peace => DiplomacySignal {
                trade_volume: 0.35,
                ..DiplomacySignal::default()
            },
        };

        let a = faction_cluster_id(source_faction);
        let b = faction_cluster_id(target_faction);
        let outcome = self.faction_relations.apply_signal(a, b, signal);
        self.emit_relation_threshold_event(source_faction, target_faction, outcome);
        self.diplomacy_events.push(DiplomacyEvent {
            tick: self.state.tick,
            faction_a: source_faction,
            faction_b: target_faction,
            kind,
        });
        let record = self
            .faction_relations
            .record(a, b)
            .expect("relation must exist after apply_signal");
        Some(FactionRelationSnapshot {
            faction_a: source_faction,
            faction_b: target_faction,
            score: record.score,
            kind: self.faction_relations.relation(a, b),
            samples: record.samples,
        })
    }

    /// Per-tick relation drift from proximity, competition, trade, religion, and combat.
    pub(crate) fn tick_faction_relation_drift(&mut self) {
        self.grief_accumulator.tick_decay();
        for eng in &self.last_tick_engagements {
            self.grief_accumulator
                .add_engagement(eng.shooter_faction, eng.target_faction);
        }

        let member_counts = rollup_cluster_member_counts(&self.world);
        let mut faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return;
        }
        let a = faction_ids[(self.state.tick as usize) % faction_ids.len()];
        let b = faction_ids[((self.state.tick as usize) + 1) % faction_ids.len()];
        let kind = if self.rng.gen_bool(0.6) {
            DiplomacyKind::TradeAgreement
        } else {
            DiplomacyKind::Conflict
        };
        match kind {
            DiplomacyKind::TradeAgreement => {
                if let Some(v) = self.state.faction_treasury.get_mut(&a) {
                    *v += Fixed::from_num(100);
                }
                if let Some(v) = self.state.faction_treasury.get_mut(&b) {
                    *v += Fixed::from_num(100);
                }
            }
            DiplomacyKind::Conflict => {
                if let Some(v) = self.state.faction_treasury.get_mut(&a) {
                    *v -= Fixed::from_num(50);
                }
                if let Some(v) = self.state.faction_treasury.get_mut(&b) {
                    *v -= Fixed::from_num(50);
                }
            }
            DiplomacyKind::Peace => {}
        }
        self.diplomacy_events.push(DiplomacyEvent {
            tick: self.state.tick,
            faction_a: a,
            faction_b: b,
            kind,
        });
    }
}
