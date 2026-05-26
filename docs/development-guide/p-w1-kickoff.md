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
| FR-CIV-TACTICS-058 | implemented | `.civsave/` folder stub (`CivSaveBundle`) |
| FR-CIV-TACTICS-059 | implemented | `civis-3d-mod-package-all` for example mods |

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
17. **Next:** Compressed `.civsave.zst`, full float data-flow trace, in-game mod install.

## Run

```bash
cargo test -p civ-tactics
cargo test -p civ-engine pending_damage
cargo test -p civ-engine war_bridge_records
just civis-3d-verify
```
