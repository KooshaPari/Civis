//! `civ-institutions` — civic institutions for the Civis simulation.
//!
//! A **civic institution** is a population-gated, per-settlement social
//! construct (Temple, Garrison, future Council / Court / School). Institutions
//! spawn when a settlement's population crosses an unlock threshold and
//! upgrade when it crosses a higher threshold.
//!
//! ## Population thresholds
//!
//! Each [`InstitutionKind`] defines two thresholds:
//!
//! - **Unlock** (L1 spawn): the smallest population at which the institution
//!   first appears.
//! - **L2 upgrade**: the population at which the institution upgrades from
//!   level 1 → level 2.
//!
//! Thresholds are exported as `pub const` so the engine's
//! [`phase_institutions`](https://docs.rs/civ_engine) logic, the
//! `civ-server` ws_bridge, and the Bevy reference client can all agree on
//! the exact cut-offs without code duplication.
//!
//! ## One-shot event semantics
//!
//! The owning engine is expected to track, for each
//! `(settlement_id, kind, level)` triple, whether it has already emitted an
//! `InstitutionEvent` for that triple. This guarantees that transient
//! population dips (settlement drops below the unlock threshold for one
//! tick) do not produce duplicate spawn events, and that L1 → L2 upgrades
//! emit exactly once.
//!
//! ## Spec coverage
//!
//! - **FR-CIV-GOV-001**: Temple spawns when a settlement crosses
//!   [`TEMPLE_UNLOCK_POPULATION`]; Garrison spawns when a settlement
//!   crosses [`GARRISON_UNLOCK_POPULATION`].
//! - **FR-CIV-GOV-002**: Civic events stream is exposed read-only via
//!   [`civ_engine::Simulation::last_tick_institution_events`].
//! - **FR-CIV-GOV-003**: L1 → L2 upgrade fires when a settlement crosses
//!   [`TEMPLE_L2_POPULATION`] (resp. [`GARRISON_L2_POPULATION`]) and is
//!   one-shot per `(settlement_id, kind, level)`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod legitimacy;

pub use legitimacy::{
    GovernanceOutcome, InstitutionLegitimacy, DEFAULT_LEGITIMACY, LEGITIMACY_COLLAPSE_THRESHOLD,
    MAX_LEGITIMACY, MIN_LEGITIMACY,
};

use serde::{Deserialize, Serialize};

/// Temple institution — religious / civic center. Spawns when a settlement
/// grows large enough to support a permanent religious functionary.
///
/// Used by:
/// - `civ_engine::Simulation::phase_institutions` — emits the `Spawned` event
/// - `civ-server` ws_bridge — surfaces to the Bevy client
/// - Religion / mood research modules — pulls belief signals from this
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// Religious / civic center.
    Temple,
    /// Military / guard post.
    Garrison,
}

impl InstitutionKind {
    /// Total number of institution kinds currently modeled. Useful for
    /// pre-allocating capacity in `BTreeMap` lookups.
    pub const COUNT: usize = 2;

    /// Returns the index of this kind in a stable, sorted iteration order.
    /// Index 0 = `Temple`, index 1 = `Garrison`.
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

/// A persisted civic institution record for a single settlement. There is at
/// most **one** active institution record per `(settlement_id, kind)` pair,
/// tracked at the highest level the settlement has ever reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Institution {
    /// Which kind of institution this record represents.
    pub kind: InstitutionKind,
    /// Current level. `1` = spawned (L1), `2` = first upgrade (L2). Higher
    /// levels may be added in future specs.
    pub level: u8,
}

/// Population threshold at which a settlement unlocks (spawns) a Temple.
/// Settlements below this population have no Temple.
pub const TEMPLE_UNLOCK_POPULATION: u32 = 50;

/// Population threshold at which a Temple upgrades from L1 to L2.
pub const TEMPLE_L2_POPULATION: u32 = 200;

/// Population threshold at which a settlement unlocks (spawns) a Garrison.
/// Settlements below this population have no Garrison.
pub const GARRISON_UNLOCK_POPULATION: u32 = 120;

/// Population threshold at which a Garrison upgrades from L1 to L2.
pub const GARRISON_L2_POPULATION: u32 = 400;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_kind_index_is_stable() {
        assert_eq!(InstitutionKind::Temple.index(), 0);
        assert_eq!(InstitutionKind::Garrison.index(), 1);
    }

    #[test]
    fn institution_kind_count_is_2() {
        assert_eq!(InstitutionKind::COUNT, 2);
    }

    #[test]
    fn thresholds_are_strictly_ordered() {
        // Unlock must be smaller than L2 upgrade so the L1 phase can
        // exist for some population range.
        assert!(TEMPLE_UNLOCK_POPULATION < TEMPLE_L2_POPULATION);
        assert!(GARRISON_UNLOCK_POPULATION < GARRISON_L2_POPULATION);
    }

    #[test]
    fn temple_unlock_lower_than_garrison() {
        // Temple is a smaller civil investment than Garrison, so a
        // settlement should reach Temple first.
        assert!(TEMPLE_UNLOCK_POPULATION < GARRISON_UNLOCK_POPULATION);
    }
}
