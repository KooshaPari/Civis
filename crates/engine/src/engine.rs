//! CivLab Simulation Engine - Core Tick Loop with ECS
//!
//! This module provides the deterministic simulation loop with entity component system.

use civ_agents::{
    cluster::{cluster_by_colocation, MembershipPayoff},
    count_civilians,
    daily_path::{pick_target, DailyPathDecision, Poi, PoiKind, PoiRegistry},
    propagate_tools, propagate_wardrobe, spawn_child_near, spawn_civilian_at, ActorVisualKind,
    Alignment, Civilian as AgentCivilian, ClusterId, ClusterMember, CohortStats, DiplomacyMatrix,
    DiplomacyOutcome, DiplomacySignal, LodTier, Needs, Position3d, Psyche, RelationKind,
    SocialGraph, Tools, Wardrobe,
};
// TODO(cleanup-surgeon): `AgentAction` is no longer re-exported from
// `civ_agents`. Downstream call-sites need to be updated to the new name
// or the type restored upstream.
use civ_agents::culture::{cultural_distance, CultureProfile};
use civ_agents::diplomacy::GriefAccumulator;
// TODO(cleanup-surgeon): `civ-audio` is not in this crate's Cargo.toml — the
//  derive_music_cue/MusicCue/SfxTrigger imports are commented until the dep
//  is restored as a sibling crate.
use civ_audio::triggers::SfxTrigger;
use civ_build::{Allocator, BuildSite, BuildingGraph, DemandSignals, ProductionEvent};
use civ_diffusion::DiffusionParams;

use civ_economy::{
    settlement_trade_flow_from_supply_demand, AllocationEngine, CapitalistAllocator, EconomyState,
    Good, LaborCapacityAllocator, MarketState, ResourceKind, SettlementTradeFlow,
};
// TODO(cleanup-surgeon): `collect_taxes` / `Taxation` were renamed/removed in
//  the civ-economy crate; rewrite the simulation tick's tax phase to the
//  new API.
// use civ_economy::{collect_taxes, Taxation};
use civ_genetics::sentience::{
    cognition_score, evaluate_sentience, CognitionTraitProfile, SentienceEvent, SentienceThreshold,
};

use civ_genetics::Dna;
use civ_genetics::Species;
use civ_mod_host::ModHost;
use civ_needs::{should_reproduce, Health as CivNeedsHealth, LifecycleLabel, LifecycleParams};
use civ_planet::{
    compute_climate, compute_weather, defaults_earthlike, Climate, GeologyMap, MoonConfig,
    PlanetConfig, WeatherCell,
};
use civ_species::evolution::EvolutionEngine;
use civ_species::express;
use civ_tactics::{
    apply_damage, evolve_doctrine, score_doctrine_fitness, tick_operational_movement,
    tick_war_bridge, CombatEngagement, DamageEvent, Doctrine, DoctrineLibrary,
    FactionEngagementStats, MilitaryPhaseConfig, MilitaryUnitSample, NoopOperationalLayer,
    OperationalLayer,
};
use civ_voxel::{
    material::WATER, DirtyChunkEvent, MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE,
};
use hecs::{Entity, World};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::ops::{Deref, DerefMut};

// Re-export types extracted into dedicated subsystem modules so downstream
// code within this crate (and lib.rs re-exports) continue to compile.
pub(crate) use crate::climate::{CoastalColumn, WATER_MARKER_MATERIAL};
pub(crate) use crate::diplomacy::{
    DiplomacyEvent, DiplomacyKind, FactionRelationRecord, FactionRelationSnapshot, FactionRelations,
};
// Economy helpers extracted into `economy_engine` module.
pub(crate) use crate::economy_engine::{economy_state_from_world, market_price_from_balance};
// Re-export types from sub-modules so downstream code (including
// lib.rs re-exports) continues to compile.
pub use self::ai_decision::{
    action_from_emotion_behavior, AgentAction, EmotionDrivenBehavior, PsycheDrivenBehavior,
};
pub use self::species_lifecycle::{
    attach_citizen_to_agents, job_type_for_civilian_id, LifecycleCounters, PopulationEvent,
};
pub(crate) use self::world_simulation::PHASE_ORDER;

// Re-export social types from the extracted social_types module so
// downstream code (including lib.rs re-exports) continues to compile.
use crate::social_types::{compute_gini, institution_kind_key};
pub use crate::social_types::{
    CohesionEvent, CohesionEventKind, CohesionSnapshot, FabricTier, KinshipEdge, KinshipKind,
    MoodSnapshot, SimSeed, StratBand, StratQuantiles, StratificationEvent, StratificationEventKind,
    StratificationReport, UnrestEvent, UnrestLevel, UnrestSnapshot, MOOD_CRIME_BASE,
    MOOD_HISTORY_CAP, MOOD_MAX, MOOD_MIN,
};

use crate::culture::{
    advance_faction_ideologies, culture_cooperation_signal, culture_openness_signal,
    FactionIdeologyState,
};
pub use crate::fixed_math::{Fixed, FixedFromNum};
// TODO(cleanup-surgeon): `language`, `psyche_behavior`, `religion` modules
//  are currently empty `pub mod` stubs. These imports are commented until
//  the real implementations are restored or the call-sites are rewritten.
use crate::language::{
    borrow_word, ensure_seeded_word, faction_isolation_pressure, person_name, person_name_meaning,
    place_name, place_name_meaning, seeded_language_state, tick_language_for_lineage,
};

use crate::lod::{should_tick_entity_with_policy, LodPolicy};
use crate::policy::ControlSignals;
use crate::policy::Policy;
use crate::policy::PolicyInput;
use crate::policy::DEFAULT_ECONOMY_POLICY;
use crate::religion::{
    apply_big_gods_response, last_religion_sample, substrate_gradients_for, ReligiousProfile,
    SubstrateGradients, MAX_MISERY_UNREST,
};
use crate::replay::{ReplayError, ReplayLog};
use crate::replay_format::{load_civreplay, save_civreplay};
use crate::tutorial::TutorialProgress;

use crate::conditions::GameOutcome;

pub mod ai_decision;
pub mod culture_phases;
pub mod military_phases;
pub mod policy_econ_phases;
pub mod social_settlement_phases;
pub mod species_lifecycle;
pub mod world_phases;
pub mod world_simulation;
pub(crate) use self::military_phases::default_faction_doctrines;
pub(crate) use self::species_lifecycle::{spawn_faction_civilians, spawn_faction_civilians_custom};
pub use self::world_phases::derive_music_cue;
pub mod compat_state;
pub use self::compat_state::{
    add_cohesion, add_trust, faction_count, last_tick_cohesion, last_tick_cohesion_settlement,
    last_tick_unrest, last_tick_unrest_settlement, set_settlement_gini, settlement_gini,
    unrest_level,
};

// --- Local stubs for removed upstream types ----------------------------------
// Moved to ai_decision.rs
// Moved to species_lifecycle.rs
// Moved to world_simulation.rs
// TODO(cleanup-surgeon): re-add stubs (16 fns + types) for D1 compile gate ----
//
// The following symbols are forward-declared placeholders so the engine
// compiles while the real implementations are restored in follow-up lanes.
// Each stub returns a safe default (0, false, empty Vec, etc.) and the body
// will be replaced when the upstream crate surfaces the real signature.

/// Stub `WorldgenConfig` for the simulation's worldgen field.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldgenConfig {
    pub seed: u64,
}

/// Per-cluster emergent music cue parameters (FR-AUDIO-wire).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MusicCue {
    /// Coarse mood tag the client renderer maps to a stem.
    pub mood: String,
    /// Loudness/intensity scalar 0..1.
    pub intensity: f32,
    /// Optional secondary tempo hint in BPM.
    pub tempo_bpm: Option<u16>,
}

/// Per-faction emergent language state (FR-LANGUAGE-001).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LanguageState {
    /// Centroid signature the language was last seeded from.
    pub seed_signature: [f32; 4],
    /// Drift rate per tick (deterministic).
    pub drift_rate: f32,
    /// Threshold at which the lineage splits.
    pub split_threshold: f32,
    /// Accumulated phoneme/lexeme inventory (deterministic; stubbed as empty).
    pub lexemes: Vec<String>,
}

/// Sentience-evaluation minimum cognition threshold (FR-CIV-GENETICS).
pub const SENTIENCE_MIN_COGNITION: f32 = 0.5;

/// Stub `to_faction` extractor for [`crate::engine::TradeRoute`]. Replaces
/// a previous `to_faction` method that lived on a wrapper type; mirrors the
/// field directly.
#[inline]
pub fn to_faction<T: Copy>(a: T, _b: T) -> T {
    a
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchCache {
    pub researched: Vec<String>,
    #[serde(default)]
    pub queued: VecDeque<String>,
    #[serde(default)]
    pub in_progress: Option<(String, u64)>,
}

/// Per-cluster stockpiles keyed by emergent settlement id.
pub type ClusterStocks = civ_economy::Stocks;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct MembershipPayoffTotals {
    pub cluster_id: u64,
    pub members: u32,
    pub total_payoff: f32,
}

// Moved to world_simulation.rs
/// Broad economic orientation inferred from a civilization's strongest signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomicFocus {
    Balanced,
    Agrarian,
    Industrial,
    Sacred,
    Mercantile,
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

// Moved to species_lifecycle.rs
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

// Moved to species_lifecycle.rs
/// A per-settlement event emitted when the expected economic focus changes
/// (FR-CIV-ECON-001 / ADR-020). Carries the settlement id, the previous and
/// proposed focus, and a human-readable cause so downstream phases and the
/// JSON-RPC bridge can attribute the transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EconomicFocusEvent {
    pub settlement_id: u32,
    pub from: EconomicFocus,
    pub to: EconomicFocus,
    pub cause: String,
}

// Moved to species_lifecycle.rs
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Soldier,
    Archer,
    Knight,
    Scout,
}

/// ECS military unit component used by spawn helpers and JSON-RPC pin export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilitaryUnit {
    /// Broad unit archetype.
    pub unit_type: UnitType,
    /// Current combat strength.
    pub strength: Fixed,
    /// Current hit points.
    pub hp: Fixed,
    /// Maximum hit points.
    pub max_hp: Fixed,
    /// Morale in fixed-point units.
    pub morale: Fixed,
    /// World position on the hex grid.
    pub position: Position,
    /// Owning faction id.
    pub faction_id: u32,
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
    #[serde(default)]
    pub last_tick_unrest_snapshots: BTreeMap<u32, UnrestSnapshot>,
    #[serde(default)]
    pub last_tick_cohesion: BTreeMap<u32, CohesionSnapshot>,
    /// Factions that chose [`crate::faction_decisions::FactionDecision::RaiseUnrestResponse`]
    /// during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_unrest_response_intents: BTreeSet<u32>,
    /// Factions that chose [`crate::faction_decisions::FactionDecision::FlagHostility`]
    /// during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_hostility_intents: BTreeSet<u32>,
    /// Factions that chose [`crate::faction_decisions::FactionDecision::FlagTradeOpen`]
    /// during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_trade_open_intents: BTreeSet<u32>,
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
            last_tick_unrest_snapshots: BTreeMap::new(),
            last_tick_cohesion: BTreeMap::new(),
            last_tick_faction_unrest_response_intents: BTreeSet::new(),
            last_tick_faction_hostility_intents: BTreeSet::new(),
            last_tick_faction_trade_open_intents: BTreeSet::new(),
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
    pub(crate) rng: SimRng,
    pub(crate) planet: PlanetConfig,
    pub(crate) moon: MoonConfig,
    worldgen: WorldgenConfig,
    pub climate: Climate,
    pub current_tick: u64,
    pending_damage: Vec<DamageEvent>,
    tick_modulo_compact: u64,
    building_graph: BuildingGraph,
    allocator: Allocator,
    diffusion_params: DiffusionParams,
    target_era: u16,
    last_cohort_stats: Option<CohortStats>,
    pub(crate) last_births: Vec<PopulationEvent>,
    pub(crate) last_deaths: Vec<PopulationEvent>,
    pub last_life_deaths: u32,
    /// Per-tick lifecycle rollup (FR-CIV-LIFE P4-A). Updated by phase_life;
    /// read by phase_economy to derive aggregate labor fraction.
    pub last_tick_lifecycle_metrics: LifecycleCounters,
    pub(crate) diplomacy_events: Vec<DiplomacyEvent>,
    pub(crate) next_civilian_id: u64,
    /// Settlement cluster ids from the most recent life rollup (FR-CIV-LIFE-030).
    /// Stored as a deterministic `Vec<u64>` so the HUD roster and JSON-RPC
    /// bridge can read it without re-deriving from the world.
    last_settlement_ids: Vec<u64>,
    research_cache: ResearchCache,
    /// Number of researched entries already translated into Tech audio triggers.
    ///
    /// `ResearchCache::researched` is append-only in normal operation. Keeping
    /// the watermark outside the serialized cache prevents existing research
    /// from replaying as fresh audio after construction or replay load.
    last_audio_researched_len: usize,
    /// Per-faction emergent era/tech progression (FR-ERA).
    pub(crate) era_progression: crate::era::EraProgressionState,
    /// Per-faction relation matrix (FR-CIV-DIPLOMACY).
    /// Stub: an empty [`FactionRelations`] until DiplomacyMatrix schema is
    /// finalized and the matrix methods replace the field-level accessors.
    pub faction_relations: FactionRelations,
    /// Per-tick grief accumulator for casualty → mourning coupling
    /// (FR-CIV-PSYCHE-911). Stub: zero-valued; full impl tracks faction losses.
    pub grief_accumulator: civ_agents::diplomacy::GriefAccumulator,
    /// Scenario-level taxation policy (FR-CIV-ECON-010).
    /// Stub default: zeroes; full impl wires `civ_economy::Taxation` settings.
    pub scenario_taxation: civ_economy::Taxation,
    belief: u64,
    /// Per-cluster culture profiles (cluster_cultures key is the cluster id).
    pub cluster_cultures: BTreeMap<u64, CultureProfile>,
    /// Per-faction cultural identity, norms, and behavior signals.
    pub faction_ideologies: BTreeMap<u32, FactionIdeologyState>,
    cluster_stocks: BTreeMap<u64, ClusterStocks>,
    /// Last-tick settlement trade flows derived during `phase_economy`.
    pub(crate) last_tick_settlement_trade_flows: Vec<SettlementTradeFlow>,
    pub last_settlement_count: u32,
    /// Per-faction aggression (u32 faction id → average aggression).
    pub faction_aggression: std::collections::BTreeMap<u32, f32>,
    /// MOAT emergence state — culture/legends/feed buffers.
    pub emergence: crate::emergence::EmergenceState,
    /// Tutorial onboarding progression surfaced to clients.
    pub tutorial_progress: TutorialProgress,
    /// Live emergence-branching state (rolling σ̄_W).
    pub emergence_branching: crate::emergence_metrics::EmergenceBranchingState,
    /// Most recent emergence sample (None before first sample).
    pub emergence_sample: Option<crate::emergence_metrics::EmergenceSample>,
    /// 3D voxel substrate (Civis 3D extension). Hosts terrain + destructible
    /// structures + tactical combat impacts. Drained per tick by
    /// [`Simulation::phase_voxel`].
    pub voxel: VoxelWorld<MaterialId>,
    /// Voxel dirty events produced during the most recent tick. Consumers
    /// (renderer protocol bridge, replay log) read this each tick; it resets
    /// at the start of every [`Simulation::tick`].
    last_tick_voxel_events: Vec<DirtyChunkEvent>,
    last_tick_voxel_damage_count: usize,
    /// Per-soldier damage pulses from the most recent tactics phase (FR-CIV-TACTICS-024).
    last_tick_combat_pulses: Vec<CombatDamagePulse>,
    /// Disasters resolved this tick (FR-CIV-LEGENDS ingest).
    pub(crate) last_tick_disaster_pulses: Vec<crate::disasters::DisasterPulse>,
    /// Engagements resolved this tick (war bridge); feeds doctrine fitness.
    pub(crate) last_tick_engagements: Vec<CombatEngagement>,
    /// `mod.loaded.v1` replay-bus JSON emitted when mods load (cleared each tick).
    last_tick_mod_lifecycle: Vec<String>,
    /// Audio events derived from substrate signals on the most recent tick
    /// (FR-AUDIO-wire). Reset at the start of every [`Simulation::tick`];
    /// populated by [`Simulation::phase_audio`] from combat pulses,
    /// construction events, and emergent disasters. Consumed by the
    /// JSON-RPC bridge (`sim.snapshot.audio_events`) and the WebSocket
    /// tick broadcast; clients translate each entry into a kira SFX
    /// trigger via [`civ_audio::triggers::trigger_to_sfx_requests`].
    last_tick_audio_events: Vec<civ_audio::triggers::SfxTrigger>,
    /// Last-tick decisions for the daily-path phase (FR-CIV-LIFE-010..016).
    pub last_tick_daily_path: Vec<DailyPathDecision>,
    /// Per-cluster payoffs emitted by the cluster phase (FR-CIV-LIFE-030..035).
    pub last_tick_cluster_payoffs: Vec<MembershipPayoffTotals>,
    /// Per-culture music cue parameters derived from emergent culture + state.
    /// Updated in `phase_audio` and surfaced on `SimulationSnapshot::music_cues`
    /// as a stable per-cluster key-value map.
    last_tick_music_cues: BTreeMap<u64, MusicCue>,
    /// Per-tick disaster events surfaced in snapshots.
    pub(crate) last_tick_disaster_events: Vec<crate::disasters::DisasterTickEvent>,
    /// Most recent deterministic victory/defeat assessment.
    pub last_game_outcome: GameOutcome,

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
    pub(crate) mod_host: ModHost,
    /// Military-phase cadence and per-tick movement pulses (FR-CIV-TACTICS-035).
    pub(crate) military_phase: MilitaryPhaseConfig,
    /// Per-faction doctrine libraries evolved on a fixed tick cadence (FR-CIV-TACTICS-010).
    faction_doctrines: Vec<DoctrineLibrary>,
    /// Coastal water columns whose water-level voxel shifts with the tide
    /// offset every tick (FR-CIV-PLANET-020). Keyed by `(x, z)` in fixed-point
    /// world coords; iteration order is deterministic.
    pub(crate) coastal_columns: BTreeMap<(i64, i64), CoastalColumn>,
    /// Per-region weather grid updated by `phase_planet` each tick (FR-CIV-PLANET-030).
    pub weather_grid: Vec<WeatherCell>,
    /// Construction queue of in-progress `BuildSite`s.
    /// Drives `phase_buildings` per-tick progress + completion (FR-CIV-BUILD-001/002).
    build_sites: Vec<BuildSite>,
    /// Construction events emitted during the most recent tick (FR-CIV-BUILD-002).
    /// Reset at the start of every [`Simulation::tick`]; surfaced through the
    /// JSON-RPC bridge so Bevy clients can render scaffolding + completion FX.
    last_tick_construction_events: Vec<ProductionEvent>,
    /// Emergent language state (FR-CIV-LANG-001). Driven by
    /// [`Simulation::phase_language`]; consumed by the diplomacy pipeline via
    /// [`language_intelligibility_peace_bonus`].
    language_state: LanguageState,
    /// Per-faction emergent language states (FR-LANGUAGE-001) used for naming
    /// and isolation-aware drift coupling.
    faction_languages: BTreeMap<u32, LanguageState>,
    /// Per-tick sentience evaluation profile (FR-CIV-GENETICS / FR-CIV-LEGENDS).
    /// Read by [`Simulation::phase_sentience`] to determine which lineages
    /// cross the cognition threshold this tick.
    sentience_profile: CognitionTraitProfile,
    /// Per-tick sentience threshold (FR-CIV-GENETICS / FR-CIV-LEGENDS). Read by
    /// [`Simulation::phase_sentience`]; mirrors the profile in
    /// `EmergenceState::new`.
    sentience_threshold: SentienceThreshold,
    /// Sentience events produced by the most recent [`Simulation::phase_sentience`]
    /// call (cleared at the start of every [`Simulation::tick`], alongside the
    /// other `last_tick_*` buffers).
    last_tick_sentience_events: Vec<SentienceEvent>,
    /// Per-settlement population snapshot, settable by tests + scenario loaders
    /// so `phase_institutions` can drive Temple/Garrison spawns deterministically
    /// (FR-CIV-GOV-001). Keyed by settlement id (`u32`).
    pub(crate) settlements: BTreeMap<u32, u32>,
    /// Currently-active institutions per settlement, keyed by
    /// `(settlement_id, kind)`. Tracks the latest known level so we can detect
    /// upgrades (FR-CIV-GOV-003).
    institutions: BTreeMap<u32, civ_institutions::Institution>,
    /// Civic events emitted by the most recent [`Simulation::phase_institutions`]
    /// call (cleared at the start of every [`Simulation::tick`], alongside the
    /// other `last_tick_*` buffers). Surfaced to the JSON-RPC bridge so the
    /// Bevy client can render the civil layer.
    last_tick_institution_events: Vec<InstitutionEvent>,
    /// Monotonic set of `(settlement_id, kind, level)` we have already emitted
    /// as an `Upgraded` event. Guarantees one-shot upgrade emission even
    /// across population dips/rebounds (FR-CIV-GOV-003).
    institution_levels_emitted: BTreeSet<(u32, u8, u8)>,

    /// Per-settlement food stock, settable by tests + scenario loaders so
    /// [`Simulation::phase_social_mood`] can derive `food_score` deterministically
    /// (FR-CIV-GOV-100). Keyed by settlement id (`u32`); missing keys default
    /// to `0` in the phase.
    pub(crate) settlement_food_stocked: BTreeMap<u32, i64>,
    /// Per-settlement housing capacity, settable by tests + scenario loaders
    /// so [`Simulation::phase_social_mood`] can compute `housing_score` as
    /// `2 * (capacity - population)` (FR-CIV-GOV-100). Keyed by settlement id
    /// (`u32`); missing keys default to `0` in the phase.
    settlement_housing_capacity: BTreeMap<u32, u32>,
    /// Per-settlement crime pressure, settable by tests + scenario loaders so
    /// [`Simulation::phase_social_mood`] can compute `crime_score`
    /// (FR-CIV-GOV-100). Keyed by settlement id (`u32`); missing keys default
    /// to `0` in the phase. Treated as `i32` so the `4 * pressure` term
    /// saturates cleanly in `i64` arithmetic.
    settlement_crime_pressure: BTreeMap<u32, i32>,
    /// Flat mood history ring (test convenience). At most
    /// [`MOOD_HISTORY_CAP`] * 8 entries are kept; older entries are drained
    /// from the front. Mirrors the per-settlement ring in
    /// `mood_history_by_settlement` for assertion convenience.
    mood_history: Vec<MoodSnapshot>,
    /// Per-settlement mood history ring. Each entry retains at most
    /// [`MOOD_HISTORY_CAP`] [`MoodSnapshot`]s in append order (oldest first).
    mood_history_by_settlement: BTreeMap<u32, Vec<MoodSnapshot>>,
    /// Per-settlement mood snapshot emitted on the most recent tick
    /// (cleared at the start of every [`Simulation::tick`], alongside the
    /// other `last_tick_*` buffers). Surfaced to the JSON-RPC bridge so
    /// clients can render the social-mood layer.
    last_tick_mood: Vec<MoodSnapshot>,

    households: BTreeMap<u64, ()>,
    household_settlement: BTreeMap<u64, u32>,
    settlement_households: BTreeMap<u32, BTreeSet<u64>>,
    household_wealth: BTreeMap<u64, i64>,
    household_power: BTreeMap<u64, i64>,
    household_bands: BTreeMap<u64, StratBand>,
    household_score: BTreeMap<u64, i64>,
    household_band_set: BTreeSet<(u32, u64, StratBand)>,
    stratification_bands_emitted: BTreeSet<(u32, u64, StratBand)>,
    last_tick_stratification: Vec<StratificationEvent>,
    last_tick_stratification_reports: BTreeMap<u32, StratificationReport>,

    /// Per-settlement religious profile keyed by settlement id.
    /// Populated by [`Simulation::phase_belief`] (FR-CIV-REL-001 §7).
    /// One entry per settlement that has been observed by the belief phase;
    /// settlements outside the religion design radius are not stored.
    pub religious_profiles: BTreeMap<u32, ReligiousProfile>,

    /// Per-tick buffer of religion events emitted by `phase_belief`. Cleared
    /// at the top of each `tick()` (mirrors the `last_tick_mood` /
    /// `last_tick_diffusion_events` pattern). Consumers (e.g. the
    /// JSON-RPC `sim.snapshot.religion` method) read this buffer once per
    /// tick.
    pub last_tick_religion_events: Vec<crate::religion::ReligionEvent>,

    // ── Phase A4: Cohesion (FR-CIV-GOV-030) ──────────────────────────────
    /// Per-actor settlement assignment used by `phase_cohesion` to group
    /// actors by settlement for fabric computation.
    /// Inserted via [`Simulation::set_settlement_actor`].
    actor_settlement: BTreeMap<u64, u32>,

    /// Per-actor hardship level (0..1000 scale). Inserted via
    /// [`Simulation::set_actor_in_settlement_hardship`]; consumed by
    /// `phase_cohesion` as a fabric-eroding input.
    actor_hardship: BTreeMap<u64, i64>,

    /// Per-actor institution presence tuple `(has_temple, has_garrison)`.
    /// Inserted via [`Simulation::set_actor_in_settlement_institutions`].
    actor_institutions: BTreeMap<u64, (bool, bool)>,

    /// Directed kinship edges indexed by actor id. Inserted via
    /// [`Simulation::register_kinship`].
    kinship: BTreeMap<u64, Vec<KinshipEdge>>,

    /// Weighted directed trust network. `trust[a][b] = amount` means actor
    /// `a` trusts actor `b` by `amount`. Inserted via [`Simulation::add_trust`].
    trust: BTreeMap<u64, BTreeMap<u64, i64>>,

    /// Last per-tick fabric score for each actor, used by `phase_cohesion`
    /// to detect delta and emit [`CohesionEvent`]s.
    last_actor_fabric: BTreeMap<u64, i64>,

    settlement_actors: BTreeMap<u32, BTreeSet<u64>>,

    /// Per-tick buffer of [`CohesionEvent`]s emitted by `phase_cohesion`.
    last_tick_cohesion_events: Vec<CohesionEvent>,

    /// Per-settlement snapshot emitted by `phase_cohesion` on the most
    /// recent tick. Surfaced via [`Simulation::last_tick_cohesion`].
    last_tick_cohesion: BTreeMap<u32, CohesionSnapshot>,
    last_tick_cohesion_snapshots: BTreeMap<u32, CohesionSnapshot>,

    // ── Phase A5: Unrest (FR-CIV-UNREST-001) ─────────────────────────────
    /// Per-settlement Gini coefficient, set externally by the simulation driver
    /// (typically derived from the `last_tick_stratification_reports` map) and
    /// consulted by [`Simulation::phase_unrest`] to amplify unrest when inequality
    /// is high.
    pub unrest_settlement_gini: BTreeMap<u32, f64>,

    /// Per-tick buffer of unrest events emitted by `phase_unrest`.
    pub last_tick_unrest: Vec<UnrestEvent>,
    last_tick_unrest_events: Vec<UnrestEvent>,
    last_tick_unrest_levels: BTreeMap<u32, UnrestLevel>,
    settlement_gini: BTreeMap<u32, i32>,

    /// Per-settlement unrest snapshot keyed by settlement id. Populated by
    /// `phase_unrest` whenever a settlement's level changes.
    pub last_tick_unrest_snapshots: BTreeMap<u32, UnrestSnapshot>,

    /// Per-settlement last unrest level, used to compute `level_delta` in the
    /// event stream. Defaults to [`UnrestLevel::Calm`] for unseen settlements.
    pub last_unrest_level: BTreeMap<u32, UnrestLevel>,

    /// Per-settlement riot accumulator used by `phase_unrest`.
    pub riot_accumulator: BTreeMap<u32, i64>,

    /// Per-settlement migrant accumulator used by `phase_unrest`.
    pub migrant_accumulator: BTreeMap<u32, i64>,

    // ── Phase A10/A11: Economic Focus (FR-CIV-ECON-001) ───────────────────
    /// Current economic focus per settlement.
    /// Populated by [`Simulation::phase_economic_focus`] each tick.
    /// Defaults to [`EconomicFocus::Balanced`] for unseen settlements.
    econ_focus: BTreeMap<u32, EconomicFocus>,

    /// Per-tick buffer of [`EconomicFocusEvent`]s emitted by
    /// [`Simulation::phase_economic_focus_pre`]. Cleared at the start of
    /// every [`Simulation::tick`]; surfaced to the JSON-RPC bridge.
    econ_focus_stability: Vec<EconomicFocusEvent>,
}

/// Per-settlement religious event emitted by [`Simulation::phase_belief`]
/// (FR-CIV-REL-001 §7 + §10 hooks). Consumed by the JSON-RPC bridge in
/// `crates/server/src/ws_bridge.rs` to surface the religion layer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReligionEvent {
    /// The settlement id this event pertains to.
    pub settlement_id: u32,
    /// Which cap was hit (or None if this is a regular profile update).
    pub kind: ReligionEventKind,
    /// The tick on which this event was emitted.
    pub tick: u64,
}

/// Distinguishes the kinds of religion events the `phase_belief` loop can
/// emit. JSON-RPC consumers use this to decide whether to surface a UI
/// notification (caps hit) or just update the profile (regular).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReligionEventKind {
    /// Regular per-tick profile update (no cap hit, no regime change).
    TickUpdate,
    /// `monitoring` deltas hit [`crate::religion::MAX_D_MONITORING_PER_TICK`].
    MonitoringCapped,
    /// `mythic_coherence` deltas hit [`crate::religion::MAX_D_COHERENCE_PER_TICK`].
    CoherenceCapped,
    /// `uncertainty_reduction` deltas hit [`crate::religion::MAX_D_UNCERT_REDUCTION_TICK`].
    UncertaintyCapped,
    /// The profile crossed the Norenzayan Big-Gods threshold upward.
    BigGodsEmerged,
    /// The profile collapsed below the dissolution threshold.
    Dissolved,
}

/// Civic institution event emitted by [`Simulation::phase_institutions`]
/// (FR-CIV-GOV-001/002/003). Consumed by the JSON-RPC bridge in
/// `crates/server/src/ws_bridge.rs` to surface the civil layer to clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstitutionEvent {
    /// The kind of institution that changed.
    pub kind: civ_institutions::InstitutionKind,
    /// The new level (1 = L1 / first spawn, 2 = L2 / first upgrade, ...).
    pub level: u8,
    /// The settlement id this event pertains to.
    pub settlement_id: u32,
}

// Social types (MoodSnapshot, StratBand, KinshipEdge, UnrestLevel, etc.)
// extracted to `social_types` module. Re-exported above via pub use crate::social_types::*.

/// Alias for [`Simulation`] so civic-tests can use the shorter name.
pub type Sim = Simulation;

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
        let civilian_count = count_civilians(&world) as u64;
        let initial_lifecycle = LifecycleCounters {
            children: 0,
            adults: civilian_count as u32,
            elders: 0,
            dead: 0,
        };
        let state = WorldState::default();

        Self {
            current_tick: 0,
            economy_state: economy_state_from_world(&state),
            market_state: MarketState::default(),
            state,
            world,
            worldgen: WorldgenConfig::default(),
            last_settlement_ids: Vec::new(),
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
            last_life_deaths: 0,
            last_tick_lifecycle_metrics: initial_lifecycle,
            econ_focus: BTreeMap::new(),
            econ_focus_stability: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache::default(),
            last_audio_researched_len: 0,
            era_progression: crate::era::EraProgressionState::default(),
            faction_relations: FactionRelations::default(),
            grief_accumulator: GriefAccumulator::default(),
            scenario_taxation: civ_economy::Taxation::default(),
            belief: 0,
            cluster_cultures: BTreeMap::new(),
            faction_ideologies: BTreeMap::new(),
            cluster_stocks: BTreeMap::new(),
            last_tick_settlement_trade_flows: Vec::new(),
            last_settlement_count: 0,
            faction_aggression: BTreeMap::new(),
            emergence: crate::emergence::EmergenceState::new(42),
            tutorial_progress: TutorialProgress::default(),
            emergence_branching: crate::emergence_metrics::EmergenceBranchingState::default(),
            emergence_sample: None,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_disaster_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            last_tick_mod_lifecycle: Vec::new(),
            last_tick_audio_events: Vec::new(),
            last_tick_daily_path: Vec::new(),
            last_tick_cluster_payoffs: Vec::new(),
            last_tick_music_cues: BTreeMap::new(),
            last_tick_disaster_events: Vec::new(),
            last_game_outcome: GameOutcome::Ongoing,
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
            build_sites: Vec::new(),
            last_tick_construction_events: Vec::new(),
            language_state: LanguageState::default(),
            faction_languages: BTreeMap::new(),
            sentience_profile: default_sentience_profile(),
            sentience_threshold: SentienceThreshold::new(SENTIENCE_MIN_COGNITION),
            last_tick_sentience_events: Vec::new(),
            settlements: BTreeMap::new(),
            institutions: BTreeMap::new(),
            last_tick_institution_events: Vec::new(),
            institution_levels_emitted: BTreeSet::new(),
            settlement_food_stocked: BTreeMap::new(),
            settlement_housing_capacity: BTreeMap::new(),
            settlement_crime_pressure: BTreeMap::new(),
            mood_history: Vec::new(),
            mood_history_by_settlement: BTreeMap::new(),
            last_tick_mood: Vec::new(),
            households: BTreeMap::new(),
            household_settlement: BTreeMap::new(),
            settlement_households: BTreeMap::new(),
            household_wealth: BTreeMap::new(),
            household_power: BTreeMap::new(),
            household_bands: BTreeMap::new(),
            household_score: BTreeMap::new(),
            household_band_set: BTreeSet::new(),
            stratification_bands_emitted: BTreeSet::new(),
            last_tick_stratification: Vec::new(),
            last_tick_stratification_reports: BTreeMap::new(),
            religious_profiles: BTreeMap::new(),
            last_tick_religion_events: Vec::new(),
            unrest_settlement_gini: BTreeMap::new(),
            last_tick_unrest: Vec::new(),
            last_tick_unrest_snapshots: BTreeMap::new(),
            last_unrest_level: BTreeMap::new(),
            riot_accumulator: BTreeMap::new(),
            migrant_accumulator: BTreeMap::new(),
            actor_settlement: BTreeMap::new(),
            actor_hardship: BTreeMap::new(),
            actor_institutions: BTreeMap::new(),
            kinship: BTreeMap::new(),
            trust: BTreeMap::new(),
            last_actor_fabric: BTreeMap::new(),
            settlement_actors: BTreeMap::new(),
            last_tick_cohesion_events: Vec::new(),
            last_tick_cohesion: BTreeMap::new(),
            last_tick_cohesion_snapshots: BTreeMap::new(),
            settlement_gini: BTreeMap::new(),
            last_tick_unrest_events: Vec::new(),
            last_tick_unrest_levels: BTreeMap::new(),
        }
    }

    /// Create simulation with custom seed (accepts SimSeed wrapper or u64)
    pub fn with_seed(seed: impl Into<SimSeed>) -> Self {
        Self::with_seed_internal(seed.into().0)
    }

    /// Internal seed method that takes raw u64
    fn with_seed_internal(seed: u64) -> Self {
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

        // Pre-seed lifecycle metrics so phase_economy's labor_fraction
        // is non-zero on the first tick (phase_life runs after phase_economy
        // in PHASE_ORDER, so the metrics wouldn't be available otherwise).
        let initial_lifecycle = LifecycleCounters {
            children: 0,
            adults: civilian_count as u32,
            elders: 0,
            dead: 0,
        };

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
            worldgen: WorldgenConfig::default(),
            last_settlement_ids: Vec::new(),
            last_tick_disaster_pulses: Vec::new(),
            climate,
            current_tick: 0,
            pending_damage: Vec::new(),
            tick_modulo_compact: 64,
            building_graph: BuildingGraph::new(),
            allocator: Allocator::new(seed),
            diffusion_params: DiffusionParams::default(),
            target_era: 1,
            last_cohort_stats: None,
            last_births: Vec::new(),
            last_deaths: Vec::new(),
            last_life_deaths: 0,
            last_tick_lifecycle_metrics: initial_lifecycle,
            econ_focus: BTreeMap::new(),
            econ_focus_stability: Vec::new(),
            diplomacy_events: Vec::new(),
            next_civilian_id: 1_000_000,
            research_cache: ResearchCache::default(),
            last_audio_researched_len: 0,
            era_progression: crate::era::EraProgressionState::default(),
            faction_relations: FactionRelations::default(),
            grief_accumulator: GriefAccumulator::default(),
            scenario_taxation: civ_economy::Taxation::default(),
            belief: 0,
            cluster_cultures: BTreeMap::new(),
            faction_ideologies: BTreeMap::new(),
            cluster_stocks: BTreeMap::new(),
            last_tick_settlement_trade_flows: Vec::new(),
            last_settlement_count: 0,
            faction_aggression: BTreeMap::new(),
            emergence: crate::emergence::EmergenceState::new(seed),
            tutorial_progress: TutorialProgress::default(),
            emergence_branching: crate::emergence_metrics::EmergenceBranchingState::default(),
            emergence_sample: None,
            voxel: VoxelWorld::new(FIXED_SCALE),
            last_tick_voxel_events: Vec::new(),
            last_tick_voxel_damage_count: 0,
            last_tick_combat_pulses: Vec::new(),
            last_tick_engagements: Vec::new(),
            last_tick_mod_lifecycle: Vec::new(),
            last_tick_audio_events: Vec::new(),
            last_tick_daily_path: Vec::new(),
            last_tick_cluster_payoffs: Vec::new(),
            last_tick_music_cues: BTreeMap::new(),
            last_tick_disaster_events: Vec::new(),
            last_game_outcome: GameOutcome::Ongoing,
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
            build_sites: Vec::new(),
            last_tick_construction_events: Vec::new(),
            language_state: LanguageState::default(),
            faction_languages: BTreeMap::new(),
            sentience_profile: default_sentience_profile(),
            sentience_threshold: SentienceThreshold::new(SENTIENCE_MIN_COGNITION),
            last_tick_sentience_events: Vec::new(),
            settlements: BTreeMap::new(),
            institutions: BTreeMap::new(),
            last_tick_institution_events: Vec::new(),
            institution_levels_emitted: BTreeSet::new(),
            settlement_food_stocked: BTreeMap::new(),
            settlement_housing_capacity: BTreeMap::new(),
            settlement_crime_pressure: BTreeMap::new(),
            mood_history: Vec::new(),
            mood_history_by_settlement: BTreeMap::new(),
            last_tick_mood: Vec::new(),
            households: BTreeMap::new(),
            household_settlement: BTreeMap::new(),
            settlement_households: BTreeMap::new(),
            household_wealth: BTreeMap::new(),
            household_power: BTreeMap::new(),
            household_bands: BTreeMap::new(),
            household_score: BTreeMap::new(),
            household_band_set: BTreeSet::new(),
            stratification_bands_emitted: BTreeSet::new(),
            last_tick_stratification: Vec::new(),
            last_tick_stratification_reports: BTreeMap::new(),
            religious_profiles: BTreeMap::new(),
            last_tick_religion_events: Vec::new(),
            unrest_settlement_gini: BTreeMap::new(),
            last_tick_unrest: Vec::new(),
            last_tick_unrest_snapshots: BTreeMap::new(),
            last_unrest_level: BTreeMap::new(),
            riot_accumulator: BTreeMap::new(),
            migrant_accumulator: BTreeMap::new(),
            actor_settlement: BTreeMap::new(),
            actor_hardship: BTreeMap::new(),
            actor_institutions: BTreeMap::new(),
            kinship: BTreeMap::new(),
            trust: BTreeMap::new(),
            last_actor_fabric: BTreeMap::new(),
            settlement_actors: BTreeMap::new(),
            last_tick_cohesion_events: Vec::new(),
            last_tick_cohesion: BTreeMap::new(),
            last_tick_cohesion_snapshots: BTreeMap::new(),
            settlement_gini: BTreeMap::new(),
            last_tick_unrest_events: Vec::new(),
            last_tick_unrest_levels: BTreeMap::new(),
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

    /// Snapshot doctrine libraries for full-state save/load serialization.
    #[must_use]
    pub(crate) fn saveable_faction_doctrines(&self) -> Vec<DoctrineLibrary> {
        self.faction_doctrines.clone()
    }

    /// Restore doctrine libraries after simulation state load.
    pub(crate) fn restore_faction_doctrines(&mut self, doctrines: Vec<DoctrineLibrary>) {
        self.faction_doctrines = doctrines;
    }

    /// Read-only view of active institutions (FR-CIV-GOV / emergence oracles).
    #[must_use]
    pub fn institutions(&self) -> &BTreeMap<u32, civ_institutions::Institution> {
        &self.institutions
    }

    /// Snapshot institution state for full-state persistence.
    #[must_use]
    pub(crate) fn saveable_institution_state(
        &self,
    ) -> (
        BTreeMap<u32, u32>,
        BTreeMap<u32, civ_institutions::Institution>,
        BTreeSet<(u32, u8, u8)>,
    ) {
        (
            self.settlements.clone(),
            self.institutions.clone(),
            self.institution_levels_emitted.clone(),
        )
    }

    /// Restore institution state after simulation state load.
    pub(crate) fn restore_institution_state(
        &mut self,
        settlements: BTreeMap<u32, u32>,
        institutions: BTreeMap<u32, civ_institutions::Institution>,

        institution_levels_emitted: BTreeSet<(u32, u8, u8)>,
    ) {
        self.settlements = settlements;
        self.institutions = institutions;
        self.institution_levels_emitted = institution_levels_emitted;
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

    /// Back-compat alias for code that still expects the old tick accessor.
    pub fn current_tick(&self) -> u64 {
        self.state.tick
    }

    /// Borrow the latest weather grid for the current tick.
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

    pub(crate) fn apply_replay_diplomacy_action(
        &mut self,
        tick: u64,
        source_faction: u32,
        target_faction: u32,
        kind: DiplomacyKind,
    ) {
        self.state.tick = tick;
        self.apply_player_diplomacy_action(source_faction, target_faction, kind);
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

    /// Disasters resolved during the most recent tick.
    #[must_use]
    pub fn last_tick_disaster_pulses(&self) -> &[crate::disasters::DisasterPulse] {
        &self.last_tick_disaster_pulses
    }

    /// Settlement cluster ids from the most recent life rollup.
    #[must_use]
    pub fn last_settlement_ids(&self) -> &[u64] {
        &self.last_settlement_ids
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

    /// Audio triggers produced during the most recent tick.
    #[must_use]
    pub fn last_tick_audio_events(&self) -> &[SfxTrigger] {
        &self.last_tick_audio_events
    }

    /// Borrow the building graph.
    pub fn building_graph(&self) -> &BuildingGraph {
        &self.building_graph
    }

    pub(crate) fn building_graph_mut(&mut self) -> &mut BuildingGraph {
        &mut self.building_graph
    }

    /// Borrow the most recent cohort diffusion statistics.
    pub fn last_cohort_stats(&self) -> Option<&CohortStats> {
        self.last_cohort_stats.as_ref()
    }

    /// Borrow the research cache.
    pub fn research_cache(&self) -> &ResearchCache {
        &self.research_cache
    }

    /// Mutable access for JSON-RPC `sim.queue_research` (FR-CIV-TECH).
    pub fn research_cache_mut(&mut self) -> &mut ResearchCache {
        &mut self.research_cache
    }

    pub fn researched_tech_count(&self) -> usize {
        self.research_cache.researched.len()
    }

    /// Read-only access to the current climate state (same-crate accessor so
    /// sibling modules such as [`crate::disasters`] can read it without the
    /// field being made `pub`).
    pub(crate) fn climate_state(&self) -> &Climate {
        &self.climate
    }

    /// Read-only access to the per-region weather grid (same-crate accessor;
    /// see [`Self::climate_state`]).
    pub(crate) fn weather_cells(&self) -> &[WeatherCell] {
        &self.weather_grid
    }

    /// Inject a climate state (same-crate test/scenario hook so sibling-module
    /// tests can stage environmental conditions without the field being `pub`).
    pub(crate) fn set_climate_state(&mut self, climate: Climate) {
        self.climate = climate;
    }

    /// Inject a weather grid (see [`Self::set_climate_state`]).
    pub(crate) fn set_weather_cells(&mut self, cells: Vec<WeatherCell>) {
        self.weather_grid = cells;
    }

    /// Current accumulated belief (faith) — the divine-powers currency.
    pub fn belief(&self) -> u64 {
        self.state.belief
    }

    /// Spend accumulated belief (faith) to invoke a divine power.
    ///
    /// Belief is the emergent currency of the disasters → faith →
    /// divine-intervention loop (FR-CIV-EMERGENCE). If at least `cost` belief
    /// has accumulated this is deducted and `true` returned; otherwise the
    /// state is untouched and `false` returned (the gate stays closed).
    pub(crate) fn try_invoke_divine_power(&mut self, cost: u64) -> bool {
        if self.state.belief >= cost {
            self.state.belief -= cost;
            true
        } else {
            false
        }
    }

    /// Borrow emergent era progression state (FR-ERA).
    #[must_use]
    pub fn era_progression(&self) -> &crate::era::EraProgressionState {
        &self.era_progression
    }

    /// Mutably borrow emergent era progression state (FR-ERA).
    pub(crate) fn era_progression_mut(&mut self) -> &mut crate::era::EraProgressionState {
        &mut self.era_progression
    }

    /// Research tier derived from unlocked faction tech, with legacy cache fallback.
    #[must_use]
    pub fn research_tier(&self) -> u64 {
        let faction_tech_tier = self
            .era_progression
            .faction_tech
            .values()
            .map(|state| u64::from(state.tech_level))
            .max()
            .unwrap_or(0);
        faction_tech_tier.max(self.research_cache.researched.len() as u64)
    }

    /// Per-faction ideology and behavior-coupling vectors.
    #[must_use]
    pub fn faction_ideologies(&self) -> &BTreeMap<u32, FactionIdeologyState> {
        &self.faction_ideologies
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
    pub fn agent_entity(&self, agent_id: u64) -> Option<Entity> {
        self.world
            .query::<&AgentCivilian>()
            .iter()
            .find_map(|(entity, civilian)| (civilian.id == agent_id).then_some(entity))
    }

    /// Apply a bounded belief pulse to the global world state.
    pub fn add_belief(&mut self, delta: i64) {
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

    /// Phase hook for emergent building emergence (FR-CIV-ARCH).
    ///
    /// Re-applies culture/biome/era facades and settlement-cluster layout to
    /// every parcel currently in the building graph.
    pub fn run_building_emergence_tick(&mut self) {
        use crate::building_emergence::{
            apply_emergence_facades, emergence_demand_signals, emergent_style_key_for_sim,
            settlement_build_anchor,
        };

        let geology = GeologyMap::seed(&self.planet);
        let (cluster_id, anchor) = settlement_build_anchor(&self.world);
        let style = emergent_style_key_for_sim(self, cluster_id, &geology, &anchor);
        let raw = DemandSignals {
            residential: 0.75,
            commercial: 0.25,
            industrial: 0.25,
            civic: 0.75,
        };
        let signals = emergence_demand_signals(self, raw, style.era);
        let allocated: Vec<_> = self.building_graph.parcels.iter().map(|p| p.id).collect();
        if !allocated.is_empty() {
            apply_emergence_facades(self, cluster_id, style, signals, &allocated);
        }
    }

    /// Default sentience profile for new civilizations (FR-CIV-GENETICS).
    /// Stub-as-associated-fn; callers invoke as `default_sentience_profile()`.
    /// The body delegates to `pub free fn default_sentience_profile` below.
    pub fn default_sentience_profile(&self) -> CognitionTraitProfile {
        default_sentience_profile()
    }

    // Moved to ai_decision.rs
    /// Stable cache key for a (resource, region) pair on the market bus.
    /// Stub: returns empty string; full impl depends on ResourceType enum schema.
    pub fn resource_market_key(_resource: &str, _region: u32) -> String {
        String::new()
    }

    /// Count of civilians grouped by settlement id (FR-CIV-SOC).
    /// Stub: returns empty map; full impl walks `self.world.query::<&Citizen>()`.
    pub fn settlement_member_counts(&self) -> BTreeMap<u32, u32> {
        BTreeMap::new()
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
        self.current_tick = self.state.tick;
        let famine_at_tick_start = self.state.resources.food.to_bits() <= 0;
        self.last_tick_combat_pulses.clear();
        self.last_tick_disaster_pulses.clear();
        self.last_tick_engagements.clear();
        self.last_tick_mod_lifecycle.clear();
        self.last_tick_construction_events.clear();
        self.last_tick_settlement_trade_flows.clear();
        self.last_tick_stratification.clear();
        self.last_tick_stratification_reports.clear();
        // Audio buffer is reset at the top of the tick so caller-side
        // builders (disasters, god-tool handlers, mod hooks) can record
        // audio events mid-tick; `phase_audio` re-emits the survivors
        // alongside combat + construction events on the wire.
        self.last_tick_audio_events.clear();
        self.last_tick_music_cues.clear();
        self.last_tick_disaster_events.clear();
        // Social-mood buffer (FR-CIV-GOV-100). `phase_social_mood` overwrites
        // this with the per-settlement snapshots each tick; downstream
        // consumers (`last_tick_mood`, `last_tick_mood_all`) read a fresh
        // value when called after `tick()` returns.
        self.last_tick_mood.clear();
        self.last_tick_cohesion_events.clear();
        self.last_tick_unrest_events.clear();
        self.econ_focus_stability.clear();

        // Phases in PHASE_ORDER (CIV-0001)
        self.phase_production();
        self.phase_citizen_lifecycle();
        self.phase_military();
        self.phase_policy();
        self.phase_economy();
        self.phase_planet();
        self.phase_disasters();
        self.diplomacy_events.clear();
        self.phase_diplomacy();
        self.phase_faction_decisions();
        self.phase_tactics();
        self.phase_voxel();
        self.phase_compact();
        self.phase_buildings();
        self.phase_life();
        self.phase_daily_path();
        self.phase_cluster();
        self.phase_research();
        self.phase_tech();
        self.phase_belief();
        self.phase_unrest();
        self.phase_cohesion();
        self.phase_social_mood();
        self.phase_economic_focus_pre();
        self.phase_stratification();
        self.phase_institutions();
        self.phase_economic_focus();
        self.phase_emergence();
        self.phase_emergence_events_close();
        self.phase_tutorial();
        self.phase_psyche_behavior();
        self.phase_culture();
        self.phase_language();
        self.phase_sentience();
        self.phase_species();
        self.phase_diffusion();
        // Run after all event-producing phases so this tick's combat,
        // construction, and disaster triggers reach the snapshot.
        self.phase_audio();
        // Victory/defeat after event phases so last_game_outcome matches this tick.
        self.phase_victory_check();
        self.replay_log.record_tick(self.state.tick);

        #[cfg(debug_assertions)]
        debug_assert!(
            crate::integrity::check_tick_integrity(self).is_ok(),
            "simulation integrity violated"
        );
    }

    /// Dispatch a single [`PHASE_ORDER`] entry to the corresponding `phase_*`
    /// method. This is the single source of truth linking the ordered phase
    /// list to actual simulation work (FR-ENGINE-phaseorder).
    ///
    /// Adding a new phase means: extend [`PHASE_ORDER`], add a `phase_*` method,
    /// and add a match arm here — three coupled edits in one file.
    fn run_phase(&mut self, phase: &str) {
        match phase {
            "production" => self.phase_production(),
            "citizen_lifecycle" => self.phase_citizen_lifecycle(),
            "military" => self.phase_military(),
            "policy" => self.phase_policy(),
            "economy" => self.phase_economy(),
            "planet" => self.phase_planet(),
            "disasters" => self.phase_disasters(),
            "diplomacy" => self.phase_diplomacy(),
            "faction_decisions" => self.phase_faction_decisions(),
            "tactics" => self.phase_tactics(),
            "voxel" => self.phase_voxel(),
            "compact" => self.phase_compact(),
            "buildings" => self.phase_buildings(),
            "daily_path" => self.phase_daily_path(),
            "life" => self.phase_life(),
            "research" => self.phase_research(),
            "tech" => self.phase_tech(),
            "belief" => self.phase_belief(),
            "unrest" => self.phase_unrest(),
            "cohesion" => self.phase_cohesion(),
            "social_mood" => self.phase_social_mood(),
            "economic_focus_pre" => self.phase_economic_focus_pre(),
            "stratification" => self.phase_stratification(),
            "institutions" => self.phase_institutions(),
            "economic_focus" => self.phase_economic_focus(),
            "emergence" => self.phase_emergence(),
            "tutorial" => self.phase_tutorial(),
            "psyche_behavior" => self.phase_psyche_behavior(),
            "culture" => self.phase_culture(),
            "language" => self.phase_language(),
            "sentience" => self.phase_sentience(),
            "species" => self.phase_species(),
            "diffusion" => self.phase_diffusion(),
            "audio" => self.phase_audio(),
            "cluster" => self.phase_cluster(),
            "victory_check" => self.phase_victory_check(),
            other => unreachable!("Simulation::run_phase: unknown phase '{other}' in PHASE_ORDER"),
        }
    }

    /// Species-level evolutionary phase (FR-CIV-SPECIES-EVOLUTION).
    ///
    /// Runs mutation, selection, crossover, and speciation on civilian DNA each
    /// tick via [`civ_species::evolution::EvolutionEngine`]. DNA mutates
    /// occasionally, species populations are pruned by fitness, and new
    /// offspring DNA is synthesised from crossover of the most fit pairs.
    fn phase_species(&mut self) {
        let engine = EvolutionEngine::default();
        let tick = self.state.tick;

        // Collect all civilians and their DNA.
        let agents: Vec<(Entity, u64, civ_genetics::Dna)> = self
            .world
            .query::<(&AgentCivilian, &civ_genetics::Dna)>()
            .iter()
            .map(|(e, (civ, dna))| (e, civ.id, dna.clone()))
            .collect();

        if agents.is_empty() {
            return;
        }

        // 1. MUTATE: each civilian's DNA may mutate based on mutation_rate.
        for (entity, _id, _dna) in &agents {
            if let Ok(mut dna_ref) = self.world.get::<&mut civ_genetics::Dna>(*entity) {
                // Build a Species wrapper for the EvolutionEngine::mutate API.
                let mut species = Species {
                    id: 0,
                    dna_class: "civilian".to_string(),
                    founder_centroid: dna_ref.clone(),
                };
                let mut local_rng = ChaCha8Rng::seed_from_u64(self.state.rng_seed ^ tick ^ _id);
                engine.mutate(&mut species, &mut local_rng);
                *dna_ref = species.founder_centroid;
            }
        }

        // Re-collect after mutation so selection sees updated DNA.
        let agents: Vec<(Entity, u64, civ_genetics::Dna)> = self
            .world
            .query::<(&AgentCivilian, &civ_genetics::Dna)>()
            .iter()
            .map(|(e, (civ, dna))| (e, civ.id, dna.clone()))
            .collect();

        // 2. SPECIATION: group civilians into genetic clusters.
        let species_pop: Vec<Species> = agents
            .iter()
            .map(|(_entity, id, dna)| Species {
                id: *id,
                dna_class: "civilian".to_string(),
                founder_centroid: dna.clone(),
            })
            .collect();

        let clusters = engine.speciation(&species_pop, 0.3);

        // 3. SELECT: apply selection pressure per species cluster.
        for cluster in &clusters {
            if cluster.len() < 2 {
                continue;
            }
            let _selected = engine.select(cluster, |s| {
                // Fitness from phenotype: average of behaviour weights.
                let phenotype = express(&s.founder_centroid);
                let w = &phenotype.behavior;
                ((w.aggression + w.curiosity + w.sociability + w.intelligence) / 4.0) as f64
            });
            // Selection is purely informational at this stage — the surviving
            // genome pools are used for crossover below.
        }

        // 4. CROSSOVER: for each pair of civilians in the same cluster that
        //    should reproduce, create offspring DNA and update one partner.
        //    Use a deterministic RNG seeded by tick + pair ids.
        for cluster in &clusters {
            if cluster.len() < 2 {
                continue;
            }
            // Take pairs (0,1), (2,3), (4,5), ... and crossover.
            let mut pairs = cluster.chunks(2);
            while let Some(pair) = pairs.next() {
                if pair.len() < 2 {
                    break;
                }
                let parent_a = &pair[0];
                let parent_b = &pair[1];
                let child = engine.crossover(parent_a, parent_b);

                // Write child DNA back to the second parent entity (the "offspring"
                // is re-embodied as a mutation of the second parent's genome).
                if let Some(child_entity) = agents
                    .iter()
                    .find(|(_, id, _)| *id == parent_b.id)
                    .map(|(e, _, _)| *e)
                {
                    if let Ok(mut dna_ref) = self.world.get::<&mut civ_genetics::Dna>(child_entity)
                    {
                        *dna_ref = child.founder_centroid;
                    }
                }
            }
        }
    }

    // Moved to world_simulation.rs
    fn phase_victory_check(&mut self) {
        self.last_game_outcome = crate::conditions::check_outcome(self);
    }

    fn phase_tutorial(&mut self) {
        let mut tutorial = std::mem::take(&mut self.tutorial_progress);
        tutorial.advance_from_sim(self);
        self.tutorial_progress = tutorial;
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
    pub(crate) fn ingest_mod_phase_lines(&mut self, lines: Vec<String>, tick: u64, phase: &str) {
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
        sim.last_audio_researched_len = sim.research_cache.researched.len();
        Ok(sim)
    }

    /// Set (or replace) the population snapshot for a settlement. Used by
    /// tests + scenario loaders to drive `phase_institutions` deterministically
    /// (FR-CIV-GOV-001).
    pub fn set_settlement_population(&mut self, settlement_id: u32, population: u32) {
        self.settlements.insert(settlement_id, population);
    }

    /// Read-only view of the civic-institution events produced by the most
    /// recent [`Self::phase_institutions`] call. Cleared at the start of
    /// every [`Simulation::tick`].
    #[must_use]
    pub fn last_tick_institution_events(&self) -> &[InstitutionEvent] {
        &self.last_tick_institution_events
    }

    /// Per-settlement per-tick social-mood snapshot, computed by
    /// [`Self::phase_social_mood`]. Cleared at the start of every
    /// [`Simulation::tick`].
    ///
    /// `last_tick_mood(settlement_id) -> Option<&MoodSnapshot>` returns the
    /// snapshot for one settlement (if any was computed this tick).
    /// `last_tick_mood_all() -> &[MoodSnapshot]` returns the full list.
    #[must_use]
    pub fn last_tick_mood(&self, settlement_id: u32) -> Option<&MoodSnapshot> {
        self.last_tick_mood
            .iter()
            .find(|s| s.settlement_id == settlement_id)
    }

    /// All per-settlement mood snapshots from the most recent tick. Read-only.
    #[must_use]
    pub fn last_tick_mood_all(&self) -> &[MoodSnapshot] {
        &self.last_tick_mood
    }

    /// Set (or replace) the food-stocked snapshot for a settlement.
    /// `phase_social_mood` consumes it to compute `food_score`. Units are
    /// the same as `Stocks::food()` (i64 food units; clamped at 0 on read).
    pub fn set_settlement_food_stocked(&mut self, settlement_id: u32, units: i64) {
        self.settlement_food_stocked.insert(settlement_id, units);
    }

    /// Set (or replace) the housing capacity for a settlement. Used by
    /// `phase_social_mood` to compute `housing_score` (capacity vs population).
    pub fn set_settlement_housing_capacity(&mut self, settlement_id: u32, units: u32) {
        self.settlement_housing_capacity
            .insert(settlement_id, units);
    }

    /// Set (or replace) the crime pressure (0..300) for a settlement. Used
    /// by `phase_social_mood` to compute `crime_score`.
    pub fn set_settlement_crime_pressure(&mut self, settlement_id: u32, units: i32) {
        self.settlement_crime_pressure.insert(settlement_id, units);
    }

    // Moved to world_simulation.rs
    // ===== phase_cohesion (FR-CIV-GOV-030) =====

    /// Register a kinship edge from `actor_id` to `target`. The edge contributes
    /// to per-settlement fabric via `phase_cohesion`.
    pub fn register_kinship(&mut self, actor_id: u64, kinship: KinshipEdge) {
        self.kinship.entry(actor_id).or_default().push(kinship);
    }

    /// Add (or subtract) trust between two actors. Negative `amount` erodes trust.
    pub fn add_trust(&mut self, actor_id: u64, target: u64, amount: i64) {
        let entry = self.trust.entry(actor_id).or_default();
        let current = entry.get(&target).copied().unwrap_or(0);
        entry.insert(target, current.saturating_add(amount));
    }

    /// Pin an actor to a settlement so `phase_cohesion` can aggregate per-actor
    /// contributions into per-settlement fabric.
    pub fn set_settlement_actor(&mut self, actor_id: u64, settlement_id: u32) {
        self.settlement_actors
            .entry(settlement_id)
            .or_default()
            .insert(actor_id);
        self.actor_settlement.insert(actor_id, settlement_id);
    }

    /// Set hardship for an actor (food scarcity, plague, war, etc.).
    /// High hardship erodes cohesion fabric.
    pub fn set_actor_in_settlement_hardship(&mut self, actor_id: u64, hardship: i64) {
        self.actor_hardship.insert(actor_id, hardship.max(0));
    }

    /// Toggle whether an actor's settlement has a Temple and/or Garrison.
    /// Both institutions mitigate hardship's impact on fabric.
    pub fn set_actor_in_settlement_institutions(
        &mut self,
        actor_id: u64,
        has_temple: bool,
        has_garrison: bool,
    ) {
        let entry = self.actor_institutions.entry(actor_id).or_default();
        *entry = (has_temple, has_garrison);
    }

    /// Per-tick stream of `CohesionEvent`s, cleared at the start of every tick.
    pub fn last_tick_cohesion(&self) -> &[CohesionEvent] {
        &self.last_tick_cohesion_events
    }

    /// Per-settlement cohesion snapshot emitted by `phase_cohesion`.
    pub fn last_tick_cohesion_settlement(&self, settlement_id: u32) -> Option<CohesionSnapshot> {
        self.last_tick_cohesion_snapshots
            .get(&settlement_id)
            .cloned()
    }

    /// Per-settlement cohesion snapshots from the most recent tick.
    pub fn last_tick_cohesion_snapshots(&self) -> &BTreeMap<u32, CohesionSnapshot> {
        &self.last_tick_cohesion_snapshots
    }

    /// Mutable access for drivers/tests that seed cohesion before
    /// [`crate::faction_decisions::compute_faction_decisions`].
    pub fn last_tick_cohesion_snapshots_mut(&mut self) -> &mut BTreeMap<u32, CohesionSnapshot> {
        &mut self.last_tick_cohesion_snapshots
    }

    /// Register a household as part of the global household population.
    /// `phase_stratification` uses the registered households to compute
    /// per-settlement quantiles, Gini coefficient, and class-mobility events.
    pub fn register_household(&mut self, household_id: u64) {
        self.household_settlement.entry(household_id).or_insert(0);
    }

    /// Register a household in a specific settlement. Idempotent: re-adding
    /// the same `(settlement_id, household_id)` pair is a no-op.
    pub fn register_household_in_settlement(&mut self, settlement_id: u32, household_id: u64) {
        self.household_settlement
            .insert(household_id, settlement_id);
        self.settlement_households
            .entry(settlement_id)
            .or_default()
            .insert(household_id);
    }

    /// Set (or replace) the wealth (in fixed-point i64) of a household.
    /// `phase_stratification` uses this to compute wealth quantiles.
    pub fn set_household_wealth(&mut self, household_id: u64, units: i64) {
        self.household_wealth.insert(household_id, units);
    }

    /// Set (or replace) the political power (in fixed-point i64) of a
    /// household. `phase_stratification` uses this to compute power quantiles
    /// and the `top_power_q1` field of the per-settlement report.
    pub fn set_household_power(&mut self, household_id: u64, units: i64) {
        self.household_power.insert(household_id, units);
    }

    /// Read-only access to the `phase_stratification` event stream for the
    /// most recent tick. The slice is reset by `tick()` and repopulated by
    /// the next `phase_stratification` invocation.
    pub fn last_tick_stratification(&self) -> &[StratificationEvent] {
        &self.last_tick_stratification
    }

    /// Per-settlement stratification report from the most recent tick, if
    /// the settlement has at least one registered household.
    pub fn last_tick_stratification_report(
        &self,
        settlement_id: u32,
    ) -> Option<StratificationReport> {
        self.last_tick_stratification_reports
            .get(&settlement_id)
            .cloned()
    }

    /// The current [`StratBand`] assigned to a `(household_id, settlement_id)`
    /// pair, if the household is registered. `None` for unknown households.
    pub fn household_band(&self, household_id: u64, settlement_id: u32) -> Option<StratBand> {
        self.household_bands
            .get(&household_id)
            .copied()
            .filter(|_| self.household_settlement.get(&household_id) == Some(&settlement_id))
    }

    /// Set the Gini coefficient for a settlement's wealth distribution. The
    /// unrest phase reads this each tick to compute unrest amplification from
    /// inequality. `gini` is expected in `[0.0, 1.0]`; values outside that
    /// range are clamped.
    pub fn set_settlement_gini(&mut self, settlement_id: u32, gini: f64) {
        let clamped = if gini.is_nan() {
            0.0
        } else {
            gini.clamp(0.0, 1.0)
        };
        self.settlement_gini
            .insert(settlement_id, (clamped * 100.0).round() as i32);
    }

    /// Read-only access to the `phase_unrest` event stream for the most
    /// recent tick. The slice is reset by `tick()` and repopulated by the
    /// next `phase_unrest` invocation.
    pub fn last_tick_unrest(&self) -> &[UnrestEvent] {
        &self.last_tick_unrest
    }

    /// Per-settlement unrest snapshot from the most recent tick, if any
    /// event was recorded for that settlement.
    pub fn last_tick_unrest_settlement(&self, settlement_id: u32) -> Option<UnrestSnapshot> {
        self.last_tick_unrest_snapshots.get(&settlement_id).cloned()
    }

    /// The current [`UnrestLevel`] for a settlement, derived from the most
    /// recent tick's snapshot. `None` if no snapshot has been recorded yet.
    pub fn unrest_level(&self, settlement_id: u32) -> Option<UnrestLevel> {
        self.last_tick_unrest_snapshots
            .get(&settlement_id)
            .map(|s| s.level)
    }

    /// Read-only view of the sentience events produced by the most recent
    /// [`Self::phase_sentience`] call. Cleared at the start of every tick
    /// (along with the other `last_tick_*` buffers) so callers observe only
    /// the current tick's crossings.
    #[must_use]
    pub fn last_tick_sentience_events(&self) -> &[SentienceEvent] {
        &self.last_tick_sentience_events
    }

    /// Read-only view of the live language state owned by the simulation.
    /// Exposed for tests + the JSON-RPC bridge (the `language` field on
    /// `sim.snapshot` is fed from this accessor).
    #[must_use]
    pub fn language_state(&self) -> &LanguageState {
        &self.language_state
    }

    /// Read-only access to per-faction language state used for naming and
    /// isolation-driven coupling diagnostics.
    #[must_use]
    pub fn faction_languages(&self) -> &BTreeMap<u32, LanguageState> {
        &self.faction_languages
    }

    pub(crate) fn set_faction_languages(
        &mut self,
        faction_languages: BTreeMap<u32, LanguageState>,
    ) {
        self.faction_languages = faction_languages;
        self.language_state = self
            .faction_languages
            .values()
            .next()
            .cloned()
            .unwrap_or_default();
    }

    /// Deterministic place naming for a specific faction.
    #[must_use]
    pub fn faction_place_name(&self, faction_id: u32, place_id: u32) -> String {
        if let Some(state) = self.faction_languages.get(&faction_id) {
            place_name(state, faction_id, place_id)
        } else {
            place_name(&self.language_state, faction_id, place_id)
        }
    }

    /// Deterministic person naming for a specific faction.
    #[must_use]
    pub fn faction_person_name(&self, faction_id: u32, person_id: u32) -> String {
        if let Some(state) = self.faction_languages.get(&faction_id) {
            person_name(state, faction_id, person_id)
        } else {
            person_name(&self.language_state, faction_id, person_id)
        }
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
            faction_eras: self.era_progression.faction_era_snapshots(self),
            tutorial_progress: self.tutorial_progress.clone(),
            music_cues: self.last_tick_music_cues.clone(),
            researched: self.research_cache.researched.clone(),
            in_progress_tech: self
                .research_cache
                .in_progress
                .as_ref()
                .map(|(tech, _)| tech.clone()),
            outcome_progress: crate::conditions::outcome_progress(self),
            last_tick_faction_unrest_response_intents: self
                .state
                .last_tick_faction_unrest_response_intents
                .clone(),
            last_tick_faction_hostility_intents: self
                .state
                .last_tick_faction_hostility_intents
                .clone(),
            last_tick_faction_trade_open_intents: self
                .state
                .last_tick_faction_trade_open_intents
                .clone(),
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

    /// Last-tick settlement trade flows computed in `phase_economy`.
    #[must_use]
    pub fn last_tick_settlement_trade_flows(&self) -> &[SettlementTradeFlow] {
        &self.last_tick_settlement_trade_flows
    }

    /// Per-tick lifecycle label counts populated by [`Simulation::phase_life`]
    /// (FR-CIV-LIFE-001/002/003). Counts each surviving civilian once,
    /// classified via [`civ_needs::classify_lifecycle`]. Read by the HUD
    /// `LifecyclePanel` and the emergence-dashboard consumer; cleared implicitly
    /// each tick by being re-populated.
    #[must_use]
    pub fn last_tick_lifecycle_metrics(&self) -> &LifecycleCounters {
        &self.last_tick_lifecycle_metrics
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

    /// HUD settlement projection + per-cluster commons from live co-location clusters.
    pub(crate) fn rollup_emergent_settlements(
        &mut self,
        cluster_member_counts: &BTreeMap<u64, u32>,
    ) {
        let mut settlement_ids = Vec::new();
        for (cluster_id, size) in cluster_member_counts {
            if *size >= SETTLEMENT_MIN_MEMBERS {
                settlement_ids.push(*cluster_id);
            }
        }
        self.last_settlement_count = settlement_ids.len() as u32;

        self.cluster_stocks
            .retain(|id, _| settlement_ids.contains(id));
        for cluster_id in settlement_ids {
            let size = cluster_member_counts.get(&cluster_id).copied().unwrap_or(0);
            let production = i64::from(size) * CLUSTER_FOOD_PRODUCTION_PER_MEMBER;
            let consumption = i64::from(size) * CLUSTER_FOOD_CONSUMPTION_PER_MEMBER;
            let stock = self.cluster_stocks.entry(cluster_id).or_default();
            stock.add(Good::Food, production);
            stock.add(Good::Food, -consumption);
        }
    }
}

// Emergence-coupling free functions extracted to `emergence_coupling` module.
pub(crate) use crate::emergence_coupling::*;

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
    /// Per-faction emergent civilization age (FR-ERA).
    #[serde(default)]
    pub faction_eras: std::collections::BTreeMap<u32, crate::era::FactionEraSnapshot>,
    pub tutorial_progress: TutorialProgress,
    /// Per-cluster music cues derived during the audio phase.
    #[serde(default)]
    pub music_cues: BTreeMap<u64, MusicCue>,
    /// Fully-researched tech ids/names from [`ResearchCache`].
    #[serde(default)]
    pub researched: Vec<String>,
    /// Tech id/name currently being researched, if any.
    #[serde(default)]
    pub in_progress_tech: Option<String>,
    /// Live progress toward the victory conditions used by `check_outcome`.
    #[serde(default)]
    pub outcome_progress: crate::conditions::OutcomeProgress,
    /// Factions that raised an unrest-response intent during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_unrest_response_intents: BTreeSet<u32>,
    /// Factions that flagged hostility intent during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_hostility_intents: BTreeSet<u32>,
    /// Factions that flagged trade-open intent during the most recent tick.
    #[serde(default)]
    pub last_tick_faction_trade_open_intents: BTreeSet<u32>,
}

// ADR-020 phase stubs (FR-PLAY-click-to-fire prerequisite: tick() compiles).
// These phases are no-op in this build; downstream mod-host + legacy code
// still references them by name.
//
// NOTE: As of the phase_institutions (FR-CIV-GOV-001/002/003) +
// phase_social_mood (FR-CIV-GOV-010) superset-merge, the canonical home
// for these phase_* methods is the *primary* `impl Simulation` block
// above (so they can be `fn` instead of `pub fn` and benefit from
// inherent-impl encapsulation). The 11 phase_* orphan stubs that used
// to live here have been **deleted** to avoid duplicate-symbol compile
// errors: each is now defined exactly once in the primary block
// (real impls for `phase_institutions` + `phase_social_mood`, no-op
// `fn ... {}` stubs for the other 9 ADR-020 placeholders). The non-phase
// stubs below (`add_cohesion`, `agent_entity`,
// `micro_actor_action_count`, `micro_descendant_action_count`) are
// *not* phase methods and remain in this trailing impl block because
// they have no primary-block duplicates.
impl Simulation {
    /// Snapshot all civilian agent identity components.
    #[must_use]
    pub fn all_agents(&self) -> Vec<AgentCivilian> {
        self.world
            .query::<&AgentCivilian>()
            .iter()
            .map(|(_, civilian)| civilian.clone())
            .collect()
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
mod engine_tests;

/// Re-export of genetics module so callers can use `crate::engine::genetics::...`.
pub mod genetics {
    /// Re-export of SentienceEvent from civ_genetics.
    pub use civ_genetics::sentience::SentienceEvent;
}
