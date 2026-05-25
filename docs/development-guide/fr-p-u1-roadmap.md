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

## Next (P-U1 backlog)

| FR ID (proposed) | Requirement | Blocker |
|------------------|-------------|---------|
| FR-CIV-UX-002 | `sim.spawn_civilian` on **civ-server** JSON-RPC | **implemented** — `sim.spawn_civilian`; Godot `CivisWsClient` |
| FR-CIV-UX-003 | `sim.place_voxel` on server | **implemented** |
| FR-CIV-UX-004 | Drag-place vehicles / airports / ports | **partial** — click-place via `sim.spawn_entity`; drag TBD |
| FR-CIV-UX-005 | Era timelapse **camera** presets | **implemented** — `camera.gd` + web dashboard wide/close/orbit |
| FR-CIV-UX-006 | Spawn palette (civilian / vehicle / airport) | **implemented** — `sim.spawn_entity`, watch `/control/spawn_entity` |
| FR-CIV-UX-006 | Spawn palette UI | **implemented** — `sim.spawn_entity` (vehicle/airport) + web palette on civ-server attach |
| — | `sim.damage` on server | **implemented** — web + Godot server attach |

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
