pub mod audio;
pub mod building_layouts;
pub mod command_queue;
pub mod conditions;
pub mod culture;
pub mod daily_path;
pub mod demographics;
pub mod disasters;
pub mod emergence;
pub mod emergence_metrics;
pub mod engine;
pub mod era;
pub mod history;
pub mod tech;
pub mod faction_emergence;
pub mod faction_decisions;
pub mod godtools;
pub mod hash_chain;
pub mod integrity;
pub mod invariants;
pub mod io;
pub mod language;
pub mod lod;
pub mod metrics;
pub mod perf;
pub mod policy;
pub mod psyche_behavior;
pub mod religion;
pub mod replay;
pub mod replay_format;
pub mod save;
pub mod save_bundle;
pub mod scenario;
pub mod spawn;
pub mod spectator;

/// Fixed-point scaling factor (1 raw unit = SCALE joules). Engine energy
/// quantities are stored in fixed-point `i64` for determinism and converted
/// to `f64`/SI at the economy boundary using this constant.
pub const SCALE: i64 = 1_000;

pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics,
};
// FR-AUDIO-wire: re-export the audio substrate's SFX trigger enum so
// downstream crates (civ-server JSON-RPC + WS bridge) can name it as
// `civ_engine::SfxTrigger` without taking a direct `civ-audio` dep.
pub use civ_audio::triggers::SfxTrigger;
pub use emergence::{
    CivAiDecision, EmergenceFeedEvent, EmergenceState,
};
pub use emergence_metrics::{
    BranchingRegime, EmergenceBranchingState, EmergenceSample,
};
pub use engine::{
    awakening_belief_gain, awakening_cohesion_gain, Building, BuildingType,
    CombatDamagePulse, DiplomacyKind, EconomicFocus, EconomicFocusEvent, FactionRelationSnapshot, Fixed, MilitaryUnit, Position,
    Simulation, UnitType, WorldState,
};

// Re-export of `grid_to_norm` and `spawn()` so callers can name them without
// pulling the private `spawn` module path.
pub use crate::spawn::{grid_to_norm, military_pin_id, spawn_military_at, unit_type_label};

// `ModGuestStateSave` lives in the `civ-mod-host` crate. Re-exported here
// so engine consumers (save_bundle, scenario) can `use civ_engine::ModGuestStateSave`
// without adding a direct `civ-mod-host` dependency.
pub use civ_mod_host::ModGuestStateSave;

// `ReplayLog` is declared `pub` in `crate::replay`. Re-exported here so
// callers can `use civ_engine::ReplayLog` without importing the private
// `crate::replay` module.
pub use crate::replay::ReplayLog;

// FR-CIV-ARCH: Emergent building layouts re-export so callers can use
// `civ_engine::EmergentLayout` and `civ_engine::LayoutStrategy` without
// directly depending on the private `building_layouts` module.
pub use building_layouts::{
    EmergentLayout, LayoutStrategy,
};
pub use era::{CivAge, CivEra, EraProgressionState, FactionEraSnapshot};
pub use history::{EraHistory, EraTransition};
pub use tech::{FactionEmergenceInputs, FactionTechState};
pub use replay::ReplayError;
pub use spawn::norm_to_grid;

// FR-CIV-GOV-001/002/003 (civ-007 institutions epic). Re-exported so callers
// (server, clients, tests) can `use civ_engine::InstitutionKind` etc. without
// pulling the `civ-institutions` crate directly.
pub use civ_institutions::{
    Institution, InstitutionKind, GARRISON_UNLOCK_POPULATION,
    TEMPLE_UNLOCK_POPULATION,
};

/// Per-settlement institution event emitted by [`crate::Simulation::phase_institutions`].
///
/// Local mirror of the engine's internal type so callers can name it without
/// pulling the engine module path directly.
pub use crate::engine::InstitutionEvent;

// FR-CIV-GOV-100 (civ-007 social-mood epic). Re-exported so callers can name
// the snapshot type as `civ_engine::MoodSnapshot` and the saturation /
// history-cap constants as `civ_engine::MOOD_*` without taking a dependency
// on the private `engine` module path.
pub use engine::{
    MoodSnapshot, MOOD_CRIME_BASE, MOOD_HISTORY_CAP, MOOD_MAX, MOOD_MIN,
};

// FR-CIV-GOV-030 (civ-007 cohesion epic). Re-exported so callers
// (server, clients, tests) can name the cohesion types as `civ_engine::KinshipEdge`
// etc. without pulling the private `engine` module path.
pub use engine::{
    add_cohesion, add_trust, faction_count, last_tick_cohesion, last_tick_cohesion_snapshot,
    CohesionCause, CohesionEdge, CohesionEvent, CohesionEventKind, CohesionKind, CohesionSnapshot,
    FabricTier, KinshipEdge, KinshipKind,
};

// FR-CIV-UNREST-001 (civ-007 unrest sub-epic). Re-exported so callers
// can name the unrest types as `civ_engine::UnrestEvent` etc.
// without pulling the private `engine` module path.
pub use engine::{
    last_tick_unrest, last_tick_unrest_settlement, set_settlement_gini, unrest_level,
    UnrestEvent, UnrestEventKind, UnrestLevel, UnrestSnapshot,
};

// FR-CIV-RELIGION (religion §7 wiring). Re-exported so callers
// (server, clients, tests) can name `ReligiousProfile`, `SubstrateGradients`,
// `ReligionEvent`, and the `apply_big_gods_response` /
// `substrate_gradients_for` / `last_religion_sample` accessors without
// pulling the `religion` module path.
pub use religion::{
    apply_big_gods_response, last_religion_sample, substrate_gradients_for, ReligionEvent,
    ReligiousProfile, SubstrateGradients,
};

// FR-CIV-PSYCHE (psyche-driven behavior gap). Re-exported so callers
// (server, tests, CLI) can use `behavior_from_psyche` and `EmotionDrivenBehavior`
// without pulling the `psyche_behavior` module path directly.
pub use psyche_behavior::{behavior_from_psyche, EmotionDrivenBehavior};
