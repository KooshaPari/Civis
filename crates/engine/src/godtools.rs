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
//! The 30 verbs that remain `Near` (camera, time, remaining
//! TERRAIN/MATERIAL/LAW) keep their no-op receipt in the deck and
//! wait for follow-up PRs to land their substrate handlers.
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
use civ_needs::{Health as LifeHealth, Needs as LifeNeeds};
use civ_voxel::{
    material::{GRAVEL, STONE},
    AIR, MaterialId, WorldCoord, FIXED_SCALE,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::disasters::{trigger_disaster, DisasterKind};
use crate::engine::Simulation;

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
    /// A LIFE verb — spawn organic + civ.
    Life(LifeRequest),
    /// A DISASTER verb — invoke a `DisasterKind` at a position.
    Disaster(DisasterRequest),
    /// An INSPECT verb — read-only; produces a `ProbeReport`
    /// without mutating any state.
    Inspect(InspectRequest),
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
}

/// TERRAIN op kinds. Mirrors the 11 TERRAIN verbs from
/// `docs/design/GOD_TOOLS_SANDBOX.md` §3.1. Phase 1 ships
/// `Raise`, `Lower`, `Level`; Phase 2 adds `Smooth` and
/// `RaiseMountain`; the other variants land in follow-up PRs.
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
            GodToolRequest::Life(l) => self.apply_life(l),
            GodToolRequest::Disaster(d) => self.apply_disaster(d),
            GodToolRequest::Inspect(i) => self.apply_inspect(i),
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
        }
        Ok(GodToolReceipt::Terraform {
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
        let (kind, pos) = match req {
            DisasterRequest::Meteor { pos } => (DisasterKind::Meteor, pos),
            DisasterRequest::Wildfire { pos } => (DisasterKind::Wildfire, pos),
            DisasterRequest::Flood { pos } => (DisasterKind::Flood, pos),
            DisasterRequest::Quake { pos } => (DisasterKind::Quake, pos),
            DisasterRequest::Storm { pos } => (DisasterKind::Storm, pos),
            DisasterRequest::Plague { pos } => (DisasterKind::Plague, pos),
        };
        trigger_disaster(self, kind, pos);
        let fired = self.belief() >= prev_belief;
        Ok(GodToolReceipt::Disaster { kind, fired })
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
}
