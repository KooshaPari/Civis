pub mod religion;
pub mod demographics;
pub mod audio;

pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
pub use audio::{audio_config_for_era, era_from_tech_level, EraAudioConfig, GameEra};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics,
};
