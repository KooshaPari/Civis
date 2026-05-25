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

## F3D0 tick throttle (server attach)

Godot does **not** render voxels from F3D0 today; it uses F3D0 (and legacy text tick shapes) only to **rate-limit** `sim.snapshot` refreshes for pins, buildings, and `is_day`. Voxel presentation stays on civ-watch terrain HTTP until a live F3D0 mesh path lands (Bevy-first).

| Convention | Godot | Bevy (`bevy-ref`) | Unreal (`CivWsClient`) |
|------------|-------|-------------------|------------------------|
| WS URL | `?tick_format=binary` | Same | Same (`CivShowGameMode`) |
| Binary magic | `F3D0` (4-byte ASCII) | `F3D0` via `parse_ws_payload` | `F3D0` in `HandleBinary` |
| On F3D0 / tick | `_maybe_refresh_snapshot()` | Decode `Frame3d` for voxels **and** poll snapshot meta | `RequestSnapshot()` throttle only |
| Snapshot throttle | `snapshot_throttle_ms` = **250** | Side-channel `sim.snapshot` every **2 s** (`SNAPSHOT_POLL_SECS`) for `is_day` / tick | `SnapshotThrottleSec` = **0.25** |
| After spawn/place RPC | Immediate `request_snapshot()` | N/A (Bevy uses poll + F3D0) | Immediate `RequestSnapshot()` on RPC ack |
| Text tick fallback | `VoxelDelta` / `BuildingDiff` / `AgentAppearance` also throttles snapshot | Skipped when `prefer_binary` | Same fields in `HandleMessage` |

Implementation: `clients/godot-ref/scripts/civis_ws_client.gd` (`_handle_packet`, `_maybe_refresh_snapshot`). Bevy decode tests: `clients/bevy-ref/src/lib.rs` (`parse_f3d0_frame`, `ws_prefer_binary`). Cross-client minimap UV rules (orthogonal): [`minimap-conventions.md`](../guides/minimap-conventions.md).

**Agent note:** Do not lower throttle below 250 ms on Godot without measuring WS RPC load; match Unreal’s 0.25 s constant when changing either client.
