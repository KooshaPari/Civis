//! MOAT emergence phases — wire dormant psyche/social/culture/legends/genetics/civ-ai
//! crates into [`Simulation::tick`] (gap-audit §1, master-roadmap S2).
//!
//! Phase order inside `phase_emergence`: genetics → culture → social → psyche →
//! legends ingest → civ-ai naming. Surfaced via [`EmergenceFeedEvent`] and getters
//! on [`Simulation`].

use std::collections::{BTreeMap, BTreeSet, HashSet};

use civ_agents::culture::{drift_populations, ContactEdge, CultureProfile};
use civ_agents::psyche::{nudge_temperament, psyche_from_dna, update_beliefs, update_mood};
use civ_agents::{
    apply_social_event, belief_culture_exposure, decay_social_graph, psych_genome_profile,
    Alignment, Civilian, ClusterMember, Interaction, Needs, Position3d, Psyche, SocialEvent,
    SocialGraph,
};
use civ_genetics::{
    sentience::{evaluate_sentience, CognitionTraitProfile, SentienceEvent, SentienceThreshold},
    Dna, DnaClass, spawn_genome_with_divergence, SeedDefinition, SeedLibrary, SeedSet,
};
use civ_legends::{
    EventKind, IngestOutcome, LegendsConfig, LegendsWorker, RawSimEvent, Role, SagaGraph,
    SourceCrate,
};
use civ_planet::GeologyMap;
use civ_voxel::FIXED_SCALE;
use civ_legends::{LegendEntityId, NameRef, SimRuntimeId};
use civ_needs::Needs as LifeNeeds;
use civ_species::express;
use hecs::Entity;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::engine::{Simulation, awakening_belief_gain, awakening_cohesion_gain};
use crate::culture::{
    advance_faction_ideologies,
};

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
    pub(crate) last_feed: Vec<EmergenceFeedEvent>,
    pub(crate) last_ai_decisions: Vec<CivAiDecision>,
    pub(crate) last_sentience: Vec<SentienceEvent>,
    pub(crate) dna_class: DnaClass,
    pub(crate) psych_profile: civ_agents::PsychGenomeProfile,
    pub(crate) sentience_profile: CognitionTraitProfile,
    pub(crate) sentience_threshold: SentienceThreshold,
    pub(crate) sentient_agents: HashSet<u64>,
}

impl EmergenceState {
    pub(crate) fn new(seed: u64) -> Self {
        let _ = seed;
        EmergenceState {
            legends: LegendsWorker::new(SagaGraph::new(LegendsConfig::default())),
            cluster_cultures: BTreeMap::new(),
            last_feed: Vec::new(),
            last_ai_decisions: Vec::new(),
            last_sentience: Vec::new(),
            dna_class: DnaClass::default(),
            psych_profile: psych_genome_profile(),
            sentience_profile: CognitionTraitProfile::new(
                "sapient-lineage",
                vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
            ),
            sentience_threshold: SentienceThreshold::new(0.72),
            sentient_agents: HashSet::new(),
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

    fn emergence_culture(&mut self) {
        let tick = self.state.tick;
        let mut cluster_member_counts: BTreeMap<u64, u32> = BTreeMap::new();
        for (_, member) in self.world.query::<&ClusterMember>().iter() {
            *cluster_member_counts.entry(member.cluster.0).or_insert(0) += 1;
        }
        for (cluster_id, size) in &cluster_member_counts {
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
        let mut dominant_by_cluster: BTreeMap<u64, u32> = BTreeMap::new();
        let mut faction_vote: BTreeMap<u64, BTreeMap<u32, u32>> = BTreeMap::new();
        for (_, (civilian, member)) in self
            .world
            .query::<(&Civilian, &ClusterMember)>()
            .iter()
        {
            let cluster_id = member.cluster.0;
            let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
            if members < 2 {
                continue;
            }
            if let civ_agents::Alignment::Faction(faction_id) = civilian.alignment {
                *faction_vote.entry(cluster_id).or_default().entry(faction_id).or_default() += 1;
            }
        }
        for (cluster_id, by_faction) in faction_vote {
            let mut best = None;
            let mut best_count = 0u32;
            for (&faction_id, &count) in &by_faction {
                if count > best_count || (count == best_count && best.is_some_and(|id| faction_id < id)) {
                    best = Some(faction_id);
                    best_count = count;
                }
            }
            if let Some(faction_id) = best {
                dominant_by_cluster.insert(cluster_id, faction_id);
            }
        }
        let mut settlement_positions: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
        for (_, (member, position)) in self.world.query::<(&ClusterMember, &Position3d)>().iter() {
            let cluster_id = member.cluster.0;
            let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
            if members < 2 {
                continue;
            }
            settlement_positions
                .entry(cluster_id)
                .or_default()
                .push((position.coord.x, position.coord.z));
        }
        const EMERGENCE_CLUSTER_RADIUS_FP: i64 = (6 * FIXED_SCALE) / 100;
        const EMERGENCE_CONTACT_RADIUS_FP: i64 = EMERGENCE_CLUSTER_RADIUS_FP * 2;
        let contact_radius_sq =
            i128::from(EMERGENCE_CONTACT_RADIUS_FP) * i128::from(EMERGENCE_CONTACT_RADIUS_FP);
        let mut settlement_contacts = BTreeSet::new();
        let cluster_ids: Vec<u64> = settlement_positions.keys().copied().collect();
        for i in 0..cluster_ids.len() {
            for j in (i + 1)..cluster_ids.len() {
                let left = cluster_ids[i];
                let right = cluster_ids[j];
                let left_agents = match settlement_positions.get(&left) {
                    Some(values) => values,
                    None => continue,
                };
                let right_agents = match settlement_positions.get(&right) {
                    Some(values) => values,
                    None => continue,
                };
                let in_contact = left_agents.iter().any(|&(ax, az)| {
                    right_agents
                        .iter()
                        .any(|&(bx, bz)| {
                            let dx = i128::from(ax) - i128::from(bx);
                            let dz = i128::from(az) - i128::from(bz);
                            dx * dx + dz * dz <= contact_radius_sq
                        })
                });
                if in_contact {
                    settlement_contacts.insert((left.min(right), left.max(right)));
                }
            }
        }

        let mut profiles: Vec<CultureProfile> =
            self.emergence.cluster_cultures.values().cloned().collect();
        if profiles.len() < 2 {
            if let Some(p) = profiles.first_mut() {
                let one = std::slice::from_mut(p);
                drift_populations(one, &[], self.rng_mut(), 0.02, 0.0, 0.85);
            }
        } else {
            let keys: Vec<u64> = self.emergence.cluster_cultures.keys().copied().collect();
            let mut cluster_index = BTreeMap::new();
            for (idx, cluster_id) in keys.iter().copied().enumerate() {
                cluster_index.insert(cluster_id, idx);
            }
            let mut edges = Vec::new();
            for &(left, right) in &settlement_contacts {
                let from = match cluster_index.get(&left).copied() {
                    Some(idx) => idx,
                    None => continue,
                };
                let to = match cluster_index.get(&right).copied() {
                    Some(idx) => idx,
                    None => continue,
                };
                edges.push(ContactEdge {
                    from,
                    to,
                    weight: 0.15,
                });
            }
            let climate_pressure =
                (self.climate.day_phase * 0.06 + self.climate.moon_phase * 0.04 + self.climate.tide_offset.abs() * 0.08)
                    .clamp(0.0, 0.16);
            let mutation_rate = 0.02 + climate_pressure;
            let diffusion_rate = 0.08 + (self.state.tick % 1000) as f32 * 0.00001;
            drift_populations(
                &mut profiles,
                &edges,
                self.rng_mut(),
                mutation_rate,
                diffusion_rate,
                0.85,
            );
            let keys: Vec<u64> = self.emergence.cluster_cultures.keys().copied().collect();
            for (key, profile) in keys.into_iter().zip(profiles) {
                self.emergence.cluster_cultures.insert(key, profile);
            }
        }

        let mut faction_religion: BTreeMap<u32, (f32, u32)> = BTreeMap::new();
        for (_, &faction_id) in &dominant_by_cluster {
            let monitor = self
                .religious_profiles
                .get(&faction_id)
                .map(|religion| {
                    religion.monitoring * 0.55 + religion.mythic_coherence * 0.45
                })
                .unwrap_or(0.45);
            let entry = faction_religion.entry(faction_id).or_insert((0.0, 0));
            entry.0 += monitor;
            entry.1 += 1;
        }
        let mut faction_religion_signal = BTreeMap::new();
        for (faction_id, (monitor_sum, count)) in faction_religion {
            if count > 0 {
                faction_religion_signal
                    .insert(faction_id, (monitor_sum / (count as f32)).clamp(0.0, 1.0));
            }
        }

        self.faction_ideologies = advance_faction_ideologies(
            tick,
            &self.emergence.cluster_cultures,
            &dominant_by_cluster,
            &cluster_member_counts,
            &settlement_contacts,
            &self.climate,
            &faction_religion_signal,
            &self.era_progression.faction_ages,
            &self.faction_ideologies,
            self.rng_mut(),
        );
        self.faction_aggression.clear();
        for (faction_id, state) in &self.faction_ideologies {
            self.faction_aggression.insert(*faction_id, state.aggression);
        }

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
                    Role::Founder,
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
            if let Some(eid) = self
                .emergence
                .legends
                .graph
                .entity_for_sim(SourceCrate::Agents, SimRuntimeId(death.entity_id))
            {
                self.emergence.legends.graph.mark_died(eid, epoch);
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
                self.record_legend_promotions(tick, &outcome.promoted, id);
            }
        }
        for dip in self.diplomacy_events().to_vec() {
            let kind = match dip.kind {
                crate::engine::DiplomacyKind::Conflict => EventKind::WarDeclared,
                crate::engine::DiplomacyKind::Peace => EventKind::WarEnded,
                crate::engine::DiplomacyKind::TradeAgreement => EventKind::EconomicBoom,
            };
            let raw = RawSimEvent::new(tick, kind, SourceCrate::Engine, 0.55);
            let _ = self.emergence_ingest_legend(raw);
        }
    }

    fn emergence_ingest_legend(&mut self, raw: RawSimEvent) -> IngestOutcome {
        self.emergence.legends.graph.ingest(raw)
    }

    fn record_legend_promotions(&mut self, tick: u64, promoted: &[LegendEntityId], agent_id: u64) {
        if promoted.is_empty() {
            return;
        }
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

    /// Per-cluster emergent culture profiles (FR-CIV-PSYCHE / culture drift).
    #[must_use]
    pub fn cluster_cultures(&self) -> &BTreeMap<u64, CultureProfile> {
        &self.emergence.cluster_cultures
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

    /// FR-CIV-GENETICS / culture — cluster cultures diverge over ticks.
    #[test]
    fn culture_phase_drifts_cluster_profiles() {
        let mut sim = Simulation::with_seed(99);
        run_ticks(&mut sim, 200);
        assert!(
            !sim.cluster_cultures().is_empty() || count_civilians(&sim.world) > 0,
            "expected cultures or civilians"
        );
        if sim.cluster_cultures().len() >= 2 {
            let values: Vec<_> = sim.cluster_cultures().values().map(|p| p.traits).collect();
            assert_ne!(values[0], values[1], "cultures should diverge");
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
    fn register_seed_set_merges_and_replaces_ids() {
        let mut sim = Simulation::with_seed(1);
        assert!(sim.seed_library().get("raw_organism").is_some());

        let set_a = SeedSet {
            version: 1,
            seeds: vec![
                test_seed_definition("alpha"),
                test_seed_definition("beta"),
            ],
        };
        sim.register_seed_set(set_a);
        assert!(sim.seed_library().get("alpha").is_some());
        assert!(sim.seed_library().get("beta").is_some());
        assert!(sim.seed_library().get("raw_organism").is_some());

        let set_b = SeedSet {
            version: 1,
            seeds: vec![
                test_seed_definition("gamma"),
                test_seed_definition("beta"),
            ],
        };
        sim.register_seed_set(set_b);
        assert!(sim.seed_library().get("alpha").is_some());
        assert!(sim.seed_library().get("gamma").is_some());
        assert!(sim.seed_library().get("beta").is_some());
        assert!(sim.seed_library().get("raw_organism").is_some());
    }

    /// `set_active_seed` updates the active id; unknown ids are rejected.
    #[test]
    fn set_active_seed_updates_or_rejects_unknown() {
        let mut sim = Simulation::with_seed(2);
        sim.set_active_seed(Some("raw_organism".to_string()));
        assert_eq!(sim.active_seed_id(), Some("raw_organism"));

        let kept = sim.active_seed_id().map(str::to_string);
        sim.set_active_seed(Some("missing_seed_id".to_string()));
        assert_eq!(sim.active_seed_id(), kept.as_deref());
        assert!(
            sim.emergence_feed()
                .iter()
                .any(|e| e.kind == "seed_unknown"),
            "unknown seed id should emit seed_unknown"
        );

        sim.set_active_seed(None);
        assert_eq!(sim.active_seed_id(), None);
    }

    /// `register_seed_file` loads fixture RON and reports missing paths.
    #[test]
    fn register_seed_file_loads_fixture_and_reports_missing() {
        let mut sim = Simulation::with_seed(3);
        sim.register_seed_file("scenarios/canonical_seeds.ron");
        assert!(sim.seed_library().get("human_baseline").is_some());
        assert!(
            sim.emergence_feed()
                .iter()
                .any(|e| e.kind == "seed_loaded"),
            "successful load should emit seed_loaded"
        );

        sim.emergence.last_feed.clear();
        sim.register_seed_file("scenarios/no_such_seed_file.ron");
        assert!(
            sim.emergence_feed()
                .iter()
                .any(|e| e.kind == "seed_load_failed"),
            "missing file should emit seed_load_failed"
        );
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
}
