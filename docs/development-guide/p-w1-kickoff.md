# P-W1 tactical warfare — kickoff

**Phase:** P-W1 (`crates/tactics`)
**Depends on:** P-V1 (voxel), P-A1 (agents)
**Branch suggestion:** `feat/p-w1-tactics` off `main` after #296 merges

## Already wired

| Link | Location |
|------|----------|
| Voxel damage | `civ_tactics::apply_damage` used in `crates/engine/src/engine.rs` tick + `apply_damage_now` |
| Replay | `DamageEvent` + `ReplayEvent::Combat` in `crates/engine/src/replay.rs` |
| Authoring | `sim.damage` (server), `POST /control/damage` (watch), web/Godot damage tool |
| Doctrine GA | `evolve_doctrine` + tests `FR-CIV-TACTICS-010/011` |

## FR status (`docs/traceability/fr-3d-matrix.md`)

| FR ID | Status | Next step |
|-------|--------|-----------|
| FR-CIV-TACTICS-000 | implemented | — |
| FR-CIV-TACTICS-001 | implemented | Voxel sphere damage + per-soldier pins |
| FR-CIV-TACTICS-010 | implemented | — |
| FR-CIV-TACTICS-020 | implemented | `line_of_sight` (voxel LOS) |
| FR-CIV-TACTICS-021 | implemented | `formation_offsets` (line / wedge / square) |
| FR-CIV-TACTICS-022 | implemented | `tick_war_bridge` in `phase_military` |
| FR-CIV-TACTICS-023 | implemented | `score_doctrine_fitness` before GA evolve |
| FR-CIV-TACTICS-024 | implemented | `CombatEngagement` + `unit_a`/`unit_b` on snapshot |
| FR-CIV-TACTICS-025 | implemented | `ReplayEvent::Combat` in replay log |
| FR-CIV-TACTICS-030 | implemented | `OperationalLayer` hook |
| FR-CIV-TACTICS-031 | implemented | `tick_operational_movement` toward enemies |
| FR-CIV-TACTICS-032 | implemented | `MilitaryUnit::hp` / `max_hp` on ECS |
| FR-CIV-TACTICS-033 | implemented | `bfs_next_step` pathfinding |
| FR-CIV-TACTICS-034 | implemented | `ModHost::military_tick` / `read_military` stub |
| FR-CIV-TACTICS-035 | implemented | movement cadence 4, war cadence 16, 2 movement pulses |
| FR-CIV-TACTICS-025-int | implemented | `replay_combat_events_restore_pending_damage` |
| FR-CIV-TACTICS-036 | implemented | voxel `grid_cell_blocked` + BFS/A* obstacles |
| FR-CIV-TACTICS-037 | implemented | `astar_path` obstacle-aware routing |
| FR-CIV-TACTICS-038 | implemented | `civlab_military_tick` WASM export + host invoke |
| FR-CIV-TACTICS-025-int2 | implemented | `replay_combat_drains_to_same_voxel_state_as_live` |
| FR-CIV-TACTICS-025-int3 | implemented | `replay_combat_log_deterministic_for_seed_rerun` |
| FR-CIV-TACTICS-039 | implemented | `grid_cell_impassable` + occupied-cell path blocking |
| FR-CIV-TACTICS-040 | implemented | `invoke_military_tick(wasm, sim_tick)` capability API |
| FR-CIV-TACTICS-041 | implemented | combat payloads in replay hash chain |
| FR-CIV-TACTICS-042 | implemented | fog-of-war gating in `tick_war_bridge` |
| FR-CIV-TACTICS-043 | implemented | Ed25519 `mod.wasm.sig` verification |
| FR-CIV-TACTICS-044 | implemented | policy/military tick capability API + SDK version |
| FR-CIV-TACTICS-045 | implemented | scenario `fog_vision_radius` wires military phase |
| FR-CIV-TACTICS-046 | implemented | `civlab_economy_tick` WASM + `ModHost::economy_tick` |
| FR-CIV-TACTICS-047 | implemented | `civlab::capability_api_version` host import |
| FR-CIV-TACTICS-048 | implemented | `mods/example-economic` + economy WASM tick test |
| FR-CIV-TACTICS-049 | implemented | `civlab::memory_read` / `memory_write` host imports |
| FR-CIV-TACTICS-050 | implemented | scenario `military:` cadence/range overrides |
| FR-CIV-TACTICS-051 | implemented | `baseline.yaml` loads `mods/example-economic` |
| FR-CIV-TACTICS-052 | implemented | per-mod guest memory snapshots on `ModHost` |
| FR-CIV-TACTICS-053 | implemented | `civlab::sim_tick` + `HOST_CAPABILITY_IMPORTS` |
| FR-CIV-TACTICS-054 | implemented | mod browser on watch/server snapshot + dashboard |
| FR-CIV-TACTICS-055 | implemented | `ModGuestStateSave` JSON export/import |
| FR-CIV-TACTICS-056 | implemented | WASM determinism scan at mod load |
| FR-CIV-TACTICS-057 | implemented | float opcode count in determinism report |
| FR-CIV-TACTICS-061 | implemented | action_emit float data-flow trace |
| FR-CIV-TACTICS-058 | implemented | `.civsave/` folder stub (`CivSaveBundle`) |
| FR-CIV-TACTICS-059 | implemented | `civis-3d-mod-package-all` for example mods |
| FR-CIV-TACTICS-060 | implemented | `.civsave.zst` compressed archive (`save_archive` / `load_archive`; civ-watch default) |
| FR-CIV-TACTICS-062 | implemented | mod catalog + `POST /control/mods/install` (civ-watch + dashboard) |
| FR-CIV-TACTICS-063 | implemented | `POST /control/mods/unload` + `mod.unloaded.v1` bus JSON |
| FR-CIV-TACTICS-064 | implemented | `POST /control/mods/upload` → `mods/uploads/*.civmod` |
| FR-CIV-TACTICS-065 | implemented | production slots `slot-1`..`slot-5` + autosave ring (10) |
| FR-CIV-TACTICS-066 | implemented | `save.slot` / `save.load` / `save.list` JSON-RPC (civ-server) |
| FR-CIV-TACTICS-067 | implemented | mod publish store `mods/publish` + HTTP API |
| FR-CIV-TACTICS-068 | implemented | mod hot reload `POST /control/mods/reload` |
| FR-CIV-TACTICS-069 | implemented | session-scoped SQLite save metadata (`civ-save-db`) |
| FR-CIV-TACTICS-070 | implemented | remote mod fetch cache `mods/remote` + HTTP API |
| FR-CIV-TACTICS-071 | implemented | CIV-0700 `world_read` / `action_emit` capability enforcement |
| FR-CIV-TACTICS-072 | implemented | `session.saved.v1` on replay bus + snapshot feed |
| FR-CIV-TACTICS-073 | implemented | web remote mod fetch UI (`GET/POST` mods/remote) |
| FR-CIV-TACTICS-074 | implemented | `PolicyMod` trait surface in civlab-sdk (CIV-0700 §5) |
| FR-CIV-TACTICS-075 | implemented | `mod.permission_violation.v1` on replay bus + snapshot/SSE |
| FR-CIV-TACTICS-076 | implemented | civ-server session-scoped `SaveDb` on `save.slot` |
| FR-CIV-TACTICS-077 | implemented | signed remote mod registry (`mods/remote-registry.json`) |
| FR-CIV-BEVY-001 | implemented | `civ-standalone` gameplay plugins (sim bridge, HUD, spawn tools, minimap) |
| FR-CIV-BEVY-002 | implemented | live attach scene sync (`live_scene`: voxel chunks + agent markers from `Frame3d`) |
| FR-CIV-BEVY-014 | implemented | `live_stream` unit tests (colors, ground Y, voxel delta apply) |
| FR-CIV-BEVY-021 | implemented | GitHub Actions `.github/workflows/civis-3d-live-smoke.yml` (headless `just civis-3d-live-smoke`) |
| FR-CIV-BEVY-025 | implemented | `live_pick::` ray–AABB unit tests in `just civis-3d-live-smoke` (item 50) |
| FR-CIV-BEVY-026 | implemented | `CIV_BEVY_BACKEND` native GPU selection + `native_backend` unit tests; README + `wgpu-native-escape-hatches.md` cross-link (item 51) |
| FR-CIV-BEVY-029 | implemented | `Frame3d::EventFeed` → `live_stream::apply_event_feed_frame` egui toasts; `live_scene` + `live_attach` connection toasts; `bevy_window` logs (item 54) |
| FR-CIV-BEVY-030 | implemented | `CivilianState` / `FactionState` wire frames → HUD population + faction chips (item 55) |
| FR-CIV-BEVY-031 | implemented | `pbr-textures` feature + `materials.rs` biome loader; sandbox-only `BiomeMaterialsPlugin` on `civ-standalone` (item 56) |
| FR-CIV-BEVY-032 | implemented | `just civis-3d-live-smoke` / CI: `event_feed::`, `menus::`, `civ-protocol-3d` `civilian_state` + `event_feed` round-trips (item 57) |
| FR-CIV-BEVY-033 | implemented | live pick → egui inspector from `CivilianState` wire entries; `game_ui::` formatting helpers (item 58) |
| FR-CIV-BEVY-034 | implemented | `DiplomacyUiPlugin` on `civ-standalone`; `FactionState` → `DipFaction` + neutral relations matrix; **G** toggles panel (item 59) |
| FR-CIV-BEVY-035 | implemented | F3D0 encode/decode round-trip for all six `Frame3d` kinds; `parse_ws_payload` decodes each kind (item 60) |
| FR-CIV-BEVY-037 | implemented | `just civis-3d-live-smoke` gates optional features: `gpu_features::` (bevy) + `materials::` (`pbr-textures`; headless, no GPU) (item 62) |

## First PR slice (recommended)

1. **Test:** `engine::tick` with queued `DamageEvent` reduces voxel count — **done**.
2. **Server:** `sim.snapshot` damage fields — **done**.
3. **Web / Watch:** combat UX — **done**.
4. **Doctrine GA** — **done**.
5. **LOS / formations / war bridge** — **done**.
6. **Per-soldier combat + doctrine fitness + operational hook** — **done** (#300).
7. **Movement + HP + replay combat** — **done** (#301).
8. **Pathfinding + more work/tick + replay combat + military mod hook** — **done** (item 9).
9. **Obstacle pathfinding + replay combat determinism + military WASM** — **done** (item 10).
10. **Occupied-cell blocking + military WASM tick API + combat hash chain** — **done** (item 11).
11. **Fog in war bridge + mod signing + WASM capability surface** — **done** (item 12).
12. **Scenario fog + economic WASM + capability host imports** — **done** (item 13).
13. **Example economic mod + memory imports + scenario military tuning** — **done** (item 14).
14. **Baseline economic mod + memory snapshots + capability API** — **done** (item 15).
15. **Guest memory save/load + mod browser + determinism scan** — **done** (item 16).
16. **CIV-1000 civsave folder + float scan report + mod packaging** — **done** (item 17).
17. **Compressed `.civsave.zst` save archives** — **done** (item 18a).
18. **action_emit float data-flow trace** — **done** (item 18b).
19. **In-game mod install** — **done** (item 18c: catalog + install API).
20. **Save/mod distribution** — **done** (item 21): slot-1..5, autosave ring, signed `.civmod` upload, mod unload.
21. **Save/mod distribution v2** — **done** (item 22): `save.slot` / `save.load` / `save.list` on civ-server; web server slot UI; `mods/publish` + publish API; `POST /control/mods/reload`.
22. **Save/mod distribution v3** — **done** (item 23): `civ-save-db` session metadata; `POST /control/mods/fetch` remote cache; CIV-0700 capability enforcement stubs.
23. **Session bus + remote UI + PolicyMod** — **done** (item 24): `session.saved.v1` on replay bus + SSE; dashboard remote fetch/cache UI; `world_read`/`action_emit` capability enforcement + `PolicyMod` trait in civlab-sdk.
24. **Permission bus + server save DB + signed registry** — **done** (item 25): `mod.permission_violation.v1` on replay bus + `sim.snapshot`; civ-server `SaveDb` on `save.slot`; `mods/remote-registry.json` allowlist for remote fetch.
25. **Bevy gameplay client** — **done** (item 26): export gameplay modules in `civ-bevy-ref`, `civ-standalone` with `bevy,egui`, sim tick + spawn-tool → `Simulation`, HUD/minimap plugins.
26. **Live WS attach + render-to-texture minimap** — **done** (item 27): `civ-standalone` server attach via `CIVIS_ATTACH=server` / `CIV_WS_URL`, `LiveAttachPlugin`, RTT minimap camera.
27. **Live attach scene sync** — **done** (item 28): `live_scene` applies `Frame3d::VoxelDelta` / `AgentAppearance`; server mode skips sandbox terrain; minimap dots from live chunks/agents.
28. **Building-diff + agent positions** — **done** (item 29): protocol `WorldXZ` / `BuildingDiffEntry`; civ-server fills frames; `live_scene` renders buildings and uses `agent_world_translation`.
29. **Live scene focus + provenance styling** — **done** (item 30): `LiveSceneFocus` drives camera rig + minimap ortho; procedural vs freehand building materials and minimap dots.
30. **Terrain anchoring for live entities** — **done** (item 31): `terrain_surface_y` snaps streamed agents/buildings to procedural height in `live_scene`.
31. **BuildingGraph on live attach** — **done** (item 32): optional `BuildingDiffFrame.graph` from civ-server; `live_scene` renders parcels with facade/provenance styling.
32. **Voxel-surface anchoring** — **done** (item 33): cached `chunk_voxels` + column sampling in `live_scene`; falls back to `terrain_surface_y`.
33. **Live attach smoke harness** — **done** (item 34): `just civis-3d-live-smoke` (F3D0 + voxel ground + standalone check); `just civis-3d-standalone-live-url URL=ws://…` for remote civ-server.
34. **Shared live ground + bevy_window parity** — **done** (item 35): `live_ground` module; `bevy_window` caches voxels, anchors agents/buildings, applies `BuildingDiff`.
35. **Shared live_stream frame apply** — **done** (item 36): `live_stream` module dedupes `LiveScenePlugin` and `bevy_window`; graph parcels render in both paths.
36. **bevy_window live minimap parity** — **done** (item 37): HUD minimap shows streamed agents, buildings, graph parcels, and camera position (matches `live_scene`).
37. **Shared live_minimap module** — **done** (item 38): `live_minimap` dedupes dot layout, UV mapping, colors, and spawn helpers for `live_scene` and `bevy_window` (`FR-CIV-BEVY-013`).
38. **live_stream unit tests** — **done** (item 39): `#[cfg(test)]` in `live_stream.rs` — color/provenance helpers, parcel→kind mapping, `live_ground_y` offsets, minimal `VoxelDeltaFrame` apply via Bevy `World`.
39. **bevy_window live scene focus** — **done** (item 40): shared `live_focus`; orbit centre lerps to streamed bounds when WS connected; minimap UV + click pan use focus rect (FR-CIV-BEVY-015).
40. **Live attach smoke harness v2** — **done** (item 41): `just civis-3d-live-smoke` runs `live_stream::`, minimap UV lib tests, and both Bevy bins (`FR-CIV-BEVY-016`).
41. **Live attach HUD scene stats** — **done** (item 42): `LiveHudSnapshot` overlay shows tick, connection, C/A/B/G counts, optional `sim.snapshot` RTT; wired in `civ-bevy-window` and `civ-standalone` server attach (`FR-CIV-BEVY-017`).
42. **WebSocket reconnect backoff + HUD connection state** — **done** (item 43): exponential reconnect backoff (1s→30s cap) in `ws_client`; HUD shows `connected` / `reconnecting` / `disconnected` via `WsConnectionState` (`FR-CIV-BEVY-018`).
43. **Live stream entity pick** — **done** (item 44): `live_pick` ray–AABB pick on left-click (skip orbit drag + minimap); HUD `sel: agent #N`; `civ-standalone` server attach via `LivePickPlugin` (`FR-CIV-BEVY-019`).
44. **bevy_window day/night sync** — **done** (item 45): `sim.snapshot` `is_day` drives `DayNightCycle` + `update_lighting` (sun/moon/clear/ambient parity with `live_attach`); web blend via `presentation_day_factor_target` (`FR-CIV-BEVY-020`).
46. **GitHub Actions live-smoke CI** — **done** (item 46): `.github/workflows/civis-3d-live-smoke.yml` runs `just civis-3d-live-smoke` on path-filtered PR/push to `clients/bevy-ref`, `crates/server`, `crates/protocol-3d`, or `justfile` (`FR-CIV-BEVY-021`).
47. **Live attach smoke harness v3** — **done** (item 47): `just civis-3d-live-smoke` runs `live_focus::` and `live_minimap::` lib tests (`FR-CIV-BEVY-022`).
48. **Event feed HUD (egui toasts + log)** — **done** (item 48): `EventFeedPlugin` on `civ-standalone`; bottom-right toasts + **L** toggles scrollable log; `live_attach` pushes `EventKind::System` on connection changes (`FR-CIV-BEVY-023`).
49. **Pause / settings menus** — **done** (item 49): `MenusPlugin` on `civ-standalone` — Escape pause overlay, settings stub, era banner; sim tick gated while paused (`FR-CIV-BEVY-024`).
50. **Live attach smoke harness v4** — **done** (item 50): `just civis-3d-live-smoke` runs `live_pick::` lib tests (`FR-CIV-BEVY-025`).
51. **Native GPU backend env + tests** — **done** (item 51): `CIV_BEVY_BACKEND` (`dx12` \| `vulkan` \| `metal`); Windows defaults DX12 \| Vulkan; `native_backend` unit tests + README; cross-link `wgpu-native-escape-hatches.md` (`FR-CIV-BEVY-026`).
52. **Live attach polish integration** — **done** (item 52): `civ-bevy-window` wires `LivePickPlugin`, minimap `MinimapRoot`, HUD `selected_live`; `live_attach` mirrors selection; smoke/README cover `live_focus` + `live_minimap` tests.
53. **Server F3D0 civilian/faction/event broadcast** — **done** (item 53): `build_frame_bundle` emits six `Frame3d` variants per tick (civilian/faction/event from sim snapshot + lifecycle stub); `ws_smoke` expects full bundle (`FR-CIV-BEVY-028`).
54. **EventFeed F3D0 → egui toasts** — **done** (item 54): `live_stream::apply_event_feed_frame` maps wire feed messages to `EventFeed` toasts; `live_scene` applies on server attach; `live_attach` connection toasts; `bevy_window` logs (`FR-CIV-BEVY-029`).
55. **Civilian/faction HUD from wire frames** — **done** (item 55): `live_stream` merges `CivilianState` / `FactionState` into HUD counts; `GameUiSnapshot` + `LiveHudSnapshot` overlay `P`/`F`; `live_scene`, `live_attach`, `civ-bevy-window` apply frames (`FR-CIV-BEVY-030`).
56. **PBR biome materials feature** — **done** (item 56): `pbr-textures` cargo feature; `materials.rs` + `BiomeMaterialsPlugin` on `civ-standalone` sandbox only; `terrain::pbr_biome_at_height` height-band material assignment; README asset/LICENSE paths (`FR-CIV-BEVY-031`).
57. **Live attach smoke harness v5** — **done** (item 57): `just civis-3d-live-smoke` runs `event_feed::`, `menus::`, and `civ-protocol-3d` `civilian_state` / `event_feed` round-trip tests (`FR-CIV-BEVY-032`).
59. **Diplomacy panel from faction frames** — **done** (item 59): `diplomacy_ui` exported; `live_stream::sync_diplomacy_from_faction_frame`; `live_attach` sync on server attach; `DiplomacyUiPlugin` on `civ-standalone`; `diplomacy_ui::` tests in smoke (`FR-CIV-BEVY-034`).
60. **F3D0 round-trip all frame kinds** — **done** (item 60): `frame_bundle_binary_roundtrip_all_kinds` in `civ-protocol-3d`; `parse_ws_payload_decodes_all_frame_kinds` in `civ-bevy-ref`; `just civis-3d-live-smoke` runs both (`FR-CIV-BEVY-035`).
61. **GPU capabilities in pause Settings** — **done** (item 61): `MenusPlugin` settings panel reads `Option<Res<GpuCapabilities>>` — backend, est. VRAM, ray tracing / DLSS / FSR (read-only); unavailable message when missing (`FR-CIV-BEVY-036`).
62. **Live attach smoke optional features** — **done** (item 62): `just civis-3d-live-smoke` runs `gpu_features::` (`bevy`) and `materials::` (`pbr-textures`; pure lib tests, no render window) (`FR-CIV-BEVY-037`).

## Run

```bash
cargo test -p civ-tactics
cargo test -p civ-engine pending_damage
cargo test -p civ-engine war_bridge_records
cargo check -p civ-bevy-ref --features bevy,egui --bin civ-standalone
just civis-3d-live-smoke
just civis-3d-verify
```
