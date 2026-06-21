//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use civ_agents::{
    count_civilians, propagate_tools, propagate_wardrobe, spawn_child_near, spawn_civilian_at,
    Civilian as AgentCivilian, CohortStats, LodTier, Needs, Position3d, Tools, Wardrobe,
};
use civ_build::{Allocator, BuildingGraph, DemandSignals};
use civ_diffusion::DiffusionParams;
use civ_economy::{AllocationEngine, CapitalistAllocator, EconomyState, MarketState};
use civ_mod_host::ModHost;
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
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::Fixed;
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
#[allow(dead_code)]
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
];

use std::collections::VecDeque;

/// Active research state (FR-CIV-SERVER-003).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchCache {
    /// Techs queued for research, in order.
    #[serde(default)]
    pub queued: VecDeque<String>,
    /// Fully-researched techs.
    #[serde(default)]
    pub researched: Vec<String>,
    /// Currently-researching tech + ticks remaining.
    #[serde(default)]
    pub in_progress: Option<(String, u32)>,
}

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
    for (faction, (center_x, center_y)) in faction_capitals.into_iter().enumerate() {
        for _ in 0..CIVILIANS_PER_FACTION {
            let grid_x = center_x + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let grid_z = center_y + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let norm_x = (grid_x as f32 / scale).clamp(0.0, 1.0);
            let norm_y = (grid_z as f32 / scale).clamp(0.0, 1.0);
            spawn_civilian_at(world, next_civilian_id, faction as u32, norm_x, norm_y, rng);
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
    /// Accumulated research points; divided by 100_000 gives the tech tier.
    #[serde(default)]
    pub research_progress: u64,
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
    last_tick_engagements: Vec<CombatEngagement>,
    /// `mod.loaded.v1` replay-bus JSON emitted when mods load (cleared each tick).
    last_tick_mod_lifecycle: Vec<String>,
    operational: NoopOperationalLayer,
    replay_log: ReplayLog,
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
        let axial_tilt_fp = i32::from(planet.axial_tilt_deg) * 1_000;
        let weather_grid = compute_weather(0, 16, axial_tilt_fp, planet.year_length_ticks);
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
            research_cache: ResearchCache,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            last_tick_mod_lifecycle: Vec::new(),
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed: 42,
                ..ReplayLog::default()
            },
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
        let axial_tilt_fp = i32::from(planet.axial_tilt_deg) * 1_000;
        let weather_grid = compute_weather(0, 16, axial_tilt_fp, planet.year_length_ticks);
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
            operational: NoopOperationalLayer,
            replay_log: ReplayLog {
                seed,
                ..ReplayLog::default()
            },
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

    /// Current tech tier: research points / 100_000.
    pub fn research_tier(&self) -> u64 {
        self.state.research_progress / 100_000
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
        self.phase_research();
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
        let axial_tilt_fp = i32::from(self.planet.axial_tilt_deg) * 1_000;
        self.weather_grid = compute_weather(
            self.state.tick,
            self.weather_grid.len().max(1) as u32,
            axial_tilt_fp,
            self.planet.year_length_ticks,
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

    /// Research phase: advance in-progress tech; pop next from queue (FR-CIV-SERVER-003).
    fn phase_research(&mut self) {
        if let Some((tech, ref mut ticks_left)) = self.research_cache.in_progress {
            if *ticks_left == 0 {
                self.research_cache.researched.push(tech.clone());
                self.state.research_progress = self.state.research_progress.saturating_add(100_000);
                self.research_cache.in_progress = self.research_cache.queued.pop_front().map(|t| (t, 100));
            } else {
                *ticks_left -= 1;
            }
        } else if let Some(next) = self.research_cache.queued.pop_front() {
            self.research_cache.in_progress = Some((next, 100));
        }
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
            let _ = spawn_child_near(&mut self.world, child_id, 0, x, y, &mut self.rng);
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
        let axial_tilt_fp = i32::from(planet_s.axial_tilt_deg) * 1_000;
        sim_s.weather_grid =
            compute_weather(summer_tick, 16, axial_tilt_fp, planet_s.year_length_ticks);
        let snap_summer = sim_s.snapshot();

        let mut sim_w = Simulation::with_seed(0);
        sim_w.state.tick = winter_tick;
        let planet_w = *sim_w.planet();
        let moon_w = *sim_w.moon();
        sim_w.climate = compute_climate(winter_tick, &planet_w, &moon_w);
        sim_w.weather_grid =
            compute_weather(winter_tick, 16, axial_tilt_fp, planet_w.year_length_ticks);
        let snap_winter = sim_w.snapshot();

        let summer_temp = snap_summer.weather_grid[equatorial_idx].temp_c_fp;
        let winter_temp = snap_winter.weather_grid[equatorial_idx].temp_c_fp;

        assert!(
            summer_temp > winter_temp,
            "summer equatorial temp ({summer_temp} fp) should exceed winter ({winter_temp} fp)"
        );

        // Determinism: re-running the same ticks must produce identical grids.
        let summer_grid_2 =
            compute_weather(summer_tick, 16, axial_tilt_fp, planet_s.year_length_ticks);
        assert_eq!(
            snap_summer.weather_grid, summer_grid_2,
            "weather grid must be deterministic across re-runs"
        );
    }
}
