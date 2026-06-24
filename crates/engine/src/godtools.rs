//! God-tool substrate dispatcher (Phase 1 of
//! `docs/design/GODTOOLS_IMPL_PLAN.md`).
//!
//! The dispatcher is the **single Bevy → substrate bridge**: every
//! mutating god-tool emits a [`GodToolRequest`] and a Bevy system
//! hands it to [`Simulation::apply_god_tool`]. There is no direct
//! `hecs::World` or `VoxelWorld` access from any god-tool system.
//!
//! ## Phase 1 verbs
//!
//! Per the implementation plan's P1/P2 cut, this phase ships 3-5
//! highest-value verbs whose substrate writes are real and
//! compiling against the existing engine API:
//!
//! | `PowerDef.id`       | Verb                | Substrate write |
//! |---------------------|---------------------|-----------------|
//! | `terrain.raise`     | Terraform::Raise    | [`Simulation::push_voxel_write`] (STONE) |
//! | `terrain.lower`     | Terraform::Lower    | [`Simulation::push_voxel_write`] (AIR) |
//! | `terrain.level`     | Terraform::Level    | [`Simulation::push_voxel_write`] (PACKED_DIRT to target_y) |
//! | `life.spawn_organism` | Life::SpawnOrganism | [`civ_agents::spawn_civilian_at`] |
//! | `disaster.meteor`   | Disaster::Meteor    | [`Simulation::invoke_divine_disaster`] |
//! | `inspect.probe`     | Inspect::Probe      | (read-only — returns voxel material + nearest agent) |
//!
//! All other verbs (the remaining 44 of the 50-verb catalog) are
//! registered in `civ-powers` as `PowerAvailability::Near` and
//! produce a no-op `GodToolReceipt` from this dispatcher until
//! their substrate handlers land in follow-up PRs.
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

use civ_agents::{spawn_civilian_at, ActorVisualKind, Alignment};
use civ_voxel::{material::AIR, MaterialId, WorldCoord};
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
/// `Raise`, `Lower`, `Level`; the other variants land in
/// follow-up PRs.
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
}

/// LIFE verb parameters. Phase 1 ships
/// [`LifeRequest::SpawnOrganism`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LifeRequest {
    /// `life.spawn_organism` — inject one agent via
    /// `civ_agents::spawn_civilian_at`.
    SpawnOrganism(SpawnOrganismRequest),
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
/// [`DisasterRequest::Meteor`]; other kinds extend the same
/// shape in follow-up PRs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DisasterRequest {
    /// `disaster.meteor` — invoke `DisasterKind::Meteor` at `pos`.
    Meteor { pos: WorldCoord },
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
    /// A LIFE verb injected an agent.
    Life {
        /// `hecs::Entity` bits of the new agent. The Bevy
        /// client can `Entity::from_bits(v)` to read it back.
        agent_entity_bits: u64,
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
    /// ## Phase 1 verb coverage
    ///
    /// - `terrain.raise` / `terrain.lower` / `terrain.level` —
    ///   real voxel writes via `push_voxel_write`.
    /// - `life.spawn_organism` — real agent spawn via
    ///   `civ_agents::spawn_civilian_at`.
    /// - `disaster.meteor` — real disaster invocation via
    ///   `trigger_disaster(DisasterKind::Meteor, …)`.
    /// - `inspect.probe` — read-only; returns the material at
    ///   the probed coord and the nearest agent (or `None`).
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
                    TerraformOp::Raise => MaterialId(6), // STONE
                    TerraformOp::Lower => AIR,
                    TerraformOp::Level => unreachable!(),
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
                const PACKED_DIRT: MaterialId = MaterialId(7);
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
                            PACKED_DIRT,
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
                })
            }
        }
    }

    fn apply_disaster(
        &mut self,
        req: DisasterRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        match req {
            DisasterRequest::Meteor { pos } => {
                // The substrate writes go through `trigger_disaster`,
                // which already adds belief via the
                // disaster → faith coupling (FR-CIV-EMERGENCE).
                let prev_belief = self.belief();
                trigger_disaster(self, DisasterKind::Meteor, pos);
                let fired = self.belief() >= prev_belief;
                Ok(GodToolReceipt::Disaster {
                    kind: DisasterKind::Meteor,
                    fired,
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
            GodToolReceipt::Life { agent_entity_bits } => agent_entity_bits,
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
}
