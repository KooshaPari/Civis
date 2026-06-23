# Civis Godot Ref

Civis 3D reference client for Godot 4. Per `docs/adr/ADR-007-three-renderers.md`,
this is the UX iteration surface for the WorldBox-style spawn editor.

**Default:** `spectator_mode = true` on `Main` (ADR-009) — read-only view; set `spectator_mode = false` in the Inspector for Place Voxel / Spawn authoring (watch attach only).

**Default attach:** `attach_mode = server` — WebSocket JSON-RPC to `civ-server` for live ticks and `sim.snapshot`; terrain still from `civ-watch` HTTP.

## Authoring paths

Two layers: **GDScript** (attach, UI, presentation) and **GDExtension** (HTTP terrain mesh + WS frame helpers). Both are required for the default server attach demo.

| Path | Location | Responsibility |
|------|----------|----------------|
| **Scripts-only** | `scripts/civis_ws_client.gd`, `main.gd`, `camera.gd`, … | WS JSON-RPC to civ-server (`health`, `sim.snapshot`, `sim.spawn_civilian`, `sim.place_voxel`); F3D0/text tick → throttled snapshot; `spectator_mode`; capsule civilians, job colors, `SpawnBurst`, foot Y via `_world_y_at_norm` |
| **GDExtension (Rust)** | `rust/src/lib.rs`, `rust/src/ws_frame.rs` | **`CivisClient`** — sync HTTP to civ-watch (`fetch_terrain`, biome/height vertex colors); **`CivisWsFrame`** — decode binary WS payloads (`F3D0` magic, JSON-RPC envelopes, legacy text `Frame3d`) for `civis_ws_client.gd` |

**Rebuild Rust DLL after protocol or terrain API changes:**

```powershell
cd clients/godot-ref/rust
cargo build
# release: cargo build --release
```

Godot loads `res://rust/target/debug/civis_godot_rust.dll` (see `civis.gdextension`). Or from repo root: `just civis-3d-godot`.

**Watch-only dev (no extension rebuild for WS):** set `attach_mode = watch` — terrain and controls use HTTP from GDScript + `CivisClient`; no civ-server WS.

Spec: [`docs/development-guide/fr-godot-attach.md`](../../docs/development-guide/fr-godot-attach.md), [`docs/guides/client-attach-matrix.md`](../../docs/guides/client-attach-matrix.md).

## Run

1. Install Godot 4.3+
2. `cd clients/godot-ref/rust && cargo build`
3. From repo root:
   ```bash
   cargo run -p civ-server   # :3000 /ws
   cargo run -p civ-watch    # :9090 terrain (required for heightmap)
   ```
4. Open `clients/godot-ref/project.godot` in Godot 4
5. Press F5 — default connects to **civ-server** WS + **civ-watch** terrain

For HTTP-only dev (no civ-server): set **Attach Mode** to `watch` on the `World` node.

## Camera controls

Orbit camera on `Camera3D` (`scripts/camera.gd`):

| Input | Action |
|-------|--------|
| **Right-drag** | Rotate around terrain centre |
| **Scroll up / down** | Zoom in / out |

Default orbit target is `(64, 12, 64)` — centre of the 128×128 terrain grid. Adjust **Orbit Target** on `Camera3D` in the Inspector if needed.

## Minimap (top-right)

128×128 panel showing a terrain height/biome color grid (same palette as the 3D mesh). A white dot marks the camera **orbit target**. **Left-click** the minimap to move the orbit target to that cell (`Camera3D.set_orbit_target` in `scripts/camera.gd`). If terrain is not loaded yet, a placeholder dot grid is shown instead.

## Dashboard controls (bottom bar)

Hover controls for tooltips. Left-click runs the active tool on terrain.

| Control | Action |
|---------|--------|
| **Place Voxel** | Select tool, pick material id (0–7), left-click terrain → `POST /control/place_voxel` |
| **Spawn** | Palette (civilian / vehicle / airport / port / hangar). Civilian: click. Others: drag-release; long drag spawns convoy (FR-CIV-UX-004). Server: `sim.spawn_entity`; watch: `POST /control/spawn_entity` |
| **Damage** | Tactical damage tool (placeholder) |
| **Inspect** | Inspect terrain cell under cursor |
| **Camera** | Camera/orbit mode (right-drag + scroll; see camera table above) |
| **Speed** | Pause, 1×, 2×, 4×, or 8× → `sim.set_speed` (server) or `POST /control/speed` (watch) |
| **Material** | Voxel material id for Place Voxel |
| **Tick / Population** | Live metrics from `sim.snapshot` (server) or `GET /snapshot` (watch) |
| **Attach** | Default **civ-server** WS + civ-watch terrain; Inspector `attach_mode` / URLs |

## Backend connection

| Service | Default URL | Used by Godot |
|---------|-------------|----------------|
| `civ-server` | `ws://127.0.0.1:3000/ws?tick_format=binary` | Default: JSON-RPC (`health`, `sim.snapshot`, `sim.set_speed`, **`sim.spawn_civilian`**, **`sim.place_voxel`**) + F3D0 |
| `civ-watch` | `http://127.0.0.1:9090` | Always: `GET /terrain`; legacy: `POST /control/*` when `attach_mode=watch` |

Spec: [`docs/development-guide/fr-godot-attach.md`](../../docs/development-guide/fr-godot-attach.md).

**Web dashboard** (ADR-009 spectator): `cd web/dashboard && npm run dev` → http://127.0.0.1:5173

- Default: attaches to `civ-server` at `:3000` (WebSocket)
- Read-only watch mode: add `?attach=watch` to use `civ-watch` at `:9090`
- When `civ-watch` is running, its built dashboard is also served at http://127.0.0.1:9090/

Override ports with `CIV_WATCH_PORT` (civ-watch) or `CIVIS_WS_ADDR` (civ-server).

## Screenshots

For ADR-007 docs, README assets, or visual regression baselines:

1. Start `cargo run -p civ-server` and `cargo run -p civ-watch` (repo root); rebuild the extension: `cd clients/godot-ref/rust && cargo build`.
2. Open `project.godot`, select the **World** root node, and adjust **Terrain Height Exaggeration** in the Inspector if relief is too flat (default `24`; web dashboard uses `12`).
3. Press **F5** and wait for the terrain mesh to load from `GET /terrain`.
4. Capture the 3D view with the default camera (`Camera3D` at roughly `(32, 50, 32)`):
   - **Editor:** focus the 3D viewport → **Editor → Take Screenshot** (writes under the Godot user data `screenshots/` folder).
   - **Game window:** **Project → Tools → Capture Screenshot**, or the screenshot shortcut while the running game window is focused.
5. Save copies for the repo under `docs/assets/godot-ref/` (e.g. `terrain-default.png`). Vertex colors come from `CivisClient.biome_color` when biomes are present, otherwise `CivisClient.height_color` per cell height.

## Layout

```
clients/godot-ref/
├── README.md
├── .gitignore
├── civis.gdextension
├── justfile
├── project.godot
├── rust/
│   ├── Cargo.toml
│   └── src/lib.rs
├── scenes/
│   └── main.tscn
└── scripts/
    ├── camera.gd
    ├── civis_ws_client.gd
    ├── main.gd
    ├── minimap.gd
    └── ui.tscn
```
