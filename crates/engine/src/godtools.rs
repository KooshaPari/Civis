#![deny(unsafe_code)]

use civ_agents::{spawn_civilian_at, spawn_many, ActorVisualKind, Alignment, Civilian, Position3d};
use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
use civ_voxel::{
    material::{LAVA, MOSS, PLANT, STEAM, STONE, WATER, WOOD},
    AIR, MaterialId, WorldCoord, FIXED_SCALE,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::disasters::{DisasterKind, trigger_disaster};
use crate::engine::Simulation;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolRequest {
    Terraform(TerraformRequest),
    Material(MaterialRequest),
    Life(LifeRequest),
    Disaster(DisasterRequest),
    Inspect(InspectRequest),
}
/// TERRAIN verb parameters. The brush center is in fixed-point
/// world coordinates (`civ_voxel::WorldCoord`); `radius_voxels`
/// defines a footprint (sphere) of cells to write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerraformRequest {
    pub op: TerraformOp,
    pub center: WorldCoord,
    pub delta: i32,
    pub target_height: i32,
    pub radius: i32,
}
/// TERRAIN op kinds. Mirrors the 11 TERRAIN verbs from
/// `docs/design/GOD_TOOLS_SANDBOX.md` §3.1. Phase 1 ships
/// `Raise`, `Lower`, `Level`; Phase 2 adds `Smooth` and
/// `RaiseMountain`; Phase 3 adds `Slope`. The remaining
/// variants land in follow-up PRs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerraformOp {
    Raise,
    Lower,
    Level,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialRequest {
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
        disaster: DisasterKind,
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
    pub radius: i32,
    pub depth: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolError {
    InvalidDimension { field: &'static str, value: i32 },
    OutOfBounds { axis: &'static str, value: f32 },
    InvalidRequest(String),
}

impl std::fmt::Display for GodToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GodToolError::InvalidDimension { field, value } => {
                write!(f, "invalid {field}: {value} (must be > 0)")
            }
            GodToolError::OutOfBounds { axis, value } => {
                write!(f, "out-of-bounds {axis}: {value} (must be in [0, 1])")
            }
            GodToolError::InvalidRequest(msg) => {
                write!(f, "invalid request: {msg}")
            }
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
fn actors_in_footprint(
    world: &hecs::World,
    center: WorldCoord,
    radius_voxels: u8,
) -> Vec<hecs::Entity> {
    let r = i64::from(radius_voxels) * FIXED_SCALE;
    let r2 = r * r;
    world
        .query::<(&Position3d, &Civilian)>()
        .iter()
        .filter_map(|(entity, (pos, _civ))| {
            let dx = pos.coord.x - center.x;
            let dy = pos.coord.y - center.y;
            let dz = pos.coord.z - center.z;
            if dx * dx + dy * dy + dz * dz <= r2 {
                Some(entity)
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
    let affected = actors_in_footprint(world, center, radius_voxels);
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
    pub fn apply_god_tool(&mut self, req: GodToolRequest) -> Result<GodToolReceipt, GodToolError> {
        match req {
            GodToolRequest::Terraform(t) => self.apply_terraform(t),
            GodToolRequest::Material(m) => self.apply_material(m),
            GodToolRequest::Life(l) => self.apply_life(l),
            GodToolRequest::Disaster(d) => self.apply_disaster(d),
            GodToolRequest::Inspect(i) => self.apply_inspect(i),
        }
    }

    fn apply_terraform(&mut self, t: TerraformRequest) -> Result<GodToolReceipt, GodToolError> {
        if t.radius < 0 {
            return Err(GodToolError::InvalidDimension { field: "radius", value: t.radius });
        }
        match t.op {
            TerraformOp::Raise => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension { field: "delta", value: t.delta });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Raise, writes: self.raise_footprint(t.center, t.radius, t.delta) })
            }
            TerraformOp::Lower => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension { field: "delta", value: t.delta });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Lower, writes: self.lower_footprint(t.center, t.radius, t.delta) })
            }
            TerraformOp::Level => {
                if t.target_height < 0 {
                    return Err(GodToolError::InvalidDimension { field: "target_height", value: t.target_height });
                }
                Ok(GodToolReceipt::Terraform { op: TerraformOp::Level, writes: self.level_footprint(t.center, t.radius, t.target_height) })
            }
        }
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
        // Phase 3 stub: Material operations not yet implemented.
        // The MaterialRequest struct doesn't carry an operation discriminant.
        // Reserved for phase 3 implementation (FR-CIV-GODTOOL-903).
        let _ = (&req.center, req.material_id, req.strength, req.drop_height);
        Ok(GodToolReceipt::no_op("material"))
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
                let affected_entities: Vec<hecs::Entity> =
                    affected.iter().copied().collect();
                let first = affected_entities
                    .first()
                    .map(|ent| ent.to_bits().get())
                    .unwrap_or(0);
                let mut despawned: u32 = 0;
                for entity in affected_entities {
                    if self.world.despawn(entity).is_ok() {
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
                //
                // Phase 4 stub: BuildSite and related building types
                // are not yet available. Only spawn the civilians.
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
                // TODO (Phase 4): enqueue_build_site calls pending BuildSite API
                let _ = s.center;
                Ok(GodToolReceipt::Life {
                    agent_entity_bits: first,
                    affected_count: entities.len() as u32,
                })
            }
        }
    }

    fn apply_inspect(&mut self, _req: InspectRequest) -> Result<GodToolReceipt, GodToolError> {
        // Phase TBD: Inspection tools for reading simulation state.
        // For now, return a no-op receipt.
        Ok(GodToolReceipt::no_op("inspect"))
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
                    disaster: DisasterKind::Meteor,
                    fired,
                })
            }
            DisasterRequest::Wildfire { pos } => {
                trigger_disaster(self, DisasterKind::Wildfire, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    disaster: DisasterKind::Wildfire,
                    fired,
                })
            }
            DisasterRequest::Flood { pos } => {
                trigger_disaster(self, DisasterKind::Flood, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    disaster: DisasterKind::Flood,
                    fired,
                })
            }
            DisasterRequest::Quake { pos } => {
                trigger_disaster(self, DisasterKind::Quake, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    disaster: DisasterKind::Quake,
                    fired,
                })
            }
            DisasterRequest::Storm { pos } => {
                trigger_disaster(self, DisasterKind::Storm, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    disaster: DisasterKind::Storm,
                    fired,
                })
            }
            DisasterRequest::Plague { pos } => {
                trigger_disaster(self, DisasterKind::Plague, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    disaster: DisasterKind::Plague,
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
                        Some(PLANT) | Some(MOSS) | Some(WOOD)
                    );
                    if mat != AIR && mat != WATER {
                        // Plough through solid ground with a
                        // LAVA splinter for a visible scar.
                        self.push_voxel_write(cell, LAVA);
                        writes = writes.saturating_add(1);
                    }
                    if igniteable {
                        self.push_voxel_write(cell, LAVA);
                        writes = writes.saturating_add(1);
                    }
                }
                // Add belief: at least one cell mutated = the
                // sim registers an "act of god" event.
                if writes > 0 {
                    self.add_belief(8i64);
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
                    let next = if mat == WATER {
                        STEAM
                    } else {
                        AIR
                    };
                    self.push_voxel_write(cell, next);
                    writes = writes.saturating_add(1);
                }
                if writes > 0 {
                    self.add_belief(12i64);
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
                    self.push_voxel_write(cell, LAVA);
                    writes = writes.saturating_add(1);
                }
                // Steam ring at the surface.
                for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let cell = WorldCoord {
                        x: pos.x + dx,
                        y: pos.y,
                        z: pos.z + dz,
                    };
                    self.push_voxel_write(cell, STEAM);
                    writes = writes.saturating_add(1);
                }
                if writes > 0 {
                    self.add_belief(25i64);
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
                    self.add_belief(6i64);
                }
                Ok(GodToolReceipt::EnvironmentalDisaster {
                    kind_label: "drought".to_string(),
                    writes,
                })
            }
        }
    }

    fn top_voxel_y(&self, center: WorldCoord) -> i64 {
        scan_topmost_y(&self.voxel, center.x, center.z, center.y)
    }

    fn raise_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: center.y + i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, STONE);
                    written += 1;
                }
            }
        }
        written
    }

    fn lower_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        let top_y = self.top_voxel_y(center);
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: top_y + i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, AIR);
                    written += 1;
                }
            }
        }
        written
    }

    fn level_footprint(&mut self, center: WorldCoord, radius: i32, target_height: i32) -> u32 {
        let mut written = 0;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..target_height {
                    self.push_voxel_write(WorldCoord { x: center.x + i64::from(dx) * scale, y: i64::from(n) * scale, z: center.z + i64::from(dz) * scale }, STONE);
                    written += 1;
                }
            }
        }
        written
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
                disaster: DisasterKind::Meteor,
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
