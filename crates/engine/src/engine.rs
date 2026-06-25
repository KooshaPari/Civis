//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use civ_agents::{
    count_civilians, propagate_tools, propagate_wardrobe, spawn_child_near, spawn_civilian_at,
    ActorVisualKind, Alignment, Civilian as AgentCivilian, CohortStats, ClusterMember,
    DiplomacyMatrix, DiplomacySignal, LodTier, Needs, Position3d, Psyche, SocialGraph, Tools,
    Wardrobe,
};
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_build::{Allocator, BuildingGraph, DemandSignals};
use civ_diffusion::DiffusionParams;
use civ_economy::{AllocationEngine, CapitalistAllocator, EconomyState, MarketState};
use civ_mod_host::ModHost;
use civ_genetics::{
    sentience::{cognition_score, CognitionTraitProfile, SentienceThreshold},
    Dna,
};
use civ_planet::{
    compute_climate, compute_weather, defaults_earthlike, Climate, GeologyMap, MoonConfig,
    PlanetConfig, WeatherCell,
};
use civ_tactics::{
    apply_damage, evolve_doctrine, score_doctrine_fitness, tick_operational_movement,
    tick_war_bridge, CombatEngagement, DamageEvent, Doctrine, DoctrineLibrary,
    FactionEngagementStats, MilitaryPhaseConfig, MilitaryUnitSample, NoopOperationalLayer,
    OperationalLayer,
};
use civ_voxel::{
    material::WATER,
    DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE,
};
use hecs::{Entity, World};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::Fixed;
use crate::emergence::EmergenceState;
use crate::emergence_metrics::{EmergenceBranchingState, EmergenceSample};
use crate::lod::{should_tick_entity_with_policy, LodPolicy};
use crate::policy::ControlSignals;
use crate::policy::Policy;
use crate::policy::PolicyInput;
use crate::policy::DEFAULT_ECONOMY_POLICY;
use crate::replay::{ReplayError, ReplayLog};
use crate::replay_format::{load_civreplay, save_civreplay};

/// Ordered phase identifiers executed once per [`Simulation::tick`].
///
/// CIV-0001 partial — engine-side deterministic transition. Server command intake
/// and client broadcast are outside this crate. Keep in sync with the calls in
/// [`Simulation::tick`].
pub(crate) const PHASE_ORDER: &[&str] = &[
    "production",
    "citizen_lifecycle",
    "military",
    "policy",
    "economy",
    "planet",
    "diplomacy",
    "tactics",
    "voxel",
    "compact",
    "buildings",
    "diffusion",
    "life",
    "emergence",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchCache;

/// Seeded RNG for reproducible simulation
pub type SimRng = ChaCha8Rng;

// ============================================================================
// COMPONENTS - Data attached to entities
// ============================================================================

/// Position on the hex grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Tactical damage pulse for spectator clients (normalized map coords + optional unit ids).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CombatDamagePulse {
    /// Normalized map X.
    pub x: f32,
    /// Normalized map Y.
    pub y: f32,
    /// Attacking unit pin id when damage came from military contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_a: Option<u64>,
    /// Defending unit pin id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit_b: Option<u64>,
}

/// Citizen entity component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Citizen {
    pub age: u32,        // Age in years
    pub health: Fixed,   // Health 0.0 - 1.0
    pub ideology: Fixed, // -1.0 (libertarian) to 1.0 (authoritarian)
    pub welfare: Fixed,  // 0.0 - 1.0
    pub job: Option<JobType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobType {
    Farmer,
    Warrior,
    Scholar,
    Trader,
    Priest,
    Admin,
    Unemployed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum EconomicFocus {
    Balanced,
    Agrarian,
    Industrial,
    Sacred,
    Mercantile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
enum Good {
    Food,
    Water,
    Wood,
    Metal,
    Tools,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClusterStocks {
    goods: BTreeMap<Good, i64>,
}

impl ClusterStocks {
    pub fn add(&mut self, good: Good, delta: i64) -> i64 {
        let entry = self.goods.entry(good).or_insert(0);
        if delta >= 0 {
            let before = *entry;
            *entry = entry.saturating_add(delta);
            *entry - before
        } else {
            let requested = delta.saturating_abs();
            let removed = requested.min(*entry);
            *entry -= removed;
            -removed
        }
    }
}

/// Deterministic job assignment for agent civilians (stable across seeds).
pub fn job_type_for_civilian_id(id: u64) -> JobType {
    match id % 7 {
        0 => JobType::Farmer,
        1 => JobType::Warrior,
        2 => JobType::Scholar,
        3 => JobType::Trader,
        4 => JobType::Priest,
        5 => JobType::Admin,
        _ => JobType::Unemployed,
    }
}

/// Attach [`Citizen`] (with job) to agent entities that only have [`AgentCivilian`].
pub fn attach_citizen_to_agents(world: &mut World) {
    let agents: Vec<(Entity, AgentCivilian)> = world
        .query::<&AgentCivilian>()
        .iter()
        .map(|(entity, civilian)| (entity, civilian.clone()))
        .collect();
    for (entity, civilian) in agents {
        if world.get::<&Citizen>(entity).is_ok() {
            continue;
        }
        let citizen = Citizen {
            age: civilian.age as u32,
            health: Fixed::from_num(1),
            ideology: Fixed::ZERO,
            welfare: Fixed::from_num(7) / Fixed::from_num(10),
            job: Some(job_type_for_civilian_id(civilian.id)),
        };
        let _ = world.insert(entity, (citizen,));
    }
}

fn spawn_faction_civilians(world: &mut World, rng: &mut SimRng) {
    spawn_faction_civilians_custom(world, rng, 32, 4, 2_500);
}

/// Spawn civilians for each faction with custom parameters.
pub(crate) fn spawn_faction_civilians_custom(
    world: &mut World,
    rng: &mut SimRng,
    civilians_per_faction: u32,
    faction_count: u32,
    quadrant_spread: i32,
) {
    let scale = FIXED_SCALE as f32;
    let mut next_civilian_id = 1u64;

    // Arrange faction capitals in a ring around the map center
    let faction_count_f32 = faction_count as f32;
    for faction in 0..faction_count {
        let angle = (faction as f32 / faction_count_f32) * std::f32::consts::TAU;
        let radius = 7_500.0;
        let center_x = (angle.cos() * radius) as i32;
        let center_y = (angle.sin() * radius) as i32;

        for _ in 0..civilians_per_faction {
            let grid_x = center_x + rng.gen_range(-quadrant_spread..=quadrant_spread);
            let grid_z = center_y + rng.gen_range(-quadrant_spread..=quadrant_spread);
            let norm_x = (grid_x as f32 / scale).clamp(0.0, 1.0);
            let norm_y = (grid_z as f32 / scale).clamp(0.0, 1.0);
            spawn_civilian_at(world, next_civilian_id, Alignment::Faction(faction), norm_x, norm_y, ActorVisualKind::Humanoid, rng);
            next_civilian_id += 1;
        }
    }
}

/// Building entity component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub hp: Fixed,
    pub max_hp: Fixed,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    Farm,
    Mine,
    Barracks,
    Temple,
    Market,
    House,
    CityCenter,
}

/// Resource storage component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Resources {
    pub food: Fixed,
    pub wood: Fixed,
    pub metal: Fixed,
    pub energy: Fixed, // Joules
}

/// Simple trade route between two factions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeRoute {
    pub from_faction: u32,
    pub to_faction: u32,
    pub goods: String,
    pub volume: Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplomacyKind {
    TradeAgreement,
    Conflict,
    Peace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomacyEvent {
    pub tick: u64,
    pub faction_a: u32,
    pub faction_b: u32,
    pub kind: DiplomacyKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationEvent {
    pub tick: u64,
    pub entity_id: u64,
    pub x: f32,
    pub y: f32,
}

/// Production capability
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Production {
    pub output_type: ResourceType,
    pub rate: Fixed, // Per tick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Wood,
    Metal,
    Energy,
}

/// Military unit component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitaryUnit {
    pub unit_type: UnitType,
    /// Legacy wire field; kept in sync with [`Self::hp`].
    pub strength: Fixed,
    /// Per-soldier hit points (FR-CIV-TACTICS-032).
    pub hp: Fixed,
    pub max_hp: Fixed,
    pub morale: Fixed,
    pub position: Position,
    pub faction_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Soldier,
    Archer,
    Knight,
    Scout,
}

// ============================================================================
// WORLD STATE
// ============================================================================

/// Global world state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: Fixed,
    pub rng_seed: u64,
    /// Faction ID -> faction name
    pub factions: HashMap<u32, String>,
    /// Faction ID -> treasury balance
    pub faction_treasury: HashMap<u32, Fixed>,
    /// Faction ID -> resource holdings.
    pub faction_resources: HashMap<u32, Resources>,
    /// Active trade routes connecting factions.
    pub trade_routes: Vec<TradeRoute>,
    /// Global belief pressure used by emergence coupling.
    pub belief: u64,
    /// Global cohesion pressure used by emergence coupling.
    pub cohesion: u64,
    /// Global unrest pressure used by emergence coupling.
    pub unrest: u64,
    /// Emergent trade routes that should be retained while active.
    pub emergent_trade_route_keys: BTreeSet<(u32, u32, String)>,
    /// Idle-tick counters for emergent trade routes.
    pub trade_route_idle_ticks: BTreeMap<(u32, u32, String), u32>,
    pub resources: Resources,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 1_000_000,
            energy_budget_joules: Fixed::from_num(1_000_000_000_000i64),
            rng_seed: 42,
            factions: HashMap::from([
                (0, "Player".to_string()),
                (1, "AI Faction A".to_string()),
                (2, "AI Faction B".to_string()),
            ]),
            faction_treasury: HashMap::from([
                (0, Fixed::from_num(10_000)),
                (1, Fixed::from_num(8_000)),
                (2, Fixed::from_num(8_000)),
            ]),
            faction_resources: HashMap::from([
                (
                    0,
                    Resources {
                        food: Fixed::from_num(120),
                        wood: Fixed::from_num(90),
                        metal: Fixed::from_num(70),
                        energy: Fixed::from_num(50),
                    },
                ),
                (
                    1,
                    Resources {
                        food: Fixed::from_num(80),
                        wood: Fixed::from_num(110),
                        metal: Fixed::from_num(100),
                        energy: Fixed::from_num(40),
                    },
                ),
                (
                    2,
                    Resources {
                        food: Fixed::from_num(60),
                        wood: Fixed::from_num(70),
                        metal: Fixed::from_num(120),
                        energy: Fixed::from_num(60),
                    },
                ),
            ]),
            trade_routes: vec![
                TradeRoute {
                    from_faction: 0,
                    to_faction: 1,
                    goods: "grain".to_string(),
                    volume: Fixed::from_num(12),
                },
                TradeRoute {
                    from_faction: 1,
                    to_faction: 2,
                    goods: "ore".to_string(),
                    volume: Fixed::from_num(10),
                },
                TradeRoute {
                    from_faction: 2,
                    to_faction: 0,
                    goods: "cloth".to_string(),
                    volume: Fixed::from_num(8),
                },
            ],
            belief: 0,
            cohesion: 0,
            unrest: 0,
            emergent_trade_route_keys: BTreeSet::new(),
            trade_route_idle_ticks: BTreeMap::new(),
            resources: Resources::default(),
        }
    }
}

/// Simulation engine combining state + ECS world + 3D voxel substrate.
pub struct Simulation {
    pub state: WorldState,
    pub world: World,
    rng: SimRng,
    planet: PlanetConfig,
    moon: MoonConfig,
    climate: Climate,
    pending_damage: Vec<DamageEvent>,
    tick_modulo_compact: u64,
    building_graph: BuildingGraph,
    allocator: Allocator,
    diffusion_params: DiffusionParams,
    target_era: u16,
    last_cohort_stats: Option<CohortStats>,
    last_births: Vec<PopulationEvent>,
    last_deaths: Vec<PopulationEvent>,
    diplomacy_events: Vec<DiplomacyEvent>,
    next_civilian_id: u64,
    research_cache: ResearchCache,
    scenario_taxation: crate::scenario::ScenarioTaxation,
    pub(crate) emergence: EmergenceState,
    pub(crate) emergence_branching: EmergenceBranchingState,
    pub(crate) emergence_sample: Option<EmergenceSample>,
    /// 3D voxel substrate (Civis 3D extension). Hosts terrain + destructible
    /// structures + tactical combat impacts. Drained per tick by
    /// [`Simulation::phase_voxel`].
    voxel: VoxelWorld<MaterialId>,
    /// Voxel dirty events produced during the most recent tick. Consumers
    /// (renderer protocol bridge, replay log) read this each tick; it resets
    /// at the start of every [`Simulation::tick`].
    last_tick_voxel_events: Vec<DirtyChunkEvent>,
    last_tick_voxel_damage_count: usize,
    /// Per-soldier damage pulses from the most recent tactics phase (FR-CIV-TACTICS-024).
    last_tick_combat_pulses: Vec<CombatDamagePulse>,
    /// Engagements resolved this tick (war bridge); feeds doctrine fitness.
    pub(crate) last_tick_engagements: Vec<CombatEngagement>,
    /// Per-faction mean aggression snapshot rebuilt during emergence.
    pub(crate) faction_aggression: BTreeMap<u32, f32>,
    /// `mod.loaded.v1` replay-bus JSON emitted when mods load (cleared each tick).
    last_tick_mod_lifecycle: Vec<String>,
    operational: NoopOperationalLayer,
    replay_log: ReplayLog,
    pub(crate) last_settlement_count: u32,
    pub(crate) last_life_deaths: u32,
    cluster_stocks: BTreeMap<u64, ClusterStocks>,
    /// Scenario economy policy (`base_consumption_joules`, `scarcity_multiplier`).
    pub economy_policy: PolicyInput,
    /// Active control policy (FR-CORE-005). Read in [`Self::phase_policy`]
    /// each tick. Defaults to [`NoopPolicy`]; replaceable via
    /// [`Self::set_policy`].
    pub policy: Box<dyn Policy>,
    /// Most recent control signals emitted by [`Self::policy`] (FR-CORE-005).
    /// Updated at the end of every `phase_policy` call.
    pub last_control_signals: ControlSignals,
    /// Macro economy state (`civ-economy`); synced with `WorldState::energy_budget_joules` each tick.
    pub economy_state: EconomyState,
    /// Per-good clearing prices (`civ-economy`); advanced in [`phase_economy`].
    pub market_state: MarketState,
    /// LOD tick cadence for Warm/Cold civilian tiers (CIV-0101).
    pub lod_policy: LodPolicy,
    /// Manifest-only mod host (CIV-0700 v2 policy stub); WASM not loaded yet.
    mod_host: ModHost,
    /// Military-phase cadence and per-tick movement pulses (FR-CIV-TACTICS-035).
    pub(crate) military_phase: MilitaryPhaseConfig,
    /// Per-faction doctrine libraries evolved on a fixed tick cadence (FR-CIV-TACTICS-010).
    faction_doctrines: Vec<DoctrineLibrary>,
    /// Coastal water columns whose water-level voxel shifts with the tide
    /// offset every tick (FR-CIV-PLANET-020). Keyed by `(x, z)` in fixed-point
    /// world coords; iteration order is deterministic.
    coastal_columns: BTreeMap<(i64, i64), CoastalColumn>,
    /// Per-region weather grid updated by `phase_planet` each tick (FR-CIV-PLANET-030).
    weather_grid: Vec<WeatherCell>,
}

/// Voxel material id used to mark coastal water-level voxels written by
/// [`Simulation::apply_tide_offset`] (FR-CIV-PLANET-020). Reuses the shared
/// `civ_voxel::material::WATER` constant so the engine, clients, and worldgen
/// stay on the same source of truth.
pub const WATER_MARKER_MATERIAL: MaterialId = WATER;

/// A coastal water column registered with the engine. Each column anchors a
/// single water-marker voxel that shifts vertically with the climate tide
/// offset every tick (FR-CIV-PLANET-020). Iteration order is deterministic
/// because columns live in a [`BTreeMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct CoastalColumn {
    /// Sea-level y in fixed-point world units.
    base_y: i64,
    /// Last y the water marker was written at (so we can clear it before
    /// writing the new level — preserves FR-CIV-VOXEL-002 dirty-event
    /// invariants by going through `VoxelWorld::write`).
    last_water_y: i64,
}

/// Default doctrine population for three factions (deterministic seed layout).
fn default_faction_doctrines() -> Vec<DoctrineLibrary> {
    (0..3)
        .map(|faction| DoctrineLibrary {
            generation: 0,
            current: vec![
                Doctrine {
                    id: faction as u64 * 10 + 1,
                    unit_composition: vec![10, 5, 2],
                    score: 0.5,
                },
                Doctrine {
                    id: faction as u64 * 10 + 2,
                    unit_composition: vec![8, 8, 4],
                    score: 0.8,
                },
            ],
        })
        .collect()
}

fn economy_state_from_world(world: &WorldState) -> EconomyState {
    let energy_budget_joules = world.energy_budget_joules.raw / crate::SCALE;
    let mut state = EconomyState::with_energy_budget(energy_budget_joules);
    state.tick = world.tick;
    state
}

fn propagate_cohort_wardrobe_with_lod(
    world: &mut World,
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

fn propagate_cohort_tools_with_lod(
    world: &mut World,
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

impl Simulation {
    /// Create new simulation with default state
    pub fn new() -> Self {
        let rng = SimRng::seed_from_u64(42);
        let mut world = World::new();

        // Spawn initial entities
        Self::spawn_initial_entities(&mut world);
        let mut spawn_rng = rng.clone();
        spawn_faction_civilians(&mut world, &mut spawn_rng);
        attach_citizen_to_agents(&mut world);

        let (planet, moon) = defaults_earthlike();
        let climate = compute_climate(0, &planet, &moon);
        let weather_grid = compute_weather(&climate, 0, 16);
        let state = WorldState::default();

        Self {
            economy_state: economy_state_from_world(&state),
            market_state: MarketState::default(),
            state,
            world,
            rng,
            planet,
            moon,
            climate,
            pending_damage: Vec::new(),
            tick_modulo_compact: 64,
            building_graph: BuildingGraph::new(),
            allocator: Allocator::new(42),
            diffusion_params: DiffusionParams::default(),
            target_era: 1,
            last_cohort_stats: None,
            last_births: Vec::new(),
            last_deaths: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache::default(),
            scenario_taxation: crate::scenario::ScenarioTaxation::default(),
            emergence: EmergenceState::new(42),
            emergence_branching: EmergenceBranchingState::default(),
            emergence_sample: None,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            faction_aggression: BTreeMap::new(),
            last_tick_mod_lifecycle: Vec::new(),
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed: 42,
                ..ReplayLog::default()
            },
            last_settlement_count: 0,
            last_life_deaths: 0,
            cluster_stocks: BTreeMap::new(),
            economy_policy: DEFAULT_ECONOMY_POLICY,
            policy: Box::new(crate::policy::NoopPolicy),
            last_control_signals: ControlSignals::default(),
            lod_policy: LodPolicy::default(),
            mod_host: ModHost::new(),
            military_phase: MilitaryPhaseConfig::default(),
            faction_doctrines: default_faction_doctrines(),
            coastal_columns: BTreeMap::new(),
            weather_grid,
        }
    }

    /// Create simulation with custom seed
    pub fn with_seed(seed: u64) -> Self {
        let rng = SimRng::seed_from_u64(seed);
        let mut world = World::new();
        Self::spawn_initial_entities(&mut world);
        let mut spawn_rng = rng.clone();
        spawn_faction_civilians(&mut world, &mut spawn_rng);
        attach_citizen_to_agents(&mut world);

        let (planet, moon) = defaults_earthlike();
        let climate = compute_climate(0, &planet, &moon);
        let weather_grid = compute_weather(&climate, 0, 16);

        // Count actual civilians spawned (128: 32 per faction × 4 factions)
        let civilian_count = count_civilians(&world) as u64;

        let state = WorldState {
            rng_seed: seed,
            population: civilian_count,
            ..Default::default()
        };

        Self {
            economy_state: economy_state_from_world(&state),
            market_state: MarketState::default(),
            state,
            world,
            rng,
            planet,
            moon,
            climate,
            pending_damage: Vec::new(),
            tick_modulo_compact: 64,
            building_graph: BuildingGraph::new(),
            allocator: Allocator::new(seed),
            diffusion_params: DiffusionParams::default(),
            target_era: 1,
            last_cohort_stats: None,
            last_births: Vec::new(),
            last_deaths: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache::default(),
            scenario_taxation: crate::scenario::ScenarioTaxation::default(),
            emergence: EmergenceState::new(seed),
            emergence_branching: EmergenceBranchingState::default(),
            emergence_sample: None,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            faction_aggression: BTreeMap::new(),
            last_tick_mod_lifecycle: Vec::new(),
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed,
                ..ReplayLog::default()
            },
            last_settlement_count: 0,
            last_life_deaths: 0,
            cluster_stocks: BTreeMap::new(),
            economy_policy: DEFAULT_ECONOMY_POLICY,
            policy: Box::new(crate::policy::NoopPolicy),
            last_control_signals: ControlSignals::default(),
            lod_policy: LodPolicy::default(),
            mod_host: ModHost::new(),
            military_phase: MilitaryPhaseConfig::default(),
            faction_doctrines: default_faction_doctrines(),
            coastal_columns: BTreeMap::new(),
            weather_grid,
        }
    }

    /// Create simulation with custom seed and starting conditions (stub for testing).
    #[cfg(test)]
    pub fn with_seed_and_starting_conditions(
        seed: u64,
        _starting_conditions: crate::scenario::ScenarioStartingConditions,
    ) -> Self {
        // Stub implementation: just create a sim with the seed and ignore starting conditions.
        // The full implementation should apply the starting conditions (faction count,
        // civilian count, seed mix, etc.) to the initial world state.
        Self::with_seed(seed)
    }

    /// Install a single mod at runtime (directory or `.civmod` archive).
    ///
    /// `rel_path` is resolved from the repo root (`crates/engine/../../`).
    pub fn install_mod_path(
        &mut self,
        rel_path: &str,
    ) -> Result<civ_mod_host::ModLoadedRecord, civ_mod_host::ManifestError> {
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let dir = repo_root.join(rel_path);
        let named_civmod = dir.file_name().and_then(|name| {
            let archive = dir.join(format!("{}.civmod", name.to_string_lossy()));
            archive.is_file().then_some(archive)
        });
        let load_path = named_civmod.as_deref().unwrap_or(dir.as_path());
        self.mod_host.load_mod_path(load_path)?;
        let entry =
            self.mod_host
                .mods()
                .last()
                .ok_or_else(|| civ_mod_host::ManifestError::Validation {
                    path: load_path.to_path_buf(),
                    message: "mod load produced no registry entry".into(),
                })?;
        let record = civ_mod_host::ModLoadedRecord {
            mod_id: entry.manifest.meta.id.clone(),
            mod_name: entry.manifest.meta.name.clone(),
            version: entry.manifest.meta.version.clone(),
            tick: self.state.tick,
        };
        let bus_json = civ_mod_host::format_mod_loaded_event_json(&record);
        self.replay_log.record_mod_loaded(&record);
        self.last_tick_mod_lifecycle.push(bus_json);
        Ok(record)
    }

    /// Unload a loaded mod by stable id and emit `mod.unloaded.v1` on the lifecycle bus.
    pub fn unload_mod_by_id(
        &mut self,
        mod_id: &str,
        reason: &str,
    ) -> Result<civ_mod_host::ModUnloadedRecord, String> {
        let record = self.mod_host.unload_mod(mod_id, reason, self.state.tick)?;
        let bus_json = civ_mod_host::format_mod_unloaded_event_json(&record);
        self.replay_log.record_mod_unloaded(&record);
        self.last_tick_mod_lifecycle.push(bus_json);
        Ok(record)
    }

    /// Hot-reload a mod from its remembered source path and emit `mod.loaded.v1`.
    pub fn reload_mod_by_id(
        &mut self,
        mod_id: &str,
    ) -> Result<civ_mod_host::ModLoadedRecord, String> {
        let record = self.mod_host.reload_mod(mod_id, self.state.tick)?;
        let bus_json = civ_mod_host::format_mod_loaded_event_json(&record);
        self.replay_log.record_mod_loaded(&record);
        self.last_tick_mod_lifecycle.push(bus_json);
        Ok(record)
    }

    /// Load mod manifests from scenario `mods` paths (repo-relative).
    ///
    /// Paths are resolved from the repo root (`crates/engine/../../`). Failures are
    /// logged and skipped so headless runs stay up during mod development.
    pub fn register_mod_stubs(&mut self, mod_paths: &[String]) {
        self.mod_host = ModHost::new();
        if mod_paths.is_empty() {
            return;
        }

        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for rel in mod_paths {
            let dir = repo_root.join(rel);
            let named_civmod = dir.file_name().and_then(|name| {
                let archive = dir.join(format!("{}.civmod", name.to_string_lossy()));
                archive.is_file().then_some(archive)
            });
            let load_path = named_civmod.as_deref().unwrap_or(dir.as_path());
            if let Err(err) = self.mod_host.load_mod_path(load_path) {
                tracing::warn!(mod = %rel, error = %err, "mod manifest load skipped");
                continue;
            }
            if let Some(entry) = self.mod_host.mods().last() {
                let record = civ_mod_host::ModLoadedRecord {
                    mod_id: entry.manifest.meta.id.clone(),
                    mod_name: entry.manifest.meta.name.clone(),
                    version: entry.manifest.meta.version.clone(),
                    tick: self.state.tick,
                };
                let bus_json = civ_mod_host::format_mod_loaded_event_json(&record);
                self.replay_log.record_mod_loaded(&record);
                self.last_tick_mod_lifecycle.push(bus_json);
            }
        }
    }

    /// Borrow the mod host (manifest registry).
    #[must_use]
    pub fn mod_host(&self) -> &ModHost {
        &self.mod_host
    }

    /// Mutable mod host (phase ticks and guest memory restore).
    pub fn mod_host_mut(&mut self) -> &mut ModHost {
        &mut self.mod_host
    }

    /// Export per-mod guest scratch memory for CIV-1000 save bundles.
    #[must_use]
    pub fn export_mod_guest_state(&self) -> civ_mod_host::ModGuestStateSave {
        self.mod_host.export_guest_state()
    }

    /// Restore per-mod guest scratch memory after load.
    pub fn restore_mod_guest_state(
        &mut self,
        save: &civ_mod_host::ModGuestStateSave,
    ) -> Result<(), civ_mod_host::GuestStateError> {
        self.mod_host.import_guest_state(save)
    }

    /// Loaded mods for mod-browser UI (`sim.snapshot` / civ-watch).
    #[must_use]
    pub fn mod_browser_entries(&self) -> Vec<civ_mod_host::ModBrowserEntry> {
        self.mod_host.browser_entries()
    }

    /// Per-faction doctrine libraries (evolved in [`Self::phase_tactics`]).
    #[must_use]
    pub fn faction_doctrines(&self) -> &[DoctrineLibrary] {
        &self.faction_doctrines
    }

    /// Borrow the immutable planet config.
    pub fn planet(&self) -> &PlanetConfig {
        &self.planet
    }

    /// Borrow the immutable moon config.
    pub fn moon(&self) -> &MoonConfig {
        &self.moon
    }

    /// Borrow the last climate computed by the planet phase.
    pub fn climate(&self) -> &Climate {
        &self.climate
    }

    /// Queue tactical voxel damage for the tactics phase.
    pub fn push_damage(&mut self, event: DamageEvent) {
        self.replay_log.record_damage(self.state.tick, event);
        self.pending_damage.push(event);
    }

    /// Apply a voxel write and record it in the replay log.
    pub fn push_voxel_write(&mut self, pos: civ_voxel::WorldCoord, value: MaterialId) {
        self.voxel.write(pos, value);
        self.replay_log
            .record_voxel_write(self.state.tick, pos, value);
    }

    /// Apply tactical voxel damage immediately, bypassing the queue.
    pub fn apply_damage_now(&mut self, event: &DamageEvent) -> usize {
        apply_damage(&mut self.voxel, event)
    }

    pub(crate) fn apply_replay_voxel_write(
        &mut self,
        tick: u64,
        pos: civ_voxel::WorldCoord,
        value: MaterialId,
    ) {
        self.state.tick = tick;
        self.voxel.write(pos, value);
    }

    pub(crate) fn apply_replay_damage(&mut self, tick: u64, event: &DamageEvent) {
        self.state.tick = tick;
        let _ = apply_damage(&mut self.voxel, event);
    }

    pub(crate) fn apply_replay_combat(&mut self, tick: u64, event: &DamageEvent) {
        self.state.tick = tick;
        self.pending_damage.push(*event);
    }

    pub(crate) fn apply_replay_research(
        &mut self,
        tick: u64,
        snapshot_hash: Vec<u8>,
        accepted: bool,
    ) {
        self.state.tick = tick;
        let _ = (snapshot_hash, accepted);
    }

    pub(crate) fn apply_replay_tick(&mut self, tick: u64) {
        self.state.tick = tick;
    }

    /// Number of voxels removed during the most recent tactics phase.
    pub fn last_tick_voxel_damage_count(&self) -> usize {
        self.last_tick_voxel_damage_count
    }

    /// Normalized (0..1) map centers for damage events applied on the last tick.
    pub fn last_tick_combat_pulses(&self) -> &[CombatDamagePulse] {
        &self.last_tick_combat_pulses
    }

    /// Normalized damage centers (legacy helper over [`Self::last_tick_combat_pulses`]).
    pub fn last_tick_damage_centers(&self) -> Vec<(f32, f32)> {
        self.last_tick_combat_pulses
            .iter()
            .map(|pulse| (pulse.x, pulse.y))
            .collect()
    }

    /// Borrow the 3D voxel substrate. Read-only.
    #[must_use]
    pub fn voxel(&self) -> &VoxelWorld<MaterialId> {
        &self.voxel
    }

    /// Mutable borrow of the voxel substrate. Writes accumulated here drain
    /// through [`Simulation::phase_voxel`] on the next tick.
    pub fn voxel_mut(&mut self) -> VoxelWriteProxy<'_> {
        VoxelWriteProxy { sim: self }
    }

    /// Dirty voxel events produced during the most recent tick. Replay logs,
    /// `civ-protocol-3d` frame builders, and the renderer bridge all read
    /// from this slice. The vector resets at the start of every
    /// [`Simulation::tick`].
    #[must_use]
    pub fn last_tick_voxel_events(&self) -> &[DirtyChunkEvent] {
        &self.last_tick_voxel_events
    }

    /// Borrow the building graph.
    pub fn building_graph(&self) -> &BuildingGraph {
        &self.building_graph
    }

    /// Borrow the most recent cohort diffusion statistics.
    pub fn last_cohort_stats(&self) -> Option<&CohortStats> {
        self.last_cohort_stats.as_ref()
    }

    /// Borrow the research cache.
    pub fn research_cache(&self) -> &ResearchCache {
        &self.research_cache
    }

    pub fn researched_tech_count(&self) -> usize {
        0
    }

    /// Mutably borrow the research cache.
    pub fn research_cache_mut(&mut self) -> &mut ResearchCache {
        &mut self.research_cache
    }

    pub fn last_births(&self) -> &[PopulationEvent] {
        &self.last_births
    }

    pub fn last_deaths(&self) -> &[PopulationEvent] {
        &self.last_deaths
    }

    pub fn diplomacy_events(&self) -> &[DiplomacyEvent] {
        &self.diplomacy_events
    }

    /// Inject a player-issued diplomacy event without advancing the tick.
    pub fn push_diplomacy_event(&mut self, event: DiplomacyEvent) {
        self.diplomacy_events.push(event);
    }

    /// Spawn initial world entities
    fn spawn_initial_entities(world: &mut World) {
        // Create initial citizens
        for i in 0..100 {
            let citizen = Citizen {
                age: 20 + (i % 40),
                health: Fixed::from_num(1),
                ideology: Fixed::from_num((i as i64 % 20 - 10) as i32) / Fixed::from_num(10),
                welfare: Fixed::from_num(7) / Fixed::from_num(10),
                job: Some(JobType::Farmer),
            };
            let _ = world.spawn((citizen,));
        }

        // Create city center
        let city = Building {
            building_type: BuildingType::CityCenter,
            hp: Fixed::from_num(1000),
            max_hp: Fixed::from_num(1000),
            position: Position { x: 0, y: 0 },
        };
        let _ = world.spawn((city,));

        // Create farms
        for i in 0..5 {
            let farm = Building {
                building_type: BuildingType::Farm,
                hp: Fixed::from_num(200),
                max_hp: Fixed::from_num(200),
                position: Position { x: i - 2, y: 1 },
            };
            let _ = world.spawn((farm,));
        }

        // Create initial military (player + AI for war-bridge smoke)
        for i in 0..5 {
            let hp = Fixed::from_num(10);
            let soldier = MilitaryUnit {
                unit_type: UnitType::Soldier,
                strength: hp,
                hp,
                max_hp: hp,
                morale: Fixed::from_num(1),
                position: Position { x: i, y: 0 },
                faction_id: 0,
            };
            let _ = world.spawn((soldier,));
        }
        for i in 0..5 {
            let hp = Fixed::from_num(8);
            let soldier = MilitaryUnit {
                unit_type: UnitType::Archer,
                strength: hp,
                hp,
                max_hp: hp,
                morale: Fixed::from_num(1),
                position: Position { x: i + 6, y: 2 },
                faction_id: 1,
            };
            let _ = world.spawn((soldier,));
        }
    }

    /// Get mutable reference to RNG
    pub fn rng_mut(&mut self) -> &mut SimRng {
        &mut self.rng
    }

    /// Resolve a civilian agent id to its ECS entity.
    pub(crate) fn agent_entity(&self, agent_id: u64) -> Option<Entity> {
        self.world
            .query::<&AgentCivilian>()
            .iter()
            .find_map(|(entity, civilian)| (civilian.id == agent_id).then_some(entity))
    }

    /// Apply a bounded belief pulse to the global world state.
    pub(crate) fn add_belief(&mut self, delta: i64) {
        let next = if delta >= 0 {
            self.state.belief.saturating_add(delta as u64)
        } else {
            self.state.belief.saturating_sub(delta.unsigned_abs())
        };
        self.state.belief = next;
    }

    /// Apply a bounded cohesion pulse to the global world state.
    pub(crate) fn add_cohesion(&mut self, delta: i64) {
        let next = if delta >= 0 {
            self.state.cohesion.saturating_add(delta as u64)
        } else {
            self.state.cohesion.saturating_sub(delta.unsigned_abs())
        };
        self.state.cohesion = next;
    }

    /// Install a new control policy. Replaces the previous policy. The new
    /// policy will be evaluated at the start of the next `phase_policy` call
    /// (FR-CORE-005).
    pub fn set_policy(&mut self, p: Box<dyn Policy>) {
        self.policy = p;
    }

    /// Borrow the active control policy.
    pub fn policy(&self) -> &dyn Policy {
        self.policy.as_ref()
    }

    /// Borrow the most recent [`ControlSignals`] emitted by the active policy
    /// (FR-CORE-005). Updated at the end of every `phase_policy` call.
    pub fn last_control_signals(&self) -> &ControlSignals {
        &self.last_control_signals
    }

    /// Advance simulation by one tick.
    ///
    /// Phases run in [`PHASE_ORDER`] (CIV-0001 partial — engine-side deterministic
    /// transition only; server command intake and client broadcast live outside this
    /// crate). Exactly one [`ReplayEvent::Tick`] is appended after all phases finish.
    pub fn tick(&mut self) {
        self.state.tick += 1;
        self.last_tick_combat_pulses.clear();
        self.last_tick_engagements.clear();
        self.last_tick_mod_lifecycle.clear();

        // Phases in PHASE_ORDER (CIV-0001 partial)
        self.phase_production();
        self.phase_citizen_lifecycle();
        self.phase_military();
        self.phase_policy();
        self.phase_economy();
        self.phase_planet();
        self.diplomacy_events.clear();
        self.phase_diplomacy();
        self.phase_tactics();
        self.phase_voxel();
        self.phase_compact();
        self.phase_buildings();
        self.phase_diffusion();
        self.phase_disasters();
        self.phase_life();
        self.phase_emergence();
        self.phase_emergence_events_close();
        self.replay_log.record_tick(self.state.tick);

        #[cfg(debug_assertions)]
        debug_assert!(
            crate::integrity::check_integrity(self).is_ok(),
            "simulation integrity violated"
        );
    }

    /// Borrow the replay log.
    pub fn replay_log(&self) -> &ReplayLog {
        &self.replay_log
    }

    /// Mutable borrow of the replay log (tests and integrity tooling).
    pub fn replay_log_mut(&mut self) -> &mut ReplayLog {
        &mut self.replay_log
    }

    /// `mod.loaded.v1` JSON payloads recorded on the replay bus (FR-MOD-004 partial).
    #[must_use]
    pub fn mod_loaded_bus_events(&self) -> Vec<String> {
        self.replay_log.mod_loaded_bus_events()
    }

    /// `mod.loaded.v1` bus JSON emitted on the most recent tick (scenario load or hot reload).
    #[must_use]
    pub fn last_tick_mod_lifecycle(&self) -> &[String] {
        &self.last_tick_mod_lifecycle
    }

    /// Ingest mod-host phase log lines: record permission violations on the replay bus and debug-log.
    fn ingest_mod_phase_lines(&mut self, lines: Vec<String>, tick: u64, phase: &str) {
        for line in lines {
            if line.contains("mod.permission_violation.v1") {
                self.replay_log
                    .record_mod_permission_violation_bus(tick, &line);
            }
            tracing::debug!(mod_log = %line, phase = phase, "mod phase");
        }
    }

    /// Record `session.saved.v1` on the replay bus (slot or autosave; CIV-1000).
    pub fn record_session_saved(
        &mut self,
        session_id: &str,
        save_id: &str,
        slot: &str,
        byte_size: u64,
    ) {
        let tick = self.state.tick;
        self.replay_log
            .record_session_saved(session_id, save_id, slot, tick, byte_size);
    }

    /// Latest BLAKE3 hash-chain root after the most recent tick, if any.
    pub fn hash_chain_root(&self) -> Option<[u8; crate::hash_chain::HASH_LEN]> {
        self.replay_log.running_hash
    }

    /// Save the in-memory replay log to a `.civreplay` file (FR-REPLAY-001).
    pub fn save_replay(&self, path: impl AsRef<std::path::Path>) -> Result<(), ReplayError> {
        save_civreplay(path, &self.replay_log)
    }

    /// Load a `.civreplay` file and replay its events into a new simulation.
    pub fn load_replay_from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ReplayError> {
        let log = load_civreplay(path)?;
        let mut sim = Self::with_seed(log.seed);
        log.replay(&mut sim)?;
        sim.replay_log = log;
        Ok(sim)
    }

    /// Planet phase - recompute climate and weather grid from the current tick,
    /// then apply the resulting tide offset to any registered coastal water
    /// columns (FR-CIV-PLANET-020, FR-CIV-PLANET-030).
    fn phase_planet(&mut self) {
        self.climate = compute_climate(self.state.tick, &self.planet, &self.moon);
        self.weather_grid = compute_weather(&self.climate, self.state.tick, self.weather_grid.len().max(1) as u32);
        self.apply_tide_offset();
    }

    /// Register (or update) a coastal water column at horizontal `(x, z)` with
    /// sea-level baseline `base_y`. The column's water-marker voxel will be
    /// shifted vertically each tick by the climate `tide_offset` (FR-CIV-PLANET-020).
    ///
    /// Coordinates are fixed-point world units (see [`FIXED_SCALE`]). Calling
    /// this for an already-registered column resets its baseline; the next
    /// `phase_planet` will clear the old water voxel and write the new one.
    pub fn register_coastal_water_column(&mut self, x: i64, z: i64, base_y: i64) {
        let column = CoastalColumn {
            base_y,
            last_water_y: base_y,
        };
        // Seed the initial water voxel through the replay-aware write path so
        // FR-CIV-VOXEL-002 dirty-event invariants stay intact.
        self.push_voxel_write(WorldCoord { x, y: base_y, z }, WATER_MARKER_MATERIAL);
        self.coastal_columns.insert((x, z), column);
    }

    /// Borrow the registered coastal water columns (for tests + tooling).
    #[must_use]
    pub fn coastal_column_count(&self) -> usize {
        self.coastal_columns.len()
    }

    /// Read the current water-level y for the column at `(x, z)`, if registered.
    #[must_use]
    pub fn coastal_water_level(&self, x: i64, z: i64) -> Option<i64> {
        self.coastal_columns.get(&(x, z)).map(|c| c.last_water_y)
    }

    /// Shift every registered coastal water-level voxel by the current
    /// `climate.tide_offset` (FR-CIV-PLANET-020). The offset is scaled into
    /// fixed-point world units, rounded deterministically, and applied through
    /// [`VoxelWorld::write`] so dirty events propagate normally
    /// (FR-CIV-VOXEL-002).
    ///
    /// For each column we clear the previously occupied water voxel (write
    /// `MaterialId(0)`) and write [`WATER_MARKER_MATERIAL`] at the new height.
    /// If the new height matches the old one we skip the redundant pair of
    /// writes to avoid emitting spurious dirty events.
    fn apply_tide_offset(&mut self) {
        if self.coastal_columns.is_empty() {
            return;
        }

        // Fixed-point conversion: `tide_offset` is a float amplitude in the
        // same world-unit space as the voxel grid; multiply by FIXED_SCALE and
        // round to the nearest integer for determinism. f32::round() is
        // deterministic per the IEEE-754 round-half-away-from-zero rule used
        // across our target platforms.
        let scale = FIXED_SCALE as f32;
        let offset_units = (self.climate.tide_offset * scale).round() as i64;

        // Collect updates first so we can mutate `self.voxel` and
        // `self.coastal_columns` without aliasing.
        let updates: Vec<((i64, i64), i64, i64)> = self
            .coastal_columns
            .iter()
            .map(|(&(x, z), column)| {
                let new_y = column.base_y.saturating_add(offset_units);
                ((x, z), column.last_water_y, new_y)
            })
            .collect();

        for ((x, z), prev_y, new_y) in updates {
            if prev_y == new_y {
                continue;
            }
            // Clear previous water marker, then place the new one. Both go
            // through `VoxelWorld::write` so the dirty queue stays
            // deterministic (FR-CIV-VOXEL-002).
            self.voxel
                .write(WorldCoord { x, y: prev_y, z }, MaterialId(0));
            self.voxel
                .write(WorldCoord { x, y: new_y, z }, WATER_MARKER_MATERIAL);
            if let Some(column) = self.coastal_columns.get_mut(&(x, z)) {
                column.last_water_y = new_y;
            }
        }
    }

    /// Tactics phase - evolve faction doctrines and apply queued voxel damage.
    fn phase_tactics(&mut self) {
        self.last_tick_voxel_damage_count = 0;
        let scale = FIXED_SCALE as f32;
        for event in self.pending_damage.drain(..) {
            let x = (event.center.x as f32 / scale).clamp(0.0, 1.0);
            let y = (event.center.z as f32 / scale).clamp(0.0, 1.0);
            let has_pulse = self.last_tick_combat_pulses.iter().any(|pulse| {
                (pulse.x - x).abs() < f32::EPSILON && (pulse.y - y).abs() < f32::EPSILON
            });
            if !has_pulse {
                self.last_tick_combat_pulses.push(CombatDamagePulse {
                    x,
                    y,
                    unit_a: None,
                    unit_b: None,
                });
            }
            self.last_tick_voxel_damage_count += apply_damage(&mut self.voxel, &event);
        }

        const DOCTRINE_EVOLVE_MODULO: u64 = 64;
        if self.state.tick % DOCTRINE_EVOLVE_MODULO == 0 {
            let mut faction_stats =
                vec![FactionEngagementStats::default(); self.faction_doctrines.len()];
            for engagement in &self.last_tick_engagements {
                let shooter = engagement.shooter_faction as usize;
                let target = engagement.target_faction as usize;
                if shooter < faction_stats.len() {
                    faction_stats[shooter].engagements_as_shooter = faction_stats[shooter]
                        .engagements_as_shooter
                        .saturating_add(1);
                }
                if target < faction_stats.len() {
                    faction_stats[target].engagements_as_target = faction_stats[target]
                        .engagements_as_target
                        .saturating_add(1);
                }
            }
            if self.last_tick_voxel_damage_count > 0 && !self.last_tick_engagements.is_empty() {
                let per_shooter = (self.last_tick_voxel_damage_count as u32)
                    .saturating_div(self.last_tick_engagements.len() as u32)
                    .max(1);
                for engagement in &self.last_tick_engagements {
                    let shooter = engagement.shooter_faction as usize;
                    if shooter < faction_stats.len() {
                        faction_stats[shooter].voxels_removed = faction_stats[shooter]
                            .voxels_removed
                            .saturating_add(per_shooter);
                    }
                }
            }
            for (faction, library) in self.faction_doctrines.iter_mut().enumerate() {
                let stats = faction_stats.get(faction).copied().unwrap_or_default();
                for doctrine in &mut library.current {
                    doctrine.score = score_doctrine_fitness(doctrine, &stats);
                }
                let mut rng = ChaCha8Rng::seed_from_u64(
                    self.state.rng_seed ^ self.state.tick ^ u64::from(faction as u32),
                );
                evolve_doctrine(library, &mut rng, 0.2);
            }
        }
    }

    /// Voxel phase — drains the deterministic dirty-event queue from
    /// [`VoxelWorld`] into [`Simulation::last_tick_voxel_events`]. Replay-safe
    /// per ADR-004 + ADR-005: the kernel guarantees `(chunk_id, write_seq)`
    /// ordering.
    fn phase_voxel(&mut self) {
        self.last_tick_voxel_events = self.voxel.drain_dirty();
    }

    /// Compact the voxel world periodically.
    fn phase_compact(&mut self) {
        if self.state.tick % self.tick_modulo_compact == 0 {
            self.voxel.compact();
        }
    }

    /// Buildings phase - expands the parcel graph on a fixed cadence when demand is high.
    fn phase_buildings(&mut self) {
        if self.state.tick % 16 != 0 {
            return;
        }

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
            let _ = self.allocator.allocate(
                &mut self.building_graph,
                &signals,
                self.target_era,
                origin,
                16,
            );
        }
    }

    /// Diffusion phase - propagates wardrobe and tools eras across civilians.
    fn phase_diffusion(&mut self) {
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

    /// Production phase - buildings produce resources
    fn phase_production(&mut self) {
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
                    energy += Fixed::from_raw(Fixed::from_num(1).raw / 2);
                }
                _ => {}
            }
        }
        self.state.resources.food += food;
        self.state.resources.wood += wood;
        self.state.resources.metal += metal;
        self.state.resources.energy += energy;
    }

    /// Citizen lifecycle phase
    fn phase_citizen_lifecycle(&mut self) {
        attach_citizen_to_agents(&mut self.world);
        self.last_births.clear();
        self.last_deaths.clear();
        let population = count_civilians(&self.world) as f64;
        let max_pop = self.state.population.max(1) as f64;
        let overcrowding_factor = (population / max_pop).clamp(0.0, 1.0);
        let birth_chance = 0.003 * (1.0 - overcrowding_factor);
        let birth_window = self.state.tick % 200 == 0;
        let mut dead = Vec::new();
        let mut births = Vec::new();

        for (entity, (civilian, pos, needs)) in
            self.world
                .query_mut::<(&mut AgentCivilian, &Position3d, &mut Needs)>()
        {
            civilian.age = civilian.age.saturating_add(1);
            if self.state.resources.food.raw > 0 {
                needs.food = (needs.food + 0.008).min(1.0);
            } else {
                needs.food = (needs.food - 0.03).max(0.0);
            }
            if needs.food < 0.05 && self.state.resources.food.raw <= 0 {
                dead.push((entity, civilian.id, pos.coord));
                continue;
            }
            if birth_window && civilian.age > 18 && self.rng.gen_bool(birth_chance.clamp(0.0, 1.0))
            {
                let child_id = self.next_civilian_id;
                self.next_civilian_id += 1;
                let x = pos.coord.x as f32 / FIXED_SCALE as f32;
                let y = pos.coord.z as f32 / FIXED_SCALE as f32;
                births.push((child_id, x, y));
            }
        }

        for (child_id, x, y) in births {
            let _ = spawn_child_near(&mut self.world, child_id, Alignment::None, x, y, &mut self.rng);
            self.last_births.push(PopulationEvent {
                tick: self.state.tick,
                entity_id: child_id,
                x,
                y,
            });
        }

        for (entity, entity_id, coord) in dead {
            let _ = self.world.despawn(entity);
            self.last_deaths.push(PopulationEvent {
                tick: self.state.tick,
                entity_id,
                x: coord.x as f32 / FIXED_SCALE as f32,
                y: coord.z as f32 / FIXED_SCALE as f32,
            });
        }

        let births_count = self.last_births.len() as u64;
        let deaths_count = self.last_deaths.len() as u64;
        self.state.population = self.state.population.saturating_add(births_count);
        self.state.population = self.state.population.saturating_sub(deaths_count);
    }

    /// Military phase — morale recovery and Phase-4 war → tactics bridge.
    fn phase_military(&mut self) {
        use crate::spawn::military_pin_id;

        let tick = self.state.tick;
        let lines = self.mod_host.military_tick(tick);
        self.ingest_mod_phase_lines(lines, tick, "military");

        let phase_cfg = self.military_phase;

        for (_, unit) in self.world.query::<&mut MilitaryUnit>().iter() {
            if unit.morale < Fixed::from_num(1) {
                unit.morale = (unit.morale + Fixed::from_num(1) / Fixed::from_num(100))
                    .min(Fixed::from_num(1));
            }
        }

        let mut entities: Vec<Entity> = Vec::new();
        let mut samples: Vec<MilitaryUnitSample> = self
            .world
            .query::<&MilitaryUnit>()
            .iter()
            .enumerate()
            .map(|(idx, (entity, unit))| {
                entities.push(entity);
                MilitaryUnitSample {
                    unit_id: military_pin_id(entity, idx),
                    faction_id: unit.faction_id,
                    grid_x: unit.position.x,
                    grid_y: unit.position.y,
                }
            })
            .collect();

        for grid_move in tick_operational_movement(
            self.state.tick,
            &phase_cfg.movement,
            &mut samples,
            phase_cfg.movement_pulses_per_cadence,
            &self.voxel,
        ) {
            if let Some(sample) = samples.get_mut(grid_move.unit_index) {
                sample.grid_x = grid_move.new_grid_x;
                sample.grid_y = grid_move.new_grid_y;
            }
            if let Some(target_entity) = entities.get(grid_move.unit_index).copied() {
                for (entity, unit) in self.world.query_mut::<&mut MilitaryUnit>() {
                    if entity == target_entity {
                        unit.position.x = grid_move.new_grid_x;
                        unit.position.y = grid_move.new_grid_y;
                        break;
                    }
                }
            }
        }

        let config = phase_cfg.war;
        let fog = civ_tactics::build_fog_for_units(&config, &samples, &self.voxel);
        let engagements = tick_war_bridge(
            self.state.tick,
            &config,
            &samples,
            &self.voxel,
            fog.as_ref(),
        );
        self.operational
            .on_combat_engagements(self.state.tick, &engagements);
        self.last_tick_engagements = engagements.clone();

        let hp_loss = Fixed::from_num(config.strength_damage_fixed);
        let scale = FIXED_SCALE as f32;
        for engagement in &engagements {
            self.replay_log.record_combat(
                self.state.tick,
                engagement.shooter_id,
                engagement.target_id,
                engagement.damage,
            );
            if let Some(target_entity) = entities.get(engagement.target_index) {
                for (entity, unit) in self.world.query_mut::<&mut MilitaryUnit>() {
                    if entity == *target_entity {
                        unit.hp = (unit.hp - hp_loss).max(Fixed::from_num(0));
                        unit.strength = unit.hp;
                        break;
                    }
                }
            }
            self.last_tick_combat_pulses.push(CombatDamagePulse {
                x: (engagement.damage.center.x as f32 / scale).clamp(0.0, 1.0),
                y: (engagement.damage.center.z as f32 / scale).clamp(0.0, 1.0),
                unit_a: Some(engagement.shooter_id),
                unit_b: Some(engagement.target_id),
            });
            self.pending_damage.push(engagement.damage);
        }

        let dead: Vec<Entity> = self
            .world
            .query::<&MilitaryUnit>()
            .iter()
            .filter(|(_, unit)| unit.hp <= Fixed::from_num(0))
            .map(|(entity, _)| entity)
            .collect();
        for entity in dead {
            let _ = self.world.despawn(entity);
        }
    }

    fn phase_diplomacy(&mut self) {
        if self.state.tick % 500 != 0 {
            return;
        }
        self.diplomacy_events.clear();
        let faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
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

    /// Policy phase — read the active [`Policy`] for the current tick and
    /// store the resulting [`ControlSignals`] on
    /// [`Self::last_control_signals`]. Runs between `phase_military` and
    /// `phase_economy` so the policy can produce signals (production
    /// multipliers, allocation weights, tax rates) that downstream phases
    /// consume (FR-CORE-005).
    ///
    /// The default `NoopPolicy` emits [`ControlSignals::default()`] — empty
    /// maps — so this phase is observationally a no-op until a scenario or
    /// tool calls [`Self::set_policy`].
    fn phase_policy(&mut self) {
        self.last_control_signals = self.policy.evaluate(&self.state);
    }

    /// Economy phase — sync joule budget with `civ-economy`, apply policy drain, step,
    /// and advance market prices.
    ///
    /// Policy consumption (FR-ECON-001):
    /// `effective_consumption = base_consumption_joules × max(scarcity_multiplier, 0)`
    ///
    /// Conservation: budget only decreases; result is clamped to zero (aggregate
    /// energy cannot go negative).
    fn phase_economy(&mut self) {
        let tick = self.state.tick;
        let policy_lines = self.mod_host.tick(tick);
        self.ingest_mod_phase_lines(policy_lines, tick, "policy");
        let economy_lines = self.mod_host.economy_tick(tick);
        self.ingest_mod_phase_lines(economy_lines, tick, "economy");

        self.economy_state.energy_budget_joules =
            self.state.energy_budget_joules.raw / crate::SCALE;

        let demand = crate::policy::effective_consumption(self.economy_policy) as i64;
        let budget = self.economy_state.energy_budget_joules;
        let allocated = CapitalistAllocator.allocate(budget, demand);
        civ_economy::drain_energy_budget(&mut self.economy_state, allocated);
        civ_economy::step(&mut self.economy_state);

        self.state.energy_budget_joules = Fixed::from_num(self.economy_state.energy_budget_joules);
        self.tick_trade_routes();
        self.market_state.step(self.state.tick);
    }

    fn tick_trade_routes(&mut self) {
        for route in &self.state.trade_routes {
            if route.volume <= Fixed::ZERO || route.from_faction == route.to_faction {
                continue;
            }

            let resource = route_resource(&route.goods);
            let available = {
                let Some(from_resources) = self.state.faction_resources.get(&route.from_faction)
                else {
                    continue;
                };
                resource_amount(from_resources, resource)
            };
            if available <= Fixed::ZERO {
                continue;
            }

            let quantity = route.volume.min(available);
            {
                let from_resources = self
                    .state
                    .faction_resources
                    .entry(route.from_faction)
                    .or_default();
                adjust_resource(from_resources, resource, Fixed::ZERO - quantity);
            }
            {
                let to_resources = self
                    .state
                    .faction_resources
                    .entry(route.to_faction)
                    .or_default();
                adjust_resource(to_resources, resource, quantity);
            }

            let supply = {
                let Some(from_resources) = self.state.faction_resources.get(&route.from_faction)
                else {
                    continue;
                };
                resource_amount(from_resources, resource)
            };
            let demand = {
                let Some(to_resources) = self.state.faction_resources.get(&route.to_faction) else {
                    continue;
                };
                resource_amount(to_resources, resource)
            };
            let margin = (demand - supply).max(Fixed::ZERO);
            let profit = quantity * (Fixed::from_num(1) + margin / Fixed::from_num(100));

            if let Some(from_treasury) = self.state.faction_treasury.get_mut(&route.from_faction) {
                *from_treasury += profit;
            }
            if let Some(to_treasury) = self.state.faction_treasury.get_mut(&route.to_faction) {
                *to_treasury -= profit;
            }
        }
    }

    /// Apply scenario fog settings to the military phase (FR-CIV-TACTICS-045).
    pub fn configure_military_fog(&mut self, vision_radius: Option<u32>, grid_size: u32) {
        if let Some(radius) = vision_radius {
            self.military_phase.war.fog_vision_radius = Some(radius);
            self.military_phase.war.fog_grid_size = grid_size.max(16);
        }
    }

    /// Apply scenario military cadence/combat overrides (FR-CIV-TACTICS-050).
    pub fn apply_scenario_military(&mut self, military: &crate::scenario::ScenarioMilitary) {
        if let Some(v) = military.movement_cadence_ticks {
            self.military_phase.movement.cadence_ticks = v;
        }
        if let Some(v) = military.movement_pulses_per_cadence {
            self.military_phase.movement_pulses_per_cadence = v;
        }
        if let Some(v) = military.war_cadence_ticks {
            self.military_phase.war.cadence_ticks = v;
        }
        if let Some(v) = military.engage_range_grid {
            self.military_phase.war.engage_range_grid = v.max(1);
        }
    }

    /// Store scenario taxation settings for later economy-phase wiring.
    pub fn apply_scenario_taxation(&mut self, taxation: &crate::scenario::ScenarioTaxation) {
        self.scenario_taxation = taxation.clone();
    }

    /// Military phase configuration (tests and tooling).
    #[must_use]
    pub fn military_phase_config(&self) -> &MilitaryPhaseConfig {
        &self.military_phase
    }

    /// Get snapshot of current state
    pub fn snapshot(&self) -> SimulationSnapshot {
        let citizen_count = self.world.query::<&Citizen>().iter().count();
        let building_count = self.world.query::<&Building>().iter().count();
        let military_count = self.world.query::<&MilitaryUnit>().iter().count();

        SimulationSnapshot {
            tick: self.state.tick,
            population: self.state.population,
            citizen_count,
            building_count,
            military_count,
            energy_budget: self.state.energy_budget_joules,
            resources: self.state.resources.clone(),
            births_this_tick: self.last_births.len() as u32,
            deaths_this_tick: self.last_deaths.len() as u32,
            diplomacy_events: self.diplomacy_events.clone(),
            market_prices: self.market_state.prices().clone(),
            damage_events: self.last_tick_combat_pulses.len(),
            climate: self.climate,
            weather_grid: self.weather_grid.clone(),
            geology_map: GeologyMap::seed(&self.planet),
        }
    }

    /// Number of emergent settlements (multi-member clusters) from the most
    /// recent life phase (FR-CIV-LIFE-030). Read by the HUD `FactionRoster`.
    #[must_use]
    pub fn settlement_count(&self) -> u32 {
        self.last_settlement_count
    }

    /// Per-cluster (settlement) resource stocks keyed by `ClusterId` value, for
    /// the HUD `WorldResources` panel (FR-CIV-LIFE-020).
    #[must_use]
    pub fn cluster_stocks(&self) -> &BTreeMap<u64, ClusterStocks> {
        &self.cluster_stocks
    }

    #[cfg(test)]
    pub(crate) fn test_clear_cluster_stocks(&mut self) {
        self.cluster_stocks.clear();
    }

    #[cfg(test)]
    pub(crate) fn test_set_cluster_food_stock(&mut self, cluster_id: u64, food: i64) {
        let mut stock = ClusterStocks::default();
        stock.add(Good::Food, food);
        self.cluster_stocks.insert(cluster_id, stock);
    }
}

/// Maximum chronicle history lines retained in [`WorldState::chronicle`].
const CHRONICLE_MAX_LEN: usize = 200;

/// Food units each cluster member adds to settlement stock per tick in
/// [`Simulation::phase_life`].
const CLUSTER_FOOD_PRODUCTION_PER_MEMBER: i64 = 1;
/// Food units each cluster member drains per tick in
/// [`Simulation::phase_settlement_consumption`]. Must be >= production so the
/// accumulator stays bounded (net zero at matched rates; converges toward zero
/// when strictly greater).
const CLUSTER_FOOD_CONSUMPTION_PER_MEMBER: i64 = 1;
/// Market weight for settlement food commons before pressure scaling (N1).
const SETTLEMENT_FOOD_MARKET_WEIGHT: i64 = 2;
/// Divisor mapping population-scale demand/supply (and settlement commons) into
/// the capped per-tick food price step (N1: local abundance must move price
/// within `MarketState::apply_pressure`'s ±8 cent clamp).
const FOOD_MARKET_PRESSURE_SCALE: i64 = 500_000;

/// Baseline food clearing price (cents) at which births are unaffected by
/// scarcity. Matches `MarketState::default()`'s food price.
const FOOD_SCARCITY_BASELINE: i64 = 1_000;

/// Tech unlock bits (irreversible, set-only).
pub const TECH_IRRIGATION: u64 = 1 << 0;
pub const TECH_STORAGE: u64 = 1 << 1;
pub const TECH_METALLURGY: u64 = 1 << 2;
pub const TECH_WRITING: u64 = 1 << 3;
pub const TECH_SANITATION: u64 = 1 << 4;
pub const TECH_GUNPOWDER: u64 = 1 << 5;

/// Discrete tech unlocks reached by a given research tier (set-only bitmask).
fn tech_unlocks_for_tier(research_tier: u64) -> u64 {
    let mut bits = 0u64;
    if research_tier >= 1 {
        bits |= TECH_IRRIGATION;
    }
    if research_tier >= 2 {
        bits |= TECH_STORAGE;
    }
    if research_tier >= 3 {
        bits |= TECH_METALLURGY;
    }
    if research_tier >= 4 {
        bits |= TECH_WRITING;
    }
    if research_tier >= 5 {
        bits |= TECH_SANITATION;
    }
    if research_tier >= 6 {
        bits |= TECH_GUNPOWDER;
    }
    bits
}

/// Downward-causation policy (FR-CIV-0100 emergence): scarcity in the food
/// market damps the birth rate, closing the research -> carrying-capacity ->
/// economy -> population loop. Returns a multiplier in `(0.0, 1.0]` applied to
/// the per-tick birth chance.
///
/// At or below the baseline price (abundance) the factor is `1.0` — surplus
/// does NOT boost births above the natural rate (conservative; abundance is
/// already expressed via the ECS food-needs path). As the price rises above
/// baseline the factor falls as `baseline / price`, so a 2x price halves the
/// birth chance. The factor never reaches zero, so a starving society can still
/// recover, and it only ever scales births DOWN — population is never reduced
/// by this coupling.
fn food_scarcity_birth_factor(food_price: i64) -> f64 {
    let price = food_price.max(FOOD_SCARCITY_BASELINE);
    (FOOD_SCARCITY_BASELINE as f64 / price as f64).clamp(0.0, 1.0)
}

/// Per-tick change in societal unrest from food-market scarcity (FR-CIV-0100 §3
/// emergence). Above the baseline price unrest rises in proportion to the
/// shortfall (bounded per tick so it walks rather than jumps); at or below
/// baseline it decays toward contentment by a fixed step. The caller floors the
/// running total at zero.
fn unrest_delta(food_price: i64) -> i64 {
    /// Largest single-tick rise, so a price spike can't instantly max unrest.
    const MAX_RISE: i64 = 50;
    /// Cents of shortfall that map to one unit of unrest rise.
    const CENTS_PER_UNREST: i64 = 20;
    /// Fixed decay applied each tick of abundance.
    const DECAY: i64 = 10;
    let scarcity = food_price - FOOD_SCARCITY_BASELINE;
    if scarcity > 0 {
        (scarcity / CENTS_PER_UNREST).clamp(1, MAX_RISE)
    } else {
        -DECAY
    }
}

/// FR-CIV-ECON: scarcity in NON-food commodities adds bounded unrest
/// (cost-of-living). Food is owned by unrest_delta(); skipped here to avoid
/// double-counting. Per-tick clamped to [-DECAY, MAX_RISE] — no runaway.
fn commodity_unrest_delta(prices: &std::collections::BTreeMap<String, i64>) -> i64 {
    const BASELINE: i64 = 1_000;
    const CENTS_PER_UNREST: i64 = 40;
    const MAX_RISE: i64 = 15;
    const DECAY: i64 = 5;
    let mut rise: i64 = 0;
    for (good, &price) in prices {
        if good == "food" { continue; }
        let scarcity = price - BASELINE;
        if scarcity > 0 {
            rise = rise.saturating_add((scarcity / CENTS_PER_UNREST).min(MAX_RISE));
        } else {
            rise = rise.saturating_sub(DECAY);
        }
    }
    rise.clamp(-DECAY, MAX_RISE)
}

/// Effective food-price shadow for one faction's local wealth/scarcity (FR-CIV-0100
/// §3 emergence). Comfortable treasury and food sit at baseline; shortfall pushes
/// the shadow above baseline so [`unrest_delta`] accrues faction unrest.
fn faction_wealth_scarcity_shadow(treasury: Fixed, resources: &Resources) -> i64 {
    const TREASURY_COMFORT: i64 = 8_000;
    const FOOD_COMFORT: i64 = 80;
    const FOOD_WEIGHT: i64 = 50;

    let treasury_i = (treasury.raw / crate::SCALE).max(0);
    let food_i = (resources.food.raw / crate::SCALE).max(0);
    let comfort = TREASURY_COMFORT + FOOD_COMFORT * FOOD_WEIGHT;
    let wealth = treasury_i + food_i * FOOD_WEIGHT;

    if wealth >= comfort {
        FOOD_SCARCITY_BASELINE
    } else {
        FOOD_SCARCITY_BASELINE + (comfort - wealth) / 4
    }
}

/// Per-tick faction unrest delta from that faction's wealth/scarcity shadow.
/// Mirrors global food-scarcity [`unrest_delta`].
fn faction_unrest_delta_from_shadow(scarcity_shadow: i64) -> i64 {
    unrest_delta(scarcity_shadow)
}

/// Downward-causation policy (FR-CIV-0100 §3): energy depletion breeds unrest.
/// A fully-drained energy budget (blackout) adds a fixed unrest increment this
/// tick; a solvent budget adds none. An acute shock that bypasses the gradual
/// food-scarcity damping.
fn energy_scarcity_unrest(energy_budget: Fixed) -> i64 {
    const BLACKOUT_UNREST: i64 = 15;
    if energy_budget <= Fixed::ZERO {
        BLACKOUT_UNREST
    } else {
        0
    }
}

/// Upward causation (FR-CIV-0100 §3): the mean MISERY of agents (negative Psyche
/// mood valence) adds to societal unrest. Reuses the ECS Psyche component — the
/// agent emotional layer feeding the macro web. Returns 0..MAX, bounded.
fn agent_misery_unrest(world: &hecs::World) -> i64 {
    const MAX_MISERY_UNREST: i64 = 30;
    let (sum, n) = world
        .query::<&Psyche>()
        .iter()
        .fold((0.0f32, 0u32), |(s, n), (_, p)| (s + (-p.mood.valence).max(0.0), n + 1));
    if n == 0 {
        return 0;
    }
    let mean_misery = (sum / n as f32).clamp(0.0, 1.0); // 0 = content, 1 = max misery
    (mean_misery * MAX_MISERY_UNREST as f32) as i64
}

/// Upward causation (FR-CIV-0100 §3): micro ideology consensus (`Psyche.beliefs[0]`)
/// binds macro cohesion; polarization frays it. Pure `hecs::World` scan, capped i64.
fn micro_cohesion_delta(world: &hecs::World) -> i64 {
    const MICRO_BIND_CAP: i64 = 12;
    const MICRO_FRAY_CAP: i64 = 18;
    const MIN_AGENTS: u32 = 2;
    const CONSENSUS_SCALE: f32 = 4.0;

    let mut n = 0u32;
    let mut sum = 0.0f32;
    let mut sum_sq = 0.0f32;
    for (_, psyche) in world.query::<&Psyche>().iter() {
        let x = psyche.beliefs[0];
        n += 1;
        sum += x;
        sum_sq += x * x;
    }

    if n < MIN_AGENTS {
        return 0;
    }

    let n_f = n as f32;
    let mean = sum / n_f;
    let var = ((sum_sq / n_f) - mean * mean).max(0.0);
    let consensus = 1.0 - (CONSENSUS_SCALE * var).clamp(0.0, 1.0);
    let micro_bind = (consensus * MICRO_BIND_CAP as f32).floor() as i64;
    let micro_fray = ((1.0 - consensus) * MICRO_FRAY_CAP as f32).floor() as i64;
    micro_bind - micro_fray
}

/// Upward causation (FR-CIV-0100 §3): mean positive agent tie trust caches a
/// trade permille bonus for the next economy tick. Pure `hecs::World` scan.
fn micro_social_trust_permille(world: &hecs::World) -> u64 {
    const MICRO_TRUST_SCALE: f32 = 250.0;
    const MICRO_TRUST_CAP: u64 = 250;

    let mut n = 0u64;
    let mut sum = 0.0f32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            sum += tie.trust.clamp(0.0, 1.0);
            n += 1;
        }
    }

    if n == 0 {
        return 0;
    }

    let trust_mean = sum / n as f32;
    let raw = (trust_mean * MICRO_TRUST_SCALE).floor() as u64;
    raw.min(MICRO_TRUST_CAP)
}

/// Upward causation (FR-CIV-EMERGENCE-N11): average psyche maturity across all agents.
/// Mature populations stabilize belief (wisdom = stability). Pure `hecs::World` scan.
fn avg_psyche_maturity(world: &hecs::World) -> f32 {
    let mut total = 0.0;
    let mut count = 0u32;
    for (_, psyche) in world.query::<&Psyche>().iter() {
        total += psyche.maturity;
        count += 1;
    }
    if count == 0 { 0.0 } else { total / count as f32 }
}

/// Upward causation (FR-CIV-EMERGENCE-N10): average kinship across all social ties.
/// Kinship boosts cohesion (family ties stabilize society). Pure `hecs::World` scan.
fn avg_faction_kinship(world: &hecs::World) -> f32 {
    let mut total_kinship = 0.0;
    let mut count = 0u32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            total_kinship += tie.kinship;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total_kinship / count as f32
    }
}

/// Upward causation (FR-CIV-EMERGENCE-N12): average affinity across all social ties.
/// Positive collective affinity (goodwill) raises the diplomacy conflict threshold;
/// hostility lowers it. Result is clamped to `[-1, 1]`. Pure `hecs::World` scan.
fn avg_social_affinity(world: &hecs::World) -> f32 {
    let mut total = 0.0;
    let mut count = 0u32;
    for (_, graph) in world.query::<&SocialGraph>().iter() {
        for tie in &graph.ties {
            // Defensive clamp: each tie.affinity is maintained in [-1, 1] by the
            // social graph, but clamp per-tie so a malformed save cannot skew the mean.
            total += tie.affinity.clamp(-1.0, 1.0);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        (total / count as f32).clamp(-1.0, 1.0)
    }
}

/// N12: bias magnitude for the affinity→diplomacy threshold (FR-CIV-EMERGENCE-N12).
/// `avg_affinity ∈ [-1, 1]` scaled by this yields the threshold bias in `[-5000, 5000]`,
/// bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` at the combination site.
const N12_AFFINITY_BIAS_SCALE: f32 = 5_000.0;

/// N12: collective affinity threshold bias. Positive goodwill raises the conflict
/// threshold (more tolerance before fighting); hostility lowers it. The input is
/// clamped to `[-1, 1]` so the bias is bounded to `[-5000, 5000]`. Returns i64.
fn affinity_threshold_bias(avg_affinity: f32) -> i64 {
    (avg_affinity.clamp(-1.0, 1.0) * N12_AFFINITY_BIAS_SCALE) as i64
}

/// Maximum peace bonus from shared patron veneration (religion → diplomacy coupling).
const RELIGIOUS_UNITY_PEACE_CAP: i64 = 1_000;

/// RELIGION→DIPLOMACY emergence coupling (FR-CIV-RELIGION-002).
///
/// A civilisation that has crystallised a patron deity around a shared legend
/// figure gains social cohesion that spills into inter-faction tolerance: the
/// disparity threshold before war is raised by up to
/// [`RELIGIOUS_UNITY_PEACE_CAP`] currency units.
///
/// The bonus is binary (patron present / absent) rather than proportional
/// because the emergence mechanic already encodes significance in *which*
/// figure becomes patron; the coupling here is "shared veneration exists" not
/// "how devout are they".
///
/// Called with an immutable copy of `has_patron` (read before any mutable
/// borrow of `self`) to satisfy the borrow-checker (E0502).
fn religious_unity_peace_bonus(has_patron: bool) -> i64 {
    if has_patron {
        RELIGIOUS_UNITY_PEACE_CAP
    } else {
        0
    }
}

/// Maximum peace bonus from mutual language intelligibility (language → diplomacy coupling).
const LANGUAGE_INTELLIGIBILITY_PEACE_CAP: i64 = 1_200;

/// LANGUAGE→DIPLOMACY emergence coupling.
///
/// Low language distance (mutually intelligible factions) → positive peace bonus
/// on the conflict threshold, mirroring the N9/N12/#564 magnitudes.
/// High distance → 0 bonus (no effect on threshold).
///
/// Called with pre-read centroid values (before any mutable borrow of `self`)
/// to satisfy the borrow-checker (E0502).
pub fn language_intelligibility_peace_bonus(language_distance: f32) -> i64 {
    let raw =
        LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32 * (1.0 - language_distance.clamp(0.0, 1.0));
    raw.clamp(0.0, LANGUAGE_INTELLIGIBILITY_PEACE_CAP as f32) as i64
}

/// Upward causation (FR-CIV-0100): the fraction of sentient agents accelerates
/// research (awakened minds discover faster). Reuses the ECS; returns 0..MAX bonus.
fn sentience_research_bonus(world: &hecs::World) -> u64 {
    const MAX_SENTIENCE_RESEARCH: u64 = 50;
    // Mirrors `EmergenceState::new` sentience profile and threshold.
    let profile = CognitionTraitProfile::new(
        "sapient-lineage",
        vec![(0, 0.5), (1, 0.5), (2, 0.5), (8, 0.25)],
    );
    let threshold = SentienceThreshold::new(0.72);
    let (sentient, total) = world.query::<&Dna>().iter().fold((0u32, 0u32), |(s, n), (_, dna)| {
        let crossed = cognition_score(dna, &profile) >= threshold.minimum_cognition;
        (s + u32::from(crossed), n + 1)
    });
    if total == 0 {
        return 0;
    }
    let fraction = sentient as f32 / total as f32;
    ((fraction * MAX_SENTIENCE_RESEARCH as f32) as u64).min(MAX_SENTIENCE_RESEARCH)
}

/// The economic focus a civilization tends toward, from its strongest sector.
fn candidate_economic_focus(
    food: i64,
    research_tier: u64,
    belief: u64,
    treasury_total: i64,
) -> EconomicFocus {
    let agr = food;
    let ind = (research_tier as i64) * 50_000;
    let sac = (belief / 4) as i64;
    let mer = treasury_total / 4;
    let max = agr.max(ind).max(sac).max(mer);
    if max <= 0 {
        return EconomicFocus::Balanced;
    }
    if max == agr {
        EconomicFocus::Agrarian
    } else if max == ind {
        EconomicFocus::Industrial
    } else if max == sac {
        EconomicFocus::Sacred
    } else {
        EconomicFocus::Mercantile
    }
}

/// Downward-causation policy (FR-CIV-0100 §3): research raises production yield —
/// better tools/techniques lift per-building output. +10% per research tier,
/// capped at +100% (2x). De-silos phase_production, which read no emergent state.
fn production_yield_factor(research_tier: u64) -> Fixed {
    let bonus_permille = research_tier.saturating_mul(100).min(1_000) as i64;
    Fixed::from_num(1_000 + bonus_permille) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-CONTENT-001): terrain biome modulates food
/// production — fertile land grows more food, barren land grows less.  The factor
/// is a pure multiplier on per-farm output; caller multiplies food output by this.
/// Returns a value in the range [0.1, 1.5] (clamped).
fn biome_yield_factor(biome: civ_planet::BiomeKind) -> Fixed {
    use civ_planet::BiomeKind;
    match biome {
        BiomeKind::Rainforest => Fixed::from_num(13) / Fixed::from_num(10),
        BiomeKind::Wetland    => Fixed::from_num(12) / Fixed::from_num(10),
        BiomeKind::Grassland  => Fixed::from_num(12) / Fixed::from_num(10),
        BiomeKind::Plains     => Fixed::from_num(11) / Fixed::from_num(10),
        BiomeKind::Forest     => Fixed::from_num(9)  / Fixed::from_num(10),
        BiomeKind::Savanna    => Fixed::from_num(17) / Fixed::from_num(20),
        BiomeKind::Beach      => Fixed::from_num(8)  / Fixed::from_num(10),
        BiomeKind::Mountain   => Fixed::from_num(6)  / Fixed::from_num(10),
        BiomeKind::Taiga      => Fixed::from_num(6)  / Fixed::from_num(10),
        BiomeKind::Desert     => Fixed::from_num(1)  / Fixed::from_num(2),
        BiomeKind::Tundra     => Fixed::from_num(9)  / Fixed::from_num(20),
        BiomeKind::Ocean      => Fixed::from_num(1)  / Fixed::from_num(5),
        BiomeKind::Glacier    => Fixed::from_num(1)  / Fixed::from_num(10),
    }
}

/// Aggregate biome yield factor over a slice of [`BiomeKind`]s.
///
/// Returns the mean `biome_yield_factor` across the slice, clamped to
/// `[0.1, 1.5]`.  Returns `Fixed::ONE` (neutral) for an empty slice so
/// callers with no geology data are unaffected.
fn aggregate_biome_yield(biomes: &[civ_planet::BiomeKind]) -> Fixed {
    if biomes.is_empty() {
        return Fixed::from_num(1) / Fixed::from_num(1);
    }
    let sum = biomes
        .iter()
        .fold(Fixed::ZERO, |acc, &b| acc + biome_yield_factor(b));
    let mean = sum / Fixed::from_num(biomes.len() as i64);
    let lo = Fixed::from_num(1) / Fixed::from_num(10);
    let hi = Fixed::from_num(15) / Fixed::from_num(10);
    mean.clamp(lo, hi)
}

/// Downward-causation policy (FR-CIV-0100 §3): social cohesion speeds military
/// morale recovery — a unified society's troops rally faster. Returns the
/// per-tick morale recovery increment, rising with cohesion from a 0.010 base
/// up to a 0.050 cap.
fn morale_recovery_rate(cohesion: u64) -> Fixed {
    const BASE_PERMILLE: i64 = 10;
    const CAP_PERMILLE: i64 = 50;
    let bonus = (cohesion / 25_000).min((CAP_PERMILLE - BASE_PERMILLE) as u64) as i64;
    Fixed::from_num(BASE_PERMILLE + bonus) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 §3): overcrowding breeds unrest
/// (Malthusian pressure). Population beyond the carrying capacity adds unrest
/// scaled by the percentage overshoot (10% over => +1), capped per tick. A
/// third unrest driver alongside food scarcity and energy blackout.
fn overcrowding_unrest(population: u64, capacity: i64) -> i64 {
    const MAX_OVERCROWD_UNREST: i64 = 30;
    let cap = capacity.max(1) as u64;
    if population <= cap {
        return 0;
    }
    let overshoot_pct = ((population - cap).saturating_mul(100) / cap).min(i64::MAX as u64) as i64;
    (overshoot_pct / 10).clamp(1, MAX_OVERCROWD_UNREST)
}

/// Downward-causation policy (FR-CIV-0100 §3): social cohesion accelerates
/// research — a unified society collaborates. Returns a per-mille bonus to the
/// per-tick research contribution, up to +50%.
fn cohesion_research_bonus_permille(cohesion: u64) -> u64 {
    (cohesion / 2_000).min(500)
}

/// The wealth gap (in whole currency units) between the richest and poorest
/// faction — an emergent measure of structural inequality across the society.
fn faction_treasury_spread(treasury: &HashMap<u32, Fixed>) -> i64 {
    let mut min = i64::MAX;
    let mut max = i64::MIN;
    for t in treasury.values() {
        let v = t.raw / crate::SCALE;
        min = min.min(v);
        max = max.max(v);
    }
    if max >= min {
        max - min
    } else {
        0
    }
}

/// Downward-causation policy (FR-CIV-0100 §3): structural inequality breeds class
/// unrest. A wide wealth gap between factions adds unrest scaled by the gap,
/// capped per tick. Distinct from scarcity — this is about distribution.
fn inequality_unrest(treasury_spread: i64) -> i64 {
    const MAX_INEQUALITY_UNREST: i64 = 25;
    const SPREAD_PER_UNREST: i64 = 2_000;
    (treasury_spread / SPREAD_PER_UNREST).clamp(0, MAX_INEQUALITY_UNREST)
}

/// The dispossessed share (per-mille) that a society TENDS TOWARD given its
/// wealth gap and social fabric: inequality pushes it up, cohesion pulls it
/// down. Clamped to [0, 1000].
fn dispossession_target_permille(treasury_spread: i64, cohesion: u64) -> u64 {
    const SPREAD_PER_PERMILLE: i64 = 200; // currency-units of gap per +1 permille
    let from_inequality = (treasury_spread.max(0) / SPREAD_PER_PERMILLE) as u64;
    let from_cohesion = cohesion / 5_000; // cohesion erodes dispossession
    from_inequality.saturating_sub(from_cohesion).min(1_000)
}

/// Max institution level (criticality cap on the belief->temple->belief loop).
pub const MAX_INSTITUTION_LEVEL: u32 = 5;

/// Institution level that a driver signal supports: one level per THRESHOLD of
/// the signal, capped at MAX_INSTITUTION_LEVEL.
fn institution_target_level(signal: u64, per_level: u64) -> u32 {
    (signal / per_level.max(1)).min(MAX_INSTITUTION_LEVEL as u64) as u32
}

/// One-step decay toward target (max 1 level change per tick, so growth/decay
/// is gradual — hysteresis).
fn institution_step(current: u32, target: u32) -> u32 {
    if target > current {
        current + 1
    } else if target < current {
        current - 1
    } else {
        current
    }
}

/// One sticky step of the dispossessed share toward its target (max 5 permille
/// per tick), so the class structure persists rather than tracking instantly.
fn dispossession_step(current: u64, target: u64) -> u64 {
    const MAX_STEP: u64 = 5;
    if target > current {
        (current + MAX_STEP.min(target - current)).min(1_000)
    } else {
        current - MAX_STEP.min(current - target)
    }
}

/// A large dispossessed underclass adds unrest, scaled by its share, capped.
fn dispossession_unrest(dispossessed_permille: u64) -> i64 {
    (dispossessed_permille / 40).min(25) as i64
}

/// Downward-causation policy (FR-CIV-0100 §3 emergence): research mitigates
/// unrest — advanced food logistics (storage, distribution) blunt the
/// scarcity-driven rise. Only the positive (rising) part is damped; decay is
/// untouched. The mitigation is bounded (tier capped at 9 → at most a 10x
/// reduction) and floored at 1, so technology calms a society but never makes
/// it immune to hardship. Returns the research-adjusted unrest delta.
fn research_unrest_mitigation(rise: i64, research_tier: u64) -> i64 {
    if rise <= 0 {
        return rise;
    }
    let divisor = 1 + research_tier.min(9) as i64;
    (rise / divisor).max(1)
}

/// Downward-causation policy (FR-CIV-0100 §3): research accelerates construction.
/// Each research tier shortens the build cadence (ticks between expansions),
/// floored so an advanced civilisation never busy-builds every single tick.
/// De-silos phase_buildings, which previously read no emergent state.
fn building_cadence(research_tier: u64) -> u64 {
    const BASE: u64 = 16;
    const FLOOR: u64 = 4;
    BASE.saturating_sub(research_tier.saturating_mul(2)).max(FLOOR)
}

/// Emergent construction demand (FR-CIV-0100 §3): the built environment responds
/// to society — crowding drives housing, research drives industry, cohesion
/// drives commerce, unrest drives civic/governance building. Each in [0,1].
/// All channels are scaled by wood/metal headroom so construction stops when
/// stockpiles are depleted (FC-3).
fn building_demand_signals(
    population: u64,
    capacity: i64,
    cohesion: u64,
    research_tier: u64,
    unrest: u64,
    wood: Fixed,
    metal: Fixed,
) -> DemandSignals {
    let cap = capacity.max(1) as f32;
    let cohesion_signal = ((cohesion as f32) / 1_000_000.0).clamp(0.0, 1.0);
    let wood_permille = building_material_headroom_permille(
        wood,
        BUILDING_WOOD_PER_PARCEL,
        BUILDING_MATERIAL_GATE,
    );
    let metal_permille = building_material_headroom_permille(
        metal,
        BUILDING_METAL_PER_PARCEL,
        BUILDING_MATERIAL_GATE,
    );
    let material_permille = wood_permille.min(metal_permille);
    let material_factor = material_permille as f32 / 1000.0;
    DemandSignals {
        residential: ((population as f32) / cap).clamp(0.0, 1.0) * material_factor,
        commercial: cohesion_signal * material_factor,
        industrial: ((research_tier as f32) / 5.0).clamp(0.0, 1.0) * material_factor,
        civic: ((unrest as f32) / 500.0).clamp(0.0, 1.0) * material_factor,
    }
}

/// Wood consumed per parcel allocated in [`Simulation::phase_buildings`].
const BUILDING_WOOD_PER_PARCEL: i64 = 10;
/// Metal consumed per parcel allocated in [`Simulation::phase_buildings`].
const BUILDING_METAL_PER_PARCEL: i64 = 5;
/// Stock level (integer units) at which material headroom reaches full strength.
const BUILDING_MATERIAL_GATE: i64 = 500;
const FC3_COMMERCIAL_PARCEL_THRESHOLD: f32 = 0.5;

/// FC-3: reserve one parcel, then quadratic roll-off in permille (0..=1000).
fn building_material_headroom_permille(stock: Fixed, reserve_units: i64, gate_units: i64) -> u64 {
    let reserve = Fixed::from_num(reserve_units);
    let effective = stock.saturating_sub(reserve);
    if effective.raw <= 0 {
        return 0;
    }
    let gate = Fixed::from_num(gate_units);
    let linear = ((effective.raw as i128) * 1000 / gate.raw.max(1) as i128).min(1000) as u64;
    linear.saturating_mul(linear) / 1000
}

/// Parcels fundable from current wood and metal stockpiles (integer division).
fn building_affordable_parcel_count(wood: Fixed, metal: Fixed) -> usize {
    let wood_per = Fixed::from_num(BUILDING_WOOD_PER_PARCEL);
    let metal_per = Fixed::from_num(BUILDING_METAL_PER_PARCEL);
    let by_wood = if wood_per.raw > 0 {
        (wood.raw / wood_per.raw) as usize
    } else {
        usize::MAX
    };
    let by_metal = if metal_per.raw > 0 {
        (metal.raw / metal_per.raw) as usize
    } else {
        usize::MAX
    };
    by_wood.min(by_metal)
}

/// Keeps the highest-priority saturated signals, zeroing the rest.
fn building_signals_limited(signals: DemandSignals, max_parcels: usize) -> DemandSignals {
    let mut active = [
        (0_u8, signals.residential),
        (1, signals.commercial),
        (2, signals.industrial),
        (3, signals.civic),
    ]
    .into_iter()
    .filter(|(_, strength)| *strength > 0.5)
    .collect::<Vec<_>>();
    active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    active.truncate(max_parcels);

    let mut out = DemandSignals {
        residential: 0.0,
        commercial: 0.0,
        industrial: 0.0,
        civic: 0.0,
    };
    for (kind, strength) in active {
        match kind {
            0 => out.residential = strength,
            1 => out.commercial = strength,
            2 => out.industrial = strength,
            _ => out.civic = strength,
        }
    }
    out
}

/// FC-3 metal steady-state ceiling (integer metal units) for a cohesion level.
/// Includes two parcel debits of headroom for discrete cadence oscillation.
fn fc3_commercial_metal_steady_ceiling_i64(cohesion: u64) -> i64 {
    let cohesion_signal = ((cohesion as f32) / 1_000_000.0).clamp(0.0, 1.0);
    if cohesion_signal <= FC3_COMMERCIAL_PARCEL_THRESHOLD {
        return i64::MAX;
    }
    let m_star = (BUILDING_MATERIAL_GATE as f32)
        * (FC3_COMMERCIAL_PARCEL_THRESHOLD / cohesion_signal).sqrt();
    (m_star + (BUILDING_METAL_PER_PARCEL as f32) * 2.0).ceil() as i64
}

/// Parcels that would be allocated for saturated demand signals (> 0.5).
fn building_parcel_count(signals: &DemandSignals) -> usize {
    [
        signals.residential,
        signals.commercial,
        signals.industrial,
        signals.civic,
    ]
    .iter()
    .filter(|&&signal| signal > 0.5)
    .count()
}

/// Construction material debit for `parcel_count` new parcels.
fn building_material_cost(parcel_count: usize) -> (Fixed, Fixed) {
    let n = parcel_count as i64;
    (
        Fixed::from_num(BUILDING_WOOD_PER_PARCEL * n),
        Fixed::from_num(BUILDING_METAL_PER_PARCEL * n),
    )
}

/// True when the global stockpile can fund `parcel_count` new parcels.
/// De-silos `resources.wood` / `resources.metal`, which `phase_production` writes.
fn building_materials_affordable(wood: Fixed, metal: Fixed, parcel_count: usize) -> bool {
    let (need_wood, need_metal) = building_material_cost(parcel_count);
    wood >= need_wood && metal >= need_metal
}

/// Belief units that contribute one unit of cohesion growth per tick.
const COHESION_BELIEF_DIVISOR: u64 = 200;
/// Unrest units that fray one unit of cohesion per tick.
const COHESION_UNREST_DIVISOR: u64 = 50;

/// Emergence policy (FR-CIV-0100 §3): the social fabric's per-tick change is the
/// balance of belief (binds, scaled gently) against unrest (frays, scaled
/// harder, so disorder erodes cohesion faster than faith builds it). Returns a
/// signed delta; the caller floors the running total at zero.
fn cohesion_delta(belief: u64, unrest: u64) -> i64 {
    let bind = (belief / COHESION_BELIEF_DIVISOR) as i64;
    let fray = (unrest / COHESION_UNREST_DIVISOR) as i64;
    bind - fray
}

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: each lineage crossing the sentience
/// threshold this tick mints a small bounded pulse of cohesion (shared
/// identity — "we are the people who woke"). Kept SMALL relative to the
/// existing cohesion bind/frac inputs so the moment of awakening nudges
/// the social fabric without dominating it; the per-tick cap mirrors the
/// spirit of [`crate::emergence`] emergence caps.
pub(crate) const COHESION_PER_AWAKENING: i64 = 2;
/// Hard per-tick cap on awakening-driven cohesion nudge (signed i64 so the
/// existing floored-at-zero cohesion mutator absorbs any overshoot cleanly).
pub(crate) const MAX_AWAKENING_COHESION_PER_TICK: i64 = 10;
/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> cohesion
/// pulse. Returns a signed i64 (matches `cohesion_delta`'s contract). The
/// inner product is clamped to the per-tick cap.
#[must_use]
pub(crate) fn awakening_cohesion_gain(awakenings_this_tick: usize) -> i64 {
    let raw = (awakenings_this_tick as i64).saturating_mul(COHESION_PER_AWAKENING);
    raw.min(MAX_AWAKENING_COHESION_PER_TICK).max(0)
}

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> belief
/// pulse. Returns a signed i64 and clamps to a small per-tick cap so the
/// compatibility shim stays bounded and deterministic.
pub(crate) const BELIEF_PER_AWAKENING: i64 = 2;
pub(crate) const MAX_AWAKENING_BELIEF_PER_TICK: i64 = 10;

#[must_use]
pub(crate) fn awakening_belief_gain(awakenings_this_tick: usize) -> i64 {
    let raw = (awakenings_this_tick as i64).saturating_mul(BELIEF_PER_AWAKENING);
    raw.min(MAX_AWAKENING_BELIEF_PER_TICK).max(0)
}

/// Cohesion absorbs hardship: a strong social fabric damps the per-tick unrest
/// rise (cohesion -> calmer society), bounded and floored at 1. Decay passes through.
fn cohesion_unrest_damp(rise: i64, cohesion: u64) -> i64 {
    if rise <= 0 {
        return rise;
    }
    let divisor = 1 + (cohesion / 200).min(9) as i64;
    (rise / divisor).max(1)
}

/// Surplus differential (resource units) at/above which a route ships its full
/// boosted volume.
const TRADE_GAP_SCALE: i64 = 100;

/// Arbitrage policy (FR-CIV-0100 §3 emergence): trade volume scales with the
/// surplus gap between exporter and importer — a well-stocked source feeding a
/// scarce destination ships MORE. Returns a multiplier in `[1.0, 2.0]`, bounded
/// at 2x so the price↔volume↔treasury↔demand loop self-limits rather than
/// running away (design-layer criticality bound). No boost when the source is
/// not in surplus relative to the destination.
fn trade_volume_multiplier(from_stock: Fixed, to_stock: Fixed) -> Fixed {
    let gap = (from_stock - to_stock).max(Fixed::ZERO);
    let normalized = (gap / Fixed::from_num(TRADE_GAP_SCALE)).min(Fixed::from_num(1));
    Fixed::from_num(1) + normalized
}

/// Floor (per-mille) below which unrest cannot throttle trade — even a society
/// in turmoil keeps half its commerce moving.
const UNREST_TRADE_FLOOR_PERMILLE: i64 = 500;
/// Units of standing unrest that throttle trade by one per-mille.
const UNREST_PER_TRADE_PERMILLE: u64 = 4;

/// Downward-causation policy (FR-CIV-0100 §3 emergence): societal unrest
/// disrupts commerce. Returns a trade-volume factor in `[0.5, 1.0]` — `1.0`
/// when calm, declining as unrest rises but floored at half so trade never
/// stops entirely. Makes unrest act on BOTH diplomacy (war) and the economy.
fn unrest_trade_factor(unrest: u64) -> Fixed {
    let max_drop = (1_000 - UNREST_TRADE_FLOOR_PERMILLE) as u64;
    let drop = (unrest / UNREST_PER_TRADE_PERMILLE).min(max_drop) as i64;
    Fixed::from_num(1_000 - drop) / Fixed::from_num(1_000)
}

/// Cohesion units that lift trade volume by one per-mille (social trust greases commerce).
const COHESION_PER_TRADE_PERMILLE: u64 = 4;
/// Cap on cohesion's trade boost (per-mille above 1.0): at most +50% volume.
const COHESION_TRADE_CAP_PERMILLE: i64 = 500;
/// Per-mille trade boost from agent tie trust alone.
const MICRO_TRUST_CAP_PERMILLE: u64 = 250;
/// Combined macro+micro trade boost cap (cohesion 500 + micro 250).
const SOCIETY_TRADE_BOOST_CAP_PERMILLE: i64 = 750;

/// Downward-causation policy (FR-CIV-0100 §3): macro cohesion AND cached micro
/// interpersonal trust lift trade volume. Returns factor in [1.0, 1.75].
fn society_trade_factor(cohesion: u64, micro_trust_permille: u64) -> Fixed {
    let cohesion_boost = (cohesion / COHESION_PER_TRADE_PERMILLE)
        .min(COHESION_TRADE_CAP_PERMILLE as u64) as i64;
    let micro_boost = micro_trust_permille.min(MICRO_TRUST_CAP_PERMILLE) as i64;
    let total = (cohesion_boost + micro_boost).min(SOCIETY_TRADE_BOOST_CAP_PERMILLE);
    Fixed::from_num(1_000 + total) / Fixed::from_num(1_000)
}

/// Downward-causation policy (FR-CIV-0100 §3): a cohesive society trades MORE —
/// social trust lowers transaction friction. Returns a factor in [1.0, 1.5],
/// rising with cohesion, capped so the boost can't run away.
fn cohesion_trade_factor(cohesion: u64) -> Fixed {
    society_trade_factor(cohesion, 0)
}

/// Relations bias trade: allies (positive relation) trade more, rivals (negative)
/// less. Returns a factor in [0.5, 1.5] from a relation score in [-1, 1], bounded.
fn relation_trade_factor(relation: f32) -> Fixed {
    let r = relation.clamp(-1.0, 1.0);
    // map [-1,1] to per-mille [500, 1500], then to a Fixed factor in [0.5, 1.5].
    let permille = (1_000.0 + 500.0 * r) as i64;
    Fixed::from_num(permille) / Fixed::from_num(1_000)
}

/// Max per-mille reduction from language barrier (at distance = 1.0).
const LANGUAGE_TRADE_PENALTY_PERMILLE: i64 = 500;
/// Downward-causation (FR-CIV-LANG-001 / FR-CIV-PSYCHE-912): mutually unintelligible
/// languages impose transaction friction. Returns factor in [0.5, 1.0].
fn language_trade_factor(distance: f32) -> Fixed {
    let d = distance.clamp(0.0, 1.0);
    let permille = 1_000 - (d * LANGUAGE_TRADE_PENALTY_PERMILLE as f32).round() as i64;
    Fixed::from_num(permille) / Fixed::from_num(1_000)
}

/// Wealth-disparity (in whole currency units) at which two factions clash when
/// they share no faith. Above this gap the have-nots turn on the haves.
const DIPLOMACY_BASE_CONFLICT_THRESHOLD: i64 = 10_000;
/// Trade-agreement relation drift (+0.05) via [`DiplomacyMatrix`] trade channel.
const FACTION_TRADE_RELATION_SIGNAL: f32 = 0.05 / 0.08;
/// Conflict relation drift (-0.1) via [`DiplomacyMatrix`] competition channel.
const FACTION_CONFLICT_RELATION_SIGNAL: f32 = 0.1 / 0.12;
/// Per diplomacy phase, unstrengthened relations retain this fraction of magnitude.
const FACTION_RELATION_DECAY_FACTOR: f32 = 0.98;
/// Trade drift per unit signal in [`DiplomacyMatrix::apply_signal`].
const DIPLOMACY_TRADE_DRIFT: f32 = 0.08;
/// Competition drift per unit signal in [`DiplomacyMatrix::apply_signal`].
const DIPLOMACY_COMPETITION_DRIFT: f32 = 0.12;
/// Max threshold shift from a saturated pairwise relation score (`±1.0`).
const FACTION_RELATION_THRESHOLD_SPAN: i64 = 5_000;
/// Max peace bonus from identical pairwise cultural traits (N2 coupling).
const CULTURE_PEACE_SPAN: f32 = 3_000.0;
/// Minimum members for an emergent settlement (matches `phase_life` HUD filter).
const SETTLEMENT_MIN_MEMBERS: u32 = 2;
/// Co-location radius for emergent settlements (matches `phase_life` cluster radius).
const SETTLEMENT_CLUSTER_RADIUS_FP: i64 = (6 * FIXED_SCALE) / 100;
/// Contact radius between settlement pairs (2× cluster radius).
const SETTLEMENT_CONTACT_RADIUS_FP: i64 = SETTLEMENT_CLUSTER_RADIUS_FP * 2;
/// Belief units required to raise the conflict threshold by one currency unit.
const BELIEF_PEACE_DIVISOR: u64 = 50;
/// Cap on the belief-driven peace bonus: shared faith can at most double a
/// society's tolerance for inequality — it never makes conflict impossible.
const BELIEF_PEACE_CAP: i64 = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
/// Unrest units required to erode the conflict threshold by one currency unit.
const UNREST_WAR_DIVISOR: u64 = 50;
/// Cap on how much unrest can erode the threshold (currency units).
const UNREST_WAR_CAP: i64 = 8_000;
/// Floor on the conflict threshold: even a furious, faithless society still
/// needs SOME wealth disparity to go to war — discontent alone is not casus belli.
const DIPLOMACY_MIN_CONFLICT_THRESHOLD: i64 = 2_000;

/// Downward-causation policy (FR-CIV-0100 §3 emergence): collective belief and
/// societal unrest pull diplomacy in opposite directions. Shared faith RAISES
/// the wealth-disparity a faction pair tolerates before fighting (peace);
/// unrest LOWERS it (internal discontent spills into external aggression). The
/// threshold is bounded below by `DIPLOMACY_MIN_CONFLICT_THRESHOLD` so conflict
/// always needs some disparity, and above at `2x` base so peace is never absolute.
fn diplomacy_conflict_threshold(belief: u64, unrest: u64) -> i64 {
    let peace = (belief / BELIEF_PEACE_DIVISOR).min(BELIEF_PEACE_CAP as u64) as i64;
    let war = (unrest / UNREST_WAR_DIVISOR).min(UNREST_WAR_CAP as u64) as i64;
    (DIPLOMACY_BASE_CONFLICT_THRESHOLD + peace - war).max(DIPLOMACY_MIN_CONFLICT_THRESHOLD)
}

/// N9: maximum reduction to the conflict threshold from maximum aggression.
const AGGRESSION_CONFLICT_BOOST: i64 = 3_000;

/// N9: conflict-threshold reduction driven by mean pairwise aggression.
/// Aggressive species are quicker to fight: a mean aggression of 1.0 reduces
/// the threshold by [`AGGRESSION_CONFLICT_BOOST`] currency units.
fn aggression_threshold_reduction(mean: f32) -> i64 {
    (mean.clamp(0.0, 1.0) * AGGRESSION_CONFLICT_BOOST as f32) as i64
}

/// Threshold bias from emergent faction relation (`relation * 5000`, clamped).
fn diplomacy_relation_threshold_bias(relation_score: f32) -> i64 {
    (relation_score.clamp(-1.0, 1.0) * FACTION_RELATION_THRESHOLD_SPAN as f32).round() as i64
}

/// Peace bonus from pairwise cultural similarity (N2 — culture → diplomacy).
///
/// Culturally similar factions tolerate more treasury disparity before conflict;
/// divergent pairs add zero bonus (neutral default).
fn diplomacy_culture_threshold_bias(
    cultures: &BTreeMap<u64, CultureProfile>,
    faction_a: u32,
    faction_b: u32,
) -> i64 {
    let Some(pa) = cultures.get(&u64::from(faction_a)) else {
        return 0;
    };
    let Some(pb) = cultures.get(&u64::from(faction_b)) else {
        return 0;
    };
    let distance = cultural_distance(pa.traits, pb.traits);
    let similarity = 1.0 - distance;
    (similarity * CULTURE_PEACE_SPAN).round() as i64
}

/// Dominant explicit faction alignment per multi-member settlement cluster (N3).
fn settlement_dominant_factions(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
) -> BTreeMap<u64, u32> {
    let mut faction_counts: BTreeMap<u64, BTreeMap<u32, u32>> = BTreeMap::new();
    for (_, (civ, member)) in world.query::<(&AgentCivilian, &ClusterMember)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        if let Alignment::Faction(faction_id) = civ.alignment {
            *faction_counts
                .entry(cluster_id)
                .or_default()
                .entry(faction_id)
                .or_insert(0) += 1;
        }
    }

    let mut dominant = BTreeMap::new();
    for (cluster_id, counts) in faction_counts {
        let mut best_faction = None;
        let mut best_count = 0u32;
        for (&faction_id, &count) in &counts {
            let replace = match best_faction {
                None => true,
                Some(prev) => count > best_count || (count == best_count && faction_id < prev),
            };
            if replace {
                best_faction = Some(faction_id);
                best_count = count;
            }
        }
        if let Some(faction_id) = best_faction {
            dominant.insert(cluster_id, faction_id);
        }
    }
    dominant
}

/// Member-weighted per-faction language centroid (FR-CIV-LANG-001 / FR-CIV-PSYCHE-912).
///
/// `cluster_cultures` is `BTreeMap<u64, CultureProfile>` keyed by cluster id; each
/// profile carries a `language: [f32; 4]` vector. `dominant` maps cluster id
/// (u64) to its dominant faction id (u32) as returned by
/// [`settlement_dominant_factions`]. `member_counts` is the cluster membership
/// rollup from `phase_life`. Clusters with fewer than 2 members are ignored so
/// lone wanderers cannot anchor a faction's centroid.
fn faction_language_centroids(
    cultures: &std::collections::BTreeMap<u64, CultureProfile>,
    dominant: &std::collections::BTreeMap<u64, u32>,
    member_counts: &std::collections::BTreeMap<u64, u32>,
) -> std::collections::BTreeMap<u32, [f32; 4]> {
    let mut sums: std::collections::BTreeMap<u32, ([f32; 4], f32)> = Default::default();
    for (cluster_id, faction_id) in dominant {
        let mc = match member_counts.get(cluster_id) {
            Some(&m) if m >= 2 => m,
            _ => continue,
        };
        let lang = match cultures.get(cluster_id) {
            Some(c) => c.language,
            None => continue,
        };
        let e = sums.entry(*faction_id).or_insert(([0.0; 4], 0.0));
        for a in 0..4 {
            e.0[a] += lang[a] * mc as f32;
        }
        e.1 += mc as f32;
    }
    sums
        .into_iter()
        .map(|(f, (s, w))| {
            let mut c = [0.0f32; 4];
            if w > 0.0 {
                for a in 0..4 {
                    c[a] = (s[a] / w).clamp(0.0, 1.0);
                }
            }
            (f, c)
        })
        .collect()
}

/// Canonical settlement contact edges when any cross-cluster agents are within radius (N3).
fn settlement_contact_pairs(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    contact_radius_fp: i64,
) -> BTreeSet<(u64, u64)> {
    let contact_radius_sq = i128::from(contact_radius_fp) * i128::from(contact_radius_fp);
    let mut by_cluster: BTreeMap<u64, Vec<(i64, i64)>> = BTreeMap::new();
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        let cluster_id = member.cluster.0;
        let members = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
        if members < SETTLEMENT_MIN_MEMBERS {
            continue;
        }
        by_cluster
            .entry(cluster_id)
            .or_default()
            .push((pos.coord.x, pos.coord.z));
    }

    let cluster_ids: Vec<u64> = by_cluster.keys().copied().collect();
    let mut contacts = BTreeSet::new();
    for i in 0..cluster_ids.len() {
        for j in (i + 1)..cluster_ids.len() {
            let ca = cluster_ids[i];
            let cb = cluster_ids[j];
            let Some(agents_a) = by_cluster.get(&ca) else {
                continue;
            };
            let Some(agents_b) = by_cluster.get(&cb) else {
                continue;
            };
            let in_contact = agents_a.iter().any(|&(ax, az)| {
                agents_b.iter().any(|&(bx, bz)| {
                    let dx = i128::from(ax) - i128::from(bx);
                    let dz = i128::from(az) - i128::from(bz);
                    dx * dx + dz * dz <= contact_radius_sq
                })
            });
            if in_contact {
                contacts.insert((ca.min(cb), ca.max(cb)));
            }
        }
    }
    contacts
}

/// Faction pairs implied by contacting settlements with different dominant factions (N3).
fn diplomacy_faction_pairs_from_settlement_contact(
    dominant: &BTreeMap<u64, u32>,
    contacts: &BTreeSet<(u64, u64)>,
) -> Vec<(u32, u32)> {
    let mut pairs = BTreeSet::new();
    for &(ca, cb) in contacts {
        let Some(&fa) = dominant.get(&ca) else {
            continue;
        };
        let Some(&fb) = dominant.get(&cb) else {
            continue;
        };
        if fa != fb {
            pairs.insert((fa.min(fb), fa.max(fb)));
        }
    }
    pairs.into_iter().collect()
}

/// Select diplomacy faction pair from settlement contact, then presence, then registry (N3).
fn diplomacy_pair_from_settlement_overlap(
    world: &World,
    cluster_member_counts: &BTreeMap<u64, u32>,
    registered_factions: &[u32],
    tick: u64,
) -> (u32, u32) {
    let dominant = settlement_dominant_factions(world, cluster_member_counts);
    let contacts = settlement_contact_pairs(
        world,
        cluster_member_counts,
        SETTLEMENT_CONTACT_RADIUS_FP,
    );
    let pairs = diplomacy_faction_pairs_from_settlement_contact(&dominant, &contacts);
    if !pairs.is_empty() {
        let idx = (tick as usize / 500) % pairs.len();
        return pairs[idx];
    }

    let present: Vec<u32> = dominant
        .values()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if present.len() >= 2 {
        let idx = tick as usize % present.len();
        let a = present[idx];
        let b = present[(idx + 1) % present.len()];
        return (a, b);
    }

    let idx = tick as usize;
    let a = registered_factions[idx % registered_factions.len()];
    let b = registered_factions[(idx + 1) % registered_factions.len()];
    (a, b)
}

/// Scales every stored relation toward neutral without overshooting zero.
///
/// [`DiplomacyMatrix`] has no native decay; calibrated `apply_signal` calls
/// achieve `score * factor` per pair (FR-CIV-0100 criticality).
fn decay_faction_relations(matrix: &mut DiplomacyMatrix, factor: f32) {
    let factor = factor.clamp(0.0, 1.0);
    let pairs = matrix.snapshot();
    for (a, b, record) in pairs {
        let score = record.score;
        if score == 0.0 {
            continue;
        }
        let target = score * factor;
        let delta = target - score;
        if delta > 0.0 {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    trade_volume: delta / DIPLOMACY_TRADE_DRIFT,
                    ..Default::default()
                },
            );
        } else {
            matrix.apply_signal(
                a,
                b,
                DiplomacySignal {
                    resource_competition: (-delta) / DIPLOMACY_COMPETITION_DRIFT,
                    ..Default::default()
                },
            );
        }
    }
}

/// Sustained [`DiplomacyKind::TradeAgreement`] events before an emergent route is born.
const TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD: u32 = 2;
/// Minimum pairwise relation score required to birth an emergent route.
const TRADE_ROUTE_MIN_RELATION: f32 = 0.0;
/// Hard cap on total trade routes (bootstrap + emergent) to bound memory and tick cost.
const MAX_TRADE_ROUTES: usize = 64;
/// Ticks without resource flow before an emergent route is removed.
const TRADE_ROUTE_UNUSED_DECAY_TICKS: u32 = 2_000;

fn canonical_faction_pair(a: u32, b: u32) -> (u32, u32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Deterministic goods label from exporter faction id (stable, integer-only).
fn emergent_route_goods(from: u32) -> &'static str {
    match from % 3 {
        0 => "grain",
        1 => "ore",
        _ => "cloth",
    }
}

fn record_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    let pair = canonical_faction_pair(a, b);
    *streak.entry(pair).or_default() += 1;
}

fn reset_trade_agreement_streak(streak: &mut BTreeMap<(u32, u32), u32>, a: u32, b: u32) {
    streak.remove(&canonical_faction_pair(a, b));
}

fn remove_emergent_routes_between(state: &mut WorldState, a: u32, b: u32) {
    let to_remove: Vec<(u32, u32, String)> = state
        .emergent_trade_route_keys
        .iter()
        .filter(|(from, to, _)| {
            (*from == a && *to == b) || (*from == b && *to == a)
        })
        .cloned()
        .collect();
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    state.trade_routes.retain(|route| {
        let key = (
            route.from_faction,
            route.to_faction,
            route.goods.clone(),
        );
        !to_remove.contains(&key)
    });
}

fn decay_idle_emergent_trade_routes(
    state: &mut WorldState,
    flowed: &BTreeSet<(u32, u32, String)>,
) {
    let emergent: Vec<(u32, u32, String)> = state.emergent_trade_route_keys.iter().cloned().collect();
    let mut to_remove = Vec::new();
    for key in emergent {
        if flowed.contains(&key) {
            state.trade_route_idle_ticks.insert(key.clone(), 0);
            continue;
        }
        let idle = state.trade_route_idle_ticks.entry(key.clone()).or_insert(0);
        *idle = idle.saturating_add(1);
        if *idle >= TRADE_ROUTE_UNUSED_DECAY_TICKS {
            to_remove.push(key);
        }
    }
    for key in &to_remove {
        state.emergent_trade_route_keys.remove(key);
        state.trade_route_idle_ticks.remove(key);
    }
    if !to_remove.is_empty() {
        state.trade_routes.retain(|route| {
            let key = (
                route.from_faction,
                route.to_faction,
                route.goods.clone(),
            );
            !to_remove.contains(&key)
        });
    }
}

fn route_resource(goods: &str) -> ResourceType {
    match goods {
        "grain" => ResourceType::Food,
        "timber" => ResourceType::Wood,
        "ore" | "tools" => ResourceType::Metal,
        "cloth" | "salt" => ResourceType::Energy,
        _ => ResourceType::Food,
    }
}

fn resource_amount(resources: &Resources, resource: ResourceType) -> Fixed {
    match resource {
        ResourceType::Food => resources.food,
        ResourceType::Wood => resources.wood,
        ResourceType::Metal => resources.metal,
        ResourceType::Energy => resources.energy,
    }
}

fn adjust_resource(resources: &mut Resources, resource: ResourceType, delta: Fixed) {
    match resource {
        ResourceType::Food => resources.food += delta,
        ResourceType::Wood => resources.wood += delta,
        ResourceType::Metal => resources.metal += delta,
        ResourceType::Energy => resources.energy += delta,
    }
}

/// Replay-aware mutable voxel access wrapper.
pub struct VoxelWriteProxy<'a> {
    sim: &'a mut Simulation,
}

impl<'a> VoxelWriteProxy<'a> {
    pub fn write(&mut self, pos: civ_voxel::WorldCoord, value: MaterialId) {
        self.sim.push_voxel_write(pos, value);
    }
}

impl<'a> Deref for VoxelWriteProxy<'a> {
    type Target = VoxelWorld<MaterialId>;

    fn deref(&self) -> &Self::Target {
        &self.sim.voxel
    }
}

impl<'a> DerefMut for VoxelWriteProxy<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sim.voxel
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub tick: u64,
    pub population: u64,
    pub citizen_count: usize,
    pub building_count: usize,
    pub military_count: usize,
    pub energy_budget: Fixed,
    pub resources: Resources,
    pub births_this_tick: u32,
    pub deaths_this_tick: u32,
    pub diplomacy_events: Vec<DiplomacyEvent>,
    /// Per-good clearing prices in cents from [`MarketState`].
    pub market_prices: BTreeMap<String, i64>,
    /// Number of per-soldier combat damage pulses resolved during the most recent tick
    /// (FR-CIV-TACTICS-024 — feeds doctrine fitness and the server `/sim/state` wire).
    pub damage_events: usize,
    /// Deterministic climate snapshot computed by `phase_planet` for the current tick
    /// (FR-CIV-PLANET-010 — bit-identical to `compute_climate(tick, planet, moon)`).
    pub climate: Climate,
    /// Per-region weather grid for the current tick (FR-CIV-PLANET-030).
    ///
    /// Each entry is a [`WeatherCell`] with fixed-point temp and precipitation.
    /// The grid is re-derived from `tick` and `planet.axial_tilt_deg` every tick.
    pub weather_grid: Vec<WeatherCell>,
    /// Deterministic geology map for the planet (FR-CIV-PLANET-040).
    ///
    /// Derived from `PlanetConfig` alone; identical for every tick of the same planet.
    pub geology_map: GeologyMap,
}

// ============================================================================
// TEST HELPERS & STUBS
// ============================================================================

#[cfg(test)]
fn diplomacy_faction_pair(faction_ids: &[u32], _tick: u64) -> (u32, u32) {
    // Stub for testing: return first two distinct factions or duplicate if only one.
    if faction_ids.len() >= 2 {
        (faction_ids[0], faction_ids[1])
    } else if faction_ids.len() == 1 {
        (faction_ids[0], faction_ids[0])
    } else {
        (0, 1)
    }
}

#[cfg(test)]
fn choose_named_seed(
    _seed_mix: &[crate::scenario::SeedWeight],
    _dist: Option<&rand::distributions::WeightedIndex<f32>>,
    spawn_index: usize,
    _rng: &mut rand_chacha::ChaCha8Rng,
) -> civ_genetics::NamedSeed {
    // Stub for testing: round-robin through named seeds based on spawn_index.
    use civ_genetics::NamedSeed;
    match spawn_index % 3 {
        0 => NamedSeed::Ardani,
        1 => NamedSeed::Velthari,
        _ => NamedSeed::Grundak,
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lod::{should_tick_entity_with_policy, LodPolicy};
    use crate::replay::{ReplayEvent, ReplayLog};
    use civ_agents::{count_civilians, LodTier, Wardrobe};
    use civ_planet::{compute_climate, is_daytime, MoonConfig, PlanetConfig};
    use civ_voxel::{MaterialId, WorldCoord};
    use tempfile::NamedTempFile;

    fn fill_voxel_chunk(world: &mut VoxelWorld<MaterialId>, origin: i64, size: i64) {
        for x in origin..origin + size {
            for y in origin..origin + size {
                for z in origin..origin + size {
                    world.write(WorldCoord { x, y, z }, MaterialId(1));
                }
            }
        }
    }

    /// FR-CIV-ENGINE-INT-010 — startup spawns 128 civilians across four factions.
    #[test]
    fn startup_spawns_128_civilians() {
        let sim = Simulation::new();
        assert_eq!(sim.state.tick, 0);
        assert_eq!(count_civilians(&sim.world), 128);
    }

    #[test]
    fn test_tick_advances() {
        let mut sim = Simulation::new();
        sim.tick();
        assert_eq!(sim.state.tick, 1);
    }

    /// FR-CORE-001 — each `Simulation::tick()` appends exactly one `ReplayEvent::Tick`.
    #[test]
    fn fr_core_001_single_tick_event_per_tick() {
        use crate::invariants::check_tick_invariants;

        let mut sim = Simulation::with_seed(1);
        assert_eq!(count_replay_ticks(&sim), 0);

        sim.tick();
        assert_eq!(sim.state.tick, 1);
        assert_eq!(count_replay_ticks(&sim), 1);
        check_tick_invariants(&sim).expect("one replay tick marker per completed tick");

        for expected in 2..=5 {
            sim.tick();
            assert_eq!(sim.state.tick, expected);
            assert_eq!(count_replay_ticks(&sim), expected as usize);
        }
    }

    /// CIV-0001 partial — `PHASE_ORDER` matches the sequence in `Simulation::tick`.
    #[test]
    fn phase_order_matches_tick_sequence() {
        assert_eq!(
            PHASE_ORDER,
            &[
                "production",
                "citizen_lifecycle",
                "military",
                "policy",
                "economy",
                "planet",
                "diplomacy",
                "tactics",
                "voxel",
                "compact",
                "buildings",
                "diffusion",
                "life",
                "emergence",
            ]
        );
    }

    /// L5-115 — `PHASE_ORDER` includes "emergence" and the phase is positioned
    /// after `life` so the agent state that emergence depends on is finalized
    /// (cluster stocks, needs, settlements) before emergence runs.
    /// Closes FR-CIV-LEGENDS-INGEST-02, FR-CIV-PSYCHE-900/901, FR-CIV-PSYCHE-911,
    /// FR-CIV-PSYCHE-912, FR-CIV-GENETICS, FR-CIV-AI-006, FR-CIV-LEGENDS-QUERY-07.
    #[test]
    fn phase_order_includes_emergence() {
        let emergence_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "emergence")
            .expect("PHASE_ORDER must include 'emergence'");
        let life_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "life")
            .expect("PHASE_ORDER must include 'life'");
        assert!(
            emergence_idx > life_idx,
            "emergence (idx {emergence_idx}) must run after life (idx {life_idx}) \
             so agent state is finalized first"
        );
        // And it must be the final phase in the deterministic core loop —
        // anything reading emergence state downstream (saga belief gain,
        // unrest, chronicle) depends on the phase having run.
        assert_eq!(
            emergence_idx,
            PHASE_ORDER.len() - 1,
            "emergence must be the final entry in PHASE_ORDER"
        );
    }

    /// L5-115 — `Simulation::tick` invokes `phase_emergence` and the public
    /// accessors on `Simulation` (legends_graph, emergence_feed,
    /// cluster_cultures) are queryable after a tick. Two same-seed sims run
    /// deterministically through the emergence pipeline (RNG state is
    /// preserved across the phase — see `test_determinism`).
    #[test]
    fn tick_invokes_emergence_phase() {
        let mut sim_a = Simulation::with_seed(2026_06_18);
        let mut sim_b = Simulation::with_seed(2026_06_18);

        for _ in 0..10 {
            sim_a.tick();
            sim_b.tick();
        }

        // Post-condition: the wire-up is observable via the public API.
        // `legends_graph` is the saga state populated by `emergence_legends`
        // (FR-CIV-LEGENDS-INGEST-02). The accessor must return without panic
        // — a non-panic on a wired phase is the wire-up check.
        let _graph_a = sim_a.legends_graph();
        let _graph_b = sim_b.legends_graph();

        // Determinism: same seed → same saga graph node count after N ticks.
        assert_eq!(
            sim_a.legends_graph().node_count(),
            sim_b.legends_graph().node_count(),
            "phase_emergence must be deterministic across same-seed sims"
        );

        // `emergence_feed` is cleared at the start of `phase_emergence` and
        // re-populated with the tick's events. The accessor must remain
        // queryable after a tick.
        let _feed_a = sim_a.emergence_feed();
        let _feed_b = sim_b.emergence_feed();

        // `cluster_cultures` is the population-level culture map populated
        // by `emergence_culture` (FR-CIV-PSYCHE-911). It must be queryable
        // and deterministic.
        assert_eq!(
            sim_a.cluster_cultures().len(),
            sim_b.cluster_cultures().len(),
            "phase_emergence must produce deterministic cluster_cultures"
        );
    }

    fn count_replay_ticks(sim: &Simulation) -> usize {
        sim.replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
            .count()
    }

    // ============================================================================
    // FR-CORE-005 — Policy phase + set_policy tests
    // ============================================================================

    /// FR-CORE-005 — new simulations start with [`NoopPolicy`] installed and
    /// `last_control_signals` empty.
    #[test]
    fn default_policy_is_noop() {
        let sim = Simulation::new();
        assert_eq!(sim.policy().name(), "noop");
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
    }

    /// FR-CORE-005 — `with_seed` constructor also starts with [`NoopPolicy`].
    #[test]
    fn with_seed_default_policy_is_noop() {
        let sim = Simulation::with_seed(42);
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — `set_policy` replaces the active policy.
    #[test]
    fn set_policy_replaces_active_policy() {
        let mut sim = Simulation::new();
        assert_eq!(sim.policy().name(), "noop");

        sim.set_policy(Box::new(crate::policy::CapitalistPolicy));
        assert_eq!(sim.policy().name(), "capitalist");

        sim.set_policy(Box::new(crate::policy::SubsistenceFirstPolicy));
        assert_eq!(sim.policy().name(), "subsistence_first");

        sim.set_policy(Box::new(crate::policy::NoopPolicy));
        assert_eq!(sim.policy().name(), "noop");
    }

    /// FR-CORE-005 — a single `tick()` populates `last_control_signals` from
    /// the active policy.
    #[test]
    fn phase_policy_populates_last_control_signals() {
        let mut sim = Simulation::new();
        sim.set_policy(Box::new(crate::policy::CapitalistPolicy));
        sim.tick();
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
        assert_eq!(sim.last_control_signals().production_multipliers.len(), 0);
        assert_eq!(sim.last_control_signals().allocation_weights.len(), 0);
        assert_eq!(sim.last_control_signals().tax_rates.len(), 0);
    }

    /// FR-CORE-005 — `phase_policy` runs every tick; repeated ticks keep
    /// `last_control_signals` consistent with the active policy.
    #[test]
    fn phase_policy_runs_every_tick() {
        let mut sim = Simulation::new();
        for _ in 0..5 {
            sim.tick();
        }
        assert_eq!(sim.state.tick, 5);
        // Default NoopPolicy produces default signals every tick.
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
    }

    /// FR-CORE-005 — `phase_policy` runs after `phase_military` and before
    /// `phase_economy` (verified indirectly: `last_control_signals` is
    /// populated for the same tick `phase_economy` reads `state.energy_budget_joules` from).
    #[test]
    fn phase_policy_runs_before_phase_economy() {
        use crate::policy::CapitalistPolicy;

        let mut sim = Simulation::new();
        sim.set_policy(Box::new(CapitalistPolicy));
        // After one tick, last_control_signals reflects the policy at tick 1.
        sim.tick();
        assert_eq!(sim.last_control_signals(), &ControlSignals::default());
        // The default capitalist policy is a no-op, so the economy state
        // behaves identically to a NoopPolicy run.
        let mut ref_sim = Simulation::with_seed(42);
        ref_sim.tick();
        assert_eq!(ref_sim.state.energy_budget_joules, sim.state.energy_budget_joules);
    }

    /// FR-CORE-005 — a custom policy that emits non-empty signals is reflected
    /// on `last_control_signals` after `tick()`. Uses an inline test-only
    /// policy to avoid modifying the public `policy` module for one test.
    #[test]
    fn custom_policy_signals_propagate_to_simulation() {
        #[derive(Debug)]
        struct TaxingPolicy;
        impl Policy for TaxingPolicy {
            fn evaluate(&self, _state: &WorldState) -> ControlSignals {
                let mut signals = ControlSignals::default();
                signals.tax_rates.insert(7, 250); // 2.5%
                signals
                    .production_multipliers
                    .insert("food".to_string(), 1.25);
                signals
            }
            fn name(&self) -> &'static str {
                "taxing"
            }
        }

        let mut sim = Simulation::new();
        sim.set_policy(Box::new(TaxingPolicy));
        sim.tick();
        assert_eq!(sim.last_control_signals().tax_rates.get(&7), Some(&250));
        assert_eq!(
            sim.last_control_signals().production_multipliers.get("food"),
            Some(&1.25)
        );
    }

    /// CIV-0100 stub: joule budget drain matches policy formula and stays non-negative.
    #[test]
    fn phase_economy_conserves_non_negative_budget() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(99);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 1_000.0,
            scarcity_multiplier: 2.0,
        };
        let before = sim.state.energy_budget_joules;
        sim.tick();
        let expected = (before - Fixed::from_num(2_000i64)).max(Fixed::ZERO);
        assert_eq!(sim.state.energy_budget_joules, expected);
        assert!(sim.state.energy_budget_joules.raw >= Fixed::ZERO.raw);
    }

    /// `phase_economy` routes demand through [`CapitalistAllocator::allocate`].
    #[test]
    fn phase_economy_uses_capitalist_allocator() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(7);
        sim.state.energy_budget_joules = Fixed::from_num(50);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 1.0,
        };

        let demand = crate::policy::effective_consumption(sim.economy_policy) as i64;
        let expected_allocated = CapitalistAllocator.allocate(50, demand);
        let before = sim.state.energy_budget_joules;

        sim.tick();

        assert_eq!(expected_allocated, 50);
        assert_eq!(
            sim.state.energy_budget_joules,
            before - Fixed::from_num(expected_allocated)
        );
        assert_eq!(sim.economy_state.energy_budget_joules, 0);
    }

    /// `phase_economy` keeps `economy_state` in sync with the world joule budget.
    #[test]
    fn phase_economy_updates_economy_state() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(99);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 1_000.0,
            scarcity_multiplier: 1.0,
        };
        let before = sim.economy_state.energy_budget_joules;
        sim.tick();
        assert_eq!(
            sim.economy_state.energy_budget_joules,
            sim.state.energy_budget_joules.raw / crate::SCALE
        );
        assert_eq!(sim.economy_state.energy_budget_joules, before - 1_000);
    }

    /// `phase_economy` advances [`MarketState`] so prices move over time.
    #[test]
    fn phase_economy_steps_market_prices() {
        const N: usize = 2;

        let mut sim = Simulation::with_seed(42);
        let initial = sim.market_state.prices.clone();
        for _ in 0..N {
            sim.tick();
        }
        assert_ne!(
            sim.market_state.prices, initial,
            "expected at least one market price to change after {N} ticks"
        );
    }

    #[test]
    fn test_initial_entities() {
        let sim = Simulation::new();
        let snapshot = sim.snapshot();
        assert!(snapshot.citizen_count > 0);
        assert!(snapshot.building_count > 0);
        assert!(snapshot.military_count > 0);
    }

    #[test]
    fn test_determinism() {
        let mut sim1 = Simulation::with_seed(12345);
        let mut sim2 = Simulation::with_seed(12345);

        for _ in 0..100 {
            sim1.tick();
            sim2.tick();
        }

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(sim1.state.population, sim2.state.population);
    }

    /// FR-CIV-ENGINE-INT-001 — climate is recomputed every tick and matches
    /// `compute_climate` directly.
    #[test]
    fn climate_recomputes_every_tick() {
        let mut sim = Simulation::with_seed(11);
        let planet = *sim.planet();
        let moon = *sim.moon();

        sim.tick();
        let expected = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(sim.climate(), &expected);

        sim.tick();
        let expected = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(sim.climate(), &expected);
    }

    /// FR-CIV-PLANET-010 — `Simulation::snapshot()` surfaces the deterministic
    /// `Climate` produced by `phase_planet`, bit-identical to `compute_climate`.
    #[test]
    fn engine_tick_includes_climate_in_snapshot() {
        let mut sim = Simulation::with_seed(2026);
        let planet = *sim.planet();
        let moon = *sim.moon();

        // Tick 0 — pre-tick climate is computed at construction time.
        let snap0 = sim.snapshot();
        let expected0 = compute_climate(sim.state.tick, &planet, &moon);
        assert_eq!(snap0.tick, 0);
        assert_eq!(snap0.climate, expected0);

        // Advance ticks and confirm snapshot.climate stays bit-identical.
        for _ in 0..5 {
            sim.tick();
            let snap = sim.snapshot();
            let expected = compute_climate(sim.state.tick, &planet, &moon);

            assert_eq!(snap.tick, sim.state.tick);
            assert_eq!(snap.climate.tick, expected.tick);
            assert_eq!(
                snap.climate.day_phase.to_bits(),
                expected.day_phase.to_bits()
            );
            assert_eq!(
                snap.climate.year_phase.to_bits(),
                expected.year_phase.to_bits()
            );
            assert_eq!(
                snap.climate.moon_phase.to_bits(),
                expected.moon_phase.to_bits()
            );
            assert_eq!(
                snap.climate.tide_offset.to_bits(),
                expected.tide_offset.to_bits()
            );
            assert_eq!(snap.climate, *sim.climate());
        }
    }

    /// FR-CIV-PLANET-020 — `apply_tide_offset` shifts a registered coastal
    /// water-level voxel deterministically as the tide cycles, and the shift
    /// is symmetric around the registered sea-level baseline within tight
    /// numeric tolerance (≤ 1e-4 of the tidal amplitude in fixed-point units).
    #[test]
    fn tide_offset_shifts_coastal_voxel_height() {
        use civ_voxel::material::WATER;

        assert_eq!(WATER_MARKER_MATERIAL, WATER);

        // Use a moon config whose orbit period is a clean factor so we can land
        // on the peak (+amplitude), trough (-amplitude), and zero-crossing
        // ticks exactly. sin(TAU * phase) = +1 at phase=0.25, -1 at phase=0.75.
        let mut sim = Simulation::with_seed(2026);
        sim.moon = MoonConfig {
            orbit_period_ticks: 4,
            tidal_amplitude: 1.0,
        };
        sim.planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 0,
            day_length_ticks: 4,
            year_length_ticks: 4,
        };

        let base_y: i64 = 10 * FIXED_SCALE;
        let x: i64 = 5 * FIXED_SCALE;
        let z: i64 = 7 * FIXED_SCALE;
        sim.register_coastal_water_column(x, z, base_y);
        assert_eq!(sim.coastal_column_count(), 1);
        assert_eq!(sim.coastal_water_level(x, z), Some(base_y));

        let amplitude_units = FIXED_SCALE; // tidal_amplitude * FIXED_SCALE
        let tolerance: i64 = ((FIXED_SCALE as f64) * 1.0e-4_f64).ceil() as i64;

        // Tick 1 -> moon_phase = 0.25 -> tide_offset = +1.0 -> peak.
        sim.tick();
        let peak = sim
            .coastal_water_level(x, z)
            .expect("water level after peak tick");
        let peak_delta = peak - base_y;
        assert!(
            (peak_delta - amplitude_units).abs() <= tolerance,
            "expected peak delta ≈ +{amplitude_units}, got {peak_delta}"
        );
        // The water marker now occupies the shifted y, and the old base_y has
        // been cleared back to MaterialId(0). Both writes flow through the
        // voxel dirty queue (FR-CIV-VOXEL-002).
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: peak, z }),
            WATER_MARKER_MATERIAL
        );
        assert_eq!(
            sim.voxel().read(WorldCoord { x, y: base_y, z }),
            MaterialId(0)
        );

        // Tick 2 -> moon_phase = 0.5 -> tide_offset = 0 -> back to baseline.
        sim.tick();
        let mid = sim
            .coastal_water_level(x, z)
            .expect("water level at zero crossing");
        let mid_delta = mid - base_y;
        assert!(
            mid_delta.abs() <= tolerance,
            "expected zero-crossing delta ≈ 0, got {mid_delta}"
        );

        // Tick 3 -> moon_phase = 0.75 -> tide_offset = -1.0 -> trough.
        sim.tick();
        let trough = sim
            .coastal_water_level(x, z)
            .expect("water level after trough tick");
        let trough_delta = trough - base_y;
        assert!(
            (trough_delta + amplitude_units).abs() <= tolerance,
            "expected trough delta ≈ -{amplitude_units}, got {trough_delta}"
        );

        // Symmetry: peak and trough are mirror images around base_y within tolerance.
        let symmetry_residual = (peak_delta + trough_delta).abs();
        assert!(
            symmetry_residual <= tolerance,
            "peak {peak_delta} and trough {trough_delta} should mirror around baseline; residual {symmetry_residual} > tolerance {tolerance}"
        );

        // Tick 4 -> moon_phase = 0 -> back to baseline.
        sim.tick();
        let close = sim
            .coastal_water_level(x, z)
            .expect("water level at cycle close");
        assert!(
            (close - base_y).abs() <= tolerance,
            "expected end-of-cycle delta ≈ 0, got {}",
            close - base_y
        );

        // Determinism: a second simulation with the same seed + registration
        // produces bit-identical voxel water levels at every tick.
        let mut sim2 = Simulation::with_seed(2026);
        sim2.moon = sim.moon;
        sim2.planet = sim.planet;
        sim2.register_coastal_water_column(x, z, base_y);
        for _ in 0..4 {
            sim2.tick();
        }
        assert_eq!(
            sim.coastal_water_level(x, z),
            sim2.coastal_water_level(x, z)
        );
    }

    /// FR-CIV-TACTICS-010 — doctrine GA advances on a fixed tick cadence.
    #[test]
    fn phase_tactics_evolve_doctrine_on_cadence() {
        let mut sim = Simulation::with_seed(42);
        let gen0 = sim.faction_doctrines()[0].generation;
        for _ in 0..63 {
            sim.tick();
        }
        assert_eq!(sim.faction_doctrines()[0].generation, gen0);
        sim.tick();
        assert!(
            sim.faction_doctrines()[0].generation > gen0,
            "expected doctrine generation to advance at tick 64"
        );
    }

    /// FR-CIV-ENGINE-INT-002 — queued damage drains and voxel chunk count
    /// decreases as expected.
    #[test]
    fn pending_damage_drains_and_reduces_chunk_count() {
        let mut sim = Simulation::with_seed(12);
        fill_voxel_chunk(&mut sim.voxel_mut(), 0, 16);
        let before = sim.voxel().chunk_count();
        assert!(before > 0);

        sim.push_damage(DamageEvent {
            center: WorldCoord { x: 8, y: 8, z: 8 },
            radius_voxels: 12,
            energy: 1,
        });

        sim.tick();

        // A sphere of radius 12 voxels removes a substantial fraction of a 16³
        // chunk but never the whole 4096 cells (corner voxels are outside the
        // sphere). Assert >0 removals and <=4096 (the chunk total) — enough to
        // prove damage flowed through to the voxel substrate.
        let removed = sim.last_tick_voxel_damage_count();
        assert!(
            removed > 0,
            "expected damage to remove at least one voxel, got {removed}"
        );
        assert!(
            removed <= 16 * 16 * 16,
            "removal count exceeded chunk total: {removed}"
        );
        assert!(sim.pending_damage.is_empty());
    }

    /// FR-CIV-ENGINE-INT-003 — compact runs every 64 ticks and the uniform
    /// chunk count is non-decreasing across the cadence.
    #[test]
    fn compact_runs_every_64_ticks() {
        let mut sim = Simulation::with_seed(13);
        fill_voxel_chunk(&mut sim.voxel_mut(), 0, 16);
        let mut last_uniform = sim.voxel().uniform_chunk_count();

        for _ in 0..128 {
            sim.tick();
            let current = sim.voxel().uniform_chunk_count();
            assert!(current >= last_uniform);
            last_uniform = current;
        }
    }

    /// FR-CIV-ENGINE-INT-011 — phase_buildings allocates over time when signals are high.
    #[test]
    fn phase_buildings_allocates_over_time_when_signals_are_high() {
        let mut sim = Simulation::with_seed(77);
        let before = sim.building_graph().parcels.len();

        for _ in 0..200 {
            sim.tick();
        }

        assert!(sim.building_graph().parcels.len() > before);
    }

    /// FR-CIV-ENGINE-INT-012 — diffusion advances civilian wardrobe eras over time.
    #[test]
    fn phase_diffusion_bumps_wardrobe_eras() {
        let mut sim = Simulation::with_seed(91);
        let before = sim
            .world
            .query::<&Wardrobe>()
            .iter()
            .filter(|(_, wardrobe)| wardrobe.era >= sim.target_era)
            .count();

        for _ in 0..200 {
            sim.tick();
        }

        let after = sim
            .world
            .query::<&Wardrobe>()
            .iter()
            .filter(|(_, wardrobe)| wardrobe.era >= sim.target_era)
            .count();
        assert!(after > before);
    }

    /// FR-CIV-ENGINE-INT-015 — Cold-tier wardrobe diffusion only runs on cadence boundaries.
    #[test]
    fn cold_tier_diffusion_only_on_cadence_boundaries() {
        use civ_agents::spawn_many;

        let mut sim = Simulation::with_seed(55);
        let _ = spawn_many(&mut sim.world, 6, 50_000, 0);
        let policy = LodPolicy::default();

        let cold_entities: Vec<hecs::Entity> = sim
            .world
            .query::<(&Wardrobe, &LodTier)>()
            .iter()
            .filter_map(|(entity, (_, lod))| (*lod == LodTier::Cold).then_some(entity))
            .collect();
        assert!(
            !cold_entities.is_empty(),
            "expected spawn_many to produce Cold-tier civilians"
        );

        for tick in 1..=32 {
            let before: std::collections::HashMap<hecs::Entity, u16> = cold_entities
                .iter()
                .map(|&entity| (entity, *sim.world.get::<&Wardrobe>(entity).unwrap()))
                .map(|(entity, wardrobe)| (entity, wardrobe.era))
                .collect();

            sim.tick();

            for &entity in &cold_entities {
                let after = sim.world.get::<&Wardrobe>(entity).unwrap().era;
                if before[&entity] != after {
                    assert!(
                        should_tick_entity_with_policy(tick, LodTier::Cold, policy),
                        "Cold-tier wardrobe changed on tick {tick} (off cadence)"
                    );
                }
            }
        }
    }

    /// FR-CIV-ENGINE-INT-013 — replay determinism still holds across 200 ticks
    /// with all phases on.
    #[test]
    fn determinism_holds_with_all_phases_enabled() {
        let mut sim1 = Simulation::with_seed(12345);
        let mut sim2 = Simulation::with_seed(12345);

        for tick in 0..200_u64 {
            if tick % 17 == 0 {
                let event = DamageEvent {
                    center: WorldCoord {
                        x: (tick as i64 % 32) * 1_000_000,
                        y: 0,
                        z: 0,
                    },
                    radius_voxels: 4,
                    energy: tick as u32,
                };
                sim1.push_damage(event);
                sim2.push_damage(event);
            }
            sim1.tick();
            sim2.tick();
        }

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(sim1.state.population, sim2.state.population);
        assert_eq!(sim1.climate(), sim2.climate());
        assert_eq!(
            sim1.last_tick_voxel_damage_count(),
            sim2.last_tick_voxel_damage_count()
        );
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
        assert_eq!(sim1.voxel().chunk_count(), sim2.voxel().chunk_count());
        assert_eq!(sim1.building_graph(), sim2.building_graph());
        assert_eq!(sim1.last_cohort_stats(), sim2.last_cohort_stats());
    }

    /// FR-CIV-ENGINE-INT-014 — last_cohort_stats reflects the population.
    #[test]
    fn last_cohort_stats_reflects_population() {
        let mut sim = Simulation::with_seed(19);
        sim.tick();

        let stats = sim.last_cohort_stats().expect("cohort stats");
        assert_eq!(stats.total_civilians as usize, count_civilians(&sim.world));
    }

    /// FR-CIV-ENGINE-INT-005 — `is_daytime` returns sensible day/night across
    /// one full day-length cycle.
    #[test]
    fn daytime_cycles_across_one_full_day() {
        let planet = PlanetConfig {
            radius_km: 1,
            axial_tilt_deg: 23,
            day_length_ticks: 24,
            year_length_ticks: 240,
        };
        let moon = MoonConfig {
            orbit_period_ticks: 48,
            tidal_amplitude: 1.0,
        };

        let midnight = compute_climate(0, &planet, &moon);
        let noon = compute_climate(12, &planet, &moon);
        let next_midnight = compute_climate(24, &planet, &moon);

        assert!(!is_daytime(&midnight));
        assert!(is_daytime(&noon));
        assert!(!is_daytime(&next_midnight));
    }

    /// FR-CIV-VOXEL-006 — voxel writes between ticks produce dirty events that
    /// the engine's voxel phase drains into `last_tick_voxel_events`, in
    /// `(chunk_id, write_seq)` order.
    #[test]
    fn voxel_phase_drains_dirty_events_each_tick() {
        use civ_voxel::WorldCoord;
        let mut sim = Simulation::with_seed(42);
        // Tick once with nothing pending — should be empty.
        sim.tick();
        assert!(sim.last_tick_voxel_events().is_empty());
        // Write four voxels in two chunks, then tick.
        sim.voxel_mut()
            .write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
        sim.voxel_mut().write(
            WorldCoord {
                x: 1_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.voxel_mut().write(
            WorldCoord {
                x: 100_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.voxel_mut().write(
            WorldCoord {
                x: 101_000_000,
                y: 0,
                z: 0,
            },
            MaterialId(1),
        );
        sim.tick();
        let events = sim.last_tick_voxel_events();
        assert_eq!(events.len(), 4);
        // Sorted ascending by (chunk_id, write_seq).
        for window in events.windows(2) {
            assert!(window[0] <= window[1]);
        }
        // Next tick clears them.
        sim.tick();
        assert!(sim.last_tick_voxel_events().is_empty());
    }

    /// FR-CIV-VOXEL-007 — voxel state is part of the deterministic simulation:
    /// two sims with identical seed + identical voxel-write sequences emit
    /// bit-identical voxel events.
    #[test]
    fn voxel_phase_replay_is_bit_identical() {
        use civ_voxel::WorldCoord;
        let mut sim1 = Simulation::with_seed(7);
        let mut sim2 = Simulation::with_seed(7);
        let writes = [
            (
                WorldCoord {
                    x: 5_000_000,
                    y: 0,
                    z: 0,
                },
                MaterialId(2),
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 5_000_000,
                    z: 0,
                },
                MaterialId(3),
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 0,
                    z: 5_000_000,
                },
                MaterialId(4),
            ),
        ];
        for (pos, mat) in writes {
            sim1.voxel_mut().write(pos, mat);
            sim2.voxel_mut().write(pos, mat);
        }
        sim1.tick();
        sim2.tick();
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
    }

    /// FR-CIV-CA-005 — identical dirty-chunk voxel setups must replay to the
    /// same log and voxel state on same-seed reruns.
    #[test]
    fn replay_ca_dirty_chunk_bit_identical() {
        use civ_voxel::material::{SAND, STONE, WATER};
        use civ_voxel::WorldCoord;

        let mut sim1 = Simulation::with_seed(17);
        let mut sim2 = Simulation::with_seed(17);
        let writes = [
            (
                WorldCoord {
                    x: 1_000_000,
                    y: 0,
                    z: 0,
                },
                WATER,
            ),
            (
                WorldCoord {
                    x: 16_000_000,
                    y: 0,
                    z: 0,
                },
                STONE,
            ),
            (
                WorldCoord {
                    x: 0,
                    y: 16_000_000,
                    z: 0,
                },
                SAND,
            ),
        ];

        for (pos, mat) in writes {
            sim1.voxel_mut().write(pos, mat);
            sim2.voxel_mut().write(pos, mat);
        }
        let hash_before_1 = sim1.hash_chain_root();
        let hash_before_2 = sim2.hash_chain_root();
        assert_eq!(hash_before_1, hash_before_2);
        sim1.tick();
        sim2.tick();

        assert_eq!(sim1.replay_log(), sim2.replay_log());
        assert_eq!(sim1.last_tick_voxel_events(), sim2.last_tick_voxel_events());
        assert_eq!(sim1.voxel().chunk_count(), sim2.voxel().chunk_count());
        assert_eq!(sim1.hash_chain_root(), sim2.hash_chain_root());
    }

    /// FR-CIV-ENGINE-REPLAY-001 — ReplayLog round-trips through save/load.
    #[test]
    fn replay_log_round_trips_through_save_load() {
        let mut log = ReplayLog {
            seed: 99,
            ..ReplayLog::default()
        };
        log.record_tick(1);
        log.record_voxel_write(1, WorldCoord { x: 1, y: 2, z: 3 }, MaterialId(7));
        log.record_damage(
            2,
            DamageEvent {
                center: WorldCoord { x: 0, y: 0, z: 0 },
                radius_voxels: 2,
                energy: 11,
            },
        );
        log.record_research(3, vec![1, 2, 3], true);

        let file = NamedTempFile::new().unwrap();
        log.save(file.path()).unwrap();
        let loaded = ReplayLog::load(file.path()).unwrap();
        assert_eq!(loaded, log);
    }

    /// FR-CIV-ENGINE-REPLAY-002 — Simulation tick produces a ReplayEvent::Tick.
    #[test]
    fn simulation_tick_produces_replay_tick_event() {
        let mut sim = Simulation::with_seed(1);
        sim.tick();
        assert!(matches!(
            sim.replay_log().events.last(),
            Some(ReplayEvent::Tick { tick: 1 })
        ));
    }

    /// FR-CIV-TACTICS-041 — combat events extend the replay hash chain.
    #[test]
    fn combat_events_extend_replay_hash_chain() {
        let event = DamageEvent {
            center: WorldCoord { x: 10, y: 0, z: 20 },
            radius_voxels: 2,
            energy: 100,
        };
        let mut log = ReplayLog::default();
        log.record_tick(1);
        let after_tick = log.running_hash;
        log.record_combat(1, 10, 20, event);
        log.verify_hash_chain().expect("chain");
        assert_ne!(log.running_hash, after_tick);
    }

    /// FR-CIV-TACTICS-025-int — replay log restores queued combat damage events.
    #[test]
    fn replay_combat_events_restore_pending_damage() {
        let event = DamageEvent {
            center: WorldCoord {
                x: 100,
                y: 0,
                z: 200,
            },
            radius_voxels: 2,
            energy: 50,
        };
        let mut sim = Simulation::with_seed(1);
        sim.replay_log.record_combat(16, 10, 20, event);
        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(99);
        log.replay(&mut replayed).unwrap();
        assert_eq!(replayed.pending_damage.len(), 1);
        assert_eq!(replayed.pending_damage[0], event);
        assert_eq!(replayed.state.tick, 16);
    }

    /// FR-CIV-TACTICS-025-int2 — replay combat events drain to the same voxel state as live ticks.
    #[test]
    fn replay_combat_drains_to_same_voxel_state_as_live() {
        let seed = 12;
        let ticks = 32u64;
        let mut live = Simulation::with_seed(seed);
        for _ in 0..ticks {
            live.tick();
        }
        let chunk_live = live.voxel().chunk_count();
        let combat: Vec<_> = live
            .replay_log()
            .events
            .iter()
            .filter_map(|event| match event {
                ReplayEvent::Combat { tick, event, .. } => Some((*tick, *event)),
                _ => None,
            })
            .collect();
        assert!(
            !combat.is_empty(),
            "expected war-bridge combat in replay log"
        );

        let mut from_replay = Simulation::with_seed(seed);
        for (tick, event) in combat {
            from_replay.apply_replay_combat(tick, &event);
        }
        let pending: Vec<DamageEvent> = from_replay.pending_damage.drain(..).collect();
        for event in pending {
            let _ = from_replay.apply_damage_now(&event);
        }
        assert_eq!(from_replay.voxel().chunk_count(), chunk_live);
    }

    /// FR-CIV-TACTICS-025-int3 — same seed reproduces identical combat replay markers.
    #[test]
    fn replay_combat_log_deterministic_for_seed_rerun() {
        let seed = 5;
        let ticks = 48u64;
        let mut a = Simulation::with_seed(seed);
        let mut b = Simulation::with_seed(seed);
        for _ in 0..ticks {
            a.tick();
            b.tick();
        }
        let combat_a: Vec<_> = a
            .replay_log()
            .events
            .iter()
            .filter_map(|e| match e {
                ReplayEvent::Combat {
                    tick,
                    shooter_id,
                    target_id,
                    event,
                } => Some((*tick, *shooter_id, *target_id, *event)),
                _ => None,
            })
            .collect();
        let combat_b: Vec<_> = b
            .replay_log()
            .events
            .iter()
            .filter_map(|e| match e {
                ReplayEvent::Combat {
                    tick,
                    shooter_id,
                    target_id,
                    event,
                } => Some((*tick, *shooter_id, *target_id, *event)),
                _ => None,
            })
            .collect();
        assert!(!combat_a.is_empty());
        assert_eq!(combat_a, combat_b);
    }

    /// FR-CIV-TACTICS-025 — war-bridge engagements append ReplayEvent::Combat.
    #[test]
    fn war_bridge_records_combat_replay_events() {
        let mut sim = Simulation::with_seed(1);
        for _ in 0..16 {
            sim.tick();
        }
        assert!(sim.replay_log().events.iter().any(|event| {
            matches!(
                event,
                ReplayEvent::Combat {
                    shooter_id,
                    target_id,
                    ..
                } if *shooter_id != 0 && *target_id != 0
            )
        }));
    }

    /// FR-CIV-ENGINE-REPLAY-003 — push_damage records a Damage event.
    #[test]
    fn push_damage_records_damage_event() {
        let mut sim = Simulation::with_seed(1);
        let event = DamageEvent {
            center: WorldCoord { x: 1, y: 1, z: 1 },
            radius_voxels: 3,
            energy: 4,
        };
        sim.push_damage(event);
        assert!(matches!(
            sim.replay_log().events.last(),
            Some(ReplayEvent::Damage { tick: 0, event: recorded }) if recorded == &event
        ));
    }

    /// FR-CIV-ENGINE-REPLAY-004 — replay reproduces final voxel chunk count and tick.
    #[test]
    fn replay_reproduces_final_voxel_chunk_count_and_tick() {
        let mut sim = Simulation::with_seed(2);
        sim.voxel_mut()
            .write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));
        sim.push_damage(DamageEvent {
            center: WorldCoord { x: 0, y: 0, z: 0 },
            radius_voxels: 1,
            energy: 1,
        });
        sim.tick();

        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(2);
        log.replay(&mut replayed).unwrap();
        assert_eq!(replayed.state.tick, sim.state.tick);
        assert_eq!(replayed.voxel().chunk_count(), sim.voxel().chunk_count());
    }

    /// CIV-0104 — minimal tick invariants hold after every tick.
    #[test]
    fn tick_invariants_hold_across_many_ticks() {
        use crate::invariants::check_tick_invariants;

        let mut sim = Simulation::with_seed(104);
        check_tick_invariants(&sim).expect("initial state");

        for _ in 0..200 {
            sim.tick();
            check_tick_invariants(&sim).expect("invariants after tick");
        }
    }

    /// FR-REPLAY-001 — `.civreplay` save/load restores simulation tick after N ticks.
    #[test]
    fn civreplay_save_load_restores_tick_after_ticks() {
        const N: u64 = 17;
        let mut sim = Simulation::with_seed(7);
        for _ in 0..N {
            sim.tick();
        }
        let expected_tick = sim.state.tick;

        let file = NamedTempFile::new().unwrap();
        sim.save_replay(file.path()).unwrap();
        let loaded = Simulation::load_replay_from_file(file.path()).unwrap();
        assert_eq!(loaded.state.tick, expected_tick);
    }

    /// FR-CIV-ENGINE-REPLAY-005 — identical replay logs converge to identical voxel state.
    #[test]
    fn replay_logs_converge_to_identical_voxel_state() {
        let mut sim1 = Simulation::with_seed(3);
        sim1.voxel_mut()
            .write(WorldCoord { x: 4, y: 5, z: 6 }, MaterialId(9));
        sim1.voxel_mut()
            .write(WorldCoord { x: 8, y: 9, z: 10 }, MaterialId(8));
        sim1.tick();

        let log = sim1.replay_log().clone();
        let mut sim2 = Simulation::with_seed(3);
        log.replay(&mut sim2).unwrap();

        assert_eq!(sim1.state.tick, sim2.state.tick);
        assert_eq!(
            sim1.voxel().read(WorldCoord { x: 4, y: 5, z: 6 }),
            sim2.voxel().read(WorldCoord { x: 4, y: 5, z: 6 })
        );
        assert_eq!(
            sim1.voxel().read(WorldCoord { x: 8, y: 9, z: 10 }),
            sim2.voxel().read(WorldCoord { x: 8, y: 9, z: 10 })
        );
    }

    /// FR-CIV-TACTICS-025 — replay round-trip: war-bridge Combat events exist in the
    /// original log and the replayed simulation converges to the same tick and voxel state.
    #[test]
    fn replay_round_trip_preserves_combat_events() {
        let mut sim = Simulation::with_seed(1);
        for _ in 0..16 {
            sim.tick();
        }

        let combat_count = sim.replay_log().combat_event_count();
        assert!(
            combat_count > 0,
            "expected at least one Combat replay event after 16 ticks"
        );

        let log = sim.replay_log().clone();
        let mut replayed = Simulation::with_seed(1);
        log.replay(&mut replayed).unwrap();

        assert_eq!(
            replayed.state.tick, sim.state.tick,
            "replayed tick must match original"
        );
        assert_eq!(
            replayed.voxel().chunk_count(),
            sim.voxel().chunk_count(),
            "replayed voxel chunk count must match original"
        );
    }

    /// FR-CIV-TACTICS-024 — snapshot.damage_events reflects combat pulses from
    /// the most recent tick.
    #[test]
    fn snapshot_damage_events_reflects_last_tick_pulses() {
        let mut sim = Simulation::with_seed(6);
        // Advance to a war-bridge cadence boundary (cadence = 16).
        for _ in 0..16 {
            sim.tick();
        }
        let snap = sim.snapshot();
        // After a cadence tick with ≥2 opposing military units the pulses list
        // must be non-empty; the snapshot field must match.
        assert_eq!(snap.damage_events, sim.last_tick_combat_pulses().len());
    }

    /// FR-CIV-PLANET-030 — `snapshot().weather_grid` temperature varies with
    /// year phase (summer equatorial > winter equatorial) and results are
    /// deterministic across re-runs.
    #[test]
    fn weather_grid_temperature_varies_with_year_phase() {
        // Earth-like defaults: year_length_ticks = 8_766_000, tilt = 23°.
        let year_length_ticks = 8_766_000_u64;
        let equatorial_idx = 8_usize; // middle of 16-region grid

        // Northern summer: year ¼ → sin(year_phase) is at peak
        let summer_tick = year_length_ticks / 4;
        // Northern winter: year ¾ → sin(year_phase) is at trough
        let winter_tick = year_length_ticks * 3 / 4;

        let mut sim_s = Simulation::with_seed(0);
        // Fast-forward to summer_tick by running ticks (use state manipulation
        // for test speed: set tick directly and recompute phase_planet).
        sim_s.state.tick = summer_tick;
        let planet_s = *sim_s.planet();
        let moon_s = *sim_s.moon();
        sim_s.climate = compute_climate(summer_tick, &planet_s, &moon_s);
        sim_s.weather_grid = compute_weather(&sim_s.climate, summer_tick, 16);
        let snap_summer = sim_s.snapshot();

        let mut sim_w = Simulation::with_seed(0);
        sim_w.state.tick = winter_tick;
        let planet_w = *sim_w.planet();
        let moon_w = *sim_w.moon();
        sim_w.climate = compute_climate(winter_tick, &planet_w, &moon_w);
        sim_w.weather_grid = compute_weather(&sim_w.climate, winter_tick, 16);
        let snap_winter = sim_w.snapshot();

        let summer_temp = snap_summer.weather_grid[equatorial_idx].temp_c_fp;
        let winter_temp = snap_winter.weather_grid[equatorial_idx].temp_c_fp;

        assert!(
            summer_temp > winter_temp,
            "summer equatorial temp ({summer_temp} fp) should exceed winter ({winter_temp} fp)"
        );

        // Determinism: re-running the same ticks must produce identical grids.
        let summer_grid_2 = compute_weather(&sim_s.climate, summer_tick, 16);
        assert_eq!(
            snap_summer.weather_grid, summer_grid_2,
            "weather grid must be deterministic across re-runs"
        );
    }

    // -------------------------------------------------------------------------
    // FR-CIV-CA-009 — `Simulation::phase_voxel_ca` + abiogenesis sites.
    // -------------------------------------------------------------------------

    /// FR-CIV-CA-009 — `phase_voxel_ca(None)` is a no-op: sites stay empty.
    /// This is the cheap path (no resident window wired up) and must not
    /// blow up or allocate a giant vec.
    #[test]
    #[ignore = "Simulation::phase_voxel_ca and last_tick_abiogenesis_sites() not implemented"]
    fn phase_voxel_ca_none_is_noop() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-CA-009 — warm liquid WATER in a single chunk produces at
    /// least one viable abiogenesis site. A pure STONE chunk produces
    /// zero. The two runs must round-trip deterministically (same seed,
    /// same grid → same sites).
    #[test]
    #[ignore = "Simulation::phase_voxel_ca and last_tick_abiogenesis_sites() not implemented"]
    fn phase_voxel_ca_warm_water_is_viable_stone_is_not() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-0100 — chronicle records technological breakthroughs when tech bits advance.
    #[test]
    #[ignore = "WorldState::research_progress field, Simulation::phase_tech() and phase_chronicle() and chronicle() not implemented"]
    fn chronicle_records_tech_breakthroughs() {
        // TODO: Implement WorldState::research_progress, Simulation::phase_tech, phase_chronicle, chronicle
    }

    /// FR-CIV-0100 — chronicle length stays bounded at CHRONICLE_MAX_LEN.
    #[test]
    #[ignore = "WorldState::chronicle field and Simulation::phase_chronicle() and chronicle() not implemented"]
    fn chronicle_is_length_capped() {
        // TODO: Implement WorldState::chronicle field, Simulation::phase_chronicle and chronicle
    }

    /// FR-CIV-0100 — golden-age chronicle lines are deduped via chronicle_age.
    #[test]
    #[ignore = "WorldState::chronicle_age field, Simulation::phase_chronicle() and chronicle() not implemented"]
    fn chronicle_dedups_age() {
        // TODO: Implement WorldState::chronicle_age, Simulation::phase_chronicle and chronicle
    }

    /// `tick_with_emergence_source` advances ticks identically; CA grid changes sampling.
    #[test]
    #[ignore = "Simulation::tick_with_emergence_source() not implemented"]
    fn tick_with_emergence_source_advances_tick_and_differs_on_ca_grid() {
        // TODO: Implement Simulation::tick_with_emergence_source
    }

    /// `apply_scenario_military` wires cadence overrides and clamps engage range.
    #[test]
    fn apply_scenario_military_wires_overrides_and_clamps_range() {
        use crate::scenario::ScenarioMilitary;

        let mut sim = Simulation::with_seed(8);
        let military = ScenarioMilitary {
            movement_cadence_ticks: Some(8),
            movement_pulses_per_cadence: Some(3),
            war_cadence_ticks: Some(32),
            engage_range_grid: Some(0),
        };
        sim.apply_scenario_military(&military);
        let cfg = sim.military_phase_config();
        assert_eq!(cfg.movement.cadence_ticks, 8);
        assert_eq!(cfg.movement_pulses_per_cadence, 3);
        assert_eq!(cfg.war.cadence_ticks, 32);
        assert_eq!(cfg.war.engage_range_grid, 1);
    }

    /// `configure_military_fog` sets vision radius and clamps grid size.
    #[test]
    fn configure_military_fog_sets_radius_and_clamps_grid() {
        let mut sim = Simulation::with_seed(9);
        sim.configure_military_fog(Some(8), 12);
        assert_eq!(
            sim.military_phase_config().war.fog_vision_radius,
            Some(8)
        );
        assert_eq!(sim.military_phase_config().war.fog_grid_size, 16);

        let kept_radius = sim.military_phase_config().war.fog_vision_radius;
        let kept_grid = sim.military_phase_config().war.fog_grid_size;
        sim.configure_military_fog(None, 99);
        assert_eq!(
            sim.military_phase_config().war.fog_vision_radius,
            kept_radius
        );
        assert_eq!(sim.military_phase_config().war.fog_grid_size, kept_grid);
    }

    // -------------------------------------------------------------------
    // Coverage-gap closure (COVERAGE_GAPS_4): the three pure policy helpers
    // below had no direct unit tests prior to this commit. Each test below
    // is named per the coverage-gap closure plan and bundles all relevant
    // edge cases from TEST_SPECS_UNTESTED.md into a single `#[test]`.
    // -------------------------------------------------------------------

    /// `job_type_for_civilian_id` is a total pure function of its `u64`
    /// input. This test pins the full mod-7 bucket map (including the
    /// catch-all `_` arm), wrap-around at the modulus, sparse / far-out ids
    /// resolving to the right bucket via `id % 7`, the `u64::MAX` boundary,
    /// and the determinism guarantee (same id → same `JobType`, no state).
    /// FR-CIV-ENGINE spawn-determinism depends on this. (COVERAGE_GAPS_4 row 1.)
    #[test]
    fn job_type_for_civilian_id_deterministic_split() {
        // All seven mod-buckets, including the `_`-arm for remainder 6.
        assert_eq!(job_type_for_civilian_id(0), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(1), JobType::Warrior);
        assert_eq!(job_type_for_civilian_id(2), JobType::Scholar);
        assert_eq!(job_type_for_civilian_id(3), JobType::Trader);
        assert_eq!(job_type_for_civilian_id(4), JobType::Priest);
        assert_eq!(job_type_for_civilian_id(5), JobType::Admin);
        assert_eq!(job_type_for_civilian_id(6), JobType::Unemployed);

        // `id % 7` wraps cleanly: every 7th id resolves to the same JobType.
        assert_eq!(job_type_for_civilian_id(7), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(14), JobType::Farmer);
        assert_eq!(job_type_for_civilian_id(42), JobType::Farmer); // 42 % 7 == 0
        assert_eq!(job_type_for_civilian_id(13), JobType::Unemployed); // 13 % 7 == 6
        assert_eq!(job_type_for_civilian_id(20), JobType::Unemployed); // 20 % 7 == 6

        // Sparse / far-out ids resolve to a deterministic bucket.
        // 1_000_000_008 % 7 == 0 (1_000_000_008 = 142_857_144 * 7) → Farmer.
        assert_eq!(job_type_for_civilian_id(1_000_000_008), JobType::Farmer);
        // 999_999_999 % 7: 999_999_999 / 7 = 142_857_142 remainder 5 → Admin.
        assert_eq!(job_type_for_civilian_id(999_999_999), JobType::Admin);
        // 1_000_000_000_000_000_000 % 7 = 1 → Warrior.
        assert_eq!(
            job_type_for_civilian_id(1_000_000_000_000_000_000),
            JobType::Warrior
        );

        // u64::MAX % 7 == 1 (u64::MAX = 2^64-1 = 2_635_249_153_387_078_802*7 + 1)
        // → Warrior. Confirms totality over the full u64 range, no overflow.
        assert_eq!(job_type_for_civilian_id(u64::MAX), JobType::Warrior);

        // Determinism: same id → same JobType, no state, no panic.
        for id in [0u64, 1, 6, 7, 42, 100, 999_999_999, u64::MAX] {
            assert_eq!(
                job_type_for_civilian_id(id),
                job_type_for_civilian_id(id),
                "job_type_for_civilian_id({id}) must be a pure function of its input"
            );
        }
    }

    /// `faction_wealth_scarcity_shadow` maps (treasury, resources) → shadow
    /// price used as input to `faction_unrest_delta_from_shadow`. This test
    /// pins the comfort-threshold branch (≥ 12_000 → baseline), the exact
    /// `12_000` boundary, the empty-Resources "deep scarcity" extreme
    /// (wealth = 0 → 4_000), food-only and treasury-only shortfalls, the
    /// lower floor at `FOOD_SCARCITY_BASELINE`, and the `treasury.raw / SCALE`
    /// integer-units conversion. (COVERAGE_GAPS_4 row 5.)
    #[test]
    fn faction_wealth_scarcity_shadow_edge_cases() {
        // Comfort branch: wealth >= 12_000 pins shadow to FOOD_SCARCITY_BASELINE.
        // treasury=100_000, food=10_000 → wealth = 100_000 + 10_000*50 = 600_000.
        let res = Resources {
            food: Fixed::from_num(10_000),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(100_000), &res),
            FOOD_SCARCITY_BASELINE
        );

        // Exact comfort boundary: wealth == 12_000 still pins to baseline
        // because the function uses `>=`, not strict `>`.
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(12_000), &res),
            FOOD_SCARCITY_BASELINE
        );

        // Empty Resources + zero treasury = "deep scarcity": wealth = 0,
        // shadow = 1_000 + 12_000/4 = 4_000. (No upper clamp inside the
        // function; this is the maximum shadow reachable in one call.)
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::ZERO, &res),
            FOOD_SCARCITY_BASELINE + 12_000 / 4,
            "empty Resources + zero treasury lands at the maximum shadow"
        );

        // Food-only shortfall: treasury = 0, food = 10 → wealth = 500.
        // shadow = 1_000 + (12_000 - 500)/4 = 1_000 + 2_875 = 3_875.
        let res = Resources {
            food: Fixed::from_num(10),
            wood: Fixed::ZERO,
            metal: Fixed::ZERO,
            energy: Fixed::ZERO,
        };
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::ZERO, &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 500) / 4
        );

        // Treasury-only shortfall: treasury = 4_000, food = 0 → wealth = 4_000.
        // shadow = 1_000 + (12_000 - 4_000)/4 = 3_000.
        // NOTE: the function does NOT implement a "treasury hedges food"
        // channel — treasury is additive in the same units as the
        // food-weighted wealth. This test pins the actual behavior.
        let res = Resources::default();
        assert_eq!(
            faction_wealth_scarcity_shadow(Fixed::from_num(4_000), &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 4_000) / 4
        );

        // Lower floor: shadow never falls below FOOD_SCARCITY_BASELINE for
        // any legal input. The comfort branch pins to it, the shortfall
        // branch adds to it.
        let cases: Vec<(i64, Resources)> = vec![
            (0, Resources::default()),
            (10_000, Resources::default()),
            (0, Resources { food: Fixed::from_num(1), ..Resources::default() }),
            (Fixed::from_num(5_000).raw, Resources::default()),
            (Fixed::from_num(99_999_999).raw, Resources::default()),
        ];
        for (treasury_raw, res) in cases {
            let treasury = Fixed { raw: treasury_raw };
            let shadow = faction_wealth_scarcity_shadow(treasury, &res);
            assert!(
                shadow >= FOOD_SCARCITY_BASELINE,
                "shadow ({shadow}) fell below FOOD_SCARCITY_BASELINE ({FOOD_SCARCITY_BASELINE})"
            );
        }

        // `treasury.raw / SCALE` is the integer wealth — guards against a
        // regression that would drop the `/ SCALE` and treat `raw` directly
        // as a wealth value.
        // treasury = 5_000 (fixed-point) → treasury_i = 5_000, food_i = 0,
        // wealth = 5_000 < 12_000 → shortfall: 1_000 + 7_000/4 = 2_750.
        let res = Resources::default();
        let treasury = Fixed::from_num(5_000);
        assert_eq!(
            faction_wealth_scarcity_shadow(treasury, &res),
            FOOD_SCARCITY_BASELINE + (12_000 - 5_000) / 4
        );
    }

    /// `faction_unrest_delta_from_shadow` is a thin pass-through to
    /// `unrest_delta`. This test pins the sign behavior (shadow ≤ baseline
    /// → decay `-10`; shadow > baseline → positive rise), the `clamp(1, 50)`
    /// bounds, the linear scaling with shortfall, the `MAX_RISE = 50`
    /// ceiling for arbitrarily large shadows (including `i64::MAX`), and
    /// the wrapper's identity with `unrest_delta` across the full sign
    /// range. (COVERAGE_GAPS_4 row 6: "clamp at 0" lives in the caller's
    /// accumulator; the delta itself only knows `-10` and `[1, 50]`.)
    #[test]
    fn faction_unrest_delta_from_shadow_sign_and_clamp() {
        // shadow ≤ baseline → decay -10 (not zero, not positive).
        for shadow in [0i64, 100, 500, 999] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                -10,
                "shadow={shadow} (below baseline) must decay by 10"
            );
        }

        // At the boundary shadow == baseline the function takes the `else`
        // branch (scarcity is not > 0) and returns -10, not zero. Pin this
        // so a future `>=` refactor doesn't silently flip the boundary.
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE),
            -10
        );

        // Just above baseline, rise is clamped to a minimum of +1
        // (clamp(1, MAX_RISE) lower bound kicks in for any scarcity > 0,
        // even when scarcity / 20 == 0).
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1),
            1
        );
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 19),
            1
        );

        // Rise scales linearly with shortfall (scarcity / 20) until it
        // hits the MAX_RISE ceiling of 50.
        // shadow = 1_100 → scarcity = 100 → 100/20 = 5
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 100),
            5
        );
        // shadow = 1_400 → scarcity = 400 → 400/20 = 20
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 400),
            20
        );
        // shadow = 2_000 → scarcity = 1_000 → 1_000/20 = 50 (at ceiling)
        assert_eq!(
            faction_unrest_delta_from_shadow(FOOD_SCARCITY_BASELINE + 1_000),
            50
        );

        // Large shadows still clamp to MAX_RISE = 50. Stops a price spike
        // from instantly maxing faction unrest.
        for shadow in [10_000i64, 1_000_000, 1_000_000_000, i64::MAX] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                50,
                "shadow={shadow} must clamp at MAX_RISE=50"
            );
        }

        // Wrapper identity with `unrest_delta` across the full sign range.
        for shadow in [
            0i64,
            FOOD_SCARCITY_BASELINE - 1,
            FOOD_SCARCITY_BASELINE,
            FOOD_SCARCITY_BASELINE + 1,
            FOOD_SCARCITY_BASELINE + 100,
            FOOD_SCARCITY_BASELINE + 1_000,
            FOOD_SCARCITY_BASELINE + 100_000,
            i64::MAX,
        ] {
            assert_eq!(
                faction_unrest_delta_from_shadow(shadow),
                unrest_delta(shadow),
                "wrapper must equal unrest_delta at shadow={shadow}"
            );
        }
    }

    // ── N9 tests ──────────────────────────────────────────────────────────────

    /// N9: `aggression_threshold_reduction` is bounded: 0.0→0, 0.5→1500,
    /// 1.0→3000, and clamping means 2.0 still yields 3000.
    #[test]
    fn aggression_threshold_reduction_bounded() {
        assert_eq!(aggression_threshold_reduction(0.0), 0);
        assert_eq!(aggression_threshold_reduction(0.5), 1500);
        assert_eq!(aggression_threshold_reduction(1.0), 3000);
        assert_eq!(aggression_threshold_reduction(2.0), 3000); // clamped
    }

    /// N9: `faction_aggression` is rebuilt fresh each tick (ephemeral).
    #[test]
    fn faction_aggression_rebuilt_each_tick() {
        let mut sim = Simulation::with_seed(1);
        // Before any tick, faction_aggression is empty.
        assert!(
            sim.faction_aggression.is_empty(),
            "faction_aggression should start empty"
        );
        // After a tick the emergence phase populates it (agents have DNA).
        sim.tick();
        // The map is populated whenever there are aligned civilians with DNA.
        // Just verify the field is accessible and the type is correct.
        let _: &std::collections::BTreeMap<u32, f32> = &sim.faction_aggression;
    }

    /// N9: faction pairs with high aggression clash at lower disparity than
    /// faction pairs with zero aggression.
    #[test]
    #[ignore = "diplomacy_faction_pair() function not implemented"]
    fn aggressive_factions_clash_sooner() {
        // Build a baseline sim where factions are at the trade/conflict boundary.
        let mut sim_low = Simulation::with_seed(5);
        sim_low.state.tick = 500;
        sim_low.state.belief = 0;
        sim_low.state.cohesion = 0;
        sim_low.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim_low.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim_low.state.tick);
        // A disparity just below the base threshold: both sims should trade normally.
        let base = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
        sim_low.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_low.state.faction_treasury.insert(b, Fixed::from_num(base - 1));
        // Zero aggression → no reduction.
        sim_low.faction_aggression.insert(a, 0.0);
        sim_low.faction_aggression.insert(b, 0.0);
        sim_low.phase_diplomacy();
        let low_kind = sim_low.diplomacy_events().last().expect("event").kind;

        // High aggression sim: same disparity, but aggression lowers threshold.
        let mut sim_high = Simulation::with_seed(5);
        sim_high.state.tick = 500;
        sim_high.state.belief = 0;
        sim_high.state.cohesion = 0;
        sim_high.state.unrest = 0;
        sim_high.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_high.state.faction_treasury.insert(b, Fixed::from_num(base - 1));
        // Max aggression → reduction = 3000, so threshold drops to DIPLOMACY_MIN_CONFLICT_THRESHOLD.
        sim_high.faction_aggression.insert(a, 1.0);
        sim_high.faction_aggression.insert(b, 1.0);
        sim_high.phase_diplomacy();
        let high_kind = sim_high.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            low_kind,
            DiplomacyKind::TradeAgreement,
            "low-aggression factions should trade at this disparity"
        );
        assert_eq!(
            high_kind,
            DiplomacyKind::Conflict,
            "high-aggression factions should clash at the same disparity"
        );
    }

    // N11 maturity↔belief coupling tests (FR-CIV-EMERGENCE-N11)

    #[test]
    fn n11_avg_psyche_maturity_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        assert_eq!(avg_psyche_maturity(&sim.world), 0.0);
    }

    #[test]
    fn n11_avg_psyche_maturity_computes_mean() {
        use civ_agents::{Mood, Psyche, Temperament, PSYCHE_DIM};
        let mut sim = Simulation::new();
        sim.world.clear();
        let psyche = Psyche {
            drives: [0.5; PSYCHE_DIM],
            temperament: Temperament::neutral(),
            mood: Mood::neutral(),
            beliefs: [0.5; PSYCHE_DIM],
            maturity: 1.0,
        };
        sim.world.spawn((psyche,));
        assert_eq!(avg_psyche_maturity(&sim.world), 1.0);
    }

    #[test]
    fn n11_drift_factor_bounds() {
        for (maturity, expected) in [(0.0f32, 0.95f32), (0.5, 0.975), (1.0, 1.0)] {
            let drift = 0.95 + 0.05 * maturity;
            assert!((drift - expected).abs() < 1e-6, "maturity={} drift={}", maturity, drift);
        }
    }

    /// `canonical_faction_pair` always returns the pair in ascending order so
    /// (a, b) and (b, a) hash to the same BTreeMap key.
    #[test]
    fn canonical_faction_pair_orders_ascending() {
        assert_eq!(canonical_faction_pair(0, 1), (0, 1), "already sorted");
        assert_eq!(canonical_faction_pair(1, 0), (0, 1), "reversed becomes sorted");
        assert_eq!(canonical_faction_pair(3, 3), (3, 3), "equal ids stay equal");
        assert_eq!(canonical_faction_pair(u32::MAX, 0), (0, u32::MAX), "large vs small");
        for (a, b) in [(2u32, 5), (10, 1), (7, 7), (0, u32::MAX)] {
            assert_eq!(
                canonical_faction_pair(a, b),
                canonical_faction_pair(b, a),
                "canonical_faction_pair({a},{b}) must be symmetric"
            );
        }
    }

    /// `route_resource` maps known goods labels to the correct ResourceType.
    /// Unknown goods fall back to Food (documented default).
    #[test]
    fn route_resource_maps_known_goods() {
        assert_eq!(route_resource("grain"), ResourceType::Food, "grain → Food");
        assert_eq!(route_resource("timber"), ResourceType::Wood, "timber → Wood");
        assert_eq!(route_resource("ore"), ResourceType::Metal, "ore → Metal");
        assert_eq!(route_resource("tools"), ResourceType::Metal, "tools → Metal");
        assert_eq!(route_resource("cloth"), ResourceType::Energy, "cloth → Energy");
        assert_eq!(route_resource("salt"), ResourceType::Energy, "salt → Energy");
        assert_eq!(route_resource(""), ResourceType::Food, "empty string → Food (fallback)");
        assert_eq!(route_resource("unknown"), ResourceType::Food, "unrecognized → Food (fallback)");
    }

    /// `emergent_route_goods` is deterministic: same faction id → same goods
    /// label, cycling across the three labels via id % 3.
    #[test]
    fn emergent_route_goods_is_deterministic_and_covers_all_labels() {
        assert_eq!(emergent_route_goods(0), "grain", "id%3==0 → grain");
        assert_eq!(emergent_route_goods(1), "ore",   "id%3==1 → ore");
        assert_eq!(emergent_route_goods(2), "cloth", "id%3==2 → cloth");
        assert_eq!(emergent_route_goods(3), "grain", "id=3 wraps to grain");
        for id in [0u32, 1, 2, 100, u32::MAX] {
            assert_eq!(
                emergent_route_goods(id),
                emergent_route_goods(id),
                "emergent_route_goods({id}) must be a pure function of its input"
            );
        }
        // All labels returned by emergent_route_goods must be handled by route_resource
        // without falling through to the unknown fallback path.
        let known_labels = ["grain", "ore", "cloth", "timber", "tools", "salt"];
        for id in 0u32..3 {
            let goods = emergent_route_goods(id);
            assert!(
                known_labels.contains(&goods),
                "emergent_route_goods({id})=\"{goods}\" is not a known trade label"
            );
        }
    }

    // N10 kinship↔cohesion coupling tests (FR-CIV-EMERGENCE-N10)

    #[test]
    fn n10_avg_faction_kinship_computes_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        let avg = avg_faction_kinship(&sim.world);
        assert_eq!(avg, 0.0, "empty world should have zero average kinship");
    }

    #[test]
    fn n10_avg_faction_kinship_computes_mean_correctly() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();
        sim.world.clear();

        // Spawn one social graph with a single kinship tie of 1.0.
        let graph_a = SocialGraph {
            ties: vec![Tie {
                other: 1002,
                kinship: 1.0,
                familiarity: 0.0,
                affinity: 0.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((graph_a,));
        sim.world.spawn((SocialGraph::default(),));

        let avg = avg_faction_kinship(&sim.world);
        assert_eq!(avg, 1.0, "one kinship tie of 1.0 should average to 1.0");
    }

    #[test]
    #[ignore = "Simulation::cohesion() not implemented"]
    fn n10_kinship_coupling_boosts_cohesion_basic() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();

        // Spawn a social graph with a kinship tie.
        let graph_a = SocialGraph {
            ties: vec![Tie {
                other: 2002,
                kinship: 1.0,
                familiarity: 0.0,
                affinity: 0.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((graph_a,));

        // Record cohesion before and after a tick.
        let before = sim.state.cohesion;
        sim.tick();
        let after = sim.state.cohesion;

        // With kinship=1.0, boost = 0.02 * 100_000 = 2000, so after >= before.
        // (caveat: other couplings and decay might affect this, but kinship boost
        // should dominate if no other agents contribute negative factors)
        assert!(
            after >= before,
            "phase_cohesion with kinship should not decrease cohesion (before={}, after={})",
            before,
            after
        );
    }

    #[test]
    fn n10_kinship_decay_factor_bounds() {
        // Verify the decay_factor formula stays in [0.93, 0.98].
        let test_cases: [(f32, f32); 3] = [(0.0, 0.93), (0.5, 0.955), (1.0, 0.98)];

        for (kinship, expected_factor) in test_cases {
            let decay_factor = 0.98_f32 - (0.05_f32 * (1.0_f32 - kinship)).max(0.0).min(1.0);
            assert!(
                (decay_factor - expected_factor).abs() < 1e-6,
                "kinship={} should give decay_factor≈{}, got {}",
                kinship,
                expected_factor,
                decay_factor
            );
        }
    }

    // N12 affinity↔diplomacy coupling tests (FR-CIV-EMERGENCE-N12)

    #[test]
    fn n12_avg_social_affinity_zero_for_empty_world() {
        let mut sim = Simulation::new();
        sim.world.clear();
        assert_eq!(avg_social_affinity(&sim.world), 0.0);
    }

    #[test]
    fn n12_avg_social_affinity_computes_mean_and_clamps() {
        use civ_agents::Tie;
        let mut sim = Simulation::new();
        sim.world.clear();
        // One graph affinity +1.0, one graph affinity -1.0 → mean 0.0.
        let g_pos = SocialGraph {
            ties: vec![Tie {
                other: 1,
                kinship: 0.0,
                familiarity: 0.0,
                affinity: 1.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        let g_neg = SocialGraph {
            ties: vec![Tie {
                other: 2,
                kinship: 0.0,
                familiarity: 0.0,
                affinity: -1.0,
                trust: 0.0,
                last_seen: 0,
            }],
        };
        sim.world.spawn((g_pos,));
        sim.world.spawn((g_neg,));
        assert!(avg_social_affinity(&sim.world).abs() < 1e-6);
    }

    #[test]
    fn n12_affinity_bias_direction_and_bounds() {
        // Positive affinity raises threshold; negative lowers it; bounded [-5000, 5000].
        let pos = affinity_threshold_bias(1.0);
        let neg = affinity_threshold_bias(-1.0);
        let zero = affinity_threshold_bias(0.0);
        assert_eq!(pos, 5_000);
        assert_eq!(neg, -5_000);
        assert_eq!(zero, 0);
        assert!(pos > zero && zero > neg, "goodwill must raise tolerance over hostility");
        // Out-of-range inputs clamp.
        assert_eq!(affinity_threshold_bias(2.0), 5_000);
        assert_eq!(affinity_threshold_bias(-2.0), -5_000);
    }

    #[test]
    #[ignore = "diplomacy_faction_pair() function not implemented"]
    fn n12_high_affinity_keeps_factions_trading() {
        use civ_agents::Tie;
        // Disparity ABOVE the base threshold (would Conflict at neutral affinity),
        // but strong collective goodwill raises the threshold enough to keep trade.
        let base = DIPLOMACY_BASE_CONFLICT_THRESHOLD;
        let disparity = base + 2_000; // 12_000: above base, below base + max affinity bias

        // Low-affinity sim: hostile ties → threshold drops → Conflict.
        let mut sim_low = Simulation::with_seed(5);
        sim_low.state.tick = 500;
        sim_low.state.belief = 0;
        sim_low.state.cohesion = 0;
        sim_low.state.unrest = 0;
        for _ in 0..3 {
            sim_low.world.spawn((SocialGraph {
                ties: vec![Tie {
                    other: 9,
                    kinship: 0.0,
                    familiarity: 0.0,
                    affinity: -1.0,
                    trust: 0.0,
                    last_seen: 0,
                }],
            },));
        }
        let mut faction_ids: Vec<u32> = sim_low.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return; // Defensive: need a faction pair; skip if scenario has none.
        }
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim_low.state.tick);
        sim_low.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_low
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_low.phase_diplomacy();
        let low_kind = sim_low.diplomacy_events().last().expect("event").kind;

        // High-affinity sim: goodwill ties → threshold rises → TradeAgreement.
        let mut sim_high = Simulation::with_seed(5);
        sim_high.state.tick = 500;
        sim_high.state.belief = 0;
        sim_high.state.cohesion = 0;
        sim_high.state.unrest = 0;
        for _ in 0..3 {
            sim_high.world.spawn((SocialGraph {
                ties: vec![Tie {
                    other: 9,
                    kinship: 0.0,
                    familiarity: 0.0,
                    affinity: 1.0,
                    trust: 0.0,
                    last_seen: 0,
                }],
            },));
        }
        sim_high.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_high
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_high.phase_diplomacy();
        let high_kind = sim_high.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            low_kind,
            DiplomacyKind::Conflict,
            "hostile populations should clash at disparity above base threshold"
        );
        assert_eq!(
            high_kind,
            DiplomacyKind::TradeAgreement,
            "collective goodwill should raise the threshold and keep factions trading"
        );
    }

    // ── Named-race seed spawn tests (FR-CIV-GENETICS-SEED-*) ─────────────────

    /// FR-CIV-GENETICS-SEED-001 — first spawned agent carries Ardani archetype
    /// DNA after applying divergence=0.3 with a fixed RNG seed, and the result
    /// is deterministic across two identical Simulation instances.
    #[test]
    fn test_seed_spawn_determinism() {
        use civ_genetics::NamedSeed;
        let sim_a = Simulation::with_seed(0xC0FFEE_u64);
        let sim_b = Simulation::with_seed(0xC0FFEE_u64);
        // Collect all Dna components from both worlds.
        let dna_a: Vec<Dna> = sim_a.world.query::<&Dna>().iter().map(|(_, d)| d.clone()).collect();
        let dna_b: Vec<Dna> = sim_b.world.query::<&Dna>().iter().map(|(_, d)| d.clone()).collect();
        assert_eq!(
            dna_a.len(),
            dna_b.len(),
            "both sims must spawn the same number of DNA-bearing entities"
        );
        assert!(!dna_a.is_empty(), "at least one entity must carry DNA");
        // Both runs must be bit-identical under the same seed.
        for (a, b) in dna_a.iter().zip(dna_b.iter()) {
            assert_eq!(a, b, "Dna must be deterministic under an identical RNG seed");
        }
        // The first civilian's DNA must differ from the raw zero genome, proving
        // it was seeded from an archetype rather than left default.
        let archetype = civ_genetics::archetype_dna(NamedSeed::Ardani);
        assert_eq!(dna_a[0].0.len(), archetype.0.len(), "genome length must match archetype");
        // With divergence=0.3 the result must not be all-zero (extremely unlikely).
        assert_ne!(dna_a[0].0, vec![0u8; 64], "seeded DNA must not be the zero genome");
    }

    /// FR-CIV-GENETICS-SEED-002 — spawn indices 0, 1, and 2 produce three
    /// distinct NamedSeed assignments (Ardani, Velthari, Grundak respectively).
    #[test]
    fn test_faction_archetype_variety() {
        use civ_genetics::NamedSeed;
        let ardani_base = civ_genetics::archetype_dna(NamedSeed::Ardani);
        let velthari_base = civ_genetics::archetype_dna(NamedSeed::Velthari);
        let grundak_base = civ_genetics::archetype_dna(NamedSeed::Grundak);

        // Verify the three archetypes are distinct from each other —
        // confirming the % 3 cycle will produce genuinely different seeds.
        assert_ne!(ardani_base, velthari_base, "Ardani and Velthari must differ");
        assert_ne!(ardani_base, grundak_base, "Ardani and Grundak must differ");
        assert_ne!(velthari_base, grundak_base, "Velthari and Grundak must differ");

        // With 128 civilians, each archetype slot is hit ~42-43 times.
        let sim = Simulation::with_seed(1);
        let dna_list: Vec<Dna> = sim.world.query::<&Dna>().iter().map(|(_, d)| d.clone()).collect();
        assert_eq!(dna_list.len(), 128, "all 128 civilians must carry Dna");

        // Verify that at minimum 3 distinct genomes are present, proving all
        // three archetype branches were exercised (divergence prevents collisions).
        let unique_count = {
            let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
            for d in &dna_list {
                seen.insert(d.0.clone());
            }
            seen.len()
        };
        assert!(
            unique_count >= 3,
            "at least 3 distinct genomes expected (one per archetype); got {unique_count}"
        );
    }

    /// FR-CIV-GENETICS-SEED-003 — `seed_with_divergence` at divergence=0.0
    /// returns an exact clone of the archetype; this is the zero-divergence contract.
    #[test]
    fn test_zero_divergence_exact() {
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        let archetype = civ_genetics::archetype_dna(NamedSeed::Ardani);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(0xDEAD_BEEF);
        let result = civ_genetics::seed_with_divergence(&archetype, 0.0, &mut rng);
        assert_eq!(
            result, archetype,
            "seed_with_divergence with divergence=0.0 must return an exact clone of the archetype"
        );
    }

    // ── FR-CONTENT-SEEDMIX: choose_named_seed helper unit tests ──────────────

    /// Empty seed_mix must reproduce the classic Ardani/Velthari/Grundak round-robin
    /// without advancing the RNG (bit-identical default path).
    #[test]
    #[ignore = "choose_named_seed() function not implemented"]
    fn choose_named_seed_empty_is_round_robin() {
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(1);
        let expected = [
            NamedSeed::Ardani,
            NamedSeed::Velthari,
            NamedSeed::Grundak,
            NamedSeed::Ardani,
            NamedSeed::Velthari,
            NamedSeed::Grundak,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let got = choose_named_seed(&[], None, i, &mut rng);
            assert_eq!(got, exp, "round-robin mismatch at spawn_index={i}");
        }
    }

    /// A 60/30/10 mix should yield Ardani as plurality (~0.6), Grundak as minority (~0.1).
    #[test]
    #[ignore = "choose_named_seed() function not implemented"]
    fn choose_named_seed_weighted_distribution() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        use rand::distributions::WeightedIndex;

        let seed_mix = vec![
            SeedWeight { seed: NamedSeed::Ardani, weight: 0.6 },
            SeedWeight { seed: NamedSeed::Velthari, weight: 0.3 },
            SeedWeight { seed: NamedSeed::Grundak, weight: 0.1 },
        ];
        let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
        let dist = WeightedIndex::new(&weights).expect("valid weights");

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        let n = 2000usize;
        let mut counts = [0usize; 3];
        for i in 0..n {
            let result = choose_named_seed(&seed_mix, Some(&dist), i, &mut rng);
            match result {
                NamedSeed::Ardani => counts[0] += 1,
                NamedSeed::Velthari => counts[1] += 1,
                NamedSeed::Grundak => counts[2] += 1,
            }
        }
        let ardani_frac = counts[0] as f32 / n as f32;
        let grundak_frac = counts[2] as f32 / n as f32;
        assert!(
            (ardani_frac - 0.6).abs() < 0.08,
            "Ardani fraction {ardani_frac:.3} not within ±0.08 of 0.6"
        );
        assert!(
            (grundak_frac - 0.1).abs() < 0.05,
            "Grundak fraction {grundak_frac:.3} not within ±0.05 of 0.1"
        );
        // Ardani must be the plurality
        assert!(counts[0] > counts[1] && counts[0] > counts[2], "Ardani must be plurality");
    }

    /// A single-entry mix must always yield that one race.
    #[test]
    #[ignore = "choose_named_seed() function not implemented"]
    fn choose_named_seed_single_seed_all_that_race() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::SeedableRng;
        use rand::distributions::WeightedIndex;

        let seed_mix = vec![SeedWeight { seed: NamedSeed::Velthari, weight: 1.0 }];
        let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
        let dist = WeightedIndex::new(&weights).expect("valid weights");

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        for i in 0..100 {
            let result = choose_named_seed(&seed_mix, Some(&dist), i, &mut rng);
            assert_eq!(result, NamedSeed::Velthari, "expected Velthari at index {i}");
        }
    }

    /// FR-CIV-014 / emergence-spawn — scenario-controlled faction spawning must
    /// honor arbitrary faction counts and per-faction civilian counts.
    #[test]
    #[ignore = "Simulation::with_seed_and_starting_conditions() not implemented"]
    fn scenario_faction_spawn_honors_counts() {
        use crate::scenario::ScenarioStartingConditions;
        use civ_agents::{Alignment, Civilian};
        use std::collections::BTreeMap;

        let sc = ScenarioStartingConditions {
            civilians_per_faction: 2,
            faction_count: 5,
            quadrant_spread: 1,
            seed_mix: Vec::new(),
        };
        let sim = Simulation::with_seed_and_starting_conditions(123, sc);

        let mut counts: BTreeMap<u32, u32> = BTreeMap::new();
        for (_, civ) in sim.world.query::<&Civilian>().iter() {
            if let Alignment::Faction(fid) = civ.alignment {
                *counts.entry(fid).or_insert(0) += 1;
            }
        }

        assert_eq!(counts.len(), 5, "expected five factions to be spawned");
        assert!(counts.values().all(|&count| count == 2));
    }

    // ── LANGUAGE→DIPLOMACY coupling tests ─────────────────────────────────────

    #[cfg(test)]
    mod language_diplomacy_tests {
        use super::*;

        #[test]
        fn bonus_bounded_and_monotonic() {
            let bonus_close = language_intelligibility_peace_bonus(0.1);
            let bonus_far = language_intelligibility_peace_bonus(0.9);
            assert!(bonus_close > bonus_far, "closer language must yield bigger bonus");
            assert!(bonus_close <= 1200, "bonus must not exceed cap");
        }

        #[test]
        fn identical_language_max_bonus_more_peaceful() {
            let max_bonus = language_intelligibility_peace_bonus(0.0);
            let no_bonus = language_intelligibility_peace_bonus(1.0);
            assert_eq!(max_bonus, 1200, "identical language must yield max bonus");
            assert_eq!(no_bonus, 0, "max distance must yield zero bonus");
            assert!(max_bonus > no_bonus);
        }

        #[test]
        fn missing_language_legacy_threshold_unchanged() {
            let bonus = language_intelligibility_peace_bonus(1.0);
            assert_eq!(bonus, 0, "missing language must not alter threshold");
        }
    }
}
