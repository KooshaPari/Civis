//! `civ-powers` — Data-driven God-Tools Power registry (FR-CIV-GODTOOL-900, -901).
//!
//! This crate is the **registry side** of the God-Tools dispatch. It defines the
//! schema of a `PowerDef` and the `PowerRegistry` collection; it is intentionally
//! **devoiced of business logic** (no Bevy types, no hecs, no `Simulation`).
//! Every handler that actually mutates state lives in `civ_engine::godtools`
//! and writes through the substrate APIs (`push_voxel_write`,
//! `invoke_divine_disaster`, `civ_agents::spawn_*`). See
//! `docs/design/GODTOOLS_IMPL_PLAN.md` for the full architecture.
//!
//! Phase 1 (this commit) ships:
//!
//! * `PowerDef` schema (id, tab, category, label, glyph, hotkey, request kind,
//!   target mask, param schema, availability, coupling note).
//! * `PowerRegistry::register` with the **AC-CPL-3 negative-field guard** that
//!   rejects `PowerDef` entries whose `writes_fields` set intersects the
//!   "never directly set" list (`culture`, `religion`, `ideology`,
//!   `alignment`, `job`, `faction_id`, `mood`).
//! * `default_powers()` returning **all 50 verbs** (TERRAIN 11 + MATERIAL 8 +
//!   LIFE 8 + DISASTER 8 + INSPECT 8 + LAW 8 + CAMERA 8 + TIME 8) with
//!   `availability: Near` (lit-but-inert — handlers land in later phases per
//!   GODTOOLS_IMPL_PLAN.md §8 P2).
//! * 3 unit tests covering the registry invariants (AC-REG-1, AC-CPL-3, count).

#![forbid(unsafe_code)]
#![allow(missing_docs)]

mod registry;

pub use registry::{PowerRegistrationError, PowerRegistry};

use serde::{Deserialize, Serialize};

/// Unique power identifier. Convention is `"<tab>.<verb>"` (e.g. `terrain.raise`,
/// `disaster.meteor`, `inspect.probe`). Used as the Holocron Deck key, the
/// JSON-RPC catalog entry, and the `civ-mod-host` host-import handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerId(pub &'static str);

impl PowerId {
    /// Construct a new `PowerId` from a string literal.
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }
    /// Borrow the inner string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// The 8 tabs of the Holocron Deck (per the spec at GOD_TOOLS_SANDBOX.md §0.3).
///
/// `Camera` and `Time` are universal verbs that are not substrate mutators in
/// the strict sense (the substrate has no `camera` field; `Time<Fixed>` lives
/// in the Bevy schedule), but they live in the registry so the deck, search
/// bar, and keycap palette can derive from one source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerTab {
    /// TERRAIN — terraform + sculpt (11 verbs). Writes the `VoxelWorld`.
    Terrain,
    /// MATERIAL — CA seeding (8 verbs). Voxel writes + life spawn + CA field.
    Material,
    /// LIFE — spawn organic + civ (8 verbs). Substrate: `civ_agents`.
    Life,
    /// DISASTER — CA energy injection (8 verbs). Substrate: `invoke_divine_disaster`.
    Disaster,
    /// INSPECT — read-only (8 verbs). **No substrate writes.**
    Inspect,
    /// LAW — parameter nudges (8 verbs). Writes a *parameter*; substrate re-derives.
    Law,
    /// CAMERA — universal verb (8 verbs). **No substrate writes.**
    Camera,
    /// TIME — clock control (8 verbs). Mutates the Bevy schedule, not the sim.
    Time,
}

impl PowerTab {
    /// Human-readable label, matching the deck's tab strip.
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

/// Mutating vs read-only vs universal — drives the deck icon and the
/// `apply_god_tool` chokepoint: `ReadOnly` and `Universal` powers are no-ops
/// at the substrate layer (AC-REG-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerCategory {
    /// Writes the substrate (terrain, life, disaster, law, material).
    Mutating,
    /// Read-only (inspect.*). The substrate is never touched.
    ReadOnly,
    /// Universal verb (camera.*, time.*). Lives on the Bevy schedule.
    Universal,
}

/// Discriminator of the substrate write the verb performs.
///
/// This is the **AC-CPL-2 chokepoint**: a `PowerDef::request` whose kind is
/// not in this enum is rejected at registration time. There is **no**
/// `ScriptedOutcome` variant on purpose — a power that wanted to bypass the
/// substrate into "the result" cannot even be expressed in this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerRequestKind {
    /// Voxel material column write (TERRAIN 11 + most MATERIAL verbs).
    TerraformEdit,
    /// Voxel material write (MATERIAL replace/erase/surface-paint/...).
    MaterialEdit,
    /// Spawn a life agent (`civ_agents::spawn_child_near` / `spawn_civilian_at`).
    ActorSpawn,
    /// Apply an effect to a footprint of actors (Bless/Curse/Heal).
    ActorEffect,
    /// Invoke a divine disaster (`Simulation::invoke_divine_disaster`).
    Disaster,
    /// Write a parameter that a substrate subsystem reads (LAW verbs).
    Law,
    /// Time control — handled by the Bevy schedule, **not** the sim dispatcher.
    Time,
    /// Inspect — read-only, returns data to the UI without mutating the sim.
    NoOp,
}

impl PowerRequestKind {
    /// True when the request is a substrate mutator. The opposite drives
    /// `Simulation::apply_god_tool`'s `Ok(no_op)` early-return for INSPECT.
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

/// Bitmask of the substrate domains a verb targets. Used by the
/// `mod_origin` filter and the deck search facets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PowerTargetMask(pub u8);

impl PowerTargetMask {
    /// Voxel world (`VoxelWorld<MaterialId>`).
    pub const VOXEL: Self = Self(1 << 0);
    /// Agent layer (`civ_agents`).
    pub const AGENT: Self = Self(1 << 1);
    /// Settlement / building graph (`civ_build::BuildingGraph`).
    pub const SETTLEMENT: Self = Self(1 << 2);
    /// CA / climate / weather field (`civ_planet`, `civ_diffusion`).
    pub const FIELD: Self = Self(1 << 3);
    /// Schedule / clock (Bevy `Time<Fixed>`, snapshot ring).
    pub const TIME: Self = Self(1 << 4);
    /// Pure UI (camera transform, HUD) — no substrate write.
    pub const UI: Self = Self(1 << 5);
    /// OR-combine two masks.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// True when the mask contains the given bit.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Availability gating (mirrors `info-views.md` §7 loud-not-silent posture):
/// `Live` powers are fully wired; `Near` powers are lit-but-inert (visible in
/// the deck, search-findable, but their apply path returns a no-op
/// `GodToolReceipt` with a "data not yet surfaced" tag); `Blind` powers are
/// disabled in the deck (greyed, with a named "coming soon" tag — AC-REG-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerAvailability {
    /// Live: the dispatch is wired and mutates the substrate.
    Live,
    /// Near: lit-but-inert, no-op at the substrate (Phase 1 ships all 50 in
    /// `Near`; Phase 2 promotes the listed verbs to `Live`).
    Near,
    /// Blind: visibly disabled with a "coming soon" tag (AC-REG-5).
    Blind,
}

/// One entry in the data-driven `PowerRegistry`.
///
/// Adding a new power = append one `PowerDef` to `default_powers()` +
/// (optionally) implement the request handler. The Holocron Deck, the search
/// bar, the Keycap Palette rim, the hotkey map, and the in-world ring all
/// read from the registry — no panel is hand-coded. (AC-REG-1, AC-REG-2.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerDef {
    /// Stable id, e.g. `"terrain.raise"`.
    pub id: PowerId,
    /// Which tab on the deck.
    pub tab: PowerTab,
    /// Mutating vs read-only vs universal.
    pub category: PowerCategory,
    /// Human label, e.g. `"Raise"`.
    pub label: &'static str,
    /// Glyph hint (material icon name in the icon atlas). Free-form; the
    /// UI maps it onto the bevy_egui / web icon set.
    pub glyph: &'static str,
    /// Optional default hotkey (None for verbs that don't get a keycap).
    pub hotkey: Option<Hotkey>,
    /// Discriminator of the substrate write — see [`PowerRequestKind`].
    pub request: PowerRequestKind,
    /// Bitmask of the substrate domains this verb targets.
    pub applies_to: PowerTargetMask,
    /// Coupling note (the charter guarantee), e.g.
    /// `"writes VoxelWorld<MaterialId>; CA settles"`. Displayed in tooltips.
    pub coupling_note: &'static str,
    /// Live / Near / Blind.
    pub availability: PowerAvailability,
}

/// Optional hotkey hint. We use a portable ASCII byte (a keycap letter or
/// digit) rather than a Bevy `KeyCode` so this crate stays Bevy-free
/// (AC-REG-2: the registry must be the single source of truth across the
/// web / Godot / Unreal mirrors, none of which share Bevy types).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hotkey(pub u8);

impl Hotkey {
    /// Construct a hotkey from a single ASCII byte (e.g. `Hotkey(b'Q')`).
    pub const fn new(byte: u8) -> Self {
        Self(byte)
    }
    /// Render as an uppercase label (e.g. `"Q"`).
    #[must_use]
    pub fn label(self) -> String {
        (self.0 as char).to_ascii_uppercase().to_string()
    }
}
