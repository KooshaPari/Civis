# Civis v0.4.0 — Reproducible Gameplay Playthrough

> **Status:** Locked for v0.4.0 (`14a1fc807d10067ce19e12b762e9321531e996c0`)
> **Last updated:** 2026-08-29
> **Audience:** QA, integration testers, demo videographers, contributor onboarding.

This document describes a deterministic, end-to-end playthrough of **Civis
v0.4.0** that exercises every shipping feature surface: the headless server,
the Bevy reference client, the full JSON-RPC method catalogue, the five
production save slots, and the god-tool verbs. The matching automation lives
in [`scripts/playthrough.sh`](../scripts/playthrough.sh) and is exercised
weekly by [`playthrough-validate.yml`](../.github/workflows/playthrough-validate.yml).

The playthrough is **idempotent** — re-running it on the same seed restores
the same world and yields byte-identical save bundles. It is also
**reproducible** — the entire flow can be replayed from the JSON-RPC
transcript without a GPU.

---

## 1. Pre-requisites

### 1.1 Hardware

| Component | Minimum                       | Recommended                           |
| --------- | ----------------------------- | ------------------------------------- |
| CPU       | x86_64 4-core                 | x86_64 8-core with AVX2               |
| RAM       | 8 GiB                         | 16 GiB                                |
| GPU       | wgpu-capable (Vulkan / DX12)  | NVIDIA RTX 20-series or newer         |
| Disk      | 12 GiB free (target + caches) | 30 GiB SSD                            |

> The Bevy client uses **wgpu** for the renderer and **DX12 Ultimate** (with
> DXR and DLSS toggles) on Windows. A discrete GPU is required for the
> windowed client. The headless server (`civ-server`) runs on any CPU.

### 1.2 Toolchain

| Tool                | Version              | Notes                                              |
| ------------------- | -------------------- | -------------------------------------------------- |
| Rust                | `stable` (≥ 1.75)    | Pinned by `rust-toolchain.toml`                    |
| `cargo`             | bundled with Rust    | Workspace uses the 2024 resolver                   |
| `rustfmt`, `clippy` | bundled components   | `rustup component add rustfmt clippy`              |
| `websocat`          | any recent release   | WebSocket client for shell-based JSON-RPC          |
| `curl`              | any recent release   | Used for the screenshot capture helper             |
| Python 3.11+        | for asset bootstrap  | Optional — only needed if rebuilding the asset DB  |

> The repo pins `rust-version = "1.94.1"` in `Cargo.toml`. Anything at or
> above that line on `stable` will resolve correctly; older toolchains fail
> the manifest check.

### 1.3 Drivers

- **Windows:** install the latest NVIDIA / AMD / Intel GPU driver. wgpu
  will pick DX12 automatically; Vulkan is used as the fallback.
- **Linux:** install `vulkan-tools` and `libwayland-dev` (or `libx11-dev`
  on X11 sessions). The server is fully headless.
- **macOS:** Metal is selected automatically; DX12/Vulkan probes are skipped.

---

## 2. Build commands

All commands assume the repository root as the current working directory.

### 2.1 Workspace compile check

```bash
rustup component add rustfmt clippy
cargo check --workspace
```

### 2.2 Release build — headless server

```bash
cargo build --release -p civ-server
```

The server binary lands at `target/release/civ-server(.exe)`. It listens on
`ws://127.0.0.1:3000/ws` by default; override the port with `CIV_SERVER_PORT`
and the bind address with `CIVIS_WS_ADDR`.

### 2.3 Release build — Bevy reference client

```bash
BEVY_ASSET_ROOT="$PWD/clients/bevy-ref" \
    cargo build --release -p civ-bevy-ref \
    --features bevy,egui,audio,client-bins \
    --bin civ-standalone
```

The standalone client embeds the simulation in-process. To attach it to a
running server instead, set:

```bash
CIVIS_ATTACH=server
CIV_WS_URL=ws://127.0.0.1:3000/ws?tick_format=binary
```

### 2.4 Build-only gate (no GPU needed)

```bash
just civis-3d-verify     # catalog + scenario + web + mod + clippy
just bevy-egui-check     # compile-only oracle for the bevy/egui feature pair
```

The verify recipes match what the CI runs on every PR and finish in well
under 10 minutes on a warm cache.

---

## 3. The scripted 10-step playthrough

Each step is described in three forms: the **human action** (mouse / keyboard
on the Bevy client), the **machine action** (the JSON-RPC request the
companion script sends), and the **expected outcome** (what the world state
should look like afterwards). The step numbers line up 1:1 with
`scripts/playthrough.sh`.

The world is reset to a fresh seeded instance at the start of every run,
so the script can be re-executed any number of times without compounding
state.

| #  | Step                          | RPC method                | HUD key     |
| -- | ----------------------------- | ------------------------- | ----------- |
| 1  | Launch server                 | (process launch)          | n/a         |
| 2  | Launch Bevy client            | (process launch)          | n/a         |
| 3  | Click "New World"             | `sim.reset`               | main menu   |
| 4  | Spawn civilians               | `sim.spawn_civilian` × N  | `G` (tools) |
| 5  | Observe AI goal tree          | `sim.get_factions`        | `D`         |
| 6  | Propose trade                 | `sim.diplomacy_action`    | `D`         |
| 7  | Declare war                   | `sim.diplomacy_action`    | `D`         |
| 8  | Save to slot-1                | `save.slot`               | `F5`        |
| 9  | Load slot-1                   | `save.load`               | `F9`        |
| 10 | God action: `smite`           | `sim.god_action`          | `G`         |

### Step 1 — Launch server

```bash
CIVIS_SAVES_DIR="$PWD/crates/server/saves" \
CIVIS_REPLAYS_DIR="$PWD/crates/server/replays" \
CIVIS_MAP_SEED=42 \
CIV_SERVER_PORT=3000 \
"$PWD/target/release/civ-server" &
SERVER_PID=$!
```

**Expected outcome:** the bridge binds `127.0.0.1:3000`, the health probe
returns `{"status":"ok"}`, and the first 10 Hz tick loop begins broadcasting
`Frame3d` bundles. The server log shows:

```
INFO civ_server::ws_bridge: ws bridge listening addr=127.0.0.1:3000 max_clients=16
INFO civ_server::ws_bridge: tick loop broadcasting format=Both
```

### Step 2 — Launch Bevy client

```bash
BEVY_ASSET_ROOT="$PWD/clients/bevy-ref" \
CIVIS_ATTACH=server \
CIV_WS_URL="ws://127.0.0.1:3000/ws?tick_format=binary" \
"$PWD/target/release/civ-standalone.exe" &
CLIENT_PID=$!
```

**Expected outcome:** the egui main menu appears within ~2 s, and the
status bar reads `attach: server @ 127.0.0.1:3000`. The first `Frame3d`
bundle arrives over the wire and the "Press Enter to begin" hint shows up.

### Step 3 — Click "New World"

```json
{"jsonrpc":"2.0","id":1,"method":"sim.reset","params":{"seed":42}}
```

**Expected outcome:** the server replies `{"seed":42,"tick":0}`. The
Bevy client transitions out of the main menu into `AppState::Loading` for
≤ 2 ticks, then into `AppState::Playing`. The minimap renders the seeded
biome and the event feed logs `"world_created"`.

### Step 4 — Spawn civilians

```json
{"jsonrpc":"2.0","id":2,"method":"sim.spawn_civilian","params":{"x":0.50,"y":0.50,"faction":0}}
{"jsonrpc":"2.0","id":3,"method":"sim.spawn_civilian","params":{"x":0.51,"y":0.49,"faction":0}}
{"jsonrpc":"2.0","id":4,"method":"sim.spawn_civilian","params":{"x":0.49,"y":0.51,"faction":1}}
{"jsonrpc":"2.0","id":5,"method":"sim.spawn_civilian","params":{"x":0.50,"y":0.52,"faction":1}}
```

**Expected outcome:** each call returns `{"ok":true,"accepted":true,"tick":N}`.
After ~30 ticks, `sim.status` reports `{"tick":~30,"population":4}` and the
event feed contains four `civilian_spawned` entries, two per faction. The
Bevy client renders the four agent dots on the terrain.

### Step 5 — Observe AI goal tree

```json
{"jsonrpc":"2.0","id":6,"method":"sim.get_factions","params":{}}
{"jsonrpc":"2.0","id":7,"method":"sim.tech_state","params":{}}
```

**Expected outcome:** `sim.get_factions` returns at minimum `faction 0`
and `faction 1`, each with a populated `goal_tree` field showing the
factions-decisions stack:

```
faction 0: ["survive","gather_food","expand_territory","study_writing"]
faction 1: ["survive","gather_food","fortify","study_agriculture"]
```

`sim.tech_state` lists the 12-technology research tree with `available`
and `researched` queues. The Bevy client (`D` key) renders the goal tree
inside the diplomacy panel.

### Step 6 — Propose trade

```json
{"jsonrpc":"2.0","id":8,"method":"sim.diplomacy_action","params":{
  "source_faction":0,"target_faction":1,"kind":"trade_agreement"
}}
```

**Expected outcome:** the server replies with `{"ok":true,"accepted":true}`,
the diplomacy relation between faction 0 and faction 1 transitions to
`TradeAgreement`, and both faction treasuries increment by **100** joules
per side (see `civ-engine/diplomacy.rs:533`). The event feed logs
`treaty` and the diplomacy panel renders a green link between the two
factions.

### Step 7 — Declare war

```json
{"jsonrpc":"2.0","id":9,"method":"sim.diplomacy_action","params":{
  "source_faction":0,"target_faction":1,"kind":"conflict"
}}
```

**Expected outcome:** the relation flips to `Conflict`, both treasuries
debit by **50** joules (the "war footing" cost), and the event feed logs
`war_declared`. After 50 ticks, the emergence dashboard `diplomacy_tension`
tile moves into the red band (`<-0.5`). The diplomacy panel shows the red
war indicator between factions 0 and 1.

### Step 8 — Save to slot-1

```json
{"jsonrpc":"2.0","id":10,"method":"save.slot","params":{"slot_name":"slot-1"}}
```

**Expected outcome:** the server writes `crates/server/saves/slot-1.civsave.zst`
and replies `{"saved":true,"slot_name":"slot-1"}`. The Bevy client HUD
briefly shows `Saved → slot-1`. `save.list` now returns one entry tagged
`save_type:"slot"`.

### Step 9 — Load slot-1

Advance at least 20 ticks so the live state diverges, then:

```json
{"jsonrpc":"2.0","id":11,"method":"save.load","params":{"slot_name":"slot-1"}}
```

**Expected outcome:** the server replies `{"loaded":true,"slot_name":"slot-1"}`
and the simulation state is replaced by the slot archive. `sim.get_tick`
returns the same tick value that was reported in step 8. The factions and
civilian counts match exactly (the archive is a complete deterministic
snapshot).

### Step 10 — God action: `smite`

```json
{"jsonrpc":"2.0","id":12,"method":"sim.god_action","params":{
  "action":"smite","x":0.50,"y":0.50,"radius_voxels":5
}}
```

**Expected outcome:** the server replies `{"ok":true,"accepted":true,"tick":N}`,
the terrain chunk at `(0.50, 0.50)` is cratered (`sim.terraform_extent` +
`sim.damage` are scheduled internally), and a `meteor_strike` event lands
in the feed. Belief across all factions ticks up by **+50** and the
emergence dashboard `belief` sparkline jumps. The Bevy client renders the
smoke plume particle system for ~2 seconds.

---

## 4. Expected aggregate outcomes

After running all ten steps once, `emergence.dashboard` over WebSocket
should report a non-empty dashboard with the following lower bounds:

| Tile                      | Expected range         |
| ------------------------- | ---------------------- |
| `entropy`                 | ≥ 0.05 (per-byte)      |
| `power_law_alpha`         | 1.5 ≤ α ≤ 3.5           |
| `novelty`                 | ≥ 0.01                 |
| `diplomacy_tension`       | ≤ -0.3 (post-war)      |
| `belief`                  | ≥ 0.20 (post-smite)    |

If any of these floors are missed the playthrough script prints a warning
line `WARN: emergence tile <name> below expected floor` but still exits 0;
the JSON-RPC contract guarantees the *mechanics* ran correctly even if
emergence is briefly below threshold.

---

## 5. Screenshot capture using wgpu surface readback

The Bevy client uses wgpu to present frames to the OS surface. To capture
a frame deterministically, run the headless render pass and copy the
swapchain texture into a CPU-readable buffer. The helper below is invoked
from `scripts/playthrough.sh` after steps 2, 5, 7 and 10.

### 5.1 Cargo feature flag

Build with the `frame-capture` feature to enable the capture path:

```bash
cargo build --release -p civ-bevy-ref \
    --features bevy,egui,client-bins,frame-capture \
    --bin civ-standalone
```

### 5.2 Environment toggle

Set `CIVIS_CAPTURE_DIR` to the directory where PNGs should be written. The
client appends a zero-padded counter (`0001.png`, `0002.png`, …) per frame
captured.

### 5.3 Wgpu surface readback helper

The capture is implemented in `clients/bevy-ref/src/render/capture.rs`
and exposed through the `sim.capture_frame` JSON-RPC method (added in
v0.4.0). The protocol is:

```json
{"jsonrpc":"2.0","id":99,"method":"sim.capture_frame","params":{
  "label":"after_step_10",
  "width":1280,"height":720
}}
```

The client replies with the absolute path of the written PNG and a
SHA-256 of the encoded bytes (so CI can verify the capture is byte-stable
across re-runs):

```json
{"path":"/tmp/civis-captures/after_step_10.png","sha256":"ab12...","bytes":9216}
```

### 5.4 Reference Rust snippet

```rust
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::Texture;
use image::{ImageBuffer, Rgba};

fn readback_surface(device: &RenderDevice, queue: &RenderQueue,
                    texture: &Texture) -> Vec<u8> {
    // 1. Copy the swapchain texture into a buffer with `COPY_DST` usage.
    let bytes_per_row = texture.size().width * 4;
    let download = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frame-readback"),
        size: (bytes_per_row * texture.size().height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::ImageCopyBuffer { buffer: &download,
            layout: wgpu::ImageDataLayout { offset: 0, bytes_per_row,
                rows_per_image: None } },
        texture.size(),
    );
    queue.submit([encoder.finish()]);

    // 2. Map the buffer synchronously and write to PNG.
    let slice = download.slice(..);
    slice.map_read(wgpu::MapMode::Read);
    // ... (block until ready, encode as PNG with `image` crate) ...
    # Ok(())
}
```

The capture is synchronised with the Bevy render schedule, so it always
fires at the *start* of a frame — the resulting PNG matches the visual
output the user would see at that instant.

### 5.5 CI artifact upload

`scripts/playthrough.sh` writes PNGs to `$CIVIS_CAPTURE_DIR` (default
`./captures/`). The companion workflow uploads the directory as a single
artifact named `civis-playthrough-captures-<run-id>` so reviewers can
eyeball the run.

---

## 6. Running the automation

```bash
# from the repository root
bash scripts/playthrough.sh

# capture screenshots and exit
CIVIS_CAPTURE_FRAMES=1 bash scripts/playthrough.sh

# target a remote server
CIV_WS_URL=ws://ci-runner-01.internal:3000/ws bash scripts/playthrough.sh
```

Exit codes:

| Code | Meaning                                                  |
| ---- | -------------------------------------------------------- |
| `0`  | All ten steps succeeded and produced the expected shape. |
| `1`  | Server failed to become healthy within 30 s.             |
| `2`  | A JSON-RPC call returned an error / unexpected payload.  |
| `3`  | A screenshot capture failed (only with capture on).      |
| `4`  | The script was invoked from outside the repo root.       |

---

## 7. CI gate

[`../.github/workflows/playthrough-validate.yml`](../.github/workflows/playthrough-validate.yml)
runs the script weekly and on every push to `main`. It publishes the
captures as a workflow artifact and posts a condensed result table to the
`playthrough-results` GitHub Gist (created on first run via the
`GIST_TOKEN` repository secret). Any non-zero exit opens an issue via the
`peter-evans/create-issue-from-file` action so the regression is triaged
on the next business day.
