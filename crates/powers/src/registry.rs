#![allow(missing_docs)]

use crate::{
    Hotkey, PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRequestKind, PowerTab,
    PowerTargetMask,
};

/// Stable identifier for a god-tool verb (e.g. `"terrain.raise"`).
///
/// `PowerId` is a newtype around `&'static str` for compile-time
/// deduplication and so the catalog is a `&'static [PowerDef]`.
/// Mod-added powers (Phase 5) will use a separate `String`-backed
/// `PowerId::runtime` variant; this Phase 1 only ships the static
/// half.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerId(&'static str);

impl PowerId {
    /// Construct a compile-time power id.
    pub const fn new_const(value: &'static str) -> Self {
        Self(value)
    }

    /// Borrow the inner string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl core::fmt::Display for PowerId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

/// The eight god-tool tabs (FR-CIV-GODTOOL-900).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerTab {
    /// Terraform / sculpt (11 verbs).
    Terrain,
    /// CA seeding (8 verbs).
    Material,
    /// Spawn organic + civ (8 verbs).
    Life,
    /// CA energy injection (8 verbs).
    Disaster,
    /// Read-only (8 verbs).
    Inspect,
    /// Parameter nudges (8 verbs).
    Law,
    /// Universal mouse-driven camera (8 verbs).
    Camera,
    /// Clock control (8 verbs).
    Time,
}

/// Whether a power mutates a substrate field, reads only, or
/// manipulates the UI/schedule only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerCategory {
    /// Writes to a substrate field that the engine already reads.
    Mutating,
    /// Read-only; no substrate write.
    ReadOnly,
    /// UI / schedule only; never touches the substrate.
    Universal,
}

/// Availability tier mirroring the info-views legibility moat —
/// Live powers stamp substrate writes; Near powers are
/// lit-but-inert in the deck; Blind powers are visibly disabled
/// with a "coming soon" tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerAvailability {
    /// Verb has a substrate handler in `civ-engine` and stamps
    /// real mutations.
    Live,
    /// Verb is registered in the deck but the substrate handler
    /// has not landed yet ("data not yet surfaced").
    Near,
    /// Verb is registered but the backing substrate is not
    /// available; visible-disabled.
    Blind,
}

/// The kind of request a power emits to the substrate. Mirrors
/// the `Simulation::apply_god_tool` dispatcher variants in
/// `crates/engine/src/godtools.rs`.
///
/// **Note:** `ScriptedOutcome` is **intentionally absent**. A power
/// that wants a scripted outcome fails to register — this is the
/// AC-CPL-3 compile-time guard from
/// `docs/design/GODTOOLS_IMPL_PLAN.md` §6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerRequestKind {
    /// `→ crates/voxel::VoxelWorld::write` (Replace, Erase, SurfacePaint, etc.)
    MaterialEdit,
    /// `→ crates/engine::Simulation::push_voxel_write` (Raise, Lower, Level, …)
    TerraformEdit,
    /// `→ civ_agents::spawn_child_near` / `spawn_many` / `spawn_civilian_at`
    ActorSpawn,
    /// `→ crates/agents::apply_actor_effect` (Bless, Curse, Heal, Plague)
    ActorEffect,
    /// `→ Simulation::invoke_divine_disaster` (Meteor, Flood, Quake, …)
    Disaster,
    /// `→ Simulation::apply_scenario_taxation` / `LawDb::apply_overlay` (Law tab)
    Law,
    /// Bevy schedule handles — never a substrate write (Time tab)
    Time,
    /// Read-only: the substrate handler is a no-op (Inspect tab)
    NoOp,
}

/// Bitset of substrate targets a power writes to. Used by the
/// deck filter UI; not enforced at registration (the substrate
/// handler is the single source of truth on what actually
/// mutates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerTargetMask(pub u8);

impl PowerTargetMask {
    /// The power targets the voxel substrate.
    pub const VOXEL: Self = Self(1 << 0);
    /// The power targets the agent substrate.
    pub const AGENT: Self = Self(1 << 1);
    /// The power targets the settlement / building substrate.
    pub const SETTLEMENT: Self = Self(1 << 2);
    /// The power targets a continuous CA field (climate, diffusion).
    pub const FIELD: Self = Self(1 << 3);
    /// The power targets the simulation schedule (time).
    pub const TIME: Self = Self(1 << 4);
}

impl std::ops::BitOr for PowerTargetMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for PowerTargetMask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// A single god-tool verb. The data-driven schema per
/// `docs/design/GOD_TOOLS_SANDBOX.md` §5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerDef {
    /// Stable id, e.g. `"terrain.raise"`.
    pub id: PowerId,
    /// Human-readable label, e.g. `"Raise"`.
    pub label: &'static str,
    /// Which tab this verb belongs to.
    pub tab: PowerTab,
    /// Mutating / read-only / universal.
    pub category: PowerCategory,
    /// Substrate request kind — drives the dispatch in
    /// `civ_engine::Simulation::apply_god_tool`.
    pub request: PowerRequestKind,
    /// Live / Near / Blind (mirrors info-views legibility moat).
    pub availability: PowerAvailability,
    /// One-line description of the substrate write (e.g.
    /// "writes `CaGrid.height`; CA settles").
    pub coupling_note: &'static str,
}

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
