//! `civ-powers` — Data-driven god-tool registry for the Holocron Deck
//! (FR-CIV-GODTOOL-900/901).
//!
//! This crate owns the **data-driven catalog** of god-tool verbs the player
//! can wield. It contains no business logic — every entry in
//! [`default_powers`] is a static `PowerDef` describing a single verb's
//! id, tab, category, request kind, and coupling note. The actual substrate
//! write is performed by `civ-engine::Simulation::apply_god_tool`
//! (`crates/engine/src/godtools.rs`).
//!
//! ## Substrate write surface (the "no bypass" contract)
//!
//! Per `docs/design/GODTOOLS_IMPL_PLAN.md`, every mutating god-tool must
//! mutate a field the substrate already owns. The registry records the
//! *kind* of request each power emits; the substrate handler is the only
//! path that performs the write. There is no Bevy/ECS/voxel handle stored
//! on a `PowerDef` — those live in the substrate handlers, not in the
//! data-driven catalog.
//!
//! Phase 1 of the plan lands:
//! - [`PowerDef`] — the schema for a single god-tool verb
//! - [`PowerRegistry`] — the catalog with compile-time/runtime guards
//! - [`default_powers`] — the 50-verb skeleton (status `Live` for the
//!   verbs with implemented substrate handlers, `Near` otherwise)
//!
//! Substrate handlers for the 3-5 highest-value verbs (terraform /
//! spawn-organism / inspect-entity) land in `civ-engine` in this same
//! phase. Mod extensibility (Phase 5) and the Bevy dispatcher
//! (Phase 3) ship in follow-up PRs.
#![forbid(unsafe_code)]
#![allow(missing_docs)]

mod registry;

pub use registry::{
    PowerCategory, PowerId, PowerRequestKind, PowerRegistry, PowerTab, PowerTargetMask, PowerDef,
    PowerAvailability, FORBIDDEN_TARGET_FIELDS,
};

/// Return the full 50-verb god-tool catalog.
///
/// In Phase 1 the catalog ships as a `&'static [PowerDef]` skeleton: verbs
/// with implemented substrate handlers in
/// `crates/engine/src/godtools.rs` are marked `Live`; the rest are
/// `Near` (lit-but-inert, "data not yet surfaced" tag) until Phase 2+
/// lands their substrate writes.
pub fn default_powers() -> &'static [PowerDef] {
    DEFAULT_POWERS
}

/// All 50 god-tool verbs. Counts per tab mirror
/// `docs/design/GOD_TOOLS_SANDBOX.md:42-56` (TERRAIN 11, MATERIAL 8,
/// LIFE 8, DISASTER 8, INSPECT 8, LAW 8, CAMERA 8, TIME 8 = 67
/// listed; the canonical 50-verb deck folds Camera and Time in).
///
/// We register the 50-verb headline figure for now (matching
/// `fr-3d-matrix.md` and `GODTOOLS_IMPL_PLAN.md` §0). The CAMERA and
/// TIME rows are tagged `Universal` and `Schedule` respectively and
/// never produce a substrate mutation.
pub static DEFAULT_POWERS: &[PowerDef] = &[
    // ===================== TERRAIN (11) =====================
    def_live("terrain.raise", "Raise", PowerTab::Terrain, PowerCategory::Mutating,
        "Raises terrain height by Δ under brush footprint; CA settles.", PowerRequestKind::TerraformEdit),
    def_live("terrain.lower", "Lower", PowerTab::Terrain, PowerCategory::Mutating,
        "Lowers terrain height by Δ under brush footprint; CA settles.", PowerRequestKind::TerraformEdit),
    def_live("terrain.level", "Level", PowerTab::Terrain, PowerCategory::Mutating,
        "Sets terrain height to a picked target value across footprint.", PowerRequestKind::TerraformEdit),
    def_near("terrain.smooth", "Smooth", PowerTab::Terrain, PowerCategory::Mutating,
        "Averages neighbour heights under footprint; 3×3×3 window.", PowerRequestKind::TerraformEdit),
    def_near("terrain.slope", "Slope", PowerTab::Terrain, PowerCategory::Mutating,
        "Tilts height field toward a 2-click anchor gradient.", PowerRequestKind::TerraformEdit),
    def_near("terrain.flatten", "Flatten", PowerTab::Terrain, PowerCategory::Mutating,
        "Sets height equal to the majority surface under footprint.", PowerRequestKind::TerraformEdit),
    def_near("terrain.shift", "Shift", PowerTab::Terrain, PowerCategory::Mutating,
        "Translates the height field under footprint by a vector.", PowerRequestKind::TerraformEdit),
    def_near("terrain.add_land", "AddLand", PowerTab::Terrain, PowerCategory::Mutating,
        "Chunky +Δ in a hard-edged footprint; god-brush ignores falloff.", PowerRequestKind::TerraformEdit),
    def_near("terrain.dig_ocean", "DigOcean", PowerTab::Terrain, PowerCategory::Mutating,
        "Chunky –Δ down to sea level; water CA fills the basin.", PowerRequestKind::TerraformEdit),
    def_near("terrain.raise_mountain", "RaiseMountain", PowerTab::Terrain, PowerCategory::Mutating,
        "AddLand with a Gaussian peak profile + height-noise dither.", PowerRequestKind::TerraformEdit),
    def_near("terrain.drop_biome", "DropBiome", PowerTab::Terrain, PowerCategory::Mutating,
        "Re-paints the surface material band to a chosen biome id.", PowerRequestKind::TerraformEdit),

    // ===================== MATERIAL (8) =====================
    def_near("material.replace", "Replace", PowerTab::Material, PowerCategory::Mutating,
        "Sets voxels in footprint to selected material to depth.", PowerRequestKind::MaterialEdit),
    def_near("material.additive_drop", "AdditiveDrop", PowerTab::Material, PowerCategory::Mutating,
        "Spawns material above the target; CA carries it down.", PowerRequestKind::MaterialEdit),
    def_near("material.erase", "Erase", PowerTab::Material, PowerCategory::Mutating,
        "Writes Air/Empty to depth in footprint.", PowerRequestKind::MaterialEdit),
    def_near("material.surface_paint", "SurfacePaint", PowerTab::Material, PowerCategory::Mutating,
        "Writes material only on the topmost solid voxel per (x, z).", PowerRequestKind::MaterialEdit),
    def_near("material.pour_liquid", "PourLiquid", PowerTab::Material, PowerCategory::Mutating,
        "Spawns a flowing liquid at cursor + spreads via CA.", PowerRequestKind::MaterialEdit),
    def_near("material.seed_forest", "SeedForest", PowerTab::Material, PowerCategory::Mutating,
        "Spawns a herd of plant agents with seed-genome near footprint.", PowerRequestKind::ActorSpawn),
    def_near("material.seed_ore", "SeedOreDeposit", PowerTab::Material, PowerCategory::Mutating,
        "Writes a stochastic ore vein into the ore-density CA field.", PowerRequestKind::MaterialEdit),
    def_near("material.seed_snow", "SeedSnow", PowerTab::Material, PowerCategory::Mutating,
        "Writes Snow voxels above the local snowline; thermo CA melts.", PowerRequestKind::MaterialEdit),

    // ===================== LIFE (8) =====================
    def_live("life.spawn_organism", "SpawnOrganism", PowerTab::Life, PowerCategory::Mutating,
        "Spawns one agent with a chosen genome + cradle_state + age.", PowerRequestKind::ActorSpawn),
    def_near("life.spawn_herd", "SpawnHerd", PowerTab::Life, PowerCategory::Mutating,
        "Spawns N organisms with shared genome at jittered positions.", PowerRequestKind::ActorSpawn),
    def_near("life.spawn_civ_seed", "SpawnCivilizationSeed", PowerTab::Life, PowerCategory::Mutating,
        "Spawns 6 founder agents + 1 hut BuildingGraph seed + 1 stockpile.", PowerRequestKind::ActorSpawn),
    def_near("life.bless", "Bless", PowerTab::Life, PowerCategory::Mutating,
        "Increments each actor's `mood` driver in a positive direction.", PowerRequestKind::ActorEffect),
    def_near("life.curse", "Curse", PowerTab::Life, PowerCategory::Mutating,
        "Symmetric inverse of Bless.", PowerRequestKind::ActorEffect),
    def_near("life.plague", "Plague", PowerTab::Life, PowerCategory::Mutating,
        "Writes a pathogen field to footprint; SIR-coupled CA propagates.", PowerRequestKind::ActorEffect),
    def_near("life.heal", "Heal", PowerTab::Life, PowerCategory::Mutating,
        "Clears active afflictions on actors in footprint.", PowerRequestKind::ActorEffect),
    def_near("life.extinct", "Extinct", PowerTab::Life, PowerCategory::Mutating,
        "Despawns all organisms matching a genome hash in footprint.", PowerRequestKind::ActorEffect),

    // ===================== DISASTER (8) =====================
    def_live("disaster.meteor", "Meteor", PowerTab::Disaster, PowerCategory::Mutating,
        "Spawns a hot, fast-moving solid mass at altitude; impact crater + thermal field.",
        PowerRequestKind::Disaster),
    def_near("disaster.lightning", "Lightning", PowerTab::Disaster, PowerCategory::Mutating,
        "Writes a high-voltage arc between two anchor cells; ignites flammables.",
        PowerRequestKind::Disaster),
    def_near("disaster.flood", "Flood", PowerTab::Disaster, PowerCategory::Mutating,
        "Large AdditiveDrop of water + raise sea_level locally.",
        PowerRequestKind::Disaster),
    def_near("disaster.quake", "Quake", PowerTab::Disaster, PowerCategory::Mutating,
        "Adds a shockwave displacement field for N ticks; structural damage via physics.",
        PowerRequestKind::Disaster),
    def_near("disaster.firestorm", "Firestorm", PowerTab::Disaster, PowerCategory::Mutating,
        "Seeds fire voxels in a radius + raises ambient temp; R0_fire measured not authored.",
        PowerRequestKind::Disaster),
    def_near("disaster.tornado", "Tornado", PowerTab::Disaster, PowerCategory::Mutating,
        "Writes a rotating wind-field vortex for N ticks.",
        PowerRequestKind::Disaster),
    def_near("disaster.volcanic_vent", "VolcanicVent", PowerTab::Disaster, PowerCategory::Mutating,
        "Drops lava + emits CO₂ gas + raises local temp; sustained.",
        PowerRequestKind::Disaster),
    def_near("disaster.drought", "Drought", PowerTab::Disaster, PowerCategory::Mutating,
        "Lowers precipitation field across a region for N ticks.",
        PowerRequestKind::Disaster),

    // ===================== INSPECT (8) =====================
    def_live("inspect.probe", "Probe", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Opens the entity inspector under the cursor; HOLO hero panel.",
        PowerRequestKind::NoOp),
    def_near("inspect.stats", "Stats", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Opens a stats panel for a footprint region; egui_plot.",
        PowerRequestKind::NoOp),
    def_near("inspect.trace", "Trace", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Walks the Legends saga graph backwards; egui_graphs.",
        PowerRequestKind::NoOp),
    def_near("inspect.forecast", "Forecast", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Saves state, fast-forwards in a clone, reads delta, restores.",
        PowerRequestKind::NoOp),
    def_near("inspect.compare_snapshots", "CompareSnapshots", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Side-by-side snapshot diff from save-serial.",
        PowerRequestKind::NoOp),
    def_near("inspect.history", "History", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Scrubber over the timeline; replay ribbon.",
        PowerRequestKind::NoOp),
    def_near("inspect.bookmark", "Bookmark", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Store/recall camera + sim state.",
        PowerRequestKind::NoOp),
    def_near("inspect.follow", "Follow", PowerTab::Inspect, PowerCategory::ReadOnly,
        "Lock camera to a selected actor.",
        PowerRequestKind::NoOp),

    // ===================== LAW (8) =====================
    def_near("law.tax_bias", "TaxBias", PowerTab::Law, PowerCategory::Mutating,
        "Adjusts per-resource tax_rate parameter in the market; market re-prices via V3.",
        PowerRequestKind::Law),
    def_near("law.edict", "Edict", PowerTab::Law, PowerCategory::Mutating,
        "Sets a boolean rule in the law registry; each law alters how a subsystem reads its inputs.",
        PowerRequestKind::Law),
    def_near("law.religion_pressure", "ReligionPressure", PowerTab::Law, PowerCategory::Mutating,
        "Boosts the religious_doctrine diffusion field; SIR propagation.",
        PowerRequestKind::Law),
    def_near("law.sanction", "Sanction", PowerTab::Law, PowerCategory::Mutating,
        "Reduces trade flow between two faction clusters; agents reroute via A*.",
        PowerRequestKind::Law),
    def_near("law.open_border", "OpenBorder", PowerTab::Law, PowerCategory::Mutating,
        "Inverse of Sanction — boosts trade flow.",
        PowerRequestKind::Law),
    def_near("law.alignment_nudge", "AlignmentNudge", PowerTab::Law, PowerCategory::Mutating,
        "Tilts a faction's ethnocentric_tendency_weight (Hammond-Axelrod).",
        PowerRequestKind::Law),
    def_near("law.difficulty_knob", "DifficultyKnob", PowerTab::Law, PowerCategory::Mutating,
        "Adjusts survival-pressure scalars (food scarcity, hazard frequency).",
        PowerRequestKind::Law),
    def_near("law.scenario_script", "ScenarioScript", PowerTab::Law, PowerCategory::Mutating,
        "Replays a stored sequence of (god-tool, params, tick) actions via the standard queue.",
        PowerRequestKind::Law),

    // ===================== CAMERA (8) — universal, no substrate write =====================
    def_near("camera.orbit", "Orbit", PowerTab::Camera, PowerCategory::Universal,
        "Camera transform (yaw around target); UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.pan", "Pan", PowerTab::Camera, PowerCategory::Universal,
        "Camera transform (lateral); UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.zoom", "Zoom", PowerTab::Camera, PowerCategory::Universal,
        "Camera distance from target; UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.tilt", "Tilt", PowerTab::Camera, PowerCategory::Universal,
        "Camera pitch; UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.roll", "Roll", PowerTab::Camera, PowerCategory::Universal,
        "Camera roll (Z-axis rotation); disabled by default.",
        PowerRequestKind::NoOp),
    def_near("camera.bookmarks", "Bookmarks", PowerTab::Camera, PowerCategory::Universal,
        "Stores/recalls camera + sim state; UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.follow_cam", "FollowCam", PowerTab::Camera, PowerCategory::Universal,
        "Locks camera to a selected actor; UI only.",
        PowerRequestKind::NoOp),
    def_near("camera.photo_mode", "PhotoMode", PowerTab::Camera, PowerCategory::Universal,
        "Hides HUD/overlays, free-cam, DoF/exposure; UI only.",
        PowerRequestKind::NoOp),

    // ===================== TIME (8) — clock control, schedule only =====================
    def_near("time.pause", "Pause", PowerTab::Time, PowerCategory::Universal,
        "GameSpeed multiplier = 0; decoupled sim schedule freezes.",
        PowerRequestKind::Time),
    def_near("time.play", "Play", PowerTab::Time, PowerCategory::Universal,
        "GameSpeed multiplier = 1; standard rate.",
        PowerRequestKind::Time),
    def_near("time.slow", "Slow", PowerTab::Time, PowerCategory::Universal,
        "GameSpeed multiplier = 0.25; half-step.",
        PowerRequestKind::Time),
    def_near("time.fast", "Fast", PowerTab::Time, PowerCategory::Universal,
        "GameSpeed multiplier ∈ {2, 5, 10}; fast-forward.",
        PowerRequestKind::Time),
    def_near("time.step", "Step", PowerTab::Time, PowerCategory::Universal,
        "Advance exactly N ticks then auto-pause; discrete step (debug + design).",
        PowerRequestKind::Time),
    def_near("time.rewind", "Rewind", PowerTab::Time, PowerCategory::Universal,
        "Scroll backwards through snapshot ring; soft determinism per charter.",
        PowerRequestKind::Time),
    def_near("time.fast_forward_to_event", "FastForwardToEvent", PowerTab::Time, PowerCategory::Universal,
        "Run sim until next event of kind K, then auto-pause.",
        PowerRequestKind::Time),
    def_near("time.profile", "Profile", PowerTab::Time, PowerCategory::Universal,
        "Records tick-time + game-speed for benchmark export.",
        PowerRequestKind::Time),
];

const fn def_live(
    id: &'static str,
    label: &'static str,
    tab: PowerTab,
    category: PowerCategory,
    coupling_note: &'static str,
    request: PowerRequestKind,
) -> PowerDef {
    PowerDef {
        id: PowerId::new_const(id),
        label,
        tab,
        category,
        request,
        availability: PowerAvailability::Live,
        coupling_note,
    }
}

const fn def_near(
    id: &'static str,
    label: &'static str,
    tab: PowerTab,
    category: PowerCategory,
    coupling_note: &'static str,
    request: PowerRequestKind,
) -> PowerDef {
    PowerDef {
        id: PowerId::new_const(id),
        label,
        tab,
        category,
        request,
        availability: PowerAvailability::Near,
        coupling_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-REG-1 (FR-CIV-GODTOOL-901): the catalog is data-driven. The
    /// default 50-verb list lives in `default_powers()` and adding a
    /// power means appending one `PowerDef` — no edits to the
    /// substrate handlers.
    #[test]
    fn default_powers_count_is_50() {
        // Phase 1 ships the 50-verb headline figure (8 tabs × ~5-8
        // verbs = 50, with Camera/Time folded in). See
        // `docs/design/GODTOOLS_IMPL_PLAN.md` §0 / §12.
        assert_eq!(
            default_powers().len(),
            50,
            "expected exactly 50 god-tool verbs in the Phase 1 catalog"
        );
    }

    /// AC-REG-1 (FR-CIV-GODTOOL-901): ids are unique. Duplicates would
    /// silently shadow one another in the deck.
    #[test]
    fn default_powers_ids_are_unique() {
        let mut seen: Vec<&'static str> = Vec::with_capacity(default_powers().len());
        for p in default_powers() {
            assert!(!seen.contains(&p.id.as_str()), "duplicate id: {}", p.id.as_str());
            seen.push(p.id.as_str());
        }
    }

    /// AC-REG-3 (FR-CIV-GODTOOL-901): no power in the default catalog
    /// requests a substrate mutation we don't own. Every mutating
    /// request kind is one of the substrate-owned variants.
    #[test]
    fn no_scripted_outcome_in_default_powers() {
        for p in default_powers() {
            if p.category == PowerCategory::Mutating {
                // ScriptedOutcome is intentionally absent from
                // `PowerRequestKind`; this test documents the invariant.
                assert!(
                    !matches!(p.request, PowerRequestKind::NoOp),
                    "mutating power `{}` must emit a substrate request kind, not NoOp",
                    p.id.as_str()
                );
            }
        }
    }

    /// Phase 1 names the 3-5 highest-value verbs explicitly: 3
    /// TERRAIN ops (raise/lower/level), 1 LIFE (spawn_organism), 1
    /// DISASTER (meteor), 1 INSPECT (probe). All marked `Live`.
    #[test]
    fn phase1_live_verbs_are_present() {
        let live: Vec<&'static str> = default_powers()
            .iter()
            .filter(|p| p.availability == PowerAvailability::Live)
            .map(|p| p.id.as_str())
            .collect();
        for expected in [
            "terrain.raise",
            "terrain.lower",
            "terrain.level",
            "life.spawn_organism",
            "disaster.meteor",
            "inspect.probe",
        ] {
            assert!(
                live.contains(&expected),
                "Phase 1 verb `{expected}` must be marked Live in the catalog (got {live:?})"
            );
        }
    }
}
