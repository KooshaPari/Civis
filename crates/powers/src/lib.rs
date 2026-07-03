#![forbid(unsafe_code)]
#![allow(missing_docs)]

mod registry;

pub use registry::{PowerRegistrationError, PowerRegistry, FORBIDDEN_TARGET_FIELDS, default_powers, default_registry};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerId(pub &'static str);

impl PowerId {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerTab {
    Terrain,
    Material,
    Life,
    Disaster,
    Inspect,
    Law,
    Camera,
    Time,
}

impl PowerTab {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PowerTab::Terrain => "TERRAIN",
            PowerTab::Material => "MATERIAL",
            PowerTab::Life => "LIFE",
            PowerTab::Disaster => "DISASTER",
            PowerTab::Inspect => "INSPECT",
            PowerTab::Law => "LAW",
            PowerTab::Camera => "CAMERA",
            PowerTab::Time => "TIME",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerCategory {
    Mutating,
    ReadOnly,
    Universal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerRequestKind {
    TerraformEdit,
    MaterialEdit,
    ActorSpawn,
    ActorEffect,
    Disaster,
    Law,
    Time,
    NoOp,
}

impl PowerRequestKind {
    #[must_use]
    pub const fn is_substrate_write(self) -> bool {
        matches!(
            self,
            Self::TerraformEdit
                | Self::MaterialEdit
                | Self::ActorSpawn
                | Self::ActorEffect
                | Self::Disaster
                | Self::Law
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerTargetMask(pub u8);

impl PowerTargetMask {
    pub const VOXEL: Self = Self(1 << 0);
    pub const AGENT: Self = Self(1 << 1);
    pub const SETTLEMENT: Self = Self(1 << 2);
    pub const FIELD: Self = Self(1 << 3);
    pub const TIME: Self = Self(1 << 4);
    pub const UI: Self = Self(1 << 5);

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerAvailability {
    Live,
    Near,
    Blind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerDef {
    pub id: PowerId,
    pub tab: PowerTab,
    pub category: PowerCategory,
    pub label: &'static str,
    pub glyph: &'static str,
    pub hotkey: Option<Hotkey>,
    pub request: PowerRequestKind,
    pub applies_to: PowerTargetMask,
    pub coupling_note: &'static str,
    pub availability: PowerAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey(pub u8);

impl Hotkey {
    pub const fn new(byte: u8) -> Self {
        Self(byte)
    }

    #[must_use]
    pub fn label(self) -> String {
        (self.0 as char).to_ascii_uppercase().to_string()
    }
}
