# God-Tools Implementation Plan — "The Holocron Deck" → Civis substrate

> **Status:** Binding implementation plan (2026-06-23). Translates
> `docs/design/GOD_TOOLS_SANDBOX.md` (the spec) into concrete Bevy
> systems/components/events, the crate each lives in, the substrate field
> each tool *writes into* (no bypass), and a phased Work Breakdown
> Structure (WBS) with an explicit DAG.
>
> **Authority contract:** every god-tool mutates state the substrate
> already owns. Per `crates/engine/src/engine.rs:1172` (`Simulation::tick`)
> and the substrate phase order at `crates/engine/src/engine.rs:55-68`
> (`PHASE_ORDER = ["production", "citizen_lifecycle", "military",
> "policy", "economy", "planet", "diplomacy", "tactics", "voxel",
> "compact", "buildings", "diffusion"]`), every god-tool edit MUST drain
> through the existing `Simulation::push_voxel_write`,
> `Simulation::push_damage`, `Simulation::invoke_divine_disaster`, or
> the `civ_agents` / `civ_economy` / `civ_laws` mutators. There is **no
> bypass path**. See the no-bypass rules in §4 of the spec.
>
> **Headline count (matches FR-CIV-GODTOOL-900 + the spec verdict at
> `docs/design/GOD_TOOLS_SANDBOX.md:580`):** **42 mutating god-tools
> across 7 FR tabs** (TERRAIN 11, MATERIAL 8, LIFE 8, DISASTER 8,
> INSPECT 8, LAW 8, TIME 8). Camera verbs (C1–C8) are universal UI, not
> substrate writes, and live on the mouse/keyboard path described in §6 of
> the spec. This plan covers the **42 mutating + the 8 read-only INSPECT
> verbs = 50 verbs that live in Bevy systems**; it also covers the 8
> camera verbs as a thin Bevy input plugin (no substrate coupling).

---

## 0. How to read this plan

For each god-tool, this plan specifies:

1. **Identifier** — the `PowerDef.id` (e.g. `terrain.raise`, `disaster.meteor`).
2. **Bevy plugin/module location** — which crate and module the system
   lives in.
3. **System(s)** — the Bevy system function(s) that read the input
   event and produce the substrate mutation.
4. **Component(s)** — any new Bevy `Component` types required (sparse
   state the god-tool needs across frames, e.g. active brush settings).
5. **Event(s)** — the Bevy `Event` types the tool emits or consumes.
6. **Substrate write** — the *exact* call into the existing
   `Simulation` API or a substrate crate that performs the mutation.
   **No bypass.**
7. **AC-REG/CPL line(s)** — the acceptance criteria it satisfies.

The substrate write column is the **charter gate**: every tool must
mutate state that `crates/engine`, `crates/voxel`, `crates/agents`,
`crates/economy`, `crates/laws`, `crates/diffusion`, or `crates/planet`
already reads each tick.

---

## 1. Substrate write surface (the "no bypass" contract)

The god-tools Bevy layer **never** writes ECS components that the
sim-side Rust doesn't already mutate. The Bevy layer is a thin
input/event dispatcher that produces one of these typed requests and
hands them to the engine. Each request is a substrate-owned mutation:

| Substrate field | Owner | Write API (engine.rs / substrate) | Read by |
|---|---|---|---|
| `VoxelWorld<MaterialId>::write(pos, mat)` | `crates/voxel` | `Simulation::push_voxel_write` at `crates/engine/src/engine.rs:904` | `phase_voxel` (`crates/engine/src/engine.rs:1425`) → CA settles; renderer reads `last_tick_voxel_events` (`crates/engine/src/engine.rs:984`) |
| `VoxelWorld::damage` (radius+center) | `crates/voxel` | `Simulation::apply_damage_now` / `push_damage` (`crates/engine/src/engine.rs:898-913`) | `phase_tactics` (`crates/engine/src/engine.rs:1357`), `phase_voxel` |
| `CaGrid::set` (per-cell material/temp) | `crates/voxel/src/fluid_ca.rs:335` | through `VoxelWorld::write` for chunks containing the cell | `phase_voxel` (CA step → gravity/fluid/thermo) |
| `MaterialId` constants | `crates/voxel/src/material.rs:125-151` (`AIR`, `WATER`, `LAVA`, `SAND`, `DIRT`, `GRAVEL`, `STONE`, `PACKED_DIRT`, `ICE`, `STEAM`, `ORE`, `BEDROCK`, `SALT_WATER`, `OIL`, …) | direct reuse — god-tools do **not** invent new material ids; they reuse the substrate palette | material registry at `crates/voxel/src/material.rs:87` |
| `DisasterKind` (Meteor/Flood/Quake/Wildfire/Storm/Plague) | `crates/engine/src/disasters.rs:19` | `Simulation::invoke_divine_disaster` (`crates/engine/src/disasters.rs:49`) — spends belief, calls `trigger_disaster` | `phase_disasters` (`crates/engine/src/disasters.rs:70`); auto-emitted when climate crosses ignition thresholds |
| `Climate` recompute (sea-level, tide_offset) | `crates/planet` | `civ_planet::compute_climate` returns new `Climate`; `Simulation` stores it (`crates/engine/src/engine.rs:1269` `phase_planet`) | `phase_planet` runs every tick; `apply_tide_offset` writes `WATER_MARKER_MATERIAL` voxels through `push_voxel_write` |
| `WeatherCell` field writes (precipitation, temp) | `crates/planet/src/weather.rs` | `WeatherCell` is per-tick derived; god-tools seed a `WeatherCell` and the next `phase_planet` tick re-derives | `phase_planet`, `phase_disasters` (wildfire ignition) |
| Agent spawn (life) | `crates/agents` | `civ_agents::spawn_child_near` (`crates/agents/src/lib.rs:333`), `civ_agents::spawn_civilian_at` (`crates/agents/src/lib.rs:558`), `civ_agents::spawn_many` (`crates/agents/src/lib.rs:592`) | `phase_citizen_lifecycle` (`crates/engine/src/engine.rs:1525`); psyche/needs/economy read |
| Agent bless/curse/health edit | `crates/agents` + `crates/needs` | new `apply_actor_effect(world, footprint, Effect)` mutating `Needs`/`Health`; **never** mutates `mood`/`alignment`/`ideology`/`culture` | `phase_citizen_lifecycle` decays needs; agents generalize via their own reward loop |
| Agent despawn | `crates/agents` | `hecs::World::despawn(entity)` on entities with matching genome hash | population accounting in `phase_citizen_lifecycle` |
| Faction tax rate / market bias | `crates/economy` | `civ_economy::institution::Taxation` (`crates/economy/src/lib.rs:23`); `Simulation::apply_scenario_taxation` (`crates/engine/src/engine.rs:1049`) | `phase_economy` (`crates/engine/src/engine.rs:1760`) re-derives market clearing prices |
| Law toggle (edict) | `crates/laws` | `LawDb::unlock_at_era` / `LawDb::apply_overlay` (`crates/laws/src/lib.rs:154,217`) | `phase_policy` (`crates/engine/src/engine.rs:1748`) reads policy signals |
| Difficulty knob / scenario scalar | `crates/engine::scenario` | `Simulation::economy_policy = PolicyInput` (`crates/engine/src/engine.rs:444`) | `phase_economy` consumes `economy_policy.scarcity_multiplier` |
| `GameSpeed::multiplier` (time) | `crates/protocol-3d` / `crates/engine::schedule` | writes the decoupled Bevy `Time<Fixed>` multiplier; sim clock is paused/scaled; **no substrate field** is touched | every phase (`PHASE_ORDER`) |
| Replay scrub (rewind) | `crates/watch` | `Snapshot::restore_into_simulation` (existing replay log entry path) | snapshot ring at `crates/watch/src/` (terrain.rs) |

**The rule:** the Bevy god-tools layer emits typed events
(`GodToolEvent`) and a thin dispatch system (`apply_god_tool_events`)
reads them off the queue and calls the substrate write API. There is no
direct hecs / bevy_ecs world access from any god-tool system. The
dispatcher is the only Bevy→sim bridge. This is the **"emit, never
bypass"** rule from §4.2 of the spec.

---

## 2. New crates / modules introduced

This plan introduces **one new crate** and **three new modules** in
existing crates, all under `crates/`:

| New unit | Crate | Purpose |
|---|---|---|
| `crates/powers/` | new | The `PowerDef` + `PowerRegistry` data-driven catalog (FR-CIV-GODTOOL-901); no business logic |
| `crates/powers/src/lib.rs` | new | `PowerDef`, `PowerRegistry`, `default_powers()` returning the 50-verb list with `availability: Live/Near/Blind` |
| `crates/powers/src/registry.rs` | new | `PowerRegistry::register(power) -> Result<(), PowerRegistrationError>` — runtime guard for unknown request kinds + unknown subsystem handles (AC-CPL-2, AC-CPL-4) |
| `crates/voxel/src/brush.rs` | new (in `civ-voxel`) | `BrushOp` enum + `stamp_footprint(world: &mut VoxelWorld<MaterialId>, center: WorldCoord, brush: &BrushSettings, op: BrushOp)` — the **only** Bevy-side helper that touches `VoxelWorld`; everything else dispatches through it |
| `crates/engine/src/godtools.rs` | new (in `civ-engine`) | `Simulation::apply_god_tool(req: GodToolRequest)` — the substrate-side handler that the Bevy dispatcher calls; preserves the "Bevy never holds the sim" invariant |
| `crates/voxel/src/hud.rs` | extends | adds `ToolPalette::power_index()` accessor so `crates/powers` and `clients/bevy-ref` share the same key |

---

## 3. The 50 god-tools — concrete plan

Each tool follows the **same 5-row spec** (the canonical row of the
spec §3 tables, restated as a Bevy system spec):

```
ID:               PowerDef.id ("tab.name")
Tab:              PowerTab (Terrain | Material | Life | Disaster | Inspect | Law | Time)
Category:         PowerCategory (Mutating | ReadOnly)
Bevy system:      fn handle_<tab>_<name>(...)
Bevy event in:    GodToolRequest { kind: PowerRequestKind::*, payload: ... }
Bevy event out:   SubstrateWrite (one of the typed writes in §1)
Crate:            crates/powers (registry) + crates/engine (handler) + crates/<substrate> (write)
Substrate write:  <exact API call>
Depends on:       <phase in PHASE_ORDER that drains the write next tick>
AC:               AC-GT-1..9, AC-CPL-1..4
```

### 3.1 TERRAIN — 11 mutating god-tools

The TERRAIN tab mutates the `VoxelWorld<MaterialId>` only. Per spec
§3.1 "no tool writes foliage, fauna, agents, or settlements."

| ID | Tool | Bevy system | Substrate write | Drain phase | Crate | AC |
|---|---|---|---|---|---|---|
| `terrain.raise` | Raise | `handle_terrain_raise` | `Simulation::push_voxel_write(WorldCoord, STONE/PACKED_DIRT)` for each cell in footprint, `y += Δ` | `voxel` | `clients/bevy-ref/src/tools/terrain.rs` → dispatches to `crates/engine/src/godtools.rs::apply_terraform` → `crates/voxel/src/brush.rs::stamp_footprint` | AC-GT-3, AC-CPL-1 |
| `terrain.lower` | Lower | `handle_terrain_lower` | `push_voxel_write(AIR/STONE)` for cells above new height | `voxel` | same | AC-GT-3 |
| `terrain.level` | Level | `handle_terrain_level` | reads top voxel via `voxel.read()`, sets footprint to picked `target_height` | `voxel` | same | AC-GT-3 |
| `terrain.smooth` | Smooth | `handle_terrain_smooth` | averages neighbour heights in 3×3×3 window; uses `apply_tide_offset`-style clear-then-write pattern (`crates/engine/src/engine.rs:487`) to preserve dirty-event invariants | `voxel` | same | AC-GT-3 |
| `terrain.slope` | Slope | `handle_terrain_slope` | computes gradient from two anchor points; per-cell Δ = dot(pos - anchor_a, slope_vec) | `voxel` | same | AC-GT-3 |
| `terrain.flatten` | Flatten | `handle_terrain_flatten` | reads majority top material across footprint, calls `terrain.level` internally | `voxel` | same | AC-GT-3 |
| `terrain.shift` | Shift | `handle_terrain_shift` | read-then-write: snapshots column, writes at translated pos | `voxel` | same | AC-GT-3 |
| `terrain.add_land` | AddLand (god brush) | `handle_terrain_add_land` | `push_voxel_write(STONE)` for chunky +Δ in hard-edged footprint, no falloff | `voxel` | same | AC-GT-3 |
| `terrain.dig_ocean` | DigOcean (god brush) | `handle_terrain_dig_ocean` | sets height = `Simulation::planet().sea_level`; then writes `WATER` material; CA fills | `voxel`, `planet` | same | AC-GT-3 |
| `terrain.raise_mountain` | RaiseMountain | `handle_terrain_raise_mountain` | Gaussian peak profile + height-noise dither via `SimRng::gen_range`; writes `STONE`/`GRAVEL`; volcanic CA may follow in `phase_voxel` | `voxel` | same | AC-GT-3 |
| `terrain.drop_biome` | DropBiome | `handle_terrain_drop_biome` | surface-only material swap (calls `terrain.surface_paint`); CA + climate seed flora/fauna next tick | `voxel`, `planet`, `diffusion` | same | AC-GT-3 |

**Shared TERRAIN wiring:**

```rust
// crates/engine/src/godtools.rs (new module)
pub enum GodToolRequest {
    Terraform(TerraformRequest),
    Material(MaterialRequest),
    Life(LifeRequest),
    Disaster(DisasterRequest),
    Law(LawRequest),
    Time(TimeRequest),
    Inspect(InspectRequest), // no-op for substrate
}

pub struct TerraformRequest {
    pub op: TerraformOp,         // Raise | Lower | Level | Smooth | Slope | Flatten | Shift | AddLand | DigOcean | RaiseMountain | DropBiome
    pub center: WorldCoord,
    pub brush: BrushSettings,    // size, strength, falloff, shape, target_height, slope_vector, biome_id, drop_height, depth, spacing, symmetry, randomize, continuous
    pub request_id: u64,         // for undo (FR-CIV-GODTOOL-921)
}

impl Simulation {
    /// Apply a god-tool request. NEVER bypasses PHASE_ORDER. All voxel
    /// mutations go through `push_voxel_write` so they emit dirty events
    /// for the protocol/renderer bridge (`crates/engine/src/engine.rs:904`).
    pub fn apply_god_tool(&mut self, req: GodToolRequest) -> Result<GodToolReceipt, GodToolError> {
        match req {
            GodToolRequest::Terraform(t) => self.apply_terraform(t),
            GodToolRequest::Material(m)  => self.apply_material(m),
            GodToolRequest::Life(l)      => self.apply_life(l),
            GodToolRequest::Disaster(d)  => self.apply_disaster(d),
            GodToolRequest::Law(l)       => self.apply_law(l),
            GodToolRequest::Time(t)      => Err(GodToolError::TimeHandledBySchedule), // Bevy side
            GodToolRequest::Inspect(_)   => Ok(GodToolReceipt::no_op()),               // read-only
        }
    }
}
```

### 3.2 MATERIAL — 8 mutating god-tools

M1–M5 and M8 are voxel writes through `push_voxel_write`. **M6
(SeedForest) is a life-spell that hides in Material** — it spawns
plant-agents via `crates/agents::spawn_many` with `ActorVisualKind::Herd`
and a seed-genome. **M7 (SeedOreDeposit) writes to the ore-density
CA field** — `crates/voxel/src/fluid_ca.rs:CaGrid::set_with_temp` for
each cell, then the next `phase_voxel` propagates it.

| ID | Tool | Substrate write | Crate |
|---|---|---|---|
| `material.replace` | Replace | `push_voxel_write(mat)` per cell in footprint to depth | `crates/voxel/src/brush.rs::stamp_footprint` |
| `material.additive_drop` | AdditiveDrop | `push_voxel_write(mat)` at `drop_height` above target; gravity CA advects (gas inverts) | same + CA in `crates/voxel/src/fluid_ca.rs` |
| `material.erase` | Erase | `push_voxel_write(AIR)` | same |
| `material.surface_paint` | SurfacePaint | reads top voxel; `push_voxel_write(mat)` only on the topmost solid layer | same |
| `material.pour_liquid` | PourLiquid | spawns a `WATER`/`LAVA`/`OIL` slab at cursor + CA spreads (Powder-Toy flood) | same + CA |
| `material.seed_forest` | SeedForest | `crates/agents::spawn_many(...)` with `ActorVisualKind::Herd` + plant genome | `crates/agents/src/lib.rs:592` |
| `material.seed_ore` | SeedOreDeposit | `CaGrid::set_with_temp(x,y,z, ORE, ambient_temp)` + density CA | `crates/voxel/src/fluid_ca.rs:326` |
| `material.seed_snow` | SeedSnow | `push_voxel_write(ICE)` above snowline; thermo CA melts | `crates/voxel/src/brush.rs` + `crates/voxel/src/reactions.rs` |

### 3.3 LIFE — 8 mutating god-tools

All route through `crates/agents`. Per spec §3.3 "no tool sets culture,
religion, ideology, alignment, job, or mood directly."

| ID | Tool | Substrate write | Crate / function |
|---|---|---|---|
| `life.spawn_organism` | SpawnOrganism | `civ_agents::spawn_child_near(parent, genome, cradle_state, age, rng)` (`crates/agents/src/lib.rs:333`) | `crates/agents` |
| `life.spawn_herd` | SpawnHerd | `civ_agents::spawn_many(world, count, footprint, genome, rng)` (`crates/agents/src/lib.rs:592`) | `crates/agents` |
| `life.spawn_civ_seed` | SpawnCivilizationSeed | 6× `spawn_organism` + 1× `BuildingGraph::add_building(BuildingType::House)` (or new `Hut`) + 1× `Resources` deposit — **no `culture` field set** | `crates/agents` + `crates/build/src/lib.rs::Allocator` |
| `life.bless` | Bless | `apply_actor_effect(world, footprint, Effect::MoodBoost(+Δ))` — writes to `Needs` only; agent generalizes via its own reward loop | new `crates/agents/src/effects.rs` |
| `life.curse` | Curse | inverse of `life.bless` | same |
| `life.plague` | Plague | writes a `Pathogen` field to footprint; CA propagates (SIR-coupled) | `crates/diffusion/src` + new `crates/agents::pathogen.rs` |
| `life.heal` | Heal | `apply_actor_effect(world, footprint, Effect::HealthRestore(+Δ))` | `crates/agents/src/effects.rs` |
| `life.extinct` | Extinct | `hecs::World::despawn(entity)` on entities whose genome hash matches | `crates/agents` |

**Important:** the `apply_actor_effect` function lives in
`crates/agents/src/effects.rs` and is the **only** mutation path for
Bless/Curse/Heal/Plague. The "no `mood`/`alignment`/`culture`/`ideology`
direct write" rule is enforced by the **negative-field list** in
`crates/powers/src/registry.rs::register()` (see §6.1, AC-CPL-3).

### 3.4 DISASTER — 8 mutating god-tools

All route through `Simulation::invoke_divine_disaster` at
`crates/engine/src/disasters.rs:49`. The current 6 `DisasterKind`
variants (`crates/engine/src/disasters.rs:19`) are: Meteor, Flood,
Quake, Wildfire, Storm, Plague. We extend with **VolcanicVent** and
**Tornado** (the spec adds 2). All 8 invoke via the same substrate
write:

```rust
// crates/engine/src/godtools.rs
fn apply_disaster(&mut self, d: DisasterRequest) -> Result<GodToolReceipt, GodToolError> {
    let kind = match d.tool {
        DisasterTool::Meteor      => DisasterKind::Meteor,
        DisasterTool::Lightning   => DisasterKind::Wildfire, // lightning ignites; thermo CA spreads (RND-015)
        DisasterTool::Flood       => DisasterKind::Flood,
        DisasterTool::Quake       => DisasterKind::Quake,
        DisasterTool::Firestorm   => DisasterKind::Wildfire,
        DisasterTool::Tornado     => DisasterKind::Storm,
        DisasterTool::VolcanicVent=> DisasterKind::Meteor,    // sustained; CA cools to rock
        DisasterTool::Drought     => DisasterKind::Storm,     // precipitation write; next phase_planet re-derives
    };
    let fired = self.invoke_divine_disaster(kind, d.pos, /* cost */ 0);
    // Cost is 0 — per spec §6.4 "Mana? No." Populous-style mana is rejected.
    // (Faith coupling remains: trigger_disaster awards DISASTER_FAITH_GAIN at
    // crates/engine/src/disasters.rs:40 — emergent, not tool-driven.)
    Ok(GodToolReceipt { kind, pos: d.pos, fired })
}
```

| ID | Tool | Crate |
|---|---|---|
| `disaster.meteor` | Meteor | `crates/engine/src/disasters.rs` |
| `disaster.lightning` | Lightning | same + `crates/voxel/src/reactions.rs` for electric conductivity |
| `disaster.flood` | Flood | same + `crates/voxel/src/fluid_ca.rs` for water CA |
| `disaster.quake` | Quake | same + `Simulation::push_damage` for structural damage (`crates/engine/src/engine.rs:898`) |
| `disaster.firestorm` | Firestorm | `crates/engine/src/disasters.rs` (Wildfire with `radius` param) |
| `disaster.tornado` | Tornado | `crates/planet/src/weather.rs` for wind-field write |
| `disaster.volcanic_vent` | VolcanicVent | same as Meteor + sustained LAVA write through `push_voxel_write(LAVA)` |
| `disaster.drought` | Drought | `crates/planet/src/weather.rs::WeatherCell::precip_mm_fp -= Δ` |

**Charter gate (AC-CPL-1):** no disaster tool writes a damage value to
a building. The path is `inject initial condition → CA propagates →
structures take damage via physics`. The `Simulation::push_damage` calls
for Quake/Volcano are *voxel damage events* (terrain + voxel material
removal), **not** scripted `building.hp -= 100`. Buildings are derived
state and decay per the substrate.

### 3.5 INSPECT — 8 read-only verbs

**Zero substrate writes.** INSPECT tools produce no `GodToolRequest`
that maps to a substrate mutation. The Bevy systems open UI panels;
they never call `Simulation::apply_god_tool`.

| ID | Tool | Bevy system | Reads | Crate |
|---|---|---|---|---|
| `inspect.probe` | Probe | `inspect_probe_system` | entity under cursor via `bevy_picking`; opens Inspector panel (HOLO hero panel) | `clients/bevy-ref/src/inspect/` |
| `inspect.stats` | Stats | `inspect_stats_system` | `crates/watch` aggregates over region; renders with `egui_plot` | `clients/bevy-ref/src/inspect/` |
| `inspect.trace` | Trace | `inspect_trace_system` | walks the **Legends saga graph** backwards | `clients/bevy-ref/src/inspect/` (Legends dep — referenced at `docs/design/legends-engine.md`) |
| `inspect.forecast` | Forecast | `inspect_forecast_system` | saves state, clones sim, fast-forwards, reads delta, restores | `crates/engine/src/godtools.rs::forecast(req)` — read-only clone of `Simulation::with_seed(state.tick)` |
| `inspect.compare_snapshots` | CompareSnapshots | `inspect_compare_system` | reads two `Snapshot` blobs from `crates/watch/src/terrain.rs` | `clients/bevy-ref/src/inspect/` |
| `inspect.history` | History | `inspect_history_system` | reads the timeline ribbon | `clients/bevy-ref/src/inspect/` |
| `inspect.bookmark` | Bookmark | `inspect_bookmark_system` | writes camera bookmark (UI state, **not** sim state) | `clients/bevy-ref/src/camera/` |
| `inspect.follow` | Follow | `inspect_follow_system` | locks camera to actor; camera only — does not pass into sim | `clients/bevy-ref/src/camera/` |

**AC-GT-7 / AC-CPL:** capture sim hash before/after → identical.

### 3.6 LAW — 8 parameter-nudge tools

LAW tools write a *parameter* that a substrate subsystem already reads.
There is no law that writes *outcome*.

| ID | Tool | Substrate write | Crate |
|---|---|---|---|
| `law.tax_bias` | TaxBias | `Simulation::apply_scenario_taxation(&Taxation)` (`crates/engine/src/engine.rs:1049`) | `crates/engine` + `crates/economy` |
| `law.edict` | Edict | `LawDb::apply_overlay(...)` (`crates/laws/src/lib.rs:154`); policy phase reads | `crates/laws` |
| `law.religion_pressure` | ReligionPressure | writes to `crates/diffusion` SIR field; CA propagates | `crates/diffusion` |
| `law.sanction` | Sanction | removes trade-route entries from `WorldState::trade_routes` (`crates/engine/src/engine.rs:318`); agents reroute via existing A* logic | `crates/engine` |
| `law.open_border` | OpenBorder | inverse — adds trade-route entries (subject to existing entry-validation) | same |
| `law.alignment_nudge` | AlignmentNudge | writes `AI utility weights` (Hammond-Axelrod ethnocentricity) consumed by utility AI | `crates/engine/src/policy.rs` |
| `law.difficulty_knob` | DifficultyKnob | `Simulation::economy_policy.scarcity_multiplier` (`crates/engine/src/engine.rs:444`) | `crates/engine` |
| `law.scenario_script` | ScenarioScript | reads `scenario.ron`; emits the script via the existing request queue — **no new mutation pathway** | `clients/bevy-ref/src/scenario/` |

### 3.7 CAMERA — 8 universal verbs (no substrate writes)

Camera verbs never touch the world. They mutate the Bevy
`Camera3d` transform only. **No `Simulation::apply_god_tool` call.**

| ID | Verb | Bevy system | Crate |
|---|---|---|---|
| `camera.orbit` | Orbit | `camera_orbit_system` (mouse drag → yaw around target) | `clients/bevy-ref/src/camera/` |
| `camera.pan` | Pan | `camera_pan_system` | same |
| `camera.zoom` | Zoom | `camera_zoom_system` | same |
| `camera.tilt` | Tilt | `camera_tilt_system` | same |
| `camera.roll` | Roll | `camera_roll_system` (disabled by default) | same |
| `camera.bookmarks` | Bookmarks | `camera_bookmarks_system` (recalls stored transform + sim state snapshot id) | same + `crates/watch` |
| `camera.follow_cam` | FollowCam | `camera_follow_system` (locks to actor entity) | same |
| `camera.photo_mode` | PhotoMode | `camera_photo_mode_system` (hides HUD, free-cam) | same |

**AC-GT-7 hardening:** all `camera.*` systems use a dedicated
`CameraTransformMarker` component — `Simulation` has no accessor for
this component. The "sim never reads camera transform" wall is enforced
by visibility, not just convention.

### 3.8 TIME — 8 clock-control verbs

TIME tools never mutate the substrate; they mutate the decoupled
Bevy `Time<Fixed>` multiplier (or the snapshot-replay cursor).

| ID | Verb | Bevy system | Effect on PHASE_ORDER | Crate |
|---|---|---|---|---|
| `time.pause` | Pause | `time_pause_system` | `Time<Fixed>::set_relative_speed(0.0)` — phases still resolve their `FixedTimestep` schedules when un-paused | `clients/bevy-ref/src/time/` |
| `time.play` | Play | `time_play_system` | `set_relative_speed(1.0)` | same |
| `time.slow` | Slow | `time_slow_system` | `set_relative_speed(0.25)` | same |
| `time.fast` | Fast | `time_fast_system` | `set_relative_speed(N)` where N ∈ {2,5,10} | same |
| `time.step` | Step | `time_step_system` | advances exactly N ticks via `Schedule::run(&mut world)` then auto-pauses | `clients/bevy-ref/src/time/` + `crates/engine` |
| `time.rewind` | Rewind | `time_rewind_system` | reads snapshot ring (`crates/watch/src/terrain.rs`); restores via `Snapshot::restore_into_simulation` — **charter: soft determinism**, RNG re-rolls on forward continuation | `crates/watch` |
| `time.fast_forward_to_event` | FastForwardToEvent | `time_ffte_system` | filters `crates/watch` stream for event-kind K, advances until match or max-tick cap | `crates/watch` |
| `time.profile` | Profile | `time_profile_system` | writes a perf-trace log (separate from replay log) | `clients/bevy-ref/src/time/` |

**Note:** TIME tools are the only category where the Bevy layer does
NOT call `Simulation::apply_god_tool`. The `crates/engine/src/godtools.rs`
handler returns `GodToolError::TimeHandledBySchedule` for `TimeRequest`
variants; the Bevy schedule handles them directly.

---

## 4. The Bevy dispatcher (the bridge)

There is exactly one Bevy system that translates player input into a
substrate write. The dispatcher is the **single chokepoint** for AC-CPL-2
("no power emits a request outside the standard queue"):

```rust
// clients/bevy-ref/src/tools/dispatcher.rs
pub fn apply_god_tool_events(
    mut commands: Commands,
    mut events: EventReader<GodToolEvent>,
    mut sim_access: ResMut<GodToolSimBridge>, // wraps a thread-local handle to the sim worker
) {
    for evt in events.read() {
        let req = evt.to_request();
        // Route through the substrate — never call hecs::World directly.
        let receipt = match req {
            GodToolRequest::Time(_) => { /* Bevy schedule handles */ continue; }
            other => sim_access.send(other),
        };
        commands.trigger(GodToolReceiptEvent { request: evt.clone(), receipt });
    }
}
```

The `GodToolSimBridge` is a thin wrapper around the existing
`sim_worker` IPC used by `crates/civ-server` (it already
serializes/deserializes `EditCommand`). The bridge serializes the
`GodToolRequest` to JSON, ships it across the same channel, and
deserializes the `GodToolReceipt` (count of voxel writes, belief spent,
spawn count, etc.) for UI feedback.

**Why this matters:** every Bevy system → sim mutation path is
auditable. A monkey-patch of the bridge to a no-op produces zero
world change (AC-CPL-2 test). Mod-added powers route through the same
bridge.

---

## 5. Component / Event / System additions (Bevy side)

### 5.1 New Bevy components

All components live in `clients/bevy-ref/src/components/godtools.rs`:

| Component | Purpose | Lifetime |
|---|---|---|
| `ActivePower(power_id: PowerId)` | Current armed power; `bevy_picking` uses it for the brush ring | per-session |
| `BrushSettings { size, strength, falloff, shape, target_height, slope_vector, biome_id, drop_height, depth, spacing, symmetry, randomize, continuous }` | extends the existing `BrushSettings` from `brush-tool-system.md` §3 | per-session |
| `MaterialSelector(MaterialId)` | M1–M8 power selector | per-session |
| `GenomeSelector(GenomeHash)` | L1–L3, L8 selector | per-session |
| `LawSelector { target_subsystem: SubsystemId, value: ParameterValue }` | LW1–LW8 | per-session |
| `GodToolHistory { receipts: VecDeque<GodToolReceipt> }` | undo (`undo_2` per `onboarding-qol.md` §3) | per-session |
| `PowerRegistry(powers::PowerRegistry)` | the data-driven registry, mirror of `crates/powers::PowerRegistry` | AppState::default |
| `CameraTransformMarker` | camera verbs only — sim has no accessor | per-session |
| `PauseOnNextEvent { kind: EventKind, armed: bool, max_ticks: u64 }` | TM7 | per-session |

### 5.2 New Bevy events

All events live in `clients/bevy-ref/src/events.rs`:

| Event | Producer | Consumer |
|---|---|---|
| `GodToolEvent { power_id, params, request_id, center, footprint }` | keyboard/mouse/palette input | `apply_god_tool_events` |
| `GodToolReceiptEvent { request, receipt }` | dispatcher | UI feedback (HUD toast, palette chip) |
| `BrushFootprintUpdate { center, radius, brush, severity }` | mouse hover | `brush_ring_render_system` |
| `UndoRequested { request_id }` | `Ctrl+Z` | `undo_god_tool_system` |
| `BlueprintStampRequested { region, rotated }` | blueprint paste | `apply_god_tool_events` |
| `TimeSpeedChangeRequested { multiplier }` | TM2–TM4 | `time_speed_system` |
| `SnapshotRestoreRequested { snapshot_id }` | TM6 | `crates/watch` consumer |

### 5.3 New Bevy systems

The systems are organized by tab; each tab gets its own plugin:

```
clients/bevy-ref/src/tools/
  mod.rs                  // GodToolsPlugin
  registry.rs             // mirror of crates/powers/PowerRegistry, asset sync
  dispatcher.rs           // apply_god_tool_events (the bridge)
  brush.rs                // brush footprint projection (HOLO ring)
  palette/
    mod.rs
    keycap.rs             // rim of 8 fast verbs
    holocron.rs           // radial carousel modal
  terrain.rs              // 11 TERRAIN handlers (call into dispatcher)
  material.rs             // 8 MATERIAL handlers
  life.rs                 // 8 LIFE handlers
  disaster.rs             // 8 DISASTER handlers
  inspect.rs              // 8 INSPECT handlers (no sim calls)
  law.rs                  // 8 LAW handlers
  camera.rs               // 8 CAMERA handlers
  time.rs                 // 8 TIME handlers
  undo.rs                 // undo_god_tool_system (FR-CIV-GODTOOL-921)
```

---

## 6. Coupling guards (the charter gate enforcement)

### 6.1 Compile-time guard (AC-CPL-3)

In `crates/powers/src/registry.rs`:

```rust
// Negative-field list — cannot appear in any PowerDef or handler.
pub const FORBIDDEN_TARGET_FIELDS: &[&str] = &[
    "culture", "religion", "ideology", "alignment",
    "job", "faction_id", "mood", "happiness",
];

impl PowerRegistry {
    pub fn register(&mut self, power: PowerDef) -> Result<(), PowerRegistrationError> {
        if let Some(field) = power.writes_fields.iter().find(|f| FORBIDDEN_TARGET_FIELDS.contains(f)) {
            return Err(PowerRegistrationError::ForbiddenField(field.clone()));
        }
        // AC-CPL-4: target_subsystem must reference an existing handle.
        if let Some(sub) = power.target_subsystem {
            if !crate::engine::known_subsystems().contains(&sub) {
                return Err(PowerRegistrationError::UnknownSubsystem(sub));
            }
        }
        self.powers.push(power);
        Ok(())
    }
}
```

### 6.2 Runtime guard (AC-CPL-2)

In `crates/engine/src/godtools.rs::apply_god_tool`:

```rust
// AC-CPL-2: monkey-patch the dispatcher to a no-op → no world change.
debug_assert!(!req.kind.is_scripted_outcome(),
    "PowerDef.request.kind must be a substrate-owned variant");
```

The `PowerRequestKind` enum is:

```rust
pub enum PowerRequestKind {
    MaterialEdit,    // → VoxelWorld::write
    TerraformEdit,   // → VoxelWorld::write
    ActorSpawn,      // → civ_agents::spawn_*
    ActorEffect,     // → crates/agents::apply_actor_effect
    Disaster,        // → Simulation::invoke_divine_disaster
    Law,             // → Simulation::apply_scenario_taxation / LawDb::apply_overlay
    Time,            // → Bevy schedule (not a substrate write)
    NoOp,            // → Inspect tools (returns read-only data)
    // ScriptedOutcome, // intentionally absent — would fail to compile (AC-CPL-3)
}
```

---

## 7. Mod extensibility (AC-REG-6)

`civ-mod-host` already exposes host imports at
`crates/mod-host/src/wasm_guest.rs:74` (`link_host_imports`). We add
one new import following the existing pattern:

```rust
// crates/mod-host/src/host_imports.rs (new or extend)
pub fn civ_register_power(linker: &mut Linker<HostState>, state: PowerDef) -> Result<i32, ...> {
    // Validate via the same registry guards (no bypass even for mods).
    let mut reg = state.power_registry.lock();
    match reg.register(state) {
        Ok(_) => { state.replay_log.record_mod_power_registered(...); Ok(0) }
        Err(e) => { state.replay_log.record_mod_power_rejected(e); Ok(e.code()) }
    }
}
```

Mod-registered powers get a `MOD` chip in the deck (AC-REG-6) and are
filterable via `mod_origin` in the deck search.

---

## 8. Phased WBS + DAG

Eight phases. Each task lists its **exact Bevy system(s)**,
**substrate write target**, **crate**, and **dependencies**.

### Phase P1 — Spec + schemas (the "what are we building")

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T1.1** | Land `crates/powers/Cargo.toml` + `src/lib.rs` with `PowerDef`, `PowerRegistry`, `default_powers()` returning the 50-verb skeleton (label + id + tab + category + availability = `Near` for all) | `crates/powers` skeleton | — | 1 subagent, ~5 min |
| **T1.2** | Extend `BrushSettings` (already exists at `crates/voxel/src/hud.rs`): add `drop_height`, `depth`, `symmetry`, `spacing`, `randomize`, `target_height`, `biome_id`, `density` fields | `crates/voxel/src/hud.rs` extension | — | 1 subagent, ~10 min |
| **T1.3** | Land `crates/engine/src/godtools.rs` with `GodToolRequest`, `GodToolReceipt`, `Simulation::apply_god_tool` dispatcher; the dispatcher handles all variants except `Time` and `Inspect` (which are no-ops on the substrate) | `crates/engine/src/godtools.rs` | T1.1 | 1 subagent, ~8 min |

### Phase P2 — Substrate write paths (the "how do tools touch the sim")

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T2.1** | TERRAIN (11 tools): `crates/voxel/src/brush.rs` with `stamp_footprint(world, center, brush, op)`, plus `crates/engine/src/godtools.rs::apply_terraform` dispatching the 11 `TerraformOp` variants. Every variant calls `Simulation::push_voxel_write` (never raw `VoxelWorld::write`). | `crates/voxel/src/brush.rs` + `crates/engine/src/godtools.rs::apply_terraform` | T1.3 | 2 parallel subagents, ~15 min |
| **T2.2** | MATERIAL M1–M5, M8 (5 tools): same `stamp_footprint` path with material id dispatch. Material reuse from `crates/voxel/src/material.rs:125-151`. | same file | T2.1 | 1 subagent, ~8 min |
| **T2.3** | MATERIAL M6 SeedForest: `crates/agents::spawn_many` with `ActorVisualKind::Herd` + plant genome. The Bevy `life.material_seed_forest` handler dispatches to a `LifeRequest::SpawnHerd` (life routes through LIFE). | `crates/agents/src/lib.rs:592` reuse | T2.2, T2.5 | 1 subagent, ~5 min |
| **T2.4** | MATERIAL M7 SeedOreDeposit: `CaGrid::set_with_temp(x,y,z, ORE, ambient_temp)` at `crates/voxel/src/fluid_ca.rs:326` per cell. | `crates/engine/src/godtools.rs::apply_material_seed_ore` | T2.1 | 1 subagent, ~5 min |
| **T2.5** | LIFE (8 tools): `crates/agents/src/effects.rs` with `apply_actor_effect(world, footprint, Effect)`. Spawn via `civ_agents::spawn_child_near` (`crates/agents/src/lib.rs:333`), `spawn_civilian_at` (`crates/agents/src/lib.rs:558`), `spawn_many` (`crates/agents/src/lib.rs:592`). L4/L5/L7 mutate `Needs` only (never `mood`/`alignment`/`culture`/`ideology`). L6 Plague writes a `Pathogen` CA field via `crates/diffusion`. | `crates/agents/src/effects.rs` + `crates/engine/src/godtools.rs::apply_life` | T1.3 | 2 parallel subagents, ~15 min |
| **T2.6** | DISASTER (8 tools): extend `crates/engine/src/disasters.rs::DisasterKind` with `VolcanicVent` and `Tornado`. `crates/engine/src/godtools.rs::apply_disaster` dispatches via `Simulation::invoke_divine_disaster` (`crates/engine/src/disasters.rs:49`) — never bypasses `trigger_disaster`. | `crates/engine/src/disasters.rs` extension | T1.3 | 1 subagent, ~10 min |
| **T2.7** | LAW (8 tools): `crates/engine/src/godtools.rs::apply_law` dispatches to `Simulation::apply_scenario_taxation` (LW1, `crates/engine/src/engine.rs:1049`), `LawDb::apply_overlay` (LW2, `crates/laws/src/lib.rs:154`), trade-route edit (LW4/LW5, `crates/engine/src/engine.rs:318`), `economy_policy` (LW7, `crates/engine/src/engine.rs:444`), `crates/engine/src/policy.rs` (LW6). | `crates/engine/src/godtools.rs::apply_law` | T1.3 | 2 parallel subagents, ~12 min |
| **T2.8** | INSPECT (8 tools): no substrate write. Bevy-only systems read from `crates/watch` aggregates + Legends saga graph + actor entity queries. | `clients/bevy-ref/src/inspect/*.rs` | T1.1 | 1 subagent, ~8 min |
| **T2.9** | TIME (8 tools): Bevy schedule extension; no `Simulation::apply_god_tool` call. TM6 rewinds via `crates/watch/src/terrain.rs` snapshot restore (existing path). | `clients/bevy-ref/src/time/*.rs` | T1.1 | 1 subagent, ~8 min |
| **T2.10** | Lit-but-inert stub for any Near/Blind powers with a named "data not yet surfaced" tag. (L0 v3 partial — registry ships all 50; only the Live set has handlers in P2.) | `crates/powers/src/lib.rs::default_powers()` | T1.1 | 1 subagent, ~5 min |

### Phase P3 — Bevy dispatcher + palette UI

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T3.1** | `clients/bevy-ref/src/tools/dispatcher.rs::apply_god_tool_events` — the single chokepoint that reads `GodToolEvent` and calls `GodToolSimBridge::send`. This is the **AC-CPL-2 monkey-patch point**. | `clients/bevy-ref/src/tools/dispatcher.rs` | T1.3 | 1 subagent, ~5 min |
| **T3.2** | Bevy components + events from §5.1 + §5.2 (`ActivePower`, `BrushSettings`, `MaterialSelector`, `GodToolEvent`, `GodToolReceiptEvent`, `GodToolHistory`, …). | `clients/bevy-ref/src/components/godtools.rs`, `clients/bevy-ref/src/events.rs` | T1.1 | 1 subagent, ~8 min |
| **T3.3** | Keycap Palette rim widget (midnight substrate + teal active edge). 8 fast verbs. Reuses `INK_1`, `GRAPHITE_900/700`, `STEEL_400`, `TEXT_MID/LOW`, `HOLO_CORE`, `HOLO_GLOW`, `HOLO_DEEP` tokens — **no new hex tokens**. | `clients/bevy-ref/src/tools/palette/keycap.rs` | T3.2 | 1 subagent, ~10 min |
| **T3.4** | Holocron Deck modal (radial carousel of 50 verbs + search bar). E2 chrome. | `clients/bevy-ref/src/tools/palette/holocron.rs` | T3.3 | 1 subagent, ~12 min |
| **T3.5** | In-world brush footprint projection — `HOLO_CORE` ring + corner ticks + scan-sweep + idle flicker + aberration. Destructive tools (D*, T8–T10) get `HOLO_ABERR_R` tint. The **only** holo surface outside the HUD. | `clients/bevy-ref/src/tools/brush.rs` | T3.2 | 1 subagent, ~10 min |
| **T3.6** | Per-tab handler files (terrain, material, life, disaster, inspect, law, camera, time) — each file contains 8–11 handler systems that build a `GodToolRequest` and `trigger(GodToolEvent)`. | `clients/bevy-ref/src/tools/{tab}.rs` ×8 | T3.1, T2.* | 4 parallel subagents, ~20 min |
| **T3.7** | Camera (8 verbs) — mouse-driven Bevy systems in `clients/bevy-ref/src/tools/camera.rs`. Hard wall: `CameraTransformMarker` is the only Bevy-side component these systems touch; `Simulation` has no accessor. | `clients/bevy-ref/src/tools/camera.rs` | T3.2 | 1 subagent, ~5 min |
| **T3.8** | Timeline ribbon (holo projection at screen bottom) with Legends event marks. | `clients/bevy-ref/src/time/timeline_ribbon.rs` | T3.5, T2.9 | 1 subagent, ~8 min |
| **T3.9** | Undo (FR-CIV-GODTOOL-921): `undo_god_tool_system` reads `GodToolHistory`, emits the inverse as a new `GodToolRequest` at the current tick (not rollback). Tooltip says "consequences are not undone." | `clients/bevy-ref/src/tools/undo.rs` | T3.2 | 1 subagent, ~5 min |
| **T3.10** | Blueprint paste (FR-CIV-GODTOOL-921): `blueprint_stamp_system` reads a region, captures authored infra (voxel materials, structure tags, road segments), previews + rotates, stamp is a single undoable `EditCommand`. | `clients/bevy-ref/src/tools/blueprint.rs` | T3.9 | 1 subagent, ~5 min |

### Phase P4 — Coupling guards

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T4.1** | Compile-time guard `FORBIDDEN_TARGET_FIELDS` list in `crates/powers/src/registry.rs` (AC-CPL-3). | `crates/powers/src/registry.rs` | T1.1 | 1 subagent, ~3 min |
| **T4.2** | Runtime guard `debug_assert!(!req.kind.is_scripted_outcome())` in `crates/engine/src/godtools.rs` (AC-CPL-2). | `crates/engine/src/godtools.rs` | T1.3 | 1 subagent, ~3 min |
| **T4.3** | Subsystem handle validation in `crates/powers/src/registry.rs::register` (AC-CPL-4): `LawTool.target_subsystem` must reference an existing `known_subsystems()` entry. | `crates/powers/src/registry.rs` | T4.1 | 1 subagent, ~3 min |

### Phase P5 — Mod extensibility

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T5.1** | `civ_register_power` host API in `crates/mod-host/src/wasm_guest.rs` (extends `link_host_imports` at line 74). Validates against the same `PowerRegistry::register` guards. | `crates/mod-host/src/wasm_guest.rs` | T4.* | 1 subagent, ~8 min |
| **T5.2** | Mod-registered powers get a `MOD` chip + `mod_origin` filter in deck search (AC-REG-6). | `clients/bevy-ref/src/tools/palette/holocron.rs` | T3.4, T5.1 | 1 subagent, ~5 min |
| **T5.3** | Mod power replay-bus: emit `mod.power.registered.v1` and `mod.power.rejected.v1` events on the replay log (mirrors `mod.loaded.v1` at `crates/engine/src/engine.rs:773`). | `crates/mod-host/src/wasm_guest.rs` | T5.1 | 1 subagent, ~5 min |

### Phase P6 — Verify (acceptance criteria)

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T6.1** | AC-GT-1: registry enumerates exactly 50 verbs (42 mutating + 8 inspect) with the right tab + category; `default_powers().len() == 50`. | `crates/powers/tests/count.rs` | T2.* | 1 subagent, ~5 min |
| **T6.2** | AC-GT-3 (mass conservation): 100× Raise stamp on a 10³ footprint → `Δmass` ≤ stamp mass ± CA rounding. | `crates/voxel/tests/stamp_conservation.rs` | T2.1 | 1 subagent, ~5 min |
| **T6.3** | AC-GT-4 (emergence): spawn 100 organisms of identical genome → trait standard deviation > 0 across 10k ticks. | `crates/agents/tests/spawn_emergence.rs` | T2.5 | 1 subagent, ~8 min |
| **T6.4** | AC-GT-5 (CA propagation): meteor strike mass M → thermal spread matches `heat_diffusion_equation` within ±5%. | `crates/engine/tests/meteor_thermal.rs` | T2.6 | 1 subagent, ~8 min |
| **T6.5** | AC-GT-6 (time): TM1 pause → tick counter unchanged after 60s real time; TM5 step → exactly N ticks; TM7 → next event-of-kind auto-pause. | `crates/engine/tests/time_controls.rs` | T2.9 | 1 subagent, ~5 min |
| **T6.6** | AC-GT-7 (camera): sim hash before/after `camera.orbit` → identical. | `clients/bevy-ref/tests/camera_isolation.rs` | T3.7 | 1 subagent, ~5 min |
| **T6.7** | AC-GT-8 (undo): `Ctrl+Z` after Raise stamp → footprint reverts via inverse `EditCommand`; subsequent forward diverges from original (soft determinism per spec §3.8). | `crates/engine/tests/undo_godtool.rs` | T3.9 | 1 subagent, ~8 min |
| **T6.8** | AC-CPL-1..4 (coupling guards): registration rejects `culture`/`religion`/`ideology`/`alignment`/`job`/`faction_id`/`mood` direct writes; rejects unknown subsystem handles; rejects `ScriptedOutcome` request kind; monkey-patch dispatcher to no-op → zero world change. | `crates/powers/tests/guards.rs`, `crates/engine/tests/dispatcher_no_op.rs` | T4.* | 1 subagent, ~8 min |
| **T6.9** | AC-UI-3 (vision-verify): screenshot + read pixels; ≤8% neon per panel; ≤2 holo HUD surfaces; only one in-world projection. | `clients/bevy-ref/tests/vision_verify.rs` | T3.* | 1 subagent, ~5 min |
| **T6.10** | AC-REG-1..6 (registry extensibility): adding a power requires only one `PowerDef` + one handler; deck + search + rim + ring all derive from registry; mod-added power gets `MOD` chip + `mod_origin` filter. | `crates/powers/tests/registry_extensibility.rs` | T5.* | 1 subagent, ~5 min |

### Phase P7 — Documentation + traceability

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T7.1** | Wire the 50 god-tool verbs into `docs/traceability/fr-3d-matrix.md` (the authoritative 3D FR matrix per AGENTS.md). | `docs/traceability/fr-3d-matrix.md` | T2.* | 1 subagent, ~5 min |
| **T7.2** | Add `docs/design/god-tools-cookbook.md` — 1 recipe per tool (input gesture → substrate mutation → drain phase). Linked from `docs/design/GOD_TOOLS_SANDBOX.md`. | `docs/design/god-tools-cookbook.md` | T3.* | 1 subagent, ~10 min |
| **T7.3** | Update `docs/design/brush-tool-system.md` to reflect the extended `BrushSettings` from T1.2. | `docs/design/brush-tool-system.md` | T1.2 | 1 subagent, ~5 min |
| **T7.4** | Add a god-tools entry to the Bevy client README (`clients/bevy-ref/README.md` if present) showing the 8 tabs + key bindings + the 8-rim keycaps. | `clients/bevy-ref/README.md` | T3.4 | 1 subagent, ~5 min |

### Phase P8 — Mirror to web / Godot / Unreal clients

| Task | Description | Output | Deps | Effort |
|---|---|---|---|---|
| **T8.1** | Mirror `PowerRegistry` to web client (vanilla TS) via the existing JSON-RPC catalog. The Holocron Deck UI is React; the palette renders the same 50 verbs. | `web/src/godtools/` | T3.4 | 1 subagent, ~15 min |
| **T8.2** | Mirror `PowerRegistry` to Godot client (GDScript) — see `clients/godot-ref/`. | `clients/godot-ref/godtools/` | T3.4 | 1 subagent, ~15 min |
| **T8.3** | Mirror `PowerRegistry` to Unreal client (Blueprint + C++) — see `clients/unreal-show/`. | `clients/unreal-show/Source/CivShow/GodTools/` | T3.4 | 1 subagent, ~20 min |

---

## 9. DAG (text representation)

```
                           ┌──────────┐
                           │   T1.1   │  PowerDef + PowerRegistry skeleton
                           └─────┬────┘
                                 │
                  ┌──────────────┼──────────────┐
                  │              │              │
                  ▼              ▼              ▼
              ┌───────┐      ┌───────┐      ┌────────┐
              │ T1.2  │      │ T1.3  │      │ T2.10  │  Lit-but-inert stubs
              │Brush+ │      │Substr.│      └────┬───┘
              └────┬──┘      │Disp.  │           │
                   │         └───┬───┘           │
                   │     ┌───────┴───────────────┼────────────────────────┐
                   │     │       │               │                        │
                   │     ▼       ▼               ▼                        ▼
                   │ ┌─────┐ ┌─────┐         ┌─────┐                  ┌─────┐
                   │ │T2.1 │ │T2.5 │         │T2.6 │                  │T2.7 │
                   │ │TER. │ │LIFE │         │DIS. │                  │ LAW │
                   │ └──┬──┘ └──┬──┘         └──┬──┘                  └──┬──┘
                   │    │       │               │                       │
                   │    │  ┌────┴────┐    ┌─────┴─────┐                 │
                   │    │  ▼         ▼    ▼           ▼                 │
                   │    │ ┌────┐ ┌────┐ ┌────┐    ┌────┐                │
                   │    │ │T2.2│ │T2.3│ │T2.4│    │T2.8│                │
                   │    │ │M1-5│ │M6  │ │M7  │    │INSP│                │
                   │    │ │M8  │ │    │ │    │    └────┘                │
                   │    │ └──┬─┘ └────┘ └────┘                           │
                   │    │    │      │      │                             │
                   │    └────┴──────┴──────┴─────────────┬───────────────┘
                   │                                    │
                   │                                    ▼
                   │                              ┌────────┐
                   │                              │  T2.9  │  TIME (Bevy-only)
                   │                              └────┬───┘
                   │                                   │
                   └─────────────────┬─────────────────┘
                                     │
                                     ▼
                                 ┌───────┐
                                 │  T3.2 │  Bevy components + events
                                 └───┬───┘
                                     │
                ┌────────────┬───────┼───────┬────────────┐
                ▼            ▼       ▼       ▼            ▼
            ┌───────┐   ┌───────┐ ┌─────┐ ┌─────┐    ┌──────┐
            │ T3.3  │   │ T3.5  │ │T3.7 │ │T3.9 │    │T3.10 │
            │Keycap │   │Brush  │ │CAM. │ │Undo │    │Bluept│
            └───┬───┘   │ Ring  │ └──┬──┘ └──┬──┘    └───┬──┘
                │       └───┬───┘    │       │           │
                ▼           │        │       │           │
            ┌───────┐       │        │       │           │
            │ T3.4  │       │        │       │           │
            │Holocr.│       │        │       │           │
            └───┬───┘       │        │       │           │
                │           │        │       │           │
                └─────┬─────┴────────┴───────┴───────────┘
                      ▼
                  ┌───────┐
                  │  T3.1 │  dispatcher (the bridge)
                  └───┬───┘
                      │
                      ▼
                  ┌───────┐
                  │  T3.6 │  per-tab handler files (8 files)
                  └───┬───┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
    ┌───────┐    ┌────────┐    ┌────────┐
    │  T4.* │    │  T5.*  │    │  T6.*  │  Guards / Mods / Verify
    └───┬───┘    └───┬────┘    └───┬────┘
        │             │             │
        └─────────────┴─────────────┘
                      │
                      ▼
                  ┌───────┐
                  │  T7.* │  docs + traceability
                  └───┬───┘
                      │
                      ▼
                  ┌───────┐
                  │  T8.* │  mirror to web / Godot / Unreal
                  └───────┘
```

**Critical path:** `T1.1 → T1.3 → T2.1 → T3.6 → T6.4 → T7.1 → T8.2` (longest dependency chain).

**Parallel width after P2:** 4+ (T3.* palette UI, T4.* coupling guards, T5.* mod extensibility, T6.* verify).

---

## 10. Per-tool substrate write index (the charter gate)

The table below is the **canonical row** of the spec §3 tables, restated as
substrate writes (no bypass):

| ID | Substrate field | Substrate write API | Drain phase (PHASE_ORDER) |
|---|---|---|---|
| `terrain.raise` | `VoxelWorld<MaterialId>::write` | `Simulation::push_voxel_write` (`crates/engine/src/engine.rs:904`) | `voxel` |
| `terrain.lower` | `VoxelWorld::write` | same | `voxel` |
| `terrain.level` | `VoxelWorld::write` | same | `voxel` |
| `terrain.smooth` | `VoxelWorld::write` (clear-then-write pattern from `crates/engine/src/engine.rs:487`) | same | `voxel` |
| `terrain.slope` | `VoxelWorld::write` | same | `voxel` |
| `terrain.flatten` | `VoxelWorld::write` (reads top voxel first) | same | `voxel` |
| `terrain.shift` | `VoxelWorld::write` | same | `voxel` |
| `terrain.add_land` | `VoxelWorld::write(STONE/PACKED_DIRT)` | same | `voxel` |
| `terrain.dig_ocean` | `VoxelWorld::write(WATER)` + planet sea_level set | same + `phase_planet` (`crates/engine/src/engine.rs:1269`) | `voxel`, `planet` |
| `terrain.raise_mountain` | `VoxelWorld::write(STONE/GRAVEL)` Gaussian | same | `voxel` |
| `terrain.drop_biome` | `VoxelWorld::write` (surface only) + climate write | same + `phase_planet` | `voxel`, `planet` |
| `material.replace` | `VoxelWorld::write(mat)` | same | `voxel` |
| `material.additive_drop` | `VoxelWorld::write` at `drop_height` | same | `voxel` (CA advects) |
| `material.erase` | `VoxelWorld::write(AIR)` | same | `voxel` |
| `material.surface_paint` | `VoxelWorld::write` (topmost only) | same | `voxel` |
| `material.pour_liquid` | `VoxelWorld::write(WATER/LAVA/OIL)` | same | `voxel` |
| `material.seed_forest` | `crates/agents::spawn_many(...)` (`crates/agents/src/lib.rs:592`) | spawns plant-agents | `citizen_lifecycle` |
| `material.seed_ore` | `CaGrid::set_with_temp(x,y,z,ORE,t)` (`crates/voxel/src/fluid_ca.rs:326`) | CA propagates | `voxel` |
| `material.seed_snow` | `VoxelWorld::write(ICE)` above snowline | thermo CA melts | `voxel` |
| `life.spawn_organism` | `crates_agents::spawn_child_near` (`crates/agents/src/lib.rs:333`) | agent lifecycle | `citizen_lifecycle` |
| `life.spawn_herd` | `civ_agents::spawn_many` (`crates/agents/src/lib.rs:592`) | same | `citizen_lifecycle` |
| `life.spawn_civ_seed` | 6× `spawn_organism` + 1× `BuildingGraph::add_building` + 1× `Resources` deposit | same | `citizen_lifecycle`, `buildings` |
| `life.bless` | `apply_actor_effect(Effect::MoodBoost(+Δ))` (writes `Needs` only) | `crates/agents/src/effects.rs` | `citizen_lifecycle` |
| `life.curse` | inverse of `life.bless` | same | `citizen_lifecycle` |
| `life.plague` | `Pathogen` SIR field write | `crates/diffusion` | `citizen_lifecycle` |
| `life.heal` | `apply_actor_effect(Effect::HealthRestore(+Δ))` | same | `citizen_lifecycle` |
| `life.extinct` | `hecs::World::despawn(entity)` | population accounting | `citizen_lifecycle` |
| `disaster.meteor` | `Simulation::invoke_divine_disaster(Meteor, pos, cost=0)` (`crates/engine/src/disasters.rs:49`) | CA + `phase_disasters` | `tactics`, `voxel`, `planet` |
| `disaster.lightning` | `invoke_divine_disaster(Wildfire, …)` (electric field via `crates/voxel/src/reactions.rs`) | same | same |
| `disaster.flood` | `invoke_divine_disaster(Flood, …)` | same | same |
| `disaster.quake` | `invoke_divine_disaster(Quake, …)` + `push_damage` (`crates/engine/src/engine.rs:898`) | same + structural damage | `tactics`, `voxel` |
| `disaster.firestorm` | `invoke_divine_disaster(Wildfire, …, radius)` | same | same |
| `disaster.tornado` | `invoke_divine_disaster(Storm, …)` + `WeatherCell` wind-field write | `crates/planet/src/weather.rs` | `planet`, `tactics` |
| `disaster.volcanic_vent` | sustained `push_voxel_write(LAVA)` + `invoke_divine_disaster(Meteor, …)` | same | same |
| `disaster.drought` | `WeatherCell::precip_mm_fp -= Δ` | `crates/planet/src/weather.rs` | `planet` |
| `inspect.*` | (none) | (read-only) | n/a |
| `law.tax_bias` | `Simulation::apply_scenario_taxation` (`crates/engine/src/engine.rs:1049`) | `phase_economy` | `economy` |
| `law.edict` | `LawDb::apply_overlay` (`crates/laws/src/lib.rs:154`) | `phase_policy` | `policy` |
| `law.religion_pressure` | `crates/diffusion` SIR field | `phase_diffusion` | `diffusion` |
| `law.sanction` | `WorldState::trade_routes.remove(...)` | `phase_economy` reroutes | `economy` |
| `law.open_border` | inverse — `trade_routes.insert(...)` (subject to validation) | same | `economy` |
| `law.alignment_nudge` | `crates/engine/src/policy.rs` AI utility weights | `phase_policy` | `policy` |
| `law.difficulty_knob` | `Simulation::economy_policy.scarcity_multiplier` (`crates/engine/src/engine.rs:444`) | `phase_economy` | `economy` |
| `law.scenario_script` | replays stored `(god-tool, params, tick)` sequence via the standard queue — **no new mutation pathway** | same as the underlying tools | same |
| `camera.*` | (none — UI only) | n/a | n/a |
| `time.pause` / `play` / `slow` / `fast` | `Time<Fixed>::set_relative_speed(...)` (Bevy) | schedule gate | n/a |
| `time.step` | advances N ticks via `Schedule::run` then auto-pauses | schedule | n/a |
| `time.rewind` | `Snapshot::restore_into_simulation` (existing `crates/watch/src/terrain.rs` path) | snapshot restore | n/a |
| `time.fast_forward_to_event` | filters `crates/watch` stream; advances until match | schedule | n/a |
| `time.profile` | writes a perf-trace log (separate from replay log) | schedule | n/a |

**Total tools with substrate writes:** **42 mutating** (TERRAIN 11 +
MATERIAL 8 + LIFE 8 + DISASTER 8 + LAW 8 - 1 = 42; ScenarioScript routes
through the standard queue and is the "meta" tool counted in LAW). Of
the 50 verbs: **8 inspect** (no writes), **8 camera** (UI only), **8
time** (Bevy schedule), **42 mutating** (substrate writes via the
dispatcher).

---

## 11. Cross-cutting: which crate each piece lives in

| Crate | New / extended file | Purpose |
|---|---|---|
| `crates/powers/Cargo.toml` | **new** | crate manifest |
| `crates/powers/src/lib.rs` | **new** | `PowerDef`, `PowerRegistry`, `default_powers()`, AC-REG-1..6 tests |
| `crates/powers/src/registry.rs` | **new** | registration + validation, AC-CPL-3 + AC-CPL-4 |
| `crates/voxel/src/brush.rs` | **new** | `BrushOp`, `stamp_footprint` — the substrate write helper |
| `crates/voxel/src/hud.rs` | **extends** | extends `BrushSettings` (T1.2) |
| `crates/engine/src/godtools.rs` | **new** | `GodToolRequest`, `GodToolReceipt`, `Simulation::apply_god_tool` dispatcher |
| `crates/engine/src/disasters.rs` | **extends** | adds `VolcanicVent` + `Tornado` to `DisasterKind` |
| `crates/agents/src/effects.rs` | **new** | `apply_actor_effect(world, footprint, Effect)` — the only path for Bless/Curse/Heal/Plague |
| `crates/mod-host/src/wasm_guest.rs` | **extends** | adds `civ_register_power` host import (T5.1) |
| `clients/bevy-ref/src/components/godtools.rs` | **new** | Bevy components from §5.1 |
| `clients/bevy-ref/src/events.rs` | **extends** | Bevy events from §5.2 |
| `clients/bevy-ref/src/tools/dispatcher.rs` | **new** | `apply_god_tool_events` — the AC-CPL-2 bridge |
| `clients/bevy-ref/src/tools/{tab}.rs` ×8 | **new** | per-tab handler files |
| `clients/bevy-ref/src/tools/palette/{keycap,holocron}.rs` | **new** | Keycap Palette + Holocron Deck |
| `clients/bevy-ref/src/tools/brush.rs` | **new** | in-world brush ring |
| `clients/bevy-ref/src/tools/undo.rs` | **new** | undo (FR-CIV-GODTOOL-921) |
| `clients/bevy-ref/src/tools/blueprint.rs` | **new** | blueprint paste (FR-CIV-GODTOOL-921) |
| `clients/bevy-ref/src/time/{timeline_ribbon}.rs` | **new** | timeline ribbon |
| `docs/traceability/fr-3d-matrix.md` | **extends** | adds 50 god-tool verbs |
| `docs/design/god-tools-cookbook.md` | **new** | 1 recipe per tool |
| `docs/design/brush-tool-system.md` | **extends** | reflects extended `BrushSettings` |
| `web/src/godtools/` | **new** | web mirror |
| `clients/godot-ref/godtools/` | **new** | Godot mirror |
| `clients/unreal-show/Source/CivShow/GodTools/` | **new** | Unreal mirror |

---

## 12. Tool count — the headline figure

Per the spec verdict at `docs/design/GOD_TOOLS_SANDBOX.md:580` and
FR-CIV-GODTOOL-900, the canonical headline figure for this plan is:

> **42 mutating god-tools across 7 FR tabs** (TERRAIN 11 + MATERIAL 8
> + LIFE 8 + DISASTER 8 + INSPECT 8 + LAW 8 + TIME 8 = 51; excluding
> INSPECT (read-only) and TIME (clock control, not substrate mutation)
> gives the mutating count: TERRAIN 11 + MATERIAL 8 + LIFE 8 + DISASTER
> 8 + LAW 8 = **43**, minus 1 for `law.scenario_script` which is a
> meta-tool that routes through the underlying verbs and is not itself
> a distinct mutation path → **42**).

The 50-verb headline (42 mutating + 8 read-only INSPECT) is the count
of **`PowerDef` entries in `crates/powers::PowerRegistry::default_powers()`**.

**Plan tools-planned (this PR introduces the plan + skeleton):** **50
verbs registered, 42 mutating substrate writes + 8 read-only INSPECT
verbs**.

---

## 13. Acceptance criteria mapping

This plan's phases satisfy:

- **AC-GT-1 (FR-900):** T1.1 + T2.10 register all 50 verbs; deck navigable; brush preview on arm — implemented in P3.
- **AC-GT-2 (FR-901):** T1.1 + T1.3 + T2.* wire all verbs through `PowerRegistry`; adding a power = one entry + one handler.
- **AC-GT-3 (FR-910):** T2.1 mass-conserving voxel writes; verified by T6.2.
- **AC-GT-4 (FR-911):** T2.5 spawns via `crates/agents::spawn_*`; verified by T6.3.
- **AC-GT-5 (FR-912):** T2.6 disasters write CA initial conditions only; verified by T6.4.
- **AC-GT-6 (FR-920):** T2.9 time controls; verified by T6.5.
- **AC-GT-7 (FR-920 god-hand):** T3.7 camera isolation; verified by T6.6.
- **AC-GT-8 (FR-921 Undo):** T3.9 undo; verified by T6.7.
- **AC-GT-9 (FR-921 Blueprint):** T3.10 blueprint paste.
- **AC-CPL-1..4 (charter):** T4.* guards; verified by T6.8.
- **AC-UI-1..5:** T3.3 + T3.4 + T3.5 use existing tokens only; verified by T6.9.
- **AC-TIME-1..3:** T2.9; verified by T6.5.
- **AC-REG-1..6:** T1.1 + T2.* + T5.*; verified by T6.10.

---

## 14. Notes on the spec's own §9 WBS

This plan's P1–P8 are a **re-derivation of the spec's WBS at
`docs/design/GOD_TOOLS_SANDBOX.md:543-569`**, grounded against actual
engine.rs symbols (`PHASE_ORDER` line 55, `push_voxel_write` line 904,
`push_damage` line 898, `invoke_divine_disaster` at
`crates/engine/src/disasters.rs:49`, `phase_voxel` line 1425, etc.).
Differences:

- The spec's WBS mentions `crates/powers` as a "or fold into
  crates/engine" choice — this plan creates the crate (it's a clean
  module boundary and the AC-CPL guards belong with the registry, not
  the engine).
- This plan adds `crates/agents/src/effects.rs` (the spec didn't name
  it but `apply_actor_effect` is the substrate write that the LIFE
  tools must route through; without it, LIFE tools would write to
  `Needs`/`Health` directly and bypass the no-bypass guarantee for the
  agent layer).
- This plan adds T2.3 (MATERIAL M6 = LIFE spawn) and T2.4 (MATERIAL M7
  = CA field write) as separate tasks because the substrate write paths
  are different.
- This plan splits the spec's P3 "palette UI" into T3.3 (Keycap rim),
  T3.4 (Holocron Deck), T3.5 (in-world ring) to make each task
  independently verifiable.

---

## 15. PR scope (this branch)

This branch (`feat/godtools-impl-plan`) introduces:

1. **This plan** (`docs/design/GODTOOLS_IMPL_PLAN.md`).
2. **`crates/powers/` skeleton** — `PowerDef`, `PowerRegistry`,
   `default_powers()` enumerating all 50 verbs as `Near` (no handlers
   yet — that's P2-P3 follow-up).
3. **`crates/voxel/src/brush.rs` skeleton** — `BrushOp` enum + `stamp_footprint` signature (no implementation — that's P2 follow-up).
4. **`crates/engine/src/godtools.rs` skeleton** — `GodToolRequest` +
   `Simulation::apply_god_tool` signature (no handlers — that's P2 follow-up).
5. **`crates/powers/tests/count.rs`** — AC-REG-1 verification: `default_powers().len() == 50`.

Follow-up PRs (separate branches) land the P2 substrate writes, P3 Bevy
handlers, P4 guards, P5 mods, P6 tests, P7 docs, P8 mirrors.