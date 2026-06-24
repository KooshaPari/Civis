# Bevy Game Gap Analysis

Date: 2026-05-27

Scope: `clients/bevy-ref/src/bin/standalone.rs` versus the web dashboard scene and adjacent dashboard UI.

## Current Bevy Reality

- The standalone Bevy binary does create a `Simulation` and tick it on a timer.
- It does render civilians from the simulation world.
- It does render a static terrain, a sun/moon/star sky cycle, and four hard-coded building cubes.
- It does not create any gameplay UI, selection UI, authoring tools, minimap, or dashboard panels.
- It does not expose the web dashboard's snapshot-driven gameplay surface.

## Build Check

- Command run: `cargo build -p civ-bevy-ref --features bevy --bin civ-standalone --release`
- Result: build was still compiling in an isolated target dir when this audit was written; no compile error was observed during the captured window.
- Practical interpretation: source review did not reveal an obvious syntax/runtime compile blocker in `standalone.rs`, but the build was not fully observed to completion in this session.

## What Standalone Actually Does

### `standalone.rs`

- It inserts `SimState(Simulation::with_seed(42))` at startup.
- It runs `update_sim`, which calls `sim_state.0.tick()` every `0.1s`.
- It syncs civilians from `sim_state.0.world` into Bevy entities in `sync_civilians`.
- It spawns:
  - one camera
  - one sun light
  - one moon light
  - one terrain mesh
  - one red sphere at world center
  - four building cubes
  - a hidden star field
- It has no `egui` or `bevy_ui` integration in this file.

### `Simulation::new()`

- `Simulation::new()` and `Simulation::with_seed()` both call:
  - `Self::spawn_initial_entities(&mut world)`
  - `spawn_faction_civilians(&mut world, &mut spawn_rng)`
  - `attach_citizen_to_agents(&mut world)`
- `spawn_initial_entities` creates:
  - 100 citizens
  - 1 city center building
  - 5 farm buildings
  - 5 soldiers for faction 0
  - 5 archers for faction 1
- `spawn_faction_civilians` creates 4 factions' worth of civilians.
- The engine test confirms startup spawns `128` civilians total.
- `WorldState::default()` seeds four factions:
  - faction 0
  - faction 1
  - faction 2
  - faction 3
- So `Simulation::new()` creates `4` factions, not an empty world.

## Web Dashboard Feature Set

The web dashboard scene drives the simulation as a snapshot-driven game client, with much broader gameplay affordances than the Bevy standalone binary.

Relevant entry points:

- [`web/dashboard/src/scene3d.tsx`](../../web/dashboard/src/scene3d.tsx)
- [`web/dashboard/src/bottom_bar.tsx`](../../web/dashboard/src/bottom_bar.tsx)
- [`web/dashboard/src/tech_tree.tsx`](../../web/dashboard/src/tech_tree.tsx)
- [`web/dashboard/src/event_feed.tsx`](../../web/dashboard/src/event_feed.tsx)

## Gap Matrix

| Feature | Web has it | Bevy standalone has it | Effort to add to Bevy | Priority | Blocked by |
|---|---|---|---|---|---|
| Civilians rendering + movement | Yes. Civilians are rendered and interpolated from snapshots in `updateCivilians` / `updateCiviliansFromRefs`. | Partial. Civilians are rendered as capsules, but only as sync output from the engine; no gameplay interaction layer. | Medium | High | A Bevy snapshot protocol or direct ECS-to-render bridge that preserves civilian positions and metadata each tick. |
| Buildings rendering | Yes. Buildings are rendered with per-kind meshes, clustering, faction tint, and occupancy cues. | Partial. Only four fixed cubes are spawned in `spawn_building_cubes`; no building data is bound to simulation buildings. | Medium | High | Building snapshot export and a renderer that maps building kind/era/faction/occupancy to meshes/materials. |
| Faction territories + colors | Yes. Territorials are drawn from `snapshot.factions` with color, radius, and conflict highlighting. | No. There is no territory layer in standalone. | Medium | High | Need faction territory data in Bevy runtime plus a territory mesh/overlay renderer. |
| Spawn tools (WorldBox-style bottom bar) | Yes. Spawn tools are exposed through the dashboard authoring flow and bottom bar controls. | No. There are no authoring tool controls in the Bevy binary. | High | High | A Bevy UI layer plus authoring RPCs or local commands for spawn tools, drag gestures, and tool state. |
| Tech tree UI | Yes. The dashboard includes a dedicated tech tree panel. | No. | High | Medium | A Bevy UI framework decision first; then a tech progression model and panel data binding. |
| Military units | Yes. `scene3d.tsx` renders military units, selection rings, and faction-based combat highlighting. | No gameplay military layer in standalone. The engine has units, but standalone does not render them or expose control/selection. | Medium | High | Military snapshot export and Bevy rendering/selection plumbing. |
| Trade routes | Yes. Trade routes are drawn and labeled between factions. | No. | Medium | Medium | Trade route data in Bevy plus line/label rendering and snapshot updates. |
| Weather effects | Yes. Rain and snow are animated from `snapshot.weather`. | Partial. Standalone has only day/night sky lighting; no rain/snow weather system. | Medium | Medium | Weather state source in the Bevy-facing simulation path and particle/FX systems. |
| Save/load | Yes. The web dashboard is wired to snapshot/session state and authoring flows that can persist or restore server state. | No visible save/load UI or flow in standalone. | High | Medium | Save bundle or replay/load plumbing exposed to the Bevy client and UI hooks. |
| Speed controls | Yes. `setSpeed`, keyboard shortcuts, and server `sim.set_speed` control simulation speed. | No. Standalone ticks at a fixed `0.1s` timer with no user-facing speed control. | Low | High | Input/UI plumbing and a simulation time-scale resource. |
| Entity selection + info panel | Yes. The scene supports inspect selection, hover tooltips, LOS rings, and a side panel. | No. Standalone has no selection raycast, tooltip, or info panel. | Medium | High | UI layer plus hit-testing against civilians/buildings/units and metadata export. |
| Event feed | Yes. The dashboard has an event feed panel and related notifications. | No. | Medium | Medium | Event stream export from the simulation into a Bevy UI panel. |
| Minimap | Yes. The dashboard has minimap-style overview behavior in the larger UI surface. | No. | Medium | Medium | World overview rendering and a compact camera/overlay path in Bevy. |

## Evidence Notes

- Bevy startup and tick loop:
  - `Simulation::with_seed(42)` is inserted at startup.
  - `update_sim` calls `tick()` on a timer.
  - `sync_civilians` turns world civilians into Bevy meshes.
- Bevy rendering is hard-coded, not gameplay-driven:
  - terrain mesh
  - one sphere
  - four building cubes
  - sky cycle
- No UI systems or UI plugin setup appear in `standalone.rs`.
- The web scene wires together:
  - civilian rendering and interpolation
  - building rendering
  - faction territories
  - military units
  - trade routes
  - weather FX
  - speed controls
  - inspect/selection handling
  - hover tooltip/info text
  - fog-of-war and LOS overlay

## Bottom Line

The Bevy standalone binary is currently a visualizer for a subset of engine state, not a feature-complete game client. It has the simulation tick and civilian sync, but it lacks the dashboard's interaction model, UI surface, and most gameplay affordances.
