#![allow(missing_docs)]

use crate::{
    Hotkey, PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRequestKind, PowerTab,
    PowerTargetMask,
};

/// The registry of all known god-tool verbs. Phase 1 ships a
/// static catalog (see [`crate::default_powers`]); Phase 5 will
/// add a mod-registration path through `civ_register_power`.
#[derive(Debug, Clone, Copy)]
pub struct PowerRegistry {
    defs: &'static [PowerDef],
}

impl PowerRegistry {
    /// Construct a registry over a fixed catalog.
    pub const fn new(defs: &'static [PowerDef]) -> Self {
        Self { defs }
    }

    /// Borrow the catalog.
    #[must_use]
    pub const fn defs(self) -> &'static [PowerDef] {
        self.defs
    }

    /// Look up a power by id.
    #[must_use]
    pub fn find(self, id: &str) -> Option<&'static PowerDef> {
        self.defs.iter().find(|p| p.id.as_str() == id)
    }

    /// Number of registered powers.
    #[must_use]
    pub const fn len(self) -> usize {
        self.defs.len()
    }

    /// `true` when the catalog is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.defs.is_empty()
    }

    /// Construct a registry from an iterator of power definitions.
    pub fn from_iter<I>(iter: I) -> Result<Self, PowerRegistrationError>
    where
        I: IntoIterator<Item = PowerDef>,
    {
        let defs: Vec<PowerDef> = iter.into_iter().collect();
        for power in &defs {
            for forbidden in FORBIDDEN_TARGET_FIELDS {
                if power.coupling_note.contains(forbidden) {
                    return Err(PowerRegistrationError::ForbiddenField(forbidden));
                }
            }
        }
        // Check for duplicate IDs
        for (i, power) in defs.iter().enumerate() {
            for other in &defs[i + 1..] {
                if power.id == other.id {
                    return Err(PowerRegistrationError::DuplicateId(power.id));
                }
            }
        }
        // Convert Vec to &'static - this requires unsafe or a Box leak
        let leaked: &'static [PowerDef] = Box::leak(defs.into_boxed_slice());
        Ok(Self { defs: leaked })
    }
}

/// Field names that no `PowerDef` or substrate handler is allowed
/// to write. Enforced by the AC-CPL-3 compile-time guard
/// (`docs/design/GODTOOLS_IMPL_PLAN.md` §6.1). The `NoOp` and
/// `Time` request kinds are substrate-exempt; all others must
/// route through a substrate-owned mutation that doesn't touch
/// these fields.
pub const FORBIDDEN_TARGET_FIELDS: &[&str] = &[
    "culture",
    "religion",
    "ideology",
    "alignment",
    "job",
    "faction_id",
    "mood",
    "happiness",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerRegistrationError {
    ForbiddenField(&'static str),
    DuplicateId(PowerId),
}

impl std::fmt::Display for PowerRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerRegistrationError::ForbiddenField(field) => {
                write!(f, "PowerDef writes forbidden field `{field}` (AC-CPL-3)")
            }
            PowerRegistrationError::DuplicateId(id) => {
                write!(f, "duplicate power id: {}", id.as_str())
            }
        }
    }
}

impl std::error::Error for PowerRegistrationError {}

/// Create a default set of power definitions (Phase 1 catalog).
pub fn default_powers() -> Vec<PowerDef> {
    vec![
        PowerDef {
            id: PowerId::new("terrain.raise"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Raise",
            glyph: "terrain_raise",
            hotkey: Some(Hotkey::new(b'W')),
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes VoxelWorld<MaterialId>; CA settles gravity/fluid",
            availability: PowerAvailability::Near,
        },
    ]
}

/// Create the default registry from Phase 1 power definitions.
pub fn default_registry() -> Result<PowerRegistry, PowerRegistrationError> {
    PowerRegistry::from_iter(default_powers())
}
