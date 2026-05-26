# Client attach matrix (3D line)

Single reference for how each client talks to **civ-server** (WS JSON-RPC + optional F3D0) and **civ-watch** (HTTP terrain / legacy controls).

| Client | Primary attach | Default URL | Terrain | Snapshot / pins | Spawn / place | F3D0 voxels | Spectator default |
|--------|----------------|-------------|---------|-----------------|---------------|-------------|-------------------|
| **Godot** (`clients/godot-ref`) | `attach_mode=server` | WS `ws://127.0.0.1:3000/ws?tick_format=binary` | HTTP `http://127.0.0.1:9090/terrain` | `sim.snapshot` on WS (F3D0-throttled) | WS: `sim.spawn_civilian`, `sim.place_voxel` | **16³ procedural mesh** when dense `voxels` (4096); else chunk markers + throttle | `spectator_mode=true` |
| **Godot** watch mode | `attach_mode=watch` | HTTP `http://127.0.0.1:9090` | Same | SSE / poll via watch | `POST /control/*` when not spectator | — | — |
| **Web dashboard** | civ-watch HTTP + optional WS | `http://127.0.0.1:9090`, dev `5173` | `/terrain` | `/snapshot` or WS | L2 authoring routes | — | `spectator_mode=false` |
| **Bevy window** | civ-server WS | `ws://127.0.0.1:3000/ws?tick_format=binary` | Optional watch HTTP | `sim.snapshot` side-channel | WS spawn RPCs | Binary F3D0 path | N/A (tooling) |
| **Unreal CivShow** | WS + watch HTTP | Same as Godot defaults in `CivShowGameMode` | `UCivProtocolClient` → `/terrain` | `UCivWsClient` → `sim.snapshot` | WS `sim.spawn_*` + HTTP `POST /control/*` | **16³ procedural mesh** when dense `voxels` (4096); else chunk markers (`OnF3d0FrameReceived`) | Editor PIE |
| **civ-server tests** | In-process | `127.0.0.1:3000` | — | JSON-RPC | Full RPC surface | Replay tests | — |

### UX-05 — `spectator_mode` defaults

ADR-009: web is **L2 authoring by default**; Godot is **spectator by default** (WorldBox-style observer until Inspector toggle). Use explicit overrides for demos so both clients match intent.

| Client | Default | How to override | When authoring enabled | Hidden / disabled tools |
|--------|---------|-----------------|------------------------|-------------------------|
| **Godot** | `spectator_mode=true` on `Main` | Inspector → `spectator_mode=false` | Place Voxel, Spawn palette, Damage | `MUTATION_TOOLS` in `main.gd` |
| **Web dashboard** | Authoring on (`readOnly=false`) | `?spectator=1` or `?authoring=0` | Spawn, place voxel, speed (L2) | `resolveAuthoringEnabled()` in `attachConfig.ts` |
| **Bevy window** | N/A (agent/CI tooling) | — | WS spawn RPCs in standalone | — |
| **Unreal CivShow** | PIE / editor session | HTTP `POST /control/*` + WS in game mode | Same spawn palette as Godot when not gated in C++ | Project-specific |
| **civ-watch built UI** | Served at `:9090` | Same query params as web when embedded | `POST /control/*` | — |

**Demo tip:** Godot F5 is read-only until you flip `spectator_mode`. Web at `http://127.0.0.1:5173` is authoring unless you add `?spectator=1`.

## UX-02 — Spawn palette (`kind` → transport)

Wire labels match [`jsonrpc-surface.md`](../api/jsonrpc-surface.md): `civilian` uses `sim.spawn_civilian`; all other kinds use `sim.spawn_entity` with `{ "kind", "x", "y", "faction"? }`.

| `kind` | civ-server WS | Godot server | Godot watch | Web L2 (server) | Web L2 (watch) | Bevy window | Unreal CivShow |
|--------|---------------|--------------|-------------|-----------------|----------------|-------------|----------------|
| `civilian` | `sim.spawn_civilian` | WS | `POST /control/spawn_entity` | `sim.spawn_civilian` | `POST /control/spawn_entity` | WS (tooling) | WS + HTTP |
| `vehicle` | `sim.spawn_entity` | WS | `POST /control/spawn_entity` | `sim.spawn_entity` | `POST /control/spawn_entity` | WS | WS + HTTP |
| `airport` | `sim.spawn_entity` | WS | `POST /control/spawn_entity` | `sim.spawn_entity` | `POST /control/spawn_entity` | WS | WS + HTTP |
| `port` | `sim.spawn_entity` | WS | `POST /control/spawn_entity` | `sim.spawn_entity` | `POST /control/spawn_entity` | — | WS + HTTP |
| `hangar` | `sim.spawn_entity` | WS | `POST /control/spawn_entity` | `sim.spawn_entity` | `POST /control/spawn_entity` | — | WS + HTTP |

Unreal: `UCivWsClient::SpawnEntity` / `UCivProtocolClient::SpawnEntity` — HTTP path is primary in PIE today; WS mirrors Godot routing (`civilian` → dedicated RPC).

### UX-04 — Minimap

| Client | Status |
|--------|--------|
| Bevy | 160×160 chunk dots; click-to-focus — [`minimap-conventions.md`](minimap-conventions.md) |
| Godot | 128×128 terrain texture + orbit dot |
| Web dashboard | Terrain preview canvas; no click-to-focus yet |
| **Unreal CivShow** | **Partial** — `ACivMinimapCapture` (256² ortho at ~(64,800,64), width 512) + `UCivMinimapWidget` UMG; **left-click** → UV → world XZ → `ACivShowGameMode::FocusCameraAtWorldLocation` |

## Services to start (local demo)

```powershell
# Terminal 1 — simulation authority
cargo run -p civ-server

# Terminal 2 — terrain + HTTP dashboard host
cargo run -p civ-watch

# Terminal 3 — client of choice (Godot F5, Bevy window, Unreal PIE, or web dev server)
```

## Environment overrides

| Variable | Default | Used by |
|----------|---------|---------|
| `CIV_SERVER_PORT` | `3000` | `civ-server` WS |
| `CIV_WATCH_PORT` | `9090` | `civ-watch` HTTP |
| `UE_ROOT` | auto-detect `UE_5.7` | `clients/unreal-show/scripts/build.ps1` |

## Parity gaps (see maturity audit)

- `civ_pins.job` may be `null` until spawn sets `Citizen.job` (Unreal `CivisJobColors` ready when wired).
- F3D0 mesh: Bevy full `Frame3d`; Godot/Unreal **16³ procedural mesh** when `deltas[].voxels` has 4096 ids, else chunk markers.

## Related docs

- [`fr-godot-attach.md`](../development-guide/fr-godot-attach.md)
- [`fr-unreal-agent-playbook.md`](../development-guide/fr-unreal-agent-playbook.md)
- [`fr-ax-dx-ux-maturity-audit.md`](../development-guide/fr-ax-dx-ux-maturity-audit.md)
- [`minimap-conventions.md`](minimap-conventions.md)
