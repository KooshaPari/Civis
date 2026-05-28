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
33. **Next:** (TBD — follow kickoff / FR matrix).

## Run

```bash
cargo test -p civ-tactics
cargo test -p civ-engine pending_damage
cargo test -p civ-engine war_bridge_records
cargo check -p civ-bevy-ref --features bevy,egui --bin civ-standalone
just civis-3d-verify
```
