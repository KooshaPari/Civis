# civ-bevy-ref

Civis Bevy 3D reference client. Per `docs/adr/ADR-007-three-renderers.md`:

> **Daily-driver for CI, deterministic replay verification, screenshot regression,
> agent-driven workflows.** Visual quality below Unreal but improving (`bevy_pbr`,
> `bevy_solari` for RT GI on **Bevy 0.18**, feature-gated via `solari`).

## Status

Pre-renderer headless smoke. The binary builds a tiny `VoxelWorld`, drains its
dirty events, meshes one populated chunk with the engine-neutral `CubicMesher`,
and prints the face count. Real Bevy rendering lands behind the `bevy` feature
flag in a follow-up PR.

## Run

```bash
cargo run -p civ-bevy-ref
```

Live window (WebSocket attach + HUD overlay):

```bash
# Headless CI gate (no GPU): F3D0 WS smoke, live_ground, live_stream, live_focus, live_minimap, live_pick, minimap UV tests, compile checks
# P-W1 item 41 / FR-CIV-BEVY-016; item 47 / FR-CIV-BEVY-022; item 50 / FR-CIV-BEVY-025 — run before merging live-attach changes
just civis-3d-live-smoke

# Start civ-server first (default ws://127.0.0.1:3000/ws, tick broadcast Both)
cargo run -p civ-server

# Bevy window prefers binary F3D0 frames — skip redundant JSON text tick pushes:
CIVIS_TICK_BROADCAST=binary cargo run -p civ-server

cargo run -p civ-bevy-ref --features bevy,client-bins --bin civ-bevy-window
```

Live window with egui shell:

```bash
cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window
```
### Local play fingerprints

Feature flags compound — pick the smallest set that matches your goal. CI and
local play intentionally diverge: compile gates stay minimal; playable builds
add audio (and optionally models / voxel / GI).

| Tier | `--features` | Gate / recipe | What you get |
|------|--------------|---------------|--------------|
| **Minimal** | `bevy,egui` | `just bevy-egui-check` / `shell_attest` | Menus, HUD, in-process sim — **no audio**, heightmap terrain fallback. This is what PR compile gates and `civis-3d-live-smoke` `cargo check` use. Desktop `[[bin]]` targets are **not** built (need `client-bins`). |
| **Native smoke** | `bevy,egui,client-bins` | `just civis-3d-standalone-smoke` | Builds `civ-standalone`, runs with `CIVIS_SMOKE_FRAMES` (default 5), exits after preflight + N Update frames. Needs a GPU. |
| **Playable** | `bevy,egui,audio,client-bins` (+ optional `models`) | `just civis-bevy-play` | Release `civ-standalone` with ambient SFX + UI sounds. Add `models` when `assets/models/*.glb` are present (otherwise procedural primitives). |
| **Full sandbox** | above + optional `voxel`, `voxel_stream`, `gi` | manual `cargo build/run` | `voxel` — volumetric CA terrain + water; `voxel_stream` — camera-driven chunk streaming (implies `voxel`); `gi` — Bevy Solari RT GI (needs DXR / Vulkan RT; degrades to no-op). Heavier compile; not in CI. |

Desktop binaries (`civ-standalone`, `civ-bevy-window`, …) require the empty **`client-bins`** feature so `cargo test --test shell_attest` does not link huge Bevy exes (rust-lld hang on Windows). Recipes under `justfile` / `Tools/play.ps1` already pass it.

**Playable run (after `just civis-bevy-play` builds release):**

```powershell
# From repo root — BEVY_ASSET_ROOT required when CWD is workspace root (see Tools/play.ps1)
$env:BEVY_ASSET_ROOT = "$PWD/clients/bevy-ref"
& "$env:CARGO_TARGET_DIR/release/civ-standalone.exe"   # default target: <repo>/target

# Optional live attach (skip local terrain; remote ticks ignore pause)
$env:CIVIS_ATTACH = "server"
$env:CIV_SERVER_PORT = "3010"   # default is 3000; matches civ-server listen port
# Or full URL (overrides host/port/path):
$env:CIV_WS_URL = "ws://127.0.0.1:3010/ws?tick_format=binary"
```

Set `CIVIS_TICK_BROADCAST=binary` on `civ-server` when using `tick_format=binary` on the URL.

### Live attach smoke (`just civis-3d-live-smoke`)

Headless gate for live attach — no window or running civ-server required:

| Step | Command (via recipe) |
|------|----------------------|
| F3D0 encode/decode | `cargo test -p civ-server frame_triple` |
| WS binary tick after sim tick | `cargo test -p civ-server --test ws_smoke ws_client_receives_binary_frame3d_after_tick` |
| Voxel column ground anchoring | `cargo test -p civ-bevy-ref --features bevy --lib live_ground::` |
| Shared frame apply (`live_stream`) | `cargo test -p civ-bevy-ref --features bevy --lib live_stream::` |
| Live scene focus (orbit + minimap bounds) | `cargo test -p civ-bevy-ref --features bevy --lib live_focus::` |
| Live minimap dots (layout, UV, spawn helpers) | `cargo test -p civ-bevy-ref --features bevy --lib live_minimap::` |
| Live viewport pick (ray–AABB helpers) | `cargo test -p civ-bevy-ref --features bevy --lib live_pick::` |
| Minimap UV mapping (`world_xz_to_minimap_uv` path) | `cargo test -p civ-bevy-ref --lib chunk_to_minimap` + `minimap_uv_to_chunk` |
| Client compile | `cargo check … civ-standalone`, `cargo check … civ-bevy-window` |

### Remote civ-server URL recipes

| Client | Local default | Remote (Tailscale / LAN) |
|--------|---------------|---------------------------|
| `civ-bevy-window` | `CIVIS_WS_URL` or `CIVIS_WS_ADDR` → `ws://127.0.0.1:3000/ws` | Set env before run, e.g. `CIVIS_WS_URL=ws://100.x.x.x:3000/ws?tick_format=binary` |
| `civ-standalone` (live attach) | `just civis-3d-standalone-live` (`CIVIS_ATTACH=server`) | `just civis-3d-standalone-live-url URL=ws://host:3000/ws?tick_format=binary` |

Prefer `tick_format=binary` on the URL when the server runs with `CIVIS_TICK_BROADCAST=binary`.

### WebSocket binary tick frames (`F3D0`)

`civ-server` defaults to `TickBroadcastFormat::Both` (JSON text + matching `F3D0`
binary frames each tick). The live window prefers binary to avoid duplicate work:

| Setting | Effect |
|---------|--------|
| `CIVIS_TICK_BROADCAST=binary` (server) | Broadcast binary `F3D0` tick frames only (`text` / `both` also accepted; default `both`) |
| `DEFAULT_WS_PREFER_BINARY=true` (constant in `lib.rs`) | Skip JSON text tick frames; decode binary `F3D0` only |
| `CIVIS_WS_BINARY=1` | Same as above (`true` / `yes` also accepted). Set `0` or `false` to process text frames too |
| `CIVIS_WS_URL` / `CIVIS_WS_ADDR` | Attach URL (same precedence as the web dashboard) |
| `tick_format=binary` query | Appended to the connect URL when binary is preferred; servers may honor this for binary-only broadcast |

Payload decode order (text or binary WebSocket frame): **F3D0 binary first**, then UTF-8 JSON fallback.

Default camera orbits chunk centre `(8, 8, 8)` at ~48 world units with 45° azimuth
and ~35° elevation — see `CameraTarget` in `src/lib.rs`.

### `civ-standalone` sandbox (HUD + menus)

Requires `--features bevy,egui,client-bins`:

```bash
cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone
```

| Input | Action |
|-------|--------|
| `Space` | Toggle pause overlay (dims world; zeros `GameSpeed` and halts in-process sim) |
| `Escape` | Close panels / also toggles pause when settings are closed |
| `?` | Controls cheat sheet |
| Pause overlay **Resume** | Dismiss overlay and restore prior sim speed |
| HUD pause / `1`–`4` | Speed chips set `GameSpeed` directly (sim pause without overlay) |
| `Shift`+`1`–`9` | Tool categories (Select…Policy) |
| `Ctrl`+`K` | Holocron Command‑K verb launcher (live attach; Enter fires `GodActionRequest`) |
| Settings (pause menu) | Graphics / audio / controls (persisted `settings.ron`); some display flags need restart |
| **L** | Toggle scrollable **Event Log** (egui); stacked toasts bottom-right (~8s) |
| `F1` | Toggle faction HUD |
| Live attach (`CIVIS_ATTACH=server`) | Toasts on WebSocket `connected` / `reconnecting` / `disconnected` (`EventKind::System`) |

Live attach (`CIVIS_ATTACH=server` or `CIV_WS_URL`) skips local terrain; pause does not gate remote ticks.

### Orbit camera controls (`civ-bevy-window`)

| Input | Action |
|-------|--------|
| Left drag | Orbit (azimuth / elevation) |
| Scroll wheel | Zoom (distance) |
| `Q` / `E` | Pivot rotate (orbit left / right) |
| `R` / `F` | Raise / lower orbit centre |
| `Home` | Reset to [`CameraTarget::default()`](src/lib.rs) |
| `=`, `+` (numpad), `[` | Zoom in (decrease distance) |
| `-`, numpad `-`, `]` | Zoom out (increase distance) |
| `W` / `A` / `S` / `D` | Pan orbit centre on the horizontal plane |
| `F3` | Toggle chunk mesh wireframe debug overlay |

### Native GPU backends (`CIV_BEVY_BACKEND`)

Bevy still routes through `wgpu`, but Civis restricts adapter search to **native HAL backends only** (no GLES, no browser WebGPU). Implementation: [`src/native_backend.rs`](src/native_backend.rs) (`native_only_backends`, `native_render_plugin`).

| `CIV_BEVY_BACKEND` | Effect |
|--------------------|--------|
| *(unset)* | Platform defaults below |
| `dx12` | Force DirectX 12 only (`d3d12`, `directx` aliases accepted) |
| `vulkan` | Force Vulkan only (`vk` alias accepted) |
| `metal` | Force Metal only (macOS) |

**Platform defaults when unset:**

| OS | Adapter backends |
|----|------------------|
| Windows | DX12 \| Vulkan |
| macOS | Metal \| Vulkan |
| Linux / other Unix | Vulkan |

Invalid values are logged and ignored; defaults apply. Wireframe overlay (`F3`) requires `WgpuFeatures::POLYGON_MODE_LINE` (enabled in `native_wgpu_settings`).

**Tests (no GPU):**

```bash
cargo test -p civ-bevy-ref --features bevy --lib native_backend
```

**Research:** native `wgpu::Device::as_hal` escape hatches for future DXR / mesh shaders — [`docs/research/wgpu-native-escape-hatches.md`](../../docs/research/wgpu-native-escape-hatches.md). Traceability: **FR-CIV-BEVY-026** / P-W1 kickoff **item 51**.

### Debug wireframe (`DebugRender`)

Press **`F3`** in `civ-bevy-window` to toggle chunk wireframe rendering. State lives in
[`DebugRender { wireframe: bool }`](src/lib.rs) (default off).

When enabled:

- Bevy 0.18 [`WireframePlugin`](https://docs.rs/bevy/latest/bevy/pbr/wireframe/struct.WireframePlugin.html) draws native line wireframes on chunk meshes (DX12 / Vulkan / Metal; requires `WgpuFeatures::POLYGON_MODE_LINE`).
- Native backend selection is documented above ([`CIV_BEVY_BACKEND`](#native-gpu-backends-civ_bevy_backend)); see also [`wgpu-native-escape-hatches.md`](../../docs/research/wgpu-native-escape-hatches.md).
- Chunk fill uses unlit [`StandardMaterial`](https://docs.rs/bevy/latest/bevy/pbr/struct.StandardMaterial.html) at low alpha ([`DEBUG_WIREFRAME_OVERLAY_ALPHA`](src/lib.rs), default `0.22`) so solid faces stay visible under the lines.
- Agent markers are unaffected.

Pure toggle logic is tested without a GPU:

```bash
cargo test -p civ-bevy-ref --features bevy debug_render_wireframe_toggle
```

Expected output (headless smoke):

```
dirty events: 64
mesh: 384 vertices, 576 indices
```

(4³ = 64 voxel writes; the 4×4×4 cube exposes 6 × 4² = 96 faces → 384 vertices,
576 indices — internal faces correctly culled by the cubic mesher.)
