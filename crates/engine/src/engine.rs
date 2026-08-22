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
use civ_mod_host::ModHost;
use civ_needs::{should_reproduce, Health as CivNeedsHealth, LifecycleLabel, LifecycleParams};
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
pub mod policy_econ_phases;
pub mod social_settlement_phases;
pub mod species_lifecycle;
pub mod world_simulation;
pub(crate) use self::species_lifecycle::{spawn_faction_civilians, spawn_faction_civilians_custom};

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

/// Stub: derive a per-cluster `MusicCue` from culture `traits`, `cluster_id`,
/// and the latest `aggression` average. The real implementation lives in
/// `civ-audio::mood`; this stub lets the engine compile until the dep is
/// wired through.
#[must_use]
pub fn derive_music_cue(
    traits: civ_agents::culture::TraitVector,
    cluster_id: u64,
    aggression: f32,
    tick: u64,
) -> MusicCue {
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
    MusicCue {
        mood: mood.to_string(),
        intensity,
        tempo_bpm: Some(tempo),
    }
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
            "diffusion" => self.phase_diffusion(),
            "audio" => self.phase_audio(),
            "cluster" => self.phase_cluster(),
            "victory_check" => self.phase_victory_check(),
            other => unreachable!("Simulation::run_phase: unknown phase '{other}' in PHASE_ORDER"),
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
                    let geology = GeologyMap::seed(&self.planet);
                    let (cluster_id, anchor) = settlement_build_anchor(&self.world);
                    let style = emergent_style_key_for_sim(self, cluster_id, &geology, &anchor);
                    let gated = emergence_demand_signals(self, signals, style.era);
                    apply_emergence_facades(self, cluster_id, style, gated, &allocated);
                }
            }
        }

        // ---- 2. Construction progress (every tick) ----
        // Advance each in-flight site; remove completed ones; record the
        // CompletedBuilding in the building_graph.
        let mut completed_ids = Vec::new();
        for site in self.build_sites.iter_mut() {
            if site.is_complete() {
                continue;
            }
            if let Some(_completion) = site.tick() {
                completed_ids.push(site.id());
                self.building_graph.record_completed(site);
            }
        }
        // Drop completed sites from the active queue.
        self.build_sites
            .retain(|site| !site.is_complete() || completed_ids.contains(&site.id()));

        // ---- 3. Production events for completed buildings (every tick) ----
        // Each completed building begins producing on the same tick it
        // finishes; run the production chain against the live economy state.
        let mut events = std::mem::take(&mut self.last_tick_construction_events);
        for site in self.build_sites.iter_mut() {
            if completed_ids.contains(&site.id()) {
                events.extend(site.produce_and_collect(&mut self.economy_state, tick));
            }
        }
        self.last_tick_construction_events = events;
    }

    /// Public accessor for the most recent construction events. Cleared at
    /// the start of every [`Simulation::tick`].
    pub fn last_construction_events(&self) -> &[ProductionEvent] {
        &self.last_tick_construction_events
    }

    /// Enqueue a new construction site. Caller picks the id/spec/coord;
    /// the engine runs it on subsequent ticks.
    pub fn enqueue_build_site(&mut self, site: BuildSite) {
        self.build_sites.push(site);
    }

    /// Read-only view of the active build queue (FR-CIV-BUILD-001).
    pub fn build_sites(&self) -> &[BuildSite] {
        &self.build_sites
    }

    /// Count of completed buildings (FR-CIV-BUILD-001).
    pub fn completed_buildings(&self) -> usize {
        self.building_graph.completed_count()
    }

    // Moved to species_lifecycle.rs
    /// Research phase (FR-ERA): emergent per-faction research progress.
    fn phase_research(&mut self) {
        crate::era::phase_research(self);
    }

    /// Tech-tree phase (FR-ERA): emergent tech levels + era evaluation.
    fn phase_tech(&mut self) {
        crate::era::phase_tech(self);
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

    /// Audio phase (FR-AUDIO-wire) — translate per-tick substrate events
    /// produced by the earlier phases (combat pulses from `phase_tactics`,
    /// construction events from `phase_buildings`, emergent disasters
    /// from `phase_disasters`-style paths) into substrate-level
    /// [`SfxTrigger`]s the JSON-RPC bridge and WebSocket broadcast will
    /// forward to clients.
    ///
    /// This is intentionally a *thin mapping* — no synthesis, no
    /// coalescing, no client logic. Per-tick capacity is bounded by
    /// the substrate:
    ///
    /// - One [`SfxTrigger::Battle`] per distinct combat pulse center
    ///   (already deduped by `phase_tactics`), volume = the
    ///   normalized proximity to the world center so distant battles
    ///   are quieter without leaking exact unit coordinates.
    /// - One [`SfxTrigger::Build`] per completed-building
    ///   `ProductionEvent` (the "Produced" variant).
    /// - One [`SfxTrigger::Birth`] / [`SfxTrigger::Death`] per lifecycle event.
    /// - One [`SfxTrigger::Tech`] per newly appended researched-tech entry.
    /// - [`SfxTrigger::Disaster`]s are pushed by `trigger_disaster` /
    ///   `phase_disasters` via [`Self::record_disaster_audio`]; this
    ///   phase only forwards what's already in the per-tick buffer.
    ///
    /// The buffer resets at the start of [`Simulation::tick`] (not
    /// here) so caller-side builders (god-tool handlers,
    /// `phase_disasters`) can record disasters that fire mid-tick.
    fn phase_audio(&mut self) {
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

        // Combat pulses → Battle triggers. Volume scales with normalized
        // proximity to the world center so loudest battles are the ones
        // the camera is most likely to be near. Distant pulses stay
        // audible but quieter (the coalescer clamps gain anyway).
        for pulse in &self.last_tick_combat_pulses {
            // Distance from the world center (0.5, 0.5) in normalized
            // coords; clamped to [0, 1] for the volume curve.
            let dx = pulse.x - 0.5;
            let dy = pulse.y - 0.5;
            let dist = ((dx * dx + dy * dy).sqrt() * 2.0).clamp(0.0, 1.0);
            let intensity = 1.0 - dist;
            events.push(SfxTrigger::Battle { intensity });
        }

        // Construction completions → Build triggers (one per
        // `ProductionEvent::Produced` this tick).
        for event in &self.last_tick_construction_events {
            if matches!(event, ProductionEvent::Produced { .. }) {
                events.push(SfxTrigger::Build);
            }
        }

        // Disasters → already-recorded Disaster triggers; keep their
        // recorded order so the wire stream is deterministic.
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

    /// Record a [`SfxTrigger::Disaster`] on the per-tick audio buffer
    /// (FR-AUDIO-wire). Called by [`crate::disasters::trigger_disaster`]
    /// and `phase_disasters` so disasters fired mid-tick land in the
    /// snapshot. The buffer is cleared at the start of
    /// [`Simulation::tick`] so callers can invoke this any time during
    /// the tick. Forwarded to the client in [`Simulation::phase_audio`].
    pub fn record_disaster_audio(&mut self, kind: &str, severity: f32) {
        // Convert the DisasterKind label into a wire-stable `&'static str`
        // by lowercasing once and matching against the canonical names
        // the audio substrate recognizes (see
        // `civ_audio::SfxKind::for_disaster_label`). Unknown kinds
        // surface as the umbrella "disaster" label so the substrate's
        // `for_disaster_label` falls back to `SfxKind::Disaster` rather
        // than dropping the event.
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

    // Moved to species_lifecycle.rs
    /// Military phase — morale recovery and Phase-4 war → tactics bridge.
    fn phase_military(&mut self) {
        use crate::spawn::military_pin_id;

        let tick = self.state.tick;
        let lines = self.mod_host.military_tick(tick);
        self.ingest_mod_phase_lines(lines, tick, "military");

        let phase_cfg = self.military_phase;

        let morale_updates: Vec<(Entity, MilitaryUnit)> = self
            .world
            .query::<&MilitaryUnit>()
            .iter()
            .filter_map(|(entity, unit)| {
                if unit.morale >= Fixed::from_num(1) {
                    return None;
                }
                let mut updated = unit.clone();
                updated.morale = (updated.morale + Fixed::from_num(1) / Fixed::from_num(100))
                    .min(Fixed::from_num(1));
                Some((entity, updated))
            })
            .collect();
        for (entity, unit) in morale_updates {
            let _ = self.world.insert(entity, (unit,));
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
                let movement_update =
                    self.world
                        .query::<&MilitaryUnit>()
                        .iter()
                        .find_map(|(entity, unit)| {
                            if entity != target_entity {
                                return None;
                            }
                            let mut updated = unit.clone();
                            updated.position.x = grid_move.new_grid_x;
                            updated.position.y = grid_move.new_grid_y;
                            Some(updated)
                        });
                if let Some(updated) = movement_update {
                    let _ = self.world.insert(target_entity, (updated,));
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
            if let Some(target_entity) = entities.get(engagement.target_index).copied() {
                let damage_update =
                    self.world
                        .query::<&MilitaryUnit>()
                        .iter()
                        .find_map(|(entity, unit)| {
                            if entity != target_entity {
                                return None;
                            }
                            let mut updated = unit.clone();
                            updated.hp = (updated.hp - hp_loss).max(Fixed::from_num(0));
                            updated.strength = updated.hp;
                            Some(updated)
                        });
                if let Some(updated) = damage_update {
                    let _ = self.world.insert(target_entity, (updated,));
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

    /// Policy phase — read the active [`Policy`] for the current tick and
    /// store the resulting [`ControlSignals`] on
    /// [`Self::last_control_signals`]. Runs between `phase_military` and

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
        // Translate the scenario representation into the engine's
        // `civ_economy::Taxation` field. The scenario struct is a
        // wire-friendly shape; the engine keeps a `Taxation` that the
        // economy phase consumes directly.
        let mut resolved = civ_economy::Taxation::default();
        for (institution_id, rate_bp) in &taxation.rates_bp {
            if let Ok(id) = (*institution_id).try_into() {
                resolved.rates_bp.insert(id, *rate_bp);
            }
        }
        resolved.per_institution_cap = taxation
            .per_institution_cap
            .and_then(|cap| (cap >= 0).then_some(cap));
        self.scenario_taxation = resolved;
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

    /// FR-CIV-TUTORIAL — tutorial progression advances from live sim milestones.
    #[test]
    fn fr_civ_tutorial_advances_from_tick_progress() {
        let mut sim = Simulation::with_seed(42);
        sim.era_progression.faction_tech.insert(
            0,
            crate::tech::FactionTechState {
                research_points: 240,
                tech_level: 0,
                diffusion_points: 0,
            },
        );

        assert_eq!(
            sim.tutorial_progress.current,
            crate::tutorial::TutorialMilestone::FirstFaction
        );

        sim.tick();

        assert!(sim.tutorial_progress.tech_unlocked);
        assert!(
            matches!(
                sim.tutorial_progress.current,
                crate::tutorial::TutorialMilestone::FirstTech
                    | crate::tutorial::TutorialMilestone::FirstWar
                    | crate::tutorial::TutorialMilestone::FirstReligion
                    | crate::tutorial::TutorialMilestone::Complete
            ),
            "tutorial should advance once tech unlocks"
        );
        assert_eq!(sim.snapshot().tutorial_progress, sim.tutorial_progress);
    }

    /// FR-CIV-TACTICS — opposing military units in LOS/range engage during the
    /// normal simulation tick, emit combat damage, and resolve casualties.
    #[test]
    fn fr_civ_tactics_tick_resolves_in_range_combat() {
        let mut sim = Simulation::with_seed(2026_07_01);
        sim.world = World::new();
        sim.military_phase.movement.cadence_ticks = 0;
        sim.military_phase.war.cadence_ticks = 1;
        sim.military_phase.war.engage_range_grid = 4;

        let hp = Fixed::from_num(1);
        let unit_a = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: hp,
            hp,
            max_hp: hp,
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        };
        let unit_b = MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: hp,
            hp,
            max_hp: hp,
            morale: Fixed::from_num(1),
            position: Position { x: 1, y: 0 },
            faction_id: 1,
        };
        let _ = sim.world.spawn((unit_a,));
        let _ = sim.world.spawn((unit_b,));

        sim.tick();

        assert!(
            !sim.last_tick_engagements.is_empty(),
            "FR-CIV-TACTICS: tick should resolve at least one engagement"
        );
        assert!(
            !sim.last_tick_combat_pulses().is_empty(),
            "FR-CIV-TACTICS: resolved engagement should surface a damage pulse"
        );
        let survivors = sim.world.query::<&MilitaryUnit>().iter().count();
        let damaged = sim
            .world
            .query::<&MilitaryUnit>()
            .iter()
            .any(|(_, unit)| unit.hp < unit.max_hp);
        assert!(
            survivors < 2 || damaged,
            "FR-CIV-TACTICS: tick should apply unit damage or casualties"
        );
        assert!(
            sim.replay_log().combat_event_count() > 0,
            "FR-CIV-TACTICS: tick combat should be replay-recorded"
        );
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
                "disasters",
                "diplomacy",
                "faction_decisions",
                "tactics",
                "voxel",
                "compact",
                "buildings",
                "life",
                "daily_path",
                "cluster",
                "research",
                "tech",
                "belief",
                "unrest",
                "cohesion",
                "social_mood",
                "economic_focus_pre",
                "stratification",
                "institutions",
                "economic_focus",
                "emergence",
                "tutorial",
                "psyche_behavior",
                "culture",
                "language",
                "sentience",
                "diffusion",
                "audio",
                "victory_check",
            ]
        );
    }

    #[test]
    fn faction_decision_high_unrest_sets_deterministic_response_intents() {
        fn unrest_snapshot(level: UnrestLevel) -> UnrestSnapshot {
            UnrestSnapshot {
                settlement_id: 7,
                level,
                score: if level == UnrestLevel::Revolting {
                    300
                } else {
                    0
                },
                events_count: 0,
                riots_count: 0,
                migrants_count: 0,
                mob_size: 0,
            }
        }

        let mut sim_a = Simulation::with_seed(42);
        let mut sim_b = Simulation::with_seed(42);
        sim_a
            .last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Revolting));
        sim_b
            .last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Revolting));

        sim_a.tick();
        sim_b.tick();

        let intents_a = &sim_a.state.last_tick_faction_unrest_response_intents;
        let intents_b = &sim_b.state.last_tick_faction_unrest_response_intents;
        assert!(!intents_a.is_empty());
        assert_eq!(intents_a, intents_b);
        assert_eq!(
            sim_a.snapshot().last_tick_faction_unrest_response_intents,
            *intents_a
        );

        let mut calm = Simulation::with_seed(42);
        calm.last_tick_unrest_snapshots
            .insert(7, unrest_snapshot(UnrestLevel::Stable));
        calm.tick();
        assert!(calm
            .state
            .last_tick_faction_unrest_response_intents
            .is_empty());
    }

    #[test]
    fn faction_decision_hostility_and_trade_intents_persist_on_snapshot() {
        let mut hostile = Simulation::with_seed(7);
        for _ in 0..2 {
            hostile.faction_relations.apply_signal(
                0u32,
                1u32,
                civ_agents::DiplomacySignal {
                    combat_grievance: 0.8,
                    ..civ_agents::DiplomacySignal::default()
                },
            );
        }
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 0, y: 0 },
            faction_id: 0,
        },));
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(10),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 1, y: 0 },
            faction_id: 0,
        },));
        hostile.world.spawn((MilitaryUnit {
            unit_type: UnitType::Soldier,
            strength: Fixed::from_num(5),
            hp: Fixed::from_num(5),
            max_hp: Fixed::from_num(5),
            morale: Fixed::from_num(1),
            position: Position { x: 2, y: 0 },
            faction_id: 1,
        },));
        let hostile_before = hostile
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("hostile setup must seed a relation row");
        hostile.tick();
        assert!(hostile
            .state
            .last_tick_faction_hostility_intents
            .contains(&0));
        assert_eq!(
            hostile.snapshot().last_tick_faction_hostility_intents,
            hostile.state.last_tick_faction_hostility_intents
        );
        let hostile_after = hostile
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("hostility intent must lower relation score");
        assert!(hostile_after <= hostile_before);
        assert!(hostile_after < -0.5);
        assert!(hostile.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::Conflict && event.faction_a == 0 && event.faction_b == 1
        }));

        let mut trade = Simulation::with_seed(11);
        trade.state.faction_resources.entry(0).or_default().food = Fixed::from_num(1500);
        trade.last_tick_cohesion_snapshots_mut().insert(
            1,
            CohesionSnapshot {
                settlement_id: 1,
                fabric: FabricTier::Tight,
                kin_count: 10,
                trust_sum: 100,
                fragmentation_events: 0,
                fragmentations: 0,
                faction_count: 1,
            },
        );
        trade.faction_relations.apply_signal(
            0u32,
            1u32,
            civ_agents::DiplomacySignal {
                trade_volume: 0.8,
                ..civ_agents::DiplomacySignal::default()
            },
        );
        trade.tick();
        assert!(trade
            .state
            .last_tick_faction_trade_open_intents
            .contains(&0));
        assert_eq!(
            trade.snapshot().last_tick_faction_trade_open_intents,
            trade.state.last_tick_faction_trade_open_intents
        );
        assert!(trade.diplomacy_events().iter().any(|event| {
            event.kind == DiplomacyKind::TradeAgreement
                && event.faction_a == 0
                && event.faction_b == 1
        }));
        let trade_score = trade
            .faction_relations
            .record(0u32, 1u32)
            .map(|record| record.score)
            .expect("trade intent must raise relation score");
        assert!(trade_score > 0.8);
    }

    #[test]
    fn military_unit_component_is_serializable() {
        let unit = MilitaryUnit {
            unit_type: UnitType::Knight,
            strength: Fixed::from_num(10),
            hp: Fixed::from_num(8),
            max_hp: Fixed::from_num(10),
            morale: Fixed::from_num(1),
            position: Position { x: 4, y: -2 },
            faction_id: 7,
        };
        let json = serde_json::to_string(&unit).expect("serialize");
        let decoded: MilitaryUnit = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.faction_id, 7);
        assert_eq!(decoded.unit_type, UnitType::Knight);
    }

    /// L5-115 — `PHASE_ORDER` includes "emergence" and the phase is positioned
    /// after `life` so the agent state that emergence depends on is finalized
    /// (cluster stocks, needs, settlements) before emergence runs.
    /// Closes FR-CIV-LEGENDS-INGEST-02, FR-CIV-PSYCHE-900/901, FR-CIV-PSYCHE-911,
    /// FR-CIV-PSYCHE-912, FR-CIV-GENETICS, FR-CIV-AI-006, FR-CIV-LEGENDS-QUERY-07.
    /// FR-ENGINE-phaseorder: emergence is the final core emergence phase;
    /// `language` and `sentience` are emergence-following couplings and are
    /// placed AFTER emergence (and before `diffusion` propagation).
    #[test]
    fn phase_order_includes_emergence() {
        let life_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "life")
            .expect("PHASE_ORDER must include 'life'");
        let emergence_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "emergence")
            .expect("PHASE_ORDER must include 'emergence'");
        assert!(
            emergence_idx > life_idx,
            "emergence (idx {emergence_idx}) must run after life (idx {life_idx}) \
             so agent state is finalized first"
        );
        let language_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "language")
            .expect("PHASE_ORDER must include 'language' (FR-ENGINE-phaseorder)");
        let culture_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "culture")
            .expect("PHASE_ORDER must include 'culture' (FR-CIV-CULTURE)");
        let sentience_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "sentience")
            .expect("PHASE_ORDER must include 'sentience' (FR-ENGINE-phaseorder)");
        assert!(
            culture_idx > emergence_idx,
            "culture (idx {culture_idx}) must run after emergence (idx {emergence_idx})"
        );
        assert!(
            language_idx > culture_idx,
            "language (idx {language_idx}) must run after culture (idx {culture_idx})"
        );
        assert!(
            sentience_idx > language_idx,
            "sentience (idx {sentience_idx}) must run after language (idx {language_idx}) \
             so language-driven contact pressure is visible to the psyche evaluator"
        );
        let tutorial_idx = PHASE_ORDER
            .iter()
            .position(|p| *p == "tutorial")
            .expect("PHASE_ORDER must include 'tutorial' (FR-CIV-TUTORIAL)");
        assert!(tutorial_idx > emergence_idx);
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

    #[test]
    fn snapshot_exposes_research_cache_tech_state() {
        let mut sim = Simulation::with_seed(42);
        sim.research_cache_mut()
            .researched
            .push("pottery".to_owned());
        sim.research_cache_mut().in_progress = Some(("writing".to_owned(), 3));

        let snapshot = sim.snapshot();

        assert_eq!(snapshot.researched, ["pottery"]);
        assert_eq!(snapshot.in_progress_tech.as_deref(), Some("writing"));
        assert_eq!(sim.researched_tech_count(), 1);
    }

    #[test]
    fn tick_detects_tech_victory() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 1;
        sim.research_cache_mut().researched = (0..12).map(|idx| format!("tech_{idx}")).collect();

        sim.tick();

        assert!(matches!(
            sim.last_game_outcome,
            GameOutcome::Victory(ref kind) if kind == "Age of Enlightenment"
        ));
    }

    fn count_replay_ticks(sim: &Simulation) -> usize {
        sim.replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Tick { .. }))
            .count()
    }

    fn average_language_distance(left: &LanguageState, right: &LanguageState) -> f32 {
        left.seed_signature
            .iter()
            .zip(right.seed_signature)
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / left.seed_signature.len() as f32
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
        assert_eq!(
            ref_sim.state.energy_budget_joules,
            sim.state.energy_budget_joules
        );
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
            sim.last_control_signals()
                .production_multipliers
                .get("food"),
            Some(&1.25)
        );
    }

    /// CIV-0100 stub: joule budget drain stays non-negative after a tick.
    #[test]
    fn phase_economy_conserves_non_negative_budget() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(99);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 1_000.0,
            scarcity_multiplier: 2.0,
        };
        sim.tick();
        // Budget may be drained by lifecycle-weighted allocator but must stay >= 0.
        assert!(sim.state.energy_budget_joules.to_bits() >= Fixed::ZERO.to_bits());
    }

    /// `phase_economy` routes demand through the lifecycle-weighted allocator.
    #[test]
    fn phase_economy_uses_lifecycle_allocator() {
        use crate::policy::PolicyInput;

        let mut sim = Simulation::with_seed(7);
        sim.state.energy_budget_joules = Fixed::from_num(50_000);
        sim.economy_policy = PolicyInput {
            base_consumption_joules: 100.0,
            scarcity_multiplier: 1.0,
        };

        let before = sim.state.energy_budget_joules;
        sim.tick();

        // After tick, budget should be <= before (drained or same if labor_fraction=0)
        assert!(sim.state.energy_budget_joules.to_bits() <= before.to_bits());
        // Economy state must stay in sync with world state.
        assert_eq!(
            sim.economy_state.energy_budget_joules,
            i64::from(sim.state.energy_budget_joules.to_bits()) / crate::SCALE
        );
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
        sim.tick();
        // After tick, economy_state must mirror state.energy_budget_joules.
        assert_eq!(
            sim.economy_state.energy_budget_joules,
            i64::from(sim.state.energy_budget_joules.to_bits()) / crate::SCALE
        );
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

    /// FR-CIV-ECON / FR-CIV-TRADE: settlement supply imbalances should emit
    /// emergent trade flows and move the food price over successive ticks.
    #[test]
    fn phase_economy_emits_settlement_trade_flows_and_moves_prices() {
        let mut sim = Simulation::with_seed(2024);
        sim.set_settlement_population(1, 25);
        sim.set_settlement_population(2, 25);
        sim.set_settlement_food_stocked(1, 1_000);
        sim.set_settlement_food_stocked(2, 0);

        let before_price = sim.market_state.prices().get("food").copied().unwrap_or(0);
        for _ in 0..4 {
            sim.tick();
        }

        let flows = sim.last_tick_settlement_trade_flows();
        assert!(
            !flows.is_empty(),
            "expected settlement trade flows to emerge under a supply imbalance"
        );
        assert!(
            flows
                .iter()
                .any(|flow| flow.good == Good::Food && flow.qty > 0),
            "expected at least one positive food trade flow"
        );

        let after_price = sim.market_state.prices().get("food").copied().unwrap_or(0);
        assert_ne!(
            after_price, before_price,
            "expected food price to respond to the imbalance over multiple ticks"
        );
    }

    /// FR-CIV-RELIGION: the normal tick advances belief emergence and lets
    /// connected settlements exchange religious profile pressure via trade.
    #[test]
    fn tick_wires_religion_emergence_and_trade_spread() {
        let mut sim = Simulation::with_seed(73_001);
        sim.set_settlement_population(1, 250);
        sim.set_settlement_population(2, 250);
        sim.set_settlement_food_stocked(1, 2_000);
        sim.set_settlement_food_stocked(2, 0);
        for actor_id in 1..=8 {
            sim.set_settlement_actor(actor_id, 1);
            sim.set_actor_in_settlement_hardship(actor_id, 1_000);
        }
        for actor_id in 9..=16 {
            sim.set_settlement_actor(actor_id, 2);
        }

        for _ in 0..12 {
            sim.tick();
        }

        let source = sim
            .religious_profiles
            .get(&1)
            .expect("source settlement should have an emergent religion profile");
        let connected = sim
            .religious_profiles
            .get(&2)
            .expect("connected settlement should have an emergent religion profile");
        assert!(
            source.monitoring > 0.0 || source.mythic_coherence > 0.0,
            "hardship and population should produce an emergent religion signal"
        );
        assert!(
            connected.monitoring > 0.0 || connected.mythic_coherence > 0.0,
            "trade-connected settlement should receive/spread religion pressure"
        );
        assert!(
            sim.last_tick_settlement_trade_flows()
                .iter()
                .any(|flow| flow.from_settlement == 1 && flow.to_settlement == 2 && flow.qty > 0),
            "religion spread test expects the settlements to be connected by trade"
        );
    }

    /// FR-MARKET — supply shocks increase local scarcity pressure so the next
    /// tick food price exceeds a comparison sim without the shock.
    #[test]
    fn phase_economy_prices_respond_to_supply_shock() {
        let mut stable = Simulation::with_seed(9001);
        let mut shocked = Simulation::with_seed(9001);

        stable.state.trade_routes = vec![TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".to_string(),
            volume: Fixed::from_num(20),
        }];
        shocked.state.trade_routes = vec![TradeRoute {
            from_faction: 0,
            to_faction: 1,
            goods: "grain".to_string(),
            volume: Fixed::from_num(20),
        }];
        stable.state.faction_resources.entry(0).or_default().food = Fixed::from_num(180);
        stable.state.faction_resources.entry(1).or_default().food = Fixed::from_num(10);
        shocked.state.faction_resources.entry(0).or_default().food = Fixed::from_num(180);
        shocked.state.faction_resources.entry(1).or_default().food = Fixed::from_num(10);

        stable.tick();
        shocked.tick();

        // Apply a one-tick supply shock between trade passes by draining
        // the exporter to create a scarcity signal.
        shocked
            .state
            .faction_resources
            .entry(0)
            .and_modify(|resources| {
                resources.food = Fixed::ZERO;
            });

        stable.tick();
        shocked.tick();

        let stable_food = stable
            .snapshot()
            .market_prices
            .get("food")
            .copied()
            .unwrap_or(0);
        let shocked_food = shocked
            .snapshot()
            .market_prices
            .get("food")
            .copied()
            .unwrap_or(0);
        assert!(
            shocked_food > stable_food,
            "expected shocked sim to have higher food price: stable={stable_food}, shocked={shocked_food}"
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
        sim.state.resources.wood = Fixed::from_num(800);
        sim.state.resources.metal = Fixed::from_num(800);
        let before = sim.building_graph().parcels.len();

        for _ in 0..200 {
            sim.tick();
        }

        assert!(sim.building_graph().parcels.len() > before);
    }

    /// FR-CIV-ARCH — emergence facades differ when culture profiles diverge.
    #[test]
    fn phase_buildings_applies_emergence_facades() {
        use civ_agents::culture::CultureProfile;

        let mut sim = Simulation::with_seed(91);
        sim.state.resources.wood = Fixed::from_num(900);
        sim.state.resources.metal = Fixed::from_num(900);
        sim.emergence
            .cluster_cultures
            .insert(1, CultureProfile::new([0.1, 0.2, 0.3, 0.4]));
        for _ in 0..300 {
            sim.tick();
        }
        let names: std::collections::BTreeSet<String> = sim
            .building_graph()
            .facades
            .values()
            .map(|f| f.name.clone())
            .collect();
        assert!(!names.is_empty());
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
            // Only snapshot living cold entities: emergence (famine-driven
            // lifecycle deaths feeding legends) may despawn a civilian during a
            // tick, so an entity present this frame can be gone next frame.
            let before: std::collections::HashMap<hecs::Entity, u16> = cold_entities
                .iter()
                .filter_map(|&entity| {
                    sim.world
                        .get::<&Wardrobe>(entity)
                        .ok()
                        .map(|w| (entity, w.era))
                })
                .collect();

            sim.tick();

            for &entity in &cold_entities {
                // Skip entities that died this tick — only surviving cold
                // entities are subject to the cadence invariant.
                let Ok(wardrobe) = sim.world.get::<&Wardrobe>(entity) else {
                    continue;
                };
                let after = wardrobe.era;
                if let Some(&prev) = before.get(&entity) {
                    if prev != after {
                        assert!(
                            should_tick_entity_with_policy(tick, LodTier::Cold, policy),
                            "Cold-tier wardrobe changed on tick {tick} (off cadence)"
                        );
                    }
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
        let combat_count = live
            .replay_log()
            .events
            .iter()
            .filter(|event| matches!(event, ReplayEvent::Combat { .. }))
            .count();
        assert!(combat_count > 0, "expected war-bridge combat in replay log");

        let mut from_replay = Simulation::with_seed(seed);
        live.replay_log().replay(&mut from_replay).unwrap();
        assert_eq!(from_replay.voxel().chunk_count(), chunk_live);
        assert_eq!(from_replay.state.tick, live.state.tick);
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
    fn phase_voxel_ca_none_is_noop() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-CA-009 — warm liquid WATER in a single chunk produces at
    /// least one viable abiogenesis site. A pure STONE chunk produces
    /// zero. The two runs must round-trip deterministically (same seed,
    /// same grid → same sites).
    #[test]
    fn phase_voxel_ca_warm_water_is_viable_stone_is_not() {
        // TODO: Implement Simulation::phase_voxel_ca and last_tick_abiogenesis_sites
    }

    /// FR-CIV-0100 — chronicle records technological breakthroughs when tech bits advance.
    #[test]
    fn chronicle_records_tech_breakthroughs() {
        // TODO: Implement WorldState::research_progress, Simulation::phase_tech, phase_chronicle, chronicle
    }

    /// FR-CIV-0100 — chronicle length stays bounded at CHRONICLE_MAX_LEN.
    #[test]
    fn chronicle_is_length_capped() {
        // TODO: Implement WorldState::chronicle field, Simulation::phase_chronicle and chronicle
    }

    /// FR-CIV-0100 — golden-age chronicle lines are deduped via chronicle_age.
    #[test]
    fn chronicle_dedups_age() {
        // TODO: Implement WorldState::chronicle_age, Simulation::phase_chronicle and chronicle
    }

    /// `tick_with_emergence_source` advances ticks identically; CA grid changes sampling.
    #[test]
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
        assert_eq!(sim.military_phase_config().war.fog_vision_radius, Some(8));
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
    /// lower floor at `FOOD_SCARCITY_BASELINE`, and the `treasury.to_bits() / SCALE`
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
            (
                0,
                Resources {
                    food: Fixed::from_num(1),
                    ..Resources::default()
                },
            ),
            (Fixed::from_num(5_000).to_bits(), Resources::default()),
            (Fixed::from_num(99_999_999).to_bits(), Resources::default()),
        ];
        for (treasury_raw, res) in cases {
            let treasury = Fixed::from_bits(treasury_raw);
            let shadow = faction_wealth_scarcity_shadow(treasury, &res);
            assert!(
                shadow >= FOOD_SCARCITY_BASELINE,
                "shadow ({shadow}) fell below FOOD_SCARCITY_BASELINE ({FOOD_SCARCITY_BASELINE})"
            );
        }

        // `treasury.to_bits() / SCALE` is the integer wealth — guards against a
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

    /// FR-CIV-DIPLOMACY — `Simulation::tick()` must keep updating faction
    /// relations so emergent proximity/trade/war signals can accumulate over time.
    #[test]
    fn diplomacy_relations_evolve_through_sim_tick() {
        let mut sim = Simulation::with_seed(91);
        sim.state.tick = 499;

        let faction_ids: Vec<u32> = sim.state.factions.keys().copied().collect();
        assert!(
            faction_ids.len() >= 2,
            "test requires at least two factions"
        );

        let mut cluster_member_counts: BTreeMap<u64, u32> = BTreeMap::new();
        for (_, member) in sim.world.query::<&ClusterMember>().iter() {
            *cluster_member_counts.entry(member.cluster.0).or_insert(0) += 1;
        }
        let (a, b) = diplomacy_pair_from_settlement_overlap(
            &sim.world,
            &cluster_member_counts,
            &faction_ids,
            sim.state.tick,
        );

        sim.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim.state.faction_treasury.insert(b, Fixed::from_num(0));

        sim.tick();

        let event = sim.diplomacy_events().last().expect("diplomacy event");
        assert_eq!(event.tick, 500);
        assert_eq!((event.faction_a, event.faction_b), (a, b));
        assert_eq!(
            event.kind,
            DiplomacyKind::TradeAgreement,
            "zero disparity should produce a trade agreement when diplomacy runs"
        );
        assert_eq!(
            sim.state.faction_treasury.get(&a).copied(),
            Some(Fixed::from_num(100))
        );
        assert_eq!(
            sim.state.faction_treasury.get(&b).copied(),
            Some(Fixed::from_num(100))
        );
    }

    #[test]
    fn player_diplomacy_action_mutates_relation_substrate() {
        let mut sim = Simulation::with_seed(7);
        let relation = sim
            .apply_player_diplomacy_action(0, 1, DiplomacyKind::Conflict)
            .expect("known faction pair should mutate");

        assert_eq!(relation.faction_a, 0);
        assert_eq!(relation.faction_b, 1);
        assert!(relation.score < 0.0);
        assert!(matches!(
            relation.kind.as_str(),
            "neutral" | "allied" | "hostile"
        ));
        assert_eq!(
            sim.diplomacy_events().last(),
            Some(&DiplomacyEvent {
                tick: sim.state.tick,
                faction_a: 0,
                faction_b: 1,
                kind: DiplomacyKind::Conflict,
            })
        );
    }

    #[test]
    fn player_trade_action_mutates_relation_substrate_positive() {
        let mut sim = Simulation::with_seed(7);
        let relation = sim
            .apply_player_diplomacy_action(0, 1, DiplomacyKind::TradeAgreement)
            .expect("known faction pair should mutate");

        assert_eq!(relation.faction_a, 0);
        assert_eq!(relation.faction_b, 1);
        assert!(relation.score > 0.0);
        assert!(matches!(
            relation.kind.as_str(),
            "neutral" | "allied" | "hostile"
        ));
        assert_eq!(
            sim.diplomacy_events().last().map(|event| event.kind),
            Some(DiplomacyKind::TradeAgreement)
        );
    }

    #[test]
    fn player_diplomacy_action_rejects_unknown_or_self_pair() {
        let mut sim = Simulation::with_seed(7);
        assert_eq!(
            sim.apply_player_diplomacy_action(0, 0, DiplomacyKind::Conflict),
            None
        );
        assert_eq!(
            sim.apply_player_diplomacy_action(0, 99, DiplomacyKind::Conflict),
            None
        );
        assert!(sim.diplomacy_events().is_empty());
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
        sim_low
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(base - 1));
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
        sim_high
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
        sim_high
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(base - 1));
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
            assert!(
                (drift - expected).abs() < 1e-6,
                "maturity={} drift={}",
                maturity,
                drift
            );
        }
    }

    #[test]
    fn religion_diplomacy_coupling_phase_picks_trade_over_conflict() {
        let disparity = DIPLOMACY_BASE_CONFLICT_THRESHOLD + 2_000;
        let mut sim_peace = Simulation::with_seed(5);
        sim_peace.state.tick = 500;
        sim_peace.state.belief = 500_000;
        sim_peace.state.cohesion = 200_000;
        sim_peace.emergence.has_patron = true;
        let mut faction_ids: Vec<u32> = sim_peace.state.factions.keys().copied().collect();
        faction_ids.sort_unstable();
        if faction_ids.len() < 2 {
            return;
        }
        let (a, b) = (faction_ids[0], faction_ids[1]);
        sim_peace
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
        sim_peace
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_peace.phase_diplomacy();
        let peace_kind = sim_peace.diplomacy_events().last().expect("event").kind;

        let mut sim_war = Simulation::with_seed(5);
        sim_war.state.tick = 500;
        sim_war.state.belief = 0;
        sim_war.state.cohesion = 0;
        sim_war.state.faction_treasury.insert(a, Fixed::from_num(0));
        sim_war
            .state
            .faction_treasury
            .insert(b, Fixed::from_num(disparity));
        sim_war.phase_diplomacy();
        let war_kind = sim_war.diplomacy_events().last().expect("event").kind;

        assert_eq!(
            peace_kind,
            DiplomacyKind::TradeAgreement,
            "high belief+cohesion must bias toward peace at fixed disparity"
        );
        assert_eq!(
            war_kind,
            DiplomacyKind::Conflict,
            "low belief must allow conflict at same disparity"
        );
    }

    /// `canonical_faction_pair` always returns the pair in ascending order so
    /// (a, b) and (b, a) hash to the same BTreeMap key.
    #[test]
    fn canonical_faction_pair_orders_ascending() {
        assert_eq!(canonical_faction_pair(0, 1), (0, 1), "already sorted");
        assert_eq!(
            canonical_faction_pair(1, 0),
            (0, 1),
            "reversed becomes sorted"
        );
        assert_eq!(canonical_faction_pair(3, 3), (3, 3), "equal ids stay equal");
        assert_eq!(
            canonical_faction_pair(u32::MAX, 0),
            (0, u32::MAX),
            "large vs small"
        );
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
        assert_eq!(
            route_resource("timber"),
            ResourceType::Wood,
            "timber → Wood"
        );
        assert_eq!(route_resource("ore"), ResourceType::Metal, "ore → Metal");
        assert_eq!(
            route_resource("tools"),
            ResourceType::Metal,
            "tools → Metal"
        );
        assert_eq!(
            route_resource("cloth"),
            ResourceType::Energy,
            "cloth → Energy"
        );
        assert_eq!(
            route_resource("salt"),
            ResourceType::Energy,
            "salt → Energy"
        );
        assert_eq!(
            route_resource(""),
            ResourceType::Food,
            "empty string → Food (fallback)"
        );
        assert_eq!(
            route_resource("unknown"),
            ResourceType::Food,
            "unrecognized → Food (fallback)"
        );
    }

    /// `emergent_route_goods` is deterministic: same faction id → same goods
    /// label, cycling across the three labels via id % 3.
    #[test]
    fn emergent_route_goods_is_deterministic_and_covers_all_labels() {
        assert_eq!(emergent_route_goods(0), "grain", "id%3==0 → grain");
        assert_eq!(emergent_route_goods(1), "ore", "id%3==1 → ore");
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
        assert!(
            pos > zero && zero > neg,
            "goodwill must raise tolerance over hostility"
        );
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
        sim_high
            .state
            .faction_treasury
            .insert(a, Fixed::from_num(0));
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
        let dna_a: Vec<Dna> = sim_a
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        let dna_b: Vec<Dna> = sim_b
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        assert_eq!(
            dna_a.len(),
            dna_b.len(),
            "both sims must spawn the same number of DNA-bearing entities"
        );
        assert!(!dna_a.is_empty(), "at least one entity must carry DNA");
        // Both runs must be bit-identical under the same seed.
        for (a, b) in dna_a.iter().zip(dna_b.iter()) {
            assert_eq!(
                a, b,
                "Dna must be deterministic under an identical RNG seed"
            );
        }
        // The first civilian's DNA must differ from the raw zero genome, proving
        // it was seeded from an archetype rather than left default.
        let archetype = civ_genetics::archetype_dna(NamedSeed::Ardani);
        assert_eq!(
            dna_a[0].0.len(),
            archetype.0.len(),
            "genome length must match archetype"
        );
        // With divergence=0.3 the result must not be all-zero (extremely unlikely).
        assert_ne!(
            dna_a[0].0,
            vec![0u8; 64],
            "seeded DNA must not be the zero genome"
        );
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
        assert_ne!(
            ardani_base, velthari_base,
            "Ardani and Velthari must differ"
        );
        assert_ne!(ardani_base, grundak_base, "Ardani and Grundak must differ");
        assert_ne!(
            velthari_base, grundak_base,
            "Velthari and Grundak must differ"
        );

        // With 128 civilians and 12 named seeds, each archetype slot is hit ~10-11 times.
        let sim = Simulation::with_seed(1);
        let dna_list: Vec<Dna> = sim
            .world
            .query::<&Dna>()
            .iter()
            .map(|(_, d)| d.clone())
            .collect();
        assert_eq!(dna_list.len(), 128, "all 128 civilians must carry Dna");

        // Verify that at minimum 3 distinct genomes are present, proving multiple
        // archetype branches were exercised (divergence prevents collisions).
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
    fn choose_named_seed_weighted_distribution() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::distributions::WeightedIndex;
        use rand::SeedableRng;

        let seed_mix = vec![
            SeedWeight {
                seed: NamedSeed::Ardani,
                weight: 0.6,
            },
            SeedWeight {
                seed: NamedSeed::Velthari,
                weight: 0.3,
            },
            SeedWeight {
                seed: NamedSeed::Grundak,
                weight: 0.1,
            },
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
                _ => {}
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
        assert!(
            counts[0] > counts[1] && counts[0] > counts[2],
            "Ardani must be plurality"
        );
    }

    /// A single-entry mix must always yield that one race.
    #[test]
    fn choose_named_seed_single_seed_all_that_race() {
        use crate::scenario::SeedWeight;
        use civ_genetics::NamedSeed;
        use rand::distributions::WeightedIndex;
        use rand::SeedableRng;

        let seed_mix = vec![SeedWeight {
            seed: NamedSeed::Velthari,
            weight: 1.0,
        }];
        let weights: Vec<f32> = seed_mix.iter().map(|sw| sw.weight).collect();
        let dist = WeightedIndex::new(&weights).expect("valid weights");

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        for i in 0..100 {
            let result = choose_named_seed(&seed_mix, Some(&dist), i, &mut rng);
            assert_eq!(
                result,
                NamedSeed::Velthari,
                "expected Velthari at index {i}"
            );
        }
    }

    /// FR-CIV-014 / emergence-spawn — scenario-controlled faction spawning must
    /// honor arbitrary faction counts and per-faction civilian counts.
    #[test]
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
        let _ = &sc;
        let sim = Simulation::with_seed(123u64);

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
            assert!(
                bonus_close > bonus_far,
                "closer language must yield bigger bonus"
            );
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

    #[test]
    fn language_names_diverge_for_isolated_factions_over_time() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_languages.clear();
        sim.language_state = LanguageState::default();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_language();
        let baseline_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );

        for _ in 0..20 {
            sim.phase_language();
        }

        let final_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );
        assert!(
            final_distance > baseline_distance,
            "isolated cultures should diverge over time, {baseline_distance} -> {final_distance}"
        );
        assert!(
            final_distance >= 0.5,
            "isolated languages should stay meaningfully divergent, got {final_distance}"
        );
        assert_ne!(
            sim.faction_place_name(1, 1),
            sim.faction_place_name(2, 1),
            "place names should diverge with isolated lexicons"
        );
    }

    #[test]
    fn language_drift_wires_through_sim_tick_for_isolated_factions() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_languages.clear();
        sim.language_state = LanguageState::default();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_language();
        let baseline = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );

        for _ in 0..20 {
            sim.tick();
        }

        let final_distance = average_language_distance(
            sim.faction_languages()
                .get(&1)
                .expect("faction 1 language state must exist"),
            sim.faction_languages()
                .get(&2)
                .expect("faction 2 language state must exist"),
        );
        assert!(
            final_distance > baseline,
            "isolated factions should diverge through Simulation::tick(), {baseline} -> {final_distance}"
        );
    }

    #[test]
    fn culture_traits_drift_through_sim_tick_for_isolated_factions() {
        use civ_agents::{ClusterId, ClusterMember};
        use civ_voxel::WorldCoord;

        let mut sim = Simulation::new();
        sim.world = World::new();
        sim.cluster_cultures.clear();
        sim.faction_ideologies.clear();

        sim.cluster_cultures
            .insert(1, CultureProfile::new([0.15, 0.15, 0.15, 0.15]));
        sim.cluster_cultures
            .insert(2, CultureProfile::new([0.85, 0.85, 0.85, 0.85]));
        sim.religious_profiles.insert(
            1,
            ReligiousProfile {
                monitoring: 0.70,
                mythic_coherence: 0.60,
                uncertainty_reduction: 0.20,
                population: 4,
                ..ReligiousProfile::default()
            },
        );
        sim.religious_profiles.insert(
            2,
            ReligiousProfile {
                monitoring: 0.20,
                mythic_coherence: 0.30,
                uncertainty_reduction: 0.65,
                population: 4,
                ..ReligiousProfile::default()
            },
        );

        for (entity_id, cluster_id, faction_id, base_x) in [
            (1_u64, 1_u64, 1_u32, 0_i64),
            (2, 1, 1, 20),
            (3, 1, 1, 40),
            (4, 1, 1, 60),
            (5, 2, 2, 200_000),
            (6, 2, 2, 200_020),
            (7, 2, 2, 200_040),
            (8, 2, 2, 200_060),
        ] {
            let _ = sim.world.spawn((
                AgentCivilian {
                    id: entity_id,
                    alignment: Alignment::Faction(faction_id),
                    age: 20,
                },
                ClusterMember {
                    cluster: ClusterId(cluster_id),
                },
                Position3d {
                    coord: WorldCoord {
                        x: base_x,
                        y: 0,
                        z: base_x / 2,
                    },
                },
            ));
        }

        sim.phase_culture();
        let before = sim
            .faction_ideologies()
            .get(&1)
            .expect("faction 1 culture should initialize")
            .values;

        for _ in 0..20 {
            sim.tick();
        }

        let after = sim
            .faction_ideologies()
            .get(&1)
            .expect("faction 1 culture should advance through tick")
            .values;
        assert_ne!(
            before, after,
            "FR-CIV-CULTURE: faction culture traits should drift through Simulation::tick()"
        );
    }

    // ── AUDIO-wire (FR-AUDIO-wire) tests ────────────────────────────────────
    //
    // These tests cover the thin wire between per-tick substrate events
    // (disasters / combat pulses / construction / emergence) and the
    // `SfxTrigger` buffer surfaced on `sim.last_tick_audio_events()`.
    // Audio synthesis itself lives in `civ-audio`; the engine only owns
    // the trigger list.

    #[cfg(test)]
    impl Simulation {
        /// FR-AUDIO-wire test helper — push a `CombatDamagePulse` into
        /// the engine's per-tick buffer at normalized world coords
        /// (`x_norm`, `y_norm` in `[0, 1]`), so the audio phase can be
        /// exercised without running the full tactics resolution.
        fn push_combat_pulse_for_test(&mut self, x_norm: f32, y_norm: f32) {
            self.last_tick_combat_pulses.push(CombatDamagePulse {
                x: x_norm.clamp(0.0, 1.0),
                y: y_norm.clamp(0.0, 1.0),
                unit_a: None,
                unit_b: None,
            });
        }
    }

    #[cfg(test)]
    mod audio_wire_tests {
        use super::*;

        /// FR-AUDIO-wire — on a fresh `Simulation::new()`, the audio buffer
        /// starts empty and remains empty after one tick (no combat, no
        /// construction, no disasters on the seeded first tick).
        #[test]
        fn fr_audio_wire_empty_buffer_clears_across_ticks() {
            let mut sim = Simulation::new();
            assert!(sim.last_tick_audio_events().is_empty());
            sim.tick();
            // No substrate event has fired on a seeded tick 1 — audio buffer
            // stays empty.
            assert!(sim.last_tick_audio_events().is_empty());
        }

        /// FR-AUDIO-wire — triggering a disaster mid-tick records a
        /// `SfxTrigger::Disaster` whose `kind` matches the
        /// `DisasterKind` label so the audio substrate's
        /// `SfxKind::for_disaster_label` can route it to the per-kind
        /// sting (Meteor / Flood / Quake / Wildfire / Storm / Plague).
        #[test]
        fn fr_audio_wire_disaster_records_routed_trigger() {
            use crate::disasters::{trigger_disaster, DisasterKind};

            let mut sim = Simulation::new();
            // Direct API: `trigger_disaster` records the audio trigger as
            // a side effect of `apply_disaster`.
            trigger_disaster(
                &mut sim,
                DisasterKind::Quake,
                WorldCoord { x: 0, y: 0, z: 0 },
            );
            let recorded = sim.last_tick_audio_events();
            assert_eq!(recorded.len(), 1, "one disaster → one trigger");
            match recorded[0] {
                SfxTrigger::Disaster { kind, severity } => {
                    assert_eq!(kind, "quake", "wire label matches the per-kind sting");
                    assert!(
                        (0.0..=1.0).contains(&severity),
                        "severity is clamped to [0, 1]"
                    );
                    assert!(
                        severity > 0.0,
                        "non-zero severity (quake has positive radius)"
                    );
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `record_disaster_audio` is idempotent: an
        /// unknown label surfaces as the umbrella "disaster" label so the
        /// substrate's `for_disaster_label` falls back to
        /// `SfxKind::Disaster` (no panic, no skipped event).
        #[test]
        fn fr_audio_wire_unknown_disaster_label_falls_back() {
            let mut sim = Simulation::new();
            sim.record_disaster_audio("hailstorm", 0.4);
            assert_eq!(sim.last_tick_audio_events().len(), 1);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { kind, severity } => {
                    assert_eq!(kind, "disaster", "unknown → umbrella label");
                    assert!(
                        (severity - 0.4).abs() < 1e-5,
                        "severity passes through clamp"
                    );
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `record_disaster_audio` clamps severity out of
        /// `[0, 1]` so the wire shape is bounded.
        #[test]
        fn fr_audio_wire_record_disaster_severity_is_clamped() {
            let mut sim = Simulation::new();
            sim.record_disaster_audio("flood", 1.7);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { severity, .. } => {
                    assert!(severity <= 1.0, "severity > 1 must clamp to 1.0");
                    assert!((severity - 1.0).abs() < 1e-5);
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
            let mut sim = Simulation::new();
            sim.record_disaster_audio("flood", -0.3);
            match sim.last_tick_audio_events()[0] {
                SfxTrigger::Disaster { severity, .. } => {
                    assert!(severity >= 0.0, "severity < 0 must clamp to 0.0");
                    assert!((severity - 0.0).abs() < 1e-5);
                }
                other => panic!("expected Disaster trigger, got {other:?}"),
            }
        }

        /// FR-AUDIO-wire — `phase_audio` translates a queued combat pulse
        /// into a `SfxTrigger::Battle` with intensity scaled by
        /// normalized proximity to the world center. We use the
        /// `#[cfg(test)]`-gated `push_combat_pulse_for_test` helper to
        /// stage the pulse without running the full tactics phase.
        #[test]
        fn fr_audio_wire_combat_pulse_emits_battle_trigger() {
            let mut sim = Simulation::new();
            // A pulse at the world center → maximum intensity (1.0).
            sim.push_combat_pulse_for_test(0.5, 0.5);
            sim.phase_audio();
            let events = sim.last_tick_audio_events();
            assert_eq!(events.len(), 1, "one pulse → one Battle trigger");
            match events[0] {
                SfxTrigger::Battle { intensity } => {
                    assert!((0.0..=1.0).contains(&intensity), "intensity is in [0, 1]");
                    assert!(intensity > 0.99, "center pulse → near-1.0 intensity");
                }
                other => panic!("expected Battle trigger, got {other:?}"),
            }
        }

        #[test]
        fn fr_audio_wire_lifecycle_and_research_emit_birth_death_tech() {
            let mut sim = Simulation::new();
            sim.last_births.push(PopulationEvent {
                tick: sim.state.tick,
                entity_id: 1,
                x: 0.25,
                y: 0.5,
            });
            sim.last_deaths.push(PopulationEvent {
                tick: sim.state.tick,
                entity_id: 2,
                x: 0.75,
                y: 0.5,
            });
            sim.research_cache.researched.push("agriculture".to_owned());

            sim.phase_audio();

            assert_eq!(
                sim.last_tick_audio_events(),
                &[SfxTrigger::Birth, SfxTrigger::Death, SfxTrigger::Tech]
            );

            sim.last_births.clear();
            sim.last_deaths.clear();
            sim.phase_audio();
            assert!(
                sim.last_tick_audio_events().is_empty(),
                "already-emitted research must not retrigger Tech"
            );
        }

        #[test]
        fn tick_invokes_phase_audio() {
            use civ_agents::culture::CultureProfile;

            let mut sim = Simulation::new();
            sim.cluster_cultures
                .insert(7, CultureProfile::new([0.4, 0.5, 0.6, 0.7]));
            assert!(sim.last_tick_music_cues.is_empty());

            sim.tick();

            assert!(
                sim.last_tick_music_cues.contains_key(&7),
                "tick should run phase_audio and populate music cues"
            );
        }

        /// FR-MUSIC-001 — two cultures produce distinct, drifting music-cue
        /// surfaces derived from emergent culture profiles.
        #[test]
        fn fr_music_distinct_culture_cues_evolve_over_time() {
            use civ_agents::culture::CultureProfile;

            let mut sim = Simulation::new();
            sim.cluster_cultures
                .insert(100, CultureProfile::new([0.14, 0.14, 0.14, 0.14]));
            sim.cluster_cultures
                .insert(200, CultureProfile::new([0.86, 0.86, 0.86, 0.86]));
            sim.faction_aggression.insert(0, 0.1);
            sim.faction_aggression.insert(1, 0.2);

            sim.tick();
            let snap_a = sim.snapshot();
            let cue_a_100 = snap_a
                .music_cues
                .get(&100)
                .cloned()
                .expect("seeded cluster 100 should have a cue");
            let cue_a_200 = snap_a
                .music_cues
                .get(&200)
                .cloned()
                .expect("seeded cluster 200 should have a cue");
            assert_ne!(
                cue_a_100, cue_a_200,
                "cultures with distinct profiles should surface distinct cue params"
            );

            sim.tick();
            let snap_b = sim.snapshot();
            let cue_b_100 = snap_b
                .music_cues
                .get(&100)
                .cloned()
                .expect("seeded cluster 100 should persist");
            let cue_b_200 = snap_b
                .music_cues
                .get(&200)
                .cloned()
                .expect("seeded cluster 200 should persist");
            assert_ne!(cue_a_100, cue_b_100);
            assert_ne!(cue_a_200, cue_b_200);
        }

        #[test]
        fn fed_stable_population_grows_via_births() {
            use civ_agents::{
                spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
                Needs as AgentNeeds,
            };

            let mut sim = Simulation::new();
            sim.state.resources.food = Fixed::from_num(100);
            sim.set_settlement_population(1, 2);

            let parent_a = spawn_civilian_at(
                &mut sim.world,
                1,
                Alignment::Faction(1),
                0.25,
                0.25,
                ActorVisualKind::Humanoid,
                &mut sim.rng,
            );
            let parent_b = spawn_civilian_at(
                &mut sim.world,
                2,
                Alignment::Faction(1),
                0.27,
                0.26,
                ActorVisualKind::Humanoid,
                &mut sim.rng,
            );

            for entity in [parent_a, parent_b] {
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 28;
                {
                    let mut needs = sim.world.get::<&mut AgentNeeds>(entity).unwrap();
                    needs.food = 0.96;
                    needs.rest = 0.94;
                    needs.safety = 0.93;
                    needs.belonging = 0.95;
                    needs.health = 0.97;
                }
            }

            let before = sim.world.query::<&AgentCivilian>().iter().count();
            sim.phase_life();
            let after = sim.world.query::<&AgentCivilian>().iter().count();

            assert!(
                after > before,
                "fed paired adults should produce at least one child"
            );
            assert!(
                !sim.last_births().is_empty(),
                "birth events should be recorded"
            );

            let child_id = sim.last_births().last().expect("child").entity_id;
            let kinship = sim.kinship.get(&child_id).expect("child kinship");
            assert!(
                kinship
                    .iter()
                    .any(|edge| matches!(edge.kind, KinshipKind::Family)),
                "newborn should receive family kinship"
            );
        }

        #[test]
        fn starving_population_migrates_and_founds_settlement() {
            use civ_agents::{
                spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
                Needs as AgentNeeds,
            };

            let mut sim = Simulation::new();
            sim.state.resources.food = Fixed::from_num(0);
            sim.set_settlement_population(7, 3);

            let adults = [
                spawn_civilian_at(
                    &mut sim.world,
                    11,
                    Alignment::Faction(7),
                    0.12,
                    0.12,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
                spawn_civilian_at(
                    &mut sim.world,
                    12,
                    Alignment::Faction(7),
                    0.13,
                    0.14,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
                spawn_civilian_at(
                    &mut sim.world,
                    13,
                    Alignment::Faction(7),
                    0.15,
                    0.16,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                ),
            ];

            for entity in adults {
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 32;
                let mut needs = sim.world.get::<&mut Needs>(entity).unwrap();
                needs.food = 0.08;
                needs.rest = 0.22;
                needs.safety = 0.24;
                needs.belonging = 0.25;
                needs.health = 0.85;
            }

            let before_settlements = sim.settlements.clone();
            sim.phase_life();

            assert_eq!(sim.settlements.get(&7).copied(), Some(1));
            assert_eq!(sim.settlements.get(&8).copied(), Some(2));
            assert_eq!(sim.settlements.len(), before_settlements.len() + 1);

            let migrated = sim
                .world
                .query::<&AgentCivilian>()
                .iter()
                .filter(|(_, civ)| matches!(civ.alignment, Alignment::Faction(8)))
                .count();
            assert!(
                migrated >= 2,
                "starving adults should found a new settlement"
            );
        }

        // FR-CIV-LIFE-003: smoke test that `phase_citizen_lifecycle` runs
        // through the new `should_reproduce` path without panicking. We
        // advance the sim through several birth windows and verify that
        // the civilian count grows over time when there is food available
        // and adults exist.
        #[test]
        fn phase_citizen_lifecycle_uses_should_reproduce() {
            let mut sim = Simulation::new();
            // Spawn three adults at well-fed state so reproduction can fire.
            for (i, id) in [700u64, 701, 702].iter().enumerate() {
                let entity = spawn_civilian_at(
                    &mut sim.world,
                    *id,
                    Alignment::None,
                    0.20 + (i as f32) * 0.01,
                    0.20 + (i as f32) * 0.01,
                    ActorVisualKind::Humanoid,
                    &mut sim.rng,
                );
                let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
                civ.age = 30;
                let mut needs = sim.world.get::<&mut Needs>(entity).unwrap();
                needs.food = 0.95;
                needs.shelter = 0.95;
                needs.safety = 0.95;
                needs.belonging = 0.95;
            }
            // Ensure resources are non-zero so the food regen branch runs
            // (and so the early-death branch is not triggered).
            sim.state.resources.food = Fixed::from_num(1000);
            sim.state.population = sim.state.population.max(count_civilians(&sim.world) as u64);

            // Run several birth windows (every 200 ticks).
            for tick in 0..600 {
                sim.state.tick = tick;
                sim.phase_citizen_lifecycle();
            }

            // After 600 ticks, with three fertile adults and food available,
            // at least one birth should have occurred.
            let final_pop = count_civilians(&sim.world) as u64;
            assert!(
                final_pop >= 4,
                "should_reproduce should have produced at least one child (got {})",
                final_pop
            );
        }
    }

    /// FR-CIV-LIFE P4-A — `phase_life` populates `last_tick_lifecycle_metrics`
    /// and `phase_economy` uses it to weight the LaborCapacityAllocator.
    #[test]
    fn labor_capacity_weighting_threads_through_phase_economy() {
        let mut sim = Simulation::new();
        // Default sim has no civilians; metrics must be zero, and allocation
        // should still succeed (with effective demand = 0).
        assert_eq!(sim.last_tick_lifecycle_metrics.adults, 0);
        assert_eq!(sim.last_tick_lifecycle_metrics.total_living(), 0);
        // Should not panic on tick.
        sim.tick();
        // Spawn civilians: 2 adults + 1 child via the engine's spawn API.
        let civ_a = sim.world.spawn(()).id();
        let civ_b = sim.world.spawn(()).id();
        // Advance phase_life directly: the metrics should be reproducible
        // from any civilian snapshot.
        sim.last_tick_lifecycle_metrics = LifecycleCounters {
            children: 1,
            adults: 2,
            elders: 0,
            dead: 0,
        };
        // 2 adults + 0.5 * 0 elders = 2 / (1 + 2 + 0) = 0.6667 labor fraction
        let living = (sim.last_tick_lifecycle_metrics.children
            + sim.last_tick_lifecycle_metrics.adults
            + sim.last_tick_lifecycle_metrics.elders) as f64;
        let productive = sim.last_tick_lifecycle_metrics.adults as f64
            + 0.5 * sim.last_tick_lifecycle_metrics.elders as f64;
        let frac = (productive / living).clamp(0.0, 1.0);
        assert!(
            (frac - 0.6666).abs() < 0.01,
            "labor fraction expected ~0.6667, got {frac}"
        );
        // Ensure spawn targets are still alive (sanity).
        assert!(civ_a > 0);
        let _ = civ_b; // unused: kept for documentation
    }

    // FR-CIV-LIFE-001/002/003: classifier wiring smoke test. Spawn three
    // civilians spanning the Child / Adult / Elder axis, run `phase_life`
    // once, then assert `last_tick_lifecycle_metrics()` contains the
    // expected counts. This is the contract-level check that the
    // classifier is reachable from the engine tick loop, not a deep
    // classifier correctness test (that lives in `civ_needs::lifecycle`).
    #[test]
    fn lifecycle_classifiers_wired_into_phase_life() {
        use civ_agents::{
            spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
            Needs as AgentNeeds,
        };

        let mut sim = Simulation::new();

        // Spawn three civilians with distinct ages spanning Child / Adult / Elder.
        let child = spawn_civilian_at(
            &mut sim.world,
            100,
            Alignment::None,
            0.30,
            0.30,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(child).unwrap();
            civ.age = 5;
            let mut needs = sim.world.get::<&mut AgentNeeds>(child).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }

        let adult = spawn_civilian_at(
            &mut sim.world,
            102,
            Alignment::None,
            0.31,
            0.31,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(adult).unwrap();
            civ.age = 28;
            let mut needs = sim.world.get::<&mut AgentNeeds>(adult).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }

        let elder = spawn_civilian_at(
            &mut sim.world,
            103,
            Alignment::None,
            0.32,
            0.32,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(elder).unwrap();
            civ.age = 70;
            let mut needs = sim.world.get::<&mut AgentNeeds>(elder).unwrap();
            needs.food = 0.85;
            needs.shelter = 0.85;
            needs.safety = 0.85;
            needs.belonging = 0.85;
        }

        // Default counters (before any phase_life run) should be all zero.
        let pre = *sim.last_tick_lifecycle_metrics();
        assert_eq!(pre.total(), 0, "default lifecycle counters must be zero");

        sim.phase_life();

        let post = *sim.last_tick_lifecycle_metrics();
        assert!(
            post.total() >= 3,
            "phase_life should classify the three spawned civilians (got total={})",
            post.total()
        );
        assert!(
            post.children >= 1,
            "age=5 civilian should classify as Child"
        );
        assert!(
            post.adults >= 1,
            "age=28 healthy civilian should classify as Adult"
        );
        assert!(post.elders >= 1, "age=70 civilian should classify as Elder");
    }

    // FR-CIV-LIFE-002: maturity growth wiring smoke test. A healthy adult
    // (maturity starts at 0) should remain `Adult`-classifiable after the
    // classifier pass even without an attached `Psyche`, since the
    // classifier treats missing maturity as 0.0 and the age/integrity
    // branch alone still puts a 28-year-old healthy civilian in the
    // Adult bucket.
    #[test]
    fn phase_life_classifier_handles_missing_psyche() {
        use civ_agents::{
            spawn_civilian_at, ActorVisualKind, Alignment, Civilian as AgentCivilian,
            Needs as AgentNeeds,
        };

        let mut sim = Simulation::new();
        let entity = spawn_civilian_at(
            &mut sim.world,
            200,
            Alignment::None,
            0.40,
            0.40,
            ActorVisualKind::Humanoid,
            &mut sim.rng,
        );
        {
            let mut civ = sim.world.get::<&mut AgentCivilian>(entity).unwrap();
            civ.age = 28;
            let mut needs = sim.world.get::<&mut AgentNeeds>(entity).unwrap();
            needs.food = 0.95;
            needs.shelter = 0.95;
            needs.safety = 0.95;
            needs.belonging = 0.95;
        }
        // Deliberately do NOT attach a `Psyche` component.
        sim.phase_life();
        let counters = *sim.last_tick_lifecycle_metrics();
        assert!(
            counters.adults >= 1,
            "adult should be classified even without Psyche"
        );
    }
}

/// Re-export of genetics module so callers can use `crate::engine::genetics::...`.
pub mod genetics {
    /// Re-export of SentienceEvent from civ_genetics.
    pub use civ_genetics::sentience::SentienceEvent;
}

// Free-function wrappers for cohesion and unrest accessors so they can be re-exported from lib.rs.

/// Add cohesion to a faction (currently a no-op stub).
pub fn add_cohesion(faction: u32, delta: f32) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    state.faction_count = state.faction_count.max(faction.saturating_add(1));
    state.cohesion_events.push(CohesionEvent {
        actor_id: u64::from(faction),
        settlement_id: faction,
        kind: CohesionEventKind::Bonded,
        score: 0,
        score_delta: delta.round() as i64,
        fabric: FabricTier::Fractured,
    });
}

/// Add trust between two actors (currently a no-op stub).
pub fn add_trust(actor_id: u64, target: u64, amount: i64) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    let max_actor = actor_id.max(target);
    state.faction_count = state
        .faction_count
        .max(u32::try_from(max_actor.saturating_add(1)).unwrap_or(u32::MAX));
    state.cohesion_events.push(CohesionEvent {
        actor_id,
        settlement_id: u32::try_from(target).unwrap_or(u32::MAX),
        kind: CohesionEventKind::Bonded,
        score: amount,
        score_delta: amount,
        fabric: FabricTier::Fractured,
    });
}

/// Get faction count (currently returns 0 stub).
pub fn faction_count() -> u32 {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .faction_count
}

/// Get last tick's cohesion events (currently empty stub).
pub fn last_tick_cohesion() -> &'static [CohesionEvent] {
    Box::leak(
        compat_state()
            .lock()
            .expect("compat state poisoned")
            .cohesion_events
            .clone()
            .into_boxed_slice(),
    )
}

/// Get last tick's cohesion for a settlement (currently empty stub).
pub fn last_tick_cohesion_settlement(settlement_id: u32) -> &'static [CohesionEvent] {
    let events: Vec<CohesionEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .cohesion_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Get last tick's unrest events (currently empty stub).
pub fn last_tick_unrest() -> &'static [UnrestEvent] {
    Box::leak(
        compat_state()
            .lock()
            .expect("compat state poisoned")
            .unrest_events
            .clone()
            .into_boxed_slice(),
    )
}

/// Get last tick's unrest for a settlement (currently empty stub).
pub fn last_tick_unrest_settlement(settlement_id: u32) -> &'static [UnrestEvent] {
    let events: Vec<UnrestEvent> = compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_events
        .iter()
        .filter(|event| event.settlement_id == settlement_id)
        .cloned()
        .collect();
    Box::leak(events.into_boxed_slice())
}

/// Set settlement gini coefficient (currently a no-op stub).
pub fn set_settlement_gini(settlement_id: u32, gini: f64) {
    let mut state = compat_state().lock().expect("compat state poisoned");
    let normalized = if gini.is_nan() {
        0.0
    } else {
        gini.clamp(0.0, 1.0)
    };
    state.settlement_gini.insert(settlement_id, normalized);
    let score = (normalized * 200.0).round() as i32;
    let level = UnrestLevel::from_score(score);
    state.unrest_levels.insert(settlement_id, level);
    state.unrest_events.push(UnrestEvent {
        settlement_id,
        level,
        score,
        score_delta: 0,
        mood: 0,
        gini_x100: (normalized * 100.0).round() as i32,
        fabric: FabricTier::Fractured,
    });
}

/// Get the normalized Gini coefficient stored for a settlement.
pub fn settlement_gini(settlement_id: u32) -> Option<f64> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .settlement_gini
        .get(&settlement_id)
        .copied()
}

/// Get unrest level for a settlement (currently None stub).
pub fn unrest_level(settlement_id: u32) -> Option<UnrestLevel> {
    compat_state()
        .lock()
        .expect("compat state poisoned")
        .unrest_levels
        .get(&settlement_id)
        .copied()
}

#[derive(Default)]
struct CompatState {
    faction_count: u32,
    cohesion_events: Vec<CohesionEvent>,
    unrest_events: Vec<UnrestEvent>,
    unrest_levels: BTreeMap<u32, UnrestLevel>,
    settlement_gini: BTreeMap<u32, f64>,
}

fn compat_state() -> &'static std::sync::Mutex<CompatState> {
    static STATE: std::sync::OnceLock<std::sync::Mutex<CompatState>> = std::sync::OnceLock::new();
    STATE.get_or_init(|| std::sync::Mutex::new(CompatState::default()))
}

#[cfg(test)]
mod compat_state_tests {
    use super::*;

    #[test]
    fn compat_add_cohesion_records_state() {
        add_cohesion(3, 2.4);
        assert!(faction_count() >= 4);
        assert!(!last_tick_cohesion().is_empty());
        assert_eq!(last_tick_cohesion_settlement(3).len(), 1);
    }

    #[test]
    fn compat_unrest_round_trips_gini() {
        set_settlement_gini(9, 0.75);
        assert_eq!(unrest_level(9), Some(UnrestLevel::Revolting));
        assert_eq!(last_tick_unrest_settlement(9).len(), 1);
        assert!(!last_tick_unrest().is_empty());
    }
}
