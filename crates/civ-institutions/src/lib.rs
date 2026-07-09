pub mod faction_split;
pub mod legitimacy;

use serde::{Deserialize, Serialize};

/// Kind of a civic institution in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// A temple — seat of religion, culture, and belief.
    Temple,
    /// A garrison — seat of military power and defense.
    Garrison,
}
