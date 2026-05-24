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
| FR-CIV-UX-002 | `sim.spawn_civilian` (or batch) on **civ-server** JSON-RPC | Server command handler + ECS spawn |
| FR-CIV-UX-003 | `sim.place_voxel` / build graph write on server | P-V2 + protocol |
| FR-CIV-UX-004 | Drag-place vehicles / airports / ports | Asset pipeline + build schema |
| FR-CIV-UX-005 | Era timelapse **camera** presets (not only tick label) | Godot scene tooling |
| FR-CIV-UX-006 | Spawn palette UI (species, faction, loadout) | P-G1 genetics API on wire |

## Authoring today

1. Set `spectator_mode = false` on `World` (Godot Inspector).
2. Set `attach_mode = watch` for mutations (server has no spawn/voxel RPC yet).
3. Run `civ-watch` + optional `civ-server` for the same seed (pins diverge until RPC lands).

## Run

```bash
cargo run -p civ-server
cargo run -p civ-watch
# Godot F5 — attach_mode server for observe; watch for author
```
