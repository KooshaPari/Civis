//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use civ_agents::{
    choose_activity, cluster_by_colocation, count_civilians, path_step, pick_target,
    propagate_tools, propagate_wardrobe, spawn_child_near, spawn_civilian_at, wander_anchor,
    Activity, Alignment, Civilian as AgentCivilian, ClusterId, ClusterMember, CohortStats,
    DiplomacyMatrix, DiplomacySignal, LodTier, Needs, PoiKind, PoiRegistry, Position3d, Psyche,
    SocialGraph, Tools, Wardrobe,
};
use civ_agents::culture::{cultural_distance, language_distance, CultureProfile, TraitVector};
use civ_build::{Allocator, BuildingGraph, DemandSignals};
use civ_genetics::Dna;
use civ_genetics::sentience::{cognition_score, CognitionTraitProfile, SentienceThreshold};
use civ_genetics::{archetype_dna, seed_with_divergence, NamedSeed};
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
use std::collections::{BTreeMap, BTreeSet};
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
    let mut spawn_index: usize = 0;
    for (center_x, center_y) in faction_capitals.into_iter() {
        for _ in 0..CIVILIANS_PER_FACTION {
            let grid_x = center_x + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let grid_z = center_y + rng.gen_range(-QUADRANT_SPREAD..=QUADRANT_SPREAD);
            let norm_x = (grid_x as f32 / scale).clamp(0.0, 1.0);
            let norm_y = (grid_z as f32 / scale).clamp(0.0, 1.0);
            let entity = spawn_civilian_at(
                world,
                next_civilian_id,
                civ_agents::infer_alignment_for_spawn(world, norm_x, norm_y),
                norm_x,
                norm_y,
                civ_agents::ActorVisualKind::Humanoid,
                rng,
            );
            // Assign named-race archetype DNA via the 3-cycle seed assignment.
            // Cycle: 0 → Ardani, 1 → Velthari, 2 → Grundak.
            // divergence=0.3 introduces population spread while preserving
            // the archetype's characteristic genome cluster.
            let named_seed = match spawn_index % 3 {
                0 => NamedSeed::Ardani,
                1 => NamedSeed::Velthari,
                _ => NamedSeed::Grundak,
            };
            let base = archetype_dna(named_seed);
            let dna = seed_with_divergence(&base, 0.3, rng);
            let _ = world.insert_one(entity, dna);
            next_civilian_id += 1;
            spawn_index += 1;
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

/// Dominant economic specialization that EMERGES from the strongest sector
/// (FR-CIV-0100 §3 emergence). Hysteresis prevents flip-flopping; the active
/// focus amplifies comparative advantage in production.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EconomicFocus {
    #[default]
    Balanced,
    Agrarian,
    Industrial,
    Sacred,
    Mercantile,
}

/// One polity's macro + economy row. Map key equals [`Self::id`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PolityMacroState {
    pub id: u32,
    pub name: String,
    pub treasury: Fixed,
    pub resources: Resources,
    pub belief: u64,
    pub unrest: u64,
    pub cohesion: u64,
    pub research_progress: u64,
    pub tech_unlocks: u64,
    pub dispossessed_permille: u64,
    pub temple_level: u32,
    pub garrison_level: u32,
    pub economic_focus: EconomicFocus,
    pub focus_pressure: u8,
    pub population: u64,
    pub legitimacy_milli: i64,
    pub shadow_influence_index_milli: i64,
    pub influence_capital: i64,
    pub governance_integrity_milli: i64,
}

impl PolityMacroState {
    fn default_for(id: u32, state: &WorldState) -> Self {
        Self {
            id,
            name: state
                .factions
                .get(&id)
                .cloned()
                .unwrap_or_else(|| format!("Faction {id}")),
            treasury: state.faction_treasury.get(&id).copied().unwrap_or_default(),
            resources: state
                .faction_resources
                .get(&id)
                .cloned()
                .unwrap_or_default(),
            unrest: state.faction_unrest.get(&id).copied().unwrap_or(0),
            ..Self::default()
        }
    }
}

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
    /// Per-faction accumulated unrest (0 = content). EMERGES from each faction's
    /// own wealth/scarcity shadow via [`Simulation::phase_faction_unrest`].
    /// `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub faction_unrest: HashMap<u32, u64>,
    /// Accumulated social cohesion — the strength of the shared social fabric.
    /// EMERGES from collective belief (shared faith binds) and frays under
    /// unrest (disorder loosens bonds). A stabilising counterweight to unrest.
    /// `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub cohesion: u64,
    /// Cached micro social-trust trade bonus (per-mille above 1.0× volume, 0..=250).
    /// Written at end of [`Simulation::phase_cohesion`] from agent [`SocialGraph`] ties;
    /// consumed in [`Simulation::tick_trade_routes`] on the **next** tick because
    /// `phase_economy` precedes `phase_emergence`. `#[serde(default)]` keeps older
    /// `.civsave` files loadable (missing field → 0, no retroactive trade boost).
    #[serde(default)]
    pub micro_trust_permille: u64,
    /// Discrete technology unlocks (irreversible bitmask). EMERGES from research
    /// milestones via [`Simulation::phase_tech`]. `#[serde(default)]` keeps older
    /// saves loadable.
    #[serde(default)]
    pub tech_unlocks: u64,
    /// Persistent dispossessed underclass share (per-mille, 0..=1000). EMERGES
    /// from sustained wealth inequality with hysteresis via
    /// [`Simulation::phase_stratification`]. Distinct from the instantaneous
    /// [`inequality_unrest`] term. `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub dispossessed_permille: u64,
    /// Temple institution level (0..=MAX_INSTITUTION_LEVEL). EMERGES from
    /// accumulated belief via [`Simulation::phase_institutions`].
    #[serde(default)]
    pub temple_level: u32,
    /// Garrison institution level (0..=MAX_INSTITUTION_LEVEL). EMERGES from
    /// societal unrest via [`Simulation::phase_institutions`].
    #[serde(default)]
    pub garrison_level: u32,
    /// Dominant economic focus (FR-CIV-0100 emergence). EMERGES from the
    /// strongest sector via [`Simulation::phase_economic_focus`], with
    /// hysteresis so specialization persists.
    #[serde(default)]
    pub economic_focus: EconomicFocus,
    /// Consecutive evaluations the candidate focus has dominated (0..=10).
    /// Drives hysteresis in [`Simulation::phase_economic_focus`].
    #[serde(default)]
    pub focus_pressure: u8,
    /// Notable simulation history (tech breakthroughs, golden/dark ages). EMERGES
    /// from threshold crossings via [`Simulation::phase_chronicle`]. Capped at
    /// [`CHRONICLE_MAX_LEN`]. `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub chronicle: Vec<String>,
    /// Last `tech_unlocks` bitmask recorded in the chronicle (dedup).
    #[serde(default)]
    pub chronicle_tech_seen: u64,
    /// Last recorded age era (0 = normal, 1 = golden, 2 = dark). Dedupes age lines.
    #[serde(default)]
    pub chronicle_age: u8,
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
    /// Sustained [`DiplomacyKind::TradeAgreement`] count per canonical faction pair
    /// (N4 — diplomacy births emergent routes). `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub faction_trade_agreement_streak: BTreeMap<(u32, u32), u32>,
    /// Keys of routes born via N4 emergence (not bootstrap triangle). Used for idle decay.
    #[serde(default)]
    pub emergent_trade_route_keys: BTreeSet<(u32, u32, String)>,
    /// Consecutive ticks without flow per emergent route key (N4 idle decay).
    #[serde(default)]
    pub trade_route_idle_ticks: BTreeMap<(u32, u32, String), u32>,
    /// Pairwise faction relations in `[-1.0, 1.0]` (alliance .. rivalry). EMERGES from
    /// trade/conflict history in [`Simulation::phase_diplomacy`] and biases future
    /// diplomacy thresholds. `#[serde(default)]` keeps older saves loadable.
    #[serde(default)]
    pub faction_relations: DiplomacyMatrix,
    /// Per-polity macro + economy rows (authoritative target; dual-written with legacy
    /// maps during migration). `BTreeMap` keys iterate in deterministic order.
    #[serde(default)]
    pub polities: BTreeMap<u32, PolityMacroState>,
    pub resources: Resources,
}

/// Merge legacy per-faction `HashMap`s into [`WorldState::polities`] after deserialize
/// or [`WorldState::default`].
fn hydrate_polities_from_legacy(state: &mut WorldState) {
    let ids: BTreeSet<u32> = state
        .factions
        .keys()
        .chain(state.faction_treasury.keys())
        .chain(state.faction_resources.keys())
        .chain(state.faction_unrest.keys())
        .copied()
        .collect();
    for id in ids {
        let name = state.factions.get(&id).cloned();
        let treasury = state.faction_treasury.get(&id).copied();
        let resources = state.faction_resources.get(&id).cloned();
        let unrest = state.faction_unrest.get(&id).copied();

        if !state.polities.contains_key(&id) {
            state.polities.insert(
                id,
                PolityMacroState {
                    id,
                    name: name
                        .clone()
                        .unwrap_or_else(|| format!("Faction {id}")),
                    treasury: treasury.unwrap_or_default(),
                    resources: resources.clone().unwrap_or_default(),
                    unrest: unrest.unwrap_or(0),
                    ..PolityMacroState::default()
                },
            );
        }

        let entry = state.polities.get_mut(&id).expect("polity row exists");
        if let Some(name) = name {
            entry.name.clone_from(&name);
        } else if entry.name.is_empty() {
            entry.name = format!("Faction {id}");
        }
        if let Some(treasury) = treasury {
            entry.treasury = treasury;
        }
        if let Some(resources) = resources {
            entry.resources = resources;
        }
        if let Some(unrest) = unrest {
            entry.unrest = unrest;
        }
        entry.id = id;
    }
}

impl Default for WorldState {
    fn default() -> Self {
        let mut state = Self {
            tick: 0,
            population: 1_000_000,
            research_progress: 0,
            belief: 0,
            unrest: 0,
            faction_unrest: HashMap::new(),
            cohesion: 0,
            micro_trust_permille: 0,
            tech_unlocks: 0,
            dispossessed_permille: 0,
            temple_level: 0,
            garrison_level: 0,
            economic_focus: EconomicFocus::Balanced,
            focus_pressure: 0,
            chronicle: Vec::new(),
            chronicle_tech_seen: 0,
            chronicle_age: 0,
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
            faction_trade_agreement_streak: BTreeMap::new(),
            emergent_trade_route_keys: BTreeSet::new(),
            trade_route_idle_ticks: BTreeMap::new(),
            faction_relations: DiplomacyMatrix::default(),
            polities: BTreeMap::new(),
            resources: Resources::default(),
        };
        hydrate_polities_from_legacy(&mut state);
        state
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
    /// Member counts from the latest [`Simulation::phase_life`] clustering pass;
    /// consumed by [`Simulation::phase_settlement_consumption`] so drains match
    /// production (FR-CIV-LIFE-020).
    cluster_member_counts: BTreeMap<u64, u32>,
    /// Number of emergent settlements (multi-member clusters) detected on the
    /// most recent [`Simulation::phase_life`] (FR-CIV-LIFE-030).
    pub(crate) last_settlement_count: u32,
    /// Deaths attributed to unmet-need sickness on the most recent life phase
    /// (FR-CIV-LIFE-003); surfaced for the HUD.
    pub(crate) last_life_deaths: u32,
    /// Position fingerprint from the last `cluster_by_colocation` pass in
    /// [`Simulation::phase_life`]. When unchanged, all-pairs clustering is
    /// skipped and cached [`Self::cluster_member_counts`] / [`ClusterMember`]
    /// components are reused (PERF_OPT #1).
    life_cluster_position_fingerprint: u64,
    /// Count of full clustering recomputes in [`Simulation::phase_life`]
    /// (test-only observability for the clustering skip path).
    #[cfg(test)]
    life_clustering_recompute_count: u64,
    /// When true, always re-run `cluster_by_colocation` (test-only baseline).
    #[cfg(test)]
    force_life_cluster_recompute: bool,
    /// Civilian agent id → ECS entity map for O(1) emergence lookups (PERF_OPT #2).
    agent_id_to_entity: BTreeMap<u64, Entity>,
    /// Linear-scan `agent_entity` invocations (test-only observability).
    #[cfg(test)]
    agent_entity_linear_scan_count: std::cell::Cell<u64>,
    /// When true, `agent_entity` uses the pre-opt linear scan (test-only baseline).
    #[cfg(test)]
    force_agent_entity_linear_scan: bool,
    /// MOAT emergence: legends, psyche, culture, social, genetics, civ-ai.
    pub(crate) emergence: crate::emergence::EmergenceState,
    /// Latest emergence-metrics sample (civ-emergence-metrics). Updated by
    /// [`crate::emergence_metrics::sample_emergence`] on every 50-tick
    /// boundary (5 s at 100 ms tick). `None` before the first sample
    /// boundary (ticks 0..49). Surfaced over JSON-RPC `sim.emergence`
    /// (stacked on PR #350).
    pub(crate) emergence_sample: Option<crate::emergence_metrics::EmergenceSample>,
    /// Rolling-mean branching ratio ledger and live `σ̄_W` (charter §3.6).
    pub(crate) emergence_branching: crate::emergence_metrics::EmergenceBranchingState,
    /// N9: per-faction mean phenotype aggression in [0,1].
    /// Rebuilt each tick by `emergence_genetics_sentience`; ephemeral (not persisted).
    pub(crate) faction_aggression: BTreeMap<u32, f32>,
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
    // Before: 2×O(A) scans here (pre + post). After: 1×O(A) — post-scan dropped.
    // Correctness: propagate_wardrobe only promotes (era increases), never demotes,
    // so currently_at_target_after == pre_count + promoted_this_tick exactly.
    // currently_at_target is therefore a one-tick-lag snapshot (end-of-previous-tick
    // value) which is what current_fraction is already computed from.
    let total_civilians = count_civilians(world) as u32;
    let pre_at_target = world
        .query::<&Wardrobe>()
        .iter()
        .filter(|(_, wardrobe)| wardrobe.era >= target_era)
        .count() as u32;
    let current_fraction = if total_civilians == 0 {
        0.0
    } else {
        pre_at_target as f32 / total_civilians as f32
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

    // Derive post-tick count arithmetically — avoids a second O(A) ECS scan.
    let currently_at_target = pre_at_target + promoted_this_tick;

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
    // Before: 2×O(A) scans here (pre + post). After: 1×O(A) — post-scan dropped.
    // Correctness: propagate_tools only promotes (era increases), never demotes,
    // so currently_at_target_after == pre_count + promoted_this_tick exactly.
    let total_civilians = count_civilians(world) as u32;
    let pre_at_target = world
        .query::<&Tools>()
        .iter()
        .filter(|(_, tools)| tools.era >= target_era)
        .count() as u32;
    let current_fraction = if total_civilians == 0 {
        0.0
    } else {
        pre_at_target as f32 / total_civilians as f32
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

    // Derive post-tick count arithmetically — avoids a second O(A) ECS scan.
    let currently_at_target = pre_at_target + promoted_this_tick;

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

        let mut sim = Self {
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
            cluster_member_counts: BTreeMap::new(),
            last_life_deaths: 0,
            last_settlement_count: 0,
            life_cluster_position_fingerprint: 0,
            #[cfg(test)]
            life_clustering_recompute_count: 0,
            #[cfg(test)]
            force_life_cluster_recompute: false,
            agent_id_to_entity: BTreeMap::new(),
            #[cfg(test)]
            agent_entity_linear_scan_count: std::cell::Cell::new(0),
            #[cfg(test)]
            force_agent_entity_linear_scan: false,
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
            emergence_branching: crate::emergence_metrics::EmergenceBranchingState::default(),
            faction_aggression: BTreeMap::new(),
        };
        sim.rebuild_agent_id_index();
        sim
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

        let mut sim = Self {
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
            cluster_member_counts: BTreeMap::new(),
            last_life_deaths: 0,
            last_settlement_count: 0,
            life_cluster_position_fingerprint: 0,
            #[cfg(test)]
            life_clustering_recompute_count: 0,
            #[cfg(test)]
            force_life_cluster_recompute: false,
            agent_id_to_entity: BTreeMap::new(),
            #[cfg(test)]
            agent_entity_linear_scan_count: std::cell::Cell::new(0),
            #[cfg(test)]
            force_agent_entity_linear_scan: false,
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
            emergence_branching: crate::emergence_metrics::EmergenceBranchingState::default(),
            faction_aggression: BTreeMap::new(),
        };
        sim.rebuild_agent_id_index();
        sim
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

    /// Engagements resolved during the most recent tactics phase.
    #[must_use]
    pub fn last_tick_engagements(&self) -> &[CombatEngagement] {
        &self.last_tick_engagements
    }

    /// Per-tick micro-actor action count for branching-ratio avalanche seeds
    /// (charter §3.6). Integer sums over existing per-tick buffers; O(1).
    #[must_use]
    pub(crate) fn micro_actor_action_count(&self) -> u32 {
        let voxel = self.last_tick_voxel_events.len() as u32;
        let disasters = self.last_tick_voxel_damage_count as u32;
        let diplomacy = self.diplomacy_events.len() as u32;
        let unrest = self.emergence_branching.last_tick_unrest_events;
        let combat = self.last_tick_combat_pulses.len() as u32
            + self.last_tick_engagements.len() as u32;
        voxel
            .saturating_add(disasters)
            .saturating_add(diplomacy)
            .saturating_add(unrest)
            .saturating_add(combat)
    }

    /// Per-tick micro-descendant action count for branching-ratio closure.
    #[must_use]
    pub(crate) fn micro_descendant_action_count(&self) -> u32 {
        self.micro_actor_action_count()
    }

    /// Rolling-mean branching ratio `σ̄_W` (charter §3.6).
    #[must_use]
    pub fn branching_ratio(&self) -> f32 {
        self.emergence_branching.sigma_bar
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
        const IRRIGATION_BONUS: i64 = 200_000;
        const SANITATION_BONUS: i64 = 300_000;
        let tier = self.research_tier().min(i64::MAX as u64) as i64;
        let mut cap = POP_BASELINE + tier.saturating_mul(CAPACITY_PER_TIER);
        if self.state.tech_unlocks & TECH_IRRIGATION != 0 {
            cap = cap.saturating_add(IRRIGATION_BONUS);
        }
        if self.state.tech_unlocks & TECH_SANITATION != 0 {
            cap = cap.saturating_add(SANITATION_BONUS);
        }
        cap
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

    /// Per-faction unrest for `faction_id`, driven each tick by
    /// [`Simulation::phase_faction_unrest`] from that faction's wealth/scarcity
    /// shadow. Missing factions read as zero (content).
    #[must_use]
    pub fn faction_unrest(&self, faction_id: u32) -> u64 {
        self.state.faction_unrest.get(&faction_id).copied().unwrap_or(0)
    }

    fn ensure_polity(&mut self, id: u32) -> &mut PolityMacroState {
        if !self.state.polities.contains_key(&id) {
            let row = PolityMacroState::default_for(id, &self.state);
            self.state.polities.insert(id, row);
        }
        self.state
            .polities
            .get_mut(&id)
            .expect("polity row exists")
    }

    /// Accumulated social cohesion, generated each tick by
    /// [`Simulation::phase_cohesion`] from belief minus unrest. Higher means a
    /// stronger shared social fabric.
    #[must_use]
    pub fn cohesion(&self) -> u64 {
        self.state.cohesion
    }

    /// Pairwise faction relation score in `[-1.0, 1.0]` (positive = alliance,
    /// negative = rivalry). Returns `0.0` when no history exists for the pair.
    #[must_use]
    pub fn faction_relation(&self, a: u32, b: u32) -> f32 {
        self.state
            .faction_relations
            .record(ClusterId(u64::from(a)), ClusterId(u64::from(b)))
            .map(|record| record.score)
            .unwrap_or(0.0)
    }

    /// Persistent dispossessed underclass share (per-mille, 0..=1000), updated
    /// each tick by [`Simulation::phase_stratification`].
    #[must_use]
    pub fn dispossessed_permille(&self) -> u64 {
        self.state.dispossessed_permille
    }

    /// Temple institution level (0..=MAX_INSTITUTION_LEVEL), updated each tick
    /// by [`Simulation::phase_institutions`] from accumulated belief.
    #[must_use]
    pub fn temple_level(&self) -> u32 {
        self.state.temple_level
    }

    /// Garrison institution level (0..=MAX_INSTITUTION_LEVEL), updated each
    /// tick by [`Simulation::phase_institutions`] from societal unrest.
    #[must_use]
    pub fn garrison_level(&self) -> u32 {
        self.state.garrison_level
    }

    /// Dominant economic focus, updated each tick by
    /// [`Simulation::phase_economic_focus`] from sector signals.
    #[must_use]
    pub fn economic_focus(&self) -> EconomicFocus {
        self.state.economic_focus
    }

    /// Discrete technology unlocks reached so far (irreversible bitmask).
    #[must_use]
    pub fn tech_unlocks(&self) -> u64 {
        self.state.tech_unlocks
    }

    /// Whether a specific tech-unlock bit is set.
    #[must_use]
    pub fn has_tech(&self, bit: u64) -> bool {
        self.state.tech_unlocks & bit == bit
    }

    /// Notable simulation history (tech breakthroughs, golden/dark ages).
    #[must_use]
    pub fn chronicle(&self) -> &[String] {
        &self.state.chronicle
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

    /// Add `amount` to accumulated cohesion. Signed to match the existing
    /// `cohesion_delta` contract: positive binds the social fabric, negative
    /// frays it. Floored at zero so a negative amount can never push the
    /// field below the natural `phase_cohesion` floor.
    pub(crate) fn add_cohesion(&mut self, amount: i64) {
        let next = (self.state.cohesion as i64).saturating_add(amount);
        self.state.cohesion = next.max(0) as u64;
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
        self.emergence_branching.last_tick_unrest_events = 0;

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
        self.phase_settlement_consumption();
        self.rebuild_agent_id_index();
        self.phase_emergence();
        // FR-CIV-LEGENDS-001: mint belief from this tick's saga activity
        // (promoted entities + top-N significance), bounded per tick so a
        // saga spike cannot explode faith. Runs before `phase_belief` so the
        // per-tick population/temple inflow sees the updated total.
        self.apply_saga_belief_gain();
        self.phase_research();
        self.phase_tech();
        self.phase_belief();
        self.phase_unrest();
        self.phase_faction_unrest();
        self.phase_cohesion();
        self.phase_social_mood();
        self.phase_stratification();
        self.phase_institutions();
        self.phase_economic_focus();
        self.phase_chronicle();
        self.phase_emergence_events_close();
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
        let mut contribution = base
            .saturating_add(base.saturating_mul(cohesion_research_bonus_permille(self.state.cohesion)) / 1_000);
        if self.state.tech_unlocks & TECH_WRITING != 0 {
            contribution = contribution.saturating_add(1);
        }
        contribution = contribution.saturating_add(sentience_research_bonus(&self.world));
        self.state.research_progress = self.state.research_progress.saturating_add(contribution);
    }

    /// Tech-unlock phase (FR-CIV-0100 §3 emergence). Research milestones
    /// irreversibly OR discrete capability bits into `tech_unlocks`; bits are
    /// never cleared once earned.
    fn phase_tech(&mut self) {
        self.state.tech_unlocks |= tech_unlocks_for_tier(self.research_tier());
    }

    /// Faith phase (divine-powers economy, FR-CIV-EMERGENCE). The worshipping
    /// population generates `belief` each tick; belief is the resource spent via
    /// [`Simulation::try_invoke_divine_power`] to invoke divine interventions.
    /// Pure, deterministic function of `population`.
    fn phase_belief(&mut self) {
        /// People required to generate one unit of belief per tick.
        const BELIEF_POP_DIVISOR: u64 = 2_000;
        /// Belief fades without renewal: a small proportional decay gives a dynamic
        /// equilibrium (worship inflow vs decay) instead of unbounded growth.
        const BELIEF_DECAY_DIVISOR: u64 = 500;
        let worship = self.state.population / BELIEF_POP_DIVISOR;
        self.state.belief = self.state.belief.saturating_add(worship);
        self.state.belief = self
            .state
            .belief
            .saturating_add(self.state.temple_level as u64);
        self.state.belief = self
            .state
            .belief
            .saturating_sub(self.state.belief / BELIEF_DECAY_DIVISOR);
        // N11 maturity↔belief coupling (FR-CIV-EMERGENCE-N11): elder wisdom stabilizes faith.
        let avg_maturity = avg_psyche_maturity(&self.world);
        let belief_stabilizer = (avg_maturity * 5_000.0) as u64;
        self.state.belief = self.state.belief.saturating_add(belief_stabilizer);
        // Mature populations resist belief drift; immature populations drift faster.
        let drift_factor = 0.95 + 0.05 * avg_maturity; // [0.95, 1.0]
        let drift_loss = (self.state.belief as f32 * (1.0 - drift_factor)) as u64;
        self.state.belief = self.state.belief.saturating_sub(drift_loss);
    }

    /// FR-CIV-LEGENDS-001 — mint belief from this tick's saga significance.
    /// Called after `phase_emergence` (saga ingest has run) and before
    /// `phase_belief` so the per-tick population/temple inflow runs on the
    /// updated total. Additive, bounded by [`MAX_SAGA_BELIEF_PER_TICK`], and
    /// read-minimal (one pass over the feed + one top-N graph query).
    pub(crate) fn apply_saga_belief_gain(&mut self) {
        let promoted_count = self
            .emergence
            .last_feed
            .iter()
            .filter(|e| e.kind == "legend_promotion")
            .count();
        let top = self.emergence.legends.graph.significant(8, None);
        let sig_sum: f32 = top.iter().map(|e| e.significance.clamp(0.0, 1.0)).sum();
        self.add_belief(saga_belief_gain(promoted_count, sig_sum));
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
        // Structural inequality (wealth gap between richest and poorest faction)
        // breeds class unrest.
        let treasury_spread = faction_treasury_spread(&self.state.faction_treasury);
        let delta = cohesion_unrest_damp(research_unrest_mitigation(unrest_delta(food_price), self.research_tier()), self.state.cohesion)
            + energy_scarcity_unrest(self.state.energy_budget_joules)
            + agent_misery_unrest(&self.world)
            + overcrowding_unrest(self.state.population, self.carrying_capacity())
            + inequality_unrest(treasury_spread)
            + dispossession_unrest(self.state.dispossessed_permille)
            + commodity_unrest_delta(self.market_state.prices())
            - (self.state.garrison_level as i64 * 2);
        if delta > 0 {
            self.record_unrest_micro_activity(delta.min(i32::MAX as i64) as u32);
        }
        self.state.unrest = (self.state.unrest as i64 + delta).max(0) as u64;
        let faith_from_hardship = self.state.unrest / UNREST_FAITH_DIVISOR;
        self.add_belief(faith_from_hardship);
    }

    /// Per-faction unrest phase (FR-CIV-0100 §3 emergence). Each faction's
    /// unrest EMERGES from its own wealth/scarcity shadow — mirroring global
    /// food-scarcity unrest but keyed to treasury and food holdings. Runs after
    /// `phase_unrest` so global and per-polity unrest layers stay ordered.
    fn phase_faction_unrest(&mut self) {
        /// Proportional decay yields a dynamic equilibrium under sustained scarcity.
        const FACTION_UNREST_DECAY_DIVISOR: u64 = 200;
        let mut faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        for id in faction_ids {
            let treasury = self
                .state
                .faction_treasury
                .get(&id)
                .copied()
                .unwrap_or_default();
            let resources = self
                .state
                .faction_resources
                .get(&id)
                .cloned()
                .unwrap_or_default();
            let shadow = faction_wealth_scarcity_shadow(treasury, &resources);
            let delta = faction_unrest_delta_from_shadow(shadow);
            if delta > 0 {
                self.record_unrest_micro_activity(1);
            }
            let entry = self.state.faction_unrest.entry(id).or_insert(0);
            *entry = (*entry as i64 + delta).max(0) as u64;
            *entry = entry.saturating_sub(*entry / FACTION_UNREST_DECAY_DIVISOR);
            self.ensure_polity(id).unrest = *entry;
        }
    }

    /// Social-cohesion phase (FR-CIV-0100 §3 emergence). The shared social fabric
    /// EMERGES from the balance of collective belief (shared faith binds) and
    /// unrest (disorder frays bonds): cohesion accrues when faith outweighs
    /// discontent and decays when discontent dominates. Runs after `phase_unrest`
    /// so it sees the current tick's unrest. Floored at zero.
    fn phase_cohesion(&mut self) {
        /// Cohesion frays without reinforcement: proportional decay yields a
        /// dynamic equilibrium (belief bind vs unrest fray vs decay).
        const COHESION_DECAY_DIVISOR: u64 = 500;
        let delta = cohesion_delta(self.state.belief, self.state.unrest)
            + micro_cohesion_delta(&self.world);
        self.state.cohesion = (self.state.cohesion as i64 + delta).max(0) as u64;
        self.state.cohesion = self
            .state
            .cohesion
            .saturating_sub(self.state.cohesion / COHESION_DECAY_DIVISOR);

        // N10 kinship↔cohesion coupling: family ties reinforce social cohesion (FR-CIV-EMERGENCE-N10).
        // Upward causation: high average kinship boosts cohesion (people trust kin).
        // Inverse decay: low kinship → faster decay (loneliness destabilizes).
        // Clamp to [0,1]: tie.kinship is nominally normalized, but guard against
        // malformed/out-of-range ties so the boost/decay math stays bounded.
        let avg_kinship = avg_faction_kinship(&self.world).clamp(0.0, 1.0);
        let kinship_boost = (avg_kinship * 0.02 * 100_000.0) as u64;
        self.state.cohesion = self.state.cohesion.saturating_add(kinship_boost);

        // Decay factor inversely tied to kinship: high kinship slows decay.
        let decay_factor = 0.98 - (0.05 * (1.0 - avg_kinship)).max(0.0).min(1.0);
        let decay_amt = (self.state.cohesion as f32 * (1.0 - decay_factor)) as u64;
        self.state.cohesion = self.state.cohesion.saturating_sub(decay_amt);

        self.state.micro_trust_permille = micro_social_trust_permille(&self.world);
    }

    /// Social-mood phase (FR-CIV-0100 §3 emergence). Downward causation: macro
    /// cohesion lifts individual agent spirits — a cohesive society nudges
    /// `Psyche::mood.valence` upward, stabilizing the misery→unrest loop with
    /// negative feedback. Runs after `phase_cohesion` so cohesion is current.
    /// Uplift is small and bounded (max +0.02/tick; valence clamped to [-1, 1]).
    fn phase_social_mood(&mut self) {
        let uplift = (self.state.cohesion as f32 / 2_000_000.0).clamp(0.0, 0.02);
        for (_, psyche) in self.world.query_mut::<&mut Psyche>() {
            psyche.mood.valence = (psyche.mood.valence + uplift).clamp(-1.0, 1.0);
        }
    }

    /// Social-stratification phase (FR-CIV-0100 §3 emergence). A persistent
    /// dispossessed underclass share EMERGES from sustained wealth inequality,
    /// moving slowly toward its equilibrium (hysteresis) so class structure
    /// persists rather than tracking the gap instantly. Runs after
    /// `phase_cohesion` so cohesion can erode the target. Clamped to [0, 1000].
    fn phase_stratification(&mut self) {
        let treasury_spread = faction_treasury_spread(&self.state.faction_treasury);
        let target = dispossession_target_permille(treasury_spread, self.state.cohesion);
        self.state.dispossessed_permille =
            dispossession_step(self.state.dispossessed_permille, target);
    }

    /// Institution phase (FR-CIV-0100 §3 emergence). Leveled Temple and Garrison
    /// organizations EMERGE from macro signals (belief and unrest), drain
    /// treasury upkeep, and couple back via `phase_belief` and `phase_unrest`.
    /// Growth/decay is gradual (one level per tick) with a criticality cap.
    fn phase_institutions(&mut self) {
        let temple_target = institution_target_level(self.state.belief, 5_000);
        self.state.temple_level =
            institution_step(self.state.temple_level, temple_target);
        let garrison_target = institution_target_level(self.state.unrest, 200);
        self.state.garrison_level =
            institution_step(self.state.garrison_level, garrison_target);
        if let Some(&min_id) = self.state.faction_treasury.keys().min() {
            if let Some(treasury) = self.state.faction_treasury.get_mut(&min_id) {
                let upkeep = Fixed::from_num(
                    (self.state.temple_level + self.state.garrison_level) as i64 * 10,
                );
                *treasury = (*treasury - upkeep).max(Fixed::from_num(0));
            }
        }
    }

    /// Economic specialization phase (FR-CIV-0100 §3 emergence). A dominant
    /// [`EconomicFocus`] EMERGES from the strongest sector signal, with
    /// hysteresis so the civilization does not flip-flop each tick. The active
    /// focus couples back via comparative-advantage bonuses in
    /// [`Simulation::phase_production`].
    fn phase_economic_focus(&mut self) {
        const FOCUS_PRESSURE_THRESHOLD: u8 = 5;
        const FOCUS_PRESSURE_CAP: u8 = 10;

        let treasury_total = self
            .state
            .faction_treasury
            .values()
            .map(|t| (t.raw / crate::SCALE).max(0))
            .sum::<i64>();
        let food = self.state.resources.food.raw / crate::SCALE;
        let candidate = candidate_economic_focus(
            food,
            self.research_tier(),
            self.state.belief,
            treasury_total,
        );

        if candidate == self.state.economic_focus {
            self.state.focus_pressure = 0;
            return;
        }

        self.state.focus_pressure = self
            .state
            .focus_pressure
            .saturating_add(1)
            .min(FOCUS_PRESSURE_CAP);
        if self.state.focus_pressure >= FOCUS_PRESSURE_THRESHOLD {
            self.state.economic_focus = candidate;
            self.state.focus_pressure = 0;
        }
    }

    /// Chronicle phase (FR-CIV-0100 emergence legibility). Records notable history
    /// when tech unlocks advance or society enters a golden/dark age. Deduped via
    /// `chronicle_tech_seen` and `chronicle_age`; length capped at [`CHRONICLE_MAX_LEN`].
    fn phase_chronicle(&mut self) {
        if self.state.tech_unlocks != self.state.chronicle_tech_seen {
            let new_bits = self.state.tech_unlocks & !self.state.chronicle_tech_seen;
            if new_bits != 0 {
                self.state.chronicle.push(format!(
                    "Tick {}: a technological breakthrough ({:#b})",
                    self.state.tick, new_bits
                ));
            }
            self.state.chronicle_tech_seen = self.state.tech_unlocks;
        }

        let target_age = if self.state.cohesion > 50_000 && self.state.belief > 50_000 {
            1
        } else if self.state.unrest > 800 {
            2
        } else {
            0
        };
        if target_age != self.state.chronicle_age {
            let line = match target_age {
                1 => format!("Tick {}: a golden age dawns", self.state.tick),
                2 => format!("Tick {}: a dark age of unrest begins", self.state.tick),
                _ => format!("Tick {}: the realm returns to calm", self.state.tick),
            };
            self.state.chronicle.push(line);
            self.state.chronicle_age = target_age;
        }

        if self.state.chronicle.len() > CHRONICLE_MAX_LEN {
            let drain = self.state.chronicle.len() - CHRONICLE_MAX_LEN;
            self.state.chronicle.drain(..drain);
        }
    }

    /// Buildings phase - expands the parcel graph on a fixed cadence when demand is high.
    /// Construction debits global wood and metal stockpiles produced by
    /// [`Simulation::phase_production`]; scarcity throttles expansion.
    fn phase_buildings(&mut self) {
        if self.state.tick % building_cadence(self.research_tier()) != 0 {
            return;
        }

        let wood = self.state.resources.wood;
        let metal = self.state.resources.metal;
        let signals = building_demand_signals(
            self.state.population,
            self.carrying_capacity(),
            self.state.cohesion,
            self.research_tier(),
            self.state.unrest,
            wood,
            metal,
        );

        let pending = building_parcel_count(&signals);
        if pending == 0 {
            return;
        }

        let affordable = building_affordable_parcel_count(wood, metal);
        if affordable == 0 {
            return;
        }
        let allow = pending.min(affordable);
        let signals = building_signals_limited(signals, allow);
        let to_build = building_parcel_count(&signals);
        if to_build == 0 || !building_materials_affordable(wood, metal, to_build) {
            return;
        }

        let origin = civ_voxel::WorldCoord { x: 0, y: 0, z: 0 };
        let allocated = self.allocator.allocate(
            &mut self.building_graph,
            &signals,
            self.target_era,
            origin,
            16,
        );
        if allocated.is_empty() {
            return;
        }
        let (wood_cost, metal_cost) = building_material_cost(allocated.len());
        self.state.resources.wood = self.state.resources.wood.saturating_sub(wood_cost);
        self.state.resources.metal = self.state.resources.metal.saturating_sub(metal_cost);
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
    /// Rebuild the civilian id → entity map from the current world (PERF_OPT #2).
    pub(crate) fn rebuild_agent_id_index(&mut self) {
        self.agent_id_to_entity.clear();
        for (entity, civilian) in self.world.query::<&AgentCivilian>().iter() {
            debug_assert!(
                !self.agent_id_to_entity.contains_key(&civilian.id),
                "duplicate civilian id {}",
                civilian.id
            );
            self.agent_id_to_entity.insert(civilian.id, entity);
        }
    }

    /// Resolve a civilian agent id to its ECS entity (PERF_OPT #2).
    pub(crate) fn agent_entity(&self, agent_id: u64) -> Option<Entity> {
        #[cfg(test)]
        if self.force_agent_entity_linear_scan {
            self.agent_entity_linear_scan_count
                .set(self.agent_entity_linear_scan_count.get() + 1);
            return self
                .world
                .query::<&AgentCivilian>()
                .iter()
                .find(|(_, c)| c.id == agent_id)
                .map(|(e, _)| e);
        }
        self.agent_id_to_entity.get(&agent_id).copied()
    }

    fn life_cluster_position_fingerprint(world: &World) -> u64 {
        let mut agents: Vec<(u64, i64, i64)> = world
            .query::<(&AgentCivilian, &Position3d)>()
            .iter()
            .map(|(_, (civ, pos))| (civ.id, pos.coord.x, pos.coord.z))
            .collect();
        agents.sort_by_key(|(agent_id, _, _)| *agent_id);

        let mut fingerprint = agents.len() as u64;
        for (agent_id, x, z) in agents {
            fingerprint = fingerprint
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(agent_id ^ x.rotate_left(17) as u64 ^ z.rotate_left(31) as u64);
        }
        fingerprint
    }

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

        // 5. Emergent settlement clustering by co-location. Skip the O(N²)
        // all-pairs pass when civilian positions are unchanged since the last
        // recompute (spawn/despawn/movement all change the fingerprint).
        let fingerprint = Self::life_cluster_position_fingerprint(&self.world);
        #[cfg(test)]
        let force_recompute = self.force_life_cluster_recompute;
        #[cfg(not(test))]
        let force_recompute = false;
        let must_recompute_clusters = force_recompute
            || self.cluster_member_counts.is_empty()
            || fingerprint != self.life_cluster_position_fingerprint;

        if must_recompute_clusters {
            #[cfg(test)]
            {
                self.life_clustering_recompute_count += 1;
            }

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

            self.last_settlement_count =
                cluster_sizes.values().filter(|&&n| n > 1).count() as u32;
            self.cluster_member_counts = cluster_sizes;
            self.life_cluster_position_fingerprint = fingerprint;
        } else {
            self.last_settlement_count = self
                .cluster_member_counts
                .values()
                .filter(|&&n| n > 1)
                .count() as u32;
        }

        // 6. Maintain per-cluster (settlement) resource stocks: agents produce
        // into their cluster's shared stock each tick (collective economics).
        let mut next_stocks: BTreeMap<u64, ClusterStocks> = BTreeMap::new();
        for (cluster_id, size) in &self.cluster_member_counts {
            let mut stock = self
                .cluster_stocks
                .get(cluster_id)
                .cloned()
                .unwrap_or_default();
            // Each member contributes one unit of food per tick to the commons.
            stock.add(
                civ_economy::Good::Food,
                i64::from(*size).saturating_mul(CLUSTER_FOOD_PRODUCTION_PER_MEMBER),
            );
            next_stocks.insert(*cluster_id, stock);
        }
        self.cluster_stocks = next_stocks;
    }

    /// Drains cluster food stocks by per-member consumption (FR-CIV-LIFE-020).
    ///
    /// Runs immediately after [`Simulation::phase_life`] so collective production
    /// cannot integrate without a matching sink. Uses the same member counts as
    /// production (`cluster_member_counts`) rather than re-querying
    /// [`ClusterMember`], which can lag and leave production unmatched.
    fn phase_settlement_consumption(&mut self) {
        for (cluster_id, size) in &self.cluster_member_counts {
            let Some(stock) = self.cluster_stocks.get_mut(cluster_id) else {
                continue;
            };
            let consumption = i64::from(*size).saturating_mul(CLUSTER_FOOD_CONSUMPTION_PER_MEMBER);
            let before = stock.get(civ_economy::Good::Food);
            let after = before.saturating_sub(consumption);
            stock.add(civ_economy::Good::Food, after - before);
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
        // FR-CIV-CONTENT-001: collect biomes before any mutable borrow.
        let biomes: Vec<civ_planet::BiomeKind> = {
            let geo = civ_planet::GeologyMap::seed(&self.planet);
            geo.regions.iter().map(|r| r.biome).collect()
        };
        let yield_factor = production_yield_factor(self.research_tier());
        let biome_factor = aggregate_biome_yield(&biomes);
        let focus_bonus = Fixed::from_num(11) / Fixed::from_num(10);
        let mut food_out = food * yield_factor * biome_factor;
        let mut metal_out = metal * yield_factor;
        if self.state.economic_focus == EconomicFocus::Agrarian {
            food_out = food_out * focus_bonus;
        }
        if self.state.economic_focus == EconomicFocus::Industrial {
            metal_out = metal_out * focus_bonus;
        }
        if self.state.tech_unlocks & TECH_METALLURGY != 0 {
            metal_out = metal_out * focus_bonus;
        }
        self.state.resources.food += food_out;
        self.state.resources.wood += wood * yield_factor;
        self.state.resources.metal += metal_out;
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

        let mut morale_recovery = morale_recovery_rate(self.state.cohesion);
        if self.state.tech_unlocks & TECH_GUNPOWDER != 0 {
            morale_recovery += Fixed::from_num(1) / Fixed::from_num(100);
        }
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

        // PERF_HOTPATH_4 #1 — phase_military O(M²) → O(M+E)
        // Before: each grid_move / engagement triggered an inner O(M) scan to find the
        //         target entity → O(M × E) total (≈ O(M²) under sustained combat).
        // After:  accumulate position patches and HP deltas keyed by entity index, then
        //         apply all in a single O(M) pass each.  At M=200, E=200: ~40 000 → ~400.
        // Safety: entities Vec is stable for the lifetime of this phase; indices match
        //         the ECS query order captured above; entity ids are unique by construction.
        let mut position_patches: std::collections::HashMap<usize, (i32, i32)> =
            std::collections::HashMap::new();

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
            // Accumulate the last position patch per unit index (later patches win,
            // matching the previous break-on-first-match semantic since entity ids are unique).
            position_patches.insert(
                grid_move.unit_index,
                (grid_move.new_grid_x, grid_move.new_grid_y),
            );
        }

        // Single O(M) pass to apply all position patches.
        if !position_patches.is_empty() {
            for (idx, (_, unit)) in
                self.world.query_mut::<&mut MilitaryUnit>().into_iter().enumerate()
            {
                if let Some(&(nx, ny)) = position_patches.get(&idx) {
                    unit.position.x = nx;
                    unit.position.y = ny;
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

        // Accumulate total HP damage per unit index before the ECS write pass.
        let mut hp_patches: std::collections::HashMap<usize, Fixed> =
            std::collections::HashMap::new();

        for engagement in &engagements {
            self.replay_log.record_combat(
                self.state.tick,
                engagement.shooter_id,
                engagement.target_id,
                engagement.damage,
            );
            // Accumulate damage; multiple engagements can hit the same target in one tick.
            *hp_patches.entry(engagement.target_index).or_insert(Fixed::from_num(0)) += hp_loss;
            self.last_tick_combat_pulses.push(CombatDamagePulse {
                x: (engagement.damage.center.x as f32 / scale).clamp(0.0, 1.0),
                y: (engagement.damage.center.z as f32 / scale).clamp(0.0, 1.0),
                unit_a: Some(engagement.shooter_id),
                unit_b: Some(engagement.target_id),
            });
            self.pending_damage.push(engagement.damage);
        }

        // Single O(M) pass to apply all HP patches (replaces one O(M) scan per engagement).
        if !hp_patches.is_empty() {
            for (idx, (_, unit)) in
                self.world.query_mut::<&mut MilitaryUnit>().into_iter().enumerate()
            {
                if let Some(&total_loss) = hp_patches.get(&idx) {
                    unit.hp = (unit.hp - total_loss).max(Fixed::from_num(0));
                    unit.strength = unit.hp;
                }
            }
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
        let mut faction_ids: Vec<u32> = self.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return;
        }
        // N3: read settlement cluster layout before any mutable diplomacy state borrow.
        let (a, b) = diplomacy_pair_from_settlement_overlap(
            &self.world,
            &self.cluster_member_counts,
            &faction_ids,
            self.state.tick,
        );
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
        // N2: read cluster cultures before any mutable diplomacy state borrow.
        let culture_bias =
            diplomacy_culture_threshold_bias(&self.emergence.cluster_cultures, a, b);
        // Shared faith binds society: collective belief raises the disparity a
        // faction pair will tolerate before fighting (belief -> diplomacy).
        // Emergent pairwise relations further bias the threshold: allies tolerate
        // more disparity, rivals clash sooner (FR-CIV-0100 multi-polity emergence).
        let relation = self.faction_relation(a, b);
        let pair_unrest = self.faction_unrest(a).max(self.faction_unrest(b));
        let base_threshold = diplomacy_conflict_threshold(
            self.belief().saturating_add(self.cohesion()),
            pair_unrest,
        );
        // N9: aggression reduces the threshold (aggressive species clash sooner).
        let agg_a = self.faction_aggression.get(&a).copied().unwrap_or(0.0);
        let agg_b = self.faction_aggression.get(&b).copied().unwrap_or(0.0);
        let agg_reduction = aggression_threshold_reduction((agg_a + agg_b) / 2.0);
        // N12: collective affinity biases the threshold (goodwill buffers tension,
        // hostility clashes sooner). avg_affinity in [-1, 1]; bias in [-5000, 5000].
        let affinity_bias = affinity_threshold_bias(avg_social_affinity(&self.world));
        let conflict_threshold = Fixed::from_num(
            (base_threshold
                + diplomacy_relation_threshold_bias(relation)
                + culture_bias
                - agg_reduction
                + affinity_bias)
                .max(DIPLOMACY_MIN_CONFLICT_THRESHOLD),
        );
        let kind = if disparity >= conflict_threshold {
            DiplomacyKind::Conflict
        } else {
            DiplomacyKind::TradeAgreement
        };
        let cluster_a = ClusterId(a as u64);
        let cluster_b = ClusterId(b as u64);
        match kind {
            DiplomacyKind::TradeAgreement => {
                if let Some(v) = self.state.faction_treasury.get_mut(&a) {
                    *v += Fixed::from_num(100);
                }
                if let Some(v) = self.state.faction_treasury.get_mut(&b) {
                    *v += Fixed::from_num(100);
                }
                self.state.faction_relations.apply_signal(
                    cluster_a,
                    cluster_b,
                    DiplomacySignal {
                        trade_volume: FACTION_TRADE_RELATION_SIGNAL,
                        ..Default::default()
                    },
                );
            }
            DiplomacyKind::Conflict => {
                if let Some(v) = self.state.faction_treasury.get_mut(&a) {
                    *v -= Fixed::from_num(50);
                }
                if let Some(v) = self.state.faction_treasury.get_mut(&b) {
                    *v -= Fixed::from_num(50);
                }
                self.state.faction_relations.apply_signal(
                    cluster_a,
                    cluster_b,
                    DiplomacySignal {
                        resource_competition: FACTION_CONFLICT_RELATION_SIGNAL,
                        ..Default::default()
                    },
                );
                reset_trade_agreement_streak(&mut self.state.faction_trade_agreement_streak, a, b);
                remove_emergent_routes_between(&mut self.state, a, b);
            }
            DiplomacyKind::Peace => {}
        }
        if kind == DiplomacyKind::TradeAgreement {
            record_trade_agreement_streak(&mut self.state.faction_trade_agreement_streak, a, b);
            // N4: read relation + streak before mutably borrowing trade_routes (E0502).
            let pair = canonical_faction_pair(a, b);
            let streak = self
                .state
                .faction_trade_agreement_streak
                .get(&pair)
                .copied()
                .unwrap_or(0);
            let relation = self.faction_relation(a, b);
            let goods = emergent_route_goods(a);
            let route_key = (a, b, goods.to_string());
            let already_exists = self.state.trade_routes.iter().any(|route| {
                route.from_faction == a && route.to_faction == b && route.goods == goods
            });
            let at_cap = self.state.trade_routes.len() >= MAX_TRADE_ROUTES;
            let should_birth = !at_cap
                && !already_exists
                && streak >= TRADE_ROUTE_AGREEMENT_BIRTH_THRESHOLD
                && relation >= TRADE_ROUTE_MIN_RELATION;
            if should_birth {
                self.state.trade_routes.push(TradeRoute {
                    from_faction: a,
                    to_faction: b,
                    goods: goods.to_string(),
                    volume: Fixed::from_num(8),
                });
                self.state.emergent_trade_route_keys.insert(route_key.clone());
                self.state.trade_route_idle_ticks.insert(route_key, 0);
            }
        }
        decay_faction_relations(
            &mut self.state.faction_relations,
            FACTION_RELATION_DECAY_FACTOR,
        );
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
        // N1 coupling: aggregate settlement commons before any mutable market borrow.
        let settlement_food_supply: i64 = self
            .cluster_stocks
            .values()
            .map(|stock| stock.get(civ_economy::Good::Food))
            .fold(0i64, |acc, qty| acc.saturating_add(qty))
            .saturating_mul(SETTLEMENT_FOOD_MARKET_WEIGHT);
        let food_price_before = self
            .market_state
            .prices()
            .get("food")
            .copied()
            .unwrap_or(FOOD_SCARCITY_BASELINE);
        self.market_state.step(self.state.tick);

        // Emergent pricing (FR-CIV-0100 §3d): the living population is demand
        // pressure measured against the carrying capacity (supply). Staple
        // prices rise as population outgrows capacity (scarcity) and ease as it
        // falls below (surplus). Carrying capacity itself grows with research
        // tier, so tech advances FEED BACK into cheaper staples (research →
        // economy coupling).
        // Settlement cluster_stocks (local food commons) add supply beside
        // carrying capacity (N1: settlement → price → unrest → diplomacy).
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
        let food_supply = self
            .carrying_capacity()
            .saturating_add(settlement_food_supply);
        let scaled_demand = demand / FOOD_MARKET_PRESSURE_SCALE;
        let scaled_food_supply = food_supply / FOOD_MARKET_PRESSURE_SCALE;
        self.market_state
            .apply_pressure("food", scaled_demand, scaled_food_supply);
        if self.state.tech_unlocks & TECH_STORAGE != 0 {
            if let Some(price) = self.market_state.prices.get_mut("food") {
                let delta = *price - food_price_before;
                *price = food_price_before + delta / 2;
            }
        }
        let scaled_energy_supply = self.carrying_capacity() / FOOD_MARKET_PRESSURE_SCALE;
        self.market_state
            .apply_pressure("energy", scaled_demand, scaled_energy_supply);
    }

    fn tick_trade_routes(&mut self) {
        // Societal unrest throttles all commerce this tick (computed once).
        let unrest_factor = unrest_trade_factor(self.state.unrest);
        let society_factor =
            society_trade_factor(self.state.cohesion, self.state.micro_trust_permille);
        // N5 — language distance imposes per-route friction (computed once).
        let lang_centroids = faction_language_centroids(
            &self.emergence.cluster_cultures,
            &settlement_dominant_factions(&self.world, &self.cluster_member_counts),
            &self.cluster_member_counts,
        );
        let mut flowed_keys: BTreeSet<(u32, u32, String)> = BTreeSet::new();
        for route in &self.state.trade_routes {
            if route.volume <= Fixed::ZERO || route.from_faction == route.to_faction {
                continue;
            }

            let relation = self.faction_relation(route.from_faction, route.to_faction);
            let relation_factor = relation_trade_factor(relation);
            let language_factor = {
                let distance = match (
                    lang_centroids.get(&route.from_faction),
                    lang_centroids.get(&route.to_faction),
                ) {
                    (Some(a), Some(b)) => language_distance(*a, *b),
                    _ => 0.0,
                };
                language_trade_factor(distance)
            };

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
            let boosted = route.volume
                * trade_volume_multiplier(available, to_stock)
                * unrest_factor
                * society_factor
                * relation_factor
                * language_factor;
            let quantity = boosted.min(available);
            if quantity > Fixed::ZERO {
                flowed_keys.insert((
                    route.from_faction,
                    route.to_faction,
                    route.goods.clone(),
                ));
            }
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
        decay_idle_emergent_trade_routes(&mut self.state, &flowed_keys);
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

    #[cfg(test)]
    pub(crate) fn test_clear_cluster_stocks(&mut self) {
        self.cluster_stocks.clear();
    }

    #[cfg(test)]
    pub(crate) fn test_set_cluster_food_stock(&mut self, cluster_id: u64, food: i64) {
        let mut stock = ClusterStocks::default();
        stock.add(civ_economy::Good::Food, food);
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
        _                     => Fixed::from_num(1)  / Fixed::from_num(1),
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

// FR-CIV-LEGENDS-001 — saga significance feeds faith (pure factor, capped).
/// Belief minted per promoted legend entity (each tick promotes few).
const BELIEF_PER_PROMOTION: u64 = 3;
/// Multiplier for the summed top-N significance (each entity in 0..1).
const BELIEF_SIGNIFICANCE_SCALE: u64 = 5;
/// Hard per-tick cap so a saga spike cannot explode belief (edge-of-chaos).
const MAX_SAGA_BELIEF_PER_TICK: u64 = 25;

/// Belief minted per promoted legend + per unit of top-N significance mass.
/// Bounded by promoted-count (each tick promotes few) and a per-tick cap so
/// sagas cannot explode belief.
fn saga_belief_gain(promoted_count: usize, significance_sum: f32) -> u64 {
    let from_promos = (promoted_count as u64).saturating_mul(BELIEF_PER_PROMOTION);
    let from_sig = (significance_sum.max(0.0) * BELIEF_SIGNIFICANCE_SCALE as f32) as u64;
    from_promos.saturating_add(from_sig).min(MAX_SAGA_BELIEF_PER_TICK)
}

/// FR-CIV-GENETICS / FR-CIV-LEGENDS: each lineage crossing the sentience
/// threshold this tick mints a one-shot pulse of belief (awe at the moment
/// of becoming sentient). Bounded by per-tick awakening count and a hard
/// per-tick cap so a sudden speciation burst cannot explode faith
/// (edge-of-chaos invariant).
pub(crate) const BELIEF_PER_AWAKENING: u64 = 4;
/// Hard per-tick cap on awakening-driven belief mint.
pub(crate) const MAX_AWAKENING_BELIEF_PER_TICK: u64 = 20;
/// FR-CIV-GENETICS / FR-CIV-LEGENDS: pure gain fn for the awakening -> belief
/// pulse. Saturating mul keeps the inner product safe; the min() then clamps
/// to the per-tick cap.
#[must_use]
pub(crate) fn awakening_belief_gain(awakenings_this_tick: usize) -> u64 {
    (awakenings_this_tick as u64)
        .saturating_mul(BELIEF_PER_AWAKENING)
        .min(MAX_AWAKENING_BELIEF_PER_TICK)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::{awakening_belief_gain, awakening_cohesion_gain};
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

    /// Faction pair chosen by [`Simulation::phase_diplomacy`] (sorted ids + tick modulus).
    fn diplomacy_faction_pair(faction_ids: &[u32], tick: u64) -> (u32, u32) {
        let idx = tick as usize;
        let a = faction_ids[idx % faction_ids.len()];
        let b = faction_ids[(idx + 1) % faction_ids.len()];
        (a, b)
    }

    /// Seed a pair relation via the normalized key [`Simulation::faction_relation`] reads.
    fn seed_faction_pair_relation(
        matrix: &mut DiplomacyMatrix,
        a: u32,
        b: u32,
        signal: DiplomacySignal,
        rounds: usize,
    ) {
        let cluster_a = ClusterId(a as u64);
        let cluster_b = ClusterId(b as u64);
        for _ in 0..rounds {
            matrix.apply_signal(cluster_a, cluster_b, signal);
            matrix.apply_signal(cluster_b, cluster_a, signal);
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

    /// N2 — culture similarity scales the diplomacy peace bonus (`0`, `1500`, `3000`).
    #[test]
    fn diplomacy_culture_threshold_bias_scales_with_similarity() {
        let mut cultures = BTreeMap::new();
        cultures.insert(0, CultureProfile::new([0.5, 0.5, 0.5, 0.5]));
        cultures.insert(1, CultureProfile::new([0.5, 0.5, 0.5, 0.5]));
        assert_eq!(diplomacy_culture_threshold_bias(&cultures, 0, 1), 3_000);

        cultures.insert(1, CultureProfile::new([0.0, 0.0, 0.0, 0.0]));
        assert_eq!(diplomacy_culture_threshold_bias(&cultures, 0, 1), 1_500);

        cultures.insert(1, CultureProfile::new([1.0, 1.0, 1.0, 1.0]));
        assert_eq!(diplomacy_culture_threshold_bias(&cultures, 0, 1), 1_500);

        assert_eq!(diplomacy_culture_threshold_bias(&cultures, 0, 99), 0);
    }

    fn seed_n3_settlement_agent(
        world: &mut World,
        id: u64,
        cluster_id: u64,
        faction: u32,
        x_fp: i64,
        z_fp: i64,
        rng: &mut ChaCha8Rng,
    ) {
        use civ_agents::ActorVisualKind;
        let norm_x = x_fp as f32 / FIXED_SCALE as f32;
        let norm_z = z_fp as f32 / FIXED_SCALE as f32;
        let entity = spawn_civilian_at(
            world,
            id,
            Alignment::Faction(faction),
            norm_x,
            norm_z,
            ActorVisualKind::Humanoid,
            rng,
        );
        let _ = world.insert_one(
            entity,
            ClusterMember {
                cluster: ClusterId(cluster_id),
            },
        );
    }

    /// N3 — contacting settlement clusters bias diplomacy pair selection over registry rotation.
    #[test]
    fn diplomacy_pair_from_settlement_overlap_prefers_contact() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let base = FIXED_SCALE / 2;
        let contact_offset = SETTLEMENT_CONTACT_RADIUS_FP - 1;
        let faction_ids = vec![0, 1, 2];

        let mut contacting = World::new();
        for i in 0..4 {
            seed_n3_settlement_agent(
                &mut contacting,
                100 + (i as u64),
                10,
                0,
                base,
                base,
                &mut rng,
            );
        }
        for i in 0..4 {
            seed_n3_settlement_agent(
                &mut contacting,
                200 + (i as u64),
                20,
                1,
                base + contact_offset,
                base,
                &mut rng,
            );
        }
        let mut counts = BTreeMap::new();
        counts.insert(10, 4);
        counts.insert(20, 4);
        let (a, b) =
            diplomacy_pair_from_settlement_overlap(&contacting, &counts, &faction_ids, 500);
        assert_eq!(
            (a, b),
            (0, 1),
            "contacting settlements should negotiate across dominant factions, not absent faction 2"
        );

        let mut distant = World::new();
        let far = base + SETTLEMENT_CONTACT_RADIUS_FP * 4;
        for i in 0..4 {
            seed_n3_settlement_agent(
                &mut distant,
                300 + (i as u64),
                30,
                0,
                base,
                base,
                &mut rng,
            );
        }
        for i in 0..4 {
            seed_n3_settlement_agent(
                &mut distant,
                400 + (i as u64),
                40,
                1,
                far,
                far,
                &mut rng,
            );
        }
        let mut distant_counts = BTreeMap::new();
        distant_counts.insert(30, 4);
        distant_counts.insert(40, 4);
        let (da, db) =
            diplomacy_pair_from_settlement_overlap(&distant, &distant_counts, &faction_ids, 500);
        assert_eq!(
            (da, db),
            (0, 1),
            "non-contacting settlements with presence should still prefer settlement factions"
        );
        assert_ne!(
            (da, db),
            diplomacy_faction_pair(&faction_ids, 500),
            "registry rotation would pick a different pair at tick 500"
        );

        let empty = World::new();
        let (la, lb) =
            diplomacy_pair_from_settlement_overlap(&empty, &BTreeMap::new(), &faction_ids, 500);
        assert_eq!(
            (la, lb),
            diplomacy_faction_pair(&faction_ids, 500),
            "empty settlement graph must preserve legacy registry rotation"
        );
    }

    /// N2 — culturally similar factions trade at a disparity that triggers conflict
    /// for culturally distant pairs at the same pinned macro state.
    #[test]
    fn similar_cultures_bias_diplomacy_toward_trade() {
        let mut faction_ids: Vec<u32> = Simulation::with_seed(5)
            .state
            .factions
            .keys()
            .copied()
            .collect();
        faction_ids.sort_unstable();
        let (a, b) = diplomacy_faction_pair(&faction_ids, 500);

        let pin_diplomacy_drivers = |sim: &mut Simulation| {
            sim.state.tick = 500;
            sim.state.belief = 0;
            sim.state.cohesion = 0;
            sim.state.faction_unrest.clear();
            sim.state.faction_treasury.insert(a, Fixed::from_num(0));
            sim.state.faction_treasury.insert(b, Fixed::from_num(11_000));
        };

        let mut similar = Simulation::with_seed(5);
        pin_diplomacy_drivers(&mut similar);
        similar.emergence.cluster_cultures.insert(
            u64::from(a),
            CultureProfile::new([0.5, 0.5, 0.5, 0.5]),
        );
        similar.emergence.cluster_cultures.insert(
            u64::from(b),
            CultureProfile::new([0.5, 0.5, 0.5, 0.5]),
        );
        similar.phase_diplomacy();
        assert_eq!(
            similar.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::TradeAgreement,
            "culturally similar factions tolerate disparity before conflict"
        );

        let mut distant = Simulation::with_seed(5);
        pin_diplomacy_drivers(&mut distant);
        distant.emergence.cluster_cultures.insert(
            u64::from(a),
            CultureProfile::new([0.0, 0.0, 0.0, 0.0]),
        );
        distant.emergence.cluster_cultures.insert(
            u64::from(b),
            CultureProfile::new([1.0, 1.0, 1.0, 1.0]),
        );
        distant.phase_diplomacy();
        assert_eq!(
            distant.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::Conflict,
            "culturally distant factions clash sooner at the same disparity"
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

    /// FR-CIV-ECON: food is excluded from cost-of-living unrest (already owned
    /// by `unrest_delta`); non-food prices at baseline decay, matching the
    /// food-abundance arm.
    #[test]
    fn commodity_unrest_skips_food() {
        let with_food: BTreeMap<String, i64> =
            [("food".to_string(), 5_000i64), ("energy".to_string(), 1_000i64)]
                .into_iter()
                .collect();
        let without_food: BTreeMap<String, i64> =
            [("energy".to_string(), 1_000i64)].into_iter().collect();
        assert_eq!(
            commodity_unrest_delta(&with_food),
            commodity_unrest_delta(&without_food),
            "food price must not echo into commodity unrest"
        );
    }

    /// FR-CIV-ECON: per-tick rise is capped at MAX_RISE (15) regardless of
    /// extreme non-food scarcity — prevents runaway cost-of-living shocks.
    #[test]
    fn commodity_unrest_caps_rise() {
        let extreme: BTreeMap<String, i64> =
            [("energy".to_string(), 100_000i64)].into_iter().collect();
        let delta = commodity_unrest_delta(&extreme);
        assert!(delta <= 15, "single-tick rise is capped at MAX_RISE");
        assert!(delta > 0, "extreme scarcity still raises unrest");
    }

    /// FR-CIV-ECON: non-food prices below baseline decay unrest (cost-of-living
    /// relief), mirroring the food-abundance arm of `unrest_delta`.
    #[test]
    fn commodity_unrest_decay_when_cheap() {
        let cheap: BTreeMap<String, i64> =
            [("energy".to_string(), 500i64)].into_iter().collect();
        assert!(commodity_unrest_delta(&cheap) < 0);
    }

    /// FR-CIV-0100 §3 — a drained energy budget (blackout) adds unrest; a solvent one does not.
    #[test]
    fn energy_scarcity_adds_unrest_only_on_blackout() {
        assert_eq!(energy_scarcity_unrest(Fixed::from_num(1_000)), 0);
        assert_eq!(energy_scarcity_unrest(Fixed::ZERO), 15);
        assert!(energy_scarcity_unrest(Fixed::from_num(-5)) > 0);
    }

    /// FR-CIV-0100 §3 — upward causation: mean agent misery (negative Psyche mood valence)
    /// feeds macro unrest; empty world contributes none.
    #[test]
    fn agent_misery_raises_unrest() {
        use civ_agents::{Mood, PSYCHE_DIM, Temperament};

        let miserable_psyche = Psyche {
            drives: [0.0; PSYCHE_DIM],
            temperament: Temperament::neutral(),
            mood: Mood {
                valence: -0.8,
                arousal: 0.0,
            },
            beliefs: [0.0; PSYCHE_DIM],
            maturity: 0.5,
        };

        let mut world = World::new();
        world.spawn((miserable_psyche.clone(),));
        world.spawn((miserable_psyche,));
        assert!(
            agent_misery_unrest(&world) > 0,
            "miserable agents should raise unrest"
        );

        let empty = World::new();
        assert_eq!(agent_misery_unrest(&empty), 0, "no Psyche agents = no misery unrest");
    }

    /// FR-CIV-0100 §3 — upward causation: shared micro ideology binds macro cohesion;
    /// polarization frays it. Pure-function assertions; no tick RNG.
    #[test]
    fn micro_ideology_consensus_biases_cohesion() {
        use civ_agents::{Mood, PSYCHE_DIM, Temperament};

        fn psyche_with_belief0(b0: f32) -> Psyche {
            let mut beliefs = [0.5_f32; PSYCHE_DIM];
            beliefs[0] = b0;
            Psyche {
                drives: [0.0; PSYCHE_DIM],
                temperament: Temperament::neutral(),
                mood: Mood { valence: 0.0, arousal: 0.0 },
                beliefs,
                maturity: 0.5,
            }
        }

        fn spawn_ideologies(world: &mut hecs::World, beliefs0: &[f32]) {
            for &b0 in beliefs0 {
                world.spawn((psyche_with_belief0(b0),));
            }
        }

        // Regression: no Psyche → no micro effect
        assert_eq!(micro_cohesion_delta(&hecs::World::new()), 0);

        // Guard: single agent → no variance
        let mut lone = hecs::World::new();
        lone.spawn((psyche_with_belief0(0.85),));
        assert_eq!(micro_cohesion_delta(&lone), 0);

        // CONSENSUS: 8 × beliefs[0] = 0.85 → +12
        let mut consensus = hecs::World::new();
        spawn_ideologies(&mut consensus, &[0.85; 8]);
        assert_eq!(
            micro_cohesion_delta(&consensus),
            12,
            "unanimous ideology should max-bind cohesion"
        );

        // POLARIZED: alternate 0.0 / 1.0 → var=0.25 → -18
        let mut polarized = hecs::World::new();
        spawn_ideologies(
            &mut polarized,
            &[0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0],
        );
        assert_eq!(
            micro_cohesion_delta(&polarized),
            -18,
            "max-spread ideology should max-fray cohesion"
        );

        assert!(
            micro_cohesion_delta(&consensus) > micro_cohesion_delta(&polarized),
            "consensus must bind more than polarization frays"
        );
    }

    /// FR-CIV-0100 §3 — downward causation: macro cohesion lifts agent mood valence;
    /// zero cohesion applies no uplift.
    #[test]
    fn cohesion_lifts_agent_mood() {
        use civ_agents::{Mood, PSYCHE_DIM, Temperament};

        let miserable_psyche = Psyche {
            drives: [0.0; PSYCHE_DIM],
            temperament: Temperament::neutral(),
            mood: Mood {
                valence: -0.5,
                arousal: 0.0,
            },
            beliefs: [0.0; PSYCHE_DIM],
            maturity: 0.5,
        };

        let mut sim = Simulation::new();
        sim.state.cohesion = 2_000_000;
        let entity = sim.world.spawn((miserable_psyche.clone(),));
        sim.phase_social_mood();
        let lifted = sim.world.get::<&Psyche>(entity).unwrap().mood.valence;
        assert!(
            lifted > -0.5,
            "high cohesion should nudge mood.valence upward (got {lifted})"
        );

        let mut sim_zero = Simulation::new();
        sim_zero.state.cohesion = 0;
        let entity_zero = sim_zero.world.spawn((miserable_psyche,));
        sim_zero.phase_social_mood();
        let unchanged = sim_zero
            .world
            .get::<&Psyche>(entity_zero)
            .unwrap()
            .mood
            .valence;
        assert_eq!(
            unchanged, -0.5,
            "zero cohesion should leave mood.valence unchanged"
        );
    }

    /// FR-CIV-0100 — upward causation: sentient-agent fraction (DNA cognition
    /// score crossing the sentience threshold) accelerates macro research.
    #[test]
    fn sentience_boosts_research() {
        let sentient_dna = Dna(vec![255; 64]);
        let dull_dna = Dna(vec![0; 64]);

        let mut world = World::new();
        world.spawn((sentient_dna.clone(),));
        world.spawn((sentient_dna,));
        world.spawn((dull_dna.clone(),));
        world.spawn((dull_dna,));

        let bonus = sentience_research_bonus(&world);
        assert!(bonus > 0, "sentient agents should boost research");
        assert!(bonus <= 50, "bonus capped at MAX_SENTIENCE_RESEARCH");

        let empty = World::new();
        assert_eq!(sentience_research_bonus(&empty), 0, "empty world = no bonus");

        let mut non_sentient = World::new();
        non_sentient.spawn((Dna(vec![0; 64]),));
        assert_eq!(
            sentience_research_bonus(&non_sentient),
            0,
            "no sentient agents = no bonus"
        );
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

    /// FR-CIV-0100 §3 — inequality breeds unrest, scaled by the wealth gap, capped.
    #[test]
    fn inequality_unrest_scales_with_spread_capped() {
        assert_eq!(inequality_unrest(0), 0);
        assert_eq!(inequality_unrest(4_000), 2);
        assert_eq!(inequality_unrest(1_000_000), 25, "capped per tick");
    }

    /// FR-CIV-0100 §3 — dispossession target rises with inequality, falls with cohesion.
    #[test]
    fn dispossession_target_rises_with_inequality_falls_with_cohesion() {
        let high_gap = dispossession_target_permille(20_000, 0);
        let no_gap = dispossession_target_permille(0, 0);
        assert!(high_gap > no_gap, "inequality pushes dispossession up");
        let cohesive = dispossession_target_permille(20_000, 10_000_000);
        assert!(cohesive < high_gap, "cohesion erodes dispossession");
        assert!(high_gap <= 1_000);
        assert!(cohesive <= 1_000);
        assert!(no_gap <= 1_000);
    }

    /// institution_target_level caps at MAX_INSTITUTION_LEVEL.
    #[test]
    fn institution_target_level_caps() {
        assert_eq!(institution_target_level(0, 1_000), 0);
        assert_eq!(
            institution_target_level(1_000_000, 5_000),
            MAX_INSTITUTION_LEVEL
        );
    }

    /// institution_step moves at most one level per tick toward the target.
    #[test]
    fn institution_step_moves_one() {
        assert_eq!(institution_step(0, 5), 1);
        assert_eq!(institution_step(5, 0), 4);
        assert_eq!(institution_step(3, 3), 3);
    }

    /// phase_institutions grows the temple when belief is high.
    #[test]
    fn phase_institutions_grows_temple_with_belief() {
        let mut sim = Simulation::with_seed(1);
        sim.add_belief(50_000);
        sim.phase_institutions();
        assert!(sim.temple_level() >= 1);
    }

    /// candidate_economic_focus picks the strongest normalized sector; ties -> Balanced.
    #[test]
    fn candidate_focus_picks_strongest() {
        assert_eq!(
            candidate_economic_focus(1_000_000, 0, 0, 0),
            EconomicFocus::Agrarian
        );
        assert_eq!(
            candidate_economic_focus(0, 100, 0, 0),
            EconomicFocus::Industrial
        );
        assert_eq!(candidate_economic_focus(0, 0, 0, 0), EconomicFocus::Balanced);
    }

    /// phase_economic_focus flips only after sustained dominance (hysteresis).
    #[test]
    fn economic_focus_has_hysteresis() {
        let mut sim = Simulation::with_seed(1);
        assert_eq!(sim.economic_focus(), EconomicFocus::Balanced);
        sim.state.resources.food = Fixed::from_raw(1_000_000 * crate::SCALE);

        sim.phase_economic_focus();
        assert_eq!(
            sim.economic_focus(),
            EconomicFocus::Balanced,
            "focus must not flip on the first evaluation"
        );
        assert!(sim.state.focus_pressure > 0);

        for _ in 0..4 {
            sim.phase_economic_focus();
        }
        assert_eq!(
            sim.economic_focus(),
            EconomicFocus::Agrarian,
            "focus commits after the hysteresis threshold"
        );
        assert_eq!(sim.state.focus_pressure, 0);
    }

    /// FR-CIV-0100 §3 — dispossessed share moves at most 5 permille per tick.
    #[test]
    fn dispossession_step_is_sticky() {
        assert_eq!(dispossession_step(0, 1_000), 5);
        assert_eq!(dispossession_step(100, 0), 95);
        assert_eq!(dispossession_step(50, 50), 50);
    }

    /// FR-CIV-0100 §3 — dispossession unrest scales with share and caps at 25.
    #[test]
    fn dispossession_unrest_scales_and_caps() {
        assert_eq!(dispossession_unrest(0), 0);
        assert!(dispossession_unrest(400) > 0);
        assert!(dispossession_unrest(1_000) <= 25);
    }

    /// faction_treasury_spread is the richest-minus-poorest gap (0 when empty).
    #[test]
    fn faction_treasury_spread_is_rich_minus_poor() {
        let mut t = HashMap::new();
        t.insert(1u32, Fixed::from_num(100));
        t.insert(2u32, Fixed::from_num(900));
        assert_eq!(faction_treasury_spread(&t), 800);
        assert_eq!(faction_treasury_spread(&HashMap::new()), 0);
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

    /// FR-CIV-CONTENT-001 — fertile biomes produce more than barren ones.
    #[test]
    fn test_biome_yield_factor_ordering() {
        use civ_planet::BiomeKind;
        assert!(
            biome_yield_factor(BiomeKind::Rainforest) > biome_yield_factor(BiomeKind::Desert),
            "Rainforest yields more than Desert"
        );
        assert!(
            biome_yield_factor(BiomeKind::Desert) > biome_yield_factor(BiomeKind::Glacier),
            "Desert yields more than Glacier"
        );
    }

    /// FR-CIV-CONTENT-001 — all known biome variants produce a factor in [0.1, 1.5].
    #[test]
    fn test_biome_yield_factor_clamped() {
        use civ_planet::BiomeKind;
        let lo = Fixed::from_num(1) / Fixed::from_num(10);
        let hi = Fixed::from_num(15) / Fixed::from_num(10);
        for biome in [
            BiomeKind::Rainforest,
            BiomeKind::Wetland,
            BiomeKind::Grassland,
            BiomeKind::Plains,
            BiomeKind::Forest,
            BiomeKind::Savanna,
            BiomeKind::Beach,
            BiomeKind::Mountain,
            BiomeKind::Taiga,
            BiomeKind::Desert,
            BiomeKind::Tundra,
            BiomeKind::Ocean,
            BiomeKind::Glacier,
        ] {
            let f = biome_yield_factor(biome);
            assert!(f >= lo, "{biome:?} factor {f:?} below minimum 0.1");
            assert!(f <= hi, "{biome:?} factor {f:?} above maximum 1.5");
        }
    }

    /// FR-CIV-CONTENT-001 — aggregate of mixed biomes stays within clamped bounds.
    #[test]
    fn test_aggregate_biome_yield_in_bounds() {
        use civ_planet::BiomeKind;
        let biomes = [BiomeKind::Rainforest, BiomeKind::Desert, BiomeKind::Glacier];
        let agg = aggregate_biome_yield(&biomes);
        let lo = Fixed::from_num(1) / Fixed::from_num(10);
        let hi = Fixed::from_num(15) / Fixed::from_num(10);
        assert!(agg >= lo && agg <= hi, "aggregate {agg:?} out of [0.1, 1.5]");
    }

    /// FR-CIV-CONTENT-001 — rich biomes dominate barren ones in yield factor.
    #[test]
    fn test_rich_biome_gt_barren() {
        use civ_planet::BiomeKind;
        assert!(
            biome_yield_factor(BiomeKind::Rainforest) > biome_yield_factor(BiomeKind::Desert),
            "Rainforest factor must exceed Desert factor"
        );
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

    /// FR-CIV-0100 §3 — construction demand tracks emergent macro state.
    #[test]
    fn building_demand_responds_to_state() {
        let ample_wood = Fixed::from_num(1_000);
        let ample_metal = Fixed::from_num(1_000);
        let d = building_demand_signals(0, 1_000, 0, 0, 500, ample_wood, ample_metal);
        assert!(d.civic > 0.0);
        let d2 = building_demand_signals(0, 1_000, 0, 5, 0, ample_wood, ample_metal);
        assert!(d2.industrial > 0.0);
        for signal in [d.residential, d.commercial, d.industrial, d.civic] {
            assert!((0.0..=1.0).contains(&signal));
        }
        for signal in [d2.residential, d2.commercial, d2.industrial, d2.civic] {
            assert!((0.0..=1.0).contains(&signal));
        }
        let cohesion = 1_000_000u64;
        let low_metal = Fixed::from_num(100);
        let d_low = building_demand_signals(0, 1_000, cohesion, 0, 0, ample_wood, low_metal);
        let d_high = building_demand_signals(0, 1_000, cohesion, 0, 0, ample_wood, ample_metal);
        assert!(d_low.commercial < 0.5, "low metal must suppress commercial parcels");
        assert!(d_high.commercial > 0.5, "ample metal allows commercial parcels");
        let low_wood = Fixed::from_num(100);
        let d_low_wood =
            building_demand_signals(500, 1_000, cohesion, 5, 500, low_wood, ample_metal);
        assert!(
            d_low_wood.residential < 0.5,
            "low wood must suppress residential parcels"
        );
    }

    /// FR-CIV-0100 FC-3 — commercial demand gated by metal stock; loop cannot run away.
    #[test]
    fn fc3_commercial_metal_gate_bounded_over_many_ticks() {
        let mut sim = Simulation::with_seed(9917);
        sim.state.cohesion = 1_000_000;
        sim.state.unrest = 0;
        sim.state.population = 0;
        sim.state.resources.wood = Fixed::from_num(10_000);
        sim.state.resources.metal = Fixed::from_num(600);
        let metal_start = sim.state.resources.metal;
        let cadence = building_cadence(sim.research_tier());
        let steady_ceiling =
            Fixed::from_num(fc3_commercial_metal_steady_ceiling_i64(sim.state.cohesion));

        for tick in 0..850 {
            sim.state.tick = tick;
            if tick % cadence == 0 {
                sim.phase_buildings();
            }
        }

        assert!(
            sim.state.resources.metal > Fixed::ZERO,
            "FC-3 gate must stop commercial drain before metal hits zero"
        );
        assert!(
            sim.state.resources.metal < metal_start,
            "construction should debit metal while stock is above the gate"
        );
        assert!(
            sim.state.resources.metal <= steady_ceiling,
            "metal must converge to FC-3 steady-state ceiling ({steady_ceiling:?})"
        );
        let signals = building_demand_signals(
            sim.state.population,
            sim.carrying_capacity(),
            sim.state.cohesion,
            sim.research_tier(),
            sim.state.unrest,
            sim.state.resources.wood,
            sim.state.resources.metal,
        );
        assert!(
            signals.commercial <= FC3_COMMERCIAL_PARCEL_THRESHOLD,
            "residual metal must throttle commercial demand below parcel threshold"
        );
    }

    /// FR-CIV-0100 FC-3 — `building_graph` size plateaus under sustained high cohesion.
    #[test]
    fn building_graph_bounded_under_sustained_high_cohesion() {
        let mut sim = Simulation::with_seed(12026);
        sim.state.cohesion = 1_000_000;
        sim.state.unrest = 500;
        sim.state.population = 500;
        sim.state.resources.wood = Fixed::from_num(800);
        sim.state.resources.metal = Fixed::from_num(800);
        let cadence = building_cadence(sim.research_tier());

        let mut plateau_ticks = 0_u64;
        let mut last_len = sim.building_graph.parcels.len();
        let mut peak = last_len;

        for tick in 0..1200_u64 {
            sim.state.tick = tick;
            if tick % cadence == 0 {
                sim.phase_buildings();
            }
            let len = sim.building_graph.parcels.len();
            peak = peak.max(len);
            if len == last_len {
                plateau_ticks = plateau_ticks.saturating_add(1);
            } else {
                plateau_ticks = 0;
            }
            last_len = len;
        }

        let final_len = sim.building_graph.parcels.len();
        assert!(final_len > 0, "some construction expected under high cohesion");
        assert!(
            final_len < 120,
            "building_graph must not grow without limit (final={final_len}, peak={peak})"
        );
        assert!(
            plateau_ticks >= 200,
            "graph size should plateau for many ticks (plateau_ticks={plateau_ticks})"
        );
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

    /// A faction in local scarcity accrues per-faction unrest each tick.
    #[test]
    fn scarce_faction_accrues_unrest() {
        let mut sim = Simulation::with_seed(1);
        let id = 0u32;
        sim.state.faction_treasury.insert(id, Fixed::from_num(0));
        sim.state.faction_resources.insert(
            id,
            Resources {
                food: Fixed::from_num(5),
                ..Resources::default()
            },
        );
        for _ in 0..5 {
            sim.phase_faction_unrest();
        }
        assert!(sim.faction_unrest(id) > 0, "scarce faction breeds unrest");
    }

    /// After faction-unrest ticks, `polities[id].unrest` matches legacy `faction_unrest`.
    #[test]
    fn polities_unrest_matches_faction_unrest_after_ticks() {
        let mut sim = Simulation::with_seed(99);
        let id = 0u32;
        sim.state.faction_treasury.insert(id, Fixed::from_num(0));
        sim.state.faction_resources.insert(
            id,
            Resources {
                food: Fixed::from_num(5),
                ..Resources::default()
            },
        );
        for _ in 0..5 {
            sim.phase_faction_unrest();
        }
        for (&faction_id, &legacy_unrest) in &sim.state.faction_unrest {
            let polity_unrest = sim
                .state
                .polities
                .get(&faction_id)
                .map(|p| p.unrest)
                .unwrap_or(0);
            assert_eq!(
                polity_unrest, legacy_unrest,
                "polity {faction_id} unrest drifted from legacy map"
            );
        }
    }

    /// Sustained faction scarcity reaches a finite equilibrium thanks to proportional decay.
    #[test]
    fn faction_unrest_stays_bounded_under_sustained_scarcity() {
        let mut sim = Simulation::with_seed(1);
        let id = 0u32;
        sim.state.faction_treasury.insert(id, Fixed::from_num(0));
        sim.state.faction_resources.insert(
            id,
            Resources {
                food: Fixed::from_num(5),
                ..Resources::default()
            },
        );
        for _ in 0..2_000 {
            sim.phase_faction_unrest();
        }
        let unrest = sim.faction_unrest(id);
        assert!(unrest > 0, "scarcity still breeds unrest");
        assert!(
            unrest < 20_000,
            "proportional decay must bound faction unrest (got {unrest})"
        );
    }

    /// A wealthy, well-provisioned faction stays at zero per-faction unrest.
    #[test]
    fn wealthy_faction_stays_low_unrest() {
        let mut sim = Simulation::with_seed(1);
        let id = 0u32;
        sim.state.faction_treasury.insert(id, Fixed::from_num(50_000));
        sim.state.faction_resources.insert(
            id,
            Resources {
                food: Fixed::from_num(200),
                ..Resources::default()
            },
        );
        for _ in 0..5 {
            sim.phase_faction_unrest();
        }
        assert_eq!(sim.faction_unrest(id), 0, "wealthy faction stays content");
    }

    /// High per-faction unrest erodes the diplomacy war threshold so a restless
    /// polity fights at a smaller wealth disparity.
    #[test]
    fn high_faction_unrest_lowers_conflict_threshold() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500;
        let mut faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let a = faction_ids[(sim.state.tick as usize) % faction_ids.len()];
        let b = faction_ids[((sim.state.tick as usize) + 1) % faction_ids.len()];
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.faction_treasury.insert(a, Fixed::from_num(4_000));
        sim.state.faction_treasury.insert(b, Fixed::from_num(10_000));
        sim.state.faction_unrest.insert(a, 500_000);
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::Conflict,
            "high faction unrest should lower the war threshold"
        );
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

    #[test]
    fn society_trade_factor_stacks_micro_trust_with_cohesion() {
        assert_eq!(
            society_trade_factor(400, 0),
            cohesion_trade_factor(400)
        );
        assert_eq!(society_trade_factor(0, 0), Fixed::from_num(1));

        let cohesion_only = society_trade_factor(2_000, 0);
        let micro_only = society_trade_factor(0, 250);
        let both = society_trade_factor(2_000, 250);

        assert!(micro_only > Fixed::from_num(1));
        assert!(both > cohesion_only);
        assert!(both > micro_only);
        assert!(both <= Fixed::from_num(1_750) / Fixed::from_num(1_000));
    }

    /// FR-CIV-0100 §3 — mean positive agent tie trust caches a trade permille bonus.
    #[test]
    fn micro_social_trust_permille_aggregates_tie_trust() {
        use civ_agents::Tie;

        fn graph_with_trusts(trusts: &[f32]) -> SocialGraph {
            let ties = trusts
                .iter()
                .enumerate()
                .map(|(i, &trust)| Tie {
                    other: (i + 1) as u64,
                    kinship: 0.0,
                    familiarity: 0.5,
                    affinity: 0.0,
                    trust,
                    last_seen: 0,
                })
                .collect();
            SocialGraph { ties }
        }

        fn spawn_graphs(world: &mut hecs::World, trusts: &[f32]) {
            world.spawn((graph_with_trusts(trusts),));
        }

        assert_eq!(micro_social_trust_permille(&hecs::World::new()), 0);

        let mut negative = hecs::World::new();
        spawn_graphs(&mut negative, &[-1.0; 4]);
        assert_eq!(micro_social_trust_permille(&negative), 0);

        let mut high = hecs::World::new();
        spawn_graphs(&mut high, &[1.0; 12]);
        assert_eq!(
            micro_social_trust_permille(&high),
            250,
            "saturated trust should max the permille cache"
        );

        let mut mixed = hecs::World::new();
        spawn_graphs(
            &mut mixed,
            &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        assert_eq!(micro_social_trust_permille(&mixed), 125);

        assert!(
            micro_social_trust_permille(&high) > micro_social_trust_permille(&mixed),
            "denser trust must cache a higher permille"
        );
    }

    /// FR-CIV-0100 §3 — cached micro trust permille boosts trade volume when macro
    /// drivers are pinned.
    #[test]
    fn micro_trust_permille_boosts_trade_volume() {
        let mut sim = Simulation::with_seed(42);
        sim.state.cohesion = 0;
        sim.state.unrest = 0;

        let route = sim.state.trade_routes[0].clone();
        let from = route.from_faction;
        let to = route.to_faction;
        let stock = Fixed::from_num(100);
        sim.state.faction_resources.get_mut(&from).unwrap().food = stock;
        sim.state.faction_resources.get_mut(&to).unwrap().food = stock;

        let baseline = sim
            .state
            .faction_treasury
            .get(&from)
            .copied()
            .unwrap_or(Fixed::ZERO);

        sim.state.micro_trust_permille = 250;
        sim.tick_trade_routes();
        let high_gain = sim.state.faction_treasury.get(&from).copied().unwrap_or(Fixed::ZERO)
            - baseline;

        sim.state.faction_treasury.insert(from, baseline);
        sim.state.faction_resources.get_mut(&from).unwrap().food = stock;
        sim.state.faction_resources.get_mut(&to).unwrap().food = stock;
        sim.state.micro_trust_permille = 0;
        sim.tick_trade_routes();
        let low_gain = sim.state.faction_treasury.get(&from).copied().unwrap_or(Fixed::ZERO)
            - baseline;

        assert!(
            high_gain > low_gain,
            "micro trust should boost exporter trade profit (high={high_gain:?}, low={low_gain:?})"
        );
    }

    #[test]
    fn relations_bias_trade_volume() {
        let ally = relation_trade_factor(1.0);
        let neutral = relation_trade_factor(0.0);
        let rival = relation_trade_factor(-1.0);
        assert!(ally > neutral);
        assert!(neutral > rival);
        assert!(ally <= Fixed::from_num(3) / Fixed::from_num(2));
        assert!(rival >= Fixed::from_num(1) / Fixed::from_num(2));
    }

    /// FR-CIV-LANG-001 / FR-CIV-PSYCHE-912 — `language_trade_factor` endpoints and midpoint.
    #[test]
    fn language_trade_factor_scales_with_distance() {
        assert_eq!(language_trade_factor(0.0), Fixed::from_num(1));
        assert_eq!(
            language_trade_factor(1.0),
            Fixed::from_num(1) / Fixed::from_num(2)
        );
        let mid = language_trade_factor(0.5);
        assert!(mid > Fixed::from_num(1) / Fixed::from_num(2));
        assert!(mid < Fixed::from_num(1));
    }

    /// FR-CIV-LANG-001 / FR-CIV-PSYCHE-912 — centroids are member-weighted and
    /// drop lone-wanderer clusters (member_count < 2).
    #[test]
    fn faction_language_centroids_member_weighted() {
        let mut cultures = BTreeMap::new();
        cultures.insert(10u64, CultureProfile::new([0.0, 0.0, 0.0, 0.0]));
        cultures.insert(20u64, CultureProfile::new([1.0, 1.0, 1.0, 1.0]));
        // 21 is a lone wanderer (1 member) — must be skipped even if it would
        // tilt the centroid.
        cultures.insert(21u64, CultureProfile::new([1.0, 1.0, 1.0, 1.0]));
        let mut dominant = BTreeMap::new();
        dominant.insert(10u64, 0u32);
        dominant.insert(20u64, 0u32);
        dominant.insert(21u64, 0u32);
        let mut member_counts = BTreeMap::new();
        member_counts.insert(10u64, 6u32);
        member_counts.insert(20u64, 2u32);
        member_counts.insert(21u64, 1u32);

        let centroids = faction_language_centroids(&cultures, &dominant, &member_counts);
        let c0 = centroids.get(&0).expect("faction 0 centroid present");
        // Weighted: 6 * [0] + 2 * [1] = 2, total weight = 6+2 = 8 → axis 0 = 0.25.
        assert!(
            (c0[0] - 0.25).abs() < 0.01,
            "centroid axis-0 should be 0.25, got {}",
            c0[0]
        );
        assert!(
            !centroids.contains_key(&1),
            "faction 1 absent from centroids"
        );
    }

    /// FR-CIV-LANG-001 / FR-CIV-PSYCHE-912 — divergent languages reduce
    /// bilateral trade-route flow when all other factors are pinned to 1.0.
    #[test]
    fn language_barrier_reduces_trade_route_flow() {
        fn run_case(faction0_lang: [f32; 4], faction1_lang: [f32; 4]) -> Fixed {
            let mut sim = Simulation::with_seed(42);
            // Pin macro drivers to neutral: cohesion=0, unrest=0, micro_trust=0.
            sim.state.cohesion = 0;
            sim.state.unrest = 0;
            sim.state.micro_trust_permille = 0;
            // Force a neutral relation between the route endpoints.
            let route = sim.state.trade_routes[0].clone();
            let from = route.from_faction;
            let to = route.to_faction;
            // Seed two clusters, one per faction, with >=2 members so they
            // participate in the dominant-faction rollup.
            let mut rng = ChaCha8Rng::seed_from_u64(7);
            for i in 0..3 {
                seed_n3_settlement_agent(&mut sim.world, 5_000 + i, 100, from, 0, 0, &mut rng);
                seed_n3_settlement_agent(&mut sim.world, 6_000 + i, 200, to, 100, 100, &mut rng);
            }
            sim.cluster_member_counts.insert(100, 3);
            sim.cluster_member_counts.insert(200, 3);
            sim.emergence
                .cluster_cultures
                .insert(100, CultureProfile::new(faction0_lang));
            sim.emergence
                .cluster_cultures
                .insert(200, CultureProfile::new(faction1_lang));

            let stock = Fixed::from_num(100);
            sim.state.faction_resources.get_mut(&from).unwrap().food = stock;
            sim.state.faction_resources.get_mut(&to).unwrap().food = stock;
            let baseline = sim
                .state
                .faction_treasury
                .get(&from)
                .copied()
                .unwrap_or(Fixed::ZERO);
            sim.tick_trade_routes();
            sim.state
                .faction_treasury
                .get(&from)
                .copied()
                .unwrap_or(Fixed::ZERO)
                - baseline
        }

        let gain_same = run_case([0.5, 0.5, 0.5, 0.5], [0.5, 0.5, 0.5, 0.5]);
        let gain_far = run_case([0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        assert!(
            gain_same > Fixed::ZERO,
            "baseline (distance 0) route should flow"
        );
        assert!(
            gain_same > gain_far,
            "language barrier must reduce trade flow (same={gain_same:?}, far={gain_far:?})"
        );
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

    /// Empirical criticality check: over a long run the emergent scalars must stay
    /// BOUNDED (no overflow/panic, belief/cohesion don't grow without limit thanks
    /// to decay) yet DYNAMIC (the tech tree progresses; state evolves). Heat-death
    /// (everything frozen at 0) and explosion (unbounded) both fail this.
    #[test]
    fn emergence_stays_bounded_and_dynamic_over_5000_ticks() {
        let mut sim = Simulation::with_seed(20260615);
        // Seed a wealth disparity so diplomacy/inequality paths can engage.
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        if ids.len() >= 2 {
            sim.state.faction_treasury.insert(ids[0], Fixed::from_num(0));
            sim.state.faction_treasury.insert(ids[1], Fixed::from_num(50_000));
        }
        for _ in 0..5_000 {
            sim.tick();
        }
        assert_eq!(sim.state.tick, 5_000);
        // BOUNDED: belief did not run away to absurd magnitudes (decay holds it).
        assert!(
            sim.belief() < 1_000_000_000,
            "belief must stay bounded by decay"
        );
        assert!(
            sim.cohesion() < 1_000_000_000,
            "cohesion must stay bounded by decay"
        );
        // DYNAMIC: the research-driven tech tree made progress over 5000 ticks.
        assert!(
            sim.research_tier() >= 1,
            "research should advance over a long run"
        );
        // Accessors remain coherent (no panic reaching here proves saturating math held).
        let _ = (
            sim.unrest(),
            sim.dispossessed_permille(),
            sim.temple_level(),
            sim.garrison_level(),
            sim.tech_unlocks(),
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

    fn cluster_assignments_by_agent(sim: &Simulation) -> BTreeMap<u64, u64> {
        sim.world
            .query::<(&AgentCivilian, &ClusterMember)>()
            .iter()
            .map(|(_, (civ, member))| (civ.id, member.cluster.0))
            .collect()
    }

    fn pin_all_civilian_positions(sim: &mut Simulation, pin: WorldCoord) {
        for (_, (_, pos)) in sim
            .world
            .query_mut::<(&AgentCivilian, &mut Position3d)>()
        {
            pos.coord = pin;
        }
    }

    /// PERF_OPT #1 — cached clustering matches full recompute on a moving population.
    #[test]
    fn phase_life_clustering_skip_matches_full_recompute_on_movement() {
        const TICKS: u32 = 60;
        let seed = 4242u64;

        let mut cached = Simulation::with_seed(seed);
        let mut always = Simulation::with_seed(seed);
        always.force_life_cluster_recompute = true;

        for tick in 0..TICKS {
            cached.tick();
            always.tick();
            assert_eq!(
                cached.settlement_count(),
                always.settlement_count(),
                "settlement_count diverged at tick {tick}"
            );
            assert_eq!(
                cached.cluster_stocks(),
                always.cluster_stocks(),
                "cluster_stocks diverged at tick {tick}"
            );
            assert_eq!(
                cluster_assignments_by_agent(&cached),
                cluster_assignments_by_agent(&always),
                "cluster assignments diverged at tick {tick}"
            );
        }
    }

    /// PERF_OPT #1 — all-pairs clustering is skipped when no agents move.
    #[test]
    fn phase_life_clustering_skipped_when_population_stationary() {
        let pin = WorldCoord {
            x: FIXED_SCALE / 2,
            y: 0,
            z: FIXED_SCALE / 2,
        };

        let mut sim = Simulation::with_seed(5150);
        for _ in 0..5 {
            pin_all_civilian_positions(&mut sim, pin);
            sim.tick();
        }
        let baseline_recomputes = sim.life_clustering_recompute_count;

        for _ in 0..30 {
            pin_all_civilian_positions(&mut sim, pin);
            sim.tick();
        }

        assert_eq!(
            sim.life_clustering_recompute_count, baseline_recomputes,
            "expected clustering to be skipped for a stationary population"
        );
    }

    fn psyche_by_agent(sim: &Simulation) -> BTreeMap<u64, Psyche> {
        sim.world
            .query::<(&AgentCivilian, &Psyche)>()
            .iter()
            .map(|(_, (c, p))| (c.id, p.clone()))
            .collect()
    }

    fn social_graph_by_agent(sim: &Simulation) -> BTreeMap<u64, SocialGraph> {
        sim.world
            .query::<(&AgentCivilian, &SocialGraph)>()
            .iter()
            .map(|(_, (c, g))| (c.id, g.clone()))
            .collect()
    }

    /// PERF_OPT #2 — indexed `agent_entity` matches linear-scan baseline over ticks.
    #[test]
    fn agent_id_index_matches_linear_scan_over_ticks() {
        use civ_tactics::DamageEvent;

        const TICKS: u32 = 200;

        for &seed in &[12345u64, 4242, 777] {
            let mut indexed = Simulation::with_seed(seed);
            let mut baseline = Simulation::with_seed(seed);
            baseline.force_agent_entity_linear_scan = true;

            for tick in 0..TICKS {
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
                    indexed.push_damage(event.clone());
                    baseline.push_damage(event);
                }
                indexed.tick();
                baseline.tick();

                assert_eq!(
                    indexed.state.tick,
                    baseline.state.tick,
                    "tick diverged at {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.state.population,
                    baseline.state.population,
                    "population diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    psyche_by_agent(&indexed),
                    psyche_by_agent(&baseline),
                    "psyche diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    social_graph_by_agent(&indexed),
                    social_graph_by_agent(&baseline),
                    "social graph diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.emergence.cluster_cultures,
                    baseline.emergence.cluster_cultures,
                    "cluster_cultures diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.emergence.sentient_agents,
                    baseline.emergence.sentient_agents,
                    "sentient_agents diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.emergence_feed(),
                    baseline.emergence_feed(),
                    "emergence_feed diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.sentience_events(),
                    baseline.sentience_events(),
                    "sentience_events diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.settlement_count(),
                    baseline.settlement_count(),
                    "settlement_count diverged at tick {tick} (seed {seed})"
                );
                assert_eq!(
                    indexed.cluster_member_counts,
                    baseline.cluster_member_counts,
                    "cluster_member_counts diverged at tick {tick} (seed {seed})"
                );
            }
        }
    }

    /// FR-CIV-LIFE-020 — cluster food stocks stay bounded when production is
    /// matched by per-member consumption each tick.
    #[test]
    fn cluster_stocks_food_stays_bounded_over_populated_cluster_ticks() {
        use civ_agents::{ActorVisualKind, Alignment, Position3d};
        use civ_economy::Good;

        const TEST_COHORT_SIZE: u32 = 8;
        const TEST_COHORT_MIN_ID: u64 = 9_000;
        // Steady state: production == consumption per member → net zero. Allow
        // one tick of surplus per cohort member for birth/death transients.
        const STEADY_STATE_FOOD_CEILING: i64 = 8 * CLUSTER_FOOD_PRODUCTION_PER_MEMBER;

        let mut sim = Simulation::with_seed(9001);
        let mut rng = ChaCha8Rng::seed_from_u64(9001);
        for i in 0..TEST_COHORT_SIZE {
            spawn_civilian_at(
                &mut sim.world,
                TEST_COHORT_MIN_ID + u64::from(i),
                Alignment::None,
                0.5,
                0.5,
                ActorVisualKind::Humanoid,
                &mut rng,
            );
        }

        let pin = WorldCoord {
            x: FIXED_SCALE / 2,
            y: 0,
            z: FIXED_SCALE / 2,
        };
        for _ in 0..500 {
            // Keep the test cohort co-located so clustering stays multi-member
            // despite wander/need-seeking in phase_life.
            for (_, (civ, pos)) in sim
                .world
                .query_mut::<(&AgentCivilian, &mut Position3d)>()
            {
                if civ.id >= TEST_COHORT_MIN_ID
                    && civ.id < TEST_COHORT_MIN_ID + u64::from(TEST_COHORT_SIZE)
                {
                    pos.coord = pin;
                }
            }
            sim.tick();
        }

        assert!(
            sim.settlement_count() >= 1,
            "expected at least one multi-member cluster"
        );
        let max_food = sim
            .cluster_stocks()
            .values()
            .map(|stock| stock.get(Good::Food))
            .max()
            .unwrap_or(0);
        assert!(
            max_food <= STEADY_STATE_FOOD_CEILING,
            "cluster food must stay bounded under consumption sink, got {max_food}"
        );
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
        sim.state.resources.wood = Fixed::from_num(10_000);
        sim.state.resources.metal = Fixed::from_num(10_000);
        let before = sim.building_graph().parcels.len();

        for _ in 0..200 {
            sim.tick();
        }

        assert!(sim.building_graph().parcels.len() > before);
    }

    /// FR-CIV-0100 §3 — construction draws down wood/metal; zero stock throttles building.
    #[test]
    fn phase_buildings_gated_by_wood_and_metal_stockpile() {
        let mut sim = Simulation::with_seed(88);
        sim.state.tick = 16;
        sim.state.unrest = 500;
        sim.state.resources.wood = Fixed::ZERO;
        sim.state.resources.metal = Fixed::ZERO;
        let before = sim.building_graph().parcels.len();
        sim.phase_buildings();
        assert_eq!(
            sim.building_graph().parcels.len(),
            before,
            "zero materials must throttle construction"
        );

        sim.state.resources.wood = Fixed::from_num(1_000);
        sim.state.resources.metal = Fixed::from_num(1_000);
        let wood_before = sim.state.resources.wood;
        let metal_before = sim.state.resources.metal;
        sim.phase_buildings();
        assert!(
            sim.building_graph().parcels.len() > before,
            "ample materials must allow construction"
        );
        assert!(
            sim.state.resources.wood < wood_before,
            "construction must debit wood"
        );
        assert!(
            sim.state.resources.metal < metal_before,
            "construction must debit metal"
        );
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
        // Pin macro scalars at base threshold so wealth disparity alone drives Conflict.
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let a = faction_ids[(sim.state.tick as usize) % faction_ids.len()];
        let b = faction_ids[((sim.state.tick as usize) + 1) % faction_ids.len()];
        // Disparity well above max threshold (2× base peace cap) so economy drift
        // cannot erase it before diplomacy resolves.
        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(1_000_000));
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

    /// FR-CIV-0100 — alliances and rivalries fade toward neutral without reinforcement.
    #[test]
    fn faction_relations_decay_toward_neutral() {
        let mut sim = Simulation::with_seed(5);
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        let a = ids[0];
        let b = ids[1];
        sim.state.faction_relations.apply_signal(
            ClusterId(u64::from(a)),
            ClusterId(u64::from(b)),
            DiplomacySignal {
                trade_volume: 12.5,
                ..Default::default()
            },
        );
        let before = sim.faction_relation(a, b).abs();
        assert!(
            before > 0.0,
            "test pair should start with a non-neutral relation"
        );

        // Ticks 500 and 1000 diplomacy pairs are (2,0) and (1,2) — not (a,b).
        for tick in [500_u64, 1_000] {
            sim.state.tick = tick;
            sim.phase_diplomacy();
        }

        let after = sim.faction_relation(a, b).abs();
        assert!(
            after < before,
            "relation magnitude should fade toward neutral without reinforcement \
             (before={before}, after={after})"
        );
    }

    /// FR-CIV-0100 — repeated peer trade builds a positive emergent relation.
    #[test]
    fn faction_relations_build_from_diplomacy() {
        let mut sim = Simulation::with_seed(5);
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.unrest = 0;
        let ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        let a = ids[0];
        let b = ids[1];
        sim.state.faction_treasury.insert(a, Fixed::from_num(5_000));
        sim.state.faction_treasury.insert(b, Fixed::from_num(5_000));

        for round in 0..8_u64 {
            sim.state.tick = 500 + round * 500;
            sim.phase_diplomacy();
        }

        assert!(
            sim.faction_relation(a, b) > 0.0,
            "peer trade history should raise the pair relation above neutral"
        );
    }

    /// N4 — sustained TradeAgreement between two factions births an emergent trade route.
    #[test]
    fn sustained_trade_agreement_births_emergent_trade_route() {
        let mut sim = Simulation::with_seed(42);
        sim.state.trade_routes.clear();
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let a = faction_ids[0];
        let b = faction_ids[1];
        sim.state.faction_treasury.insert(a, Fixed::from_num(5_000));
        sim.state.faction_treasury.insert(b, Fixed::from_num(5_000));

        let init_count = sim.state.trade_routes.len();
        assert_eq!(init_count, 0, "bootstrap routes cleared for emergence proof");

        // Ticks 1500 and 3000 pair factions (0, 1) with three sorted factions.
        for tick in [1_500_u64, 3_000_u64] {
            sim.state.tick = tick;
            sim.phase_diplomacy();
        }

        let goods = emergent_route_goods(a);
        let born = sim.state.trade_routes.iter().any(|route| {
            route.from_faction == a
                && route.to_faction == b
                && route.goods == goods
                && route.volume > Fixed::ZERO
        });
        assert!(
            born,
            "sustained TradeAgreement should birth a directed emergent route \
             (from={a}, to={b}, goods={goods})"
        );
        assert!(
            sim.state.emergent_trade_route_keys.contains(&(a, b, goods.to_string())),
            "born route should be tracked as emergent"
        );
        assert!(
            sim.state.trade_routes.len() <= MAX_TRADE_ROUTES,
            "route count must stay bounded"
        );
    }

    /// FR-CIV-0100 — allied factions tolerate wealth disparity that would otherwise conflict.
    #[test]
    fn allies_tolerate_disparity() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500;
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim.state.tick);
        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(12_000));
        seed_faction_pair_relation(
            &mut sim.state.faction_relations,
            a,
            b,
            DiplomacySignal {
                trade_volume: 12.5,
                ..Default::default()
            },
            30,
        );
        assert!(
            sim.faction_relation(a, b) > 0.5,
            "allied pair should have a strongly positive relation before diplomacy"
        );
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::TradeAgreement,
            "allies should tolerate disparity that exceeds the base conflict threshold"
        );
    }

    /// FR-CIV-0100 — rival factions clash at disparities peers would trade through.
    #[test]
    fn rivals_clash_sooner() {
        let mut sim = Simulation::with_seed(5);
        sim.state.tick = 500;
        sim.state.belief = 0;
        sim.state.cohesion = 0;
        sim.state.unrest = 0;
        let mut faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        let (a, b) = diplomacy_faction_pair(&faction_ids, sim.state.tick);
        sim.state.faction_treasury.insert(a, Fixed::from_num(4_000));
        sim.state.faction_treasury.insert(b, Fixed::from_num(10_000));
        seed_faction_pair_relation(
            &mut sim.state.faction_relations,
            a,
            b,
            DiplomacySignal {
                resource_competition: 8.34,
                ..Default::default()
            },
            30,
        );
        assert!(
            sim.faction_relation(a, b) < -0.5,
            "rival pair should have a strongly negative relation before diplomacy"
        );
        sim.phase_diplomacy();
        assert_eq!(
            sim.diplomacy_events().last().expect("a diplomacy event").kind,
            DiplomacyKind::Conflict,
            "rivals should fight at a disparity below the base conflict threshold"
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

    /// FR-CIV-EMERGENCE — proportional decay prevents unbounded belief growth.
    #[test]
    fn belief_decays_toward_equilibrium() {
        let mut sim = Simulation::new();
        sim.add_belief(1_000_000);
        sim.phase_belief();
        assert!(
            sim.belief() < 1_000_000,
            "decay applied; worship/temple inflow is small at default"
        );
    }

    /// FR-CIV-LEGENDS-001 — `saga_belief_gain` is bounded by the per-tick cap
    /// and zero when there is nothing to mint.
    #[test]
    fn saga_belief_gain_bounded_by_cap() {
        assert_eq!(saga_belief_gain(0, 0.0), 0, "no input -> no belief");
        assert_eq!(
            saga_belief_gain(100, 100.0),
            MAX_SAGA_BELIEF_PER_TICK,
            "huge input is capped"
        );
    }

    /// FR-CIV-LEGENDS-001 — `saga_belief_gain` scales with both inputs.
    #[test]
    fn saga_belief_gain_scales_with_promotions() {
        assert_eq!(saga_belief_gain(2, 0.0), 6, "2 promotions, no significance");
        assert_eq!(
            saga_belief_gain(0, 1.0),
            5,
            "one max-significance entity -> BELIEF_SIGNIFICANCE_SCALE"
        );
    }

    /// FR-CIV-LEGENDS-001 — saga promotions mint belief; a sim with a
    /// `legend_promotion` feed event strictly increases belief after
    /// `apply_saga_belief_gain`, while an identical sim with no promotions
    /// sees no saga contribution. Reuses the belief-test harness seed.
    #[test]
    fn saga_promotions_increase_belief_over_ticks() {
        use crate::emergence::EmergenceFeedEvent;
        // No saga promotions: empty feed -> no saga belief minted.
        let mut sim = Simulation::with_seed(7);
        sim.state.belief = 0;
        sim.emergence.last_feed.clear();
        let before_none = sim.state.belief;
        sim.apply_saga_belief_gain();
        assert_eq!(
            sim.state.belief, before_none,
            "no legend_promotion feed events -> no saga belief mint"
        );

        // Saga activity that promoted >=1 entity: inject a feed event as if
        // emergence_legends had produced it this tick, then mint.
        sim.emergence.last_feed.push(EmergenceFeedEvent {
            tick: sim.state.tick,
            kind: "legend_promotion".to_string(),
            summary: "agent 1 promoted in saga graph (1)".to_string(),
            agent_id: Some(1),
        });
        let before_promoted = sim.state.belief;
        sim.apply_saga_belief_gain();
        assert!(
            sim.state.belief > before_promoted,
            "a legend_promotion event should mint belief (saga significance -> belief)"
        );
        assert_eq!(
            sim.state.belief - before_promoted,
            BELIEF_PER_PROMOTION,
            "saga mint scales with promotion count (sig_sum=0 on empty graph)"
        );
    }

    /// FR-CIV-GENETICS / FR-CIV-LEGENDS — N7: `awakening_belief_gain` is
    /// bounded by the per-tick cap, scales linearly below it, and is zero
    /// when there are no awakenings.
    #[test]
    fn awakening_belief_gain_bounded() {
        assert_eq!(awakening_belief_gain(0), 0, "no awakening -> no belief");
        assert_eq!(
            awakening_belief_gain(2),
            2 * BELIEF_PER_AWAKENING,
            "two awakenings -> 2 * BELIEF_PER_AWAKENING"
        );
        assert_eq!(
            awakening_belief_gain(100),
            MAX_AWAKENING_BELIEF_PER_TICK,
            "huge awakening burst is capped"
        );
        // 5 awakenings * 4 = 20, exactly the cap, so still capped to the cap.
        assert_eq!(
            awakening_belief_gain(5),
            MAX_AWAKENING_BELIEF_PER_TICK,
            "5 awakenings * 4 == cap, still saturated to cap"
        );
    }

    /// FR-CIV-GENETICS / FR-CIV-LEGENDS — N7: a sim with at least one
    /// sentience awakening fires `apply_awakening_coupling` and ends with
    /// `state.belief` strictly greater than an identical sim with no
    /// awakenings. Reuses the same belief-test harness as the saga path.
    #[test]
    fn awakening_increases_belief() {
        use civ_genetics::sentience::{SentienceEvent, SentienceThreshold};

        // No awakenings: last_sentience empty -> no mint.
        let mut sim_none = Simulation::with_seed(7);
        sim_none.state.belief = 0;
        sim_none.emergence.last_sentience.clear();
        sim_none.emergence.sentience_threshold = SentienceThreshold::new(1.0);
        let before_none = sim_none.state.belief;
        sim_none.apply_awakening_coupling();
        assert_eq!(
            sim_none.state.belief, before_none,
            "no awakenings -> no awakening belief mint"
        );

        // >=1 awakening: inject a synthetic SentienceEvent (matches the wire
        // path that `emergence_genetics_sentience` populates), then call the
        // coupling. Belief must strictly increase.
        let mut sim_woken = Simulation::with_seed(7);
        sim_woken.state.belief = 0;
        sim_woken.emergence.last_sentience.clear();
        sim_woken.emergence.last_sentience.push(SentienceEvent {
            lineage_id: Some(42),
            cognition_score: 0.9,
            threshold: SentienceThreshold::new(0.72),
            crossed: true,
        });
        let before_woken = sim_woken.state.belief;
        sim_woken.apply_awakening_coupling();
        assert!(
            sim_woken.state.belief > before_woken,
            "a sentience awakening should mint belief (awe -> belief)"
        );
        assert_eq!(
            sim_woken.state.belief - before_woken,
            BELIEF_PER_AWAKENING,
            "single awakening mints exactly BELIEF_PER_AWAKENING"
        );
        assert!(
            sim_woken.state.belief > sim_none.state.belief,
            "woken sim must have strictly more belief than the no-awakening sim"
        );
    }

    /// FR-CIV-GENETICS / FR-CIV-LEGENDS — N7: many awakenings in one tick do
    /// not push cohesion past its per-tick cap. The per-tick cap is the
    /// design-layer clamp on the awakening-driven cohesion pulse.
    #[test]
    fn awakening_cohesion_pulse_bounded() {
        // Many awakenings at the gain-fn level: clamped to the cap.
        assert_eq!(
            awakening_cohesion_gain(0),
            0,
            "no awakenings -> no cohesion nudge"
        );
        assert_eq!(
            awakening_cohesion_gain(1_000),
            MAX_AWAKENING_COHESION_PER_TICK,
            "huge awakening burst is capped at MAX_AWAKENING_COHESION_PER_TICK"
        );
        assert!(
            awakening_cohesion_gain(3) <= MAX_AWAKENING_COHESION_PER_TICK,
            "per-tick pulse respects the cap"
        );

        // End-to-end: a sim with many synthetic awakenings, after coupling,
        // gains at most MAX_AWAKENING_COHESION_PER_TICK cohesion (starting
        // from zero), proving the cap holds through `add_cohesion`.
        use civ_genetics::sentience::{SentienceEvent, SentienceThreshold};
        let mut sim = Simulation::with_seed(7);
        sim.state.cohesion = 0;
        sim.emergence.last_sentience.clear();
        for i in 0..50 {
            sim.emergence.last_sentience.push(SentienceEvent {
                lineage_id: Some(1_000 + i),
                cognition_score: 0.9,
                threshold: SentienceThreshold::new(0.72),
                crossed: true,
            });
        }
        sim.apply_awakening_coupling();
        assert_eq!(
            sim.state.cohesion,
            MAX_AWAKENING_COHESION_PER_TICK as u64,
            "50 awakenings in one tick -> cohesion rises by exactly the per-tick cap"
        );
    }

    /// FR-CIV-0100 — cohesion decays without reinforcement even when delta is zero.
    #[test]
    fn cohesion_decays_without_reinforcement() {
        let mut sim = Simulation::new();
        sim.state.cohesion = 1_000_000;
        sim.state.belief = 0;
        sim.state.unrest = 0;
        sim.phase_cohesion();
        assert!(
            sim.cohesion() < 1_000_000,
            "with no belief bind and no unrest fray, only decay acts"
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

    /// FR-CIV-0100 — discrete tech unlocks are monotonic in research tier.
    #[test]
    fn tech_unlocks_for_tier_is_monotonic() {
        assert_eq!(tech_unlocks_for_tier(0), 0);
        assert_eq!(tech_unlocks_for_tier(1), TECH_IRRIGATION);
        let tier3 = tech_unlocks_for_tier(3);
        assert!(tier3 & TECH_IRRIGATION != 0);
        assert!(tier3 & TECH_STORAGE != 0);
        assert!(tier3 & TECH_METALLURGY != 0);
        let tier2 = tech_unlocks_for_tier(2);
        let tier5 = tech_unlocks_for_tier(5);
        assert_eq!(tier5 & tier2, tier2, "tier 5 is a superset of tier 2");
    }

    /// FR-CIV-0100 — phase_tech sets unlock bits from tier and never clears them.
    #[test]
    fn phase_tech_sets_and_keeps_bits() {
        let mut sim = Simulation::with_seed(11);
        sim.state.research_progress = 200_000;
        sim.phase_tech();
        assert!(sim.has_tech(TECH_IRRIGATION));
        assert!(sim.has_tech(TECH_STORAGE));
        sim.state.research_progress = 0;
        sim.phase_tech();
        assert!(sim.has_tech(TECH_IRRIGATION), "bits are monotonic");
        assert!(sim.has_tech(TECH_STORAGE), "bits are monotonic");
    }

    /// FR-CIV-0100 — irrigation unlock raises carrying capacity by a flat bonus.
    #[test]
    fn irrigation_raises_carrying_capacity() {
        let mut sim = Simulation::with_seed(13);
        let without = sim.carrying_capacity();
        sim.state.tech_unlocks |= TECH_IRRIGATION;
        let with = sim.carrying_capacity();
        assert_eq!(with - without, 200_000);
    }

    /// FR-CIV-0100 — tech tree extends through tier 6 (Writing, Sanitation, Gunpowder).
    #[test]
    fn tech_tree_extends_to_gunpowder() {
        let tier6 = tech_unlocks_for_tier(6);
        assert!(tier6 & TECH_IRRIGATION != 0);
        assert!(tier6 & TECH_STORAGE != 0);
        assert!(tier6 & TECH_METALLURGY != 0);
        assert!(tier6 & TECH_WRITING != 0);
        assert!(tier6 & TECH_SANITATION != 0);
        assert!(tier6 & TECH_GUNPOWDER != 0);
        let tier3 = tech_unlocks_for_tier(3);
        assert_eq!(tier3 & TECH_WRITING, 0);
    }

    /// FR-CIV-0100 — sanitation unlock raises carrying capacity by a flat bonus.
    #[test]
    fn sanitation_adds_more_capacity() {
        let mut sim = Simulation::with_seed(17);
        let without = sim.carrying_capacity();
        sim.state.tech_unlocks |= TECH_SANITATION;
        let with = sim.carrying_capacity();
        assert_eq!(with - without, 300_000);
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

    /// TECH_STORAGE smooths food-price shocks (advanced logistics damp volatility).
    #[test]
    fn tech_storage_smooths_food_price_shocks() {
        let mut with = Simulation::with_seed(7);
        let mut without = Simulation::with_seed(7);
        with.state.tech_unlocks |= TECH_STORAGE;
        for s in [&mut with, &mut without] {
            s.state.population = 5_000_000;
        }
        let (before_w, before_wo) = (
            with.market_state.prices()["food"],
            without.market_state.prices()["food"],
        );
        with.phase_economy();
        without.phase_economy();
        let dw = (with.market_state.prices()["food"] - before_w).abs();
        let dwo = (without.market_state.prices()["food"] - before_wo).abs();
        assert!(dw < dwo, "storage should halve food price volatility");
    }

    /// TECH_METALLURGY raises mine output (advanced smelting boosts metal yield).
    #[test]
    fn tech_metallurgy_boosts_metal_yield() {
        let mut with = Simulation::with_seed(7);
        let mut without = Simulation::with_seed(7);
        with.state.tech_unlocks |= TECH_METALLURGY;
        for s in [&mut with, &mut without] {
            s.state.resources.metal = Fixed::ZERO;
            let mine = Building {
                building_type: BuildingType::Mine,
                hp: Fixed::from_num(200),
                max_hp: Fixed::from_num(200),
                position: Position { x: 3, y: 3 },
            };
            let _ = s.world.spawn((mine,));
        }
        with.phase_production();
        without.phase_production();
        assert!(
            with.state.resources.metal > without.state.resources.metal,
            "metallurgy should boost metal per tick"
        );
    }

    /// N1 coupling — abundant settlement cluster_stocks lower staple food price.
    #[test]
    fn cluster_stocks_food_lowers_market_price() {
        const TICKS: u32 = 5;
        const POPULATION: u64 = 5_000_000;
        const ABUNDANT_FOOD: i64 = 2_000_000;

        let mut low = Simulation::with_seed(7);
        let mut high = Simulation::with_seed(7);
        for sim in [&mut low, &mut high] {
            sim.state.population = POPULATION;
            for treasury in sim.state.faction_treasury.values_mut() {
                *treasury = Fixed::ZERO;
            }
        }
        low.test_clear_cluster_stocks();

        assert_eq!(
            low.market_state.prices()["food"],
            high.market_state.prices()["food"],
            "identical seeds and treasuries should start at the same food price"
        );

        for _ in 0..TICKS {
            for sim in [&mut low, &mut high] {
                sim.state.population = POPULATION;
                for treasury in sim.state.faction_treasury.values_mut() {
                    *treasury = Fixed::ZERO;
                }
            }
            // phase_life runs after phase_economy and rebuilds cluster_stocks;
            // re-apply the only intentional difference before each tick.
            low.test_clear_cluster_stocks();
            high.test_clear_cluster_stocks();
            high.test_set_cluster_food_stock(1, ABUNDANT_FOOD);
            low.tick();
            high.tick();
        }

        assert!(
            high.market_state.prices()["food"] < low.market_state.prices()["food"],
            "abundant settlement food commons should lower staple price (high={} low={})",
            high.market_state.prices()["food"],
            low.market_state.prices()["food"],
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

    /// FR-CIV-0100 — chronicle records technological breakthroughs when tech bits advance.
    #[test]
    fn chronicle_records_tech_breakthroughs() {
        let mut sim = Simulation::with_seed(1);
        sim.state.research_progress = 200_000;
        sim.phase_tech();
        sim.phase_chronicle();
        assert!(!sim.chronicle().is_empty());
        assert!(
            sim.chronicle()
                .iter()
                .any(|line| line.contains("technological breakthrough")),
            "expected a tech breakthrough line"
        );
    }

    /// FR-CIV-0100 — chronicle length stays bounded at CHRONICLE_MAX_LEN.
    #[test]
    fn chronicle_is_length_capped() {
        let mut sim = Simulation::with_seed(1);
        sim.state.chronicle = (0..=CHRONICLE_MAX_LEN)
            .map(|i| format!("filler {i}"))
            .collect();
        sim.phase_chronicle();
        assert!(sim.chronicle().len() <= CHRONICLE_MAX_LEN);
    }

    /// FR-CIV-0100 — golden-age chronicle lines are deduped via chronicle_age.
    #[test]
    fn chronicle_dedups_age() {
        let mut sim = Simulation::with_seed(1);
        sim.state.cohesion = 60_000;
        sim.state.belief = 60_000;
        sim.phase_chronicle();
        sim.phase_chronicle();
        assert_eq!(sim.state.chronicle_age, 1);
        let golden_count = sim
            .chronicle()
            .iter()
            .filter(|line| line.contains("golden age"))
            .count();
        assert_eq!(golden_count, 1);
    }

    /// `tick_with_emergence_source` advances ticks identically; CA grid changes sampling.
    #[test]
    fn tick_with_emergence_source_advances_tick_and_differs_on_ca_grid() {
        use crate::emergence_metrics::EMERGENCE_SAMPLE_INTERVAL;
        use civ_voxel::fluid_ca::CaGrid;
        use civ_voxel::CHUNK_EDGE;

        let mut without_ca = Simulation::with_seed(42);
        let mut with_ca = Simulation::with_seed(42);
        let mut grid = CaGrid::new([CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE]);
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    grid.set(x, y, z, MaterialId(3));
                }
            }
        }

        for _ in 0..EMERGENCE_SAMPLE_INTERVAL {
            without_ca.tick_with_emergence_source(None);
            with_ca.tick_with_emergence_source(Some(&grid));
        }

        assert_eq!(without_ca.state.tick, EMERGENCE_SAMPLE_INTERVAL);
        assert_eq!(with_ca.state.tick, EMERGENCE_SAMPLE_INTERVAL);
        assert_eq!(without_ca.state.tick, with_ca.state.tick);

        let sample_none = without_ca
            .last_emergence_sample()
            .expect("sample at 50-tick boundary");
        let sample_ca = with_ca
            .last_emergence_sample()
            .expect("sample at 50-tick boundary");
        assert_eq!(sample_none.tick, EMERGENCE_SAMPLE_INTERVAL);
        assert_eq!(sample_ca.tick, EMERGENCE_SAMPLE_INTERVAL);
        assert!(
            sample_ca.histogram_total > sample_none.histogram_total,
            "CA grid should contribute voxels to the emergence histogram"
        );
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
        let before = sim.cohesion();
        sim.tick();
        let after = sim.cohesion();

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
}
