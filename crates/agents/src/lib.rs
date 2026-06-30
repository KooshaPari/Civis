//! civ-agents — civilian agent ECS components + LOD tick + per-civilian
//! wardrobe / tools state.
//!
//! Components live in a shared `hecs::World`. This crate ships:
//!
//! - `Civilian` — identity + age + alignment
//! - `Wardrobe` — current clothing era + material slot (diffusion-driven)
//! - `Tools` — current tool era + material slot (diffusion-driven)
//! - `Needs` — utility-AI scalar weights (food, shelter, safety, belonging)
//! - `LodTier::{Hot, Warm, Cold}` — simulation fidelity level
//! - `Position3d` — fixed-point world coordinates (composes with `civ-voxel`'s
//!   `WorldCoord`)
//!
//! Diffusion-driven propagation: `propagate_era` consults a tech-adoption
//! S-curve from `civ-diffusion` to decide whether to bump a civilian's
//! `wardrobe.era` or `tools.era` this tick. Per ADR-008 + the always-auto
//! determinism rules, propagation is pure / seeded — no time leaks.
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-AGENTS-*`.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

pub mod cluster;
pub mod culture;
pub mod daily_path;
pub mod diplomacy;
pub mod psyche;
pub mod social;

pub use cluster::{
    cluster_by_colocation, reconcile_membership, should_join, should_leave, ClusterId,
    ClusterMember, MembershipPayoff,
};
pub use daily_path::{
    choose_activity, need_for_poi_kind, path_step, pick_target, poi_kind_for_need, score_poi,
    wander_anchor, Activity, DailyGoal, DailyPathDecision, Poi, PoiKind, PoiRegistry,
};
pub use diplomacy::{
    DiplomacyMatrix, DiplomacyOutcome, DiplomacySignal, RelationKind, RelationRecord,
};
pub use psyche::{
    belief_culture_exposure, psych_genome_profile, Mood, PsychGenomeProfile, Psyche, Temperament,
    PSYCHE_DIM,
};
pub use social::{
    apply_social_event, decay_social_graph, relation_label, Interaction, RelationLabel,
    SocialEvent, SocialGraph, Tie, MAX_TIES,
};

use civ_diffusion::{advance as diffusion_advance, DiffusionParams};
use civ_voxel::{MaterialId, WorldCoord};
use hecs::World;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version. Bumped on breaking changes.
pub const SCHEMA_VERSION: &str = "0.1.0";

/// Minimum number of positive social ties required to form a new faction.
/// A "positive" tie is one with affinity >= FRIEND_AFFINITY_THRESHOLD or
/// kinship >= KIN_THRESHOLD.
pub const FORM_FACTION_COHESION: usize = 3;

/// Minimum number of existing ties to current faction members required to join
/// an existing faction.
pub const JOIN_FACTION_ACCEPTANCE: usize = 2;

/// Affinity floor above which a cooperative tie counts as "positive" for
/// faction cohesion. A single cooperative interaction (affinity > 0) qualifies;
/// hostile ties (affinity <= 0) do not.
pub const FRIEND_AFFINITY_THRESHOLD: f32 = 0.0;

/// Kinship threshold above which a tie is considered "kin" for faction
/// formation/joining purposes.
pub const KIN_THRESHOLD: f32 = 0.5;

/// Returns true if the tie is positive (cooperative or kin) for faction cohesion.
fn is_positive_tie(tie: &Tie) -> bool {
    tie.affinity > FRIEND_AFFINITY_THRESHOLD || tie.kinship >= KIN_THRESHOLD
}

/// Which CC0 glTF rig the Bevy client should render for this agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorVisualKind {
    /// Humanoid civilian (KayKit Knight / capsule fallback).
    Humanoid,
    /// Non-combat herd / fauna (skeleton minion rig).
    Herd,
}

/// Marks the visual model variant chosen at spawn (herd tool vs organism).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorVisual(pub ActorVisualKind);

/// Civilian identity component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Civilian {
    /// Stable agent ID.
    pub id: u64,
    /// Emergent alignment: none, a faction, or another life-framework entity.
    pub alignment: Alignment,
    /// Age in years (game-time).
    pub age: u16,
}

/// Emergent political/social alignment for a civilian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Alignment {
    /// Civilian has not aligned yet.
    #[default]
    None,
    /// Civilian aligns to an existing faction.
    Faction(u32),
    /// Civilian aligns to another life-framework entity by id.
    OtherEntity(u64),
}

impl Alignment {
    /// Construct an explicit faction alignment.
    pub const fn with_faction(faction_id: u32) -> Self {
        Self::Faction(faction_id)
    }

    /// Construct an explicit entity alignment.
    pub const fn with_entity(entity_id: u64) -> Self {
        Self::OtherEntity(entity_id)
    }

    /// Attempt to form a new faction with this agent as founder (FR-CIV-EMERGENCE-001).
    ///
    /// Formation is gated on emergent social cohesion: the founder must hold at
    /// least [`FORM_FACTION_COHESION`] positive ties (cooperative or kin). Below
    /// that, no faction coalesces and the alignment is left unchanged.
    pub fn form_faction(self, graph: &SocialGraph, new_faction_id: u32, _founder_id: u64) -> Self {
        let cohesion = graph.ties.iter().filter(|tie| is_positive_tie(tie)).count();
        if cohesion >= FORM_FACTION_COHESION {
            Self::Faction(new_faction_id)
        } else {
            self
        }
    }

    /// Attempt to join an existing faction (FR-CIV-EMERGENCE-002).
    ///
    /// Joining is gated on social acceptance: the joiner must hold at least
    /// [`JOIN_FACTION_ACCEPTANCE`] positive ties to current `members` of the
    /// target faction. Ties to non-members and hostile ties do not count.
    pub fn join_faction(
        self,
        graph: &SocialGraph,
        faction_id: u32,
        _agent_id: u64,
        members: &[u64],
    ) -> Self {
        let acceptance = graph
            .ties
            .iter()
            .filter(|tie| members.contains(&tie.other) && is_positive_tie(tie))
            .count();
        if acceptance >= JOIN_FACTION_ACCEPTANCE {
            Self::Faction(faction_id)
        } else {
            self
        }
    }
}

/// Infer alignment for a new spawn at `(x, y)`.
///
/// Returns the majority non-None alignment among all civilians within a
/// `SPAWN_RADIUS` of the spawn point. This is the minimal slice of
/// FR-CIV-EMERGENCE-010 (kinship proximity) and FR-CIV-EMERGENCE-002
/// (agent bootstrap from local context).
pub fn infer_alignment_for_spawn(world: &World, x: f32, y: f32) -> Alignment {
    const SPAWN_RADIUS: f32 = 0.05;
    let mut counts: HashMap<Alignment, usize> = HashMap::new();
    for (_, (civilian, pos)) in world.query::<(&Civilian, &Position3d)>().iter() {
        let px = pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32;
        let py = pos.coord.z as f32 / civ_voxel::FIXED_SCALE as f32;
        let dx = px - x;
        let dy = py - y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq <= SPAWN_RADIUS * SPAWN_RADIUS {
            *counts.entry(civilian.alignment).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .filter(|(alignment, _)| *alignment != Alignment::None)
        .max_by_key(|(_, count)| *count)
        .map(|(alignment, _)| alignment)
        .unwrap_or(Alignment::None)
}

/// Wardrobe state. The `era` is the civilian's currently worn-tech era;
/// `material` is the visible material slot the renderer uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wardrobe {
    /// Era index of the currently-worn clothing.
    pub era: u16,
    /// Material slot driving renderer colour / texture.
    pub material: MaterialId,
}

/// Tool inventory state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tools {
    /// Era index of the currently-held tools.
    pub era: u16,
    /// Material slot.
    pub material: MaterialId,
}

/// Utility-AI scalar weights ∈ `[0, 1]`. Higher = more pressing need.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Needs {
    /// Food need.
    pub food: f32,
    /// Shelter need.
    pub shelter: f32,
    /// Safety need (combat/disease threat).
    pub safety: f32,
    /// Belonging need (social).
    pub belonging: f32,
}

/// Home assignment for a civilian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HomeAssignment {
    /// Stable building ID for the assigned home.
    pub building_id: u64,
}

/// Simulation fidelity tier. Far-from-camera civilians collapse to lower tiers
/// to bound the per-tick cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LodTier {
    /// Full fidelity — every tick.
    Hot,
    /// Reduced fidelity — every 4 ticks.
    Warm,
    /// Gestalt — every 16 ticks.
    Cold,
}

/// Fixed-point world position. Composes with `civ-voxel`'s `WorldCoord` at
/// `FIXED_SCALE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position3d {
    /// `WorldCoord` from `civ-voxel`.
    pub coord: WorldCoord,
}

/// Per-tick movement direction for a civilian.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    /// Normalised x direction.
    pub dx: f32,
    /// Normalised y direction.
    pub dy: f32,
}

/// Spawn-time bundle for velocity, wardrobe, tools, needs, and LOD tier.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CivilianBundle {
    /// Initial movement direction.
    pub velocity: Velocity,
    /// Starting wardrobe.
    pub wardrobe: Wardrobe,
    /// Starting tools.
    pub tools: Tools,
    /// Starting needs.
    pub needs: Needs,
    /// Simulation fidelity tier.
    pub lod: LodTier,
}

impl CivilianBundle {
    /// Era-zero civilian with balanced needs and hot LOD.
    pub fn newborn_default(velocity: Velocity) -> Self {
        Self {
            velocity,
            wardrobe: Wardrobe {
                era: 0,
                material: MaterialId(0),
            },
            tools: Tools {
                era: 0,
                material: MaterialId(0),
            },
            needs: Needs {
                food: 0.25,
                shelter: 0.25,
                safety: 0.25,
                belonging: 0.25,
            },
            lod: LodTier::Hot,
        }
    }
}

/// Spawn a child civilian near a parent position using the supplied RNG.
#[must_use]
pub fn child_bundle_from_parent(rng: &mut ChaCha8Rng) -> CivilianBundle {
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    CivilianBundle {
        velocity: Velocity {
            dx: angle.cos(),
            dy: angle.sin(),
        },
        wardrobe: Wardrobe {
            era: 0,
            material: MaterialId(0),
        },
        tools: Tools {
            era: 0,
            material: MaterialId(0),
        },
        needs: Needs {
            food: 0.25,
            shelter: 0.25,
            safety: 0.25,
            belonging: 0.25,
        },
        lod: LodTier::Hot,
    }
}

/// Spawn a child civilian near the supplied normalized position.
pub fn spawn_child_near(
    world: &mut World,
    id: u64,
    alignment: Alignment,
    x: f32,
    y: f32,
    rng: &mut ChaCha8Rng,
) -> hecs::Entity {
    let nx = (x + rng.gen_range(-0.015..0.015)).clamp(0.01, 0.99);
    let ny = (y + rng.gen_range(-0.015..0.015)).clamp(0.01, 0.99);
    spawn_civilian(
        world,
        Civilian {
            id,
            alignment,
            age: 0,
        },
        Position3d {
            coord: WorldCoord {
                x: (nx * civ_voxel::FIXED_SCALE as f32) as i64,
                y: 0,
                z: (ny * civ_voxel::FIXED_SCALE as f32) as i64,
            },
        },
        child_bundle_from_parent(rng),
    )
}

/// Utility-AI action priority derived from unmet needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedAction {
    /// Seek food.
    FindFood,
    /// Seek shelter.
    FindShelter,
    /// Escape danger.
    Flee,
    /// Seek social contact.
    Socialize,
    /// No urgent action.
    Idle,
}

/// Utility weights used when scoring unmet needs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UtilityWeights {
    /// Food weight.
    pub food: f32,
    /// Shelter weight.
    pub shelter: f32,
    /// Safety weight.
    pub safety: f32,
    /// Belonging weight.
    pub belonging: f32,
}

/// Utility-AI needs vector for scoring higher-level actions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NeedsVector {
    /// Hunger pressure.
    pub hunger: f32,
    /// Safety pressure.
    pub safety: f32,
    /// Social pressure.
    pub social: f32,
    /// Purpose pressure.
    pub purpose: f32,
    /// Spirituality pressure.
    pub spirituality: f32,
}

impl NeedsVector {
    /// Highest current need pressure.
    #[must_use]
    pub fn urgency(&self) -> f32 {
        self.hunger
            .max(self.safety)
            .max(self.social)
            .max(self.purpose)
            .max(self.spirituality)
    }
}

/// Coarse world context used by the utility scorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Food is nearby.
    pub food_nearby: bool,
    /// Threat is nearby.
    pub threat_nearby: bool,
    /// Other agents are nearby.
    pub others_nearby: bool,
    /// Temple is nearby.
    pub temple_nearby: bool,
}

/// High-level action considered by the utility scorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    /// Consume food.
    Eat,
    /// Avoid danger.
    Flee,
    /// Seek social contact.
    Socialize,
    /// Do productive work.
    Work,
    /// Visit a temple.
    Pray,
}

/// Score an action for the current needs and world snapshot.
pub trait UtilityScorer {
    /// Score one action. Higher is better.
    fn score_action(
        &self,
        action: &AgentAction,
        needs: &NeedsVector,
        world_state: &WorldSnapshot,
    ) -> f32;
}

/// Simple context-sensitive utility scorer.
#[derive(Debug, Clone, Copy, Default)]
pub struct BasicUtilityScorer;

impl UtilityScorer for BasicUtilityScorer {
    fn score_action(
        &self,
        action: &AgentAction,
        needs: &NeedsVector,
        world_state: &WorldSnapshot,
    ) -> f32 {
        match action {
            AgentAction::Eat if world_state.food_nearby => needs.hunger * 1.5,
            AgentAction::Flee if world_state.threat_nearby => needs.safety * 2.0,
            AgentAction::Socialize if world_state.others_nearby => needs.social * 1.0,
            AgentAction::Work => needs.purpose * 1.0,
            AgentAction::Pray if world_state.temple_nearby => needs.spirituality * 1.0,
            _ => 0.0,
        }
    }
}

/// Pick the highest-scoring available action.
#[must_use]
pub fn choose_action(
    needs: &NeedsVector,
    available: &[AgentAction],
    scorer: &dyn UtilityScorer,
    world: &WorldSnapshot,
) -> AgentAction {
    assert!(
        !available.is_empty(),
        "available actions must not be empty"
    );

    let mut best_action = available[0];
    let mut best_score = scorer.score_action(&best_action, needs, world);
    for action in &available[1..] {
        let score = scorer.score_action(action, needs, world);
        if score > best_score {
            best_action = *action;
            best_score = score;
        }
    }
    best_action
}

/// Cohort-level diffusion statistics for a target era.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CohortStats {
    /// Number of civilians promoted this tick.
    pub promoted_this_tick: u32,
    /// Number of civilians already at or above the target era after the tick.
    pub currently_at_target: u32,
    /// Total civilians considered in the cohort.
    pub total_civilians: u32,
    /// Current adoption fraction at the start of the tick.
    pub current_fraction: f32,
}

/// Spawn one civilian entity in a `hecs::World`.
pub fn spawn_civilian(
    world: &mut World,
    civilian: Civilian,
    position: Position3d,
    bundle: CivilianBundle,
) -> hecs::Entity {
    let CivilianBundle {
        velocity,
        wardrobe,
        tools,
        needs,
        lod,
    } = bundle;
    world.spawn((civilian, position, velocity, wardrobe, tools, needs, lod))
}

/// Return a normalized direction from a civilian toward a home coordinate.
#[must_use]
pub fn drift_toward_home(
    civilian_pos: &Position3d,
    home_pos: &Position3d,
    current_velocity: Velocity,
    shelter_need: f32,
) -> Velocity {
    if shelter_need <= 0.5 {
        return current_velocity;
    }

    let dx = home_pos.coord.x as f32 - civilian_pos.coord.x as f32;
    let dz = home_pos.coord.z as f32 - civilian_pos.coord.z as f32;
    let len = (dx * dx + dz * dz).sqrt();
    if len <= f32::EPSILON {
        return current_velocity;
    }

    Velocity {
        dx: dx / len,
        dy: dz / len,
    }
}

/// Spawn one civilian at a specific normalized terrain position.
pub fn spawn_civilian_at(
    world: &mut World,
    id: u64,
    alignment: Alignment,
    x: f32,
    y: f32,
    visual: ActorVisualKind,
    rng: &mut ChaCha8Rng,
) -> hecs::Entity {
    let angle = rng.gen::<f32>() * std::f32::consts::TAU;
    let velocity = Velocity {
        dx: angle.cos(),
        dy: angle.sin(),
    };
    let coord = WorldCoord {
        x: (x.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64,
        y: 0,
        z: (y.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64,
    };
    let entity = spawn_civilian(
        world,
        Civilian {
            id,
            alignment,
            age: 18 + (id % 50) as u16,
        },
        Position3d { coord },
        CivilianBundle::newborn_default(velocity),
    );
    let _ = world.insert_one(entity, ActorVisual(visual));
    entity
}

/// Spawn a deterministic batch of civilians with sequential IDs.
pub fn spawn_many(
    world: &mut World,
    count: u32,
    seed_civilian_id: u64,
    faction: u32,
) -> Vec<hecs::Entity> {
    let mut entities = Vec::with_capacity(count as usize);
    for offset in 0..count {
        let civilian = Civilian {
            id: seed_civilian_id + u64::from(offset),
            alignment: Alignment::with_faction(faction),
            age: 18 + (offset % 50) as u16,
        };
        let position = Position3d {
            coord: WorldCoord { x: 0, y: 0, z: 0 },
        };
        let wardrobe = Wardrobe {
            era: 0,
            material: MaterialId(0),
        };
        let tools = Tools {
            era: 0,
            material: MaterialId(0),
        };
        let needs = Needs {
            food: 0.25,
            shelter: 0.25,
            safety: 0.25,
            belonging: 0.25,
        };
        let lod = match offset % 3 {
            0 => LodTier::Hot,
            1 => LodTier::Warm,
            _ => LodTier::Cold,
        };
        let angle = (offset as f32) * 0.5;
        entities.push(spawn_civilian(
            world,
            civilian,
            position,
            CivilianBundle {
                velocity: Velocity {
                    dx: angle.cos(),
                    dy: angle.sin(),
                },
                wardrobe,
                tools,
                needs,
                lod,
            },
        ));
    }
    entities
}

/// Count civilian entities in the world.
pub fn count_civilians(world: &World) -> usize {
    world.query::<&Civilian>().iter().count()
}

/// Score unmet needs using utility weights.
pub fn score_needs(needs: &Needs, weights: &UtilityWeights) -> f32 {
    needs.food * weights.food
        + needs.shelter * weights.shelter
        + needs.safety * weights.safety
        + needs.belonging * weights.belonging
}

/// Pick the highest-priority unmet need.
pub fn top_action(needs: &Needs, weights: &UtilityWeights) -> NeedAction {
    let scores = [
        (needs.food * weights.food, NeedAction::FindFood),
        (needs.shelter * weights.shelter, NeedAction::FindShelter),
        (needs.safety * weights.safety, NeedAction::Flee),
        (needs.belonging * weights.belonging, NeedAction::Socialize),
    ];
    scores
        .into_iter()
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(_, action)| action)
        .unwrap_or(NeedAction::Idle)
}

/// Return whether a civilian should tick on the current simulation tick.
pub fn should_tick_now(tier: LodTier, current_tick: u64) -> bool {
    match tier {
        LodTier::Hot => true,
        LodTier::Warm => current_tick % 4 == 0,
        LodTier::Cold => current_tick % 16 == 0,
    }
}

fn movement_speed_factor(tier: LodTier) -> f32 {
    match tier {
        LodTier::Hot => 0.002,
        LodTier::Warm => 0.0014,
        LodTier::Cold => 0.0009,
    }
}

/// Advance civilian movement one tick.
pub fn tick_movement<F>(world: &mut World, terrain_size: u32, rng: &mut ChaCha8Rng, is_walkable: F)
where
    F: Fn(f32, f32) -> bool,
{
    for (_, (pos, vel, _, lod)) in
        world.query_mut::<(&mut Position3d, &mut Velocity, &Civilian, &LodTier)>()
    {
        if !should_tick_now(*lod, terrain_size as u64) {
            continue;
        }

        if rng.gen_range(0..50) == 0 {
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            vel.dx = angle.cos();
            vel.dy = angle.sin();
        }

        let (x, y) = (
            pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32,
            pos.coord.z as f32 / civ_voxel::FIXED_SCALE as f32,
        );
        let speed = movement_speed_factor(*lod);
        let next_x = (x + vel.dx * speed).clamp(0.01, 0.99);
        let next_y = (y + vel.dy * speed).clamp(0.01, 0.99);
        let mut final_x = next_x;
        let mut final_y = next_y;

        if !is_walkable(next_x, next_y) {
            vel.dx = -vel.dx;
            vel.dy = -vel.dy;
            final_x = (x + vel.dx * speed).clamp(0.01, 0.99);
            final_y = (y + vel.dy * speed).clamp(0.01, 0.99);
            if !is_walkable(final_x, final_y) {
                final_x = x;
                final_y = y;
            }
        }

        pos.coord.x = (final_x * civ_voxel::FIXED_SCALE as f32) as i64;
        pos.coord.z = (final_y * civ_voxel::FIXED_SCALE as f32) as i64;
    }
}

/// Drive era propagation for one tick on one civilian's [`Wardrobe`]. The
/// `target_era` is the civilization-wide "current best" era; this tick may or
/// may not promote the civilian based on the diffusion S-curve sample and
/// the RNG draw.
///
/// Returns `true` if the wardrobe era was bumped.
pub fn propagate_wardrobe(
    wardrobe: &mut Wardrobe,
    target_era: u16,
    civ_adoption_fraction: f32,
    params: DiffusionParams,
    rng: &mut ChaCha8Rng,
) -> bool {
    if wardrobe.era >= target_era {
        return false;
    }
    // Per-tick adoption rate as a probability that THIS civilian flips this tick.
    let rate = diffusion_advance(civ_adoption_fraction, params) - civ_adoption_fraction;
    if rng.gen::<f32>() < rate.max(0.0) {
        wardrobe.era += 1;
        return true;
    }
    false
}

/// Drive era propagation for one tick on one civilian's [`Tools`].
pub fn propagate_tools(
    tools: &mut Tools,
    target_era: u16,
    civ_adoption_fraction: f32,
    params: DiffusionParams,
    rng: &mut ChaCha8Rng,
) -> bool {
    if tools.era >= target_era {
        return false;
    }
    let rate = diffusion_advance(civ_adoption_fraction, params) - civ_adoption_fraction;
    if rng.gen::<f32>() < rate.max(0.0) {
        tools.era += 1;
        return true;
    }
    false
}

/// Propagate wardrobe diffusion across the whole civilian cohort.
pub fn propagate_cohort_wardrobe(
    world: &mut World,
    target_era: u16,
    params: DiffusionParams,
    rng: &mut ChaCha8Rng,
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
    for (_, wardrobe) in world.query_mut::<&mut Wardrobe>() {
        if wardrobe.era < target_era
            && propagate_wardrobe(wardrobe, target_era, current_fraction, params, rng)
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

/// Propagate tools diffusion across the whole civilian cohort.
pub fn propagate_cohort_tools(
    world: &mut World,
    target_era: u16,
    params: DiffusionParams,
    rng: &mut ChaCha8Rng,
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
    for (_, tools) in world.query_mut::<&mut Tools>() {
        if tools.era < target_era
            && propagate_tools(tools, target_era, current_fraction, params, rng)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// Covers FR-CIV-AGENTS-000 — exposes a semver-like schema version stub.
    #[test]
    fn schema_version_stub() {
        assert!(!SCHEMA_VERSION.is_empty());
        let core = SCHEMA_VERSION.split('-').next().unwrap();
        let segments: Vec<&str> = core.split('.').collect();
        assert_eq!(segments.len(), 3);
        assert!(segments.iter().all(|part| !part.is_empty()));
    }

    /// Covers FR-CIV-AGENTS-001 — wardrobe + tools state ticks deterministically under
    /// a fixed seed (replay-safe).
    #[test]
    fn wardrobe_tools_deterministic() {
        let params = DiffusionParams::default();
        let mut wardrobe_a = Wardrobe {
            era: 1,
            material: MaterialId(2),
        };
        let mut wardrobe_b = wardrobe_a;
        let mut tools_a = Tools {
            era: 1,
            material: MaterialId(3),
        };
        let mut tools_b = tools_a;
        let mut r1 = rng(42);
        let mut r2 = rng(42);
        for _ in 0..200 {
            propagate_wardrobe(&mut wardrobe_a, 5, 0.3, params, &mut r1);
            propagate_wardrobe(&mut wardrobe_b, 5, 0.3, params, &mut r2);
            propagate_tools(&mut tools_a, 5, 0.3, params, &mut r1);
            propagate_tools(&mut tools_b, 5, 0.3, params, &mut r2);
        }
        assert_eq!(wardrobe_a, wardrobe_b);
        assert_eq!(tools_a, tools_b);
    }

    /// Covers FR-CIV-AGENTS-001 — propagate_wardrobe is deterministic under a fixed
    /// seed (replay-safe).
    #[test]
    fn propagate_wardrobe_is_deterministic() {
        let params = DiffusionParams::default();
        let mut w1 = Wardrobe {
            era: 1,
            material: MaterialId(2),
        };
        let mut w2 = w1;
        let mut r1 = rng(42);
        let mut r2 = rng(42);
        for _ in 0..200 {
            propagate_wardrobe(&mut w1, 5, 0.3, params, &mut r1);
            propagate_wardrobe(&mut w2, 5, 0.3, params, &mut r2);
        }
        assert_eq!(w1, w2);
    }

    /// Covers FR-CIV-AGENTS-002 — propagation never goes backwards.
    #[test]
    fn propagation_is_monotone() {
        let params = DiffusionParams::default();
        let mut w = Wardrobe {
            era: 2,
            material: MaterialId(1),
        };
        let mut r = rng(7);
        for _ in 0..1000 {
            propagate_wardrobe(&mut w, 5, 0.5, params, &mut r);
        }
        assert!(w.era >= 2);
        assert!(w.era <= 5);
    }

    /// Covers FR-CIV-AGENTS-003 — civilians at the target era do not propagate further.
    #[test]
    fn at_target_era_is_a_noop() {
        let params = DiffusionParams::default();
        let mut w = Wardrobe {
            era: 5,
            material: MaterialId(1),
        };
        let mut r = rng(1);
        for _ in 0..100 {
            assert!(!propagate_wardrobe(&mut w, 5, 0.99, params, &mut r));
        }
        assert_eq!(w.era, 5);
    }

    /// Covers FR-CIV-AGENTS-010 — gestalt (Cold) and Hot tiers agree at shared sync ticks.
    #[test]
    fn lod_gestalt_no_divergence() {
        let params = DiffusionParams::default();
        let mut hot = Wardrobe {
            era: 1,
            material: MaterialId(1),
        };
        let mut cold = hot;
        let mut hot_rng = rng(42);
        let mut cold_rng = rng(42);

        for tick in 0..32 {
            if should_tick_now(LodTier::Hot, tick) {
                propagate_wardrobe(&mut hot, 5, 0.3, params, &mut hot_rng);
            }
            if should_tick_now(LodTier::Cold, tick) {
                propagate_wardrobe(&mut cold, 5, 0.3, params, &mut cold_rng);
            }
        }

        assert_eq!(hot, cold);
    }

    /// Covers FR-CIV-AGENTS-010 — LodTier debug / equality holds across copies.
    #[test]
    fn lod_tier_equality_round_trips() {
        for t in [LodTier::Hot, LodTier::Warm, LodTier::Cold] {
            let copy = t;
            assert_eq!(t, copy);
        }
    }

    /// Covers FR-CIV-AGENTS-011 — fixture access: civilians compose with civ-genetics
    /// and civ-species without leaking RNG into the type system.
    #[test]
    fn civilian_composition_smoke() {
        use civ_genetics::{Dna, DnaClass};
        use civ_species::{express, Phenotype};
        let class = DnaClass::default();
        let dna = Dna::zero(class.length);
        let _: Phenotype = express(&dna);
        let _civ = Civilian {
            id: 0,
            alignment: Alignment::with_faction(1),
            age: 30,
        };
    }

    /// Covers FR-CIV-AGENTS-020 — spawn_civilian inserts all requested components.
    #[test]
    fn spawn_civilian_inserts_components() {
        let mut world = World::new();
        let civ = Civilian {
            id: 11,
            alignment: Alignment::with_faction(7),
            age: 24,
        };
        let pos = Position3d {
            coord: WorldCoord { x: 0, y: 0, z: 0 },
        };
        let wardrobe = Wardrobe {
            era: 2,
            material: MaterialId(3),
        };
        let tools = Tools {
            era: 4,
            material: MaterialId(5),
        };
        let needs = Needs {
            food: 0.1,
            shelter: 0.2,
            safety: 0.3,
            belonging: 0.4,
        };
        let lod = LodTier::Warm;
        let entity = spawn_civilian(
            &mut world,
            civ.clone(),
            pos,
            CivilianBundle {
                velocity: Velocity { dx: 1.0, dy: 0.0 },
                wardrobe,
                tools,
                needs,
                lod,
            },
        );

        assert_eq!(&*world.get::<&Civilian>(entity).unwrap(), &civ);
        assert_eq!(&*world.get::<&Position3d>(entity).unwrap(), &pos);
        assert_eq!(
            &*world.get::<&Velocity>(entity).unwrap(),
            &Velocity { dx: 1.0, dy: 0.0 }
        );
        assert_eq!(&*world.get::<&Wardrobe>(entity).unwrap(), &wardrobe);
        assert_eq!(&*world.get::<&Tools>(entity).unwrap(), &tools);
        assert_eq!(&*world.get::<&Needs>(entity).unwrap(), &needs);
        assert_eq!(&*world.get::<&LodTier>(entity).unwrap(), &lod);
    }

    /// Covers FR-CIV-AGENTS-021 — spawn_many produces sequential IDs.
    #[test]
    fn spawn_many_produces_sequential_ids() {
        let mut world = World::new();
        let entities = spawn_many(&mut world, 4, 100, 9);
        assert_eq!(entities.len(), 4);
        let mut ids = Vec::new();
        for entity in entities {
            let civ = world.get::<&Civilian>(entity).unwrap();
            ids.push(civ.id);
        }
        assert_eq!(ids, vec![100, 101, 102, 103]);
    }

    /// Covers FR-CIV-AGENTS-022 — count_civilians reports the current world total.
    #[test]
    fn count_civilians_reports_correctly() {
        let mut world = World::new();
        assert_eq!(count_civilians(&world), 0);
        spawn_many(&mut world, 3, 1, 2);
        assert_eq!(count_civilians(&world), 3);
    }

    /// AGENTS-040 — tick_movement updates position.
    #[test]
    fn tick_movement_updates_position() {
        let mut world = World::new();
        let mut rng = rng(7);
        let entity = spawn_civilian(
            &mut world,
            Civilian {
                id: 1,
                alignment: Alignment::None,
                age: 20,
            },
            Position3d {
                coord: WorldCoord {
                    x: 500_000,
                    y: 0,
                    z: 500_000,
                },
            },
            CivilianBundle {
                velocity: Velocity { dx: 1.0, dy: 0.0 },
                wardrobe: Wardrobe {
                    era: 0,
                    material: MaterialId(0),
                },
                tools: Tools {
                    era: 0,
                    material: MaterialId(0),
                },
                needs: Needs {
                    food: 0.25,
                    shelter: 0.25,
                    safety: 0.25,
                    belonging: 0.25,
                },
                lod: LodTier::Hot,
            },
        );
        let before = *world.get::<&Position3d>(entity).unwrap();
        tick_movement(&mut world, 128, &mut rng, |_, _| true);
        let after = *world.get::<&Position3d>(entity).unwrap();
        assert_ne!(before, after);
    }

    /// AGENTS-041 — tick_movement respects terrain bounds.
    #[test]
    fn tick_movement_respects_bounds() {
        let mut world = World::new();
        let mut rng = rng(11);
        let entity = spawn_civilian(
            &mut world,
            Civilian {
                id: 2,
                alignment: Alignment::None,
                age: 22,
            },
            Position3d {
                coord: WorldCoord {
                    x: 990_000,
                    y: 0,
                    z: 990_000,
                },
            },
            CivilianBundle {
                velocity: Velocity { dx: 1.0, dy: 1.0 },
                wardrobe: Wardrobe {
                    era: 0,
                    material: MaterialId(0),
                },
                tools: Tools {
                    era: 0,
                    material: MaterialId(0),
                },
                needs: Needs {
                    food: 0.25,
                    shelter: 0.25,
                    safety: 0.25,
                    belonging: 0.25,
                },
                lod: LodTier::Hot,
            },
        );
        tick_movement(&mut world, 128, &mut rng, |_, _| true);
        let pos = world.get::<&Position3d>(entity).unwrap();
        let x = pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32;
        let y = pos.coord.z as f32 / civ_voxel::FIXED_SCALE as f32;
        assert!((0.01..=0.99).contains(&x));
        assert!((0.01..=0.99).contains(&y));
    }

    /// AGENTS-042 — civilians avoid water.
    #[test]
    fn tick_movement_avoids_water() {
        let mut world = World::new();
        let mut rng = rng(19);
        let entity = spawn_civilian(
            &mut world,
            Civilian {
                id: 3,
                alignment: Alignment::None,
                age: 23,
            },
            Position3d {
                coord: WorldCoord {
                    x: 500_000,
                    y: 0,
                    z: 500_000,
                },
            },
            CivilianBundle {
                velocity: Velocity { dx: 1.0, dy: 0.0 },
                wardrobe: Wardrobe {
                    era: 0,
                    material: MaterialId(0),
                },
                tools: Tools {
                    era: 0,
                    material: MaterialId(0),
                },
                needs: Needs {
                    food: 0.25,
                    shelter: 0.25,
                    safety: 0.25,
                    belonging: 0.25,
                },
                lod: LodTier::Hot,
            },
        );
        tick_movement(&mut world, 128, &mut rng, |x, _| x < 0.500_001);
        let pos = world.get::<&Position3d>(entity).unwrap();
        assert!(pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32 <= 0.500_001);
    }

    /// Covers FR-CIV-AGENTS-030 — cohort wardrobe propagation is deterministic under a
    /// fixed seed.
    #[test]
    fn propagate_cohort_wardrobe_is_deterministic() {
        let params = DiffusionParams::default();
        let mut w1 = World::new();
        let mut w2 = World::new();
        spawn_many(&mut w1, 8, 10, 1);
        spawn_many(&mut w2, 8, 10, 1);
        let mut r1 = rng(99);
        let mut r2 = rng(99);

        let s1 = propagate_cohort_wardrobe(&mut w1, 4, params, &mut r1);
        let s2 = propagate_cohort_wardrobe(&mut w2, 4, params, &mut r2);

        assert_eq!(s1, s2);
        let a: Vec<u16> = w1.query::<&Wardrobe>().iter().map(|(_, w)| w.era).collect();
        let b: Vec<u16> = w2.query::<&Wardrobe>().iter().map(|(_, w)| w.era).collect();
        assert_eq!(a, b);
    }

    /// Covers FR-CIV-AGENTS-031 — stats.total_civilians matches count_civilians.
    #[test]
    fn cohort_stats_total_matches_count_civilians() {
        let params = DiffusionParams::default();
        let mut world = World::new();
        spawn_many(&mut world, 5, 20, 3);
        let mut r = rng(7);
        let stats = propagate_cohort_wardrobe(&mut world, 3, params, &mut r);
        assert_eq!(stats.total_civilians as usize, count_civilians(&world));
    }

    /// Covers FR-CIV-AGENTS-032 — after many ticks, currently_at_target approaches total_civilians.
    #[test]
    fn cohort_wardrobe_converges_over_many_ticks() {
        let params = DiffusionParams::default();
        let mut world = World::new();
        spawn_many(&mut world, 12, 30, 4);
        let mut r = rng(1234);
        let target = 5;
        for _ in 0..2000 {
            propagate_cohort_wardrobe(&mut world, target, params, &mut r);
        }
        let stats = propagate_cohort_wardrobe(&mut world, target, params, &mut r);
        assert_eq!(stats.currently_at_target, stats.total_civilians);
        assert!((stats.current_fraction - 1.0).abs() < f32::EPSILON);
    }

    /// Covers FR-CIV-AGENTS-033 — propagate_tools mirrors propagate_wardrobe behaviour.
    #[test]
    fn propagate_cohort_tools_mirrors_wardrobe() {
        let params = DiffusionParams::default();
        let mut wardrobe_world = World::new();
        let mut tools_world = World::new();
        spawn_many(&mut wardrobe_world, 9, 40, 5);
        spawn_many(&mut tools_world, 9, 40, 5);
        let mut r1 = rng(222);
        let mut r2 = rng(222);

        let wardrobe_stats = propagate_cohort_wardrobe(&mut wardrobe_world, 4, params, &mut r1);
        let tools_stats = propagate_cohort_tools(&mut tools_world, 4, params, &mut r2);

        assert_eq!(wardrobe_stats, tools_stats);
        let wardrobe_eras: Vec<u16> = wardrobe_world
            .query::<&Wardrobe>()
            .iter()
            .map(|(_, w)| w.era)
            .collect();
        let tool_eras: Vec<u16> = tools_world
            .query::<&Tools>()
            .iter()
            .map(|(_, t)| t.era)
            .collect();
        assert_eq!(wardrobe_eras, tool_eras);
    }

    /// Covers FR-CIV-AGENTS-034 — zero civilians produces zero-everything stats without panicking.
    #[test]
    fn empty_cohort_returns_zeroed_stats() {
        let params = DiffusionParams::default();
        let mut world = World::new();
        let mut r = rng(1);
        let wardrobe_stats = propagate_cohort_wardrobe(&mut world, 2, params, &mut r);
        let tools_stats = propagate_cohort_tools(&mut world, 2, params, &mut r);
        assert_eq!(
            wardrobe_stats,
            CohortStats {
                promoted_this_tick: 0,
                currently_at_target: 0,
                total_civilians: 0,
                current_fraction: 0.0,
            }
        );
        assert_eq!(tools_stats, wardrobe_stats);
    }

    /// Covers FR-CIV-AGENTS-023 — score_needs is a deterministic weighted sum.
    #[test]
    fn score_needs_returns_deterministic_sums() {
        let needs = Needs {
            food: 0.5,
            shelter: 0.25,
            safety: 0.75,
            belonging: 0.125,
        };
        let weights = UtilityWeights {
            food: 2.0,
            shelter: 3.0,
            safety: 4.0,
            belonging: 5.0,
        };
        assert_eq!(
            score_needs(&needs, &weights),
            0.5 * 2.0 + 0.25 * 3.0 + 0.75 * 4.0 + 0.125 * 5.0
        );
    }

    /// Covers FR-CIV-AGENTS-024 — top_action selects the highest-weighted unmet need.
    #[test]
    fn top_action_picks_highest_weighted_unmet_need() {
        let needs = Needs {
            food: 0.1,
            shelter: 0.9,
            safety: 0.2,
            belonging: 0.3,
        };
        let weights = UtilityWeights {
            food: 1.0,
            shelter: 10.0,
            safety: 2.0,
            belonging: 3.0,
        };
        assert_eq!(top_action(&needs, &weights), NeedAction::FindShelter);
    }

    /// Covers utility-AI urgency as the maximum need pressure.
    #[test]
    fn needs_vector_urgency_returns_maximum_need() {
        let needs = NeedsVector {
            hunger: 0.4,
            safety: 0.9,
            social: 0.1,
            purpose: 0.7,
            spirituality: 0.6,
        };
        assert_eq!(needs.urgency(), 0.9);
    }

    /// Covers contextual utility scoring for the basic scorer.
    #[test]
    fn basic_utility_scorer_scores_contextual_actions() {
        let scorer = BasicUtilityScorer;
        let needs = NeedsVector {
            hunger: 0.8,
            safety: 0.6,
            social: 0.5,
            purpose: 0.4,
            spirituality: 0.3,
        };
        let world = WorldSnapshot {
            food_nearby: true,
            threat_nearby: true,
            others_nearby: true,
            temple_nearby: true,
        };

        assert_eq!(
            scorer.score_action(&AgentAction::Eat, &needs, &world),
            0.8 * 1.5
        );
        assert_eq!(
            scorer.score_action(&AgentAction::Flee, &needs, &world),
            0.6 * 2.0
        );
        assert_eq!(
            scorer.score_action(&AgentAction::Socialize, &needs, &world),
            0.5
        );
        assert_eq!(scorer.score_action(&AgentAction::Work, &needs, &world), 0.4);
        assert_eq!(
            scorer.score_action(&AgentAction::Pray, &needs, &world),
            0.3
        );

        let empty_world = WorldSnapshot::default();
        assert_eq!(
            scorer.score_action(&AgentAction::Eat, &needs, &empty_world),
            0.0
        );
        assert_eq!(
            scorer.score_action(&AgentAction::Flee, &needs, &empty_world),
            0.0
        );
        assert_eq!(
            scorer.score_action(&AgentAction::Socialize, &needs, &empty_world),
            0.0
        );
        assert_eq!(
            scorer.score_action(&AgentAction::Pray, &needs, &empty_world),
            0.0
        );
    }

    /// Covers greedy action selection from the highest available utility.
    #[test]
    fn choose_action_picks_highest_scoring_available_action() {
        let scorer = BasicUtilityScorer;
        let needs = NeedsVector {
            hunger: 0.9,
            safety: 0.2,
            social: 0.1,
            purpose: 0.7,
            spirituality: 0.4,
        };
        let world = WorldSnapshot {
            food_nearby: true,
            threat_nearby: false,
            others_nearby: false,
            temple_nearby: false,
        };
        let available = [AgentAction::Work, AgentAction::Eat, AgentAction::Pray];

        assert_eq!(
            choose_action(&needs, &available, &scorer, &world),
            AgentAction::Eat
        );
    }

    /// Covers deterministic tie-breaking in the greedy selector.
    #[test]
    fn choose_action_keeps_first_action_on_tie() {
        let scorer = BasicUtilityScorer;
        let needs = NeedsVector {
            hunger: 0.5,
            safety: 0.5,
            social: 0.5,
            purpose: 0.5,
            spirituality: 0.5,
        };
        let world = WorldSnapshot::default();
        let available = [AgentAction::Work, AgentAction::Eat];

        assert_eq!(
            choose_action(&needs, &available, &scorer, &world),
            AgentAction::Work
        );
    }

    /// Covers FR-CIV-EMERGENCE-010 — spawn alignment is inferred from nearby
    /// civilians (kinship proximity).
    #[test]
    fn infer_alignment_for_spawn_majority() {
        let mut world = World::new();
        let mut rng = rng(42);

        // Spawn a cluster of faction-1 civilians near (0.5, 0.5)
        for i in 0..3 {
            let angle = (i as f32) * 0.5;
            spawn_civilian_at(
                &mut world,
                100 + i as u64,
                Alignment::with_faction(1),
                0.5 + angle.cos() * 0.01,
                0.5 + angle.sin() * 0.01,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }

        // Spawn one faction-2 civilian nearby
        spawn_civilian_at(
            &mut world,
            200,
            Alignment::with_faction(2),
            0.51,
            0.51,
            ActorVisualKind::Humanoid,
            &mut rng,
        );

        // Spawn a faction-3 civilian far away
        spawn_civilian_at(
            &mut world,
            300,
            Alignment::with_faction(3),
            0.9,
            0.9,
            ActorVisualKind::Humanoid,
            &mut rng,
        );

        // Spawn point near the faction-1 cluster → should infer faction 1
        assert_eq!(
            infer_alignment_for_spawn(&world, 0.5, 0.5),
            Alignment::with_faction(1)
        );

        // Spawn point near the distant faction-3 civilian → should infer faction 3
        assert_eq!(
            infer_alignment_for_spawn(&world, 0.9, 0.9),
            Alignment::with_faction(3)
        );

        // Spawn point far from everyone → should be None
        assert_eq!(infer_alignment_for_spawn(&world, 0.1, 0.1), Alignment::None);
    }

    /// Covers FR-CIV-EMERGENCE-010 — unaligned neighbors do not force a
    /// spurious alignment; None is returned when only `Alignment::None`
    /// civilians are nearby.
    #[test]
    fn infer_alignment_for_spawn_unaligned_only() {
        let mut world = World::new();
        let mut rng = rng(42);
        for i in 0..3 {
            spawn_civilian_at(
                &mut world,
                400 + i as u64,
                Alignment::None,
                0.5,
                0.5,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }
        assert_eq!(infer_alignment_for_spawn(&world, 0.5, 0.5), Alignment::None);
    }

    /// Covers FR-CIV-EMERGENCE-001 — form_faction succeeds only when the founder
    /// has at least [`FORM_FACTION_COHESION`] positive social ties
    /// (e.g. affinity >= FRIEND_AFFINITY_THRESHOLD or kinship >= KIN_THRESHOLD).
    /// Below the threshold, alignment is left unchanged.
    #[test]
    fn form_faction_requires_cohesion_threshold() {
        let mut graph = SocialGraph::default();
        // Zero ties → no cohesion → cannot form a faction.
        assert_eq!(Alignment::None.form_faction(&graph, 1, 42), Alignment::None);

        // Add FORM_FACTION_COHESION - 1 positive ties — still under threshold.
        for i in 0..(FORM_FACTION_COHESION - 1) as u64 {
            apply_social_event(
                &mut graph,
                SocialEvent {
                    a: 42,
                    b: 100 + i,
                    kind: Interaction::Cooperated { benefit: 1.0 },
                    tick: 1,
                },
            );
        }
        assert_eq!(Alignment::None.form_faction(&graph, 1, 42), Alignment::None);

        // Push the count to the cohesion threshold — faction is formed.
        apply_social_event(
            &mut graph,
            SocialEvent {
                a: 42,
                b: 999,
                kind: Interaction::Cooperated { benefit: 1.0 },
                tick: 1,
            },
        );
        assert_eq!(
            Alignment::None.form_faction(&graph, 7, 42),
            Alignment::with_faction(7)
        );
    }

    /// Covers FR-CIV-EMERGENCE-001 — `kin`-tagged ties also count toward the
    /// cohesion threshold (kinship is a strong bond).
    #[test]
    fn form_faction_kin_ties_count_toward_cohesion() {
        let mut graph = SocialGraph::default();
        for i in 0..FORM_FACTION_COHESION as u64 {
            apply_social_event(
                &mut graph,
                SocialEvent {
                    a: 42,
                    b: 200 + i,
                    kind: Interaction::Kin,
                    tick: 1,
                },
            );
        }
        assert_eq!(
            Alignment::None.form_faction(&graph, 3, 42),
            Alignment::with_faction(3)
        );
    }

    /// Covers FR-CIV-EMERGENCE-002 — join_faction succeeds only when the joiner
    /// has at least [`JOIN_FACTION_ACCEPTANCE`] existing ties to current members
    /// of the target faction. Without those ties, the join is rejected and the
    /// alignment is left unchanged.
    #[test]
    fn join_faction_requires_acceptance_ties() {
        let mut graph = SocialGraph::default();

        // Faction membership set: {100, 101, 102, 103} (ids of current members).
        let members = [100u64, 101, 102, 103];
        // No ties yet → no acceptance → cannot join.
        assert_eq!(
            Alignment::None.join_faction(&graph, 5, 42, &members),
            Alignment::None
        );

        // Add JOIN_FACTION_ACCEPTANCE - 1 ties — still under threshold.
        for i in 0..(JOIN_FACTION_ACCEPTANCE - 1) as u64 {
            apply_social_event(
                &mut graph,
                SocialEvent {
                    a: 42,
                    b: 100 + i,
                    kind: Interaction::Cooperated { benefit: 1.0 },
                    tick: 1,
                },
            );
        }
        assert_eq!(
            Alignment::None.join_faction(&graph, 5, 42, &members),
            Alignment::None
        );

        // Add the final tie — acceptance threshold met → can join.
        apply_social_event(
            &mut graph,
            SocialEvent {
                a: 42,
                b: 100 + (JOIN_FACTION_ACCEPTANCE - 1) as u64,
                kind: Interaction::Cooperated { benefit: 1.0 },
                tick: 1,
            },
        );
        assert_eq!(
            Alignment::None.join_faction(&graph, 5, 42, &members),
            Alignment::with_faction(5)
        );
    }

    /// Covers FR-CIV-EMERGENCE-002 — a tie to a non-member does not count
    /// toward the acceptance threshold.
    #[test]
    fn join_faction_ties_to_non_members_do_not_count() {
        let mut graph = SocialGraph::default();
        let members = [100u64, 101];

        // Add many ties, but only to non-members.
        for i in 0..(JOIN_FACTION_ACCEPTANCE + 2) as u64 {
            apply_social_event(
                &mut graph,
                SocialEvent {
                    a: 42,
                    b: 900 + i,
                    kind: Interaction::Cooperated { benefit: 1.0 },
                    tick: 1,
                },
            );
        }
        assert_eq!(
            Alignment::None.join_faction(&graph, 5, 42, &members),
            Alignment::None
        );
    }

    /// Covers FR-CIV-EMERGENCE-002 — a hostile tie to a member does not count
    /// toward the acceptance threshold (negative affinity is a rival, not a
    /// social acceptance signal).
    #[test]
    fn join_faction_hostile_ties_do_not_count() {
        let mut graph = SocialGraph::default();
        let members = [100u64, 101, 102, 103, 104];

        // Defection pushes affinity deeply negative — no acceptance.
        for &m in &members {
            apply_social_event(
                &mut graph,
                SocialEvent {
                    a: 42,
                    b: m,
                    kind: Interaction::Defected { harm: 1.0 },
                    tick: 1,
                },
            );
        }
        assert_eq!(
            Alignment::None.join_faction(&graph, 5, 42, &members),
            Alignment::None
        );
    }

    /// Covers FR-CIV-AGENTS-025 — should_tick_now respects LOD modulo cadence.
    #[test]
    fn should_tick_now_respects_lod_modulo() {
        assert!(should_tick_now(LodTier::Hot, 1));
        assert!(should_tick_now(LodTier::Warm, 4));
        assert!(!should_tick_now(LodTier::Warm, 5));
        assert!(should_tick_now(LodTier::Cold, 16));
        assert!(!should_tick_now(LodTier::Cold, 17));
    }

    /// `child_bundle_from_parent` is deterministic with unit velocity and default needs.
    #[test]
    fn child_bundle_from_parent_is_deterministic_unit_velocity() {
        let mut r1 = rng(99);
        let mut r2 = rng(99);
        let b1 = child_bundle_from_parent(&mut r1);
        let b2 = child_bundle_from_parent(&mut r2);
        assert_eq!(b1.velocity, b2.velocity);
        let len_sq = b1.velocity.dx * b1.velocity.dx + b1.velocity.dy * b1.velocity.dy;
        assert!((len_sq - 1.0).abs() < 1e-5);
        assert_eq!(b1.needs.food, 0.25);
        assert_eq!(b1.needs.shelter, 0.25);
        assert_eq!(b1.needs.safety, 0.25);
        assert_eq!(b1.needs.belonging, 0.25);
        assert_eq!(b1.wardrobe.era, 0);
        assert_eq!(b1.tools.era, 0);
        assert_eq!(b1.lod, LodTier::Hot);
    }

    /// `spawn_child_near` spawns a newborn hot civilian within the parent offset band.
    #[test]
    fn spawn_child_near_spawns_newborn_within_offset_band() {
        let mut world = World::new();
        let mut child_rng = rng(42);
        let entity = spawn_child_near(
            &mut world,
            1001,
            Alignment::with_faction(1),
            0.5,
            0.5,
            &mut child_rng,
        );
        let civilian = world.get::<&Civilian>(entity).unwrap();
        let pos = world.get::<&Position3d>(entity).unwrap();
        let lod = world.get::<&LodTier>(entity).unwrap();
        assert_eq!(civilian.age, 0);
        assert_eq!(*lod, LodTier::Hot);
        let norm_x = pos.coord.x as f32 / civ_voxel::FIXED_SCALE as f32;
        let norm_z = pos.coord.z as f32 / civ_voxel::FIXED_SCALE as f32;
        assert!((norm_x - 0.5).abs() <= 0.015 + 1e-4);
        assert!((norm_z - 0.5).abs() <= 0.015 + 1e-4);
        assert!((0.01..=0.99).contains(&norm_x));
        assert!((0.01..=0.99).contains(&norm_z));
    }

    /// `drift_toward_home` steers toward home when shelter need is high.
    #[test]
    fn drift_toward_home_steers_when_shelter_need_high() {
        let scale = civ_voxel::FIXED_SCALE as f32;
        let here = Position3d {
            coord: WorldCoord { x: 0, y: 0, z: 0 },
        };
        let home = Position3d {
            coord: WorldCoord {
                x: scale as i64,
                y: 0,
                z: scale as i64,
            },
        };
        let current = Velocity {
            dx: 0.3,
            dy: 0.7,
        };
        let steered = drift_toward_home(&here, &home, current, 0.75);
        let len_sq = steered.dx * steered.dx + steered.dy * steered.dy;
        assert!((len_sq - 1.0).abs() < 1e-5);
        assert!(steered.dx > 0.0);
        assert!(steered.dy > 0.0);

        let unchanged = drift_toward_home(&here, &home, current, 0.5);
        assert_eq!(unchanged, current);

        let at_home = drift_toward_home(&here, &here, current, 0.9);
        assert_eq!(at_home, current);
    }
}
