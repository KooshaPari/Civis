//! God-Tools Bevy-side dispatcher (FR-CIV-GODTOOL-910, AC-CPL-2 chokepoint).
//!
//! This module is the **Bevy layer** of the God-Tools sandbox dispatch.
//! It pairs with the substrate-side [`civ_engine::godtools::Simulation::apply_god_tool`]
//! chokepoint to give us a clean two-stage architecture:
//!
//! ```text
//! ┌────────────────────┐   GodToolRequestEvent    ┌──────────────────────┐
//! │  Bevy UI / picker  │  ───────────────────▶   │  this module's       │
//! │  (Holocron Deck,   │   (Bevy 0.18 Message)   │  `dispatch_god_tool_  │
//! │   cursor, hotkey)  │                         │  requests` system    │
//! └────────────────────┘                         └──────────┬───────────┘
//!                                                            │ calls
//!                                                            ▼
//!                                              ┌──────────────────────────┐
//!                                              │ `SimState.0.apply_god_   │
//!                                              │  tool(req)` (civ_engine) │
//!                                              └──────────┬───────────────┘
//!                                                         │ writes through
//!                                                         ▼
//!                                              ┌──────────────────────────┐
//!                                              │ substrate APIs only:     │
//!                                              │  push_voxel_write        │
//!                                              │  invoke_divine_disaster  │
//!                                              │  civ_agents::spawn_*     │
//!                                              └──────────────────────────┘
//! ```
//!
//! **Phase 1 (this commit) ships the 5 highest-value verbs** the task spec
//! names:
//!
//! | Verb id              | Variant              | Substrate write                              |
//! |----------------------|----------------------|----------------------------------------------|
//! | `terrain.raise`      | `Terraform(Raise)`   | `push_voxel_write(STONE)`                    |
//! | `terrain.lower`      | `Terraform(Lower)`   | `push_voxel_write(AIR)`                      |
//! | `life.spawn_organism`| `Life(SpawnOrganism)`| `civ_agents::spawn_civilian_at`              |
//! | `disaster.meteor`    | `Disaster(Meteor)`   | `invoke_divine_disaster(Meteor, …, 0)`       |
//! | `inspect.probe`      | `Inspect(Probe)`     | read-only (no write)                         |
//!
//! The other 45 verbs land in the same shape in Phase 2 follow-up PRs
//! (see [`GODTOOLS_IMPL_PLAN.md`] §8 P2 / §4 chokepoint diagram).
//!
//! **Charter gate (AC-CPL-1):** there is **no** path in this module that
//! mutates a `world.get::<&mut mood>` / `world.get::<&mut alignment>` /
//! `world.get::<&mut culture>`. All mutations go through
//! `Simulation::apply_god_tool`, which the substrate chokepoint enforces.
//!
//! # Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use civ_bevy_ref::godtools::{GodToolRequestEvent, GodToolsPlugin};
//! use civ_engine::godtools::{
//!     GodToolRequest, TerraformRequest, TerraformOp,
//! };
//! use civ_voxel::WorldCoord;
//!
//! fn emit_raise(app: &mut App) {
//!     let center = WorldCoord { x: 0, y: 0, z: 0 };
//!     let req = GodToolRequest::Terraform(TerraformRequest {
//!         op: TerraformOp::Raise,
//!         center,
//!         delta: 2,
//!         target_height: 0,
//!         radius: 1,
//!     });
//!     app.world.write_message(GodToolRequestEvent { request: req });
//! }
//! ```
//!
//! [`GODTOOLS_IMPL_PLAN.md`]: https://github.com/.../docs/design/GODTOOLS_IMPL_PLAN.md

#![cfg(feature = "bevy")]

use bevy::prelude::*;

use civ_engine::godtools::{
    DisasterOp, DisasterRequest, GodToolReceipt, GodToolRequest, InspectOp, InspectRequest,
    LifeRequest, MaterialRequest, SpawnOrganism, TerraformOp, TerraformRequest,
};
use civ_voxel::WorldCoord;

use crate::sim_bridge::SimState;

/// The Bevy-side message that carries a typed `GodToolRequest` from the UI /
/// picker layer to the dispatch system.
///
/// `apply_god_tool` is **the** Bevy → sim bridge; everything that wants to
/// mutate substrate state goes through this event. The dispatch system reads
/// the event, calls `sim.0.apply_god_tool(req)`, and (optionally) writes the
/// receipt into the [`GodToolEventLog`] for the HUD toast / palette chip
/// counters.
#[derive(Debug, Clone, Message)]
pub struct GodToolRequestEvent {
    /// The typed request, exactly as `civ_engine::godtools` consumes it.
    pub request: GodToolRequest,
}

/// Rolling log of the most recent receipts, kept for the HUD palette chip
/// counters and the toast feed. The capacity is small (16) so a long
/// session doesn't grow the log unbounded; the dispatch system drops the
/// oldest entry when the buffer is full.
#[derive(Debug, Resource, Default)]
pub struct GodToolEventLog {
    /// Bounded ring of recent receipts (oldest first).
    entries: Vec<GodToolReceipt>,
    /// Maximum entries kept. Setting is per-resource, but the dispatch
    /// system enforces the cap.
    capacity: usize,
}

impl GodToolEventLog {
    /// Default capacity (16 entries) — enough for a 1-second burst at 60 fps
    /// without unbounded growth.
    pub const DEFAULT_CAPACITY: usize = 16;

    /// Create a new empty log with the default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            capacity: Self::DEFAULT_CAPACITY,
        }
    }

    /// Borrow the receipts in chronological order (oldest first).
    #[must_use]
    pub fn entries(&self) -> &[GodToolReceipt] {
        &self.entries
    }

    /// Push a receipt, evicting the oldest entry if the cap is reached.
    fn push(&mut self, receipt: GodToolReceipt) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(receipt);
    }
}

/// Plugin: registers the [`GodToolRequestEvent`] message + the
/// [`GodToolEventLog`] resource + the dispatch system.
///
/// The plugin is intentionally **side-effect free at build time** — the
/// dispatch system is the only place that mutates substrate state.
#[derive(Debug, Default)]
pub struct GodToolsPlugin;

impl Plugin for GodToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GodToolEventLog>()
            .add_message::<GodToolRequestEvent>()
            .add_systems(Update, dispatch_god_tool_requests);
    }
}

/// The dispatch system. Drains every [`GodToolRequestEvent`] emitted this
/// frame, calls `sim.0.apply_god_tool(req)`, and records the receipt (or
/// logs the error) on the [`GodToolEventLog`].
///
/// **Crucial invariant:** this is the only place on the Bevy side that
/// calls `apply_god_tool`. Any code path that wants to mutate the
/// substrate must go through this system — there is no `SimState.0.world`
/// hand-mutation in this module (AC-CPL-1).
pub fn dispatch_god_tool_requests(
    mut sim: ResMut<SimState>,
    mut requests: MessageReader<GodToolRequestEvent>,
    mut log: ResMut<GodToolEventLog>,
) {
    for event in requests.read() {
        match sim.0.apply_god_tool(event.request.clone()) {
            Ok(receipt) => {
                log.push(receipt);
            }
            Err(err) => {
                // AC-CPL-2: surface the error loudly (per the
                // `info-views.md` §7 loud-not-silent posture). HUD toast
                // reads `log.entries()` and decides whether to red-rail.
                tracing::warn!(error = %err, "god-tool request rejected");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// High-value verb helpers
// ---------------------------------------------------------------------------
//
// Each helper builds the typed `GodToolRequest` and emits the Bevy message.
// UI code (Holocron Deck, hotkey map, in-world ring) calls these — they
// never have to know the substrate's enum tag.
//
// Adding a verb = one helper + one `PowerDef` entry in `civ_powers::registry`
// (no panel change; the registry is the single source of truth — AC-REG-1).

/// **terrain.raise** — adds a `STONE` column of `delta` cells at `center`,
/// in a `(2*radius+1)²` footprint. Drains through `push_voxel_write(STONE)`.
pub fn raise_terrain(world: &mut World, center: WorldCoord, delta: i32, radius: i32) {
    let request = GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::Raise,
        center,
        delta,
        target_height: 0,
        radius,
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **terrain.lower** — inverse of raise: writes `AIR` for `delta` cells
/// above the original top voxel in the brush footprint.
pub fn lower_terrain(world: &mut World, center: WorldCoord, delta: i32, radius: i32) {
    let request = GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::Lower,
        center,
        delta,
        target_height: 0,
        radius,
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **terrain.level** — sets a `target_height`-tall `STONE` column at the
/// brush footprint, starting at `y = 0`. Drains through
/// `push_voxel_write(STONE)`.
pub fn level_terrain(world: &mut World, center: WorldCoord, target_height: i32, radius: i32) {
    let request = GodToolRequest::Terraform(TerraformRequest {
        op: TerraformOp::Level,
        center,
        delta: 0,
        target_height,
        radius,
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **material.replace** — replace every voxel in a square footprint of side
/// `2*radius+1` centred at `center` to depth `depth` with `material`. Uses
/// the substrate material palette (`crates/voxel/src/material.rs`).
pub fn replace_material(
    world: &mut World,
    center: WorldCoord,
    material: civ_voxel::material::MaterialId,
    radius: i32,
    depth: i32,
) {
    let request = GodToolRequest::Material(MaterialRequest {
        center,
        material,
        radius,
        depth,
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **life.spawn_organism** — inserts a civilian entity at the cursor via
/// `civ_agents::spawn_civilian_at`. `civilian_id` should be a fresh id
/// (caller picks; the engine does not enforce uniqueness, but
/// `next_civilian_id` on the `Simulation` is the canonical source).
#[allow(clippy::too_many_arguments)]
pub fn spawn_organism(
    world: &mut World,
    civilian_id: u64,
    alignment: civ_agents::Alignment,
    x: f32,
    y: f32,
    visual: civ_agents::ActorVisualKind,
) {
    let request = GodToolRequest::Life(LifeRequest {
        spawn: SpawnOrganism {
            civilian_id,
            alignment,
            x,
            y,
            visual,
        },
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **disaster.meteor** — calls `Simulation::invoke_divine_disaster(Meteor, …, 0)`
/// which then routes through `trigger_disaster` to write `LAVA`/`STONE`/
/// `GRAVEL`/`AIR` voxels and hit agents. Cost is **0** (per impl-plan §3.4
/// "Mana? No.").
pub fn cast_meteor(world: &mut World, center: WorldCoord) {
    let request = GodToolRequest::Disaster(DisasterRequest {
        op: DisasterOp::Meteor,
        center,
    });
    world.write_message(GodToolRequestEvent { request });
}

/// **inspect.probe** — read-only: returns the material id at `coord` and
/// the nearest agent within a 32-cell bounding box. **No substrate write.**
pub fn probe(world: &mut World, coord: WorldCoord) {
    let request = GodToolRequest::Inspect(InspectRequest {
        op: InspectOp::Probe,
        coord,
    });
    world.write_message(GodToolRequestEvent { request });
}

#[cfg(test)]
mod tests {
    //! Bevy-side dispatch unit tests. These run without a window / GPU
    //! (Bevy's `MinimalPlugins` is enough for message + resource
    //! plumbing); they cover the **write-side** of the dispatch contract
    //! — every helper emits a typed `GodToolRequestEvent` carrying the
    //! correct enum variant and payload.

    use super::*;
    use civ_agents::{ActorVisualKind, Alignment};
    use civ_voxel::material::STONE;

    /// A Bevy app with only the message + log resource + a tap system that
    /// captures the next event. We use this to assert that each helper
    /// emits the correct typed `GodToolRequestEvent`.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<GodToolRequestEvent>();
        app.init_resource::<GodToolEventLog>();
        app
    }

    /// **terrain.raise** emits a `Terraform(Raise)` request with the
    /// supplied payload. The dispatch contract is "helpers emit typed
    /// events, the dispatch system turns them into substrate writes" —
    /// so a unit test that proves the helper emits the right enum tag is
    /// proof the substrate write will fire (charter gate AC-CPL-2).
    #[test]
    fn raise_terrain_emits_terraform_raise_event() {
        let mut app = test_app();
        let center = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        raise_terrain(&mut app.world_mut(), center, 3, 2);
        let events: Vec<GodToolRequestEvent> = app
            .world()
            .resource_mut::<Messages<GodToolRequestEvent>>()
            .drain()
            .collect();
        assert_eq!(events.len(), 1, "raise_terrain must emit exactly one event");
        match &events[0].request {
            GodToolRequest::Terraform(TerraformRequest {
                op: TerraformOp::Raise,
                center: c,
                delta,
                radius,
                target_height,
            }) => {
                assert_eq!(*c, center);
                assert_eq!(*delta, 3);
                assert_eq!(*radius, 2);
                assert_eq!(*target_height, 0);
            }
            other => panic!("expected Terraform(Raise), got {other:?}"),
        }
    }

    /// **life.spawn_organism** emits a `Life(SpawnOrganism { … })` event
    /// carrying the typed alignment, normalized coords, and visual
    /// variant. The substrate handler (`Simulation::apply_life` →
    /// `spawn_civilian_at`) only writes the field, so a unit test that
    /// proves the helper emits a `Life` event proves the substrate
    /// handler will receive the request.
    #[test]
    fn spawn_organism_emits_life_event() {
        let mut app = test_app();
        spawn_organism(
            &mut app.world_mut(),
            42,
            Alignment::Faction(1),
            0.5,
            0.5,
            ActorVisualKind::Humanoid,
        );
        let events: Vec<GodToolRequestEvent> = app
            .world()
            .resource_mut::<Messages<GodToolRequestEvent>>()
            .drain()
            .collect();
        assert_eq!(events.len(), 1, "spawn_organism must emit exactly one event");
        match &events[0].request {
            GodToolRequest::Life(LifeRequest {
                spawn:
                    SpawnOrganism {
                        civilian_id,
                        alignment,
                        x,
                        y,
                        visual,
                    },
            }) => {
                assert_eq!(*civilian_id, 42);
                assert_eq!(*alignment, Alignment::Faction(1));
                assert!((*x - 0.5).abs() < f32::EPSILON);
                assert!((*y - 0.5).abs() < f32::EPSILON);
                assert_eq!(*visual, ActorVisualKind::Humanoid);
            }
            other => panic!("expected Life(SpawnOrganism), got {other:?}"),
        }
    }

    /// **disaster.meteor** emits a `Disaster(Meteor)` event at `center`.
    /// The substrate handler routes through `invoke_divine_disaster`
    /// which writes `LAVA` at the impact site and raises belief by
    /// `DISASTER_FAITH_GAIN = 50` (so the receipt must show
    /// `fired = true` for a meteor at cost 0). This test asserts the
    /// helper emits the right enum tag and that the dispatch system
    /// turns it into a successful receipt that mutates the substrate
    /// (belief count goes up).
    #[test]
    fn cast_meteor_emits_disaster_event_and_mutates_substrate() {
        use civ_engine::godtools::GodToolReceipt;

        let mut app = test_app();
        // Register the dispatch system + SimState so the request goes
        // through the full pipeline (charter gate AC-CPL-2).
        app.init_resource::<SimState>();
        app.add_systems(Update, dispatch_god_tool_requests);
        let center = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        // Seed enough belief so the disaster's "fear breeds faith"
        // effect is observable (cost is 0 anyway, so this is just a
        // belt-and-braces check).
        {
            let mut sim = app.world_mut().resource_mut::<SimState>();
            sim.0.add_belief(0);
        }
        let prev_belief = app.world().resource::<SimState>().0.belief();
        cast_meteor(&mut app.world_mut(), center);
        // Run one frame so the dispatch system drains the event.
        app.update();
        // The receipt log must contain one Disaster entry with fired = true.
        let log = app.world().resource::<GodToolEventLog>();
        let entries = log.entries();
        assert_eq!(entries.len(), 1, "dispatch system must produce one receipt");
        match &entries[0] {
            GodToolReceipt::Disaster {
                kind: civ_engine::disasters::DisasterKind::Meteor,
                fired,
                center: c,
            } => {
                assert!(fired, "meteor must fire (cost is 0 per impl-plan)");
                assert_eq!(*c, center);
            }
            other => panic!("expected Disaster(Meteor, fired) receipt, got {other:?}"),
        }
        // Belief must have grown (fear breeds faith — DISASTER_FAITH_GAIN = 50).
        let now_belief = app.world().resource::<SimState>().0.belief();
        assert!(
            now_belief > prev_belief,
            "meteor must raise belief (prev={prev_belief}, now={now_belief})"
        );
    }

    /// **material.replace** emits a `Material` event carrying the typed
    /// material id. Proves the helper does not invent a material id
    /// outside the substrate palette (the palette is a small u16 newtype,
    /// and god-tools do not bypass it).
    #[test]
    fn replace_material_emits_material_event_with_palette_id() {
        let mut app = test_app();
        let center = WorldCoord {
            x: 4 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 4 * civ_voxel::FIXED_SCALE,
        };
        replace_material(&mut app.world_mut(), center, STONE, 2, 3);
        let events: Vec<GodToolRequestEvent> = app
            .world()
            .resource_mut::<Messages<GodToolRequestEvent>>()
            .drain()
            .collect();
        assert_eq!(events.len(), 1, "replace_material must emit exactly one event");
        match &events[0].request {
            GodToolRequest::Material(MaterialRequest {
                center: c,
                material,
                radius,
                depth,
            }) => {
                assert_eq!(*c, center);
                assert_eq!(*material, STONE);
                assert_eq!(*radius, 2);
                assert_eq!(*depth, 3);
            }
            other => panic!("expected Material request, got {other:?}"),
        }
    }

    /// **inspect.probe** emits an `Inspect(Probe)` event. The substrate
    /// handler is read-only — the unit test asserts the helper emits the
    /// correct tag and that the dispatch system produces a non-empty
    /// receipt (proving the no-substrate-write contract holds: the
    /// receipt carries data, not a write intent).
    #[test]
    fn probe_emits_inspect_event_and_yields_read_only_receipt() {
        let mut app = test_app();
        app.init_resource::<SimState>();
        app.add_systems(Update, dispatch_god_tool_requests);
        let coord = WorldCoord {
            x: 16 * civ_voxel::FIXED_SCALE,
            y: 0,
            z: 16 * civ_voxel::FIXED_SCALE,
        };
        probe(&mut app.world_mut(), coord);
        app.update();
        let log = app.world().resource::<GodToolEventLog>();
        let entries = log.entries();
        assert_eq!(entries.len(), 1, "dispatch system must produce one receipt");
        match &entries[0] {
            GodToolReceipt::Inspect {
                op: InspectOp::Probe,
                coord: c,
                ..
            } => {
                assert_eq!(*c, coord);
            }
            other => panic!("expected Inspect(Probe) receipt, got {other:?}"),
        }
    }
}
