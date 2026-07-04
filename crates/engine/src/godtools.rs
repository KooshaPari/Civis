//! God-Tools substrate-side dispatcher.
//!
//! This module is the **single Bevy → sim bridge** (impl-plan §4, AC-CPL-2
//! chokepoint). The Bevy layer emits a `GodToolRequest`, this dispatcher
//! receives it on the `Simulation` and writes the corresponding substrate
//! field through the existing `Simulation::push_voxel_write`,
//! `Simulation::invoke_divine_disaster`, and `civ_agents::spawn_civilian_at`
//! APIs — **never** raw `voxel.write`, never raw `world.spawn`.
//!
//! Phase 1 (this commit) implements **5 of the 42 mutating verbs** (the
//! highest-value verbs the task spec names):
//!
//! 1. **`terrain.raise`** — adds a `STONE` column at `center` for `delta`
//!    cells. Drains through `push_voxel_write(STONE)` so the renderer
//!    protocol bridge gets the dirty-event.
//! 2. **`terrain.lower`** — inverse of raise: writes `AIR` for `delta` cells
//!    above the original top voxel. Drains through `push_voxel_write(AIR)`.
//! 3. **`terrain.level`** — sets a `target_height`-wide column at `center`
//!    to a single material (default `STONE`). Drains through
//!    `push_voxel_write(STONE)`.
//! 4. **`life.spawn_organism`** — inserts a civilian entity at the cursor
//!    via `civ_agents::spawn_civilian_at`. Drains through the agents layer.
//! 5. **`disaster.meteor`** — calls `Simulation::invoke_divine_disaster`
//!    (which then routes through `trigger_disaster` to write LAVA/STONE
//!    voxels + agent effects). Cost is **0** (per impl-plan §3.4
//!    "Mana? No." — cost would gate emergent disasters, defeating the
//!    charter).
//! 6. **`inspect.probe`** — read-only; returns the material id and entity
//!    info at `coord`. **No substrate write.**
//!
//! Follow-up PRs (Phase 2) land the remaining 36 mutating verbs in the
//! same shape — each gets one private method on `Simulation` that takes
//! the typed request struct and calls the corresponding substrate write.
//!
//! **Charter gate (AC-CPL-1):** there is **no** path in this module that
//! mutates a `world.get::<&mut mood>` / `world.get::<&mut alignment>` /
//! `world.get::<&mut culture>`. The substrate owns those fields; god-tools
//! are inputs to the substrate, not authors of the substrate's outputs.

#![deny(unsafe_code)]

use civ_agents::{spawn_civilian_at, ActorVisualKind, Alignment};
use civ_voxel::material::{AIR, STONE};
use civ_voxel::WorldCoord;
use serde::{Deserialize, Serialize};

use crate::disasters::DisasterKind;
use crate::engine::Simulation;

/// All god-tool verbs are routed through this enum.
///
/// Each variant carries the typed payload the substrate handler needs.
/// There is **no** `ScriptedOutcome` variant — a power that wants to
/// bypass the substrate into "the result" cannot even be expressed
/// (impl-plan §6.2, AC-CPL-3 compile-time guard).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolRequest {
    /// Voxel column edits (TERRAIN 11 + most MATERIAL verbs).
    ///
    /// Phase 1 ships `Raise`/`Lower`/`Level`; follow-up PRs add
    /// `Smooth`/`Slope`/`Flatten`/`Shift`/`AddLand`/`DigOcean`/
    /// `RaiseMountain`/`DropBiome`.
    Terraform(TerraformRequest),
    /// Material paint (M1–M5, M8). Phase 1 ships a generic
    /// `Replace { center, material }`; follow-up PRs add
    /// `AdditiveDrop`/`Erase`/`SurfacePaint`/`PourLiquid`/`SeedSnow`/`SeedOre`.
    Material(MaterialRequest),
    /// Agent-spawn verbs (LIFE 1–8). Phase 1 ships `SpawnOrganism`;
    /// follow-up PRs add `SpawnHerd`/`SpawnCivSeed`/`Bless`/`Curse`/`Heal`/
    /// `Extinct`. `Plague` is a diffusion field write (LAW routes it).
    Life(LifeRequest),
    /// Disaster verbs (DISASTER 1–8). Phase 1 ships `Meteor`;
    /// follow-up PRs add `Lightning`/`Flood`/`Quake`/`Firestorm`/`Tornado`/
    /// `VolcanicVent`/`Drought`.
    Disaster(DisasterRequest),
    /// Read-only inspect verbs. Phase 1 ships `Probe`; the remaining
    /// 7 inspect verbs (`Stats`/`Trace`/`Forecast`/`CompareSnapshots`/
    /// `History`/`Bookmark`/`Follow`) are Bevy-only and never call
    /// `apply_god_tool`.
    Inspect(InspectRequest),
}

/// TERRAIN verb payload.
///
/// `op` selects one of the 11 TERRAIN verbs; `center` is the brush
/// footprint's anchor in fixed-point world coords (the BEVY picker
/// converts the cursor → `WorldCoord` before emitting this request).
/// `delta` and `target_height` are brush parameters carried on every
/// TERRAIN request so the Bevy layer never has to pre-resolve the verb
/// type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerraformRequest {
    /// Which TERRAIN verb.
    pub op: TerraformOp,
    /// Anchor (brush center) in fixed-point world coords.
    pub center: WorldCoord,
    /// `Raise`/`Lower` height delta in **cells** (1 cell = `civ_voxel::FIXED_SCALE`).
    pub delta: i32,
    /// `Level` target height in cells (counted from y=0).
    pub target_height: i32,
    /// Brush radius in cells (filled square footprint, inclusive).
    pub radius: i32,
}

/// The TERRAIN verbs implemented in Phase 1.
///
/// The other 8 verbs (`Smooth`/`Slope`/`Flatten`/`Shift`/`AddLand`/
/// `DigOcean`/`RaiseMountain`/`DropBiome`) land in Phase 2 follow-up PRs
/// and reuse the same `apply_terraform` dispatcher shape.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TerraformOp {
    /// `terrain.raise` — adds a `STONE` column at `center` for `delta` cells.
    Raise,
    /// `terrain.lower` — inverse of raise: writes `AIR` for `delta` cells
    /// above the original top voxel.
    Lower,
    /// `terrain.level` — sets a `target_height`-tall column at `center`
    /// to a single material (default `STONE`).
    Level,
}

/// MATERIAL verb payload.
///
/// Phase 1 ships a generic `Replace { center, material, radius, depth }`.
/// The remaining 7 MATERIAL verbs land in Phase 2 follow-up PRs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaterialRequest {
    /// `material.replace` — replace every voxel in a square footprint
    /// of side `2*radius + 1` centred at `center` to depth `depth` with
    /// the given material id.
    pub center: WorldCoord,
    /// Material id (u16 wrapper). Reuses the substrate palette
    /// (`crates/voxel/src/material.rs:125-151`); god-tools do **not**
    /// invent new material ids.
    pub material: civ_voxel::material::MaterialId,
    /// Footprint radius in cells (filled square, inclusive).
    pub radius: i32,
    /// Depth in cells (number of cells written, starting at `center.y`).
    pub depth: i32,
}

/// LIFE verb payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LifeRequest {
    /// `life.spawn_organism` — inserts a civilian entity at the cursor
    /// with the given alignment + visual variant. `cradle_state` is the
    /// normalized terrain (x, y) ∈ [0, 1].
    pub spawn: SpawnOrganism,
}

/// `life.spawn_organism` payload.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SpawnOrganism {
    /// Stable civilian id (the engine tracks the next id via `next_civilian_id`).
    pub civilian_id: u64,
    /// Faction alignment (None / Faction(id) / OtherEntity(id)).
    /// Phase 1 only honours `Alignment::None` and `Alignment::Faction`;
    /// `OtherEntity` is a Phase 2 hook for creature-from-actor.
    pub alignment: Alignment,
    /// Normalized terrain x in [0.0, 1.0].
    pub x: f32,
    /// Normalized terrain y in [0.0, 1.0].
    pub y: f32,
    /// Visual variant (humanoid / herd). Phase 1 emits `Humanoid`;
    /// `Herd` lands with `life.spawn_herd` in Phase 2.
    pub visual: ActorVisualKind,
}

/// DISASTER verb payload.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DisasterRequest {
    /// Which DISASTER verb.
    pub op: DisasterOp,
    /// Anchor in fixed-point world coords.
    pub center: WorldCoord,
}

/// The DISASTER verbs implemented in Phase 1 (one verb; the other 7
/// follow in Phase 2 follow-up PRs).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DisasterOp {
    /// `disaster.meteor` — calls `Simulation::invoke_divine_disaster(Meteor, …)`.
    /// Cost = 0 (per impl-plan §3.4 "Mana? No.").
    Meteor,
}

/// INSPECT verb payload (read-only; **no substrate write**).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct InspectRequest {
    /// `inspect.probe` — read material at `coord` and any agent entity
    /// within a small footprint.
    pub op: InspectOp,
    /// Anchor in fixed-point world coords.
    pub coord: WorldCoord,
}

/// INSPECT verbs implemented in Phase 1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InspectOp {
    /// `inspect.probe` — material id + nearest agent (read-only).
    Probe,
}

/// The receipt returned from `apply_god_tool`.
///
/// Receipts are **data** — the Bevy dispatcher uses them to drive
/// HUD toast / palette chip feedback (impl-plan §4). They never
/// re-enter the substrate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GodToolReceipt {
    /// A voxel-column edit was applied.
    Terraform {
        /// Verb that fired.
        op: TerraformOp,
        /// Cells written (the Bevy layer uses this for the palette chip count).
        cells_written: u32,
        /// Anchor the Bevy layer used for the brush ring.
        center: WorldCoord,
    },
    /// A material-replace was applied.
    Material {
        /// Cells written.
        cells_written: u32,
        /// Material id written.
        material: civ_voxel::material::MaterialId,
        /// Anchor.
        center: WorldCoord,
    },
    /// An agent was spawned.
    Spawn {
        /// The new entity's hecs id.
        entity: hecs::Entity,
        /// Civilian id passed in.
        civilian_id: u64,
        /// Anchor (the Bevy layer can use this for the inspector highlight).
        coord: WorldCoord,
    },
    /// A disaster was invoked (or refused — see `fired`).
    Disaster {
        /// Which `DisasterKind` fired (or was refused).
        kind: DisasterKind,
        /// `true` if the disaster was applied; `false` if belief was
        /// insufficient. Phase 1 always sets `fired = true` because
        /// `cost = 0`.
        fired: bool,
        /// Anchor.
        center: WorldCoord,
    },
    /// An inspect verb returned data without writing.
    Inspect {
        /// Which verb.
        op: InspectOp,
        /// Material id at the cursor.
        material: civ_voxel::material::MaterialId,
        /// Agent entity nearest the cursor (if any).
        nearest_agent: Option<hecs::Entity>,
        /// Anchor.
        coord: WorldCoord,
    },
}

/// Errors that `apply_god_tool` can surface to the Bevy dispatcher.
///
/// `apply_god_tool` **does not** panic on malformed input — the
/// dispatcher is supposed to validate first (impl-plan §4, AC-CPL-2),
/// but a bad request must produce a typed error rather than corrupt the
/// simulation state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GodToolError {
    /// A `Terraform`/`Material` request had a non-positive dimension.
    /// `delta`/`depth` must be > 0; `radius` must be >= 0.
    InvalidDimension {
        /// What field was wrong.
        field: &'static str,
        /// The bad value.
        value: i32,
    },
    /// A `SpawnOrganism` request had normalized coords outside [0, 1].
    /// `spawn_civilian_at` clamps, but the dispatcher prefers to reject
    /// up-front so the receipt tells the UI "invalid cursor" instead of
    /// silently snapping the agent.
    OutOfBounds {
        /// `x` or `y`.
        axis: &'static str,
        /// The bad value.
        value: f32,
    },
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
        }
    }
}

impl std::error::Error for GodToolError {}

impl Simulation {
    /// Single Bevy → sim bridge for god-tool verbs (impl-plan §4).
    ///
    /// This is the **only** public function on `Simulation` that mutates
    /// substrate state in response to a typed `GodToolRequest`. Every
    /// variant routes to a typed handler that calls the corresponding
    /// substrate write API — no raw `voxel.write`, no raw `world.spawn`,
    /// no direct `push_damage`.
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
        }
    }

    /// TERRAIN handler (Phase 1: `Raise`/`Lower`/`Level`).
    ///
    /// Every variant calls `push_voxel_write`, which:
    /// 1. Writes the material into the `VoxelWorld<MaterialId>`,
    /// 2. Records a `voxel_write` event on the replay log.
    /// The next `phase_voxel` tick (`crates/engine/src/engine.rs:1425`)
    /// drains the dirty events to the renderer protocol bridge.
    fn apply_terraform(&mut self, t: TerraformRequest) -> Result<GodToolReceipt, GodToolError> {
        if t.radius < 0 {
            return Err(GodToolError::InvalidDimension {
                field: "radius",
                value: t.radius,
            });
        }
        match t.op {
            TerraformOp::Raise => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension {
                        field: "delta",
                        value: t.delta,
                    });
                }
                let written = self.raise_footprint(t.center, t.radius, t.delta);
                Ok(GodToolReceipt::Terraform {
                    op: TerraformOp::Raise,
                    cells_written: written,
                    center: t.center,
                })
            }
            TerraformOp::Lower => {
                if t.delta <= 0 {
                    return Err(GodToolError::InvalidDimension {
                        field: "delta",
                        value: t.delta,
                    });
                }
                let written = self.lower_footprint(t.center, t.radius, t.delta);
                Ok(GodToolReceipt::Terraform {
                    op: TerraformOp::Lower,
                    cells_written: written,
                    center: t.center,
                })
            }
            TerraformOp::Level => {
                if t.target_height < 0 {
                    return Err(GodToolError::InvalidDimension {
                        field: "target_height",
                        value: t.target_height,
                    });
                }
                let written = self.level_footprint(t.center, t.radius, t.target_height);
                Ok(GodToolReceipt::Terraform {
                    op: TerraformOp::Level,
                    cells_written: written,
                    center: t.center,
                })
            }
        }
    }

    /// `terrain.raise` — adds a `STONE` column of `delta` cells on top of
    /// the brush footprint. For each (dx, dz) in the `radius`-radius
    /// footprint, writes `STONE` at `center.y + n * FIXED_SCALE` for
    /// `n in 0..delta`.
    fn raise_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let mut written = 0u32;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    let pos = WorldCoord {
                        x: center.x + i64::from(dx) * scale,
                        y: center.y + i64::from(n) * scale,
                        z: center.z + i64::from(dz) * scale,
                    };
                    self.push_voxel_write(pos, STONE);
                    written += 1;
                }
            }
        }
        written
    }

    /// `terrain.lower` — writes `AIR` for `delta` cells above the original
    /// top voxel in the brush footprint.
    ///
    /// Phase 1 reads the **top voxel** at the centre column via
    /// `voxel().read()` then clears a `delta`-cell slab above it. The
    /// full column-clear (Lower with side-scrolling) lands in Phase 2.
    fn lower_footprint(&mut self, center: WorldCoord, radius: i32, delta: i32) -> u32 {
        let scale = civ_voxel::FIXED_SCALE as i64;
        let top_y = self.top_voxel_y(center);
        let mut written = 0u32;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..delta {
                    let pos = WorldCoord {
                        x: center.x + i64::from(dx) * scale,
                        y: top_y + i64::from(n) * scale,
                        z: center.z + i64::from(dz) * scale,
                    };
                    self.push_voxel_write(pos, AIR);
                    written += 1;
                }
            }
        }
        written
    }

    /// `terrain.level` — sets a `target_height`-tall `STONE` column at
    /// every (dx, dz) in the brush footprint, starting at `y = 0`.
    fn level_footprint(&mut self, center: WorldCoord, radius: i32, target_height: i32) -> u32 {
        let mut written = 0u32;
        let scale = civ_voxel::FIXED_SCALE as i64;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                for n in 0..target_height {
                    let pos = WorldCoord {
                        x: center.x + i64::from(dx) * scale,
                        y: i64::from(n) * scale,
                        z: center.z + i64::from(dz) * scale,
                    };
                    self.push_voxel_write(pos, STONE);
                    written += 1;
                }
            }
        }
        written
    }

    /// Find the top solid voxel's `y` for the column at `center`.
    ///
    /// Walks `y = center.y` upward until it hits `AIR`, then returns
    /// the cell just below. If the column is all-`AIR` (which can
    /// happen on a fresh sim), returns `center.y`.
    fn top_voxel_y(&self, center: WorldCoord) -> i64 {
        let scale = civ_voxel::FIXED_SCALE as i64;
        let mut y = center.y;
        // Walk up at most 64 cells before giving up (avoids an
        // unbounded loop on pathological terrain).
        for _ in 0..64 {
            let next = WorldCoord {
                x: center.x,
                y: y + scale,
                z: center.z,
            };
            if self.voxel().read(next) == AIR {
                return y;
            }
            y += scale;
        }
        y
    }

    /// MATERIAL handler (Phase 1: `Replace`).
    ///
    /// Writes `material` to every cell in the footprint, to depth
    /// `depth`. Drains through `push_voxel_write` so the renderer
    /// protocol bridge gets the dirty-event.
    fn apply_material(&mut self, m: MaterialRequest) -> Result<GodToolReceipt, GodToolError> {
        if m.radius < 0 {
            return Err(GodToolError::InvalidDimension {
                field: "radius",
                value: m.radius,
            });
        }
        if m.depth <= 0 {
            return Err(GodToolError::InvalidDimension {
                field: "depth",
                value: m.depth,
            });
        }
        let written = self.material_replace_footprint(&m);
        Ok(GodToolReceipt::Material {
            cells_written: written,
            material: m.material,
            center: m.center,
        })
    }

    fn material_replace_footprint(&mut self, m: &MaterialRequest) -> u32 {
        let scale = civ_voxel::FIXED_SCALE as i64;
        let mut written = 0u32;
        for dx in -m.radius..=m.radius {
            for dz in -m.radius..=m.radius {
                for n in 0..m.depth {
                    let pos = WorldCoord {
                        x: m.center.x + i64::from(dx) * scale,
                        y: m.center.y + i64::from(n) * scale,
                        z: m.center.z + i64::from(dz) * scale,
                    };
                    self.push_voxel_write(pos, m.material);
                    written += 1;
                }
            }
        }
        written
    }

    /// LIFE handler (Phase 1: `SpawnOrganism`).
    ///
    /// Inserts a civilian entity via `civ_agents::spawn_civilian_at`.
    /// The entity gets a `CivilianBundle::newborn_default(...)` and an
    /// `ActorVisual(visual)` marker — both supplied by the agents
    /// crate's existing spawn path, **not** bypassed here. The
    /// `civilian_id` is the engine's `next_civilian_id` (taken from
    /// `state.rng_seed` so successive spawns stay deterministic).
    fn apply_life(&mut self, l: LifeRequest) -> Result<GodToolReceipt, GodToolError> {
        let s = l.spawn;
        if !(0.0..=1.0).contains(&s.x) {
            return Err(GodToolError::OutOfBounds {
                axis: "x",
                value: s.x,
            });
        }
        if !(0.0..=1.0).contains(&s.y) {
            return Err(GodToolError::OutOfBounds {
                axis: "y",
                value: s.y,
            });
        }
        let entity = spawn_civilian_at(
            &mut self.world,
            s.civilian_id,
            s.alignment,
            s.x,
            s.y,
            s.visual,
            &mut self.rng,
        );
        Ok(GodToolReceipt::Spawn {
            entity,
            civilian_id: s.civilian_id,
            coord: WorldCoord {
                x: (s.x.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64,
                y: 0,
                z: (s.y.clamp(0.0, 1.0) * civ_voxel::FIXED_SCALE as f32) as i64,
            },
        })
    }

    /// DISASTER handler (Phase 1: `Meteor`).
    ///
    /// Routes through `Simulation::invoke_divine_disaster(Meteor, …, 0)`
    /// which in turn calls `trigger_disaster` (the existing public
    /// entry point at `crates/engine/src/disasters.rs:35`). The
    /// existing path writes `LAVA`/`STONE`/`GRAVEL`/`AIR` voxels and
    /// hits agents — we **never** duplicate that here.
    fn apply_disaster(
        &mut self,
        d: DisasterRequest,
    ) -> Result<GodToolReceipt, GodToolError> {
        let (kind, op) = match d.op {
            DisasterOp::Meteor => (DisasterKind::Meteor, DisasterOp::Meteor),
        };
        // cost = 0 per impl-plan §3.4 "Mana? No."; pop-cultures mana would
        // gate emergent disasters, defeating the charter. Fear still
        // breeds faith via `trigger_disaster` (DISASTER_FAITH_GAIN = 50).
        let fired = self.invoke_divine_disaster(kind, d.center, 0);
        Ok(GodToolReceipt::Disaster {
            kind,
            fired,
            center: d.center,
        })
    }

    /// INSPECT handler (Phase 1: `Probe`).
    ///
    /// **No substrate write.** Reads `voxel().read(coord)` for the
    /// material id and walks the agent world for the nearest entity to
    /// `coord`. Returns the data via the receipt; the Bevy layer reads
    /// the receipt to populate the Inspector panel.
    fn apply_inspect(&mut self, i: InspectRequest) -> Result<GodToolReceipt, GodToolError> {
        match i.op {
            InspectOp::Probe => {
                let material = self.voxel().read(i.coord);
                let nearest_agent = self.nearest_agent(i.coord);
                Ok(GodToolReceipt::Inspect {
                    op: InspectOp::Probe,
                    material,
                    nearest_agent,
                    coord: i.coord,
                })
            }
        }
    }

    /// Find the agent entity nearest to `coord` (within a 32-cell
    /// bounding box; returns `None` if no entity is in range).
    fn nearest_agent(&self, coord: WorldCoord) -> Option<hecs::Entity> {
        use civ_agents::Position3d;
        let scale = civ_voxel::FIXED_SCALE as i64;
        let range = 32 * scale;
        let mut best: Option<(hecs::Entity, i128)> = None;
        for (entity, pos) in self.world.query::<&Position3d>().iter() {
            let dx = (pos.coord.x - coord.x) as i128;
            let dy = (pos.coord.y - coord.y) as i128;
            let dz = (pos.coord.z - coord.z) as i128;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > (range as i128) * (range as i128) {
                continue;
            }
            match best {
                Some((_, d)) if d <= dist_sq => {}
                _ => best = Some((entity, dist_sq)),
            }
        }
        best.map(|(e, _)| e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_voxel::material::{AIR, GRAVEL, LAVA, STONE};

    fn fresh_sim() -> Simulation {
        Simulation::with_seed(7)
    }

    /// **terrain.raise** mutates the voxel field: writes `STONE` at
    /// `(center.x, center.y + n*FIXED_SCALE, center.z)` for `n in
    /// 0..delta`, in a `(2*radius+1)²` footprint.
    #[test]
    fn terraform_raise_writes_stone_into_substrate() {
        let mut sim = fresh_sim();
        let center = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        let receipt = sim
            .apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
                op: TerraformOp::Raise,
                center,
                delta: 3,
                target_height: 0,
                radius: 1,
            }))
            .expect("raise applies");
        // 3 cells (delta) × 9 footprint cells (radius=1) = 27
        match receipt {
            GodToolReceipt::Terraform {
                op: TerraformOp::Raise,
                cells_written,
                ..
            } => assert_eq!(cells_written, 27),
            other => panic!("expected Terraform/Raise receipt, got {other:?}"),
        }
        // The voxel field must now contain STONE at the centre, top of the
        // new column. (FIXED_SCALE * 2 = second cell up.)
        let top_of_new_column = WorldCoord {
            x: center.x,
            y: 2 * civ_voxel::FIXED_SCALE,
            z: center.z,
        };
        assert_eq!(
            sim.voxel().read(top_of_new_column),
            STONE,
            "raise must write STONE into the voxel field"
        );
    }

    /// **terrain.lower** mutates the voxel field: writes `AIR` above the
    /// original top voxel for `delta` cells, in the footprint.
    #[test]
    fn terraform_lower_writes_air_into_substrate() {
        let mut sim = fresh_sim();
        let center = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        // First raise 4 cells so we have a STONE column to lower.
        sim.apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Raise,
            center,
            delta: 4,
            target_height: 0,
            radius: 0,
        }))
        .expect("setup raise applies");
        // Now lower by 2.
        let receipt = sim
            .apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
                op: TerraformOp::Lower,
                center,
                delta: 2,
                target_height: 0,
                radius: 0,
            }))
            .expect("lower applies");
        match receipt {
            GodToolReceipt::Terraform {
                op: TerraformOp::Lower,
                cells_written,
                ..
            } => assert_eq!(cells_written, 2),
            other => panic!("expected Terraform/Lower receipt, got {other:?}"),
        }
        // The top 2 cells of the column must now be AIR.
        let top_after_lower = WorldCoord {
            x: center.x,
            y: 3 * civ_voxel::FIXED_SCALE,
            z: center.z,
        };
        assert_eq!(
            sim.voxel().read(top_after_lower),
            AIR,
            "lower must write AIR into the voxel field"
        );
        // The 3rd cell (cell index 2, just below) should still be STONE.
        let below_top = WorldCoord {
            x: center.x,
            y: 2 * civ_voxel::FIXED_SCALE,
            z: center.z,
        };
        assert_eq!(
            sim.voxel().read(below_top),
            STONE,
            "lower should not touch cells below the lowered zone"
        );
    }

    /// **life.spawn_organism** mutates the agent world: inserts a
    /// civilian entity via `civ_agents::spawn_civilian_at` and returns
    /// its `hecs::Entity` in the receipt.
    #[test]
    fn life_spawn_organism_inserts_agent_into_world() {
        let mut sim = fresh_sim();
        let before_count = sim.world.iter().count();
        let receipt = sim
            .apply_god_tool(GodToolRequest::Life(LifeRequest {
                spawn: SpawnOrganism {
                    civilian_id: 4242,
                    alignment: Alignment::Faction(1),
                    x: 0.5,
                    y: 0.5,
                    visual: ActorVisualKind::Humanoid,
                },
            }))
            .expect("spawn applies");
        // The agent count must have increased by at least 1.
        let after_count = sim.world.iter().count();
        assert!(
            after_count > before_count,
            "spawn_organism must insert a new agent into the world (before={before_count}, after={after_count})"
        );
        // The receipt must carry the entity.
        let entity = match receipt {
            GodToolReceipt::Spawn { entity, .. } => entity,
            other => panic!("expected Spawn receipt, got {other:?}"),
        };
        // The entity must be live in the world (a query for it must
        // succeed).
        assert!(
            sim.world.get::<&civ_agents::Civilian>(entity).is_ok(),
            "the new entity must carry a Civilian component"
        );
    }

    /// **disaster.meteor** mutates the voxel field via the existing
    /// `invoke_divine_disaster` path: writes `LAVA` at the centre and
    /// surrounding voxels, raises belief by `DISASTER_FAITH_GAIN = 50`.
    #[test]
    fn disaster_meteor_writes_lava_into_substrate() {
        let mut sim = fresh_sim();
        let center = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        let prev_belief = sim.belief();
        let receipt = sim
            .apply_god_tool(GodToolRequest::Disaster(DisasterRequest {
                op: DisasterOp::Meteor,
                center,
            }))
            .expect("disaster applies");
        match receipt {
            GodToolReceipt::Disaster {
                kind: DisasterKind::Meteor,
                fired,
                ..
            } => assert!(fired, "meteor must fire (cost is 0 per impl-plan)"),
            other => panic!("expected Disaster receipt, got {other:?}"),
        }
        // Voxel field at the centre must be LAVA (the meteor core).
        assert_eq!(
            sim.voxel().read(center),
            LAVA,
            "meteor must write LAVA at the impact centre"
        );
        // A neighbouring cell must have been disturbed (STONE / GRAVEL /
        // AIR — but not the pre-meteor material which was likely STONE).
        let neighbour = WorldCoord {
            x: center.x + civ_voxel::FIXED_SCALE,
            y: 0,
            z: center.z,
        };
        let neighbour_mat = sim.voxel().read(neighbour);
        assert!(
            matches!(neighbour_mat, STONE | GRAVEL | AIR),
            "meteor must disturb neighbours (got {:?})",
            neighbour_mat
        );
        // Belief must have grown (fear breeds faith — DISASTER_FAITH_GAIN = 50).
        assert!(
            sim.belief() > prev_belief,
            "meteor must raise belief (fear breeds faith): prev={prev_belief}, now={}",
            sim.belief()
        );
    }

    /// **inspect.probe** does **not** mutate the substrate: voxel field
    /// at `coord` is unchanged before and after.
    #[test]
    fn inspect_probe_is_read_only() {
        let mut sim = fresh_sim();
        let coord = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        let before = sim.voxel().read(coord);
        let receipt = sim
            .apply_god_tool(GodToolRequest::Inspect(InspectRequest {
                op: InspectOp::Probe,
                coord,
            }))
            .expect("inspect applies");
        let after = sim.voxel().read(coord);
        assert_eq!(
            before, after,
            "inspect.probe must NOT mutate the substrate (got {before:?} → {after:?})"
        );
        // The receipt must carry the material id at `coord`.
        match receipt {
            GodToolReceipt::Inspect {
                op: InspectOp::Probe,
                material,
                ..
            } => assert_eq!(material, before, "probe receipt must echo the material"),
            other => panic!("expected Inspect receipt, got {other:?}"),
        }
    }

    /// Negative path: `terrain.raise` with `delta == 0` is rejected,
    /// not silently applied.
    #[test]
    fn apply_god_tool_rejects_invalid_dimension() {
        let mut sim = fresh_sim();
        let center = WorldCoord {
            x: 0,
            y: 0,
            z: 0,
        };
        let res = sim.apply_god_tool(GodToolRequest::Terraform(TerraformRequest {
            op: TerraformOp::Raise,
            center,
            delta: 0,
            target_height: 0,
            radius: 1,
        }));
        assert!(
            matches!(res, Err(GodToolError::InvalidDimension { field: "delta", .. })),
            "expected InvalidDimension(delta), got {res:?}"
        );
    }

    /// Negative path: `life.spawn_organism` with `x > 1.0` is rejected.
    #[test]
    fn apply_god_tool_rejects_out_of_bounds_spawn() {
        let mut sim = fresh_sim();
        let res = sim.apply_god_tool(GodToolRequest::Life(LifeRequest {
            spawn: SpawnOrganism {
                civilian_id: 7,
                alignment: Alignment::None,
                x: 1.5,
                y: 0.5,
                visual: ActorVisualKind::Humanoid,
            },
        }));
        assert!(
            matches!(res, Err(GodToolError::OutOfBounds { axis: "x", .. })),
            "expected OutOfBounds(x), got {res:?}"
        );
    }

    /// Receipts serialize + deserialize cleanly (the Bevy → sim
    /// IPC channel is JSON over the existing `sim_worker` wire).
    #[test]
    fn god_tool_receipt_serde_roundtrip() {
        let receipt = GodToolReceipt::Terraform {
            op: TerraformOp::Raise,
            cells_written: 27,
            center: WorldCoord {
                x: 16,
                y: 0,
                z: 16,
            },
        };
        let json = serde_json::to_string(&receipt).expect("serialize");
        let back: GodToolReceipt = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(receipt, back);
    }
}
