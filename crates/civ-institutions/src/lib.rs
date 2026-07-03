//! `civ-institutions` - civic institutions for the Civis simulation.
//!
//! This crate provides the minimal civic-institution types consumed by
//! `civ-engine`: the institution kind/record types, population thresholds,
//! and the supporting legitimacy/faction-split helpers.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod faction_split;
pub mod legitimacy;

use serde::{Deserialize, Serialize};

/// Temple institution - religious / civic center.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// Religious / civic center.
    Temple,
    /// Military / guard post.
    Garrison,
}

impl InstitutionKind {
    /// Total number of institution kinds currently modeled.
    pub const COUNT: usize = 2;

    /// Returns the index of this kind in a stable, sorted iteration order.
    pub fn index(self) -> usize {
        match self {
            InstitutionKind::Temple => 0,
            InstitutionKind::Garrison => 1,
        }
    }

    /// Returns the human-readable name of this institution kind.
    pub fn as_str(self) -> &'static str {
        match self {
            InstitutionKind::Temple => "Temple",
            InstitutionKind::Garrison => "Garrison",
        }
    }
}

/// A persisted civic institution record for a single settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Institution {
    /// Which kind of institution this record represents.
    pub kind: InstitutionKind,
    /// Current level. `1` = spawned (L1), `2` = first upgrade (L2).
    pub level: u8,
}

/// Population threshold at which a settlement unlocks a Temple.
pub const TEMPLE_UNLOCK_POPULATION: u32 = 50;

/// Population threshold at which a Temple upgrades from L1 to L2.
pub const TEMPLE_L2_POPULATION: u32 = 200;

/// Population threshold at which a settlement unlocks a Garrison.
pub const GARRISON_UNLOCK_POPULATION: u32 = 120;

/// Population threshold at which a Garrison upgrades from L1 to L2.
pub const GARRISON_L2_POPULATION: u32 = 400;

pub use faction_split::{
    maybe_split_faction, Faction, FactionSplitEvent, InstitutionCohesion, DEFAULT_COHESION_THRESHOLD,
    MAX_COHESION, MIN_COHESION,
};
pub use legitimacy::{
    GovernanceOutcome, InstitutionLegitimacy, DEFAULT_LEGITIMACY, LEGITIMACY_COLLAPSE_THRESHOLD,
    MAX_LEGITIMACY, MIN_LEGITIMACY,
};
