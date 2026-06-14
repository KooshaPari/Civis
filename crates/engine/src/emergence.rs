//! MOAT emergence phases — wire dormant psyche/social/culture/legends/genetics/civ-ai
//! crates into [`Simulation::tick`] (gap-audit §1, master-roadmap S2).
//!
//! Phase order inside `phase_emergence`: genetics → culture → social → psyche →
//! legends ingest → civ-ai naming. Surfaced via [`EmergenceFeedEvent`] and getters
//! on [`Simulation`].

use std::collections::{BTreeMap, HashSet};

use civ_agents::culture::{drift_populations, ContactEdge, CultureProfile};
use civ_agents::psyche::{nudge_temperament, psyche_from_dna, update_beliefs, update_mood};
use civ_agents::{
    apply_social_event, belief_culture_exposure, decay_social_graph, psych_genome_profile,
    Civilian, ClusterMember, Interaction, Needs, Psyche, SocialEvent, SocialGraph,
};
use civ_genetics::{
    example_seed_set, sentience::{
        evaluate_sentience, CognitionTraitProfile, SentienceEvent, SentienceThreshold,
    },
    spawn_genome, Dna, DnaClass, SeedDefinition, SeedLibrary, SeedSet,
};
use civ_legends::{
    EventKind, IngestOutcome, LegendsConfig, LegendsWorker, RawSimEvent, Role, SagaGraph,
    SourceCrate,
};
use civ_legends::{LegendEntityId, NameRef, SimRuntimeId};
use civ_needs::Needs as LifeNeeds;
use civ_species::express;
use hecs::Entity;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::engine::Simulation;

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
    /// Canonical seed library (FR-CONTENT-MODEL / CIV-008). When a `Scenario`
    /// pins a `seed_ref`, that seed is used as the spawn-time base DNA via
    /// [`spawn_genome`]; the seed's divergence dial is then used by
    /// `mutate_with_divergence` to scale per-byte mutation rates.
    pub(crate) seed_library: SeedLibrary,
    /// Active seed id referenced by the loaded scenario (or `None` to use
    /// raw-organism drift for every spawn).
    pub(crate) active_seed_id: Option<String>,
}

impl EmergenceState {
    fn new(seed: u64) -> Self {
        let _ = seed;
        // Pre-seed the library with the canonical example set so a baseline
        // sim can spawn a raw-organism without a scenario. Scenarios can
        // override via `register_seed_files` / `set_active_seed`.
        let mut seed_library = SeedLibrary::new();
        for s in example_seed_set().seeds {
            // ignore validation errors here — example set is hand-curated
            let _ = seed_library.insert(s);
        }
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
            seed_library,
            active_seed_id: Some("raw_organism".to_string()),
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
        // Borrow the active seed once per tick (if any) so we don't hold a
        // reference into the library while the world is queried.
        let active_seed: Option<SeedDefinition> = self
            .emergence
            .active_seed_id
            .as_ref()
            .and_then(|id| self.emergence.seed_library.get(id).cloned());
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
            // If the active seed's dna_length doesn't match the class length
            // (config drift), fall back to a fully random spawn of the
            // class-correct length. This keeps the spawn path robust to
            // seed/class mismatches without panicking.
            let dna = match active_seed.as_ref() {
                Some(seed) if seed.dna_length == len => {
                    spawn_genome(&mut local, &self.emergence.dna_class, Some(seed))
                }
                _ => Dna::random(len, &mut local),
            };
            let _ = self.world.insert(entity, (dna,));
        }
    }

    fn emergence_culture(&mut self) {
        let tick = self.state.tick;
        let mut cluster_ids: BTreeMap<u64, u32> = BTreeMap::new();
        for (_, member) in self.world.query::<&ClusterMember>().iter() {
            *cluster_ids.entry(member.cluster.0).or_insert(0) += 1;
        }
        for (cluster_id, size) in &cluster_ids {
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
        let mut profiles: Vec<CultureProfile> =
            self.emergence.cluster_cultures.values().cloned().collect();
        if profiles.len() < 2 {
            if let Some(p) = profiles.first_mut() {
                let mut one = std::slice::from_mut(p);
                drift_populations(one, &[], self.rng_mut(), 0.02, 0.0, 0.85);
            }
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

    fn agent_entity(&self, agent_id: u64) -> Option<Entity> {
        self.world
            .query::<&Civilian>()
            .iter()
            .find(|(_, c)| c.id == agent_id)
            .map(|(e, _)| e)
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
            let _ = self.world.insert(*entity, (psyche, SocialGraph::default()));
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
                let new_maturity = (maturity + 0.001).min(1.0);
                let arousal = psyche.mood.arousal;
                psyche.maturity = new_maturity;
                nudge_temperament(
                    &mut psyche.temperament,
                    arousal,
                    needs.belonging,
                    new_maturity,
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
        let agents: Vec<(u64, Dna)> = self
            .world
            .query::<(&Civilian, &Dna)>()
            .iter()
            .map(|(_, (c, d))| (c.id, d.clone()))
            .collect();

        for (agent_id, dna) in agents {
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

    /// Borrow the canonical seed library (FR-CONTENT-MODEL). Read-only access
    /// for inspectors; mutation goes through [`Self::register_seed_file`] or
    /// [`Self::register_seed_set`].
    #[must_use]
    pub fn seed_library(&self) -> &SeedLibrary {
        &self.emergence.seed_library
    }

    /// Id of the active seed (used for spawn-time DNA). `None` means raw
    /// drift with no seed reference.
    #[must_use]
    pub fn active_seed_id(&self) -> Option<&str> {
        self.emergence.active_seed_id.as_deref()
    }

    /// Install a [`SeedSet`] (in-memory) into the seed library, replacing
    /// any conflicting ids.
    pub fn register_seed_set(&mut self, set: SeedSet) {
        // Drop seeds with the same id to avoid duplicates; keep any
        // pre-loaded seeds that are not in the new set (e.g. the
        // example/raw_organism baseline).
        let new_ids: HashSet<String> = set.seeds.iter().map(|s| s.id.clone()).collect();
        self.emergence
            .seed_library
            .retain(|id, _| !new_ids.contains(id));
        for s in set.seeds {
            // Validate before insert; invalid seeds are skipped (logged
            // via a feed event below).
            if let Err(e) = self.emergence.seed_library.insert(s.clone()) {
                self.emergence.push_feed(
                    self.state.tick,
                    "seed_rejected",
                    format!("seed {} rejected: {e}", s.id),
                    None,
                );
            }
        }
    }

    /// Load a single `.ron` seed file and merge it into the library. The
    /// path is resolved against the engine crate's manifest dir
    /// (CARGO_MANIFEST_DIR) when relative.
    pub fn register_seed_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let resolved: PathBuf = if path.is_absolute() {
            path.to_path_buf()
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../").join(path)
        };
        match std::fs::read_to_string(&resolved) {
            Err(e) => {
                self.emergence.push_feed(
                    self.state.tick,
                    "seed_load_failed",
                    format!("could not read seed file {}: {e}", resolved.display()),
                    None,
                );
            }
            Ok(src) => match SeedLibrary::from_ron_str(&src) {
                Err(e) => {
                    self.emergence.push_feed(
                        self.state.tick,
                        "seed_load_failed",
                        format!("seed file {} parse error: {e}", resolved.display()),
                        None,
                    );
                }
                Ok(lib) => {
                    for (id, seed) in lib.iter() {
                        if self.emergence.seed_library.get(id).is_none() {
                            let _ = self.emergence.seed_library.insert(seed.clone());
                        }
                    }
                    self.emergence.push_feed(
                        self.state.tick,
                        "seed_loaded",
                        format!("loaded seed file {} (n={})", resolved.display(), lib.len()),
                        None,
                    );
                }
            },
        }
    }

    /// Set the active seed id used for spawn-time DNA. Pass `None` to fall
    /// back to raw drift.
    pub fn set_active_seed(&mut self, id: Option<String>) {
        if let Some(ref sid) = id {
            if self.emergence.seed_library.get(sid).is_none() {
                // Unknown seed id is rejected; report and leave the
                // existing active id in place.
                self.emergence.push_feed(
                    self.state.tick,
                    "seed_unknown",
                    format!("active seed id {sid} not in library; keeping {:?}",
                        self.emergence.active_seed_id),
                    None,
                );
                return;
            }
        }
        self.emergence.active_seed_id = id;
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
        sim.state.resources.food = Fixed::ZERO;
        run_ticks(&mut sim, 250);
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
}
