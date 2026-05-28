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
# Headless protocol smoke (no window): F3D0 tick + voxel ground + standalone compile
just civis-3d-live-smoke

# Start civ-server first (default ws://127.0.0.1:3000/ws, tick broadcast Both)
cargo run -p civ-server

# Bevy window prefers binary F3D0 frames — skip redundant JSON text tick pushes:
CIVIS_TICK_BROADCAST=binary cargo run -p civ-server

cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window

# Standalone gameplay client on a remote civ-server (e.g. Tailscale proxy):
# just civis-3d-standalone-live-url URL=ws://host:3000/ws?tick_format=binary
```

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

### Orbit camera controls (`civ-bevy-window`)

| Input | Action |
|-------|--------|
| Left drag | Orbit (azimuth / elevation) |
| Scroll wheel | Zoom (distance) |
| `R` | Reset to [`CameraTarget::default()`](src/lib.rs) |
| `=`, `+` (numpad), `[` | Zoom in (decrease distance) |
| `-`, numpad `-`, `]` | Zoom out (increase distance) |
| `W` / `A` / `S` / `D` | Pan orbit centre on the horizontal plane (stub) |
| `F3` | Toggle chunk mesh wireframe debug overlay |

### Debug wireframe (`DebugRender`)

Press **`F3`** in `civ-bevy-window` to toggle chunk wireframe rendering. State lives in
[`DebugRender { wireframe: bool }`](src/lib.rs) (default off).

When enabled:

- Bevy 0.18 [`WireframePlugin`](https://docs.rs/bevy/latest/bevy/pbr/wireframe/struct.WireframePlugin.html) draws native line wireframes on chunk meshes (DX12 / Vulkan / Metal; requires `WgpuFeatures::POLYGON_MODE_LINE`).
- Renderer adapters are restricted to **native HAL backends** (DX12 + Vulkan on Windows) via [`native_backend`](src/native_backend.rs). Override with `CIV_BEVY_BACKEND=dx12|vulkan|metal`. Future DXR/mesh-shader work uses `wgpu::Device::as_hal` — see `docs/research/wgpu-native-escape-hatches.md`.
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
