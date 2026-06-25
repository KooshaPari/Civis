pub mod religion;
pub mod demographics;

pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics,
};
// FR-AUDIO-wire: re-export the audio substrate's SFX trigger enum so
// downstream crates (civ-server JSON-RPC + WS bridge) can name it as
// `civ_engine::SfxTrigger` without taking a direct `civ-audio` dep.
pub use civ_audio::triggers::SfxTrigger;
