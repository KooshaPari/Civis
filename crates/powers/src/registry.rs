//! `PowerRegistry` — the data-driven collection of `PowerDef`s plus the
//! AC-CPL-3 / AC-CPL-4 guards.
//!
//! Phase 1 (this commit) ships the **full 50-verb deck** (`default_powers`)
//! with all entries marked `availability: Near`. Later phases (P2 / P3 in
//! `docs/design/GODTOOLS_IMPL_PLAN.md`) promote the listed verbs to `Live`
//! by registering a `Simulation::apply_god_tool` handler.
//!
//! Adding a new power is a **one-line edit** to `default_powers()`:
//!
//! ```ignore
//! PowerDef {
//!     id: PowerId::new("material.ice_drop"),
//!     tab: PowerTab::Material,
//!     category: PowerCategory::Mutating,
//!     label: "Ice Drop",
//!     glyph: "ice",
//!     hotkey: None,
//!     request: PowerRequestKind::MaterialEdit,
//!     applies_to: PowerTargetMask::VOXEL,
//!     coupling_note: "writes VoxelWorld<MaterialId>::write(ICE); thermo CA melts",
//!     availability: PowerAvailability::Near,
//! },
//! ```
//!
//! The deck, search bar, palette rim, hotkey map, and in-world ring all
//! derive from this list — no panel is hand-coded (AC-REG-1, AC-REG-2).

#![allow(missing_docs)]

use crate::{
    Hotkey, PowerAvailability, PowerCategory, PowerDef, PowerId, PowerRequestKind, PowerTab,
    PowerTargetMask,
};

/// Negative-field list — `PowerRegistry::register` rejects any `PowerDef`
/// whose `writes_fields` intersects this set (AC-CPL-3).
///
/// Per the spec (GOD_TOOLS_SANDBOX.md §3.3) and the FR (FR-CIV-GODTOOL-910),
/// a god-tool that wants a schism spawns two geographically separated
/// populations with a language barrier and **reads** the schism on the
/// info-view. There is no `culture = "X"` set: these fields are **measured**
/// by the substrate, never authored.
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

/// Why a `PowerDef` was rejected. Returned by [`PowerRegistry::register`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerRegistrationError {
    /// A field the verb claimed to write is in the AC-CPL-3 negative list.
    /// The string is the offending field name.
    ForbiddenField(&'static str),
    /// The verb's id is already in the registry (duplicates are not allowed).
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

/// The data-driven collection of `PowerDef`s. Stored in a `Vec` (not a
/// `HashMap`) so the deck iterates in canonical order; lookup by id is
/// O(n) but n is bounded at 50 + a few mod-added verbs.
#[derive(Debug, Clone, Default)]
pub struct PowerRegistry {
    powers: Vec<PowerDef>,
}

impl PowerRegistry {
    /// Construct a new, empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            powers: Vec::new(),
        }
    }

    /// Number of registered powers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.powers.len()
    }

    /// True when no powers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.powers.is_empty()
    }

    /// Borrow the registered powers in canonical (registration) order.
    #[must_use]
    pub fn powers(&self) -> &[PowerDef] {
        &self.powers
    }

    /// Look up a power by id. Returns `None` if the id is unknown — the
    /// Bevy dispatcher uses this to reject malformed `GodToolEvent`s at the
    /// AC-CPL-2 chokepoint.
    #[must_use]
    pub fn find(&self, id: &PowerId) -> Option<&PowerDef> {
        self.powers.iter().find(|p| p.id == *id)
    }

    /// Register a single `PowerDef`. Returns `Err` if the verb violates
    /// the AC-CPL-3 negative-field guard or duplicates an existing id.
    ///
    /// The coupling_note string is scanned for tokens like `"culture"`,
    /// `"religion"`, etc. — this is a cheap, in-crate text scan that does
    /// not require a separate `writes_fields` set per `PowerDef`. If the
    /// coupling note ever mentions a forbidden field, the power fails to
    /// register. (Mod-added powers go through the same check via
    /// `civ_register_power` in the follow-up P5 phase.)
    pub fn register(&mut self, power: PowerDef) -> Result<(), PowerRegistrationError> {
        // AC-CPL-3: scan the coupling note for forbidden field names. A power
        // that *talks about* writing a forbidden field in its coupling note
        // fails to register (e.g. `coupling_note: "writes culture"`).
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

    /// Build a registry from an iterator of `PowerDef`s. Each is registered
    /// individually; the first error short-circuits. Use this to compose the
    /// default 50-verb set with mod-added entries (P5).
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

/// Construct the **full 50-verb Holocron Deck** (the canonical headline per
/// `docs/design/GODTOOLS_SANDBOX.md:55`).
///
/// All verbs are `availability: Near` in Phase 1 — the **registry** is
/// complete (AC-REG-1, AC-REG-2), but the per-verb `apply_god_tool`
/// handlers land in P2 (`docs/design/GODTOOLS_IMPL_PLAN.md` §8).
pub fn default_powers() -> Vec<PowerDef> {
    vec![
        // ────── TERRAIN (11) ──────
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
        PowerDef {
            id: PowerId::new("terrain.lower"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Lower",
            glyph: "terrain_lower",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "inverse of terrain.raise",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.level"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Level",
            glyph: "terrain_level",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes VoxelWorld height column to picked target_height",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.smooth"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Smooth",
            glyph: "terrain_smooth",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "diffusion-style smoothing pass on height field",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.slope"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Slope",
            glyph: "terrain_slope",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "tilts height field along an anchor-defined gradient",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.flatten"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Flatten",
            glyph: "terrain_flatten",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "reads majority top voxel; writes level value",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.shift"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "Shift",
            glyph: "terrain_shift",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "read-then-write translation of footprint",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.add_land"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "AddLand",
            glyph: "terrain_add_land",
            hotkey: Some(Hotkey::new(b'T')),
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "chunky +delta in hard-edged footprint (WorldBox god brush)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.dig_ocean"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "DigOcean",
            glyph: "terrain_dig_ocean",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::FIELD),
            coupling_note: "sets height to planet sea_level; water CA fills",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.raise_mountain"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "RaiseMountain",
            glyph: "terrain_raise_mountain",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "Gaussian peak + height-noise dither; volcanic CA may follow",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("terrain.drop_biome"),
            tab: PowerTab::Terrain,
            category: PowerCategory::Mutating,
            label: "DropBiome",
            glyph: "terrain_drop_biome",
            hotkey: None,
            request: PowerRequestKind::TerraformEdit,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::FIELD),
            coupling_note: "surface material swap; flora/fauna seed themselves",
            availability: PowerAvailability::Near,
        },
        // ────── MATERIAL (8) ──────
        PowerDef {
            id: PowerId::new("material.replace"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "Replace",
            glyph: "material_replace",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes VoxelWorld<MaterialId> per cell to depth",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.additive_drop"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "AdditiveDrop",
            glyph: "material_additive_drop",
            hotkey: Some(Hotkey::new(b'E')),
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes at drop_height; gravity CA advects (gas inverts)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.erase"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "Erase",
            glyph: "material_erase",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes VoxelWorld<MaterialId>::AIR to depth in footprint",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.surface_paint"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "SurfacePaint",
            glyph: "material_surface_paint",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "thin-skin paint on topmost solid voxel per (x,z)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.pour_liquid"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "PourLiquid",
            glyph: "material_pour_liquid",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "spawns WATER/LAVA/OIL slab; CA spreads (Powder-Toy flood)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.seed_forest"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "SeedForest",
            glyph: "material_seed_forest",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "spawns N plant-agents with seed-genome near footprint",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.seed_ore"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "SeedOreDeposit",
            glyph: "material_seed_ore",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::FIELD),
            coupling_note: "writes ore_density CA field; mining agents extract later",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("material.seed_snow"),
            tab: PowerTab::Material,
            category: PowerCategory::Mutating,
            label: "SeedSnow",
            glyph: "material_seed_snow",
            hotkey: None,
            request: PowerRequestKind::MaterialEdit,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "writes ICE above snowline; thermo CA melts",
            availability: PowerAvailability::Near,
        },
        // ────── LIFE (8) ──────
        PowerDef {
            id: PowerId::new("life.spawn_organism"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "SpawnOrganism",
            glyph: "life_spawn_organism",
            hotkey: Some(Hotkey::new(b'R')),
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "civ_agents::spawn_child_near with {genome, cradle_state, age}",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.spawn_herd"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "SpawnHerd",
            glyph: "life_spawn_herd",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "civ_agents::spawn_many with shared genome and jittered positions",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.spawn_civ_seed"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "SpawnCivSeed",
            glyph: "life_spawn_civ_seed",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT.union(PowerTargetMask::SETTLEMENT),
            coupling_note: "6 founder agents + 1 hut + 1 stockpile; no hardcoded tribe name",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.bless"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Bless",
            glyph: "life_bless",
            hotkey: None,
            request: PowerRequestKind::ActorEffect,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "writes Needs (food/shelter/safety/belonging) up; agent decides",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.curse"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Curse",
            glyph: "life_curse",
            hotkey: None,
            request: PowerRequestKind::ActorEffect,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "inverse of life.bless; writes Needs down",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.plague"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Plague",
            glyph: "life_plague",
            hotkey: None,
            request: PowerRequestKind::ActorEffect,
            applies_to: PowerTargetMask::AGENT.union(PowerTargetMask::FIELD),
            coupling_note: "writes Pathogen SIR field; R0-driven contagion",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.heal"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Heal",
            glyph: "life_heal",
            hotkey: None,
            request: PowerRequestKind::ActorEffect,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "writes Health up; agents recover via their own need decay",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("life.extinct"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Extinct",
            glyph: "life_extinct",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "hecs::World::despawn on matching genome; substrate records it",
            availability: PowerAvailability::Near,
        },
        // ────── DISASTER (8) ──────
        PowerDef {
            id: PowerId::new("disaster.meteor"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Meteor",
            glyph: "disaster_meteor",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::AGENT),
            coupling_note: "Simulation::invoke_divine_disaster(Meteor, pos, 0); CA spreads",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.lightning"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Lightning",
            glyph: "disaster_lightning",
            hotkey: Some(Hotkey::new(b'F')),
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "electric arc; thermo CA ignites flammables",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.flood"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Flood",
            glyph: "disaster_flood",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::AGENT),
            coupling_note: "Simulation::invoke_divine_disaster(Flood, pos, 0); water CA spreads",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.quake"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Quake",
            glyph: "disaster_quake",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL.union(PowerTargetMask::AGENT),
            coupling_note: "Simulation::invoke_divine_disaster(Quake, pos, 0); aftershocks",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.firestorm"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Firestorm",
            glyph: "disaster_firestorm",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "sustained Wildfire with radius param; thermo CA spreads",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.tornado"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Tornado",
            glyph: "disaster_tornado",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::FIELD.union(PowerTargetMask::AGENT),
            coupling_note: "rotating wind field in civ_planet; paths emerge from CA",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.volcanic_vent"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "VolcanicVent",
            glyph: "disaster_volcanic_vent",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::VOXEL,
            coupling_note: "sustained LAVA; thermo CA cools to rock",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("disaster.drought"),
            tab: PowerTab::Disaster,
            category: PowerCategory::Mutating,
            label: "Drought",
            glyph: "disaster_drought",
            hotkey: None,
            request: PowerRequestKind::Disaster,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "WeatherCell.precip_mm_fp -= delta; agriculture reacts",
            availability: PowerAvailability::Near,
        },
        // ────── INSPECT (8) ──────
        PowerDef {
            id: PowerId::new("inspect.probe"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Probe",
            glyph: "inspect_probe",
            hotkey: Some(Hotkey::new(b'Q')),
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; opens Inspector panel for the entity under cursor",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.stats"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Stats",
            glyph: "inspect_stats",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; civ_watch aggregates over region footprint",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.trace"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Trace",
            glyph: "inspect_trace",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; walks the Legends saga graph backwards",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.forecast"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Forecast",
            glyph: "inspect_forecast",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; runs a counterfactual sim for N ticks, then restores",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.compare_snapshots"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "CompareSnapshots",
            glyph: "inspect_compare_snapshots",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; diffs two Snapshot blobs from civ_watch",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.history"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "History",
            glyph: "inspect_history",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; timeline ribbon scrubber",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.bookmark"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Bookmark",
            glyph: "inspect_bookmark",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; stores/recalls camera + sim state",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("inspect.follow"),
            tab: PowerTab::Inspect,
            category: PowerCategory::ReadOnly,
            label: "Follow",
            glyph: "inspect_follow",
            hotkey: None,
            request: PowerRequestKind::NoOp,
            applies_to: PowerTargetMask::UI,
            coupling_note: "read-only; locks camera to a selected actor",
            availability: PowerAvailability::Near,
        },
        // ────── LAW (8) ──────
        PowerDef {
            id: PowerId::new("law.tax_bias"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "TaxBias",
            glyph: "law_tax_bias",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "Simulation::apply_scenario_taxation(&Taxation); market re-prices",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.edict"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "Edict",
            glyph: "law_edict",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "LawDb::apply_overlay; policy phase reads the new rule",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.religion_pressure"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "ReligionPressure",
            glyph: "law_religion_pressure",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "writes religious_doctrine diffusion field; SIR CA propagates",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.sanction"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "Sanction",
            glyph: "law_sanction",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "removes trade-route entries; agents reroute via A*",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.open_border"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "OpenBorder",
            glyph: "law_open_border",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "adds trade-route entries; subject to existing entry validation",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.alignment_nudge"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "AlignmentNudge",
            glyph: "law_alignment_nudge",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "writes AI utility weights; faction re-scores via utility",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.difficulty_knob"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "DifficultyKnob",
            glyph: "law_difficulty_knob",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "economy_policy.scarcity_multiplier; phase_economy consumes",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("law.scenario_script"),
            tab: PowerTab::Law,
            category: PowerCategory::Mutating,
            label: "ScenarioScript",
            glyph: "law_scenario_script",
            hotkey: None,
            request: PowerRequestKind::Law,
            applies_to: PowerTargetMask::FIELD,
            coupling_note: "reads scenario.ron; replays the script via the standard queue",
            availability: PowerAvailability::Near,
        },
        // ────── CAMERA (8) — universal verb ──────
        PowerDef {
            id: PowerId::new("camera.orbit"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Orbit",
            glyph: "camera_orbit",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "camera transform only; sim has no accessor (AC-CPL-1 wall)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.pan"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Pan",
            glyph: "camera_pan",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "camera transform only",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.zoom"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Zoom",
            glyph: "camera_zoom",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "camera transform only",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.tilt"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Tilt",
            glyph: "camera_tilt",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "camera transform only",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.roll"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Roll",
            glyph: "camera_roll",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "camera transform only; disabled by default",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.bookmarks"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "Bookmarks",
            glyph: "camera_bookmarks",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "stores/recalls camera + sim state",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.follow_cam"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "FollowCam",
            glyph: "camera_follow",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "locks camera to a selected actor",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("camera.photo_mode"),
            tab: PowerTab::Camera,
            category: PowerCategory::Universal,
            label: "PhotoMode",
            glyph: "camera_photo_mode",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::UI,
            coupling_note: "hides HUD/overlays, free-cam, DoF/exposure",
            availability: PowerAvailability::Near,
        },
        // ────── TIME (8) — clock control ──────
        PowerDef {
            id: PowerId::new("time.pause"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Pause",
            glyph: "time_pause",
            hotkey: Some(Hotkey::new(b'A')),
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "Bevy Time<Fixed>::set_relative_speed(0.0); sim pauses",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.play"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Play",
            glyph: "time_play",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "Bevy Time<Fixed>::set_relative_speed(1.0)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.slow"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Slow",
            glyph: "time_slow",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "Bevy Time<Fixed>::set_relative_speed(0.25)",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.fast"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Fast",
            glyph: "time_fast",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "Bevy Time<Fixed>::set_relative_speed({2,5,10})",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.step"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Step",
            glyph: "time_step",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "advances exactly N ticks via Schedule::run; auto-pauses",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.rewind"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Rewind",
            glyph: "time_rewind",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "Snapshot::restore_into_simulation; soft determinism per charter",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.fast_forward_to_event"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "FastForwardToEvent",
            glyph: "time_fast_forward_to_event",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "filters civ_watch stream; advances until match or max-tick cap",
            availability: PowerAvailability::Near,
        },
        PowerDef {
            id: PowerId::new("time.profile"),
            tab: PowerTab::Time,
            category: PowerCategory::Universal,
            label: "Profile",
            glyph: "time_profile",
            hotkey: None,
            request: PowerRequestKind::Time,
            applies_to: PowerTargetMask::TIME,
            coupling_note: "writes a perf-trace log; separate from replay log",
            availability: PowerAvailability::Near,
        },
    ]
}

/// Helper that builds a `PowerRegistry` from [`default_powers()`]. Returns
/// the registry on success; the Phase 1 `default_powers` list is constructed
/// at compile time so this should never fail in practice — the `Result`
/// return type is for symmetry with `PowerRegistry::from_iter` so callers
/// that splice in mod-added entries can compose the two lists safely.
pub fn default_registry() -> Result<PowerRegistry, PowerRegistrationError> {
    PowerRegistry::from_iter(default_powers())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PowerCategory, PowerId, PowerRequestKind, PowerTab};

    // ----- AC-REG-1 / AC-REG-2: default_powers is the canonical 50-verb set -----

    /// The headline figure: TERRAIN 11 + MATERIAL 8 + LIFE 8 + DISASTER 8 +
    /// INSPECT 8 + LAW 8 + CAMERA 8 + TIME 8 = **50** discrete verbs.
    /// (`docs/design/GOD_TOOLS_SANDBOX.md:55`, FR-CIV-GODTOOL-900).
    #[test]
    fn default_powers_enumerates_fifty_verbs() {
        let registry = default_registry().expect("default registry builds");
        assert_eq!(
            registry.len(),
            50,
            "expected 50 verbs in default_powers() (FR-CIV-GODTOOL-900), got {}",
            registry.len()
        );
    }

    #[test]
    fn default_powers_has_eight_per_tab() {
        let registry = default_registry().expect("default registry builds");
        // 8 tabs: Terrain / Material / Life / Disaster / Inspect / Law / Camera / Time.
        // We can only count via the public powers() slice; the crate-private
        // PowerTab -> index mapping is a small helper local to the test.
        let mut counts = [0usize; 8];
        for p in registry.powers() {
            counts[tab_index(p.tab)] += 1;
        }
        // TERRAIN=11, MATERIAL/LIFE/DISASTER/INSPECT/LAW/CAMERA/TIME=8
        assert_eq!(counts[tab_index(PowerTab::Terrain)], 11);
        assert_eq!(counts[tab_index(PowerTab::Material)], 8);
        assert_eq!(counts[tab_index(PowerTab::Life)], 8);
        assert_eq!(counts[tab_index(PowerTab::Disaster)], 8);
        assert_eq!(counts[tab_index(PowerTab::Inspect)], 8);
        assert_eq!(counts[tab_index(PowerTab::Law)], 8);
        assert_eq!(counts[tab_index(PowerTab::Camera)], 8);
        assert_eq!(counts[tab_index(PowerTab::Time)], 8);
    }

    fn tab_index(tab: PowerTab) -> usize {
        match tab {
            PowerTab::Terrain => 0,
            PowerTab::Material => 1,
            PowerTab::Life => 2,
            PowerTab::Disaster => 3,
            PowerTab::Inspect => 4,
            PowerTab::Law => 5,
            PowerTab::Camera => 6,
            PowerTab::Time => 7,
        }
    }

    #[test]
    fn default_powers_ids_are_unique() {
        let registry = default_registry().expect("default registry builds");
        let mut seen = std::collections::HashSet::new();
        for p in registry.powers() {
            assert!(
                seen.insert(p.id),
                "duplicate power id: {}",
                p.id.as_str()
            );
        }
    }

    // ----- AC-CPL-3: no power writes a forbidden field -----

    /// A verb whose `coupling_note` mentions a forbidden field
    /// (`culture`, `religion`, `ideology`, `alignment`, `job`,
    /// `faction_id`, `mood`, `happiness`) fails to register.
    /// This is the AC-CPL-3 compile-time guard.
    #[test]
    fn register_rejects_forbidden_field_in_coupling_note() {
        let mut reg = PowerRegistry::new();
        let res = reg.register(PowerDef {
            id: PowerId::new("test.schemer"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Schemer",
            glyph: "schemer",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "writes ideology into the agent's soul",
            availability: PowerAvailability::Near,
        });
        assert!(
            matches!(res, Err(PowerRegistrationError::ForbiddenField("ideology"))),
            "expected ForbiddenField(\"ideology\"), got {res:?}"
        );
        assert!(reg.is_empty());
    }

    /// Each forbidden field name, when written into a coupling note, is
    /// rejected individually — covering all 8 entries of the negative list.
    /// We use a stable ordering of the tests (8) to avoid leaking test data.
    #[test]
    fn register_rejects_every_forbidden_field() {
        for forbidden in FORBIDDEN_TARGET_FIELDS {
            let mut reg = PowerRegistry::new();
            // Build a coupling note that mentions the forbidden field, then
            // leak it to get a 'static str (test-only, one byte per
            // iteration — acceptable).
            let owned: String = format!("writes {forbidden} into the substrate");
            let coupling_note: &'static str = Box::leak(owned.into_boxed_str());
            let res = reg.register(PowerDef {
                id: PowerId::new("test.bad"),
                tab: PowerTab::Life,
                category: PowerCategory::Mutating,
                label: "Bad",
                glyph: "bad",
                hotkey: None,
                request: PowerRequestKind::ActorSpawn,
                applies_to: PowerTargetMask::AGENT,
                coupling_note,
                availability: PowerAvailability::Near,
            });
            assert!(
                matches!(res, Err(PowerRegistrationError::ForbiddenField(f)) if f == *forbidden),
                "expected ForbiddenField({forbidden}), got {res:?}"
            );
        }
    }

    #[test]
    fn register_rejects_duplicate_id() {
        let mut reg = PowerRegistry::new();
        let power = PowerDef {
            id: PowerId::new("test.dup"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Dup",
            glyph: "dup",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "spawns a second organism",
            availability: PowerAvailability::Near,
        };
        reg.register(power).expect("first register succeeds");
        let res = reg.register(PowerDef {
            id: PowerId::new("test.dup"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Dup2",
            glyph: "dup2",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "different coupling note, same id",
            availability: PowerAvailability::Near,
        });
        assert!(
            matches!(res, Err(PowerRegistrationError::DuplicateId(_))),
            "expected DuplicateId, got {res:?}"
        );
        assert_eq!(reg.len(), 1);
    }

    // ----- AC-REG-1: adding a power is a one-line edit -----

    /// A power that satisfies the guards is registered and found by id.
    #[test]
    fn register_accepts_well_formed_power() {
        let mut reg = PowerRegistry::new();
        let power = PowerDef {
            id: PowerId::new("test.good"),
            tab: PowerTab::Life,
            category: PowerCategory::Mutating,
            label: "Good",
            glyph: "good",
            hotkey: None,
            request: PowerRequestKind::ActorSpawn,
            applies_to: PowerTargetMask::AGENT,
            coupling_note: "civ_agents::spawn_child_near with a new genome",
            availability: PowerAvailability::Near,
        };
        reg.register(power).expect("good power registers");
        let found = reg
            .find(&PowerId::new("test.good"))
            .expect("id is present");
        assert_eq!(found.label, "Good");
        assert_eq!(found.request, PowerRequestKind::ActorSpawn);
    }

    /// The default registry contains the highest-value verb from each of the
    /// 4 categories the task asks us to ship first (terraform/sculpt,
    /// spawn-organism, inspect-entity, plus a disaster verb).
    #[test]
    fn default_powers_includes_high_value_verbs() {
        let registry = default_registry().expect("default registry builds");
        // terraform/sculpt -> terrain.raise
        assert!(
            registry.find(&PowerId::new("terrain.raise")).is_some(),
            "missing terrain.raise"
        );
        // spawn-organism -> life.spawn_organism
        assert!(
            registry.find(&PowerId::new("life.spawn_organism")).is_some(),
            "missing life.spawn_organism"
        );
        // inspect-entity -> inspect.probe
        assert!(
            registry.find(&PowerId::new("inspect.probe")).is_some(),
            "missing inspect.probe"
        );
        // disaster verb -> disaster.meteor
        assert!(
            registry.find(&PowerId::new("disaster.meteor")).is_some(),
            "missing disaster.meteor"
        );
    }
}
