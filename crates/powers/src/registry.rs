#![allow(missing_docs)]

use crate::{
    Hotkey, PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRequestKind, PowerTab,
    PowerTargetMask,
};

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

#[derive(Debug, Clone, Default)]
pub struct PowerRegistry {
    powers: Vec<PowerDef>,
}

impl PowerRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self { powers: Vec::new() }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.powers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.powers.is_empty()
    }

    #[must_use]
    pub fn powers(&self) -> &[PowerDef] {
        &self.powers
    }

    #[must_use]
    pub fn find(&self, id: &PowerId) -> Option<&PowerDef> {
        self.powers.iter().find(|p| p.id == *id)
    }

    pub fn register(&mut self, power: PowerDef) -> Result<(), PowerRegistrationError> {
        for forbidden in FORBIDDEN_TARGET_FIELDS {
            if power.coupling_note.contains(forbidden) {
                return Err(PowerRegistrationError::ForbiddenField(forbidden));
            }
        }
        if self.find(&power.id).is_some() {
            return Err(PowerRegistrationError::DuplicateId(power.id));
        }
        self.powers.push(power);
        Ok(())
    }

    pub fn from_iter<I>(iter: I) -> Result<Self, PowerRegistrationError>
    where
        I: IntoIterator<Item = PowerDef>,
    {
        let mut reg = Self::new();
        for p in iter {
            reg.register(p)?;
        }
        Ok(reg)
    }
}

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

pub fn default_registry() -> Result<PowerRegistry, PowerRegistrationError> {
    PowerRegistry::from_iter(default_powers())
}

