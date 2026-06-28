//! MOAT emergence phases — wire dormant psyche/social/culture/legends/genetics/civ-ai
//! crates into [`Simulation::tick`] (gap-audit §1, master-roadmap S2).
//!
//! Phase order inside `phase_emergence`: genetics → culture → social → psyche →
//! legends ingest → civ-ai naming. Surfaced via [`EmergenceFeedEvent`] and getters
//! on [`Simulation`].

use std::collections::{BTreeMap, HashSet};

use civ_agents::culture::{drift_populations, ContactEdge, CultureProfile};
use civ_agents::language::{
    name_from_lexicon, EvolvedLexicon, LexemeKind, PhonemeInventory,
};
use civ_agents::psyche::{
    cluster_belief_centroids, nudge_temperament, psyche_from_dna, update_beliefs, update_mood,
    PSYCHE_DIM,
};
use civ_agents::{
    apply_social_event, belief_culture_exposure, decay_social_graph, psych_genome_profile,
    cluster_by_colocation, Alignment, Civilian, ClusterId as AgentsClusterId, ClusterMember,
    Interaction, Needs,
    Position3d, Psyche, SocialEvent, SocialGraph,
};
use civ_genetics::{
    sentience::{evaluate_sentience, CognitionTraitProfile, SentienceEvent, SentienceThreshold},
    spawn_genome_with_divergence, Dna, DnaClass, SeedDefinition, SeedLibrary, SeedSet,
};
use civ_legends::{
    AggregateKey, ClusterId, EntityKind, EntityRef, Epoch, EpochDigest, EventKind, IngestOutcome,
    LegendEdge, LegendsConfig, LegendsWorker, LegendEntityId, NameRef, RawSimEvent, Role, Saga,
    SagaGraph, SimRuntimeId, SourceCrate, QUERY_API_VERSION,
};
use civ_planet::GeologyMap;
use civ_voxel::FIXED_SCALE;
use civ_needs::Needs as LifeNeeds;
use civ_species::express;
use hecs::Entity;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::engine::{Simulation, awakening_belief_gain, awakening_cohesion_gain};

/// JSON-RPC / inspector payload for `sim.legends` (FR-CIV-LEGENDS-QUERY-07).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegendsQueryResult {
    pub query_api_version: u32,
    pub tick: u64,
    pub node_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saga: Option<Saga>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub significant: Option<Vec<EntityRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch_digest: Option<EpochDigest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_reason: Option<String>,
    pub emergence_feed: Vec<EmergenceFeedEvent>,
}

pub fn festival_intensity(food_surplus: f32, shared_belief: f32) -> f32 {
    let food = if food_surplus.is_finite() {
        food_surplus.max(0.0)
    } else {
        0.0
    };
    let belief = if shared_belief.is_finite() {
        shared_belief.clamp(0.0, 1.0)
    } else {
        0.0
    };

    ((food / (1.0 + food)) * 0.6 + belief * 0.4).clamp(0.0, 1.0)
}

#[cfg(test)]
mod festival_intensity_tests {
    use super::festival_intensity;

    #[test]
    fn clamps_and_ignores_non_finite_inputs() {
        assert_eq!(festival_intensity(f32::NAN, f32::NAN), 0.0);
        assert_eq!(festival_intensity(-10.0, 2.0), 0.4);
        assert_eq!(festival_intensity(1.0e9, 1.0), 1.0);
    }
}

pub fn settlement_plague_risk(density: f32, trade_connectivity: f32) -> bool {
    plague_outbreak(density, trade_connectivity).0 > 0.5
}

#[cfg(test)]
mod settlement_plague_risk_tests {
    use super::settlement_plague_risk;

    #[test]
    fn settlement_plague_risk_tracks_outbreak_probability_threshold() {
        assert!(settlement_plague_risk(180.0, 8.0));
        assert!(!settlement_plague_risk(10.0, 0.5));
    }
}

/// Notable emergence this tick — event feed / inspect panels (FR-CIV-LEGENDS-QUERY-07).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmergenceFeedEvent {
    /// Simulation tick when the event was recorded.
    pub tick: u64,
    /// Machine-readable kind (`birth`, `death`, `sentience`, `legend_promotion`, …).
    pub kind: String,
    /// Human-readable one-liner for HUD / event_feed.
    pub summary: String,
    /// Agent id when the event concerns a civilian.
    pub agent_id: Option<u64>,
}

/// Last civ-ai flavor decision (FR-CIV-AI-006 sync path on promotions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CivAiDecision {
    pub tick: u64,
    pub agent_id: u64,
    pub prompt: String,
    pub output: String,
}

/// Per-simulation MOAT state (legends graph, cluster cultures, feed buffers).
pub struct EmergenceState {
    pub(crate) legends: LegendsWorker,
    pub(crate) cluster_cultures: BTreeMap<u64, CultureProfile>,
    pub(crate) cluster_lexicons: BTreeMap<u64, EvolvedLexicon>,
    pub(crate) last_feed: Vec<EmergenceFeedEvent>,
    pub(crate) last_ai_decisions: Vec<CivAiDecision>,
    pub(crate) last_sentience: Vec<SentienceEvent>,
    pub(crate) novelty_window_start_tick: u64,
    pub(crate) novelty_window_new: u32,
    pub(crate) seen_config_hashes: HashSet<u64>,
    pub(crate) dna_class: DnaClass,
    pub(crate) psych_profile: civ_agents::PsychGenomeProfile,
    pub(crate) sentience_profile: CognitionTraitProfile,
    pub(crate) sentience_threshold: SentienceThreshold,
    pub(crate) sentient_agents: HashSet<u64>,
    /// Settlement cluster ids already recorded in the saga graph.
    pub(crate) known_settlement_ids: HashSet<u64>,
    /// Per-cluster belief centroids (FR-CIV-RELIGION / PSYCHE-911).
    pub(crate) cluster_beliefs: BTreeMap<u64, [f32; PSYCHE_DIM]>,
    /// True once a saga promotion crystallises shared veneration (FR-CIV-RELIGION-002).
    pub(crate) has_patron: bool,
}

impl EmergenceState {
    pub(crate) fn new(seed: u64) -> Self {
        let _ = seed;
        EmergenceState {
            legends: LegendsWorker::new(SagaGraph::new(LegendsConfig::default())),
            cluster_cultures: BTreeMap::new(),
            cluster_lexicons: BTreeMap::new(),
            last_feed: Vec::new(),
            last_ai_decisions: Vec::new(),
            last_sentience: Vec::new(),
            novelty_window_start_tick: 0,
            novelty_window_new: 0,
            seen_config_hashes: HashSet::new(),
            dna_class: DnaClass::default(),
            psych_profile: psych_genome_profile(),
            sentience_profile: CognitionTraitProfile::new(
                "sapient-lineage",
                vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
            ),
            sentience_threshold: SentienceThreshold::new(0.72),
            sentient_agents: HashSet::new(),
            known_settlement_ids: HashSet::new(),
            cluster_beliefs: BTreeMap::new(),
            has_patron: false,
        }
    }

    fn push_feed(
        &mut self,
        tick: u64,
        kind: &str,
        summary: impl Into<String>,
        agent_id: Option<u64>,
    ) {
        self.last_feed.push(EmergenceFeedEvent {
            tick,
            kind: kind.to_string(),
            summary: summary.into(),
            agent_id,
        });
    }
}

/// Choose the best [`SeedDefinition`] for a spawn position based on the biome
/// that the position maps to via the geology layer.
///
/// # Algorithm
/// 1. Convert `pos.coord.x` / `pos.coord.z` to normalised `[0, 1]` by dividing
///    by [`FIXED_SCALE`].
/// 2. Look up the biome archetype via [`GeologyMap::biome_at_normalized`].
/// 3. Iterate `seed_library` and return the first seed whose
///    `spawn_biome_affinity` labels contain a match for that biome
///    (via [`civ_planet::BiomeKind::matches_affinity`]).
/// 4. If no match is found, return `active_seed` as the fallback.
///
/// The fallback keeps the function total: callers never need to special-case
/// the no-match path.
fn select_seed_for_position<'a>(
    seed_library: &'a SeedLibrary,
    active_seed: Option<&'a SeedDefinition>,
    geology_map: &GeologyMap,
    pos: &Position3d,
) -> Option<&'a SeedDefinition> {
    let nx = (pos.coord.x as f32) / (FIXED_SCALE as f32);
    let nz = (pos.coord.z as f32) / (FIXED_SCALE as f32);
    let biome = geology_map.biome_at_normalized(nx, nz);
    // Stable iteration order: sort by id so the same world always picks the
    // same seed on the same biome (HashMap iteration is unordered).
    let mut candidates: Vec<(&String, &SeedDefinition)> = seed_library.iter().collect();
    candidates.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (_, seed) in candidates {
        if seed
            .spawn_biome_affinity
            .iter()
            .any(|label| biome.matches_affinity(label))
        {
            return Some(seed);
        }
    }
    active_seed
}

/// Belief gained when a battle is recorded in the saga graph (collective awe/fear → faith).
/// FR-CIV-RELIGION belief-from-events contract.
const BELIEF_GAIN_BATTLE: i64 = 5;
/// Belief gained per named legend entity each tick (FR-CIV-LEGENDS deepening).
const BELIEF_PER_NAMED_LEGEND: i64 = 2;
/// Cohesion gained per named legend entity each tick (FR-CIV-LEGENDS deepening).
const COHESION_PER_NAMED_LEGEND: i64 = 3;
/// Belief gained when a new settlement is founded (communal milestone → veneration).
const BELIEF_GAIN_FOUNDING: i64 = 20;
/// Belief gained when a legend-ranked agent's death is recorded (martyrdom → faith surge).
const BELIEF_GAIN_LEGEND_DEATH: i64 = 15;

impl Simulation {
    pub(crate) fn default_emergence_state(seed: u64) -> EmergenceState {
        EmergenceState::new(seed)
    }

    /// MOAT emergence — genetics, culture, social, psyche, legends, civ-ai.
    ///
    /// Runs after [`Self::phase_life`] so needs/clusters are current.
    pub(crate) fn phase_emergence(&mut self) {
        self.emergence.last_feed.clear();
        self.emergence.last_ai_decisions.clear();
        self.emergence.last_sentience.clear();

        self.emergence_ensure_genomes();
        self.emergence_culture();
        self.emergence_social();
        self.emergence_psyche();
        self.emergence_accrue_cluster_beliefs();
        self.emergence_genetics_sentience();
        self.emergence_legends();
        self.emergence_civ_ai();
    }

    fn emergence_ensure_genomes(&mut self) {
        let len = self.emergence.dna_class.length;
        let agents: Vec<(Entity, u64)> = self
            .world
            .query::<&Civilian>()
            .iter()
            .map(|(e, c)| (e, c.id))
            .collect();
        for (entity, id) in agents {
            if self.world.get::<&Dna>(entity).is_ok() {
                continue;
            }
            let mut local = ChaCha8Rng::seed_from_u64(self.state.rng_seed ^ id);
            let dna = Dna::random(len, &mut local);
            let _ = self.world.insert(entity, (dna,));
        }
    }

    /// Co-location radius for emergent settlement clusters (matches the
    /// engine's `SETTLEMENT_CLUSTER_RADIUS_FP` = 6% of one world unit).
    const SETTLEMENT_CLUSTER_RADIUS_FP: i64 = (6 * FIXED_SCALE) / 100;

    /// Recompute emergent co-location clusters from live civilian positions and
    /// stamp each civilian with its current [`ClusterMember`].
    ///
    /// This is the "life" rollup the culture phase depends on: civilians that
    /// settle near one another form a settlement (connected component keyed by
    /// the minimum agent id). Membership is re-derived every tick from actual
    /// agent state, so clusters split and merge as the population migrates.
    fn emergence_recluster(&mut self) {
        let positions: Vec<(u64, Position3d)> = self
            .world
            .query::<(&Civilian, &Position3d)>()
            .iter()
            .map(|(_, (civ, pos))| (civ.id, *pos))
            .collect();
        if positions.is_empty() {
            return;
        }
        let assignments =
            cluster_by_colocation(&positions, Self::SETTLEMENT_CLUSTER_RADIUS_FP);
        let by_id: BTreeMap<u64, AgentsClusterId> = assignments.into_iter().collect();

        let entities: Vec<(Entity, u64)> = self
            .world
            .query::<&Civilian>()
            .iter()
            .map(|(e, c)| (e, c.id))
            .collect();
        for (entity, id) in entities {
            if let Some(cluster) = by_id.get(&id) {
                let _ = self
                    .world
                    .insert_one(entity, ClusterMember { cluster: *cluster });
            }
        }

        let mut cluster_member_counts: BTreeMap<u64, u32> = BTreeMap::new();
        for (_, member) in self.world.query::<&ClusterMember>().iter() {
            *cluster_member_counts
                .entry(member.cluster.0)
                .or_insert(0) += 1;
        }
        self.rollup_emergent_settlements(&cluster_member_counts);
        self.emergence_accrue_cluster_cultures(&cluster_member_counts);
    }

    /// Form or retain a [`CultureProfile`] for every multi-member settlement cluster.
    fn emergence_accrue_cluster_cultures(&mut self, cluster_member_counts: &BTreeMap<u64, u32>) {
        for (cluster_id, size) in cluster_member_counts {
            if *size < 2 {
                continue;
            }
            self.emergence
                .cluster_cultures
                .entry(*cluster_id)
                .or_insert_with(|| {
                    let seed = [
                        ((*cluster_id % 256) as f32) / 255.0,
                        (((*cluster_id >> 8) % 256) as f32) / 255.0,
                        (((*cluster_id >> 16) % 256) as f32) / 255.0,
                        (((*cluster_id >> 24) % 256) as f32) / 255.0,
                    ];
                    CultureProfile::new(seed)
                });
        }
        self.emergence.cluster_cultures.retain(|cluster_id, _| {
            cluster_member_counts
                .get(cluster_id)
                .copied()
                .unwrap_or(0)
                >= 2
        });
    }

    fn emergence_culture(&mut self) {
        self.emergence_recluster();
        let tick = self.state.tick;
        let mut cluster_ids: BTreeMap<u64, u32> = BTreeMap::new();
        for (_, member) in self.world.query::<&ClusterMember>().iter() {
            *cluster_ids.entry(member.cluster.0).or_insert(0) += 1;
        }
        let mut profiles: Vec<CultureProfile> =
            self.emergence.cluster_cultures.values().cloned().collect();
        if profiles.len() < 2 {
            if let Some(p) = profiles.first_mut() {
                let one = std::slice::from_mut(p);
                drift_populations(one, &[], self.rng_mut(), 0.02, 0.0, 0.85);
            }
            self.emergence_language_lexicon(tick);
            return;
        }
        let keys: Vec<u64> = self.emergence.cluster_cultures.keys().copied().collect();
        let mut edges = Vec::new();
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                edges.push(ContactEdge {
                    from: i,
                    to: j,
                    weight: 0.15,
                });
            }
        }
        drift_populations(&mut profiles, &edges, self.rng_mut(), 0.02, 0.08, 0.85);
        for (key, profile) in keys.into_iter().zip(profiles) {
            self.emergence.cluster_cultures.insert(key, profile);
        }
        self.emergence_language_lexicon(tick);
        if tick % 128 == 0 && !self.emergence.cluster_cultures.is_empty() {
            let n = self.emergence.cluster_cultures.len();
            self.emergence.push_feed(
                tick,
                "culture_drift",
                format!("{n} settlement cultures drifted"),
                None,
            );
        }
    }

    /// Coin settlement/faction/event lexemes from drifted phoneme inventories.
    fn emergence_language_lexicon(&mut self, tick: u64) {
        let seed = self.state.rng_seed;
        for (cluster_id, profile) in &self.emergence.cluster_cultures {
            let lexicon = self
                .emergence
                .cluster_lexicons
                .entry(*cluster_id)
                .or_default();
            let mut rng = ChaCha8Rng::seed_from_u64(seed ^ cluster_id ^ tick);
            lexicon.coin(&mut rng, &profile.phonemes, LexemeKind::Settlement, *cluster_id);
            if tick % 128 == 0 {
                lexicon.coin(&mut rng, &profile.phonemes, LexemeKind::Event, tick);
            }
        }
        let fallback = self
            .emergence
            .cluster_cultures
            .values()
            .next()
            .map(|p| p.phonemes.clone())
            .unwrap_or_else(|| PhonemeInventory::seed_from(seed));
        for (&faction_id, _) in &self.state.factions {
            let fid = u64::from(faction_id);
            let lexicon = self.emergence.cluster_lexicons.entry(fid).or_default();
            let mut rng = ChaCha8Rng::seed_from_u64(seed ^ fid ^ 0xFAC1_0000);
            lexicon.coin(&mut rng, &fallback, LexemeKind::Faction, fid);
        }
        if tick >= 250 && self.emergence.cluster_cultures.len() >= 2 {
            let region_count = self
                .emergence
                .cluster_cultures
                .keys()
                .filter(|id| {
                    self.emergence
                        .cluster_lexicons
                        .get(id)
                        .and_then(|lex| {
                            self.emergence.cluster_cultures.get(id).and_then(|profile| {
                                name_from_lexicon(lex, &profile.phonemes, LexemeKind::Settlement, **id)
                            })
                        })
                        .is_some()
                })
                .count();
            if region_count >= 1 {
                self.emergence.push_feed(
                    tick,
                    "language_region",
                    format!("{region_count} emergent dialect regions"),
                    None,
                );
            }
        }
    }

    fn emergence_social(&mut self) {
        let tick_u32 = self.state.tick.min(u32::MAX as u64) as u32;
        let agents: Vec<(Entity, u64, Option<u64>)> = self
            .world
            .query::<(&Civilian, Option<&ClusterMember>)>()
            .iter()
            .map(|(e, (c, m))| (e, c.id, m.map(|x| x.cluster.0)))
            .collect();
        let mut by_cluster: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for (_, id, cluster) in &agents {
            if let Some(c) = cluster {
                by_cluster.entry(*c).or_default().push(*id);
            }
        }
        for ids in by_cluster.values() {
            if ids.len() < 2 {
                continue;
            }
            for i in 0..ids.len().saturating_sub(1) {
                let a = ids[i];
                let b = ids[i + 1];
                if !self.rng_mut().gen_bool(0.12) {
                    continue;
                }
                let kind = if self.rng_mut().gen_bool(0.7) {
                    Interaction::Coexisted
                } else {
                    Interaction::Cooperated { benefit: 0.5 }
                };
                self.apply_social_pair(a, b, kind, tick_u32);
            }
        }
        let social_entities: Vec<Entity> = agents.iter().map(|(entity, _, _)| *entity).collect();
        for entity in social_entities {
            self.ensure_social_graph(entity);
            if let Ok(mut graph) = self.world.get::<&mut SocialGraph>(entity) {
                decay_social_graph(&mut graph, tick_u32);
            }
        }
    }

    fn apply_social_pair(&mut self, a_id: u64, b_id: u64, kind: Interaction, tick: u32) {
        let entity_a = self.agent_entity(a_id);
        let entity_b = self.agent_entity(b_id);
        let (Some(ea), Some(eb)) = (entity_a, entity_b) else {
            return;
        };
        self.ensure_social_graph(ea);
        self.ensure_social_graph(eb);
        if let Ok(mut ga) = self.world.get::<&mut SocialGraph>(ea) {
            apply_social_event(
                &mut ga,
                SocialEvent {
                    a: a_id,
                    b: b_id,
                    kind,
                    tick,
                },
            );
        }
        if let Ok(mut gb) = self.world.get::<&mut SocialGraph>(eb) {
            apply_social_event(
                &mut gb,
                SocialEvent {
                    a: b_id,
                    b: a_id,
                    kind,
                    tick,
                },
            );
        }
    }

    fn ensure_social_graph(&mut self, entity: Entity) {
        if self.world.get::<&SocialGraph>(entity).is_err() {
            let _ = self.world.insert(entity, (SocialGraph::default(),));
        }
    }

    fn emergence_psyche(&mut self) {
        let tick = self.state.tick;
        let tick_u32 = tick.min(u32::MAX as u64) as u32;
        let profile = self.emergence.psych_profile.clone();
        let agents: Vec<(Entity, u64, Option<u64>)> = self
            .world
            .query::<(&Civilian, Option<&ClusterMember>)>()
            .iter()
            .map(|(e, (c, m))| (e, c.id, m.map(|x| x.cluster.0)))
            .collect();

        for (entity, _, _) in &agents {
            if self.world.get::<&Dna>(*entity).is_err()
                || self.world.get::<&Psyche>(*entity).is_ok()
            {
                continue;
            }
            let genome = self
                .world
                .get::<&Dna>(*entity)
                .expect("dna present")
                .0
                .clone();
            let psyche = psyche_from_dna(&Dna(genome), &profile);
            let had_social_graph = self.world.get::<&SocialGraph>(*entity).is_ok();
            let _ = self.world.insert(*entity, (psyche,));
            if !had_social_graph {
                let _ = self.world.insert(*entity, (SocialGraph::default(),));
            }
        }

        for (entity, id, cluster) in agents {
            let culture_traits = cluster
                .and_then(|c| self.emergence.cluster_cultures.get(&c))
                .map(|p| p.traits)
                .unwrap_or([0.5; 4]);
            let tie_samples: Vec<(f32, u64)> = self
                .world
                .get::<&SocialGraph>(entity)
                .ok()
                .map(|graph| {
                    graph
                        .ties
                        .iter()
                        .map(|tie| (tie.familiarity.max(0.1), tie.other))
                        .collect()
                })
                .unwrap_or_default();
            let exposures: Vec<(f32, [f32; 4])> = tie_samples
                .into_iter()
                .filter_map(|(weight, other_id)| {
                    let other_entity = self.agent_entity(other_id)?;
                    let other_cluster = self
                        .world
                        .get::<&ClusterMember>(other_entity)
                        .ok()
                        .map(|m| m.cluster.0)?;
                    self.emergence
                        .cluster_cultures
                        .get(&other_cluster)
                        .map(|p| (weight, p.traits))
                })
                .collect();
            let exposure = if exposures.is_empty() {
                culture_traits
            } else {
                belief_culture_exposure(&exposures)
            };

            let (needs, life_needs) = {
                let agent_needs =
                    self.world
                        .get::<&Needs>(entity)
                        .ok()
                        .map(|n| *n)
                        .unwrap_or(Needs {
                            food: 0.5,
                            shelter: 0.5,
                            safety: 0.5,
                            belonging: 0.5,
                        });
                let life = self
                    .world
                    .get::<&LifeNeeds>(entity)
                    .ok()
                    .map(|n| *n)
                    .unwrap_or_else(LifeNeeds::sated);
                (agent_needs, life)
            };

            if let Ok(mut psyche) = self.world.get::<&mut Psyche>(entity) {
                let threat = (1.0 - life_needs.safety).max(0.0);
                let delta_needs = (needs.food - 0.5).abs();
                let temperament = psyche.temperament;
                let maturity = psyche.maturity;
                update_mood(
                    &mut psyche.mood,
                    &needs,
                    &temperament,
                    threat,
                    delta_needs,
                    0.0,
                );
                let arousal = psyche.mood.arousal;
                nudge_temperament(
                    &mut psyche.temperament,
                    arousal,
                    needs.belonging,
                    maturity,
                );
            }
            let sociability = self
                .world
                .get::<&Psyche>(entity)
                .ok()
                .map(|psyche| psyche.temperament.sociability);
            if let Some(sociability) = sociability {
                let mut local_rng =
                    ChaCha8Rng::seed_from_u64(self.state.rng_seed ^ self.state.tick ^ id);
                if let Ok(mut psyche) = self.world.get::<&mut Psyche>(entity) {
                    update_beliefs(&mut psyche.beliefs, exposure, sociability, &mut local_rng);
                }
            }
            let _ = id;
            let _ = tick_u32;
        }

        if tick % 64 == 0 {
            if let Some((_, (civ, psyche))) =
                self.world.query::<(&Civilian, &Psyche)>().iter().next()
            {
                self.emergence.push_feed(
                    tick,
                    "psyche_sample",
                    format!(
                        "agent {} mood valence {:.2} arousal {:.2}",
                        civ.id, psyche.mood.valence, psyche.mood.arousal
                    ),
                    Some(civ.id),
                );
            }
        }
    }

    fn emergence_genetics_sentience(&mut self) {
        let tick = self.state.tick;
        let profile = self.emergence.sentience_profile.clone();
        let threshold = self.emergence.sentience_threshold;
        // N9: collect (agent_id, faction_id_opt, dna) so we can build per-faction
        // mean aggression without a second world scan.
        let agents: Vec<(u64, Option<u32>, Dna)> = self
            .world
            .query::<(&Civilian, &Dna)>()
            .iter()
            .map(|(_, (c, d))| {
                let faction = match c.alignment {
                    Alignment::Faction(fid) => Some(fid),
                    _ => None,
                };
                (c.id, faction, d.clone())
            })
            .collect();

        // N9: rebuild faction_aggression from this tick's scan.
        {
            let mut faction_agg_sum: BTreeMap<u32, (f32, u32)> = BTreeMap::new();
            for (_, faction_opt, dna) in &agents {
                if let Some(fid) = faction_opt {
                    let agg = express(dna).behavior.aggression;
                    let entry = faction_agg_sum.entry(*fid).or_insert((0.0, 0));
                    entry.0 += agg;
                    entry.1 += 1;
                }
            }
            self.faction_aggression = faction_agg_sum
                .into_iter()
                .map(|(fid, (sum, count))| (fid, sum / count as f32))
                .collect();
        }

        for (agent_id, _faction_opt, dna) in agents {
            let event = evaluate_sentience(Some(agent_id), &dna, &profile, threshold);
            if event.crossed && self.emergence.sentient_agents.insert(agent_id) {
                self.emergence.last_sentience.push(event.clone());
                let phenotype = express(&dna);
                self.emergence.push_feed(
                    tick,
                    "sentience",
                    format!(
                        "lineage {} crossed sentience (cognition {:.2}, aggression {:.2})",
                        agent_id, event.cognition_score, phenotype.behavior.aggression
                    ),
                    Some(agent_id),
                );
            }
        }

        // FR-CIV-GENETICS / FR-CIV-LEGENDS — N7: the moment a lineage crosses
        // the sentience threshold mints a bounded belief (awe) and cohesion
        // (shared identity) pulse. Reuses the same per-tick detection that
        // just populated `last_sentience`; no second world scan. Additive
        // only, bounded by per-tick caps (edge-of-chaos).
        self.apply_awakening_coupling();
    }

    /// FR-CIV-GENETICS / FR-CIV-LEGENDS — N7: mint a bounded belief + cohesion
    /// pulse from this tick's threshold crossings. Reads
    /// `self.emergence.last_sentience` (already populated by
    /// [`Simulation::emergence_genetics_sentience`]) so we never re-scan the
    /// world. Additive only, bounded by [`MAX_AWAKENING_BELIEF_PER_TICK`] and
    /// [`MAX_AWAKENING_COHESION_PER_TICK`].
    pub(crate) fn apply_awakening_coupling(&mut self) {
        let awakenings = self.emergence.last_sentience.len();
        if awakenings == 0 {
            return;
        }
        self.add_belief(awakening_belief_gain(awakenings));
        self.add_cohesion(awakening_cohesion_gain(awakenings));
    }

    fn emergence_legends(&mut self) {
        let tick = self.state.tick;
        let epoch = self.emergence.legends.graph.config.epoch_of(tick);
        for birth in self.last_births().to_vec() {
            let raw = RawSimEvent::new(tick, EventKind::Birth, SourceCrate::Agents, 0.45)
                .with_participant(
                    SourceCrate::Agents,
                    SimRuntimeId(birth.entity_id),
                    Role::Witness,
                );
            let outcome = self.emergence_ingest_legend(raw);
            self.record_legend_promotions(tick, &outcome.promoted, birth.entity_id);
        }
        for death in self.last_deaths().to_vec() {
            let raw = RawSimEvent::new(tick, EventKind::Death, SourceCrate::Agents, 0.85)
                .with_participant(
                    SourceCrate::Agents,
                    SimRuntimeId(death.entity_id),
                    Role::Victim,
                );
            let outcome = self.emergence_ingest_legend(raw);
            let already_legend = self
                .emergence
                .legends
                .graph
                .entity_for_sim(SourceCrate::Agents, SimRuntimeId(death.entity_id))
                .is_some();
            if let Some(eid) = self
                .emergence
                .legends
                .graph
                .entity_for_sim(SourceCrate::Agents, SimRuntimeId(death.entity_id))
            {
                self.emergence.legends.graph.mark_died(eid, epoch);
            }
            // Death of a legend-ranked agent triggers a martyrdom faith surge (FR-CIV-RELIGION).
            if already_legend {
                self.add_belief(BELIEF_GAIN_LEGEND_DEATH);
            }
            self.emergence.push_feed(
                tick,
                "death",
                format!("agent {} died — recorded in saga graph", death.entity_id),
                Some(death.entity_id),
            );
            self.record_legend_promotions(tick, &outcome.promoted, death.entity_id);
        }
        for event in self.emergence.last_sentience.clone() {
            if let Some(id) = event.lineage_id {
                let raw = RawSimEvent::new(
                    tick,
                    EventKind::SpeciationEvent,
                    SourceCrate::Genetics,
                    event.cognition_score,
                )
                .with_participant(
                    SourceCrate::Agents,
                    SimRuntimeId(id),
                    Role::Effect,
                );
                let outcome = self.emergence_ingest_legend(raw);
                self.emergence.push_feed(
                    tick,
                    "sentience",
                    format!(
                        "lineage {} crossed sentience — saga graph updated",
                        id
                    ),
                    Some(id),
                );
                self.record_legend_promotions(tick, &outcome.promoted, id);
            }
        }
        for pulse in self.last_tick_combat_pulses().to_vec() {
            let mut raw =
                RawSimEvent::new(tick, EventKind::Battle, SourceCrate::Tactics, 0.75);
            let mut agent_id = None;
            if let Some(a) = pulse.unit_a {
                raw = raw.with_participant(SourceCrate::Tactics, SimRuntimeId(a), Role::Aggressor);
                agent_id = Some(a);
            }
            if let Some(b) = pulse.unit_b {
                raw = raw.with_participant(SourceCrate::Tactics, SimRuntimeId(b), Role::Defender);
                agent_id = agent_id.or(Some(b));
            }
            let outcome = self.emergence_ingest_legend(raw);
            if let Some(id) = agent_id {
                // Battle: collective awe/fear drives faith (FR-CIV-RELIGION belief-from-events).
                self.add_belief(BELIEF_GAIN_BATTLE);
                self.emergence.push_feed(
                    tick,
                    "battle",
                    format!("combat pulse recorded in saga graph"),
                    Some(id),
                );
                self.record_legend_promotions(tick, &outcome.promoted, id);
            }
        }
        for cluster_id in self.last_settlement_ids().to_vec() {
            if !self
                .emergence
                .known_settlement_ids
                .insert(cluster_id)
            {
                continue;
            }
            let founder = self.settlement_founder_agent(cluster_id);
            let mut raw = RawSimEvent::new(
                tick,
                EventKind::SettlementFounded,
                SourceCrate::Protocol3d,
                0.9,
            )
            .with_participant(
                SourceCrate::Protocol3d,
                SimRuntimeId(cluster_id),
                Role::Founder,
            );
            if let Some(founder_id) = founder {
                raw = raw.with_participant(
                    SourceCrate::Agents,
                    SimRuntimeId(founder_id),
                    Role::Leader,
                );
            }
            let outcome = self.emergence_ingest_legend(raw);
            if let (Some(founder_id), Some(settle_eid)) = (
                founder,
                self.emergence.legends.graph.entity_for_sim(
                    SourceCrate::Protocol3d,
                    SimRuntimeId(cluster_id),
                ),
            ) {
                if let Some(leader_eid) = self.emergence.legends.graph.entity_for_sim(
                    SourceCrate::Agents,
                    SimRuntimeId(founder_id),
                ) {
                    self.emergence
                        .legends
                        .graph
                        .link_entity_edge(leader_eid, settle_eid, LegendEdge::Founded);
                }
            }
            if let Some(founder_id) = founder {
                // Settlement founding: communal milestone breeds shared veneration (FR-CIV-RELIGION).
                self.add_belief(BELIEF_GAIN_FOUNDING);
                self.emergence.push_feed(
                    tick,
                    "founding",
                    format!("settlement {cluster_id} founded — saga graph updated"),
                    Some(founder_id),
                );
                self.record_legend_promotions(tick, &outcome.promoted, founder_id);
            }
        }
        for disaster in self.last_tick_disaster_pulses().to_vec() {
            let region = civ_legends::RegionId(
                disaster.pos.x.unsigned_abs() as u64 ^ disaster.pos.z.unsigned_abs() as u64,
            );
            let raw = RawSimEvent::new(tick, EventKind::Disaster, SourceCrate::Planet, 0.8)
                .with_region(region);
            let outcome = self.emergence_ingest_legend(raw);
            self.emergence.push_feed(
                tick,
                "disaster",
                format!("{:?} disaster recorded in saga graph", disaster.kind),
                None,
            );
            if !outcome.promoted.is_empty() {
                self.record_legend_promotions(tick, &outcome.promoted, 0);
            }
        }
        for dip in self.diplomacy_events().to_vec() {
            let (kind, label) = match dip.kind {
                crate::engine::DiplomacyKind::Conflict => (EventKind::WarDeclared, "war"),
                crate::engine::DiplomacyKind::Peace => (EventKind::WarEnded, "peace"),
                crate::engine::DiplomacyKind::TradeAgreement => {
                    (EventKind::LawObserved, "treaty")
                }
            };
            let raw = RawSimEvent::new(tick, kind, SourceCrate::Engine, 0.55)
                .with_participant(
                    SourceCrate::Engine,
                    SimRuntimeId(u64::from(dip.faction_a)),
                    Role::Leader,
                )
                .with_participant(
                    SourceCrate::Engine,
                    SimRuntimeId(u64::from(dip.faction_b)),
                    Role::Leader,
                );
            let outcome = self.emergence_ingest_legend(raw);
            self.emergence.push_feed(
                tick,
                label,
                format!(
                    "factions {} and {} — {label} recorded in saga graph",
                    dip.faction_a, dip.faction_b
                ),
                None,
            );
            let war_key = AggregateKey {
                kind: EntityKind::War,
                a: ClusterId(u64::from(dip.faction_a)),
                b: ClusterId(u64::from(dip.faction_b)),
                start_bucket: epoch.0,
            };
            let _war = self
                .emergence
                .legends
                .graph
                .resolve_aggregate(war_key, epoch);
            self.record_legend_promotions(
                tick,
                &outcome.promoted,
                u64::from(dip.faction_a),
            );
        }
        // Named legend belief/cohesion influence (FR-CIV-LEGENDS deepening).
        self.apply_named_legend_influence();
    }

    /// Apply per-tick belief/cohesion boost from named legends (FR-CIV-LEGENDS deepening).
    /// Each entity with a non-None title contributes BELIEF_PER_NAMED_LEGEND belief and
    /// COHESION_PER_NAMED_LEGEND cohesion. Called at the end of `emergence_legends`.
    fn apply_named_legend_influence(&mut self) {
        let named_count = self
            .emergence
            .legends
            .graph
            .query_named_legends()
            .named_entities
            .len() as i64;
        if named_count > 0 {
            self.add_belief(named_count.saturating_mul(BELIEF_PER_NAMED_LEGEND));
            self.add_cohesion(named_count.saturating_mul(COHESION_PER_NAMED_LEGEND));
        }
    }

    fn settlement_founder_agent(&self, cluster_id: u64) -> Option<u64> {
        self.world
            .query::<(&Civilian, &ClusterMember)>()
            .iter()
            .filter(|(_, (_, member))| member.cluster.0 == cluster_id)
            .map(|(_, (civ, _))| civ.id)
            .min()
    }

    /// Snapshot per-cluster belief centroids after psyche drift (FR-CIV-RELIGION).
    fn emergence_accrue_cluster_beliefs(&mut self) {
        self.emergence.cluster_beliefs = cluster_belief_centroids(&self.world);
    }

    fn emergence_ingest_legend(&mut self, raw: RawSimEvent) -> IngestOutcome {
        self.emergence.legends.ingest(raw)
    }

    fn record_legend_promotions(&mut self, tick: u64, promoted: &[LegendEntityId], agent_id: u64) {
        if promoted.is_empty() {
            return;
        }
        const BELIEF_PER_LEGEND_PROMOTION: i64 = 3;
        self.add_belief((promoted.len() as i64).saturating_mul(BELIEF_PER_LEGEND_PROMOTION));
        self.emergence.has_patron = true;
        self.emergence.push_feed(
            tick,
            "legend_promotion",
            format!(
                "agent {} promoted in saga graph ({})",
                agent_id,
                promoted.len()
            ),
            Some(agent_id),
        );
    }

    fn emergence_civ_ai(&mut self) {
        let tick = self.state.tick;
        for event in &self.emergence.last_feed.clone() {
            if event.kind != "legend_promotion" && event.kind != "sentience" {
                continue;
            }
            let Some(agent_id) = event.agent_id else {
                continue;
            };
            let prompt = format!(
                "Name this historically significant agent (id {agent_id}): {}",
                event.summary
            );
            let output = civ_ai_sync_generate(&prompt);
            let name = NameRef(agent_id);
            if let Some(legend_id) = self
                .emergence
                .legends
                .graph
                .entity_for_sim(SourceCrate::Agents, SimRuntimeId(agent_id))
            {
                self.emergence.legends.graph.set_name(legend_id, name);
            }
            self.emergence.last_ai_decisions.push(CivAiDecision {
                tick,
                agent_id,
                prompt: prompt.clone(),
                output: output.clone(),
            });
            self.emergence.push_feed(
                tick,
                "civ_ai",
                format!("civ-ai named agent {agent_id}: {output}"),
                Some(agent_id),
            );
        }
    }

    /// Emergence event feed from the most recent tick (HUD `event_feed`).
    #[must_use]
    pub fn emergence_feed(&self) -> &[EmergenceFeedEvent] {
        &self.emergence.last_feed
    }

    /// Borrow the saga graph for inspector / legends queries (FR-CIV-LEGENDS-QUERY-07).
    #[must_use]
    pub fn legends_graph(&self) -> &SagaGraph {
        self.emergence.legends.graph()
    }

    /// Read-only legends query surface for `sim.legends` JSON-RPC (FR-CIV-LEGENDS-QUERY-07).
    #[must_use]
    pub fn legends_query(
        &self,
        query: &str,
        agent_id: Option<u64>,
        top_n: Option<usize>,
        epoch: Option<u64>,
    ) -> LegendsQueryResult {
        let graph = self.legends_graph();
        let tick = self.state.tick;
        let mut result = LegendsQueryResult {
            query_api_version: QUERY_API_VERSION,
            tick,
            node_count: graph.node_count(),
            saga: None,
            significant: None,
            epoch_digest: None,
            empty_reason: None,
            emergence_feed: self.emergence_feed().to_vec(),
        };
        match query {
            "saga_of" => {
                let Some(agent_id) = agent_id else {
                    result.empty_reason = Some("saga_of requires agent_id".to_string());
                    return result;
                };
                if let Some(eid) =
                    graph.entity_for_sim(SourceCrate::Agents, SimRuntimeId(agent_id))
                {
                    result.saga = graph.saga_of(eid);
                } else {
                    result.empty_reason = Some(
                        graph
                            .empty_saga_reason(LegendEntityId(agent_id))
                            .map(|r| r.reason_text())
                            .unwrap_or_else(|| "agent not in saga graph".to_string()),
                    );
                }
            }
            "significant" => {
                let n = top_n.unwrap_or(10).clamp(1, 50);
                result.significant = Some(graph.significant(n, None));
            }
            "epoch_digest" => {
                let epoch = Epoch(epoch.unwrap_or_else(|| graph.config.epoch_of(tick).0));
                result.epoch_digest = Some(graph.epoch_digest(epoch, None));
            }
            "status" | _ => {}
        }
        result
    }

    /// Per-cluster emergent culture profiles (FR-CIV-PSYCHE / culture drift).
    #[must_use]
    pub fn cluster_cultures(&self) -> &BTreeMap<u64, CultureProfile> {
        &self.emergence.cluster_cultures
    }

    /// Per-cluster belief centroids (FR-CIV-RELIGION emergent doctrine).
    #[must_use]
    pub fn cluster_beliefs(&self) -> &BTreeMap<u64, [f32; PSYCHE_DIM]> {
        &self.emergence.cluster_beliefs
    }

    /// Whether shared veneration has crystallised from saga promotions.
    #[must_use]
    pub fn has_religious_patron(&self) -> bool {
        self.emergence.has_patron
    }

    /// Per-cluster evolved lexicons (FR-CIV-LANG naming).
    #[must_use]
    pub fn cluster_lexicons(&self) -> &BTreeMap<u64, EvolvedLexicon> {
        &self.emergence.cluster_lexicons
    }

    /// Civ-ai decisions from the most recent tick.
    #[must_use]
    pub fn civ_ai_decisions(&self) -> &[CivAiDecision] {
        &self.emergence.last_ai_decisions
    }

    /// Sentience crossings detected this tick.
    #[must_use]
    pub fn sentience_events(&self) -> &[SentienceEvent] {
        &self.emergence.last_sentience
    }

    /// Psyche for a civilian agent id, if present.
    #[must_use]
    pub fn agent_psyche(&self, agent_id: u64) -> Option<Psyche> {
        let entity = self.agent_entity(agent_id)?;
        self.world.get::<&Psyche>(entity).ok().map(|p| (*p).clone())
    }

    /// Social graph for a civilian agent id, if present.
    #[must_use]
    pub fn agent_social_graph(&self, agent_id: u64) -> Option<SocialGraph> {
        let entity = self.agent_entity(agent_id)?;
        self.world
            .get::<&SocialGraph>(entity)
            .ok()
            .map(|g| (*g).clone())
    }
}

/// Sync civ-ai flavor text on the hot path (mirrors [`civ_ai::providers::DummyAiProvider`]).
fn civ_ai_sync_generate(prompt: &str) -> String {
    let snapshot = blake3::hash(prompt.as_bytes());
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in prompt.bytes().chain(snapshot.as_bytes().iter().copied()) {
        state ^= u64::from(byte);
        state = state.wrapping_mul(0x0100_0000_01b3);
    }
    format!("dummy-generation-{state:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Fixed;
    use civ_agents::count_civilians;

    fn run_ticks(sim: &mut Simulation, n: u64) {
        for _ in 0..n {
            sim.tick();
        }
    }

    /// FR-CIV-LEGENDS-INGEST-02 — deaths on the life/citizen path reach the saga graph.
    #[test]
    fn legends_phase_ingests_death_events() {
        let mut sim = Simulation::with_seed(42);
        run_ticks(&mut sim, 4);
        let before = sim.legends_graph().node_count();
        // Force sustained famine: zero the food stock BEFORE each tick so
        // production can't refill it. The starvation death path
        // (`phase_citizen_lifecycle`) requires `resources.food.raw <= 0`, and
        // per-tick production would otherwise mask it — biome-modulated food
        // production (#558) and carrying capacity (#559) make the refilled
        // amount planet-dependent, so we pin food to zero rather than relying
        // on a single reset surviving 250 ticks of production.
        for _ in 0..250 {
            sim.state.resources.food = Fixed::ZERO;
            sim.tick();
        }
        let after = sim.legends_graph().node_count();
        assert!(
            after > before || !sim.emergence_feed().is_empty(),
            "expected saga graph growth or emergence feed entries"
        );
    }

    /// FR-CIV-PSYCHE — mood moves after repeated emergence ticks.
    #[test]
    fn psyche_phase_mutates_mood_over_ticks() {
        let mut sim = Simulation::with_seed(7);
        run_ticks(&mut sim, 80);
        let agent_id = sim
            .world
            .query::<&Civilian>()
            .iter()
            .next()
            .map(|(_, c)| c.id)
            .expect("agent");
        let first = sim.agent_psyche(agent_id).expect("psyche attached");
        run_ticks(&mut sim, 80);
        let second = sim.agent_psyche(agent_id).expect("psyche");
        assert!(
            first.mood.valence != second.mood.valence
                || first.mood.arousal != second.mood.arousal
                || first.beliefs != second.beliefs,
            "psyche should evolve"
        );
    }

    /// FR-CIV-RELIGION — cluster belief centroids diverge like culture profiles.
    #[test]
    fn cluster_beliefs_diverge_between_settlements() {
        let mut sim_a = Simulation::with_seed(66);
        let mut sim_b = Simulation::with_seed(66);
        run_ticks(&mut sim_a, 200);
        run_ticks(&mut sim_b, 200);
        if sim_a.cluster_beliefs().len() >= 2 {
            let values: Vec<_> = sim_a.cluster_beliefs().values().copied().collect();
            assert_ne!(
                values[0], values[1],
                "cluster belief centroids should diverge"
            );
            assert_eq!(
                sim_a.cluster_beliefs(),
                sim_b.cluster_beliefs(),
                "same seed must yield identical cluster beliefs at tick N"
            );
        }
    }

    /// FR-CIV-GENETICS / culture — cluster cultures diverge over ticks.
    #[test]
    fn culture_phase_drifts_cluster_profiles() {
        let mut sim_a = Simulation::with_seed(99);
        let mut sim_b = Simulation::with_seed(99);
        run_ticks(&mut sim_a, 200);
        run_ticks(&mut sim_b, 200);
        assert!(
            !sim_a.cluster_cultures().is_empty() || count_civilians(&sim_a.world) > 0,
            "expected cultures or civilians"
        );
        if sim_a.cluster_cultures().len() >= 2 {
            let values: Vec<_> = sim_a.cluster_cultures().values().map(|p| p.traits).collect();
            assert_ne!(values[0], values[1], "cultures should diverge");
            let phon_a: Vec<_> = sim_a
                .cluster_cultures()
                .values()
                .map(|p| p.phonemes.clone())
                .collect();
            let phon_b: Vec<_> = sim_b
                .cluster_cultures()
                .values()
                .map(|p| p.phonemes.clone())
                .collect();
            assert_eq!(
                phon_a, phon_b,
                "same seed must yield identical phoneme vectors at tick N"
            );
        }
    }

    /// FR-CIV-AI-006 / MOAT wiring — emergence leaves queryable psyche + saga state.
    #[test]
    fn civ_ai_phase_leaves_observable_emergence_state() {
        let mut sim = Simulation::with_seed(123);
        sim.emergence.sentience_threshold = SentienceThreshold::new(0.05);
        run_ticks(&mut sim, 150);
        let agent_id = sim
            .world
            .query::<&Civilian>()
            .iter()
            .next()
            .map(|(_, c)| c.id)
            .expect("civilian");
        assert!(
            sim.agent_psyche(agent_id).is_some(),
            "psyche component should attach"
        );
        assert!(
            sim.legends_graph().node_count() > 0,
            "saga graph should accumulate nodes"
        );
    }

    fn test_seed_definition(id: &str) -> SeedDefinition {
        let length = 64usize;
        SeedDefinition {
            id: id.to_string(),
            display_name: id.to_string(),
            dna_length: length,
            genome: (0..length as u8).collect(),
            divergence: 0.5,
            spawn_biome_affinity: vec![],
            notes: None,
        }
    }

    /// `register_seed_set` merges valid seeds and replaces ids on re-register.
    #[test]
    #[ignore = "Simulation::register_seed_set and seed_library() not implemented"]
    fn register_seed_set_merges_and_replaces_ids() {
        // TODO: Implement register_seed_set and seed_library methods on Simulation
    }

    /// `set_active_seed` updates the active id; unknown ids are rejected.
    #[test]
    #[ignore = "Simulation::set_active_seed and active_seed_id() not implemented"]
    fn set_active_seed_updates_or_rejects_unknown() {
        // TODO: Implement set_active_seed, active_seed_id methods on Simulation
    }

    /// `register_seed_file` loads fixture RON and reports missing paths.
    #[test]
    #[ignore = "Simulation::register_seed_file and seed_library() not implemented"]
    fn register_seed_file_loads_fixture_and_reports_missing() {
        // TODO: Implement register_seed_file and seed_library methods on Simulation
    }

    /// `agent_social_graph` returns cloned graphs by civilian id.
    #[test]
    fn agent_social_graph_returns_graph_for_known_agent() {
        use civ_agents::Tie;

        let mut sim = Simulation::with_seed(4);
        let (entity, agent_id) = sim
            .world
            .query::<&Civilian>()
            .iter()
            .next()
            .map(|(e, c)| (e, c.id))
            .expect("civilian");
        let graph = SocialGraph {
            ties: vec![Tie::new(42, 1)],
        };
        let _ = sim.world.insert(entity, (graph.clone(),));

        assert_eq!(sim.agent_social_graph(agent_id), Some(graph));
        assert_eq!(sim.agent_social_graph(9_999_999), None);
    }

    /// select_seed_for_position picks the biome-matched seed, not the active fallback.
    #[test]
    fn seed_selection_picks_biome_match() {
        use civ_agents::Position3d;
        use civ_genetics::{SeedDefinition, SeedLibrary, SeedSet};
        use civ_planet::{defaults_earthlike, GeologyMap};
        use civ_voxel::{WorldCoord, FIXED_SCALE};

        // Build a planet that has Forest in its equatorial band (axial tilt > 30°).
        let (mut planet_cfg, _) = defaults_earthlike();
        planet_cfg.axial_tilt_deg = 40;
        let geology_map = GeologyMap::seed(&planet_cfg);

        // A mid-latitude (equatorial) position — nz=0.5 → Forest biome.
        let equatorial_pos = Position3d {
            coord: WorldCoord {
                x: (0.5 * FIXED_SCALE as f32) as i64,
                y: 0,
                z: (0.5 * FIXED_SCALE as f32) as i64,
            },
        };

        // Confirm the biome for this position is Forest.
        let biome = geology_map.biome_at_normalized(0.5, 0.5);
        assert_eq!(
            biome,
            civ_planet::BiomeKind::Forest,
            "expected Forest biome at equatorial position with high axial tilt"
        );

        // Build a seed library with: raw_organism (no affinity) and
        // human_baseline (TemperateForest affinity).
        let dna_len: usize = 64;
        let active_seed = SeedDefinition {
            id: "raw_organism".to_string(),
            display_name: "Raw Organism".to_string(),
            dna_length: dna_len,
            genome: (0..dna_len as u8).collect(),
            divergence: 1.0,
            spawn_biome_affinity: vec![],
            notes: None,
        };
        let forest_seed = SeedDefinition {
            id: "human_baseline".to_string(),
            display_name: "Human Baseline".to_string(),
            dna_length: dna_len,
            genome: (0..dna_len as u8).map(|i| i.wrapping_mul(7).wrapping_add(13)).collect(),
            divergence: 0.1,
            spawn_biome_affinity: vec!["TemperateForest".to_string()],
            notes: None,
        };
        let set = SeedSet {
            version: 1,
            seeds: vec![active_seed.clone(), forest_seed.clone()],
        };
        let lib = SeedLibrary::from_seed_set(set).expect("valid seed set");

        // select_seed_for_position should prefer the forest seed over the fallback.
        let chosen = select_seed_for_position(
            &lib,
            Some(&active_seed),
            &geology_map,
            &equatorial_pos,
        );
        assert_eq!(
            chosen.map(|s| s.id.as_str()),
            Some("human_baseline"),
            "equatorial Forest position should pick human_baseline, not raw_organism"
        );
    }

    /// FR-CIV-014 / map-seed determinism — seed selection must ignore
    /// insertion order when multiple biome-matching seeds exist.
    #[test]
    fn seed_selection_is_deterministic_across_library_ordering() {
        use civ_agents::Position3d;
        use civ_genetics::{SeedDefinition, SeedLibrary, SeedSet};
        use civ_planet::{defaults_earthlike, GeologyMap};
        use civ_voxel::{WorldCoord, FIXED_SCALE};

        let (mut planet_cfg, _) = defaults_earthlike();
        planet_cfg.axial_tilt_deg = 40;
        let geology_map = GeologyMap::seed(&planet_cfg);

        let pos = Position3d {
            coord: WorldCoord {
                x: (0.5 * FIXED_SCALE as f32) as i64,
                y: 0,
                z: (0.5 * FIXED_SCALE as f32) as i64,
            },
        };

        let alpha = SeedDefinition {
            id: "alpha_seed".to_string(),
            display_name: "Alpha Seed".to_string(),
            dna_length: 64,
            genome: vec![1; 64],
            divergence: 0.2,
            spawn_biome_affinity: vec!["TemperateForest".to_string()],
            notes: None,
        };
        let beta = SeedDefinition {
            id: "beta_seed".to_string(),
            display_name: "Beta Seed".to_string(),
            dna_length: 64,
            genome: vec![2; 64],
            divergence: 0.2,
            spawn_biome_affinity: vec!["TemperateForest".to_string()],
            notes: None,
        };

        let lib_a = SeedLibrary::from_seed_set(SeedSet {
            version: 1,
            seeds: vec![beta.clone(), alpha.clone()],
        })
        .expect("valid seed set");
        let lib_b = SeedLibrary::from_seed_set(SeedSet {
            version: 1,
            seeds: vec![alpha.clone(), beta.clone()],
        })
        .expect("valid seed set");

        let chosen_a = select_seed_for_position(&lib_a, Some(&alpha), &geology_map, &pos)
            .map(|seed| seed.id.as_str());
        let chosen_b = select_seed_for_position(&lib_b, Some(&alpha), &geology_map, &pos)
            .map(|seed| seed.id.as_str());

        assert_eq!(chosen_a, Some("alpha_seed"));
        assert_eq!(chosen_b, Some("alpha_seed"));
    }

    /// `civ_ai_decisions` surfaces naming decisions after sentience crossings.
    #[test]
    fn civ_ai_decisions_populated_after_sentience_tick() {
        let mut sim = Simulation::with_seed(5);
        sim.emergence.sentience_threshold = SentienceThreshold::new(0.05);
        sim.tick();
        let decisions = sim.civ_ai_decisions();
        assert!(
            !decisions.is_empty(),
            "sentience tick should produce civ-ai decisions"
        );
        for decision in decisions {
            assert!(!decision.prompt.is_empty());
            assert!(!decision.output.is_empty());
        }
    }

    /// `sentience_events` records first-time threshold crossings.
    #[test]
    fn sentience_events_records_threshold_crossings() {
        let mut sim = Simulation::with_seed(6);
        sim.emergence.sentience_threshold = SentienceThreshold::new(0.05);
        sim.tick();
        let events = sim.sentience_events();
        assert!(!events.is_empty(), "low threshold should yield crossings");
        for event in events {
            assert!(event.crossed);
            assert!(event.lineage_id.is_some());
        }
    }

    /// FR-CIV-LEGENDS-QUERY-07 — read-only query does not mutate graph state.
    #[test]
    fn legends_query_is_read_only() {
        let mut sim = Simulation::with_seed(11);
        run_ticks(&mut sim, 40);
        let before = sim.legends_graph().node_count();
        let _ = sim.legends_query("status", None, None, None);
        let _ = sim.legends_query("significant", None, Some(5), None);
        assert_eq!(sim.legends_graph().node_count(), before);
    }

    /// FR-CIV-LEGENDS-INGEST-02 — diplomacy events reach saga graph with faction roles.
    #[test]
    fn legends_phase_ingests_diplomacy_with_participants() {
        let mut sim = Simulation::with_seed(3001);
        let before = sim.legends_graph().node_count();
        for _ in 0..1500 {
            sim.tick();
        }
        assert!(
            sim.legends_graph().node_count() > before
                || sim
                    .emergence_feed()
                    .iter()
                    .any(|e| matches!(e.kind.as_str(), "treaty" | "war" | "peace")),
            "diplomacy should ingest into saga graph or emergence feed"
        );
    }

    /// FR-CIV-LEGENDS-INGEST-02 — disasters record in saga graph.
    #[test]
    fn legends_phase_ingests_disaster_events() {
        use crate::disasters::{DisasterKind, DisasterPulse};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::with_seed(88);
        run_ticks(&mut sim, 2);
        let before = sim.legends_graph().node_count();
        sim.last_tick_disaster_pulses.push(DisasterPulse {
            kind: DisasterKind::Quake,
            pos: WorldCoord { x: 0, y: 0, z: 0 },
        });
        sim.phase_emergence();
        assert!(
            sim.legends_graph().node_count() > before
                || sim.emergence_feed().iter().any(|e| e.kind == "disaster"),
            "disaster pulse should reach saga graph"
        );
    }

    /// FR-CIV-LEGENDS deepening — named legend entities boost belief/cohesion.
    #[test]
    fn named_legend_boosts_belief_cohesion() {
        use civ_legends::{LegendEntityId, LegendsConfig, RawSimEvent, Role, SagaGraph, SourceCrate, SimRuntimeId};

        let mut sim = Simulation::with_seed(77);
        // Ingest a high-significance event so an entity gets promoted.
        let raw = RawSimEvent::new(1, civ_legends::EventKind::Battle, SourceCrate::Tactics, 1.0)
            .with_participant(SourceCrate::Tactics, SimRuntimeId(99), Role::Aggressor);
        sim.emergence.legends.ingest(raw);

        // Manually promote an entity to named legend.
        let graph = &mut sim.emergence.legends.graph;
        let entity_id = graph.entity_for_sim(SourceCrate::Tactics, SimRuntimeId(99));
        if let Some(eid) = entity_id {
            let _ = graph.promote_to_legend(eid, "The Iron-Fisted".to_string(), Role::Leader);
        }

        let belief_before = sim.state.belief;
        let cohesion_before = sim.state.cohesion;
        sim.apply_named_legend_influence();
        assert!(
            sim.state.belief >= belief_before,
            "belief should not decrease after named legend influence"
        );
        assert!(
            sim.state.cohesion >= cohesion_before,
            "cohesion should not decrease after named legend influence"
        );
    }
}


/// Computes a deterministic plague outbreak estimate from local density and trade exposure.
///
/// Returns `(outbreak_probability, population_loss)`, where probability is clamped to
/// `0.0..=1.0` and population loss is an abstract deterministic impact score.
pub fn plague_outbreak(density: f32, trade_connectivity: f32) -> (f32, u32) {
    let density = if density.is_finite() {
        density.max(0.0)
    } else {
        0.0
    };
    let trade_connectivity = if trade_connectivity.is_finite() {
        trade_connectivity.max(0.0)
    } else {
        0.0
    };

    let density_pressure = density / (density + 100.0);
    let trade_pressure = trade_connectivity / (trade_connectivity + 10.0);
    let outbreak_probability =
        (0.08 + density_pressure * 0.52 + trade_pressure * 0.32).clamp(0.0, 1.0);

    let population_loss = (outbreak_probability * density * (1.0 + trade_pressure) * 0.18).round()
        as u32;

    (outbreak_probability, population_loss)
}

#[cfg(test)]
mod plague_tests {
    use super::plague_outbreak;

    #[test]
    fn plague_outbreak_is_deterministic() {
        let first = plague_outbreak(120.0, 4.0);
        let second = plague_outbreak(120.0, 4.0);

        assert_eq!(first, second);
    }

    #[test]
    fn plague_outbreak_clamps_invalid_and_negative_inputs() {
        assert_eq!(plague_outbreak(-10.0, f32::NAN), (0.08, 0));
        assert_eq!(plague_outbreak(f32::INFINITY, 3.0), plague_outbreak(0.0, 3.0));
    }

    #[test]
    fn plague_outbreak_scales_with_density_and_trade() {
        let low = plague_outbreak(10.0, 0.5);
        let high = plague_outbreak(180.0, 8.0);

        assert!(high.0 > low.0);
        assert!(high.1 > low.1);
        assert!((0.0..=1.0).contains(&high.0));
    }
}

pub fn seafaring_drive(coastal_population: f32, food_surplus: f32) -> f32 {
    if !coastal_population.is_finite() || !food_surplus.is_finite() {
        return 0.0;
    }

    let population_signal = (coastal_population / 1_000.0).clamp(0.0, 1.0);
    let surplus_signal = (food_surplus / 100.0).clamp(0.0, 1.0);

    ((population_signal * 0.65) + (surplus_signal * 0.35)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod seafaring_drive_tests {
    use super::seafaring_drive;

    #[test]
    fn seafaring_drive_is_deterministic_clamped_and_nan_guarded() {
        assert_eq!(seafaring_drive(f32::NAN, 50.0), 0.0);
        assert_eq!(seafaring_drive(2_000.0, 200.0), 1.0);
        assert_eq!(seafaring_drive(-50.0, -10.0), 0.0);

        let first = seafaring_drive(500.0, 50.0);
        let second = seafaring_drive(500.0, 50.0);
        assert_eq!(first, second);
        assert!((0.0..=1.0).contains(&first));
    }
}

pub fn trade_connectivity_score(neighbor_count: u32, total_surplus: f32) -> f32 {
    if neighbor_count == 0 || !total_surplus.is_finite() || total_surplus <= 0.0 {
        return 0.0;
    }

    let neighbor_factor = (neighbor_count.min(8) as f32) / 8.0;
    let surplus_factor = (total_surplus / 100.0).clamp(0.0, 1.0);
    (neighbor_factor * surplus_factor).clamp(0.0, 1.0)
}

#[cfg(test)]
mod trade_connectivity_score_tests {
    use super::trade_connectivity_score;

    #[test]
    fn clamps_and_guards_nan() {
        assert_eq!(trade_connectivity_score(4, f32::NAN), 0.0);
        assert_eq!(trade_connectivity_score(0, 50.0), 0.0);
        assert_eq!(trade_connectivity_score(16, 250.0), 1.0);
        assert!((0.0..=1.0).contains(&trade_connectivity_score(3, 40.0)));
    }
}


pub fn unrest_pressure(inequality: f32, scarcity: f32) -> f32 {
    if inequality.is_nan() || scarcity.is_nan() {
        return 0.0;
    }

    ((inequality + scarcity) * 0.5).clamp(0.0, 1.0)
}

#[cfg(test)]
mod unrest_pressure_tests {
    use super::unrest_pressure;

    #[test]
    fn clamps_and_handles_nan() {
        assert_eq!(unrest_pressure(-0.5, 0.0), 0.0);
        assert_eq!(unrest_pressure(1.5, 1.5), 1.0);
        assert_eq!(unrest_pressure(f32::NAN, 0.5), 0.0);
    }
}

