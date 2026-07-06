pub mod faction_split;
pub mod legitimacy;

use serde::{Deserialize, Serialize};

/// Population threshold at which a settlement can spawn a Temple (FR-CIV-GOV-001).
pub const TEMPLE_UNLOCK_POPULATION: u32 = 100;
/// Population threshold at which a Temple upgrades to L2 (FR-CIV-GOV-003).
pub const TEMPLE_L2_POPULATION: u32 = 500;
/// Population threshold at which a settlement can spawn a Garrison (FR-CIV-GOV-001).
pub const GARRISON_UNLOCK_POPULATION: u32 = 200;
/// Population threshold at which a Garrison upgrades to L2 (FR-CIV-GOV-003).
pub const GARRISON_L2_POPULATION: u32 = 800;

/// Kind of a civic institution in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InstitutionKind {
    /// A temple — seat of religion, culture, and belief.
    #[default]
    Temple,
    /// A garrison — seat of military power and defense.
    Garrison,
}

/// Per-settlement civic institution record (FR-CIV-GOV-001).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Institution {
    /// Settlement id this institution belongs to.
    pub settlement_id: u32,
    /// Institution kind.
    pub kind: InstitutionKind,
    /// Current level (1 = L1 / first spawn, 2 = L2 / first upgrade, ...).
    pub level: u8,
    /// Tick the institution was last upgraded.
    pub last_upgrade_tick: u64,
}

impl Institution {
    /// Build a fresh institution at `level` for `settlement_id` + `kind`.
    #[must_use]
    pub fn new(settlement_id: u32, kind: InstitutionKind, level: u8) -> Self {
        Self {
            settlement_id,
            kind,
            level,
            last_upgrade_tick: 0,
        }
    }
}
