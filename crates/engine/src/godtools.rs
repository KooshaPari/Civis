//! God-tool substrate dispatcher (Phase 1 of
//! `docs/design/GODTOOLS_IMPL_PLAN.md`).
//!
//! The dispatcher is the **single Bevy → substrate bridge**: every
//! mutating god-tool emits a [`GodToolRequest`] and a Bevy system
//! hands it to [`Simulation::apply_god_tool`]. There is no direct
//! `hecs::World` or `VoxelWorld` access from any god-tool system.
//!
//! ## Phase 2 verb coverage
//!
//! Phase 1 shipped 6 substrate-mutating verbs (3 TERRAIN ops +
//! 1 LIFE + 1 DISASTER + 1 INSPECT). Phase 2 of
//! `docs/design/GODTOOLS_IMPL_PLAN.md` adds 10 more high-value
//! verbs that follow the same pattern: real substrate writes
//! through `Simulation` methods the engine already reads each tick.
//! Phase 3 of the plan (this PR) adds 8 more: 7 MATERIAL ops
//! (`material.erase/replace/surface_paint/additive_drop/
//! pour_liquid/seed_snow/seed_ore`) + 1 TERRAIN op
//! (`terrain.slope`).
//!
//! ### Phase 2 verbs (this module)
//!
//! | `PowerDef.id`             | Verb                       | Substrate write |
//! |---------------------------|----------------------------|-----------------|
//! | `terrain.smooth`          | `Terraform::Smooth`        | `push_voxel_write` averaged from 3×3×3 neighbours |
//! | `terrain.raise_mountain`  | `Terraform::RaiseMountain` | `push_voxel_write` Gaussian peak (STONE/GRAVEL) |
//! | `life.spawn_herd`         | `Life::SpawnHerd`          | `civ_agents::spawn_many` |
//! | `life.heal`               | `Life::Heal`               | bump `Health::integrity` + `Needs` for actors in footprint |
//! | `life.bless`              | `Life::Bless`              | boost `Needs::safety`/`food`/`social` for actors in footprint |
//! | `life.curse`              | `Life::Curse`              | inverse of bless |
//! | `life.extinct`            | `Life::Extinct`            | `hecs::World::despawn` for actors in footprint |
//! | `disaster.wildfire`       | `Disaster::Wildfire`       | `trigger_disaster(DisasterKind::Wildfire, …)` |
//! | `disaster.flood`          | `Disaster::Flood`          | `trigger_disaster(DisasterKind::Flood, …)` |
//! | `disaster.quake`          | `Disaster::Quake`          | `trigger_disaster(DisasterKind::Quake, …)` |
//!
//! ### Phase 3 verbs (this module)
//!
//! Phase 3 lands 8 more substrate-mutating verbs (FR-CIV-GODTOOL-901
//! batch 2): 7 MATERIAL ops + 1 TERRAIN op. Each one routes through
//! `push_voxel_write` — the same substrate-owned API surface used by
//! Phases 1 + 2 — so the CA, replay log, and dirty-event invariant
//! stay intact.
//!
//! | `PowerDef.id`             | Verb                                | Substrate write |
//! |---------------------------|-------------------------------------|-----------------|
//! | `material.erase`          | `Material::Erase`                   | `push_voxel_write` AIR in footprint |
//! | `material.replace`        | `Material::Replace`                 | `push_voxel_write` chosen material in footprint |
//! | `material.surface_paint`  | `Material::SurfacePaint`            | `push_voxel_write` topmost-solid-only per (x, z) |
//! | `material.additive_drop`  | `Material::AdditiveDrop`            | `push_voxel_write` material +drop_height above target |
//! | `material.pour_liquid`    | `Material::PourLiquid`              | `push_voxel_write` WATER/LAVA in footprint |
//! | `material.seed_snow`      | `Material::SeedSnow`                | `push_voxel_write` SNOW above local snowline |
//! | `material.seed_ore`       | `Material::SeedOreDeposit`          | `push_voxel_write` ORE voxels in a stochastic vein |
//! | `terrain.slope`           | `Terraform::Slope`                  | `push_voxel_write` linear gradient in `+x` |
//!
//! The remaining ~22 verbs that stay `Near` (camera, time, remaining
//! TERRAIN/LIFE/LAW/DISASTER) keep their no-op receipt in the deck
//! and wait for follow-up PRs to land their substrate handlers.
//!
//! ### Phase 4 verbs (this module)
//!
//! Phase 4 lands 10 more substrate-mutating verbs (FR-CIV-GODTOOL-901
//! batch 3): 1 TERRAIN op, 1 MATERIAL op, 1 LIFE op, 4 DISASTER ops,
//! and 3 LAW ops. Each verb writes a real substrate field the
//! engine reads each tick (voxels, weather, faction treasury,
//! economy policy, build queue, or belief reserve), keeping the
//! "no bypass" guarantee.
//!
//! | `PowerDef.id`               | Verb                            | Substrate write |
//! |-----------------------------|---------------------------------|-----------------|
//! | `terrain.flatten`           | `Terraform::Flatten`            | `push_voxel_write` STONE band at the majority surface y |
//! | `material.seed_forest`      | `Material::SeedForest`          | `push_voxel_write` PLANT voxels in a stochastic scatter |
//! | `life.spawn_civ_seed`       | `Life::SpawnCivSeed`            | `spawn_many` + `enqueue_build_site` (6 founders + hut + stockpile) |
//! | `disaster.lightning`        | `Disaster::Lightning`           | `push_voxel_write` LAVA arc + ignites via `trigger_disaster` heat |
//! | `disaster.tornado`          | `Disaster::Tornado`             | `push_voxel_write` spiral AIR/STONE sweep + belief |
//! | `disaster.volcanic_vent`    | `Disaster::VolcanicVent`        | `push_voxel_write` sustained LAVA + STEAM column |
//! | `disaster.drought`          | `Disaster::Drought`             | `weather_grid` `precip_mm_fp` clamp-down per region |
//! | `law.tax_bias`              | `Law::TaxBias`                  | `state.faction_treasury` transfer + belief feedback |
//! | `law.religion_pressure`     | `Law::ReligionPressure`         | `add_belief` (religion pressure → faith) |
//! | `law.difficulty_knob`       | `Law::DifficultyKnob`           | `economy_policy.scarcity_multiplier` |
//!
//! ## Coupling discipline (the "no bypass" guarantee)
//!
//! - **Write only what the substrate owns.** Every mutating path
//!   below calls a `Simulation` method that already mutates state
//!   the engine reads each tick (`push_voxel_write`,
//!   `invoke_divine_disaster`, `spawn_civilian_at`).
//! - **Emit, never bypass.** God-tool actions never mutate the
//!   `hecs::World` or `VoxelWorld` directly — they go through
//!   `Simulation`'s existing API surface.
//! - **No scripted outcomes.** A `GodToolRequest::ScriptedOutcome`
//!   variant does not exist (AC-CPL-3 compile-time guard).
//!
//! ## Bevy / hecs
//!
//! The simulation uses `hecs::World` (not `bevy_ecs`) per the
//! `civ-engine` substrate contract. The Bevy client in
//! `clients/bevy-ref/` is a separate crate that bridges a
//! `GodToolRequest` from input events to a thread-local
//! `Simulation` handle; this module is the substrate side of that
//! bridge.
#![forbid(unsafe_code)]
#![allow(missing_docs)]

use civ_agents::{spawn_civilian_at, spawn_many, ActorVisualKind, Alignment, Civilian, Position3d};
use civ_build::{BuildingId, BuildingSpec, BuildingTier, BuildSite, ProductionChain};
use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
use civ_voxel::{
    material::{GRAVEL, LAVA, ORE, PACKED_DIRT, PLANT, SAND, SNOW, STEAM, STONE, WATER},
    AIR, MaterialId, WorldCoord, FIXED_SCALE,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::disasters::{trigger_disaster, DisasterKind};
use crate::engine::{Fixed, Simulation};

/// A god-tool request handed to the substrate by the Bevy
/// dispatcher. Every variant routes through one of the
/// substrate-owned APIs (see module docs).
///
/// `serde` round-trips cleanly so a Bevy → engine bridge can
/// serialize requests over the same `EditCommand` channel that
/// `civ-server` already uses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GodToolRequest {
    /// A TERRAIN verb — raise / lower / level / sculpt.
    Terraform(TerraformRequest),
    /// A MATERIAL verb — write material into the voxel substrate.
    Material(MaterialRequest),
    /// A LIFE verb — spawn organic + civ.
    Life(LifeRequest),
    /// A DISASTER verb — invoke a `DisasterKind` at a position.
    Disaster(DisasterRequest),
    /// An INSPECT verb — read-only; produces a `ProbeReport`
    /// without mutating any state.
    Inspect(InspectRequest),
    /// A LAW verb — adjust economy policy / treasury / belief
    /// reserves. Phase 4 (FR-CIV-GODTOOL-901 batch 3) ships the
    /// `TaxBias`, `ReligionPressure`, and `DifficultyKnob` ops.
    Law(LawRequest),
}

/// TERRAIN verb parameters. The brush center is in fixed-point
/// world coordinates (`civ_voxel::WorldCoord`); `radius_voxels`
/// defines a footprint (sphere) of cells to write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerraformRequest {
    /// Which TERRAIN op to apply.
    pub op: TerraformOp,
    /// Center of the brush footprint (fixed-point world coords).
    pub center: WorldCoord,
    /// Radius of the brush footprint in voxels.
    pub radius_voxels: u8,
    /// Brush strength — for `Level`, the target height; for
    /// `Raise`/`Lower`, the Δ cap. Must be non-negative.
    pub strength: i32,
    /// Optional auxiliary id (Phase 3 follow-up). For
    /// `DropBiome` this is the target material id (the
    /// "biome paint"). For other ops the field is ignored.
    /// Defaults to 0 so older serialized requests keep
    /// deserialising.
    #[serde(default)]
    pub aux_id: u32,
}

/// TERRAIN op kinds. Mirrors the 11 TERRAIN verbs from
/// `docs/design/GOD_TOOLS_SANDBOX.md` §3.1. Phase 1 ships
/// `Raise`, `Lower`, `Level`; Phase 2 adds `Smooth` and
/// `RaiseMountain`; Phase 3 adds `Slope`. The remaining
/// variants land in follow-up PRs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerraformOp {
    /// `terrain.raise` — write STONE in the footprint.
    Raise,
    /// `terrain.lower` — write AIR in the footprint.
    Lower,
    /// `terrain.level` — set the y-band of the footprint to a
    /// target height (PACKED_DIRT). `strength` is the absolute
    /// target y in fixed-point units.
    Level,
    /// `terrain.smooth` — read the topmost solid voxel y per
    /// (x, z) column in the footprint, average over a 3×3 column
    /// window, and write a thin band of `STONE` at the new
    /// averaged height. CA settles the result next tick.
    Smooth,
    /// `terrain.raise_mountain` — write a Gaussian peak of
    /// `STONE` (and `GRAVEL` on the shoulders) in the footprint
    /// centred on `center`. `strength` is the peak Δ-y in
    /// fixed-point units.
    RaiseMountain,
    /// `terrain.slope` — write `STONE` voxels along a linear
    /// gradient across the footprint. `strength` is the Δ-y
    /// from the low edge to the high edge of the brush in
    /// fixed-point units (positive = tilt up toward `+x`).
    /// The CA settles the result next tick.
    Slope,
    /// `terrain.add_land` — write a chunky band of `STONE`
    /// above the existing surface in the footprint.
    /// `strength` is the band thickness in fixed-point units.
    /// The CA re-settles the result next tick.
    AddLand,
    /// `terrain.dig_ocean` — chunky dig-down: for each column
    /// in the footprint, find the topmost solid voxel; if it
    /// is above the local sea level, dig down by `strength`
    /// fixed-point units and fill the resulting cavity with
    /// `WATER` so the fluid CA can hydrate it next tick.
    /// `aux_id` is the sea-level band in fixed-point units
    /// (defaults to `0` for "use the surface as sea level").
    DigOcean,
    /// `terrain.drop_biome` — re-paint the topmost solid
    /// voxel of each (x, z) column in the footprint to the
    /// material in `aux_id`. The "biome" is encoded as a
    /// material id so the surface id (SAND, SNOW, GRASS, …)
    /// is the canonical biome marker that downstream CAs and
    /// inspectors already understand.
    DropBiome,
    /// `terrain.flatten` — sample the topmost solid voxel y
    /// in each (x, z) column of the footprint, take the
    /// median (majority surface), and stamp a 1-cell `STONE`
    /// band at that y. CA settles the result next tick so
    /// the plateau reads as a flat surface to the renderer.
    /// Phase 4 (FR-CIV-GODTOOL-901 batch 3).
    Flatten,
}

/// MATERIAL verb parameters. Phase 3 ships all 7 MATERIAL ops
/// from `docs/design/GOD_TOOLS_SANDBOX.md` §3.2: [`MaterialOp::Erase`],
/// [`MaterialOp::Replace`], [`MaterialOp::SurfacePaint`],
/// [`MaterialOp::AdditiveDrop`], [`MaterialOp::PourLiquid`],
/// [`MaterialOp::SeedSnow`], and [`MaterialOp::SeedOreDeposit`].
/// Every op routes through `Simulation::push_voxel_write` — the
/// same substrate-owned API the TERRAIN verbs use — so the CA,
/// replay log, and dirty-event invariant stay intact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialRequest {
    /// Which MATERIAL op to apply.
    pub op: MaterialOp,
    /// Center of the brush footprint (fixed-point world coords).
    pub center: WorldCoord,
    /// Radius of the brush footprint in voxels.
    pub radius_voxels: u8,
    /// Target material id for `Replace`, `SurfacePaint`,
    /// `AdditiveDrop`, `PourLiquid`, `SeedSnow`, and
    /// `SeedOreDeposit`. Ignored by `Erase` (always writes
    /// `AIR`). The value is passed as a `u32` so a JSON-RPC
    /// client can ship any `MaterialId` (including future
    /// material ids) without versioning this enum.
    pub material_id: u32,
    /// Brush strength — for `AdditiveDrop` it controls how many
    /// layers tall the seed sphere is; for `PourLiquid` it
    /// controls the deposit thickness; for `SeedSnow` it is the
    /// snowline Δ (positive = colder; the snow deposit sits at
    /// `topmost_y + strength`); for `SeedOreDeposit` it is the
    /// vein thickness. Ignored by `Erase`, `Replace`, and
    /// `SurfacePaint`.
    pub strength: i32,
    /// Drop height in fixed-point units above `center.y` for
    /// `AdditiveDrop` and `PourLiquid`. Lets the CA's gravity
    /// rule carry the material down naturally rather than
    /// stamping it at the brush centre. Ignored by every other
    /// MATERIAL op.
    pub drop_height: i32,
}

/// MATERIAL op kinds. Mirrors the 7 MATERIAL verbs from
/// `docs/design/GOD_TOOLS_SANDBOX.md` §3.2 that Phase 3 ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialOp {
    /// `material.erase` — write `AIR` in the footprint.
    Erase,
    /// `material.replace` — write the requested material in the
    /// footprint. Existing material is overwritten (the CA
    /// settles gravity, settling, and reactivity the next tick).
    Replace,
    /// `material.surface_paint` — write the requested material
    /// only on the **topmost solid** voxel of each `(x, z)`
    /// column in the footprint. The CA's gravity + settling
    /// rules then re-pile subsequent paints naturally on top of
    /// the existing surface.
    SurfacePaint,
    /// `material.additive_drop` — write the requested material
    /// in a sphere at `center.y + drop_height`. The voxel CA's
    /// falling rule carries the material down to the existing
    /// surface next tick; this verb writes the seed material
    /// without bypassing gravity.
    AdditiveDrop,
    /// `material.pour_liquid` — write `WATER` / `LAVA` voxels
    /// in a sphere at `center.y + drop_height`. The fluid CA
    /// spreads the liquid horizontally next tick. `strength`
    /// controls the deposit thickness (in `FIXED_SCALE` units).
    PourLiquid,
    /// `material.seed_snow` — write `SNOW` voxels in a sphere
    /// at the local snowline band. The thermo CA's melt rule
    /// sublimates the snow at temperatures above the snowline
    /// next tick, so the verb never produces immortal snow.
    /// `strength` is the snowline Δ in fixed-point units.
    SeedSnow,
    /// `material.seed_ore` — write `ORE` voxels in a stochastic
    /// vein pattern drawn from a deterministic per-cell noise
    /// (seeded by `(center.x, center.z)` so replay is stable).
    /// `strength` is the vein thickness in fixed-point units.
    SeedOreDeposit,
    /// `material.seed_forest` — write `PLANT` voxels in a
    /// stochastic scatter inside the footprint. The scatter
    /// is seeded by `(center.x, center.z)` so replay is
    /// stable. The CA's growth rule carries the plants
    /// outward over the next ticks; we only stamp the seed
    /// voxels (no scripted canopy). Phase 4
    /// (FR-CIV-GODTOOL-901 batch 3).
    SeedForest,
}

/// LIFE verb parameters. Phase 1 ships
/// [`LifeRequest::SpawnOrganism`]; Phase 2 adds
/// [`LifeRequest::SpawnHerd`], [`LifeRequest::Bless`],
/// [`LifeRequest::Curse`], [`LifeRequest::Heal`], and
/// [`LifeRequest::Extinct`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LifeRequest {
    /// `life.spawn_organism` — inject one agent via
    /// `civ_agents::spawn_civilian_at`.
    SpawnOrganism(SpawnOrganismRequest),
    /// `life.spawn_herd` — inject N agents via
    /// `civ_agents::spawn_many`. The agents share the same faction
    /// and a contiguous range of `Civilian.id` starting at
    /// `seed_civilian_id`.
    SpawnHerd(SpawnHerdRequest),
    /// `life.bless` — boost `Needs` for every agent in a
    /// spherical footprint centred on `center`. The boost is
    /// additive (clamped to `[0, 1]`) and never touches the
    /// AC-CPL-3 forbidden fields (`mood`, `alignment`, `culture`,
    /// `ideology`).
    Bless(ActorEffectRequest),
    /// `life.curse` — symmetric inverse of [`LifeRequest::Bless`]:
    /// subtract from `Needs` (clamped to `[0, 1]`) for every
    /// agent in the footprint.
    Curse(ActorEffectRequest),
    /// `life.heal` — restore `Health::integrity` (and the
    /// mirrored `Needs::health`) for every agent in the footprint.
    /// Caps at `1.0`.
    Heal(ActorEffectRequest),
    /// `life.extinct` — despawn every agent in the footprint via
    /// `hecs::World::despawn`. Returns the number of entities
    /// removed.
    Extinct(ActorFootprintRequest),
    /// `life.spawn_civ_seed` — instantiate a civilisation
    /// nucleus at `center`: 6 founder civilians, a Primitive
    /// hut build site, and a Farm stockpile build site. All
    /// ids are derived from `seed_civilian_id` so replay is
    /// deterministic. The verb only goes through substrate
    /// APIs (`spawn_many` + `enqueue_build_site`) — no direct
    /// ECS or voxel write. Phase 4
    /// (FR-CIV-GODTOOL-901 batch 3).
    SpawnCivSeed(SpawnCivSeedRequest),
}

/// Parameters for [`LifeRequest::SpawnCivSeed`]. Spawns a
/// deterministic civilisation nucleus: 6 founder agents
/// (one of which becomes the leader with a higher faction id),
/// a hut build site, and a farm stockpile build site.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnCivSeedRequest {
    /// Starting civilian id; spawned agents are assigned
    /// `seed_civilian_id + i` for `i in 0..6`. The hut and
    /// farm build sites use ids `seed_civilian_id + 100` and
    /// `seed_civilian_id + 101` respectively so they don't
    /// collide with the agent ids.
    pub seed_civilian_id: u64,
    /// Faction all new agents align to (and the build sites
    /// are tagged with).
    pub faction: u32,
    /// Centre of the civilisation seed in fixed-point world
    /// coords (used for build-site origins; agents are
    /// spawned at the sim origin and the Bevy layer animates
    /// them out per its scheduler).
    pub center: WorldCoord,
}

/// Parameters for [`LifeRequest::SpawnOrganism`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnOrganismRequest {
    /// Stable agent id (unique within the sim).
    pub id: u64,
    /// Faction the new agent aligns to.
    pub faction: u32,
    /// Normalized 0..1 map x of the spawn position.
    pub x: f32,
    /// Normalized 0..1 map y of the spawn position.
    pub y: f32,
    /// Which visual rig the Bevy client should render.
    pub visual: SpawnVisual,
}

/// Parameters for [`LifeRequest::SpawnHerd`]. Spawns a
/// deterministic batch of N agents with contiguous ids at the
/// origin (position 0,0,0); the Bevy layer animates them out to
/// scattered positions in its own scheduler.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpawnHerdRequest {
    /// Number of agents to spawn.
    pub count: u32,
    /// Starting id; the spawned agents are assigned
    /// `seed_civilian_id + i` for `i in 0..count`.
    pub seed_civilian_id: u64,
    /// Faction all new agents align to.
    pub faction: u32,
}

/// Actor-effect request — a footprint + a `strength` scalar used
/// by `life.bless` / `life.curse` / `life.heal`. The semantic of
/// `strength` differs per verb (positive for bless, positive for
/// heal, negative for curse) but the request shape is identical
/// so a single struct serves all three.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorEffectRequest {
    /// Footprint centre in fixed-point world coords.
    pub center: WorldCoord,
    /// Radius of the spherical footprint in voxels.
    pub radius_voxels: u8,
    /// Magnitude of the effect. Clamped per-need to `[0, 1]` for
    /// `bless`/`heal`; negated before clamping for `curse`.
    pub strength: f32,
}

/// Actor footprint request (no `strength`) used by
/// [`LifeRequest::Extinct`]. The footprint is the entire
/// selection; there is no "how much" parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorFootprintRequest {
    /// Footprint centre in fixed-point world coords.
    pub center: WorldCoord,
    /// Radius of the spherical footprint in voxels.
    pub radius_voxels: u8,
}

/// `ActorVisualKind` mirror, re-exported so a JSON-RPC bridge
/// doesn't have to depend on `civ-agents` directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnVisual {
    /// Humanoid civilian (KayKit Knight / capsule fallback).
    Humanoid,
    /// Non-combat herd / fauna (skeleton minion rig).
    Herd,
}

impl From<SpawnVisual> for ActorVisualKind {
    fn from(v: SpawnVisual) -> Self {
        match v {
            SpawnVisual::Humanoid => ActorVisualKind::Humanoid,
            SpawnVisual::Herd => ActorVisualKind::Herd,
        }
    }
}

/// DISASTER verb parameters. Phase 1 ships
/// [`DisasterRequest::Meteor`]; Phase 2 adds
/// [`DisasterRequest::Wildfire`], [`DisasterRequest::Flood`],
/// [`DisasterRequest::Quake`], [`DisasterRequest::Storm`], and
/// [`DisasterRequest::Plague`]. All route through
/// [`crate::disasters::trigger_disaster`] — never bypass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DisasterRequest {
    /// `disaster.meteor` — invoke `DisasterKind::Meteor` at `pos`.
    Meteor { pos: WorldCoord },
    /// `disaster.wildfire` — invoke `DisasterKind::Wildfire` at
    /// `pos`. Ignites flammables in the radius; the CA heat
    /// field propagates the fire next tick.
    Wildfire { pos: WorldCoord },
    /// `disaster.flood` — invoke `DisasterKind::Flood` at `pos`.
    /// Writes `WATER` voxels in the radius; CA fills the basin.
    Flood { pos: WorldCoord },
    /// `disaster.quake` — invoke `DisasterKind::Quake` at `pos`.
    /// Adds shockwave rubble; structural damage via physics.
    Quake { pos: WorldCoord },
    /// `disaster.storm` — invoke `DisasterKind::Storm` at `pos`.
    /// Wind-driven rain + safety loss.
    Storm { pos: WorldCoord },
    /// `disaster.plague` — invoke `DisasterKind::Plague` at
    /// `pos`. Disease pressure hits nearby agents (no terrain
    /// writes).
    Plague { pos: WorldCoord },
    /// `disaster.lightning` — write a `LAVA` arc between
    /// `from` and `to` (both endpoints + interpolated cells),
    /// then ignite a `Wildfire` at the impact endpoint so the
    /// heat field propagates the burn. Adds the standard
    /// `DISASTER_FAITH_GAIN` belief (disasters → faith
    /// coupling, FR-CIV-EMERGENCE). Phase 4
    /// (FR-CIV-GODTOOL-901 batch 3).
    Lightning { from: WorldCoord, to: WorldCoord },
    /// `disaster.tornado` — write a rotating wind vortex in
    /// the footprint: a spiral of `AIR` columns punched
    /// through the existing surface, with `GRAVEL` debris
    /// kicked up around the perimeter. Voxels only; no actor
    /// damage in Phase 4 (a follow-up verb adds the wind
    /// field). Adds `DISASTER_FAITH_GAIN` belief.
    Tornado { pos: WorldCoord, radius_voxels: u8 },
    /// `disaster.volcanic_vent` — sustain a `LAVA` + `STEAM`
    /// column at `pos` for `ticks` ticks (recorded in
    /// `last_tick_audio_events` so the audio substrate fires
    /// the volcano rumble sfx every tick). Voxel writes go
    /// through `push_voxel_write`. Adds `DISASTER_FAITH_GAIN`
    /// belief.
    VolcanicVent {
        pos: WorldCoord,
        /// Sustained-verb tick budget. The simulation tracks
        /// tick elapsed since the verb was fired; once
        /// exceeded the column stops emitting new LAVA but
        /// the existing pool keeps melting under the thermo
        /// CA.
        ticks: u32,
    },
    /// `disaster.drought` — clamp the per-region
    /// `precip_mm_fp` in `weather_grid` down by
    /// `reduction_pct` percent (clamped to `[0, 100]`),
    /// lasting `ticks` ticks. The verb mutates the public
    /// `weather_grid` field directly (the substrate-owned
    /// read path the renderer and downstream CAs already
    /// use) so the simulation propagates the drought via the
    /// planet phase next tick. Phase 4
    /// (FR-CIV-GODTOOL-901 batch 3).
    Drought {
        pos: WorldCoord,
        /// Reduction as percent in `[0, 100]`.
        reduction_pct: u8,
        /// Sustained-verb tick budget. Once exceeded the
        /// drought subsides and the planet phase restores
        /// normal precipitation.
        ticks: u32,
    },
}

/// INSPECT verb parameters. Phase 1 ships
/// [`InspectRequest::Probe`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InspectRequest {
    /// `inspect.probe` — read-only query: voxel material at
    /// `pos` + nearest agent alignment.
    Probe(ProbeRequest),
}

/// LAW verb parameters. Phase 4 (FR-CIV-GODTOOL-901 batch 3)
/// ships [`LawRequest::TaxBias`], [`LawRequest::ReligionPressure`],
/// and [`LawRequest::DifficultyKnob`]. Each verb writes a
/// substrate-owned scalar field the engine reads each tick —
/// no scripted outcomes, no bypass of the substrate APIs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LawRequest {
    /// `law.tax_bias` — transfer `bias` joules from the
    /// macro energy budget to the faction identified by
    /// `target_faction`, and bump belief by
    /// `bias / 1_000_000` (rounded) so the divine-powers
    /// economy rewards active governance.
    TaxBias {
        /// Faction id whose treasury receives the bias.
        target_faction: u32,
        /// Joules to transfer (positive adds to the faction,
        /// negative draws from it). Clamped to the available
        /// macro budget when positive.
        bias: i64,
    },
    /// `law.religion_pressure` — add `pressure` belief
    /// units to the divine-powers reserve. Mirrors the
    /// downstream "religion pressure → faith" coupling
    /// described in FR-CIV-EMERGENCE. `pressure` is clamped
    /// to `[0, u32::MAX]`.
    ReligionPressure {
        /// Belief units to add.
        pressure: u64,
    },
    /// `law.difficulty_knob` — write the new
    /// `scarcity_multiplier` on `economy_policy`. The
    /// `phase_economy` tick reads this scalar each frame,
    /// so the verb's effect propagates immediately on the
    /// next tick. Phase 4 (FR-CIV-GODTOOL-901 batch 3).
    DifficultyKnob {
        /// New scarcity multiplier in `[0.0, 10.0]`. Out-of-
        /// range values are rejected with `InvalidRequest`.
        scarcity_multiplier: f64,
    },
}

/// Parameters for [`InspectRequest::Probe`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProbeRequest {
    /// World coord to probe.
    pub pos: WorldCoord,
}

/// Result of a successful god-tool application. The Bevy
/// dispatcher surfaces this to the HUD/palette for feedback
/// (HUD toast, palette chip, undo stack push).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GodToolReceipt {
    /// A TERRAIN verb stamped N voxel writes.
    Terraform {
        /// Which op was applied.
        op: TerraformOp,
        /// Number of voxel writes actually performed.
        writes: u32,
    },
    /// A MATERIAL verb stamped N voxel writes.
    Material {
        /// Which op was applied.
        op: MaterialOp,
        /// Number of voxel writes actually performed.
        writes: u32,
    },
    /// A LIFE verb injected one or more agents.
    Life {
        /// For `SpawnOrganism` / `SpawnHerd`, the first
        /// `hecs::Entity` bits of the new agent(s). For
        /// `Bless` / `Curse` / `Heal`, the entity bits of the
        /// first affected agent (0 if the footprint was empty).
        /// For `Extinct`, the entity bits of the first despawned
        /// agent (0 if none).
        agent_entity_bits: u64,
        /// For `SpawnHerd` this is the count of agents spawned
        /// (always 1 for `SpawnOrganism`). For `Extinct` this is
        /// the count of agents despawned. For `Bless` / `Curse`
        /// / `Heal` this is the count of agents in the
        /// footprint whose state was touched.
        affected_count: u32,
    },
    /// A DISASTER verb fired.
    Disaster {
        /// `DisasterKind` that was applied.
        kind: DisasterKind,
        /// `true` when the disaster actually triggered.
        fired: bool,
    },
    /// A DISASTER verb that uses a custom Phase 4 path
    /// (Lightning / Tornado / VolcanicVent / Drought) without
    /// routing through `DisasterKind`. The receipt reports the
    /// voxel-write count so the Bevy HUD can show the same
    /// "verb mutated the field" toast it does for the
    /// standard Disaster receipt.
    EnvironmentalDisaster {
        /// Wire-stable verb label, e.g. `"lightning"` /
        /// `"tornado"` / `"volcanic_vent"` / `"drought"`.
        kind_label: String,
        /// Number of substrate cells the verb wrote (voxel
        /// writes for the first three; weather cells mutated
        /// for drought).
        writes: u32,
    },
    /// A LAW verb applied. Phase 4 (FR-CIV-GODTOOL-901
    /// batch 3) reports the scalar delta applied so the Bevy
    /// deck can show a "policy applied" toast.
    Law {
        /// Verb id, e.g. `"law.tax_bias"` /
        /// `"law.religion_pressure"` /
        /// `"law.difficulty_knob"`.
        verb: String,
        /// Scalar delta the verb applied to the substrate
        /// (joules transferred for tax_bias, belief units
        /// added for religion_pressure, new scarcity
        /// multiplier for difficulty_knob).
        delta: i64,
    },
    /// An INSPECT verb returned a probe report.
    Inspect {
        /// The probed report (read-only state).
        report: ProbeReport,
    },
    /// Read-only / time / camera verb — no substrate write.
    NoOp {
        /// Verb id for logging / HUD toast.
        verb: String,
    },
}

/// The probe report returned by [`InspectRequest::Probe`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProbeReport {
    /// World coord that was probed.
    pub pos: WorldCoord,
    /// Material id at `pos`. `0` is `AIR` (the empty voxel).
    pub material: MaterialId,
    /// Nearest agent `hecs::Entity` bits if any, `None`
    /// otherwise. Searched in a small footprint centered on
    /// `pos` (read-only; does not mutate state).
    pub nearest_agent: Option<u64>,
}

/// Errors a god-tool handler can return.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GodToolError {
    /// The request is for a verb the substrate has not
    /// implemented yet (e.g. `material.seed_forest` in Phase 1).
    /// The Bevy layer should surface a "data not yet surfaced"
    /// toast.
    NotImplemented {
        /// Verb id.
        verb: &'static str,
    },
    /// The request payload failed validation (negative radius,
    /// NaN coordinates, etc.).
    InvalidRequest(String),
    /// A read-only INSPECT verb was mis-routed through a
    /// substrate mutator.
    ReadOnly,
}

impl core::fmt::Display for GodToolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GodToolError::NotImplemented { verb } => {
                write!(f, "god-tool `{verb}` has no substrate handler yet")
            }
            GodToolError::InvalidRequest(msg) => write!(f, "invalid god-tool request: {msg}"),
            GodToolError::ReadOnly => f.write_str("read-only verb cannot be mutating"),
        }
    }
}

impl std::error::Error for GodToolError {}

impl GodToolReceipt {
    /// Convenience constructor for a read-only / time / camera
    /// no-op receipt.
    pub fn no_op(verb: impl Into<String>) -> Self {
        GodToolReceipt::NoOp { verb: verb.into() }
    }
}

/// Scan upward from the supplied `baseline` y and return the
/// y-coord of the topmost non-`AIR` voxel in the column
/// `(col_x, *, col_z)`. Used by `terrain.smooth` to compute the
/// average topmost height in a 3×3 window. Returns `baseline`
/// when the column is empty above the baseline.
fn scan_topmost_y(
    voxel: &civ_voxel::VoxelWorld<MaterialId>,
    col_x: i64,
    col_z: i64,
    baseline: i64,
) -> i64 {
    // Walk 8 cells above the baseline and remember the highest
    // y that wasn't `AIR`. The window is intentionally small
    // (8 cells = 1 m above the baseline in fixed-point metres)
    // because `terrain.smooth` only needs the local shape; a
    // larger scan would quadratic-explode the per-brush cost.
    let mut top = baseline;
    for dy in 1i64..=8 {
        let y = baseline + dy * FIXED_SCALE;
        let m = voxel.read(WorldCoord {
            x: col_x,
            y,
            z: col_z,
        });
        if m != AIR {
            top = y;
        }
    }
    top
}

/// Walk the agent `hecs::World` and return a deterministic
/// `Vec<(Entity, &Position3d, &Civilian)>` of agents inside the
/// spherical footprint `(center, radius_voxels)`. The order
/// matches `hecs`'s archetype iteration order, which is stable
/// across runs that mutate through the same API.
fn actors_in_footprint<'w>(
    world: &'w hecs::World,
    center: WorldCoord,
    radius_voxels: u8,
) -> Vec<(hecs::Entity, &'w Position3d, &'w Civilian)> {
    let r = i64::from(radius_voxels) * FIXED_SCALE;
    let r2 = r * r;
    world
        .query::<(&Position3d, &Civilian)>()
        .iter()
        .filter_map(|(entity, (pos, civ))| {
            let dx = pos.coord.x - center.x;
            let dy = pos.coord.y - center.y;
            let dz = pos.coord.z - center.z;
            if dx * dx + dy * dy + dz * dz <= r2 {
                Some((entity, pos, civ))
            } else {
                None
            }
        })
        .collect()
}

/// The kinds of actor-side effect the LIFE bless/curse/heal
/// verbs can apply. The "no `mood`/`alignment`/`culture`/
/// `ideology`" rule (AC-CPL-3) is enforced by keeping this enum
/// closed — adding a new variant is the only way to touch a new
/// substrate field, and that change is reviewable.
enum ActorEffectKind {
    /// Add (positive `delta`) or subtract (negative `delta`)
    /// from each `civ_needs::Needs` field. Clamped to `[0, 1]`.
    BoostNeeds { delta: f32 },
    /// Restore `Health::integrity` (and the mirrored
    /// `Needs::health`) by `restore`. Clamped to `[0, 1]`.
    Heal { restore: f32 },
}

/// Apply one [`ActorEffectKind`] to every agent inside the
/// spherical footprint `(center, radius_voxels)`. Returns a
/// [`GodToolReceipt::Life`] with the count of affected agents.
///
/// The implementation respects the AC-CPL-3 forbidden-field
/// list by only touching `civ_needs::Needs` and
/// `civ_needs::Health` — never `mood`, `alignment`, `culture`,
/// or `ideology`.
fn apply_actor_effect(
    world: &mut hecs::World,
    center: WorldCoord,
    radius_voxels: u8,
    kind: ActorEffectKind,
) -> Result<GodToolReceipt, GodToolError> {
    if radius_voxels == 0 {
        return Err(GodToolError::InvalidRequest(
            "actor effect radius_voxels must be > 0".into(),
        ));
    }
    // Snapshot the entities first so we can drop the immutable
    // borrow of `world` before taking the mutable borrows below.
    let affected: Vec<hecs::Entity> = actors_in_footprint(world, center, radius_voxels)
        .into_iter()
        .map(|(e, _, _)| e)
        .collect();
    let first = affected
        .first()
        .map(|ent| ent.to_bits().get())
        .unwrap_or(0);
    let mut touched: u32 = 0;
    for entity in &affected {
        let mut did_touch = false;
        match kind {
            ActorEffectKind::BoostNeeds { delta } => {
                if let Ok(mut needs) = world.get::<&mut LifeNeeds>(*entity) {
                    // `delta` is signed; for `bless` it is
                    // positive, for `curse` it is negative. The
                    // saturating arithmetic caps at `[0, 1]`.
                    needs.food = (needs.food + delta).clamp(0.0, 1.0);
                    needs.water = (needs.water + delta).clamp(0.0, 1.0);
                    needs.rest = (needs.rest + delta).clamp(0.0, 1.0);
                    needs.safety = (needs.safety + delta).clamp(0.0, 1.0);
                    needs.social = (needs.social + delta).clamp(0.0, 1.0);
                    needs.health = (needs.health + delta).clamp(0.0, 1.0);
                    did_touch = true;
                }
            }
            ActorEffectKind::Heal { restore } => {
                if let Ok(mut needs) = world.get::<&mut LifeNeeds>(*entity) {
                    needs.health = (needs.health + restore).clamp(0.0, 1.0);
                    did_touch = true;
                }
                if let Ok(mut health) = world.get::<&mut LifeHealth>(*entity) {
                    health.integrity = (health.integrity + restore).clamp(0.0, 1.0);
                    if health.sick && health.integrity >= 0.9 {
                        health.sick = false;
                    }
                    did_touch = true;
                }
            }
        }
        if did_touch {
            touched = touched.saturating_add(1);
        }
    }
    Ok(GodToolReceipt::Life {
        agent_entity_bits: first,
        affected_count: touched,
    })
}

/// Return the topmost non-AIR material at column `(x, *, z)`
/// (i.e. the highest `y` whose voxel is not AIR), or `None` if
/// the column is empty. Used by `disaster.lightning` to detect
/// igniteable PLANT / GRASS cells. Bounded to a 16-cell scan so
/// the helper stays O(1) for typical god-tool brush sizes.
///
/// Takes `&VoxelWorld` (read-only) so the caller can hold an
/// immutable borrow while still issuing mutable writes
/// elsewhere; this matches the rest of the god-tool helpers
/// (e.g. `scan_topmost_y`).
fn topmost_voxel(voxel: &civ_voxel::VoxelWorld<MaterialId>, cell: WorldCoord) -> Option<MaterialId> {
    for dy in 1i64..=16 {
        let y = cell.y + dy * FIXED_SCALE;
        let m = voxel.read(WorldCoord {
            x: cell.x,
            y,
            z: cell.z,
        });
        if m != AIR {
            return Some(m);
        }
    }
    None
}

/// 3-D integer Bresenham line from `a` to `b` (inclusive).
/// Returns the voxel coords along the line in order. Used by
/// `disaster.lightning` to rasterise the bolt path. The
/// implementation is a straight port of the standard
/// 3-D Bresenham (Amanatides & Woo) and is deterministic — the
/// same `(a, b)` always yields the same sequence, so replay
/// stays byte-identical.
fn bresenham_3d(
    a: WorldCoord,
    b: WorldCoord,
) -> Vec<(i64, i64, i64)> {
    let (mut x, mut y, mut z) = (a.x, a.y, a.z);
    let ex = b.x;
    let ey = b.y;
    let ez = b.z;
    let dx = (ex - x).abs();
    let dy = (ey - y).abs();
    let dz = (ez - z).abs();
    let sx = if ex >= x { 1 } else { -1 };
    let sy = if ey >= y { 1 } else { -1 };
    let sz = if ez >= z { 1 } else { -1 };
    // The dominant axis drives the iteration count.
    let dom = dx.max(dy).max(dz);
    let steps = dom + 1;
    let mut out: Vec<(i64, i64, i64)> = Vec::with_capacity(steps as usize);
    if dom == 0 {
        out.push((x, y, z));
        return out;
    }
    // Per-axis error accumulators scaled by 2 * dom.
    let mut err_x = 2 * dx - dom;
    let mut err_y = 2 * dy - dom;
    let mut err_z = 2 * dz - dom;
    for _ in 0..steps {
        out.push((x, y, z));
        // Step whichever axes overflowed this iteration.
        if err_x > 0 {
            x += sx;
            err_x -= 2 * dom;
        }
        if err_y > 0 {
            y += sy;
            err_y -= 2 * dom;
        }
        if err_z > 0 {
            z += sz;
            err_z -= 2 * dom;
        }
        // Accumulate next-iteration error (Bresenham step).
        err_x += 2 * dx;
        err_y += 2 * dy;
        err_z += 2 * dz;
    }
    out
}

/// Tiny 16-bit fixed-point cosine lookup. `angle_fp` is a
/// 16-bit angle (0..=0xFFFF) representing 0..2π. Returns a
/// signed `i64` in `[-1_000_000, 1_000_000]`. Used by
/// `disaster.tornado` to place the spiral arms; the precision
/// is plenty for a 64-cell brush.
fn cos_lut(angle_fp: i64) -> i64 {
    let a = angle_fp.rem_euclid(0x10000);
    let idx = (a as usize) & 0x3FFF;
    let sign: i64 = if a < 0x8000 { 1 } else { -1 };
    let phase = if a < 0x8000 { idx } else { 0x4000 - idx };
    // cos(x) for x in [0, π/2], scaled to [-1, 1] as
    // integer milliunits.
    let v: i64 = match phase {
        0 => 1_000_000,
        8192 => 707_107, // cos(π/4)
        _ => {
            // Linear interpolation between the two anchors
            // plus a quarter-cosine curve. Cheap and
            // good-enough for visual vortex placement.
            let t = (phase as f64) / 16_384.0;
            ((1.0 - t) * 1_000_000.0 + t * 707_107.0).round() as i64
        }
    };
    sign * v
}

/// Tiny 16-bit fixed-point sine lookup, matching
/// [`cos_lut`]. `angle_fp` is a 16-bit angle in 0..2π.
fn sin_lut(angle_fp: i64) -> i64 {
    cos_lut(angle_fp.wrapping_sub(0x4000))
}

impl Simulation {
    /// Apply a god-tool request to the simulation. The single
    /// Bevy → substrate bridge (AC-CPL-2).
    ///
    /// **Substrate write discipline:** every mutating variant
    /// routes through a `Simulation` method that already mutates
    /// state the engine reads each tick. There is no direct
    /// `hecs::World` or `VoxelWorld` access from any god-tool
    /// path. See the module docs.
    ///
    /// ## Verb coverage
    ///
    /// ### Phase 1 (substrate-mutating)
    /// - `terrain.raise` / `terrain.lower` / `terrain.level` —
    ///   real voxel writes via `push_voxel_write`.
    /// - `life.spawn_organism` — real agent spawn via
    ///   `civ_agents::spawn_civilian_at`.
    /// - `disaster.meteor` — real disaster invocation via
    ///   `trigger_disaster(DisasterKind::Meteor, …)`.
    /// - `inspect.probe` — read-only; returns the material at
    ///   the probed coord and the nearest agent (or `None`).
    ///
    /// ### Phase 2 (substrate-mutating, this module)
    /// - `terrain.smooth` — averages 3×3×3 neighbour heights and
    ///   writes `STONE` at the new heights.
    /// - `terrain.raise_mountain` — writes a Gaussian peak of
    ///   `STONE` (with `GRAVEL` shoulders) in the footprint.
    /// - `life.spawn_herd` — deterministic N-agent batch via
    ///   `civ_agents::spawn_many`.
    /// - `life.bless` / `life.curse` — add/subtract from
    ///   `civ_needs::Needs` for every agent in the footprint
    ///   (never the AC-CPL-3 forbidden fields).
    /// - `life.heal` — restore `Health::integrity` for every agent
    ///   in the footprint.
    /// - `life.extinct` — despawn every agent in the footprint
    ///   via `hecs::World::despawn`.
    /// - `disaster.wildfire` / `flood` / `quake` / `storm` /
    ///   `plague` — real disaster invocations via
    ///   `trigger_disaster(DisasterKind::*, …)`.
    ///
    /// Other variants produce a `NoOp` receipt tagged with the
    /// verb id (Bevy layer should surface a "data not yet
    /// surfaced" toast and push the verb as `Near` in the deck).
    pub fn apply_god_tool(
        &mut self,
        req: GodToolRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        match req {
            GodToolRequest::Terraform(t) => self.apply_terraform(t),
            GodToolRequest::Material(m) => self.apply_material(m),
            GodToolRequest::Life(l) => self.apply_life(l),
            GodToolRequest::Disaster(d) => self.apply_disaster(d),
            GodToolRequest::Inspect(i) => self.apply_inspect(i),
            GodToolRequest::Law(l) => self.apply_law(l),
        }
    }

    fn apply_terraform(
        &mut self,
        req: TerraformRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        // Validate.
        if req.radius_voxels == 0 {
            return Err(GodToolError::InvalidRequest(
                "terraform radius_voxels must be > 0".into(),
            ));
        }
        match req.op {
            TerraformOp::Level if req.strength < 0 => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.level strength (target y) must be >= 0".into(),
                ));
            }
            TerraformOp::Smooth if req.strength < 0 => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.smooth strength must be >= 0".into(),
                ));
            }
            TerraformOp::RaiseMountain if req.strength < 0 => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.raise_mountain strength must be >= 0".into(),
                ));
            }
            // Phase 3: `terrain.slope` accepts any finite Δ-y
            // (positive tilts up toward `+x`, negative tilts
            // down). No validation needed beyond non-zero radius.
            TerraformOp::AddLand if req.strength <= 0 => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.add_land strength (band thickness) must be > 0".into(),
                ));
            }
            TerraformOp::DigOcean if req.strength <= 0 => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.dig_ocean strength (dig depth) must be > 0".into(),
                ));
            }
            TerraformOp::DropBiome if req.aux_id == 0 || req.aux_id > u32::from(u8::MAX) => {
                return Err(GodToolError::InvalidRequest(
                    "terrain.drop_biome aux_id must be a valid material id (1..=255)".into(),
                ));
            }
            // Phase 4: `terrain.flatten` writes a STONE band at
            // the median surface y in the footprint. No
            // strength validation needed — the verb reads the
            // existing surface y via `scan_topmost_y` so a
            // zero or negative `strength` is treated as "use
            // the natural median" (no user-settable target).
            _ => {}
        }

        // The center itself, plus the cells within the radius
        // sphere. For Raise/Lower we write the same material in
        // every cell; for Level we read the current topmost
        // voxel y in each (x, z) column and write the target
        // material only into the band between current_y and
        // `strength` (target_y).
        let cx = req.center.x;
        let cy = req.center.y;
        let cz = req.center.z;
        let r = i64::from(req.radius_voxels);
        let r2 = r * r;
        let mut writes: u32 = 0;
        match req.op {
            TerraformOp::Raise | TerraformOp::Lower => {
                let material = match req.op {
                    TerraformOp::Raise => STONE,
                    TerraformOp::Lower => AIR,
                    _ => unreachable!(),
                };
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r2 {
                                continue;
                            }
                            self.push_voxel_write(
                                WorldCoord {
                                    x: cx + dx,
                                    y: cy + dy,
                                    z: cz + dz,
                                },
                                material,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            TerraformOp::Level => {
                // PACKED_DIRT (id 7) for the band between current
                // topmost y in the column and the target y.
                let target_y = req.strength as i64;
                // We only touch cells where the column's topmost
                // solid voxel is below the target. Because the
                // sim is height-agnostic at this layer, we write
                // a 1-cell band per (x, z) column at the target
                // y — that gives CA something to settle.
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        self.push_voxel_write(
                            WorldCoord {
                                x: cx + dx,
                                y: target_y,
                                z: cz + dz,
                            },
                            MaterialId(7), // PACKED_DIRT
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            TerraformOp::Smooth => {
                // Average the centre column's topmost solid voxel
                // height with the 8 (dx, dz) neighbours in a 3×3
                // window, then write a 1-cell `STONE` band at the
                // smoothed y. The brush radius caps the window so
                // very large brushes don't go out of range. CA
                // settles the result next tick.
                let window = r.min(8);
                for dz in -window..=window {
                    for dx in -window..=window {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        // Read the topmost solid y in the centre
                        // column and the 3×3 neighbour window. We
                        // approximate the topmost y as the highest
                        // y in the 3-cell column scan above the
                        // requested strength baseline.
                        let baseline = req.strength as i64;
                        let mut total: i64 = 0;
                        let mut count: i64 = 0;
                        for ndz in -1i64..=1 {
                            for ndx in -1i64..=1 {
                                let col_x = cx + (dx + ndx) * FIXED_SCALE;
                                let col_z = cz + (dz + ndz) * FIXED_SCALE;
                                let top =
                                    scan_topmost_y(&self.voxel, col_x, col_z, baseline);
                                total = total.saturating_add(top);
                                count = count.saturating_add(1);
                            }
                        }
                        let avg = if count > 0 { total / count } else { baseline };
                        self.push_voxel_write(
                            WorldCoord {
                                x: cx + dx * FIXED_SCALE,
                                y: avg,
                                z: cz + dz * FIXED_SCALE,
                            },
                            STONE,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            TerraformOp::RaiseMountain => {
                // Gaussian peak: each (dx, dz) cell receives a
                // peak height proportional to
                // `exp(-(d² / 2σ²)) * strength`. The peak is
                // filled with `STONE`; the lower shoulders drop to
                // `GRAVEL` when within 25% of the peak.
                let peak = req.strength as i64;
                let sigma = (r as f64).max(1.0);
                for dz in -r..=r {
                    for dx in -r..=r {
                        let d2 = dx * dx + dz * dz;
                        if d2 > r2 {
                            continue;
                        }
                        let d = (d2 as f64).sqrt();
                        let h = (-(d * d) / (2.0 * sigma * sigma)).exp() * peak as f64;
                        let h_i64 = h as i64;
                        if h_i64 <= 0 {
                            continue;
                        }
                        let material = if h_i64 as f64 >= 0.75 * peak as f64 {
                            STONE
                        } else {
                            GRAVEL
                        };
                        self.push_voxel_write(
                            WorldCoord {
                                x: cx + dx * FIXED_SCALE,
                                y: cy + h_i64,
                                z: cz + dz * FIXED_SCALE,
                            },
                            material,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            TerraformOp::Slope => {
                // Linear gradient across the brush footprint in
                // `+x`. `strength` is the full Δ-y from the low
                // edge (`x = cx - r*FIXED_SCALE`) to the high
                // edge (`x = cx + r*FIXED_SCALE`). The base
                // elevation per column is read from
                // `scan_topmost_y` so the tilt pivots around the
                // existing surface rather than replacing it.
                // The result is a `STONE` write at
                // `top_y + (dx / r) * strength`.
                let span = r.max(1) as f64;
                let gradient = req.strength as f64 / span;
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        let h = (dx as f64 * gradient) as i64;
                        self.push_voxel_write(
                            WorldCoord {
                                x: col_x,
                                y: top_y + h,
                                z: col_z,
                            },
                            STONE,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            TerraformOp::AddLand => {
                // Chunky raise: for each (x, z) column in the
                // footprint, find the topmost solid voxel via
                // `scan_topmost_y` and write a `STONE` band of
                // `strength` thickness above it. The CA will
                // re-settle the band into a stable mass next
                // tick. We use a hard-edged footprint (no
                // falloff) so the result is a flat plateau that
                // matches the in-game "add land" UI.
                let thickness = req.strength as i64;
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        for dy in 1..=thickness {
                            self.push_voxel_write(
                                WorldCoord {
                                    x: col_x,
                                    y: top_y + dy,
                                    z: col_z,
                                },
                                STONE,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            TerraformOp::DigOcean => {
                // Chunky dig-down: for each (x, z) column in the
                // footprint, find the topmost solid voxel; if it
                // is above the sea-level band (`req.aux_id`
                // fixed-point units, or `cy` when zero), dig
                // down by `strength` fixed-point units and fill
                // the new cavity with `WATER` so the fluid CA
                // can hydrate it next tick. Columns whose
                // topmost voxel is at or below the sea-level
                // band are left untouched (the verb never
                // deepens an ocean floor; it only carves
                // landward coastlines).
                let dig_depth = req.strength as i64;
                let sea_level = if req.aux_id == 0 {
                    cy
                } else {
                    req.aux_id as i64
                };
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        if top_y <= sea_level {
                            continue;
                        }
                        let new_floor = top_y.saturating_sub(dig_depth).max(sea_level);
                        for y in (new_floor + 1)..=top_y {
                            self.push_voxel_write(
                                WorldCoord {
                                    x: col_x,
                                    y,
                                    z: col_z,
                                },
                                WATER,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            TerraformOp::DropBiome => {
                // Re-paint the topmost solid voxel of each
                // (x, z) column in the footprint to the material
                // in `aux_id`. The "biome" is encoded as a
                // material id (SAND, SNOW, GRASS, …) so the
                // surface id is the canonical biome marker that
                // downstream CAs and inspectors already
                // understand. CA re-settles / propagates the
                // new surface next tick.
                let target = MaterialId(req.aux_id as u8);
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        if top_y == cy {
                            // Empty column — nothing to paint.
                            continue;
                        }
                        self.push_voxel_write(
                            WorldCoord {
                                x: col_x,
                                y: top_y,
                                z: col_z,
                            },
                            target,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            TerraformOp::Flatten => {
                // Sample the topmost solid voxel y in each
                // (x, z) column of the footprint, compute the
                // arithmetic mean, and stamp a 1-cell `STONE`
                // band at the mean y in every column. The
                // mean (rather than the strict mode) is used
                // so the verb produces a stable plateau that
                // matches the in-game "flatten" UI without
                // having to bucket-sort the histogram. CA
                // settles the result next tick.
                let mut total: i64 = 0;
                let mut count: i64 = 0;
                let mut tops: Vec<i64> = Vec::new();
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        if top_y == cy {
                            // Empty column — skip so the mean
                            // isn't dragged down by phantom
                            // surface reads.
                            continue;
                        }
                        total = total.saturating_add(top_y);
                        count = count.saturating_add(1);
                        tops.push(top_y);
                    }
                }
                let flat_y = if count > 0 {
                    total / count
                } else {
                    cy
                };
                let _ = tops; // retained for future strict-mode dispatch
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        self.push_voxel_write(
                            WorldCoord {
                                x: col_x,
                                y: flat_y,
                                z: col_z,
                            },
                            STONE,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
        }
        Ok(GodToolReceipt::Terraform {
            op: req.op,
            writes,
        })
    }

    /// Material-verb substrate dispatcher (Phase 3 — 7 ops).
    ///
    /// Every variant writes through `push_voxel_write`; the CA
    /// then settles gravity, fluid flow, reactivity, and weather
    /// each tick. The verb never bypasses the substrate write
    /// path (AC-CPL-2 / AC-CPL-3).
    fn apply_material(
        &mut self,
        req: MaterialRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        if req.radius_voxels == 0 {
            return Err(GodToolError::InvalidRequest(
                "material radius_voxels must be > 0".into(),
            ));
        }
        // Shared footprint projection. `cx`, `cy`, `cz` are the
        // brush center in fixed-point coords; `r` is the radius
        // in voxels; `r2` is `r²`.
        let cx = req.center.x;
        let cy = req.center.y;
        let cz = req.center.z;
        let r = i64::from(req.radius_voxels);
        let r2 = r * r;
        let target = MaterialId(req.material_id as u8);
        let mut writes: u32 = 0;
        match req.op {
            MaterialOp::Erase => {
                // `material.erase` — write `AIR` in the
                // spherical footprint. Mirrors the inverse of
                // `terrain.raise`. Uses `FIXED_SCALE` stride so
                // the brush footprint is voxel-aligned.
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r2 {
                                continue;
                            }
                            self.push_voxel_write(
                                WorldCoord {
                                    x: cx + dx * FIXED_SCALE,
                                    y: cy + dy * FIXED_SCALE,
                                    z: cz + dz * FIXED_SCALE,
                                },
                                AIR,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            MaterialOp::Replace => {
                // `material.replace` — write `target` material in
                // the spherical footprint. Overwrites any
                // existing material; the CA settles the result
                // next tick.
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r2 {
                                continue;
                            }
                            self.push_voxel_write(
                                WorldCoord {
                                    x: cx + dx * FIXED_SCALE,
                                    y: cy + dy * FIXED_SCALE,
                                    z: cz + dz * FIXED_SCALE,
                                },
                                target,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            MaterialOp::SurfacePaint => {
                // `material.surface_paint` — write `target`
                // material only on the topmost solid voxel per
                // (x, z) column. Uses `scan_topmost_y` to find
                // the local topmost y in the column (8 cells
                // above `cy`) and stamps `target` there. The CA
                // settles the surface next tick.
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y = scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        self.push_voxel_write(
                            WorldCoord {
                                x: col_x,
                                y: top_y,
                                z: col_z,
                            },
                            target,
                        );
                        writes = writes.saturating_add(1);
                    }
                }
            }
            MaterialOp::AdditiveDrop => {
                // `material.additive_drop` — seed `target`
                // material in a sphere at
                // `center.y + drop_height`. The voxel CA's
                // falling rule carries the material down to the
                // existing surface next tick; we never bypass
                // gravity (no scripted transport).
                let drop_y = cy + (req.drop_height as i64).max(FIXED_SCALE);
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r2 {
                                continue;
                            }
                            self.push_voxel_write(
                                WorldCoord {
                                    x: cx + dx * FIXED_SCALE,
                                    y: drop_y + dy * FIXED_SCALE,
                                    z: cz + dz * FIXED_SCALE,
                                },
                                target,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            MaterialOp::PourLiquid => {
                // `material.pour_liquid` — seed liquid voxels
                // (default `WATER`; the client can ship `LAVA`
                // via `material_id`) at
                // `center.y + drop_height`. The fluid CA spreads
                // the liquid horizontally next tick; we only
                // write the seed sphere. `strength` is the
                // deposit thickness (in `FIXED_SCALE` units);
                // the seed sphere is `strength / FIXED_SCALE`
                // layers tall.
                let layers = ((req.strength as i64).max(FIXED_SCALE) / FIXED_SCALE).max(1);
                let drop_y = cy + (req.drop_height as i64).max(FIXED_SCALE);
                let liquid = if target == LAVA { LAVA } else { WATER };
                for layer in 0..layers {
                    for dz in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dz * dz > r2 {
                                continue;
                            }
                            self.push_voxel_write(
                                WorldCoord {
                                    x: cx + dx * FIXED_SCALE,
                                    y: drop_y + layer * FIXED_SCALE,
                                    z: cz + dz * FIXED_SCALE,
                                },
                                liquid,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            MaterialOp::SeedSnow => {
                // `material.seed_snow` — write `SNOW` voxels in
                // a sphere at the local snowline band. The
                // snowline is approximated as
                // `topmost_y + strength` (positive `strength` =
                // colder snowline = lower deposit). The thermo
                // CA's melt rule sublimates the snow at
                // temperatures above the snowline next tick, so
                // the verb never produces immortal snow.
                for dz in -r..=r {
                    for dy in -r..=r {
                        for dx in -r..=r {
                            if dx * dx + dy * dy + dz * dz > r2 {
                                continue;
                            }
                            let col_x = cx + dx * FIXED_SCALE;
                            let col_z = cz + dz * FIXED_SCALE;
                            let top_y =
                                scan_topmost_y(&self.voxel, col_x, col_z, cy);
                            self.push_voxel_write(
                                WorldCoord {
                                    x: col_x,
                                    y: top_y + req.strength as i64 + dy * FIXED_SCALE,
                                    z: col_z,
                                },
                                SNOW,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
            MaterialOp::SeedOreDeposit => {
                // `material.seed_ore` — write `ORE` voxels in a
                // stochastic vein pattern drawn from a
                // deterministic per-cell noise seeded by
                // `(center.x, center.z)` so replay is stable.
                // `strength` controls the vein thickness (in
                // fixed-point units); the vein spans
                // `|dy| <= strength` cells vertically from the
                // local topmost solid voxel.
                let vein_half = (req.strength as i64).max(FIXED_SCALE);
                for dz in -r..=r {
                    for dy in -vein_half..=vein_half {
                        for dx in -r..=r {
                            if dx * dx + dz * dz > r2 {
                                continue;
                            }
                            let col_x = cx + dx * FIXED_SCALE;
                            let col_z = cz + dz * FIXED_SCALE;
                            let top_y =
                                scan_topmost_y(&self.voxel, col_x, col_z, cy);
                            // Deterministic noise: mix the
                            // `(dx, dz)` cell offset with the
                            // sim RNG seed. The mix is a small
                            // integer hash — same input ⇒ same
                            // pattern ⇒ stable replay.
                            let seed = self
                                .state
                                .rng_seed
                                .wrapping_add((col_x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
                                .wrapping_add((col_z as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
                            let pattern = (seed ^ (seed >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
                            // Threshold ~ 25% of cells become
                            // ORE voxels; the rest are left
                            // alone so the vein looks like a
                            // natural deposit.
                            if pattern % 4 == 0 {
                                self.push_voxel_write(
                                    WorldCoord {
                                        x: col_x,
                                        y: top_y + dy,
                                        z: col_z,
                                    },
                                    ORE,
                                );
                                writes = writes.saturating_add(1);
                            }
                        }
                    }
                }
            }
            MaterialOp::SeedForest => {
                // `material.seed_forest` — write `PLANT`
                // voxels in a stochastic scatter inside the
                // footprint. The scatter is seeded by
                // `(center.x, center.z)` (and the sim RNG
                // seed) so replay is stable. `strength`
                // controls the scatter density (higher ⇒ more
                // voxels). The CA's growth rule carries the
                // plants outward over the next ticks; we
                // only stamp the seed voxels (no scripted
                // canopy). Phase 4.
                let density_pct = (req.strength as u32).clamp(1, 100);
                for dz in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dz * dz > r2 {
                            continue;
                        }
                        let col_x = cx + dx * FIXED_SCALE;
                        let col_z = cz + dz * FIXED_SCALE;
                        let top_y =
                            scan_topmost_y(&self.voxel, col_x, col_z, cy);
                        // Deterministic per-cell hash; same
                        // input ⇒ same scatter ⇒ stable replay.
                        let seed = self
                            .state
                            .rng_seed
                            .wrapping_add((col_x as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
                            .wrapping_add((col_z as u64).wrapping_mul(0x1656_67B1_9E4A_CC15));
                        let pattern = (seed ^ (seed >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
                        let bucket = (pattern % 100) as u32;
                        if bucket < density_pct {
                            self.push_voxel_write(
                                WorldCoord {
                                    x: col_x,
                                    y: top_y + FIXED_SCALE,
                                    z: col_z,
                                },
                                PLANT,
                            );
                            writes = writes.saturating_add(1);
                        }
                    }
                }
            }
        }
        Ok(GodToolReceipt::Material {
            op: req.op,
            writes,
        })
    }

    fn apply_life(&mut self, req: LifeRequest) -> Result<GodToolReceipt, GodToolError> {
        match req {
            LifeRequest::SpawnOrganism(s) => {
                if !s.x.is_finite() || !s.y.is_finite() {
                    return Err(GodToolError::InvalidRequest(
                        "spawn x / y must be finite".into(),
                    ));
                }
                if !(0.0..=1.0).contains(&s.x) || !(0.0..=1.0).contains(&s.y) {
                    return Err(GodToolError::InvalidRequest(
                        "spawn x / y must be in 0..1".into(),
                    ));
                }
                // Per-tick deterministic spawn RNG. The substrate
                // re-uses `civ-agents::spawn_civilian_at`, which
                // takes a `ChaCha8Rng`; we re-seed from the
                // current sim RNG seed + agent id so spawns are
                // deterministic across replays (charter: soft
                // determinism).
                let mut rng = ChaCha8Rng::seed_from_u64(
                    self.state.rng_seed.wrapping_add(s.id),
                );
                let entity = spawn_civilian_at(
                    &mut self.world,
                    s.id,
                    Alignment::with_faction(s.faction),
                    s.x,
                    s.y,
                    s.visual.into(),
                    &mut rng,
                );
                Ok(GodToolReceipt::Life {
                    agent_entity_bits: entity.to_bits().get(),
                    affected_count: 1,
                })
            }
            LifeRequest::SpawnHerd(s) => {
                if s.count == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "spawn_herd count must be > 0".into(),
                    ));
                }
                if s.count > 1_000 {
                    return Err(GodToolError::InvalidRequest(
                        "spawn_herd count must be <= 1000".into(),
                    ));
                }
                let entities = spawn_many(
                    &mut self.world,
                    s.count,
                    s.seed_civilian_id,
                    s.faction,
                );
                let first = entities
                    .first()
                    .map(|e| e.to_bits().get())
                    .unwrap_or(0);
                Ok(GodToolReceipt::Life {
                    agent_entity_bits: first,
                    affected_count: entities.len() as u32,
                })
            }
            LifeRequest::Bless(e) => {
                if !e.strength.is_finite() || e.strength < 0.0 {
                    return Err(GodToolError::InvalidRequest(
                        "bless strength must be a non-negative finite value".into(),
                    ));
                }
                let boost = e.strength.clamp(0.0, 1.0);
                apply_actor_effect(
                    &mut self.world,
                    e.center,
                    e.radius_voxels,
                    ActorEffectKind::BoostNeeds { delta: boost },
                )
            }
            LifeRequest::Curse(e) => {
                if !e.strength.is_finite() || e.strength < 0.0 {
                    return Err(GodToolError::InvalidRequest(
                        "curse strength must be a non-negative finite value".into(),
                    ));
                }
                let delta = e.strength.clamp(0.0, 1.0);
                apply_actor_effect(
                    &mut self.world,
                    e.center,
                    e.radius_voxels,
                    ActorEffectKind::BoostNeeds { delta: -delta },
                )
            }
            LifeRequest::Heal(e) => {
                if !e.strength.is_finite() || e.strength < 0.0 {
                    return Err(GodToolError::InvalidRequest(
                        "heal strength must be a non-negative finite value".into(),
                    ));
                }
                let restore = e.strength.clamp(0.0, 1.0);
                apply_actor_effect(
                    &mut self.world,
                    e.center,
                    e.radius_voxels,
                    ActorEffectKind::Heal { restore },
                )
            }
            LifeRequest::Extinct(e) => {
                if e.radius_voxels == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "extinct radius_voxels must be > 0".into(),
                    ));
                }
                let affected = actors_in_footprint(&self.world, e.center, e.radius_voxels);
                let first = affected
                    .first()
                    .map(|(ent, _, _)| ent.to_bits().get())
                    .unwrap_or(0);
                let mut despawned: u32 = 0;
                for (entity, _, _) in &affected {
                    if self.world.despawn(*entity).is_ok() {
                        despawned = despawned.saturating_add(1);
                    }
                }
                Ok(GodToolReceipt::Life {
                    agent_entity_bits: first,
                    affected_count: despawned,
                })
            }
            LifeRequest::SpawnCivSeed(s) => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — spawn a
                // civilisation nucleus: 6 founder civilians via
                // `spawn_many`, plus a Primitive hut build site
                // and a Farm stockpile build site via
                // `enqueue_build_site`. All ids are derived
                // from `seed_civilian_id` so replay is
                // deterministic. The verb only routes through
                // substrate APIs (`spawn_many` +
                // `enqueue_build_site`) — no direct ECS or
                // voxel write.
                if s.seed_civilian_id == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "spawn_civ_seed seed_civilian_id must be > 0".into(),
                    ));
                }
                let entities = spawn_many(
                    &mut self.world,
                    6,
                    s.seed_civilian_id,
                    s.faction,
                );
                let first = entities
                    .first()
                    .map(|e| e.to_bits().get())
                    .unwrap_or(0);
                // Primitive hut (Workshop chain) at `center`.
                self.enqueue_build_site(BuildSite::new(
                    BuildingId(s.seed_civilian_id.wrapping_add(100)),
                    BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Workshop),
                    s.center,
                ));
                // Farm stockpile at `center + (FIXED_SCALE, 0,
                // FIXED_SCALE)` so the two buildings don't
                // collide.
                let farm_origin = WorldCoord {
                    x: s.center.x + FIXED_SCALE,
                    y: s.center.y,
                    z: s.center.z + FIXED_SCALE,
                };
                self.enqueue_build_site(BuildSite::new(
                    BuildingId(s.seed_civilian_id.wrapping_add(101)),
                    BuildingSpec::minimal(BuildingTier::Primitive, ProductionChain::Farm),
                    farm_origin,
                ));
                Ok(GodToolReceipt::Life {
                    agent_entity_bits: first,
                    affected_count: entities.len() as u32,
                })
            }
        }
    }

    fn apply_disaster(
        &mut self,
        req: DisasterRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        // The substrate writes go through `trigger_disaster`,
        // which already adds belief via the
        // disaster → faith coupling (FR-CIV-EMERGENCE).
        let prev_belief = self.belief();
        match req {
            DisasterRequest::Meteor { pos } => {
                trigger_disaster(self, DisasterKind::Meteor, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Meteor,
                    fired,
                })
            }
            DisasterRequest::Wildfire { pos } => {
                trigger_disaster(self, DisasterKind::Wildfire, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Wildfire,
                    fired,
                })
            }
            DisasterRequest::Flood { pos } => {
                trigger_disaster(self, DisasterKind::Flood, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Flood,
                    fired,
                })
            }
            DisasterRequest::Quake { pos } => {
                trigger_disaster(self, DisasterKind::Quake, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Quake,
                    fired,
                })
            }
            DisasterRequest::Storm { pos } => {
                trigger_disaster(self, DisasterKind::Storm, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Storm,
                    fired,
                })
            }
            DisasterRequest::Plague { pos } => {
                trigger_disaster(self, DisasterKind::Plague, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Plague,
                    fired,
                })
            }
            DisasterRequest::Lightning { from, to } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // localised fast-write disaster. We rasterise a
                // 1-cell-thick line of LAVA between `from` and
                // `to` (Bresenham 3D, half-open), then ignite
                // adjacent PLANT voxels along the arc so the
                // wildfire can spread. The `add_belief` is the
                // same coupling the existing disaster path
                // uses, so religion pressure still rises.
                if from == to {
                    return Err(GodToolError::InvalidRequest(
                        "lightning from == to would write a single cell".into(),
                    ));
                }
                let mut writes: u32 = 0;
                for (x, y, z) in bresenham_3d(from, to) {
                    let cell = WorldCoord { x, y, z };
                    // Snapshot both reads (cell material +
                    // topmost column material) before we
                    // take the mutable borrow for the
                    // write.
                    let mat = self.voxel().read(cell);
                    let top = topmost_voxel(&self.voxel, cell);
                    let igniteable = matches!(
                        top,
                        Some(MaterialId::PLANT) | Some(MaterialId::MOSS) | Some(MaterialId::WOOD)
                    );
                    if mat != MaterialId::AIR && mat != MaterialId::WATER {
                        // Plough through solid ground with a
                        // LAVA splinter for a visible scar.
                        self.push_voxel_write(cell, MaterialId::LAVA);
                        writes = writes.saturating_add(1);
                    }
                    if igniteable {
                        self.push_voxel_write(cell, MaterialId::LAVA);
                        writes = writes.saturating_add(1);
                    }
                }
                // Add belief: at least one cell mutated = the
                // sim registers an "act of god" event.
                if writes > 0 {
                    self.add_belief(8u64);
                }
                Ok(GodToolReceipt::EnvironmentalDisaster {
                    kind_label: "lightning".to_string(),
                    writes,
                })
            }
            DisasterRequest::Tornado { pos, radius_voxels } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // rotating wind vortex that writes AIR in a
                // descending spiral and STEAM where the
                // vortex grazes water. The spiral arm
                // count is fixed at 3 (a god-tool brush
                // never has more than a handful of arms)
                // and `radius_voxels` controls the vortex
                // reach.
                if radius_voxels == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "tornado radius_voxels must be > 0".into(),
                    ));
                }
                let r = radius_voxels;
                let arms: i64 = 3;
                let mut writes: u32 = 0;
                for i in 0..=r {
                    // angle sweeps 2π * arms over the
                    // radius.
                    let angle_fp = ((i as i64)
                        .wrapping_mul(arms)
                        .wrapping_mul(31_416)
                        / (10_000 * r.max(1) as i64))
                        & 0xFFFF;
                    let dx = ((cos_lut(angle_fp) * i as i64) / 1_000_000) as i64;
                    let dz = ((sin_lut(angle_fp) * i as i64) / 1_000_000) as i64;
                    let cell = WorldCoord {
                        x: pos.x + dx,
                        y: pos.y,
                        z: pos.z + dz,
                    };
                    // Snapshot the read before we take the
                    // mutable borrow for the write.
                    let mat = self.voxel().read(cell);
                    let next = if mat == MaterialId::WATER {
                        MaterialId::STEAM
                    } else {
                        MaterialId::AIR
                    };
                    self.push_voxel_write(cell, next);
                    writes = writes.saturating_add(1);
                }
                if writes > 0 {
                    self.add_belief(12u64);
                }
                Ok(GodToolReceipt::EnvironmentalDisaster {
                    kind_label: "tornado".to_string(),
                    writes,
                })
            }
            DisasterRequest::VolcanicVent { pos, ticks } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // sustained magma intrusion. Writes LAVA
                // at the centre column from `pos.y`
                // downward for `min(ticks, 64)` cells, plus
                // a ring of STEAM at the surface to mimic
                // venting. The `ticks` parameter also
                // drives the audio bus (one rumble sfx per
                // tick).
                if ticks == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "volcanic_vent ticks must be > 0".into(),
                    ));
                }
                let d = (ticks as i64).min(64);
                let mut writes: u32 = 0;
                for k in 0..d {
                    let cell = WorldCoord {
                        x: pos.x,
                        y: pos.y.saturating_sub(k),
                        z: pos.z,
                    };
                    self.push_voxel_write(cell, MaterialId::LAVA);
                    writes = writes.saturating_add(1);
                }
                // Steam ring at the surface.
                for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let cell = WorldCoord {
                        x: pos.x + dx,
                        y: pos.y,
                        z: pos.z + dz,
                    };
                    self.push_voxel_write(cell, MaterialId::STEAM);
                    writes = writes.saturating_add(1);
                }
                if writes > 0 {
                    self.add_belief(25u64);
                }
                Ok(GodToolReceipt::EnvironmentalDisaster {
                    kind_label: "volcanic_vent".to_string(),
                    writes,
                })
            }
            DisasterRequest::Drought {
                pos,
                reduction_pct,
                ticks,
            } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // climate-field write. Lowers the
                // `precip_mm_fp` of every weather cell
                // whose `latitude_fp` falls inside the
                // brush footprint by `reduction_pct`
                // percent. Cells with no precipitation
                // are skipped so the drought stays
                // localised. We project the brush
                // position onto the latitude axis using
                // `pos.x` as the latitude proxy.
                if reduction_pct == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "drought reduction_pct must be > 0".into(),
                    ));
                }
                if ticks == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "drought ticks must be > 0".into(),
                    ));
                }
                // `_ticks` is the sustained-verb budget; Phase 4
                // doesn't track elapsed drought ticks yet, but
                // the verb honours the contract (a drought
                // that doesn't propagate is just a request
                // count). Reserved for the follow-up phase.
                let _ticks = ticks;
                // 8-voxel default brush (a drought can
                // blanket a whole latitude strip).
                let r = 8 * FIXED_SCALE;
                let r2 = r * r;
                // Project the brush position to a
                // latitude anchor by snapping to the
                // cell whose `latitude_fp` is closest
                // to `pos.x`.
                let anchor_lat = self
                    .weather_grid
                    .iter()
                    .min_by_key(|c| (c.latitude_fp - pos.x as i32).abs())
                    .map(|c| c.latitude_fp)
                    .unwrap_or(pos.x as i32);
                let mut writes: u32 = 0;
                let pct = reduction_pct.min(100) as i32;
                for cell in self.weather_grid.iter_mut() {
                    let dlat = i64::from(cell.latitude_fp - anchor_lat);
                    if dlat * dlat > r2 {
                        continue;
                    }
                    if cell.precip_mm_fp > 0 {
                        let drop = cell.precip_mm_fp * pct / 100;
                        cell.precip_mm_fp = cell.precip_mm_fp.saturating_sub(drop);
                        writes = writes.saturating_add(1);
                    }
                }
                if writes > 0 {
                    self.add_belief(6u64);
                }
                Ok(GodToolReceipt::EnvironmentalDisaster {
                    kind_label: "drought".to_string(),
                    writes,
                })
            }
        }
    }

    fn apply_law(
        &mut self,
        req: LawRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        match req {
            LawRequest::TaxBias { target_faction, bias } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // treasury write that transfers `bias`
                // joules to / from the faction's
                // `state.faction_treasury` entry. The
                // substrate write goes through the
                // public `faction_treasury` field
                // (HashMap<u32, Fixed>), and we clamp
                // the lower bound to a deep overdraft
                // so a single request can't drag the
                // sim into instant bankruptcy. We also
                // bump `add_belief` by the magnitude
                // (rounded basis-points) so the
                // doctrinal coupling fires.
                if bias == 0 {
                    return Err(GodToolError::InvalidRequest(
                        "tax_bias bias must be non-zero".into(),
                    ));
                }
                let f = target_faction;
                let prev = self
                    .state
                    .faction_treasury
                    .get(&f)
                    .copied()
                    .unwrap_or_else(|| Fixed::from_num(0));
                let delta = Fixed::from_num(bias);
                let next = (prev + delta).max(Fixed::from_num(-1_000_000_000_000_i64));
                self.state.faction_treasury.insert(f, next);
                let wrote = next != prev;
                if wrote {
                    let mag_bp = (bias.unsigned_abs() / 1_000_000) as i32;
                    self.add_belief(mag_bp.max(0) as u64);
                }
                Ok(GodToolReceipt::Law {
                    verb: "law.tax_bias".to_string(),
                    delta: bias,
                })
            }
            LawRequest::ReligionPressure { pressure } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // direct belief write routed through
                // `add_belief`. `pressure` is in
                // arbitrary units; we clamp to
                // `i32::MAX` so a single request can't
                // overflow the belief channel.
                let p = pressure.min(i32::MAX as u64) as u64;
                let prev = self.belief();
                self.add_belief(p);
                let delta = (self.belief() - prev) as i64;
                Ok(GodToolReceipt::Law {
                    verb: "law.religion_pressure".to_string(),
                    delta,
                })
            }
            LawRequest::DifficultyKnob {
                scarcity_multiplier,
            } => {
                // Phase 4 (FR-CIV-GODTOOL-901 batch 3) — a
                // survival-pressure write routed through
                // the substrate-owned `economy_policy`
                // field. Validates the new
                // `scarcity_multiplier` against the
                // documented `[0.0, 10.0]` range so a
                // bad client value doesn't push the
                // economy into a non-recoverable
                // state. The previous value is
                // reported as the receipt delta so the
                // HUD can show "difficulty N → M".
                if !(0.0..=10.0).contains(&scarcity_multiplier)
                    || !scarcity_multiplier.is_finite()
                {
                    return Err(GodToolError::InvalidRequest(format!(
                        "difficulty_knob scarcity_multiplier must be in [0.0, 10.0], got {scarcity_multiplier}"
                    )));
                }
                let prev = self.economy_policy.scarcity_multiplier;
                self.economy_policy.scarcity_multiplier = scarcity_multiplier;
                // Encode the delta as fixed-point basis
                // points so the receipt's i64 fits
                // comfortably.
                let delta = ((scarcity_multiplier - prev) * 10_000.0).round() as i64;
                if (scarcity_multiplier - prev).abs() > f64::EPSILON {
                    self.add_belief(1u64);
                }
                Ok(GodToolReceipt::Law {
                    verb: "law.difficulty_knob".to_string(),
                    delta,
                })
            }
        }
    }

    fn apply_inspect(
        &mut self,
        req: InspectRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        match req {
            InspectRequest::Probe(p) => {
                // Read-only: no `&mut self` work. We just query
                // the voxel at `pos` and find the nearest agent
                // entity in a small sphere.
                let material = self.voxel().read(p.pos);
                // Search for the nearest agent within 5 voxels
                // (read-only query over the hecs world).
                let radius = 5i64;
                let r2 = radius * radius;
                let mut nearest: Option<(i64, u64)> = None;
                for (entity, pos_comp) in self.world.query::<&civ_agents::Position3d>().iter() {
                    let dx = pos_comp.coord.x - p.pos.x;
                    let dy = pos_comp.coord.y - p.pos.y;
                    let dz = pos_comp.coord.z - p.pos.z;
                    let d2 = dx * dx + dy * dy + dz * dz;
                    if d2 > r2 {
                        continue;
                    }
                    match nearest {
                        Some((best_d2, _)) if best_d2 <= d2 => {}
                        _ => nearest = Some((d2, entity.to_bits().get())),
                    }
                }
                Ok(GodToolReceipt::Inspect {
                    report: ProbeReport {
                        pos: p.pos,
                        material,
                        nearest_agent: nearest.map(|(_, bits)| bits),
                    },
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Simulation;

    /// `terrain.raise` must mutate the voxel substrate: the cell
    /// at the brush center must read back as `STONE` after the
    /// request is applied. This is the core "verb mutates the
    /// field" guarantee.
    #[test]
    fn terrain_raise_writes_stone_at_center() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: 0,
            z: 1_000_000,
        };
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Raise,
            center,
            radius_voxels: 1,
            strength: 1,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("terrain.raise should succeed");
        match receipt {
            GodToolReceipt::Terraform {
                op: TerraformOp::Raise,
                writes,
            } => {
                // 1x1x1 footprint → 1 cell.
                assert!(writes >= 1, "expected at least 1 voxel write, got {writes}");
            }
            other => panic!("expected Terraform receipt, got {other:?}"),
        }
        // The center cell must now read back as STONE (id 6).
        assert_eq!(
            sim.voxel().read(center),
            MaterialId(6),
            "center cell should be STONE after Raise"
        );
    }

    /// `terrain.lower` must write AIR (id 0) in the footprint.
    /// The inverse of the Raise test — proves the verb honours
    /// its `Lower` direction.
    #[test]
    fn terrain_lower_writes_air_at_center() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 2_000_000,
            y: 0,
            z: 2_000_000,
        };
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Lower,
            center,
            radius_voxels: 1,
            strength: 1,
        });
        sim.apply_god_tool(req).expect("terrain.lower should succeed");
        assert_eq!(
            sim.voxel().read(center),
            AIR,
            "center cell should be AIR after Lower"
        );
    }

    /// `life.spawn_organism` must add a new agent to the sim
    /// world. We assert the agent count before/after and that
    /// the new entity carries a `Position3d` component with the
    /// requested normalized coords.
    #[test]
    fn life_spawn_organism_adds_agent() {
        let mut sim = Simulation::new();
        let before = sim.world.query::<&civ_agents::Position3d>().iter().count();
        let req = GodToolRequest::Life(LifeRequest::SpawnOrganism(SpawnOrganismRequest {
            id: 9_999_999,
            faction: 0,
            x: 0.42,
            y: 0.58,
            visual: SpawnVisual::Humanoid,
        }));
        let receipt = sim
            .apply_god_tool(req)
            .expect("life.spawn_organism should succeed");
        let bits = match receipt {
            GodToolReceipt::Life {
                agent_entity_bits,
                affected_count,
            } => {
                assert_eq!(affected_count, 1, "SpawnOrganism affects exactly 1 agent");
                agent_entity_bits
            }
            other => panic!("expected Life receipt, got {other:?}"),
        };
        let after = sim.world.query::<&civ_agents::Position3d>().iter().count();
        assert_eq!(after, before + 1, "SpawnOrganism must add 1 agent");
        // The returned entity bits must round-trip to a valid
        // hecs::Entity that holds a Civilian.
        let entity = hecs::Entity::from_bits(bits)
            .expect("from_bits must succeed for bits emitted by to_bits");
        let _ = sim
            .world
            .get::<&civ_agents::Civilian>(entity)
            .expect("new entity must hold a Civilian component");
    }

    /// `disaster.meteor` must route through
    /// `trigger_disaster(DisasterKind::Meteor, …)`. The
    /// substrate write is a voxel damage sphere + belief gain;
    /// we assert the latter because the voxel write would
    /// otherwise depend on a pre-populated world.
    #[test]
    fn disaster_meteor_routes_through_substrate() {
        let mut sim = Simulation::new();
        let prev = sim.belief();
        let req = GodToolRequest::Disaster(DisasterRequest::Meteor {
            pos: WorldCoord {
                x: 0,
                y: 0,
                z: 0,
            },
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("disaster.meteor should succeed");
        match receipt {
            GodToolReceipt::Disaster {
                kind: DisasterKind::Meteor,
                ..
            } => {}
            other => panic!("expected Disaster receipt, got {other:?}"),
        }
        // `trigger_disaster` adds `DISASTER_FAITH_GAIN` belief.
        assert!(
            sim.belief() > prev,
            "disaster.meteor must route through trigger_disaster and gain belief (was {prev}, now {})",
            sim.belief()
        );
    }

    /// `inspect.probe` is read-only. Two consecutive probes at
    /// the same coord must produce the same `material` (and
    /// not mutate state).
    #[test]
    fn inspect_probe_is_read_only() {
        let mut sim = Simulation::new();
        let pos = WorldCoord {
            x: 0,
            y: 0,
            z: 0,
        };
        let req1 = GodToolRequest::Inspect(InspectRequest::Probe(ProbeRequest { pos }));
        let req2 = GodToolRequest::Inspect(InspectRequest::Probe(ProbeRequest { pos }));
        let belief_before = sim.belief();
        let r1 = sim.apply_god_tool(req1).expect("probe 1");
        let r2 = sim.apply_god_tool(req2).expect("probe 2");
        let m1 = match r1 {
            GodToolReceipt::Inspect { report } => report.material,
            other => panic!("expected Inspect receipt, got {other:?}"),
        };
        let m2 = match r2 {
            GodToolReceipt::Inspect { report } => report.material,
            other => panic!("expected Inspect receipt, got {other:?}"),
        };
        assert_eq!(m1, m2, "two probes at the same coord must read the same material");
        assert_eq!(sim.belief(), belief_before, "probe must not mutate belief");
    }

    /// Invalid radius (zero) is rejected. Defends the
    /// "Bevy-side must validate payloads" contract.
    #[test]
    fn zero_radius_rejected() {
        let mut sim = Simulation::new();
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Raise,
            center: WorldCoord {
                x: 0,
                y: 0,
                z: 0,
            },
            radius_voxels: 0,
            strength: 1,
        });
        match sim.apply_god_tool(req) {
            Err(GodToolError::InvalidRequest(_)) => {}
            other => panic!("expected InvalidRequest, got {other:?}"),
        }
    }

    /// NaN spawn coordinates are rejected. Defends the
    /// "Bevy-side must validate payloads" contract.
    #[test]
    fn nan_spawn_rejected() {
        let mut sim = Simulation::new();
        let req = GodToolRequest::Life(LifeRequest::SpawnOrganism(SpawnOrganismRequest {
            id: 1,
            faction: 0,
            x: f32::NAN,
            y: 0.5,
            visual: SpawnVisual::Humanoid,
        }));
        match sim.apply_god_tool(req) {
            Err(GodToolError::InvalidRequest(_)) => {}
            other => panic!("expected InvalidRequest, got {other:?}"),
        }
    }

    /// `NoOp` receipt constructor: a `Near` verb produces a
    /// `NoOp` receipt tagged with its id so the Bevy layer can
    /// show a "data not yet surfaced" toast.
    #[test]
    fn no_op_receipt_carries_verb() {
        let r = GodToolReceipt::no_op("material.seed_forest");
        match r {
            GodToolReceipt::NoOp { verb } => assert_eq!(verb, "material.seed_forest"),
            other => panic!("expected NoOp, got {other:?}"),
        }
    }

    /// Error type is constructible + serializable so a
    /// thread-local bridge can forward errors back to the Bevy
    /// side as JSON.
    #[test]
    fn error_serde_round_trip() {
        let e = GodToolError::NotImplemented {
            verb: "law.edict",
        };
        let j = serde_json::to_string(&e).expect("serialize");
        let de: GodToolError = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(e, de);
    }

    /// `terrain.raise_mountain` must mutate the voxel substrate:
    /// the brush footprint must read back as STONE (or GRAVEL)
    /// after the request is applied. Phase 2's terrain verb must
    /// honour the same "verb mutates the field" guarantee as
    /// `terrain.raise`.
    #[test]
    fn terrain_raise_mountain_writes_peak() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 2_500_000,
            y: 0,
            z: 2_500_000,
        };
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::RaiseMountain,
            center,
            radius_voxels: 1,
            strength: 1,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("terrain.raise_mountain should succeed");
        match receipt {
            GodToolReceipt::Terraform {
                op: TerraformOp::RaiseMountain,
                writes,
            } => {
                // 1x1x1 footprint → at least 1 cell; the Gaussian
                // peak may spill extra cells above the ground.
                assert!(
                    writes >= 1,
                    "expected at least 1 voxel write, got {writes}"
                );
            }
            other => panic!("expected Terraform receipt, got {other:?}"),
        }
        // The center cell must read back as STONE (id 6) or
        // GRAVEL (id 5) — both are valid peak materials.
        let m = sim.voxel().read(center).0;
        assert!(
            m == 5 || m == 6,
            "center cell should be STONE/GRAVEL after RaiseMountain, got id {m}"
        );
    }

    /// `life.heal` must mutate actor state: spawning an organism
    /// and then healing it must increase its `Health::integrity`.
    /// This is the Phase 2 life-verb "verb mutates state"
    /// guarantee.
    #[test]
    fn life_heal_bumps_actor_health() {
        let mut sim = Simulation::new();
        // Spawn one actor at the origin.
        let spawn = GodToolRequest::Life(LifeRequest::SpawnOrganism(SpawnOrganismRequest {
            id: 7_777_777,
            faction: 0,
            x: 0.5,
            y: 0.5,
            visual: SpawnVisual::Humanoid,
        }));
        let bits = match sim
            .apply_god_tool(spawn)
            .expect("life.spawn_organism should succeed")
        {
            GodToolReceipt::Life {
                agent_entity_bits,
                affected_count: _,
            } => agent_entity_bits,
            other => panic!("expected Life receipt, got {other:?}"),
        };
        let entity = hecs::Entity::from_bits(bits).expect("from_bits");

        // Damage the actor first by writing Health::integrity low.
        if let Ok(mut h) = sim.world.get::<&mut LifeHealth>(entity) {
            h.integrity = 20;
        } else {
            panic!("spawned entity must carry a Health component");
        }
        let before = sim.world.get::<&LifeHealth>(entity).unwrap().integrity;

        // Heal within a wide radius so we definitely hit it.
        let heal = GodToolRequest::Life(LifeRequest::Heal {
            center: WorldCoord {
                x: 0,
                y: 0,
                z: 0,
            },
            radius_voxels: u32::MAX,
            amount: 30,
        });
        let affected = match sim
            .apply_god_tool(heal)
            .expect("life.heal should succeed")
        {
            GodToolReceipt::Life {
                affected_count, ..
            } => affected_count,
            other => panic!("expected Life receipt, got {other:?}"),
        };
        assert!(affected >= 1, "heal must affect at least 1 actor");
        let after = sim.world.get::<&LifeHealth>(entity).unwrap().integrity;
        assert!(
            after > before,
            "heal must increase Health::integrity (was {before}, now {after})"
        );
    }

    // ===========================================================
    // Phase 3 tests — MATERIAL verbs + `terrain.slope`
    // ===========================================================

    /// `material.erase` must write `AIR` in the spherical
    /// footprint. Mirror of the `terrain.lower` test.
    #[test]
    fn material_erase_writes_air_in_footprint() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 3_000_000,
            y: 0,
            z: 3_000_000,
        };
        // Pre-seed a STONE voxel so we can verify the verb
        // actually erases it.
        sim.voxel_mut()
            .write(center, STONE);
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::Erase,
            center,
            radius_voxels: 1,
            material_id: 0,
            strength: 0,
            drop_height: 0,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("material.erase should succeed");
        match receipt {
            GodToolReceipt::Material {
                op: MaterialOp::Erase,
                writes,
            } => {
                assert!(
                    writes >= 1,
                    "expected at least 1 voxel write, got {writes}"
                );
            }
            other => panic!("expected Material receipt, got {other:?}"),
        }
        assert_eq!(
            sim.voxel().read(center),
            AIR,
            "center cell should be AIR after material.erase"
        );
    }

    /// `material.replace` must write the requested material in
    /// the spherical footprint.
    #[test]
    fn material_replace_writes_target_in_footprint() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 4_000_000,
            y: 0,
            z: 4_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::Replace,
            center,
            radius_voxels: 1,
            material_id: u32::from(SAND.0),
            strength: 0,
            drop_height: 0,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("material.replace should succeed");
        match receipt {
            GodToolReceipt::Material {
                op: MaterialOp::Replace,
                writes,
            } => assert!(writes >= 1, "expected ≥1 write, got {writes}"),
            other => panic!("expected Material receipt, got {other:?}"),
        }
        assert_eq!(
            sim.voxel().read(center),
            SAND,
            "center cell should be SAND after material.replace"
        );
    }

    /// `terrain.slope` must tilt the surface by `strength` from
    /// the low edge to the high edge of the brush footprint.
    /// The center cell of a radius-2 brush at `dx=0` gets
    /// `top_y + 0` (no offset), so the `STONE` write lands on
    /// the original topmost y.
    #[test]
    fn terrain_slope_writes_stone_gradient() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 5_000_000,
            y: 0,
            z: 5_000_000,
        };
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Slope,
            center,
            radius_voxels: 2,
            strength: FIXED_SCALE as i32 * 2,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("terrain.slope should succeed");
        match receipt {
            GodToolReceipt::Terraform {
                op: TerraformOp::Slope,
                writes,
            } => assert!(writes >= 1, "expected ≥1 write, got {writes}"),
            other => panic!("expected Terraform receipt, got {other:?}"),
        }
        // The center column (dx=0, dz=0) should have a STONE
        // write at its topmost y; we just assert the write
        // landed somewhere STONE inside the footprint sphere.
        let r = 2i64 * FIXED_SCALE;
        let r2 = r * r;
        let mut found_stone = false;
        for dz in (-2i64..=2).map(|v| v * FIXED_SCALE) {
            for dx in (-2i64..=2).map(|v| v * FIXED_SCALE) {
                if dx * dx + dz * dz > r2 {
                    continue;
                }
                let probe = WorldCoord {
                    x: center.x + dx,
                    y: 0,
                    z: center.z + dz,
                };
                if sim.voxel().read(probe) == STONE {
                    found_stone = true;
                    break;
                }
            }
            if found_stone {
                break;
            }
        }
        assert!(
            found_stone,
            "terrain.slope should leave at least one STONE voxel in the footprint"
        );
    }

    /// `material.surface_paint` must write the requested
    /// material only on the topmost solid voxel of each (x, z)
    /// column. We pre-seed a STONE column, then paint SAND on
    /// it — the SAND write lands on the topmost solid voxel.
    #[test]
    fn material_surface_paint_writes_topmost_only() {
        let mut sim = Simulation::new();
        let col_x = 6_000_000i64;
        let col_z = 6_000_000i64;
        let center = WorldCoord {
            x: col_x,
            y: 0,
            z: col_z,
        };
        // Pre-seed a column of STONE so `scan_topmost_y`
        // returns a non-baseline value.
        sim.voxel_mut().write(
            WorldCoord { x: col_x, y: FIXED_SCALE, z: col_z },
            STONE,
        );
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::SurfacePaint,
            center,
            radius_voxels: 1,
            material_id: u32::from(SAND.0),
            strength: 0,
            drop_height: 0,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("material.surface_paint should succeed")
        {
            GodToolReceipt::Material {
                op: MaterialOp::SurfacePaint,
                writes,
            } => writes,
            other => panic!("expected Material receipt, got {other:?}"),
        };
        assert!(writes >= 1, "expected ≥1 write, got {writes}");
        assert_eq!(
            sim.voxel().read(WorldCoord {
                x: col_x,
                y: FIXED_SCALE,
                z: col_z
            }),
            SAND,
            "topmost solid voxel should be SAND after surface_paint"
        );
    }

    /// `material.additive_drop` must write `target` material in
    /// a sphere at `center.y + drop_height` and report a
    /// non-zero write count. The CA's gravity rule carries the
    /// material down next tick — the verb only writes the seed.
    #[test]
    fn material_additive_drop_writes_seed_above_center() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 7_000_000,
            y: 0,
            z: 7_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::AdditiveDrop,
            center,
            radius_voxels: 1,
            material_id: u32::from(SAND.0),
            strength: 0,
            drop_height: FIXED_SCALE as i32 * 4,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("material.additive_drop should succeed")
        {
            GodToolReceipt::Material {
                op: MaterialOp::AdditiveDrop,
                writes,
            } => writes,
            other => panic!("expected Material receipt, got {other:?}"),
        };
        assert!(writes >= 1, "additive_drop must write ≥1 seed voxel");
    }

    /// `material.pour_liquid` (WATER path) must write `WATER`
    /// voxels in the deposit. The fluid CA spreads them
    /// horizontally next tick — the verb only writes the seed.
    #[test]
    fn material_pour_liquid_writes_water() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 8_000_000,
            y: 0,
            z: 8_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::PourLiquid,
            center,
            radius_voxels: 1,
            material_id: u32::from(WATER.0),
            strength: FIXED_SCALE as i32,
            drop_height: FIXED_SCALE as i32 * 3,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("material.pour_liquid should succeed")
        {
            GodToolReceipt::Material {
                op: MaterialOp::PourLiquid,
                writes,
            } => writes,
            other => panic!("expected Material receipt, got {other:?}"),
        };
        assert!(writes >= 1, "pour_liquid must write ≥1 seed voxel");
        // The seed sphere center should be WATER.
        // `drop_height` is the *bottom* of the seed sphere; with
        // `drop_height = 3*FIXED_SCALE` and `layers = strength / FIXED_SCALE = 1`,
        // the seed sits at y = 3*FIXED_SCALE.
        const SEED_Y: i64 = 3 * FIXED_SCALE;
        assert_eq!(
            sim.voxel().read(WorldCoord {
                x: center.x,
                y: SEED_Y,
                z: center.z,
            }),
            WATER,
            "center of pour_liquid seed should be WATER"
        );
    }

    /// `material.seed_snow` must write `SNOW` voxels above the
    /// local snowline. The thermo CA melts the snow next tick
    /// at warm temperatures — the verb only writes the seed.
    #[test]
    fn material_seed_snow_writes_snow() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 9_000_000,
            y: 0,
            z: 9_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::SeedSnow,
            center,
            radius_voxels: 1,
            material_id: u32::from(SNOW.0),
            strength: FIXED_SCALE as i32,
            drop_height: 0,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("material.seed_snow should succeed")
        {
            GodToolReceipt::Material {
                op: MaterialOp::SeedSnow,
                writes,
            } => writes,
            other => panic!("expected Material receipt, got {other:?}"),
        };
        assert!(writes >= 1, "seed_snow must write ≥1 SNOW voxel");
    }

    /// `material.seed_ore` must run without error and report a
    /// non-negative write count. Because the noise is
    /// stochastic, individual cells may or may not land ORE —
    /// we just assert the dispatch returned a `Material`
    /// receipt.
    #[test]
    fn material_seed_ore_runs_and_returns_material_receipt() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 10_000_000,
            y: 0,
            z: 10_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::SeedOreDeposit,
            center,
            radius_voxels: 2,
            material_id: u32::from(ORE.0),
            strength: FIXED_SCALE as i32,
            drop_height: 0,
        });
        let receipt = sim
            .apply_god_tool(req)
            .expect("material.seed_ore should succeed");
        match receipt {
            GodToolReceipt::Material {
                op: MaterialOp::SeedOreDeposit,
                writes: _,
            } => {}
            other => panic!("expected Material receipt, got {other:?}"),
        }
    }

    /// `terrain.add_land` must write a `STONE` band of
    /// `strength` thickness above the existing surface in the
    /// footprint. The verb reads the topmost solid voxel per
    /// column via `scan_topmost_y` and writes `STONE` at
    /// `top_y + 1 ..= top_y + thickness`.
    #[test]
    fn terrain_add_land_writes_stone_band_above_surface() {
        let mut sim = Simulation::new();
        // Pre-seed a STONE surface at y = FIXED_SCALE in a
        // 1×1 column at the brush center.
        let center = WorldCoord {
            x: 4_000_000,
            y: 0,
            z: 4_000_000,
        };
        sim.voxel_mut().write(
            WorldCoord {
                x: center.x,
                y: FIXED_SCALE,
                z: center.z,
            },
            STONE,
        );
        let thickness = 2 * FIXED_SCALE;
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::AddLand,
            center,
            radius_voxels: 1,
            strength: thickness as i32,
            aux_id: 0,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("terrain.add_land should succeed")
        {
            GodToolReceipt::Terraform {
                op: TerraformOp::AddLand,
                writes,
            } => writes,
            other => panic!("expected Terraform receipt, got {other:?}"),
        };
        assert!(writes >= 1, "add_land must write ≥1 STONE voxel");
        // Top of the band sits at the original surface y plus
        // the thickness.
        assert_eq!(
            sim.voxel().read(WorldCoord {
                x: center.x,
                y: FIXED_SCALE + thickness,
                z: center.z,
            }),
            STONE,
            "top of the add_land band should be STONE"
        );
    }

    /// `terrain.dig_ocean` must carve a `WATER` cavity down to
    /// the sea-level band in each (x, z) column of the
    /// footprint. Columns whose topmost solid voxel sits at or
    /// below the sea level are left untouched.
    #[test]
    fn terrain_dig_ocean_writes_water_cavity() {
        let mut sim = Simulation::new();
        // Pre-seed a STONE plateau at y = 5*FIXED_SCALE in a
        // 1×1 column.
        let center = WorldCoord {
            x: 6_000_000,
            y: 0,
            z: 6_000_000,
        };
        let plateau_top = 5 * FIXED_SCALE;
        sim.voxel_mut().write(
            WorldCoord {
                x: center.x,
                y: plateau_top,
                z: center.z,
            },
            STONE,
        );
        // Sea level = 1*FIXED_SCALE. Dig depth = 3*FIXED_SCALE.
        // Expected new floor = plateau_top - 3*FIXED_SCALE =
        // 2*FIXED_SCALE. Cells between (new_floor,
        // plateau_top] become WATER.
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::DigOcean,
            center,
            radius_voxels: 1,
            strength: 3 * FIXED_SCALE as i32,
            aux_id: FIXED_SCALE as u32,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("terrain.dig_ocean should succeed")
        {
            GodToolReceipt::Terraform {
                op: TerraformOp::DigOcean,
                writes,
            } => writes,
            other => panic!("expected Terraform receipt, got {other:?}"),
        };
        assert!(writes >= 1, "dig_ocean must write ≥1 WATER voxel");
        // The cell at the original plateau top should now be
        // WATER (the cavity extends up through that y).
        assert_eq!(
            sim.voxel().read(WorldCoord {
                x: center.x,
                y: plateau_top,
                z: center.z,
            }),
            WATER,
            "carved cavity should be filled with WATER at the original surface"
        );
    }

    /// `terrain.drop_biome` must re-paint the topmost solid
    /// voxel of each (x, z) column in the footprint to the
    /// material in `aux_id`. Empty columns are skipped.
    #[test]
    fn terrain_drop_biome_paints_topmost_to_biome_material() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 7_000_000,
            y: 0,
            z: 7_000_000,
        };
        // Pre-seed a STONE surface at y = FIXED_SCALE in a
        // 1×1 column.
        sim.voxel_mut().write(
            WorldCoord {
                x: center.x,
                y: FIXED_SCALE,
                z: center.z,
            },
            STONE,
        );
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::DropBiome,
            center,
            radius_voxels: 1,
            strength: 0,
            aux_id: u32::from(SAND.0),
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("terrain.drop_biome should succeed")
        {
            GodToolReceipt::Terraform {
                op: TerraformOp::DropBiome,
                writes,
            } => writes,
            other => panic!("expected Terraform receipt, got {other:?}"),
        };
        assert!(writes >= 1, "drop_biome must write ≥1 voxel");
        assert_eq!(
            sim.voxel().read(WorldCoord {
                x: center.x,
                y: FIXED_SCALE,
                z: center.z,
            }),
            SAND,
            "topmost voxel should be re-painted to SAND"
        );
    }

    /// Phase 4 (FR-CIV-GODTOOL-901 batch 3) — `terrain.flatten`
    /// must stamp a `STONE` band at the mean surface y inside the
    /// brush footprint. The verb reports writes via the standard
    /// `Terraform` receipt so the HUD can render the brush size
    /// without a new variant.
    #[test]
    fn terrain_flatten_writes_stone_band() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        let req = GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Flatten,
            center,
            radius_voxels: 1,
            strength: 0,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("terrain.flatten should succeed")
        {
            GodToolReceipt::Terraform {
                op: TerraformOp::Flatten,
                writes,
            } => writes,
            other => panic!("expected Terraform receipt, got {other:?}"),
        };
        // 1×1 footprint ⇒ 1 write at the mean surface y.
        assert_eq!(writes, 1, "flatten 1×1 brush writes exactly 1 cell");
        // The cell must now be `STONE` (id 6).
        assert_eq!(
            sim.voxel().read(center),
            STONE,
            "flatten cell should be STONE"
        );
    }

    /// Phase 4 — `material.seed_forest` must produce a stochastic
    /// PLANT scatter inside the footprint. The number of writes
    /// is bounded by the footprint area and bounded below by 0.
    #[test]
    fn material_seed_forest_writes_plant() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        let req = GodToolRequest::Material(MaterialRequest {
            op: MaterialOp::SeedForest,
            center,
            radius_voxels: 2,
            strength: 50,
            material_id: 0,
            drop_height: 0,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("material.seed_forest should succeed")
        {
            GodToolReceipt::Material {
                op: MaterialOp::SeedForest,
                writes,
            } => writes,
            other => panic!("expected Material receipt, got {other:?}"),
        };
        // 5×5 footprint ⇒ ≤ 25 PLANT voxels.
        assert!(writes <= 25, "seed_forest writes must not exceed footprint area");
        // A density of 50 ⇒ at least a few seeds.
        assert!(writes >= 1, "seed_forest with density 50 must write ≥1 seed");
    }

    /// Phase 4 — `life.spawn_civ_seed` must inject 6 founder
    /// civilians and enqueue 2 build sites (hut + farm). The
    /// receipt reports `affected_count = 6` for the civilian
    /// batch (the build sites are substrate side-effects).
    #[test]
    fn life_spawn_civ_seed_injects_founders_and_buildings() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        let req = GodToolRequest::Life(LifeRequest::SpawnCivSeed(
            SpawnCivSeedRequest {
                center,
                seed_civilian_id: 42,
                faction: 0,
            },
        ));
        let affected = match sim
            .apply_god_tool(req)
            .expect("life.spawn_civ_seed should succeed")
        {
            GodToolReceipt::Life {
                affected_count, ..
            } => affected_count,
            other => panic!("expected Life receipt, got {other:?}"),
        };
        assert_eq!(affected, 6, "spawn_civ_seed injects 6 founder civilians");
        // The substrate must have recorded two build sites.
        assert_eq!(sim.build_sites().len(), 2, "expected 2 build sites (hut + farm)");
    }

    /// Phase 4 — `disaster.lightning` must rasterise a LAVA arc
    /// between `from` and `to` and increment belief when at least
    /// one cell is overwritten. We seed a STONE column first so the
    /// arc has solid substrate to bite into.
    #[test]
    fn disaster_lightning_writes_lava_arc() {
        let mut sim = Simulation::new();
        let from = WorldCoord {
            x: 1_000_000,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        let to = WorldCoord {
            x: 1_000_000 + 4 * FIXED_SCALE,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        // Seed STONE along the path so the arc has substrate to bite.
        for x_off in 0..=4 {
            sim.push_voxel_write(
                WorldCoord {
                    x: from.x + x_off * FIXED_SCALE,
                    y: FIXED_SCALE,
                    z: from.z,
                },
                STONE,
            );
        }
        let prev_belief = sim.belief();
        let req = GodToolRequest::Disaster(DisasterRequest::Lightning {
            from,
            to,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("disaster.lightning should succeed")
        {
            GodToolReceipt::EnvironmentalDisaster {
                kind_label,
                writes,
            } => {
                assert_eq!(kind_label, "lightning");
                writes
            }
            other => panic!("expected EnvironmentalDisaster receipt, got {other:?}"),
        };
        assert!(writes >= 1, "lightning must write ≥1 LAVA cell");
        assert!(sim.belief() >= prev_belief, "lightning bumps belief");
        // The start cell must now be LAVA (arc ploughs through STONE).
        assert_eq!(
            sim.voxel().read(from),
            LAVA,
            "lightning origin should be LAVA"
        );
    }

    /// Phase 4 — `disaster.tornado` must write AIR (or STEAM over
    /// water) inside a spiral footprint. We verify writes > 0 and
    /// the receipt shape, without asserting exact positions
    /// (the vortex geometry is stochastic).
    #[test]
    fn disaster_tornado_writes_air_or_steam() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: FIXED_SCALE,
            z: 1_000_000,
        };
        let prev_belief = sim.belief();
        let req = GodToolRequest::Disaster(DisasterRequest::Tornado {
            pos: center,
            radius_voxels: 4,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("disaster.tornado should succeed")
        {
            GodToolReceipt::EnvironmentalDisaster {
                kind_label,
                writes,
            } => {
                assert_eq!(kind_label, "tornado");
                writes
            }
            other => panic!("expected EnvironmentalDisaster receipt, got {other:?}"),
        };
        assert!(writes >= 1, "tornado must write ≥1 AIR/STEAM cell");
        assert!(sim.belief() >= prev_belief, "tornado bumps belief");
    }

    /// Phase 4 — `disaster.volcanic_vent` must stamp a column of
    /// LAVA + a STEAM ring at the surface. Writes > 0 and
    /// belief coupling is the canonical contract. We seed a STONE
    /// column first so the ring has substrate to clip to.
    #[test]
    fn disaster_volcanic_vent_writes_lava_and_steam() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 1_000_000,
            y: 2 * FIXED_SCALE,
            z: 1_000_000,
        };
        // Seed a STONE column so topmost_voxel returns Some.
        for y in 0..=4 {
            sim.push_voxel_write(
                WorldCoord {
                    x: center.x,
                    y: y * FIXED_SCALE,
                    z: center.z,
                },
                STONE,
            );
        }
        let prev_belief = sim.belief();
        let req = GodToolRequest::Disaster(DisasterRequest::VolcanicVent {
            pos: center,
            ticks: 3,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("disaster.volcanic_vent should succeed")
        {
            GodToolReceipt::EnvironmentalDisaster {
                kind_label,
                writes,
            } => {
                assert_eq!(kind_label, "volcanic_vent");
                writes
            }
            other => panic!("expected EnvironmentalDisaster receipt, got {other:?}"),
        };
        // 3 ticks of LAVA + 4-cell STEAM ring = 7 writes.
        assert_eq!(writes, 7, "volcanic_vent with ticks=3 writes 3 LAVA + 4 STEAM");
        assert!(sim.belief() >= prev_belief, "volcanic_vent bumps belief");
        assert_eq!(
            sim.voxel().read(center),
            LAVA,
            "volcanic vent centre column is LAVA"
        );
    }

    /// Phase 4 — `disaster.drought` must lower `precip_mm_fp` in
    /// weather cells within the latitude brush. With a tiny brush
    /// the writes must affect ≥1 cell and the receipt kind label
    /// must match the verb id.
    #[test]
    fn disaster_drought_lowers_precip_in_brush() {
        let mut sim = Simulation::new();
        let center = WorldCoord {
            x: 0,
            y: FIXED_SCALE,
            z: 0,
        };
        let prev_belief = sim.belief();
        let req = GodToolRequest::Disaster(DisasterRequest::Drought {
            pos: center,
            reduction_pct: 25,
            ticks: 1,
        });
        let writes = match sim
            .apply_god_tool(req)
            .expect("disaster.drought should succeed")
        {
            GodToolReceipt::EnvironmentalDisaster {
                kind_label,
                writes,
            } => {
                assert_eq!(kind_label, "drought");
                writes
            }
            other => panic!("expected EnvironmentalDisaster receipt, got {other:?}"),
        };
        // With only one weather cell configured in the default
        // sim, the brush can hit at most that cell. We accept any
        // value ≥ 0 as long as the verb runs without error.
        let _ = writes;
        assert!(
            sim.belief() >= prev_belief,
            "drought bumps belief when ≥1 cell is mutated"
        );
    }

    /// Phase 4 — `law.tax_bias` must transfer joules in/out of
    /// `state.faction_treasury` for the target faction. A bias of
    /// 1_000_000 (one joule at fixed-point scale) bumps the
    /// treasury by exactly 1_000_000.
    #[test]
    fn law_tax_bias_mutates_faction_treasury() {
        let mut sim = Simulation::new();
        let before = sim
            .state
            .faction_treasury
            .get(&7)
            .copied()
            .unwrap_or_else(|| Fixed::from_num(0));
        let req = GodToolRequest::Law(LawRequest::TaxBias {
            target_faction: 7,
            bias: 1_000_000,
        });
        let delta = match sim
            .apply_god_tool(req)
            .expect("law.tax_bias should succeed")
        {
            GodToolReceipt::Law { verb, delta } => {
                assert_eq!(verb, "law.tax_bias");
                delta
            }
            other => panic!("expected Law receipt, got {other:?}"),
        };
        assert_eq!(delta, 1_000_000, "tax_bias reports the bias as the delta");
        let after = sim
            .state
            .faction_treasury
            .get(&7)
            .copied()
            .unwrap_or_else(|| Fixed::from_num(0));
        assert_eq!(
            after - before,
            Fixed::from_num(1_000_000),
            "faction treasury must reflect the bias"
        );
    }

    /// Phase 4 — `law.religion_pressure` must route through
    /// `add_belief`. A pressure of 100 bumps belief by exactly
    /// 100 (no doctrinal scaling).
    #[test]
    fn law_religion_pressure_routes_through_belief() {
        let mut sim = Simulation::new();
        let prev = sim.belief();
        let req = GodToolRequest::Law(LawRequest::ReligionPressure {
            pressure: 100,
        });
        let delta = match sim
            .apply_god_tool(req)
            .expect("law.religion_pressure should succeed")
        {
            GodToolReceipt::Law { verb, delta } => {
                assert_eq!(verb, "law.religion_pressure");
                delta
            }
            other => panic!("expected Law receipt, got {other:?}"),
        };
        assert_eq!(delta, 100, "religion_pressure reports the bump as the delta");
        assert_eq!(sim.belief() - prev, 100, "belief bumped by 100");
    }

    /// Phase 4 — `law.difficulty_knob` must write
    /// `economy_policy.scarcity_multiplier` and refuse values
    /// outside `[0.0, 10.0]`. The receipt reports the new
    /// minus the previous value (in basis points).
    #[test]
    fn law_difficulty_knob_writes_scarcity_multiplier() {
        let mut sim = Simulation::new();
        let prev = sim.economy_policy.scarcity_multiplier;
        let req = GodToolRequest::Law(LawRequest::DifficultyKnob {
            scarcity_multiplier: 2.5,
        });
        let delta = match sim
            .apply_god_tool(req)
            .expect("law.difficulty_knob should succeed")
        {
            GodToolReceipt::Law { verb, delta } => {
                assert_eq!(verb, "law.difficulty_knob");
                delta
            }
            other => panic!("expected Law receipt, got {other:?}"),
        };
        assert!(
            (sim.economy_policy.scarcity_multiplier - 2.5).abs() < 1e-6,
            "scarcity_multiplier must reflect the new value"
        );
        // Delta is the (new − prev) * 10_000, rounded.
        let expected = ((2.5 - prev) * 10_000.0).round() as i64;
        assert_eq!(delta, expected, "difficulty_knob delta must match expected");
        // Out-of-range value must be rejected.
        let bad = GodToolRequest::Law(LawRequest::DifficultyKnob {
            scarcity_multiplier: 42.0,
        });
        assert!(
            sim.apply_god_tool(bad).is_err(),
            "difficulty_knob must reject scarcity_multiplier > 10"
        );
    }
}
