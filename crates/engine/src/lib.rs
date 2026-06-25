pub mod audio;
pub mod command_queue;
pub mod conditions;
pub mod daily_path;
pub mod demographics;
pub mod disasters;
pub mod emergence;
pub mod emergence_metrics;
pub mod engine;
pub mod era;
pub mod faction_emergence;
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
    awakening_belief_gain, awakening_cohesion_gain, grid_to_norm, spawn, Building, BuildingType,
    CombatDamagePulse, DiplomacyKind, Fixed, MilitaryUnit, ModGuestStateSave, Position, ReplayLog,
    Simulation, UnitType, WorldState,
};
pub use replay::ReplayError;
pub use spawn::norm_to_grid;
