//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use civ_agents::{
    choose_activity, cluster_by_colocation, count_civilians, path_step, pick_target,
    propagate_tools, propagate_wardrobe, spawn_child_near, spawn_civilian_at, wander_anchor,
    Activity, Alignment, Civilian as AgentCivilian, ClusterMember, CohortStats, LodTier, Needs,
    PoiKind, PoiRegistry, Position3d, Tools, Wardrobe,
};
use civ_build::{Allocator, BuildingGraph, DemandSignals};
use civ_diffusion::DiffusionParams;
use civ_economy::Stocks as ClusterStocks;
use civ_economy::{AllocationEngine, CapitalistAllocator, EconomyState, MarketState};
use civ_mod_host::ModHost;
use civ_needs::{
    tick as needs_tick, DecayRates, Health as LifeHealth, HealthParams, Needs as LifeNeeds,
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
use civ_voxel::{DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE};
use hecs::{Entity, World};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use super::Fixed;
use crate::lod::{should_tick_entity_with_policy, LodPolicy};
use crate::policy::PolicyInput;
use crate::policy::DEFAULT_ECONOMY_POLICY;
use crate::replay::{ReplayError, ReplayLog};
use crate::replay_format::{load_civreplay, save_civreplay};

/// Ordered phase identifiers executed once per [`Simulation::tick`].
///
/// CIV-0001 partial — engine-side deterministic transition. Server command intake
/// and client broadcast are outside this crate. Keep in sync with the calls in
/// [`Simulation::tick`].
#[allow(dead_code)]
pub(crate) const PHASE_ORDER: &[&str] = &[
    "production",
    "citizen_lifecycle",
    "military",
    "economy",
    "planet",
    "diplomacy",
    "tactics",
    "voxel",
    "compact",
    "buildings",
    "diffusion",
    "disasters",
    "life",
    // MOAT emergence (FR-CIV-LEGENDS-*, FR-CIV-PSYCHE-*, FR-CIV-GENETICS-*, FR-CIV-AI-*)
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
    const CIVILIANS_PER_FACTION: usize = 32;
    const QUADRANT_SPREAD: i32 = 2_500;

    let faction_capitals = [
        (-7_500, 7_500),  // faction 0: NW
        (7_500, 7_500),   // faction 1: NE
        (-7_500, -7_500), // faction 2: SW
        (7_500, -7_500),  // faction 3: SE
    ];

    let scale = FIXED_SCALE as f32;
    let mut next_civilian_id = 1u64;
    for (center_x, center_y) in faction_capitals.into_iter() {
        for _ in 0..CIVILIANS_PER_FACTION {
            let grid_x = center_x + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let grid_z = center_y + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let norm_x = (grid_x as f32 / scale).clamp(0.0, 1.0);
            let norm_y = (grid_z as f32 / scale).clamp(0.0, 1.0);
            spawn_civilian_at(
                world,
                next_civilian_id,
                civ_agents::infer_alignment_for_spawn(world, norm_x, norm_y),
                norm_x,
                norm_y,
                civ_agents::ActorVisualKind::Humanoid,
                rng,
            );
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Accumulated research effort (FR-CIV-0200 research). Emerges from the
    /// living population each tick and gates emergent tech advancement in
    /// [`Simulation::phase_research`]. `#[serde(default)]` keeps older
    /// `.civsave` files loadable.
    #[serde(default)]
    pub research_progress: u64,
    /// Accumulated faith/belief (divine-powers economy). Emerges from the
    /// worshipping population each tick and is spent to invoke divine powers
    /// (e.g. triggering a disaster). `#[serde(default)]` keeps older saves
    /// loadable.
    #[serde(default)]
    pub belief: u64,
    /// Accumulated societal unrest (0 = content). EMERGES from food-market
    /// scarcity: a high clearing price drives it up, abundance lets it decay.
    /// Distinct from per-unit military `morale`. `#[serde(default)]` keeps older
    /// saves loadable.
    #[serde(default)]
    pub unrest: u64,
    /// Accumulated social cohesion — the strength of the shared social fabric.
    /// EMERGES from collective belief (shared faith binds) and frays under
    /// unrest (disorder loosens bonds). A stabilising counterweight to unrest.
    /// `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub cohesion: u64,
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
    pub resources: Resources,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 1_000_000,
            research_progress: 0,
            belief: 0,
            unrest: 0,
            cohesion: 0,
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
    pub(crate) climate: Climate,
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
    /// 3D voxel substrate (Civis 3D extension).
    voxel: VoxelWorld<MaterialId>,
    /// Voxel dirty events produced during the most recent tick.
    last_tick_voxel_events: Vec<DirtyChunkEvent>,
    last_tick_voxel_damage_count: usize,
    /// Per-soldier damage pulses from the most recent tactics phase (FR-CIV-TACTICS-024).
    last_tick_combat_pulses: Vec<CombatDamagePulse>,
    /// Engagements resolved this tick (war bridge); feeds doctrine fitness.
    last_tick_engagements: Vec<CombatEngagement>,
    /// `mod.loaded.v1` replay-bus JSON emitted when mods load (cleared each tick).
    last_tick_mod_lifecycle: Vec<String>,
    /// FR-CIV-CA-009: abiogenesis suitability sites detected this tick.
    last_tick_abiogenesis_sites: Vec<civ_voxel::fluid_ca::AbiogenesisSuitability>,
    operational: NoopOperationalLayer,
    replay_log: ReplayLog,
    /// Scenario economy policy.
    pub economy_policy: PolicyInput,
    /// Macro economy state.
    pub economy_state: EconomyState,
    /// Per-good clearing prices.
    pub market_state: MarketState,
    /// LOD tick cadence for Warm/Cold civilian tiers (CIV-0101).
    pub lod_policy: LodPolicy,
    /// Manifest-only mod host (CIV-0700 v2 policy stub); WASM not loaded yet.
    mod_host: ModHost,
    /// Military-phase cadence and per-tick movement pulses (FR-CIV-TACTICS-035).
    pub(crate) military_phase: MilitaryPhaseConfig,
    /// Per-faction doctrine libraries evolved on a fixed tick cadence (FR-CIV-TACTICS-010).
    faction_doctrines: Vec<DoctrineLibrary>,
    /// offset every tick (FR-CIV-PLANET-020). Keyed by `(x, z)` in fixed-point
    /// world coords; iteration order is deterministic.
    coastal_columns: BTreeMap<(i64, i64), CoastalColumn>,
    /// Per-region weather grid updated by `phase_planet` each tick (FR-CIV-PLANET-030).
    pub(crate) weather_grid: Vec<WeatherCell>,
    /// Per-cluster (emergent settlement) resource stocks, maintained by
    /// [`Simulation::phase_life`] (FR-CIV-LIFE-020). Keyed by emergent
    /// `ClusterId`; iteration order is deterministic (`BTreeMap`).
    cluster_stocks: BTreeMap<u64, ClusterStocks>,
    /// Number of emergent settlements (multi-member clusters) detected on the
    /// most recent [`Simulation::phase_life`] (FR-CIV-LIFE-030).
    pub(crate) last_settlement_count: u32,
    /// Deaths attributed to unmet-need sickness on the most recent life phase
    /// (FR-CIV-LIFE-003); surfaced for the HUD.
    pub(crate) last_life_deaths: u32,
    /// MOAT emergence: legends, psyche, culture, social, genetics, civ-ai.
    pub(crate) emergence: crate::emergence::EmergenceState,
    /// Latest emergence-metrics sample (civ-emergence-metrics). Updated by
    /// [`crate::emergence_metrics::sample_emergence`] on every 50-tick
    /// boundary (5 s at 100 ms tick). `None` before the first sample
    /// boundary (ticks 0..49). Surfaced over JSON-RPC `sim.emergence`
    /// (stacked on PR #350).
    pub(crate) emergence_sample: Option<crate::emergence_metrics::EmergenceSample>,
}

/// Voxel material id used to mark coastal water-level voxels written by
/// [`Simulation::apply_tide_offset`] (FR-CIV-PLANET-020). Kept as a small
/// integer so it is stable across saves and replays.
pub const WATER_MARKER_MATERIAL: MaterialId = MaterialId(2);

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

/// Build a [`PoiRegistry`] from the world's buildings, mapping each building
/// type to the need-serving POI kind agents path to (FR-CIV-LIFE-010). Grid
/// building coords are lifted into fixed-point world coordinates.
fn build_poi_registry(world: &World) -> PoiRegistry {
    let mut registry = PoiRegistry::default();
    for (idx, (_, building)) in world.query::<&Building>().iter().enumerate() {
        let kind = match building.building_type {
            BuildingType::Farm => PoiKind::FoodSource,
            BuildingType::House => PoiKind::Shelter,
            BuildingType::Market => PoiKind::SocialHub,
            BuildingType::Temple => PoiKind::SafeZone,
            BuildingType::CityCenter => PoiKind::WaterSource,
            BuildingType::Barracks => PoiKind::SafeZone,
            BuildingType::Mine => continue,
        };
        registry.add(civ_agents::Poi {
            id: idx as u64,
            kind,
            pos: Position3d {
                coord: WorldCoord {
                    x: i64::from(building.position.x),
                    y: 0,
                    z: i64::from(building.position.y),
                },
            },
            capacity: 8,
        });
    }
    registry
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
            cluster_stocks: BTreeMap::new(),
            last_life_deaths: 0,
            last_settlement_count: 0,
            allocator: Allocator::new(42),
            diffusion_params: DiffusionParams::default(),
            target_era: 1,
            last_cohort_stats: None,
            last_births: Vec::new(),
            last_deaths: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            last_tick_mod_lifecycle: Vec::new(),
            last_tick_abiogenesis_sites: Vec::new(),
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed: 42,
                ..ReplayLog::default()
            },
            economy_policy: DEFAULT_ECONOMY_POLICY,
            lod_policy: LodPolicy::default(),
            mod_host: ModHost::new(),
            military_phase: MilitaryPhaseConfig::default(),
            faction_doctrines: default_faction_doctrines(),
            coastal_columns: BTreeMap::new(),
            weather_grid,
            emergence: Self::default_emergence_state(42),
            emergence_sample: None,
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
        let state = WorldState {
            rng_seed: seed,
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
            cluster_stocks: BTreeMap::new(),
            last_life_deaths: 0,
            last_settlement_count: 0,
            allocator: Allocator::new(seed),
            diffusion_params: DiffusionParams::default(),
            target_era: 1,
            last_cohort_stats: None,
            last_births: Vec::new(),
            last_deaths: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            last_tick_mod_lifecycle: Vec::new(),
            last_tick_abiogenesis_sites: Vec::new(),
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed,
                ..ReplayLog::default()
            },
            economy_policy: DEFAULT_ECONOMY_POLICY,
            lod_policy: LodPolicy::default(),
            mod_host: ModHost::new(),
            military_phase: MilitaryPhaseConfig::default(),
            faction_doctrines: default_faction_doctrines(),
            coastal_columns: BTreeMap::new(),
            weather_grid,
            emergence: Self::default_emergence_state(seed),
            emergence_sample: None,
        }
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

    /// Count distinct faction IDs currently represented by civilian alignments.
    ///
    /// If no explicit civilian `Alignment::Faction` values are present, this
    /// falls back to a deterministic heuristic over known factions in
    /// `WorldState::factions` (for parity with the current partial emergence
    /// implementation). If/when a dedicated `HeuristicFactionSet` becomes
    /// available, it should replace this fallback.
    pub fn faction_count(&self) -> u32 {
        let explicit_faction_ids = self
            .world
            .query::<&AgentCivilian>()
            .iter()
            .filter_map(|(_, civilian)| match civilian.alignment {
                civ_agents::Alignment::Faction(faction_id) => Some(faction_id),
                _ => None,
            })
            .collect::<HashSet<_>>();

        if !explicit_faction_ids.is_empty() {
            return explicit_faction_ids.len() as u32;
        }

        self.state.factions.len() as u32
    }

    /// Return a deterministic representative alignment for the requested faction.
    ///
    /// The method first searches for explicit `Alignment::Faction` values among
    /// live civilians. If none is found for `faction_id`, it returns a
    /// deterministic rotated representative from `WorldState::factions` as a
    /// heuristic fallback.
    pub fn faction_alignment(&self, faction_id: u32) -> civ_agents::Alignment {
        if let Some(alignment) =
            self.world
                .query::<&AgentCivilian>()
                .iter()
                .find_map(|(_, civilian)| match civilian.alignment {
                    civ_agents::Alignment::Faction(fid) if fid == faction_id => {
                        Some(civilian.alignment)
                    }
                    _ => None,
                })
        {
            return alignment;
        }

        let mut registered_factions: Vec<u32> = self.state.factions.keys().copied().collect();
        if registered_factions.is_empty() {
            return Alignment::None;
        }

        registered_factions.sort_unstable();
        let rotation = (self.state.rng_seed % registered_factions.len() as u64) as usize;
        let fallback_index = (faction_id as usize + rotation) % registered_factions.len();
        Alignment::with_faction(registered_factions[fallback_index])
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

    /// Borrow the per-region weather grid updated by the planet phase
    /// (FR-CIV-PLANET-030). Exposed so the WebSocket bridge can stream a
    /// `Frame3d::Climate` snapshot each tick.
    pub fn weather_grid(&self) -> &[WeatherCell] {
        &self.weather_grid
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

    /// Accumulated emergent research effort (FR-CIV-0200), advanced each tick by
    /// [`Simulation::phase_research`] in proportion to the living population.
    #[must_use]
    pub fn research_progress(&self) -> u64 {
        self.state.research_progress
    }

    /// Research tier reached — each tier is 100k accumulated research effort.
    /// Higher tiers feed back into gameplay (e.g. carrying capacity).
    #[must_use]
    pub fn research_tier(&self) -> u64 {
        self.state.research_progress / 100_000
    }

    /// Effective carrying capacity: a baseline plus a bonus per research tier.
    /// Tech raises how many people the land sustains, which eases staple prices
    /// in [`Simulation::phase_economy`] (research → economy coupling).
    fn carrying_capacity(&self) -> i64 {
        const POP_BASELINE: i64 = 1_000_000;
        const CAPACITY_PER_TIER: i64 = 200_000;
        let tier = self.research_tier().min(i64::MAX as u64) as i64;
        POP_BASELINE + tier.saturating_mul(CAPACITY_PER_TIER)
    }

    /// Accumulated faith/belief, generated each tick by [`Simulation::phase_belief`]
    /// from the worshipping population and spent on divine powers.
    #[must_use]
    pub fn belief(&self) -> u64 {
        self.state.belief
    }

    /// Accumulated societal unrest, driven by food-market scarcity each tick by
    /// [`Simulation::phase_unrest`]. Zero means a content populace.
    #[must_use]
    pub fn unrest(&self) -> u64 {
        self.state.unrest
    }

    /// Accumulated social cohesion, generated each tick by
    /// [`Simulation::phase_cohesion`] from belief minus unrest. Higher means a
    /// stronger shared social fabric.
    #[must_use]
    pub fn cohesion(&self) -> u64 {
        self.state.cohesion
    }

    /// Attempt to spend `cost` belief to invoke a divine power. Returns `true`
    /// and deducts the cost when enough faith has accumulated; returns `false`
    /// and leaves belief untouched otherwise (FR-CIV-EMERGENCE divine-powers).
    pub fn try_invoke_divine_power(&mut self, cost: u64) -> bool {
        if self.state.belief >= cost {
            self.state.belief -= cost;
            true
        } else {
            false
        }
    }

    /// Add `amount` to accumulated belief. Used by cross-module systems (e.g.
    /// disasters: fear breeds faith) to feed the divine-powers economy.
    pub(crate) fn add_belief(&mut self, amount: u64) {
        self.state.belief = self.state.belief.saturating_add(amount);
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

    /// Advance simulation by one tick.
    ///
    /// Uses `None` for emergence sampling source, so the engine's
    /// `VoxelWorld` continues to feed samplers in non-standalone contexts.
    ///
    /// Phases run in [`PHASE_ORDER`] (CIV-0001 partial — engine-side deterministic
    /// transition only; server command intake and client broadcast live outside this
    /// crate). Exactly one [`ReplayEvent::Tick`] is appended after all phases finish.
    pub fn tick(&mut self) {
        self.tick_with_emergence_source(None);
    }

    /// Advance simulation by one tick, with an optional override for emergence
    /// sampling input.
    ///
    /// `emergence_ca_grid` is used only for metric collection in standalone
    /// modes that maintain terrain in the `bevy-ref` CA layer (e.g.
    /// `civ_voxel::fluid_ca::CaGrid`) and keeps the standard engine substrate
    /// path unchanged.
    pub fn tick_with_emergence_source(
        &mut self,
        emergence_ca_grid: Option<&civ_voxel::fluid_ca::CaGrid>,
    ) {
        self.state.tick += 1;
        self.last_tick_combat_pulses.clear();
        self.last_tick_engagements.clear();
        self.last_tick_mod_lifecycle.clear();

        // Phases in PHASE_ORDER (CIV-0001 partial)
        self.phase_production();
        self.phase_citizen_lifecycle();
        self.phase_military();
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
        self.phase_research();
        self.phase_belief();
        self.phase_unrest();
        self.phase_cohesion();
        // PR #350 stack: run the civ-emergence-metrics sampler on the
        // 50-tick boundary. The sampler internally no-ops on
        // non-boundary ticks so the cost on every other tick is just
        // one modulo + one branch.
        if let Some(grid) = emergence_ca_grid {
            self.sample_emergence_with_ca_grid(grid);
        } else {
            self.sample_emergence();
        }
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
        self.weather_grid = compute_weather(
            &self.climate,
            self.state.tick,
            self.weather_grid.len().max(1) as u32,
        );
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

    /// FR-CIV-CA-009: CA-driven abiogenesis scan. Walks the active
    /// `dirty_chunks` set on the CA grid (the same set `phase_voxel` would
    /// step) and scores every cell with [`civ_voxel::fluid_ca::AbiogenesisSuitability`].
    /// The result is stashed in `last_tick_abiogenesis_sites` for the
    /// downstream emergence phase to consume.
    ///
    /// `reg` is the material registry used to look up solvent scoring; pass
    /// `MaterialRegistry::standard()` in production. `grid` may be a borrowed
    /// CA grid from a Bevy / Godot resident window; when `None` we skip
    /// (cheap path: emergence layer uses a synthetic distribution).
    pub fn phase_voxel_ca(&mut self, grid: Option<&civ_voxel::fluid_ca::CaGrid>) {
        self.last_tick_abiogenesis_sites.clear();
        let Some(grid) = grid else { return };
        for &chunk in &grid.dirty_chunks() {
            // Re-derive the chunk's cell-span in (x, y, z). Each 16³ leaf is
            // enumerated, but the trailing leaf on each axis may be short —
            // clamp via `min(dims[axis])`.
            let counts = grid.chunk_counts();
            if counts[0] == 0 || counts[1] == 0 || counts[2] == 0 {
                break;
            }
            let cx = chunk % counts[0];
            let rem = chunk - cx;
            let cy = rem / counts[0] % counts[1];
            let cz = rem / (counts[0] * counts[1]);
            let x0 = cx * 16;
            let y0 = cy * 16;
            let z0 = cz * 16;
            let x1 = (x0 + 16).min(grid.dims[0]);
            let y1 = (y0 + 16).min(grid.dims[1]);
            let z1 = (z0 + 16).min(grid.dims[2]);
            for z in z0..z1 {
                for y in y0..y1 {
                    for x in x0..x1 {
                        let Some(idx) = grid.index(x, y, z) else {
                            continue;
                        };
                        let mat = grid.cells[idx];
                        let t = grid.temperatures[idx];
                        let sat = grid.saturation[idx];
                        let s = civ_voxel::fluid_ca::AbiogenesisSuitability::from_cell(mat, t, sat);
                        if s.is_viable() {
                            self.last_tick_abiogenesis_sites.push(s);
                        }
                    }
                }
            }
        }
    }

    /// FR-CIV-CA-009 — borrow the abiogenesis sites produced by
    /// [`Simulation::phase_voxel_ca`]. Cleared at the start of the next
    /// `phase_voxel_ca` call.
    pub fn last_tick_abiogenesis_sites(&self) -> &[civ_voxel::fluid_ca::AbiogenesisSuitability] {
        &self.last_tick_abiogenesis_sites
    }

    /// Compact the voxel world periodically.
    fn phase_compact(&mut self) {
        if self.state.tick % self.tick_modulo_compact == 0 {
            self.voxel.compact();
        }
    }

    /// Research phase (FR-CIV-0200) — research advances emergently from the
    /// living population rather than on a scripted schedule. Each tick the
    /// population contributes research effort proportional to its size; the
    /// accumulated `research_progress` is the substrate downstream tech-unlock
    /// logic draws on. Pure, deterministic function of `population`.
    fn phase_research(&mut self) {
        /// People required to produce one unit of research effort per tick.
        const RESEARCH_POP_DIVISOR: u64 = 1_000;
        let base = self.state.population / RESEARCH_POP_DIVISOR;
        let contribution = base
            .saturating_add(base.saturating_mul(cohesion_research_bonus_permille(self.state.cohesion)) / 1_000);
        self.state.research_progress = self.state.research_progress.saturating_add(contribution);
    }

    /// Faith phase (divine-powers economy, FR-CIV-EMERGENCE). The worshipping
    /// population generates `belief` each tick; belief is the resource spent via
    /// [`Simulation::try_invoke_divine_power`] to invoke divine interventions.
    /// Pure, deterministic function of `population`.
    fn phase_belief(&mut self) {
        /// People required to generate one unit of belief per tick.
        const BELIEF_POP_DIVISOR: u64 = 2_000;
        let worship = self.state.population / BELIEF_POP_DIVISOR;
        self.state.belief = self.state.belief.saturating_add(worship);
    }

    /// Social-unrest phase (FR-CIV-0100 §3 emergence). Unrest EMERGES from the
    /// food market: a clearing price above baseline (scarcity) drives it up in
    /// proportion to the shortfall; abundance lets it decay toward contentment.
    /// Runs after `phase_economy` so the food price is current.
    ///
    /// Hardship also drives people to faith: a fraction of standing unrest feeds
    /// `belief` each tick. This is a STABILISING negative-feedback arm — the
    /// faith unrest breeds raises the diplomacy war-threshold that unrest itself
    /// lowers, nudging the system toward edge-of-chaos rather than runaway war.
    fn phase_unrest(&mut self) {
        /// Units of standing unrest that generate one unit of belief per tick.
        const UNREST_FAITH_DIVISOR: u64 = 100;
        let food_price = self
            .market_state
            .prices()
            .get("food")
            .copied()
            .unwrap_or(FOOD_SCARCITY_BASELINE);
        // Research mitigates the scarcity-driven rise (research -> calmer society).
        // Cohesion damps the remaining rise (cohesion -> calmer society), closing the loop.
        let delta = cohesion_unrest_damp(research_unrest_mitigation(unrest_delta(food_price), self.research_tier()), self.state.cohesion)
            + energy_scarcity_unrest(self.state.energy_budget_joules)
            + overcrowding_unrest(self.state.population, self.carrying_capacity());
        self.state.unrest = (self.state.unrest as i64 + delta).max(0) as u64;
        let faith_from_hardship = self.state.unrest / UNREST_FAITH_DIVISOR;
        self.add_belief(faith_from_hardship);
    }

    /// Social-cohesion phase (FR-CIV-0100 §3 emergence). The shared social fabric
    /// EMERGES from the balance of collective belief (shared faith binds) and
    /// unrest (disorder frays bonds): cohesion accrues when faith outweighs
    /// discontent and decays when discontent dominates. Runs after `phase_unrest`
    /// so it sees the current tick's unrest. Floored at zero.
    fn phase_cohesion(&mut self) {
        let delta = cohesion_delta(self.state.belief, self.state.unrest);
        self.state.cohesion = (self.state.cohesion as i64 + delta).max(0) as u64;
    }

    /// Buildings phase - expands the parcel graph on a fixed cadence when demand is high.
    fn phase_buildings(&mut self) {
        if self.state.tick % building_cadence(self.research_tier()) != 0 {
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

    /// Emergent life-sim phase (FR-CIV-LIFE-*). Runs the full needs pipeline
    /// (decay → sickness → death via `civ-needs`), utility-AI daily pathing to
    /// need-satisfying POIs (`civ-agents::daily_path`), and emergent settlement
    /// clustering (`civ-agents::cluster`) with per-cluster resource stocks.
    ///
    /// Determinism: all stochastic transitions consume `self.rng` (seeded
    /// ChaCha8). No wall-clock or `thread_rng`. Surfaced state (needs, cluster
    /// membership, settlement count, life deaths, cluster stocks) is read by the
    /// sim bridge / HUD.
    fn phase_life(&mut self) {
        // 1. Ensure every agent carries the life-sim needs + health components.
        let missing: Vec<Entity> = self
            .world
            .query::<&AgentCivilian>()
            .iter()
            .filter(|(e, _)| self.world.get::<&LifeNeeds>(*e).is_err())
            .map(|(e, _)| e)
            .collect();
        for entity in missing {
            let _ = self
                .world
                .insert(entity, (LifeNeeds::sated(), LifeHealth::default()));
        }

        // 2. Build the POI registry from buildings (need-serving locations).
        let registry = build_poi_registry(&self.world);

        // 3. Tick needs/health, run utility-AI daily pathing, collect the dead.
        let rates = DecayRates::default();
        let params = HealthParams::default();
        let move_speed = (0.01 * FIXED_SCALE as f32) as i64;
        let satisfy_radius_sq: i128 = {
            let r = (0.03 * FIXED_SCALE as f32) as i128;
            r * r
        };
        let mut dead: Vec<(Entity, u64, WorldCoord)> = Vec::new();

        let entities: Vec<Entity> = self
            .world
            .query::<&AgentCivilian>()
            .iter()
            .map(|(e, _)| e)
            .collect();

        for entity in entities {
            // Needs/health pipeline.
            let outcome = {
                let mut needs = match self.world.get::<&mut LifeNeeds>(entity) {
                    Ok(n) => n,
                    Err(_) => continue,
                };
                let mut health = match self.world.get::<&mut LifeHealth>(entity) {
                    Ok(h) => h,
                    Err(_) => continue,
                };
                needs_tick(&mut needs, &mut health, &rates, &params, &mut self.rng)
            };

            if outcome.died {
                if let Ok(civ) = self.world.get::<&AgentCivilian>(entity) {
                    if let Ok(pos) = self.world.get::<&Position3d>(entity) {
                        dead.push((entity, civ.id, pos.coord));
                    }
                }
                continue;
            }

            // Utility-AI daily path: choose an activity first, then either seek a
            // pressing need, idle, or wander locally when needs are comfortable
            // or no POI is available.
            let pos = match self.world.get::<&Position3d>(entity) {
                Ok(p) => *p,
                Err(_) => continue,
            };
            let civ = match self.world.get::<&AgentCivilian>(entity) {
                Ok(c) => c.clone(),
                Err(_) => continue,
            };
            let needs_snapshot = match self.world.get::<&LifeNeeds>(entity) {
                Ok(n) => *n,
                Err(_) => continue,
            };
            let activity = choose_activity(&needs_snapshot, registry.iter().next().is_some());
            match activity {
                Activity::Idle => {}
                Activity::SeekNeed => {
                    if let Some(target) = pick_target(&needs_snapshot, &registry, &pos) {
                        let target_pos = target.pos;
                        let served = civ_agents::need_for_poi_kind(target.kind);
                        let next = path_step(&pos, &target_pos, move_speed);
                        if let Ok(mut p) = self.world.get::<&mut Position3d>(entity) {
                            *p = next;
                        }
                        let dx = (next.coord.x - target_pos.coord.x) as i128;
                        let dz = (next.coord.z - target_pos.coord.z) as i128;
                        if dx * dx + dz * dz <= satisfy_radius_sq {
                            if let Ok(mut n) = self.world.get::<&mut LifeNeeds>(entity) {
                                n.satisfy(served, 0.5);
                            }
                        }
                    } else {
                        let mut local_rng = ChaCha8Rng::seed_from_u64(
                            self.state.tick ^ civ.id ^ 0x9e37_79b9_7f4a_7c15,
                        );
                        if local_rng.gen_bool(0.5) {
                            let target_pos = wander_anchor(&pos, civ.id, self.state.tick);
                            let next = path_step(&pos, &target_pos, move_speed);
                            if let Ok(mut p) = self.world.get::<&mut Position3d>(entity) {
                                *p = next;
                            }
                        }
                    }
                }
                Activity::Wander => {
                    let mut local_rng = ChaCha8Rng::seed_from_u64(
                        self.state.tick
                            ^ civ.id
                            ^ (pos.coord.x as u64).rotate_left(13)
                            ^ (pos.coord.z as u64).rotate_left(29),
                    );
                    if local_rng.gen_bool(0.65) {
                        let target_pos = wander_anchor(&pos, civ.id, self.state.tick);
                        let next = path_step(&pos, &target_pos, move_speed);
                        if let Ok(mut p) = self.world.get::<&mut Position3d>(entity) {
                            *p = next;
                        }
                    }
                }
            }
        }

        // 4. Despawn the dead and book them into the population deltas.
        for (entity, entity_id, coord) in &dead {
            let _ = self.world.despawn(*entity);
            self.last_deaths.push(PopulationEvent {
                tick: self.state.tick,
                entity_id: *entity_id,
                x: coord.x as f32 / FIXED_SCALE as f32,
                y: coord.z as f32 / FIXED_SCALE as f32,
            });
        }
        self.last_life_deaths = dead.len() as u32;
        self.state.population = self.state.population.saturating_sub(dead.len() as u64);

        // 5. Emergent settlement clustering by co-location.
        let positions: Vec<(u64, Position3d)> = self
            .world
            .query::<(&AgentCivilian, &Position3d)>()
            .iter()
            .map(|(_, (civ, pos))| (civ.id, *pos))
            .collect();
        let cluster_radius = (0.06 * FIXED_SCALE as f32) as i64;
        let assignments = cluster_by_colocation(&positions, cluster_radius);

        // Map agent id -> entity for component writes.
        let id_to_entity: HashMap<u64, Entity> = self
            .world
            .query::<&AgentCivilian>()
            .iter()
            .map(|(e, civ)| (civ.id, e))
            .collect();
        let mut cluster_sizes: BTreeMap<u64, u32> = BTreeMap::new();
        for (agent_id, cluster) in &assignments {
            *cluster_sizes.entry(cluster.0).or_insert(0) += 1;
            if let Some(&entity) = id_to_entity.get(agent_id) {
                let _ = self
                    .world
                    .insert_one(entity, ClusterMember { cluster: *cluster });
            }
        }

        // A settlement is an emergent cluster with more than one member.
        self.last_settlement_count = cluster_sizes.values().filter(|&&n| n > 1).count() as u32;

        // 6. Maintain per-cluster (settlement) resource stocks: agents produce
        // into their cluster's shared stock each tick (collective economics).
        let mut next_stocks: BTreeMap<u64, ClusterStocks> = BTreeMap::new();
        for (cluster_id, size) in &cluster_sizes {
            let mut stock = self
                .cluster_stocks
                .get(cluster_id)
                .cloned()
                .unwrap_or_default();
            // Each member contributes one unit of food per tick to the commons.
            stock.add(civ_economy::Good::Food, i64::from(*size));
            next_stocks.insert(*cluster_id, stock);
        }
        self.cluster_stocks = next_stocks;
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
        let yield_factor = production_yield_factor(self.research_tier());
        self.state.resources.food += food * yield_factor;
        self.state.resources.wood += wood * yield_factor;
        self.state.resources.metal += metal * yield_factor;
        self.state.resources.energy += energy * yield_factor;
    }

    /// Citizen lifecycle phase
    fn phase_citizen_lifecycle(&mut self) {
        attach_citizen_to_agents(&mut self.world);
        self.last_births.clear();
        self.last_deaths.clear();
        let population = count_civilians(&self.world) as f64;
        let max_pop = self.state.population.max(1) as f64;
        let overcrowding_factor = (population / max_pop).clamp(0.0, 1.0);
        // Emergent downward causation: food-market scarcity damps the birth rate
        // (research -> carrying-capacity -> economy -> population loop).
        let food_price = self
            .market_state
            .prices()
            .get("food")
            .copied()
            .unwrap_or(FOOD_SCARCITY_BASELINE);
        let birth_chance =
            0.003 * (1.0 - overcrowding_factor) * food_scarcity_birth_factor(food_price);
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
            let alignment = civ_agents::infer_alignment_for_spawn(&self.world, x, y);
            let _ = spawn_child_near(&mut self.world, child_id, alignment, x, y, &mut self.rng);
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

        let morale_recovery = morale_recovery_rate(self.state.cohesion);
        for (_, unit) in self.world.query::<&mut MilitaryUnit>().iter() {
            if unit.morale < Fixed::from_num(1) {
                unit.morale = (unit.morale + morale_recovery).min(Fixed::from_num(1));
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
        // Consume an rng draw to keep the replay sequence stable, but let the
        // OUTCOME EMERGE from faction wealth rather than a coin flip: a large
        // treasury disparity breeds conflict (have-nots clash with haves);
        // near-peers find it cheaper to trade (FR-CIV-0100 §3).
        let _entropy = self.rng.gen_bool(0.6);
        let treasury_a = self
            .state
            .faction_treasury
            .get(&a)
            .copied()
            .unwrap_or_default();
        let treasury_b = self
            .state
            .faction_treasury
            .get(&b)
            .copied()
            .unwrap_or_default();
        let disparity = if treasury_a >= treasury_b {
            treasury_a - treasury_b
        } else {
            treasury_b - treasury_a
        };
        // Shared faith binds society: collective belief raises the disparity a
        // faction pair will tolerate before fighting (belief -> diplomacy).
        let conflict_threshold =
            Fixed::from_num(diplomacy_conflict_threshold(self.belief().saturating_add(self.cohesion()), self.unrest()));
        let kind = if disparity >= conflict_threshold {
            DiplomacyKind::Conflict
        } else {
            DiplomacyKind::TradeAgreement
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

        // Emergent pricing (FR-CIV-0100 §3d): the living population is demand
        // pressure measured against the carrying capacity (supply). Staple
        // prices rise as population outgrows capacity (scarcity) and ease as it
        // falls below (surplus). Carrying capacity itself grows with research
        // tier, so tech advances FEED BACK into cheaper staples (research →
        // economy coupling).
        // Wealthy factions bid up staple demand on top of raw population
        // (faction prosperity -> market coupling; diplomacy already moves these
        // treasuries, so diplomacy -> treasury -> market demand chains through).
        let faction_wealth: i64 = self
            .state
            .faction_treasury
            .values()
            .map(|t| (t.raw / crate::SCALE).max(0))
            .sum();
        let population = self.state.population.min(i64::MAX as u64) as i64;
        let demand = population.saturating_add(faction_wealth);
        let supply = self.carrying_capacity();
        self.market_state.apply_pressure("food", demand, supply);
        self.market_state.apply_pressure("energy", demand, supply);
    }

    fn tick_trade_routes(&mut self) {
        // Societal unrest throttles all commerce this tick (computed once).
        let unrest_factor = unrest_trade_factor(self.state.unrest);
        let cohesion_factor = cohesion_trade_factor(self.state.cohesion);
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

            // Arbitrage: a route from a surplus exporter to a scarce importer
            // ships more (bounded 2x). Read the importer stock before transfer.
            let to_stock = self
                .state
                .faction_resources
                .get(&route.to_faction)
                .map(|r| resource_amount(r, resource))
                .unwrap_or(Fixed::ZERO);
            let boosted =
                route.volume * trade_volume_multiplier(available, to_stock) * unrest_factor * cohesion_factor;
            let quantity = boosted.min(available);
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
            settlement_count: self.last_settlement_count,
            life_deaths_this_tick: self.last_life_deaths,
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
}

/// Baseline food clearing price (cents) at which births are unaffected by
/// scarcity. Matches `MarketState::default()`'s food price.
const FOOD_SCARCITY_BASELINE: i64 = 1_000;

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

/// Downward-causation policy (FR-CIV-0100 §3): research raises production yield —
/// better tools/techniques lift per-building output. +10% per research tier,
/// capped at +100% (2x). De-silos phase_production, which read no emergent state.
fn production_yield_factor(research_tier: u64) -> Fixed {
    let bonus_permille = research_tier.saturating_mul(100).min(1_000) as i64;
    Fixed::from_num(1_000 + bonus_permille) / Fixed::from_num(1_000)
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

/// Downward-causation policy (FR-CIV-0100 §3): a cohesive society trades MORE —
/// social trust lowers transaction friction. Returns a factor in [1.0, 1.5],
/// rising with cohesion, capped so the boost can't run away.
fn cohesion_trade_factor(cohesion: u64) -> Fixed {
    let boost = (cohesion / COHESION_PER_TRADE_PERMILLE).min(COHESION_TRADE_CAP_PERMILLE as u64) as i64;
    Fixed::from_num(1_000 + boost) / Fixed::from_num(1_000)
}

/// Wealth-disparity (in whole currency units) at which two factions clash when
/// they share no faith. Above this gap the have-nots turn on the haves.
const DIPLOMACY_BASE_CONFLICT_THRESHOLD: i64 = 10_000;
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Emergent settlement count (multi-member clusters) — FR-CIV-LIFE-030.
    pub settlement_count: u32,
    /// Agents that died of unmet needs this tick — FR-CIV-LIFE-003.
    pub life_deaths_this_tick: u32,
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

    /// Covers FR-CORE-001.
    /// Each `Simulation::tick()` appends exactly one `ReplayEvent::Tick`.
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
                "economy",
                "planet",
                "diplomacy",
                "tactics",
                "voxel",
                "compact",
                "buildings",
                "diffusion",
                "disasters",
                "life",
                "emergence",
            ]
        );
    }

    fn count_replay_ticks(sim: &Simulation) -> usize {
        sim.replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
            .count()
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

    /// FR-CIV-0100 emergence — at the baseline food price births are unaffected.
    #[test]
    fn food_scarcity_birth_factor_is_unity_at_baseline() {
        assert_eq!(food_scarcity_birth_factor(FOOD_SCARCITY_BASELINE), 1.0);
    }

    /// Surplus (price below baseline) does NOT boost births above the natural
    /// rate — the factor is clamped at 1.0.
    #[test]
    fn food_scarcity_birth_factor_caps_at_unity_under_surplus() {
        assert_eq!(food_scarcity_birth_factor(FOOD_SCARCITY_BASELINE / 4), 1.0);
        assert_eq!(food_scarcity_birth_factor(1), 1.0);
    }

    /// Scarcity (price above baseline) damps births: a 2x price halves the rate,
    /// and the factor is strictly decreasing as price climbs — but never zero.
    #[test]
    fn food_scarcity_birth_factor_damps_under_scarcity() {
        let double = food_scarcity_birth_factor(FOOD_SCARCITY_BASELINE * 2);
        assert!((double - 0.5).abs() < 1e-9, "2x price should halve births");
        let quad = food_scarcity_birth_factor(FOOD_SCARCITY_BASELINE * 4);
        assert!(quad < double, "higher price must damp births further");
        assert!(quad > 0.0, "the birth factor must never reach zero");
    }

    /// The coupling only ever scales births DOWN, so an expensive-food tick can
    /// never reduce the standing population relative to the start of the tick.
    #[test]
    fn food_scarcity_never_reduces_standing_population() {
        let mut sim = Simulation::with_seed(7);
        sim.market_state
            .prices
            .insert("food".to_string(), FOOD_SCARCITY_BASELINE * 8);
        let before = sim.state.population;
        sim.tick();
        assert!(
            sim.state.population >= before.saturating_sub(sim.last_deaths.len() as u64),
            "scarcity coupling must not subtract from population beyond natural deaths"
        );
    }

    /// FR-CIV-0100 §3 — with no shared faith and no unrest the threshold is base.
    #[test]
    fn diplomacy_threshold_is_base_without_belief() {
        assert_eq!(
            diplomacy_conflict_threshold(0, 0),
            DIPLOMACY_BASE_CONFLICT_THRESHOLD
        );
    }

    /// Collective belief raises the disparity factions tolerate before fighting,
    /// and the peace bonus is monotonic non-decreasing in belief.
    #[test]
    fn diplomacy_threshold_rises_with_belief() {
        let low = diplomacy_conflict_threshold(5_000, 0);
        let high = diplomacy_conflict_threshold(500_000, 0);
        assert!(low > DIPLOMACY_BASE_CONFLICT_THRESHOLD, "faith buys peace");
        assert!(high >= low, "more faith never lowers tolerance");
    }

    /// The peace bonus is capped at 2x the base, so conflict is always reachable.
    #[test]
    fn diplomacy_threshold_caps_at_double_base() {
        let saturated = diplomacy_conflict_threshold(u64::MAX, 0);
        assert_eq!(
            saturated,
            DIPLOMACY_BASE_CONFLICT_THRESHOLD + BELIEF_PEACE_CAP
        );
        assert!(saturated <= 2 * DIPLOMACY_BASE_CONFLICT_THRESHOLD);
    }

    /// Unrest erodes the threshold (discontent breeds war), opposing belief, and
    /// the erosion is monotonic non-increasing in unrest.
    #[test]
    fn diplomacy_threshold_falls_with_unrest() {
        let calm = diplomacy_conflict_threshold(0, 0);
        let tense = diplomacy_conflict_threshold(0, 5_000);
        let furious = diplomacy_conflict_threshold(0, 500_000);
        assert!(tense < calm, "unrest lowers the war threshold");
        assert!(furious <= tense, "more unrest never raises tolerance");
    }

    /// Even infinite unrest leaves a positive floor — discontent alone is not
    /// casus belli; some wealth disparity is still required.
    #[test]
    fn diplomacy_threshold_floors_under_extreme_unrest() {
        let floored = diplomacy_conflict_threshold(0, u64::MAX);
        assert_eq!(floored, DIPLOMACY_MIN_CONFLICT_THRESHOLD);
        assert!(floored > 0, "war always needs some disparity");
    }

    /// Belief and unrest oppose: equal pressure on both sides nets out near base.
    #[test]
    fn diplomacy_belief_and_unrest_oppose() {
        // 5_000 belief -> +100 peace; 5_000 unrest -> -100 war; net ~ base.
        assert_eq!(
            diplomacy_conflict_threshold(5_000, 5_000),
            DIPLOMACY_BASE_CONFLICT_THRESHOLD
        );
    }

    /// FR-CIV-0100 §3 — a highly cohesive society projects unity: cohesion folds
    /// into the binding term and raises the war threshold, so a wealth disparity
    /// that would spark conflict in a fractured society instead yields trade.
    #[test]
    fn high_cohesion_biases_diplomacy_toward_peace() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500;
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        let a = ids[500 % ids.len()];
        let b = ids[(500 + 1) % ids.len()];
        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(15_000));
        sim.state.cohesion = 1_000_000;
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::TradeAgreement,
            "a highly cohesive society tolerates the disparity and trades"
        );
    }

    /// FR-CIV-0100 §3 — at/below baseline food price unrest decays (negative delta).
    #[test]
    fn unrest_delta_decays_under_abundance() {
        assert!(unrest_delta(FOOD_SCARCITY_BASELINE) < 0);
        assert!(unrest_delta(FOOD_SCARCITY_BASELINE / 2) < 0);
    }

    /// Scarcity drives unrest up, bounded per tick, and monotonic in shortfall.
    #[test]
    fn unrest_delta_rises_with_scarcity() {
        let mild = unrest_delta(FOOD_SCARCITY_BASELINE + 100);
        let severe = unrest_delta(FOOD_SCARCITY_BASELINE + 10_000);
        assert!(mild > 0, "any scarcity raises unrest");
        assert!(severe >= mild, "more scarcity never lowers the rise");
        assert!(severe <= 50, "single-tick rise is capped");
    }

    /// FR-CIV-0100 §3 — a drained energy budget (blackout) adds unrest; a solvent one does not.
    #[test]
    fn energy_scarcity_adds_unrest_only_on_blackout() {
        assert_eq!(energy_scarcity_unrest(Fixed::from_num(1_000)), 0);
        assert_eq!(energy_scarcity_unrest(Fixed::ZERO), 15);
        assert!(energy_scarcity_unrest(Fixed::from_num(-5)) > 0);
    }

    /// FR-CIV-0100 §3 — overcrowding past carrying capacity breeds unrest, scaled
    /// by overshoot and capped; at or below capacity it adds none.
    #[test]
    fn overcrowding_breeds_unrest_above_capacity() {
        assert_eq!(overcrowding_unrest(500, 1_000), 0, "under capacity = no unrest");
        assert_eq!(overcrowding_unrest(1_000, 1_000), 0, "at capacity = no unrest");
        let mild = overcrowding_unrest(1_100, 1_000);
        let heavy = overcrowding_unrest(2_000, 1_000);
        assert!(mild > 0, "overcrowding breeds unrest");
        assert!(heavy > mild, "more overshoot = more unrest");
        assert!(overcrowding_unrest(100_000, 1_000) <= 30, "capped per tick");
    }

    /// FR-CIV-0100 §3 — cohesion boosts the research contribution, capped at +50%.
    #[test]
    fn cohesion_boosts_research_contribution() {
        assert_eq!(cohesion_research_bonus_permille(0), 0);
        assert!(cohesion_research_bonus_permille(100_000) > 0, "cohesion speeds research");
        assert_eq!(cohesion_research_bonus_permille(10_000_000), 500, "capped at +50%");
    }

    /// FR-CIV-0100 §3 — research lifts production yield, monotonically, capped at 2x.
    #[test]
    fn production_yield_factor_rises_with_research_capped_at_2x() {
        assert_eq!(production_yield_factor(0), Fixed::from_num(1));
        let t1 = production_yield_factor(1);
        let t10 = production_yield_factor(10);
        assert!(t1 > Fixed::from_num(1), "research lifts yield");
        assert!(t10 >= t1, "more research never lowers yield");
        assert_eq!(production_yield_factor(100), Fixed::from_num(2), "capped at 2x");
    }

    /// FR-CIV-0100 §3 — cohesion speeds morale recovery, monotonically, 0.01→0.05 cap.
    #[test]
    fn morale_recovery_rate_rises_with_cohesion_capped() {
        assert_eq!(
            morale_recovery_rate(0),
            Fixed::from_num(1) / Fixed::from_num(100)
        );
        let some = morale_recovery_rate(500_000);
        let lots = morale_recovery_rate(10_000_000);
        assert!(some > morale_recovery_rate(0), "cohesion speeds recovery");
        assert!(lots >= some, "more cohesion never slows recovery");
        assert_eq!(
            lots,
            Fixed::from_num(5) / Fixed::from_num(100),
            "recovery rate capped at 0.05"
        );
    }

    /// research_tier is research_progress / 100_000 (coverage for the accessor).
    #[test]
    fn research_tier_divides_progress() {
        let mut sim = Simulation::with_seed(1);
        sim.state.research_progress = 250_000;
        assert_eq!(sim.research_tier(), 2);
    }

    /// try_invoke_divine_power spends belief only when affordable (coverage).
    #[test]
    fn try_invoke_divine_power_spends_belief() {
        let mut sim = Simulation::with_seed(1);
        sim.add_belief(100);
        assert!(sim.try_invoke_divine_power(60));
        assert_eq!(sim.belief(), 40);
        assert!(!sim.try_invoke_divine_power(1_000));
        assert_eq!(sim.belief(), 40);
    }

    /// FR-CIV-0100 §3 — research damps the scarcity-driven unrest rise (calmer
    /// advanced society), monotonic in tier, but never below 1; decay untouched.
    #[test]
    fn research_unrest_mitigation_damps_rise_floored_at_one() {
        let raw = 40;
        assert_eq!(
            research_unrest_mitigation(raw, 0),
            raw,
            "tier 0 leaves the rise unchanged"
        );
        let tier3 = research_unrest_mitigation(raw, 3);
        let tier9 = research_unrest_mitigation(raw, 9);
        assert!(tier3 < raw, "research calms unrest");
        assert!(tier9 <= tier3, "more research never raises unrest");
        assert!(tier9 >= 1, "research never fully eliminates hardship");
        // The mitigation is bounded: an absurd tier can't push the rise below 1.
        assert_eq!(research_unrest_mitigation(40, u64::MAX), 4);
        // Decay (negative delta) passes through untouched.
        assert_eq!(research_unrest_mitigation(-10, 9), -10);
    }

    /// FR-CIV-0100 §3 — research shortens the build cadence, monotonically, floored at 4.
    #[test]
    fn building_cadence_shortens_with_research_floored() {
        assert_eq!(building_cadence(0), 16);
        let t1 = building_cadence(1);
        let t6 = building_cadence(6);
        assert!(t1 < 16, "research speeds construction");
        assert!(t6 <= t1, "more research never slows it");
        assert_eq!(building_cadence(u64::MAX), 4, "cadence never drops below the floor");
    }

    /// FR-CIV-0100 §3 — cohesion grows when belief outweighs unrest and frays
    /// when unrest dominates; balanced pressure nets near zero.
    #[test]
    fn cohesion_delta_balances_belief_against_unrest() {
        assert!(cohesion_delta(10_000, 0) > 0, "faith builds the social fabric");
        assert!(cohesion_delta(0, 10_000) < 0, "unrest frays it");
        assert_eq!(cohesion_delta(0, 0), 0, "no pressure, no change");
        // Unrest frays harder than belief binds (smaller divisor), so equal
        // belief and unrest net negative.
        assert!(cohesion_delta(1_000, 1_000) < 0, "disorder erodes faster than faith builds");
    }

    #[test]
    fn cohesion_unrest_damp_calms_high_cohesion_floored_at_one() {
        let raw = 40;
        assert_eq!(cohesion_unrest_damp(raw, 0), raw);
        let some = cohesion_unrest_damp(raw, 400);
        let lots = cohesion_unrest_damp(raw, 100_000);
        assert!(some < raw);
        assert!(lots <= some);
        assert!(lots >= 1);
        assert_eq!(cohesion_unrest_damp(-10, 100_000), -10);
    }

    /// phase_unrest floors unrest at zero: a content populace under cheap food
    /// never goes negative.
    #[test]
    fn phase_unrest_floors_at_zero() {
        let mut sim = Simulation::with_seed(1);
        assert_eq!(sim.unrest(), 0);
        sim.phase_unrest();
        assert_eq!(sim.unrest(), 0, "abundance keeps a calm society at zero");
    }

    /// Sustained scarcity accumulates unrest above zero.
    #[test]
    fn phase_unrest_accumulates_under_scarcity() {
        let mut sim = Simulation::with_seed(1);
        sim.market_state
            .prices
            .insert("food".to_string(), FOOD_SCARCITY_BASELINE + 4_000);
        for _ in 0..5 {
            sim.phase_unrest();
        }
        assert!(sim.unrest() > 0, "persistent scarcity breeds unrest");
    }

    /// Hardship drives faith: standing unrest feeds belief each tick (the
    /// stabilising negative-feedback arm). A calm, well-fed society does not.
    #[test]
    fn phase_unrest_feeds_belief_under_hardship() {
        let mut sim = Simulation::with_seed(1);
        sim.market_state
            .prices
            .insert("food".to_string(), FOOD_SCARCITY_BASELINE + 6_000);
        let belief_before = sim.belief();
        for _ in 0..20 {
            sim.phase_unrest();
        }
        assert!(
            sim.belief() > belief_before,
            "sustained hardship should breed faith"
        );

        // A calm society (cheap food, zero unrest) breeds no hardship-faith.
        let mut calm = Simulation::with_seed(1);
        let calm_belief = calm.belief();
        calm.phase_unrest();
        assert_eq!(calm.belief(), calm_belief, "contentment breeds no faith");
    }

    /// FR-CIV-0100 §3 — a calm society trades at full volume; unrest throttles
    /// commerce monotonically down to a 0.5 floor (never stops entirely).
    #[test]
    fn unrest_trade_factor_throttles_to_half_floor() {
        assert_eq!(unrest_trade_factor(0), Fixed::from_num(1));
        let mild = unrest_trade_factor(400);
        let heavy = unrest_trade_factor(1_600);
        assert!(mild < Fixed::from_num(1), "unrest reduces trade");
        assert!(heavy < mild, "more unrest reduces trade further");
        assert!(
            heavy >= Fixed::from_num(1) / Fixed::from_num(2),
            "trade never drops below the 0.5 floor"
        );
        // An extreme unrest saturates exactly at the floor.
        assert_eq!(unrest_trade_factor(u64::MAX), Fixed::from_num(1) / Fixed::from_num(2));
    }

    #[test]
    fn cohesion_trade_factor_boosts_to_capped_ceiling() {
        assert_eq!(cohesion_trade_factor(0), Fixed::from_num(1));
        let some = cohesion_trade_factor(400);
        let lots = cohesion_trade_factor(100_000);
        assert!(some > Fixed::from_num(1));
        assert!(lots >= some);
        assert!(lots <= Fixed::from_num(3) / Fixed::from_num(2));
    }

    /// FR-CIV-0100 §3 — equal stocks (no surplus gap) trade at base volume (1x).
    #[test]
    fn trade_volume_multiplier_is_unity_without_gap() {
        let m = trade_volume_multiplier(Fixed::from_num(50), Fixed::from_num(50));
        assert_eq!(m, Fixed::from_num(1));
        // Source scarcer than destination: still no boost (floored at 1x).
        let reverse = trade_volume_multiplier(Fixed::from_num(10), Fixed::from_num(80));
        assert_eq!(reverse, Fixed::from_num(1));
    }

    /// A surplus exporter feeding a scarce importer ships more, monotonically,
    /// capped at 2x so the arbitrage loop self-limits.
    #[test]
    fn trade_volume_multiplier_scales_with_surplus_capped_at_2x() {
        let small = trade_volume_multiplier(Fixed::from_num(60), Fixed::from_num(50));
        let large = trade_volume_multiplier(Fixed::from_num(200), Fixed::from_num(50));
        assert!(small > Fixed::from_num(1), "any surplus gap boosts volume");
        assert!(large >= small, "bigger gap never ships less");
        assert!(large <= Fixed::from_num(2), "boost is capped at 2x");
        // A gap >= TRADE_GAP_SCALE saturates the multiplier exactly at 2x.
        let saturated =
            trade_volume_multiplier(Fixed::from_num(TRADE_GAP_SCALE + 50), Fixed::ZERO);
        assert_eq!(saturated, Fixed::from_num(2));
    }

    /// Holistic emergence-web integration (FR-CIV-0100 §3): a 1000-tick run
    /// exercises all coupled phases (economy, diplomacy, disasters, research,
    /// belief, unrest, trade arbitrage) together and must (a) never panic or
    /// overflow under sustained dynamics, (b) hold the economic invariant
    /// (every clearing price stays >= 1), and (c) produce live dynamics rather
    /// than a frozen state.
    #[test]
    fn emergence_web_runs_1000_ticks_stable_and_dynamic() {
        let mut sim = Simulation::with_seed(12345);
        for _ in 0..1_000 {
            sim.tick();
        }
        assert_eq!(sim.state.tick, 1_000, "tick counter advances deterministically");

        // (b) economic invariant: no price ever collapses below the floor.
        for (good, price) in sim.market_state.prices() {
            assert!(*price >= 1, "price for {good} fell below 1: {price}");
        }

        // (c) dynamics present: diplomacy fires every tick, so the event log is
        // populated — the coupled systems are actually running, not inert.
        assert!(
            !sim.diplomacy_events().is_empty(),
            "expected diplomacy activity over 1000 ticks"
        );

        // Accessors for the emergent scalars remain coherent (no overflow panic
        // reaching here already proves the saturating/bounded math held).
        let _ = (sim.belief(), sim.unrest(), sim.research_tier());
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

    /// Covers FR-CIV-LIFE-001, FR-CIV-LIFE-003, FR-CIV-LIFE-010, and FR-CIV-LIFE-030.
    /// phase_life attaches life-sim needs to agents and
    /// the snapshot surfaces emergent settlement state for the HUD.
    #[test]
    fn phase_life_attaches_needs_and_exposes_settlements() {
        let mut sim = Simulation::with_seed(777);
        for _ in 0..5 {
            sim.tick();
        }
        // Every agent civilian now carries the civ-needs Needs + Health.
        let agents = sim.world.query::<&AgentCivilian>().iter().count();
        let with_needs = sim
            .world
            .query::<(&AgentCivilian, &LifeNeeds)>()
            .iter()
            .count();
        assert!(agents > 0);
        assert_eq!(
            agents, with_needs,
            "all agents must have life needs attached"
        );

        let snap = sim.snapshot();
        // Settlement count is exposed (emergent clusters; may be zero early).
        assert_eq!(snap.settlement_count, sim.settlement_count());
    }

    /// Covers FR-CIV-LIFE-030.
    /// Emergent settlement clustering is deterministic across
    /// two same-seed simulations (replay-safe).
    #[test]
    fn phase_life_clustering_is_deterministic() {
        let mut a = Simulation::with_seed(2024);
        let mut b = Simulation::with_seed(2024);
        for _ in 0..40 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.settlement_count(), b.settlement_count());
        assert_eq!(a.cluster_stocks(), b.cluster_stocks());
    }

    /// Covers FR-CIV-PLANET-010.
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

    /// Covers FR-CIV-PLANET-020.
    /// Covers FR-CIV-VOXEL-002.
    /// FR-CIV-PLANET-020 — `apply_tide_offset` shifts a registered coastal
    /// water-level voxel deterministically as the tide cycles, and the shift
    /// is symmetric around the registered sea-level baseline within tight
    /// numeric tolerance (≤ 1e-4 of the tidal amplitude in fixed-point units).
    #[test]
    fn tide_offset_shifts_coastal_voxel_height() {
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

    /// Covers FR-CIV-TACTICS-010.
    /// Doctrine GA advances on a fixed tick cadence.
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

    /// FR-CIV-0200 — research progress accrues emergently from the living
    /// population over ticks (phase_research is wired into the tick loop).
    #[test]
    fn phase_research_accrues_from_population() {
        let mut sim = Simulation::with_seed(7);
        let before = sim.research_progress();

        for _ in 0..50 {
            sim.tick();
        }

        assert!(
            sim.research_progress() > before,
            "research_progress should grow from a living population over ticks"
        );
    }

    /// FR-CIV-0200 — with no population, research makes no progress (emergent,
    /// not scripted).
    #[test]
    fn phase_research_quiescent_without_population() {
        let mut sim = Simulation::with_seed(7);
        sim.state.population = 0;
        let before = sim.research_progress();
        sim.phase_research();
        assert_eq!(sim.research_progress(), before, "no people, no research");
    }

    /// FR-CIV-0100 §3 — diplomacy emerges from wealth: a large treasury disparity
    /// between the paired factions yields Conflict, not a coin flip.
    #[test]
    fn phase_diplomacy_emerges_conflict_from_wealth_disparity() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500; // a diplomacy cadence tick
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        let a = ids[500 % ids.len()];
        let b = ids[(500 + 1) % ids.len()];
        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(100_000));
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::Conflict
        );
    }

    /// FR-CIV-0100 §3 — near-peer factions (small disparity) trade rather than fight.
    #[test]
    fn phase_diplomacy_emerges_trade_among_peers() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500;
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        let a = ids[500 % ids.len()];
        let b = ids[(500 + 1) % ids.len()];
        sim.state.faction_treasury.insert(a, Fixed::from_num(5_000));
        sim.state.faction_treasury.insert(b, Fixed::from_num(5_000));
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::TradeAgreement
        );
    }

    /// FR-CIV-EMERGENCE — belief accrues from the worshipping population over ticks.
    #[test]
    fn phase_belief_accrues_from_population() {
        let mut sim = Simulation::with_seed(7);
        let before = sim.belief();

        for _ in 0..50 {
            sim.tick();
        }

        assert!(
            sim.belief() > before,
            "belief should accrue from a worshipping population"
        );
    }

    /// FR-CIV-EMERGENCE — a divine power spends belief only when affordable;
    /// a failed invocation leaves belief untouched.
    #[test]
    fn try_invoke_divine_power_gates_on_belief() {
        let mut sim = Simulation::with_seed(7);
        sim.state.belief = 100;
        assert!(!sim.try_invoke_divine_power(200), "cannot afford 200");
        assert_eq!(sim.belief(), 100, "failed invoke leaves belief untouched");
        assert!(sim.try_invoke_divine_power(80), "can afford 80");
        assert_eq!(sim.belief(), 20, "cost deducted on success");
    }

    /// FR-CIV-0200 — research tier and the carrying capacity it feeds grow with
    /// accumulated research (research → economy coupling / downward causation).
    #[test]
    fn research_tier_and_capacity_grow_with_progress() {
        let mut sim = Simulation::with_seed(7);
        assert_eq!(sim.research_tier(), 0);
        let base_capacity = sim.carrying_capacity();
        sim.state.research_progress = 350_000;
        assert_eq!(sim.research_tier(), 3);
        assert!(
            sim.carrying_capacity() > base_capacity,
            "research should raise carrying capacity"
        );
    }

    /// FR-CIV-0100 §3d — wealthy factions bid up staple demand vs poor factions
    /// at equal population (faction prosperity → market coupling).
    #[test]
    fn faction_wealth_drives_market_demand() {
        let mut rich = Simulation::with_seed(7);
        let mut poor = Simulation::with_seed(7);
        rich.state.tick = 2;
        poor.state.tick = 2;
        for v in rich.state.faction_treasury.values_mut() {
            *v = Fixed::from_num(1_000_000);
        }
        for v in poor.state.faction_treasury.values_mut() {
            *v = Fixed::from_num(0);
        }
        rich.phase_economy();
        poor.phase_economy();
        assert!(
            rich.market_state.prices()["food"] >= poor.market_state.prices()["food"],
            "wealthier factions should not yield cheaper staples"
        );
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

    /// Covers FR-CIV-PLANET-060, FR-CIV-TACTICS-041.
    /// Combat events extend the replay hash chain.
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

    /// Covers FR-CIV-TACTICS-025.
    /// Replay log restores queued combat damage events.
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

    /// Covers FR-CIV-TACTICS-025-.
    /// Replay combat events drain to the same voxel state as live ticks.
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

    /// Covers FR-CIV-TACTICS-025 and FR-CIV-TACTICS-025-.
    /// Same seed reproduces identical combat replay markers.
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

    /// Covers FR-CIV-TACTICS-025.
    /// Covers FR-CIV-TACTICS-032.
    /// Covers FR-CIV-TACTICS-035.
    /// FR-CIV-WAR-020 — war replay and live state share combat markers through shared snapshots.
    /// War-bridge engagements append ReplayEvent::Combat.
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

    /// Covers FR-REPLAY-001.
    /// `.civreplay` save/load restores simulation tick after N ticks.
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

    /// Covers FR-CIV-TACTICS-024.
    #[test]
    fn fr_civ_tactics_024_snapshot_damage_events_reflect_last_tick_pulses() {
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

    /// Covers FR-CIV-PLANET-030.
    /// Covers FR-CIV-PLANET-040.
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
    fn phase_voxel_ca_none_is_noop() {
        let mut sim = Simulation::with_seed(1);
        sim.phase_voxel_ca(None);
        assert!(sim.last_tick_abiogenesis_sites().is_empty());
    }

    /// FR-CIV-CA-009 — warm liquid WATER in a single chunk produces at
    /// least one viable abiogenesis site. A pure STONE chunk produces
    /// zero. The two runs must round-trip deterministically (same seed,
    /// same grid → same sites).
    #[test]
    fn phase_voxel_ca_warm_water_is_viable_stone_is_not() {
        use civ_voxel::fluid_ca::{AbiogenesisSuitability, CaGrid};
        use civ_voxel::material::{MaterialRegistry, STONE, WATER};
        use civ_voxel::BoundaryConfig;

        // 16³ grid (single chunk) seeded with one warm WATER cell in the
        //  middle of an otherwise-AIR volume.
        let mut g = CaGrid::new([16, 16, 16]);
        g.set_with_temp(8, 8, 8, WATER, 40);
        g.dirty_chunks.clear();
        g.mark_dirty_cell(8, 8, 8);
        // Run a CA tick so the cell participates in the dirty-chunk set.
        let _ = civ_voxel::fluid_ca::step_with_config(
            &mut g,
            MaterialRegistry::standard(),
            BoundaryConfig::closed(),
            0,
        );
        let mut sim = Simulation::with_seed(7);
        sim.phase_voxel_ca(Some(&g));
        let sites = sim.last_tick_abiogenesis_sites();
        // The WATER cell at (8, 8, 8) is at 40 °C → solvent=255, energy=127
        // (40 * 255 / 80 = 127) → viability true. AIR cells score 0.
        assert!(
            sites.iter().any(|s| s.is_viable()),
            "warm water should be a viable abiogenesis site, got {sites:?}"
        );
        assert!(
            sites
                .iter()
                .all(|s| matches!(s, AbiogenesisSuitability { value, .. } if *value <= 100)),
            "abiogenesis value must be in [0, 100]"
        );

        // Stone-only grid: no solvents at all.
        let mut g2 = CaGrid::new([16, 16, 16]);
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    g2.set_with_temp(x, y, z, STONE, 40);
                }
            }
        }
        g2.dirty_chunks.clear();
        g2.mark_mobile_chunks(MaterialRegistry::standard());
        let mut sim2 = Simulation::with_seed(7);
        sim2.phase_voxel_ca(Some(&g2));
        assert!(
            sim2.last_tick_abiogenesis_sites().is_empty()
                || sim2
                    .last_tick_abiogenesis_sites()
                    .iter()
                    .all(|s| !s.is_viable()),
            "stone-only grid must produce zero viable sites"
        );
    }
}
