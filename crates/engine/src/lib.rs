pub mod religion;
pub mod demographics;

pub mod audio;
pub mod command_queue;
pub mod ca_budget;
pub mod conditions;
pub mod engine;
pub mod faction_emergence;
pub mod hash_chain;
pub mod integrity;
pub mod invariants;
pub mod io;
pub mod lod;
pub mod metrics;
pub mod policy;
pub mod replay;
pub mod replay_format;
pub mod scenario;
pub mod spawn;
pub mod spectator;

pub use audio::{audio_config_for_era, era_from_tech_level, EraAudioConfig, GameEra};
pub use conditions::{check_outcome, GameOutcome};
pub use ca_budget::{CaTickBudget, StepOutcome, step_with_budget};
pub use engine::{
    Building, BuildingType, Citizen, CombatDamagePulse, DiplomacyEvent, DiplomacyKind, JobType,
    MilitaryUnit, PopulationEvent, Position, Production, ResourceType, Resources, Simulation,
    SimulationSnapshot, UnitType, WorldState,
};
