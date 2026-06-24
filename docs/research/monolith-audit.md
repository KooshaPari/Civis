# Civis Monolith Audit

Scope:
- `crates/` and `clients/` `.rs` files over 300 lines
- `web/dashboard/src/` `.ts` / `.tsx` files over 500 lines
- `clients/godot-ref/` `.gd` files over 200 lines
- `lib.rs` and `engine.rs` files with mixed concerns that should be split further

This audit is based on file line counts and a structural read of the largest entrypoints.

## Oversized files

### Rust files over 300 lines

| File | Lines | Primary concern |
|---|---:|---|
| `crates/watch/src/main.rs` | 4365 | Server bootstrap, API routing, simulation worker, snapshot synthesis, saves, mods, remote mod fetch, and test harness all in one file |
| `crates/engine/src/engine.rs` | 2506 | Domain model, ECS simulation state, tick phases, replay/save/load, mod-host integration, and snapshot assembly |
| `crates/server/src/jsonrpc.rs` | 2428 | JSON-RPC schema, parsing, dispatch planning, snapshot serialization, and test fixtures |
| `crates/server/tests/ws_smoke.rs` | 1935 | Large integration test suite; test helpers and scenario coverage are heavily bundled |
| `crates/mod-host/src/lib.rs` | 1776 | Host policy, guest lifecycle, archive/manifest handling, determinism checks, and event formatting |
| `crates/server/src/ws_bridge.rs` | 1257 | WebSocket bridge lifecycle, command intake, transport encoding, and snapshot publishing |
| `crates/agents/src/lib.rs` | 1092 | Agent data model, spawning, movement, needs propagation, wardrobe/tools, and state transfer |
| `clients/bevy-ref/src/lib.rs` | 994 | Pure helpers, URL/env resolution, chunk math, minimap math, presentation helpers, and frame parsers |
| `clients/bevy-ref/src/bin/bevy_window.rs` | 846 | Window bootstrap, CLI parsing, rendering setup, resource wiring, and runtime controls |
| `crates/build/src/lib.rs` | 651 | Building allocation heuristics, graph mutation, and demand-driven construction rules |
| `crates/research/src/lib.rs` | 649 | Research policy, capability or progress bookkeeping, and scenario-facing domain helpers |
| `crates/engine/src/replay.rs` | 600 | Replay event model, serialization, hash-chain interaction, and restore logic |
| `crates/tactics/src/formation.rs` | 579 | Formation layout, tactical positioning, and geometry helpers |
| `crates/mod-host/src/wasm_guest.rs` | 562 | Guest runtime plumbing, tick execution, serialization, and error handling |
| `crates/engine/src/scenario.rs` | 511 | Scenario loading, validation, and scenario-to-engine conversion |
| `crates/tactics/src/lib.rs` | 483 | Public facade with too many tactical subdomains re-exported together |
| `clients/bevy-ref/src/bin/standalone.rs` | 472 | Standalone app bootstrap and renderer/window wiring |
| `crates/tactics/src/war_bridge.rs` | 467 | Tactical bridge between tactical state and engine/replay surfaces |
| `crates/economy/src/institution.rs` | 365 | Institution state, balance logic, and economic policy interaction |
| `crates/save-db/src/lib.rs` | 358 | Persistence format, filesystem access, and save metadata handling |
| `crates/protocol-3d/src/lib.rs` | 336 | Protocol wire types mixed with serialization helpers |
| `crates/diffusion/src/lib.rs` | 335 | Diffusion rules, math helpers, and domain state |
| `crates/tactics/src/fog_of_war.rs` | 332 | Visibility simulation and grid projection logic |
| `crates/species/src/lib.rs` | 325 | Species state, traits, and domain helpers |
| `crates/tactics/src/los.rs` | 324 | Line-of-sight geometry and visibility helpers |
| `crates/engine/src/save_bundle.rs` | 320 | Save bundle structure, versioning, and archive conversion |
| `crates/tactics/src/pathfinding.rs` | 318 | Pathfinding algorithm, grid queries, and movement helpers |

### Web dashboard files over 500 lines

| File | Lines | Primary concern |
|---|---:|---|
| `web/dashboard/src/scene3d.tsx` | 2939 | Scene orchestration, terrain rebuilds, entity rendering, camera control, overlays, effects, and server interaction |
| `web/dashboard/src/bottom_bar.tsx` | 839 | Bottom-bar layout, control wiring, state-dependent visibility, and interactive panel logic |
| `web/dashboard/src/store.tsx` | 576 | Global dashboard state, snapshot normalization, and action reducers/selectors |

### Godot files over 200 lines

| File | Lines | Primary concern |
|---|---:|---|
| `clients/godot-ref/scripts/main.gd` | 504 | Scene bootstrap, terrain mesh build, UI binding, input handling, spawn tools, and attach-mode behavior |

## Files that are especially monolithic

### `crates/engine/src/engine.rs`

This is the clearest Rust monolith in the repo.

It mixes:
- core simulation domain types (`Citizen`, `Building`, `Resources`, `MilitaryUnit`, `WorldState`)
- simulation application service logic (`Simulation::new`, `with_seed`, `tick`, phase methods)
- persistence and replay concerns (`save_replay`, `load_replay_from_file`, replay log mutation)
- mod-host integration (`install_mod_path`, `reload_mod_by_id`, guest state export/restore)
- snapshot and spectator assembly (`snapshot`, `last_tick_*` accessors)
- world mutation helpers (`push_damage`, `push_voxel_write`, `voxel_mut`)

Recommended split:
- `domain.rs` for entity/state types and small pure helpers
- `simulation.rs` for `Simulation` lifecycle and phase orchestration
- `phases/` for `phase_*` functions, one file per phase or per phase cluster
- `replay.rs` for replay-log, save/load, and hash-chain integration
- `mods.rs` for host lifecycle and guest-state adapters
- `snapshot.rs` for `SimulationSnapshot` and spectator export
- `world.rs` for voxel/building/resource mutation helpers

Hexagonal pattern:
- Domain core: entity/state structs, resource math, deterministic helpers
- Application layer: `Simulation` orchestrates phase order and delegates outward
- Ports: replay persistence, mod host, voxel world, filesystem save/load
- Adapters: `civ_mod_host`, `civ_voxel`, `std::fs`, `serde`

Pure domain logic:
- `job_type_for_civilian_id`
- `resource_amount`
- `adjust_resource`
- likely most arithmetic inside `phase_*` once lifted out of IO/world mutation

Infrastructure:
- mod host install/reload/export/restore
- `save_replay`, `load_replay_from_file`
- voxel write proxy and filesystem touchpoints

Presentation / export:
- `snapshot`
- spectator-facing pulse/accessor methods
- last-tick summary getters

### `crates/server/src/jsonrpc.rs`

This file is a protocol aggregator rather than a focused parser.

It mixes:
- wire enums and request/response types
- parse/validate routines
- dispatch-planning logic
- snapshot marshaling from engine state
- save-slot and replay request parsing
- large test coverage for all methods

Recommended split:
- `wire.rs` for request/response/error types and method enums
- `parse.rs` for structural validation and parameter parsing
- `dispatch.rs` for `DispatchContext`, `DispatchPlan`, and `DispatchEffect`
- `snapshot.rs` for `SnapshotFields` and simulation-to-JSON conversion
- `mod.rs` or `lib.rs` as a thin public facade

Hexagonal pattern:
- Domain/protocol core: request/response shapes, method enum, error codes
- Application layer: `dispatch_request`
- Ports: simulation state, save-store access, role policy
- Adapters: `serde_json`, file/snapshot serialization, engine bridge access

Pure domain logic:
- `JsonRpcMethod::parse_name`
- `JsonRpcResponse::success/failure`
- `JsonRpcParseError::{code,message,into_error}`
- parameter decoding and validation helpers

Infrastructure:
- save/replay path parsing
- snapshot extraction from `civ_engine::Simulation`
- dispatch effects that mutate external state

Presentation:
- `snapshot_result_json`
- `encode_response`
- fields that only format data for the wire

### `crates/watch/src/main.rs`

This is the largest monolith in the repo and should be split first.

It mixes:
- server bootstrap and routing
- simulation worker loop
- snapshot synthesis for the dashboard
- terrain generation/cache
- save and slot management
- mod catalog / install / publish / fetch / remote cache handling
- test harness and integration coverage

Recommended split:
- `main.rs` for startup only
- `app.rs` for `AppState` and router composition
- `simulation_worker.rs` for background tick/update loop
- `snapshot.rs` for `Snapshot` and all snapshot assembly helpers
- `terrain.rs` for terrain cache and conversion helpers
- `controls.rs` for mutation endpoints (`place_voxel`, `spawn_civilian`, `damage`, `speed`)
- `saves.rs` for save/load/slot helpers
- `mods/` for catalog, remote registry, publish, upload, fetch, install/unload/reload

Hexagonal pattern:
- Domain/application core: snapshot assembly, terrain and economy calculations, trade/disaster summaries
- Ports: `Simulation`, `SaveDb`, mod archive IO, HTTP requests to remote registries
- Adapters: Axum handlers, filesystem access, HTTP client, SSE responses, JSON serialization

Pure domain logic:
- weather/season math
- `faction_for_point`
- `tech_tree`
- `economy_snapshot`
- `buildings`, `roads`, `trade_routes`
- `apply_trade_routes`
- resource adjustment helpers

Infrastructure:
- all Axum handlers
- save/slot persistence
- remote mod fetch/publish/upload/download
- repository/path validation

Presentation:
- `snapshot_handler`, `terrain_handler`
- SSE event formatting
- API response DTOs used only for dashboard consumption

### `clients/bevy-ref/src/lib.rs`

This is mostly a utility facade, but it still bundles too many concerns.

Recommended split:
- `camera.rs` for `CameraTarget` and orbit math
- `render_state.rs` for `DebugRender`
- `ws.rs` for WS URL helpers and frame parsing
- `chunk.rs` for chunk id math and render-distance helpers
- `minimap.rs` for minimap conversions
- `presentation.rs` for color/ambient/day-night helpers

Hexagonal pattern:
- Pure domain helpers: chunk math, minimap math, color interpolation, frame parsing
- Adapter boundary: environment-variable parsing and URL resolution
- Presentation: clear color, ambient brightness, mesh LOD choices

### `clients/bevy-ref/src/bin/bevy_window.rs`

This is an application bootstrap file that has grown into a small composition root.

Recommended split:
- `args.rs` for CLI/config parsing
- `bootstrap.rs` for window and renderer setup
- `input.rs` for keyboard/mouse interaction
- `scene.rs` for world/mesh setup

Hexagonal pattern:
- App composition root at the edge
- Domain-neutral scene state lives in library modules
- Renderer/window APIs are adapters

### `crates/mod-host/src/lib.rs`

This file likely needs a boundary-oriented split, even though mod hosting is inherently infrastructure-heavy.

Recommended split:
- `policy.rs` for capability and approval rules
- `manifest.rs` for archive/manifest parsing
- `guest.rs` for guest lifecycle and state save/restore
- `determinism.rs` for deterministic checks
- `events.rs` for loaded/unloaded/error event formatting

Hexagonal pattern:
- Core policy and manifest validation at the center
- IO/archive/wasm execution as adapters
- event formatting as presentation at the edge

### `crates/server/src/ws_bridge.rs`

Recommended split:
- `bridge.rs` for bridge state machine
- `transport.rs` for WebSocket framing and send/receive
- `commands.rs` for command intake and dispatch
- `snapshot.rs` for outbound snapshot encoding

Hexagonal pattern:
- Command-processing core
- WebSocket transport adapter
- simulation state as an input/output port

### `crates/agents/src/lib.rs`

Recommended split:
- `citizen.rs` for data and lifecycle
- `needs.rs` for hunger/wardrobe/tools propagation
- `movement.rs` for position and drift
- `spawn.rs` for creation helpers

Hexagonal pattern:
- Domain state and update rules at center
- ECS/world access as an adapter

### `clients/godot-ref/scripts/main.gd`

This script is acting as a full scene controller.

Recommended split:
- `terrain.gd` for terrain loading and mesh generation
- `ui_bindings.gd` for bottom-bar setup and visibility logic
- `input_tools.gd` for tool selection and spawn-drag handling
- `attach_modes.gd` for standalone vs server attachment behavior

Hexagonal pattern:
- Scene state and interaction rules at the center
- Godot nodes, timers, and network client are adapters

Pure domain logic:
- terrain coordinate conversion
- spawn-kind selection rules
- tool visibility gating

Infrastructure:
- node lookups, timers, mesh construction, WS client hookup

Presentation:
- UI state binding and viewport/camera presentation

## Other oversized files

These are over the threshold, but the split priority is lower than the files above.

### `crates/server/tests/ws_smoke.rs`

- Recommendation: split by feature area into `health`, `snapshot`, `control`, `mods`, `save`, and `stream` test modules.
- Pattern: test pyramid at the edge, with shared fixture builders in a small `support.rs`.
- Pure logic: shared assertions and fixture builders.
- Infrastructure: live WebSocket/process bootstrapping and file-system setup.

### `crates/build/src/lib.rs`

- Recommendation: split demand analysis, allocation policy, and graph mutation into separate modules.
- Pattern: pure build policy core with adapter-backed graph mutation.

### `crates/research/src/lib.rs`

- Recommendation: separate research model, progression rules, and scenario integration.
- Pattern: domain model plus thin scenario adapter.

### `crates/engine/src/replay.rs`

- Recommendation: separate event schema, codec, and restore/apply logic.
- Pattern: replay format core plus filesystem adapter.

### `crates/engine/src/scenario.rs`

- Recommendation: separate schema, loader, validator, and conversion into simulation inputs.
- Pattern: DTO/parsing edge around a pure scenario domain model.

### `crates/tactics/src/*` monoliths

Files in this cluster are already somewhat modular, but several are still too large for long-term maintainability.

- `formation.rs`: split formation layout math from search/selection helpers.
- `war_bridge.rs`: split replay/event bridge from tactical state mutation.
- `fog_of_war.rs`: split visibility rules from grid projection.
- `los.rs`: keep geometry helpers pure and isolate any world adapters.
- `pathfinding.rs`: split algorithm core from grid access and heuristics.
- `lib.rs`: keep as a thin facade only.

### Smaller oversized library files

- `crates/economy/src/institution.rs`: split institution model, accounting, and policy effects.
- `crates/save-db/src/lib.rs`: split file-format, filesystem IO, and metadata cataloging.
- `crates/protocol-3d/src/lib.rs`: split wire types from conversion helpers.
- `crates/diffusion/src/lib.rs`: split diffusion math from state orchestration.
- `crates/species/src/lib.rs`: split species data, trait rules, and derived calculations.

## Decomposition priority

If the goal is to reduce monolith risk fast, the order should be:
1. `crates/watch/src/main.rs`
2. `crates/engine/src/engine.rs`
3. `crates/server/src/jsonrpc.rs`
4. `web/dashboard/src/scene3d.tsx`
5. `clients/godot-ref/scripts/main.gd`
6. `crates/server/src/ws_bridge.rs`
7. `crates/mod-host/src/lib.rs`
8. `clients/bevy-ref/src/lib.rs`

The rest can be handled opportunistically once the first four are decomposed.
