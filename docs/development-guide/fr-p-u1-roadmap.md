# FR-CIV-UX — P-U1 WorldBox UX roadmap

**Phase:** P-U1 (`clients/godot-ref`)  
**Depends on:** P-A1 (agents), P-V2 (build substrate)

## Implemented

| FR ID | Summary | Evidence |
|-------|---------|----------|
| FR-CIV-UX-000 | N spawns → N entity events | `ux::spawn_emits_entity_events` |
| FR-CIV-UX-001 | Timelapse speeds without divergence | `ux::timelapse_no_divergence` |
| FR-CIV-GODOT-ATTACH-000..004 | civ-server WS + civ-watch terrain | `civis_ws_client.gd`, `main.gd` |
| — | Buildings + job-colored pins in 3D | `main.gd` `_sync_buildings`, `_sync_civilians` |
| — | Era label in HUD | `era_timelapse.gd`, `EraLabel` |
| FR-CIV-UX-004 | Drag-place + convoy along path (vehicle/airport/port) | `ux::convoy_positions`, `main.gd`, `scene3d.tsx`, `spawnConvoy.ts` |
| FR-CIV-UX-006 | Spawn palette: civilian, vehicle, airport, port | `spawn.rs`, `jsonrpc.rs`, `watch` `/control/spawn_entity`, Godot + web bottom bar |

## Post P-U1 slice (landed)

| Item | Evidence |
|------|----------|
| Unreal WS | `CivWsClient.cpp`, `CivShowGameMode` dual HTTP+WS attach + `ApplyDayNight` |
| Bevy minimap | `bevy_window.rs` click-to-focus; `ws_client` `sim.snapshot` meta → `is_day` lighting |
| Agents ↔ spectator | `spectator.rs` `civ_pins` from `Position3d`; `ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot` |
| L5 (scoped) | `fr-l5-visual-pass.md`; Godot capsules + `SpawnBurst`; Bevy/Unreal/Godot day-night |

## Authoring today

1. Set `spectator_mode = false` on `World` (Godot Inspector).
2. Default `attach_mode = server` — spawn/place via `sim.spawn_civilian` / `sim.place_voxel` (same timeline as web spectator).
3. Run `civ-server` + `civ-watch` (terrain HTTP). Use `attach_mode = watch` only for legacy HTTP controls.

## Run

```bash
cargo run -p civ-server
cargo run -p civ-watch
# Godot F5 — attach_mode server for observe; watch for author
```
