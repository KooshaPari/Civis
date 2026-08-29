//! World simulation phases: voxel, buildings, production, diffusion, audio,
//! and disaster methods extracted from engine.rs (Pass 9 — Civis Engine
//! Decomposition).

use crate::engine::{Building, BuildingType, CombatDamagePulse, Simulation};
use crate::fixed_math::Fixed;
use crate::lod::should_tick_entity_with_policy;
use crate::lod::LodPolicy;
use crate::settlement_helpers::{settlement_dominant_factions, settlement_member_counts};
use crate::SimRng;
use civ_agents::count_civilians;
use civ_agents::culture::TraitVector;
use civ_agents::{CohortStats, LodTier, Tools, Wardrobe};
use civ_audio::triggers::SfxTrigger;
use civ_build::{BuildSite, DemandSignals, ProductionEvent};
use civ_diffusion::DiffusionParams;
use std::collections::BTreeMap;

#[inline]
fn propagate_cohort_wardrobe_with_lod(
    world: &mut hecs::World,
    target_era: u16,
    params: DiffusionParams,
    rng: &mut SimRng,
    tick: u64,
    policy: LodPolicy,
) -> CohortStats {
    let total_civilians = count_civilians(world) as u32;
    let mut currently_at_target = world
        .query::<&Wardrobe>()
        .iter()
        .filter(|(_, wardrobe)| wardrobe.era >= target_era)
        .count() as u32;
    let current_fraction = if total_civilians == 0 {
        0.0
    } else {
        currently_at_target as f32 / total_civilians as f32
    };

    let mut promoted_this_tick = 0_u32;
    for (_, (wardrobe, lod)) in world.query_mut::<(&mut Wardrobe, &LodTier)>().into_iter() {
        if !should_tick_entity_with_policy(tick, *lod, policy) {
            continue;
        }
        if wardrobe.era < target_era
            && civ_agents::propagate_wardrobe(wardrobe, target_era, current_fraction, params, rng)
        {
            promoted_this_tick += 1;
        }
    }

    currently_at_target = world
        .query::<&Wardrobe>()
        .iter()
        .filter(|(_, wardrobe)| wardrobe.era >= target_era)
        .count() as u32;

    CohortStats {
        promoted_this_tick,
        currently_at_target,
        total_civilians,
        current_fraction,
    }
}

#[inline]
fn propagate_cohort_tools_with_lod(
    world: &mut hecs::World,
    target_era: u16,
    params: DiffusionParams,
    rng: &mut SimRng,
    tick: u64,
    policy: LodPolicy,
) -> CohortStats {
    let total_civilians = count_civilians(world) as u32;
    let mut currently_at_target = world
        .query::<&Tools>()
        .iter()
        .filter(|(_, tools)| tools.era >= target_era)
        .count() as u32;
    let current_fraction = if total_civilians == 0 {
        0.0
    } else {
        currently_at_target as f32 / total_civilians as f32
    };

    let mut promoted_this_tick = 0_u32;
    for (_, (tools, lod)) in world.query_mut::<(&mut Tools, &LodTier)>().into_iter() {
        if !should_tick_entity_with_policy(tick, *lod, policy) {
            continue;
        }
        if tools.era < target_era
            && civ_agents::propagate_tools(tools, target_era, current_fraction, params, rng)
        {
            promoted_this_tick += 1;
        }
    }

    currently_at_target = world
        .query::<&Tools>()
        .iter()
        .filter(|(_, tools)| tools.era >= target_era)
        .count() as u32;

    CohortStats {
        promoted_this_tick,
        currently_at_target,
        total_civilians,
        current_fraction,
    }
}

/// Derive a per-cluster music cue from culture traits (stub).
#[inline]
pub fn derive_music_cue(
    traits: TraitVector,
    cluster_id: u64,
    aggression: f32,
    tick: u64,
) -> crate::engine::MusicCue {
    let trait_mean = traits.iter().copied().sum::<f32>() / traits.len() as f32;
    let cultural_pulse = (((tick.wrapping_add(cluster_id)) % 16) as f32 / 15.0 - 0.5) * 0.08;
    let intensity = (0.25 + trait_mean * 0.55 + aggression.clamp(0.0, 1.0) * 0.2 + cultural_pulse)
        .clamp(0.0, 1.0);
    let mood = if trait_mean < 0.3 {
        "pastoral"
    } else if trait_mean < 0.55 {
        "balanced"
    } else if trait_mean < 0.75 {
        "driven"
    } else {
        "ceremonial"
    };
    let tempo = (72.0
        + trait_mean * 42.0
        + aggression.clamp(0.0, 1.0) * 18.0
        + ((tick.wrapping_add(cluster_id)) % 8) as f32)
        .round()
        .clamp(40.0, 180.0) as u16;
    crate::engine::MusicCue {
        mood: mood.to_string(),
        intensity,
        tempo_bpm: Some(tempo),
    }
}

impl Simulation {
    /// Voxel phase — drains the deterministic dirty-event queue from
    /// [`VoxelWorld`](civ_voxel::VoxelWorld) into
    /// `Simulation::last_tick_voxel_events`.
    pub(crate) fn phase_voxel(&mut self) {
        self.last_tick_voxel_events = self.voxel.drain_dirty();
    }

    /// Compact the voxel world periodically.
    pub(crate) fn phase_compact(&mut self) {
        if self.state.tick % self.tick_modulo_compact == 0 {
            self.voxel.compact();
        }
    }

    /// Construction sites phase - expands the parcel graph on a fixed cadence when demand is high.
    /// (Renamed from `phase_buildings` so the engine can introduce a `phase_buildings`
    ///  that drives the `building_layouts` module on its own cadence — see
    ///  `Simulation::phase_buildings` in `engine.rs`.)
    pub(crate) fn phase_construction_sites(&mut self) {
        let tick = self.state.tick;

        // ---- 1. Parcel allocation cadence (every 16 ticks) ----
        if tick % 16 == 0 {
            let signals = DemandSignals {
                residential: 0.75,
                commercial: 0.25,
                industrial: 0.25,
                civic: 0.75,
            };

            if [
                signals.residential,
                signals.commercial,
                signals.industrial,
                signals.civic,
            ]
            .iter()
            .any(|signal| *signal > 0.5)
            {
                let origin = civ_voxel::WorldCoord { x: 0, y: 0, z: 0 };
                let allocated = self.allocator.allocate(
                    &mut self.building_graph,
                    &signals,
                    self.target_era,
                    origin,
                    16,
                );
                if !allocated.is_empty() {
                    use crate::building_emergence::{
                        apply_emergence_facades, emergence_demand_signals,
                        emergent_style_key_for_sim, settlement_build_anchor,
                    };
                    let geology = civ_planet::GeologyMap::seed(&self.planet);
                    let (cluster_id, anchor) = settlement_build_anchor(&self.world);
                    let style = emergent_style_key_for_sim(self, cluster_id, &geology, &anchor);
                    let gated = emergence_demand_signals(self, signals, style.era);
                    apply_emergence_facades(self, cluster_id, style, gated, &allocated);
                }
            }
        }

        // ---- 2. Construction progress + 3. production events (single pass) ----
        // PERF: previously this was three separate iterations over
        // `self.build_sites` (progress / retain / produce). Folding the
        // production-events pass into the same loop halves the iteration
        // count and keeps the borrow checker happy with `iter_mut()`.
        let mut events = std::mem::take(&mut self.last_tick_construction_events);
        let mut completed_ids: smallvec::SmallVec<[civ_build::BuildingId; 8]> =
            smallvec::SmallVec::new();
        for site in self.build_sites.iter_mut() {
            if site.is_complete() {
                continue;
            }
            if site.tick().is_some() {
                let site_id = site.id();
                completed_ids.push(site_id);
                self.building_graph.record_completed(site);
                events.extend(site.produce_and_collect(&mut self.economy_state, tick));
            }
        }
        // Drop sites that completed on a previous tick (their `id()` won't
        // appear in `completed_ids` for this tick).
        self.build_sites
            .retain(|site| !site.is_complete() || completed_ids.contains(&site.id()));
        self.last_tick_construction_events = events;
    }

    /// Public accessor for the most recent construction events.
    pub fn last_construction_events(&self) -> &[ProductionEvent] {
        &self.last_tick_construction_events
    }

    /// Enqueue a new construction site.
    pub fn enqueue_build_site(&mut self, site: BuildSite) {
        self.build_sites.push(site);
    }

    /// Read-only view of the active build queue.
    pub fn build_sites(&self) -> &[BuildSite] {
        &self.build_sites
    }

    /// Count of completed buildings.
    pub fn completed_buildings(&self) -> usize {
        self.building_graph.completed_count()
    }

    /// Research phase (FR-ERA): emergent per-faction research progress.
    pub(crate) fn phase_research(&mut self) {
        crate::era::phase_research(self);
    }

    /// Tech-tree phase (FR-ERA): emergent tech levels + era evaluation.
    pub(crate) fn phase_tech(&mut self) {
        crate::era::phase_tech(self);
    }

    /// Diffusion phase - propagates wardrobe and tools eras across civilians.
    pub(crate) fn phase_diffusion(&mut self) {
        let tick = self.state.tick;
        let policy = self.lod_policy;
        let wardrobe_stats = propagate_cohort_wardrobe_with_lod(
            &mut self.world,
            self.target_era,
            self.diffusion_params,
            &mut self.rng,
            tick,
            policy,
        );
        let _tools_stats = propagate_cohort_tools_with_lod(
            &mut self.world,
            self.target_era,
            self.diffusion_params,
            &mut self.rng,
            tick,
            policy,
        );

        debug_assert_eq!(
            wardrobe_stats.total_civilians,
            count_civilians(&self.world) as u32
        );
        self.last_cohort_stats = Some(wardrobe_stats);
    }

    /// Audio phase (FR-AUDIO-wire) — translate per-tick substrate events
    /// into substrate-level [`SfxTrigger`]s.
    pub(crate) fn phase_audio(&mut self) {
        let mut events: Vec<SfxTrigger> =
            Vec::with_capacity(self.last_tick_audio_events.capacity());

        events.extend(self.last_births.iter().map(|_| SfxTrigger::Birth));
        events.extend(self.last_deaths.iter().map(|_| SfxTrigger::Death));

        let researched_len = self.research_cache.researched.len();
        if researched_len > self.last_audio_researched_len {
            events
                .extend((self.last_audio_researched_len..researched_len).map(|_| SfxTrigger::Tech));
        }
        self.last_audio_researched_len = researched_len;

        for pulse in &self.last_tick_combat_pulses {
            let dx = pulse.x - 0.5;
            let dy = pulse.y - 0.5;
            let dist = ((dx * dx + dy * dy).sqrt() * 2.0).clamp(0.0, 1.0);
            let intensity = 1.0 - dist;
            events.push(SfxTrigger::Battle { intensity });
        }

        for event in &self.last_tick_construction_events {
            if matches!(event, ProductionEvent::Produced { .. }) {
                events.push(SfxTrigger::Build);
            }
        }

        for trigger in &self.last_tick_audio_events {
            if matches!(trigger, SfxTrigger::Disaster { .. }) {
                events.push(*trigger);
            }
        }

        let cluster_member_counts = settlement_member_counts(&self.world);
        let dominant = settlement_dominant_factions(&self.world, &cluster_member_counts);
        let mut cues = BTreeMap::new();
        for (&cluster_id, profile) in &self.cluster_cultures {
            let faction_id = dominant.get(&cluster_id).copied();
            let aggression = faction_id
                .and_then(|id| self.faction_aggression.get(&id))
                .copied()
                .unwrap_or(0.0);
            cues.insert(
                cluster_id,
                derive_music_cue(profile.traits, cluster_id, aggression, self.state.tick),
            );
        }
        self.last_tick_music_cues = cues;

        self.last_tick_audio_events = events;
    }

    /// Record a [`SfxTrigger::Disaster`] on the per-tick audio buffer.
    pub fn record_disaster_audio(&mut self, kind: &str, severity: f32) {
        let label: &'static str = match kind.to_ascii_lowercase().as_str() {
            "meteor" => "meteor",
            "flood" => "flood",
            "quake" | "earthquake" => "quake",
            "wildfire" | "fire" => "wildfire",
            "storm" => "storm",
            "plague" => "plague",
            _ => "disaster",
        };
        self.last_tick_audio_events.push(SfxTrigger::Disaster {
            kind: label,
            severity: severity.clamp(0.0, 1.0),
        });
    }

    pub(crate) fn push_disaster_event(&mut self, event: crate::disasters::DisasterTickEvent) {
        let kind = match event.kind {
            crate::disasters::DisasterKind::Meteor => "meteor",
            crate::disasters::DisasterKind::Flood => "flood",
            crate::disasters::DisasterKind::Quake => "quake",
            crate::disasters::DisasterKind::Wildfire => "wildfire",
            crate::disasters::DisasterKind::Storm => "storm",
            crate::disasters::DisasterKind::Drought => "drought",
            crate::disasters::DisasterKind::Plague => "plague",
        };
        self.record_disaster_audio(kind, 1.0);
        self.last_tick_disaster_events.push(event);
    }

    /// Production phase - buildings produce resources.
    #[inline]
    pub(crate) fn phase_production(&mut self) {
        let mut food = Fixed::ZERO;
        let wood = Fixed::ZERO;
        let mut metal = Fixed::ZERO;
        let mut energy = Fixed::ZERO;

        for (_, building) in self.world.query::<&Building>().iter() {
            match building.building_type {
                BuildingType::Farm => {
                    food += Fixed::from_num(1);
                }
                BuildingType::Mine => {
                    metal += Fixed::from_num(1);
                }
                BuildingType::CityCenter => {
                    energy += Fixed::from_bits(Fixed::from_num(1).to_bits() / 2);
                }
                _ => {}
            }
        }
        self.state.resources.food += food;
        self.state.resources.wood += wood;
        self.state.resources.metal += metal;
        self.state.resources.energy += energy;
    }
}
