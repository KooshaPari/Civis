# FR-CIV-GODOT-ATTACH — Godot client backends

**ADR:** [ADR-007](../adr/ADR-007-three-renderers.md), [ADR-009](../adr/ADR-009-web-client-strategy.md)  
**Implementation:** `clients/godot-ref/scripts/`

| FR ID | Requirement | Acceptance |
|-------|-------------|------------|
| FR-CIV-GODOT-ATTACH-000 | Default attach `civ-server` WebSocket JSON-RPC | `attach_mode=server`, `CivisWsClient` connects, `health` + `sim.snapshot` succeed |
| FR-CIV-GODOT-ATTACH-001 | `sim.set_speed` drives server tick loop | Speed option calls RPC; civilians move without manual tick |
| FR-CIV-GODOT-ATTACH-002 | Terrain from civ-watch while on server attach | `GET /terrain` via `CivisClient` after WS connect |
| FR-CIV-GODOT-ATTACH-003 | F3D0 tick triggers throttled `sim.snapshot` | Binary `F3D0` or text `Frame3d` → snapshot refresh ≤250ms |
| FR-CIV-GODOT-ATTACH-004 | `attach_mode=watch` preserves HTTP/SSE path | `civ-watch` timer snapshot + `POST /control/*` when not spectator |
| FR-CIV-GODOT-UX-000 | N spawns → N entity events | `ux::spawn_emits_entity_events` (Rust unit test) |
| FR-CIV-UX-002 | Server spawn via WS | `CivisWsClient.spawn_civilian` → `sim.spawn_civilian` |
| FR-CIV-UX-003 | Server voxel write via WS | `CivisWsClient.place_voxel` → `sim.place_voxel` |

## Run

```bash
cargo run -p civ-server    # :3000 /ws
cargo run -p civ-watch     # :9090 terrain (required for heightmap)
# Godot F5 — attach_mode server (default)
```

## Inspector

| Property | Default | Meaning |
|----------|---------|---------|
| `attach_mode` | `server` | `server` or `watch` |
| `civ_server_ws` | `ws://127.0.0.1:3000/ws?tick_format=binary` | JSON-RPC + F3D0 |
| `civ_watch_http` | `http://127.0.0.1:9090` | Terrain + watch-mode controls |
| `spectator_mode` | `true` | Hide Place/Spawn/Damage |
