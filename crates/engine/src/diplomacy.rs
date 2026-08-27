//! Diplomacy subsystem for the simulation engine.
//!
//! This module contains types and helpers related to faction relations,
//! diplomacy events, and inter-faction signals.

use civ_agents::{DiplomacyOutcome, DiplomacySignal, RelationKind};
use civ_diplomacy::{
    AllianceManager, AlliancePurpose, CulturalExchange, CulturalVectors, PeaceNegotiation,
    PolityId as DipPolityId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Deep diplomacy state (alliance formation, peace negotiations, cultural assimilation)
// ---------------------------------------------------------------------------

/// State container for the deep diplomacy subsystems.
/// Holds the alliance manager, active peace negotiations, and cultural exchanges.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeepDiplomacyState {
    /// Alliance formation manager.
    pub alliance_manager: AllianceManager,
    /// Active peace negotiations keyed by (proposer_id, target_id).
    pub active_negotiations: BTreeMap<(u32, u32), PeaceNegotiation>,
    /// Active cultural exchanges keyed by (source_id, target_id).
    pub active_exchanges: BTreeMap<(u32, u32), CulturalExchange>,
    /// Per-faction cultural vectors for assimilation tracking.
    pub faction_cultures: BTreeMap<u32, CulturalVectors>,
    /// Per-faction resource counts (fixed-point, scaled x100).
    pub faction_resources: BTreeMap<u32, i32>,
    /// Active wars: pairs of factions currently at war.
    pub active_wars: BTreeSet<(u32, u32)>,
    /// Conflict start ticks for active wars.
    pub war_start_ticks: BTreeMap<(u32, u32), u64>,
    /// Cumulative casualties per faction in active wars.
    pub war_casualties: BTreeMap<u32, u32>,
}

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
    /// Map a continuous relation score to a qualitative [`RelationKind`].
    /// Thresholds mirror the canonical buckets from `DiplomacyMatrix::relation_kind`
    /// in `civ-agents` for cross-layer consistency.
    fn score_to_kind(score: f32) -> RelationKind {
        if score < -0.5 {
            RelationKind::War
        } else if score < -0.2 {
            RelationKind::Rivalry
        } else if score < 0.2 {
            RelationKind::Neutral
        } else if score < 0.5 {
            RelationKind::Trade
        } else {
            RelationKind::Alliance
        }
    }

    pub fn apply_signal<A, B>(&mut self, a: A, b: B, signal: DiplomacySignal) -> DiplomacyOutcome
    where
        A: Into<u32>,
        B: Into<u32>,
    {
        let (a, b) = (a.into(), b.into());
        let entry = self.rows.entry((a, b)).or_default();
        let before = Self::score_to_kind(entry.score);
        entry.score =
            (entry.score + signal.trade_volume - signal.combat_grievance).clamp(-1.0, 1.0);
        entry.samples = entry.samples.saturating_add(1);
        DiplomacyOutcome {
            before,
            after: Self::score_to_kind(entry.score),
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
    ///
    /// Runs every 500 ticks via [`Self::phase_diplomacy`].  Performs three things:
    /// 1. Calls [`Self::tick_faction_relation_drift`] for per-tick grief decay,
    ///    engagement tracking, treasury adjustments, and a random pair interaction.
    /// 2. Iterates every faction pair, applies an ambient trade/grievance signal
    ///    to the relation matrix, and detects qualitative threshold crossings.
    /// 3. Emits [`EmergenceFeedEvent`]s for any threshold changes so the saga
    ///    graph and HUD event feed stay current.
    pub fn run_macro_diplomacy_event(&mut self) {
        // 1. Per-tick drift: grief decay, engagement tracking, treasury adjustments.
        self.tick_faction_relation_drift();

        // 2. Iterate all faction pairs and apply ambient interaction signals.
        let faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        if faction_ids.len() < 2 {
            return;
        }
        for i in 0..faction_ids.len() {
            for j in (i + 1)..faction_ids.len() {
                let a = faction_ids[i];
                let b = faction_ids[j];

                // Ambient interaction: modest trade flow, occasional grievance.
                let trade = self.rng.gen_range(0.0..0.3);
                let grievance = self.rng.gen_range(0.0..0.2);
                let signal = DiplomacySignal {
                    trade_volume: trade,
                    combat_grievance: grievance,
                    ..DiplomacySignal::default()
                };

                let outcome = self.faction_relations.apply_signal(a, b, signal);

                // 3. Emit an emergence feed event on qualitative threshold crossings.
                if outcome.before != outcome.after {
                    self.emit_relation_threshold_event(a, b, outcome);
                }
            }
        }
    }

    /// Emit a relation-threshold-crossing event (FR-CIV-DIPLOMACY).
    ///
    /// Pushes an [`EmergenceFeedEvent`] with the kind `"relation_shift"` so it
    /// surfaces in the HUD event feed and can be ingested by the legends saga
    /// graph downstream.
    pub fn emit_relation_threshold_event(
        &mut self,
        faction_a: u32,
        faction_b: u32,
        outcome: civ_agents::DiplomacyOutcome,
    ) {
        let tick = self.state.tick;
        let summary = format!(
            "factions {faction_a} and {faction_b}: {:?} -> {:?} (score {:.3})",
            outcome.before, outcome.after, outcome.score,
        );
        self.emergence
            .last_feed
            .push(crate::emergence::EmergenceFeedEvent {
                tick,
                kind: "relation_shift".to_string(),
                summary,
                agent_id: None,
            });
    }

    pub(crate) fn phase_diplomacy(&mut self) {
        if self.state.tick % 500 != 0 {
            return;
        }
        self.run_macro_diplomacy_event();

        // Deep diplomacy: alliance checks every 1000 ticks.
        if self.state.tick % 1000 == 0 {
            self.tick_deep_diplomacy_alliances();
        }
        // Deep diplomacy: peace negotiations every 500 ticks.
        self.tick_deep_diplomacy_peace();
        // Deep diplomacy: cultural assimilation is continuous (every 500 tick phase).
        self.tick_deep_diplomacy_assimilation();
    }

    /// Evaluate and form/dissolve alliances based on faction compatibility.
    /// Runs every 1000 ticks.
    pub(crate) fn tick_deep_diplomacy_alliances(&mut self) {
        let faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        if faction_ids.len() < 2 {
            return;
        }
        let tick = self.state.tick;

        // Check all faction pairs for potential alliance formation.
        for i in 0..faction_ids.len() {
            for j in (i + 1)..faction_ids.len() {
                let a = faction_ids[i];
                let b = faction_ids[j];

                // Check if already allied (either direction).
                let already_allied = self
                    .deep_diplomacy
                    .alliance_manager
                    .alliances_for(DipPolityId::new(a))
                    .iter()
                    .any(|alliance| alliance.members.contains(&DipPolityId::new(b)));
                if already_allied {
                    continue;
                }

                // Compute criteria from existing relation matrix.
                let record = self.faction_relations.record(a, b);
                let score = record.map(|r| r.score).unwrap_or(0.0);
                let shared_enemy_score = if score < -0.5 { 500 } else { 0 };
                let trade_volume_score = ((score + 1.0) * 500.0) as i32;
                let cultural_similarity_score = (self
                    .deep_diplomacy
                    .faction_cultures
                    .get(&a)
                    .zip(self.deep_diplomacy.faction_cultures.get(&b))
                    .map(|(ca, cb)| {
                        let dist = civ_diplomacy::cultural_distance(ca, cb);
                        10_000 - dist.min(10_000)
                    })
                    .unwrap_or(5000)) as i32;
                let res_a = self
                    .deep_diplomacy
                    .faction_resources
                    .get(&a)
                    .copied()
                    .unwrap_or(0);
                let res_b = self
                    .deep_diplomacy
                    .faction_resources
                    .get(&b)
                    .copied()
                    .unwrap_or(0);
                let combined_strength = res_a + res_b;

                let criteria = civ_diplomacy::AllianceProposalCriteria {
                    shared_enemy_score,
                    trade_volume_score,
                    cultural_similarity_score,
                    combined_strength,
                };

                if self
                    .deep_diplomacy
                    .alliance_manager
                    .evaluate_alliance_proposal(&criteria)
                {
                    let mut members = BTreeSet::new();
                    members.insert(DipPolityId::new(a));
                    members.insert(DipPolityId::new(b));
                    let _ = self.deep_diplomacy.alliance_manager.form_alliance(
                        members,
                        AlliancePurpose::Military,
                        tick,
                    );

                    self.diplomacy_events.push(DiplomacyEvent {
                        tick,
                        faction_a: a,
                        faction_b: b,
                        kind: DiplomacyKind::Peace,
                    });
                }
            }
        }
    }

    /// Evaluate active peace negotiations and advance or resolve them.
    /// Runs every 500 ticks.
    pub(crate) fn tick_deep_diplomacy_peace(&mut self) {
        let tick = self.state.tick;
        let completed: Vec<(u32, u32)> = self
            .deep_diplomacy
            .active_negotiations
            .keys()
            .copied()
            .collect();

        for (a, b) in completed {
            if let Some(neg) = self.deep_diplomacy.active_negotiations.get(&(a, b)) {
                let wear_a = *self.deep_diplomacy.war_casualties.get(&a).unwrap_or(&0) as i32 * 10;
                let wear_b = *self.deep_diplomacy.war_casualties.get(&b).unwrap_or(&0) as i32 * 10;
                let res_a = self
                    .deep_diplomacy
                    .faction_resources
                    .get(&a)
                    .copied()
                    .unwrap_or(0);
                let res_b = self
                    .deep_diplomacy
                    .faction_resources
                    .get(&b)
                    .copied()
                    .unwrap_or(0);
                let strength_ratio = if res_b > 0 {
                    (res_a * 100) / res_b
                } else {
                    i32::MAX
                };
                if neg.evaluate_peace(wear_a, wear_b, strength_ratio) {
                    if let Some(neg_mut) = self.deep_diplomacy.active_negotiations.get_mut(&(a, b))
                    {
                        let _ = neg_mut.accept();
                    }
                }
            }
        }

        // Remove resolved negotiations and end wars.
        let resolved: Vec<(u32, u32)> = self
            .deep_diplomacy
            .active_negotiations
            .iter()
            .filter(|(_, neg)| {
                neg.status == civ_diplomacy::PeaceNegotiationStatus::Accepted
                    || neg.status == civ_diplomacy::PeaceNegotiationStatus::Rejected
            })
            .map(|(&(a, b), _)| (a, b))
            .collect();
        for (a, b) in resolved {
            self.deep_diplomacy.active_negotiations.remove(&(a, b));
            self.deep_diplomacy.active_wars.remove(&(a, b));
            self.deep_diplomacy.active_wars.remove(&(b, a));
        }
    }

    /// Apply cultural assimilation between neighboring factions.
    /// Runs continuously (every 500-tick phase).
    pub(crate) fn tick_deep_diplomacy_assimilation(&mut self) {
        let faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        if faction_ids.len() < 2 {
            return;
        }
        let tick = self.state.tick;

        for i in 0..faction_ids.len() {
            for j in (i + 1)..faction_ids.len() {
                let a = faction_ids[i];
                let b = faction_ids[j];
                let key = (a, b);

                // Skip if an exchange is already active.
                if self.deep_diplomacy.active_exchanges.contains_key(&key) {
                    continue;
                }

                // Check if factions are allied (allow assimilation for allied pairs).
                let allied = self
                    .deep_diplomacy
                    .alliance_manager
                    .alliances_for(DipPolityId::new(a))
                    .iter()
                    .any(|alliance| alliance.members.contains(&DipPolityId::new(b)));
                if !allied {
                    continue;
                }

                // Create a cultural exchange for allied pairs.
                if let (Some(ca), Some(cb)) = (
                    self.deep_diplomacy.faction_cultures.get(&a).copied(),
                    self.deep_diplomacy.faction_cultures.get(&b).copied(),
                ) {
                    let dist = civ_diplomacy::cultural_distance(&ca, &cb);
                    if dist > 0 {
                        let intensity = (10_000 - dist.min(10_000)) / 10; // 0-1000
                        if let Ok(exchange) = CulturalExchange::new(
                            DipPolityId::new(a),
                            DipPolityId::new(b),
                            intensity,
                            ca,
                            tick,
                        ) {
                            let (new_cb, _) = exchange.apply_assimilation(&ca, &cb);
                            self.deep_diplomacy.faction_cultures.insert(b, new_cb);
                            self.deep_diplomacy.active_exchanges.insert(key, exchange);
                        }
                    }
                }
            }
        }
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
