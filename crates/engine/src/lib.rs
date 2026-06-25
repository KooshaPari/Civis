pub mod religion;
pub mod demographics;

pub use religion::{emerge_belief, spread_religion, Belief, BeliefConcept, Religion};
pub use demographics::{
    carrying_capacity_from_food, tick_demographics, total_population, AgeGroup, Demographics,
};

// FR-CIV-GOV-001/002/003 (civ-007 institutions epic). Re-exported so callers
// (server, clients, tests) can `use civ_engine::InstitutionKind` etc. without
// pulling the `civ-institutions` crate directly.
pub use civ_institutions::{
    Institution, InstitutionEvent, InstitutionKind, GARRISON_UNLOCK_POPULATION,
    TEMPLE_UNLOCK_POPULATION, TEMPLE_TO_GARRISON_RATIO,
};
