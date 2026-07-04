# Client Launch Readiness — Bevy reference client (`civ-bevy-window`)

**Date:** 2026-06-24
**Branch:** `research/client-launch-readiness`
**Worktree:** `C:/Users/koosh/Dev/Civis/.worktrees/wt-launch`
**Scope:** Read-only audit of `clients/` + `crates/server/` + `docs/` (per task brief).

This report inventories the actual playable-AAA "last mile" path: from `cargo run` to a player clicking a god-tool button and seeing a visible world change. Every link in that loop is listed; for each link I name the file + function and whether it is wired today. The ordered fix list at the bottom is the smallest set of concrete code changes required to reach the first click-to-fire loop on the Bevy primary client.

---

## 1. Primary client

`clients/bevy-ref/` is the daily-driver per ADR-007 ("Bevy … daily-driver for CI, deterministic replay verification, screenshot regression, agent-driven workflows") and per `clients/bevy-ref/README.md:5-7`.

The windowed launcher binary is `civ-bevy-window` (`clients/bevy-ref/src/bin/bevy_window.rs:1`). It requires features `bevy,egui` (`clients/bevy-ref/Cargo.toml:170-174`).

### Current launch command

Two equivalent verified incantations; both are documented in `justfile` and `clients/bevy-ref/README.md`.

**A) Just recipe (preferred — sets `BEVY_ASSET_ROOT` so the bin finds `clients/bevy-ref/assets` when launched from the workspace root):**

```bash
just play-window
# underlying command (justfile:261-264):
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window
# with: BEVY_ASSET_ROOT=$(pwd)/clients/bevy-ref
```

**B) Manual (the `bevy_window.rs` `App` does not require the asset-root env on a clean build, but the verified recipe does):**

```bash
# Terminal 1 — backend
cargo run -p civ-server            # listens on ws://127.0.0.1:3000/ws

# Terminal 2 — windowed client (binary requires --features bevy,egui)
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window
```

Without `civ-server` running, the window opens, sits on the splash overlay (`AppState::Connecting` in `clients/bevy-ref/src/bin/bevy_window.rs:82-89`), and auto-recovers via the `WsClient` reconnect loop (`clients/bevy-ref/src/ws_client.rs:60-87`).

### What works today (with `civ-server` running)

| Step | Where | Status |
|------|-------|--------|
| `cargo run` opens a window | `bin/bevy_window.rs:266-362` (`main`) | works |
| Splash overlay renders until connected | `bin/bevy_window.rs:459-498` (`spawn_connecting_overlay`) + `drive_app_state` line 449-457 | works |
| Connects to `civ-server` WS at `ws://127.0.0.1:3000/ws` | `bin/bevy_window.rs:666` (`WsClient::spawn_with_config(resolve_live_ws_url(), …)`) using `resolve_live_ws_url` in `src/lib.rs:481-488` | works (defaults via env: `CIV_WS_URL` > `CIV_SERVER_PORT`+`CIV_WS_HOST`+`CIV_WS_PATH` > `ws://127.0.0.1:3000/ws`) |
| Receives `F3D0` binary tick frames | `bin/bevy_window.rs:834-903` (`apply_live_frames`) → `apply_voxel_delta_frame` / `apply_building_diff_frame` / `apply_civilian_state_frame` / `apply_faction_state_frame` / `apply_event_feed_frame` in `src/live_stream.rs` | works (the live-stream apply pipeline is the same one used by `civ-standalone`) |
| Spawns voxel chunks from streamed frames | `live_stream.rs:apply_voxel_delta_frame:486` | works |
| Spawns streamed agents / buildings / graph parcels | `live_stream.rs:apply_agent_appearance_frame_with_labels:590` / `apply_building_diff_frame:684` | works |
| Renders 3D scene (camera, sun, ambient, chunks) | `src/bevy_render.rs:spawn_default_scene:100` + `bevy_window.rs:update_orbit_camera_transform:1046-1057` | works |
| HUD overlay (FPS, tick, WS state, scene counts) | `bin/bevy_window.rs:1059-1077` (`update_hud`) → `src/lib.rs:LiveHudSnapshot::format_overlay:355` | works |
| Minimap dots | `bin/bevy_window.rs:1091-1295` (`update_minimap`) | works |
| Scenario launch panel (preset, seed, speed) → JSON-RPC `sim.load_scenario` / `sim.set_speed` / `sim.reset` | `bin/bevy_window.rs:364-446` (`scenario_panel_input`) | works — wired end-to-end |
| Tile popup → JSON-RPC `sim.inspect_tile` | `bin/bevy_window.rs:1469-1509` (`minimap_popup_ui`) | works — wired end-to-end |
| Minimap click → camera focus | `bin/bevy_window.rs:1297-1390` (`minimap_click_focus`) | works |
| `F3` wireframe toggle | `bin/bevy_window.rs:743-747` (`debug_render_input`) + `sync_chunk_debug_render:795-832` | works |

### What does NOT work today

| Step | Where | Status |
|------|-------|--------|
| **Click a god-tool button and see the world change** | God UI → JSON-RPC → server → next tick `VoxelDelta` → chunk repaint | **broken in 3 places** — see §3 |

The HUD, the streamed world, the orbit camera, and the scenario launch all flow through the same WS pipeline that the god tools are *supposed* to flow through. The gap is downstream of the click: the click handler, the JSON-RPC method, and the server dispatch each have a defect.

---

## 2. The intended god-tool loop (what the code wants to do)

1. User presses `G` → `GodPanelState.visible = true`.
   - `clients/bevy-ref/src/god_panel.rs:38` `toggle_god_panel` (system).
2. User picks an action (smite / bless / earthquake / plague / miracle) and magnitude + coords/faction.
   - `clients/bevy-ref/src/god_panel.rs:67-105` (`draw_god_panel` — egui window).
3. User clicks **Invoke**.
   - `clients/bevy-ref/src/god_panel.rs:107-112` — captures `action_name` into local `fire` variable.
4. `fire` is enqueued; after the egui closure, `clients/bevy-ref/src/god_panel.rs:119-131` builds a JSON-RPC payload and calls:
   ```rust
   bridge.client.send_rpc("sim.god_action", payload);
   ```
5. The `WsClient` enqueues the JSON-RPC frame onto the live WS connection.
   - `clients/bevy-ref/src/ws_client.rs:189-198` (`send_rpc(&str, serde_json::Value)`).
6. **Server receives** `sim.god_action`, dispatches a `DispatchEffect::GodAction` to the bridge, which mutates the `Simulation` (calls `civ_engine::disasters::trigger_disaster` / `sim.add_belief` / treasury credit).
   - Side-effect handler: `crates/server/src/ws_bridge.rs:1207-1294` (it is duplicated three times in the file — see §3.C).
7. On the next sim tick, `civ-server` broadcasts a `VoxelDeltaFrame` (and/or `EventFeedFrame` for belief) over the WS.
   - `crates/server/src/lib.rs` tick loop → broadcast pipeline.
8. `civ-bevy-window` receives the frame, `apply_voxel_delta_frame` (`clients/bevy-ref/src/live_stream.rs:486`) repaints the affected chunk meshes → **visible change in the world**.

Steps 1–3 are wired. Step 5 is wired. Step 8 is wired (already exercised by the sim's own voxel mutations). Steps 4 and 6 are broken in three distinct places:

---

## 3. Missing links (file + function, in dependency order)

### A. **`crates/server/src/jsonrpc.rs:35-103` + `:146-183` — `sim.god_action` is not a registered JSON-RPC method.**

`JsonRpcMethod` enum (`jsonrpc.rs:35-103`) has no `SimGodAction` variant. `as_str` (`jsonrpc.rs:107-143`) and `parse_name` (`jsonrpc.rs:146-183`) therefore return `None` for the wire string `"sim.god_action"`. When the server parses the request it returns `MethodNotFound` (`jsonrpc.rs:319-359`, `parse_request`).

The `DispatchEffect::GodAction` variant (`jsonrpc.rs:1178-1189`) exists and the bridge handler is implemented, but **no code path in `dispatch_request` (`jsonrpc.rs:1485-2080`) ever constructs it** — grep across the workspace confirms zero `DispatchEffect::GodAction {` constructions.

### B. **`clients/bevy-ref/src/god_panel.rs:7` — broken `LiveBridge` import.**

```rust
use crate::live_stream::LiveBridge;
```

`LiveBridge` is declared as a *private* `struct` in the binary crate file `clients/bevy-ref/src/bin/bevy_window.rs:158-160`:

```rust
#[derive(Resource)]
struct LiveBridge {
    client: WsClient,
}
```

It is not exported from the library (`grep "pub struct LiveBridge" clients/bevy-ref/src/` returns nothing), and `clients/bevy-ref/src/live_stream.rs` does not define `LiveBridge` at all. Therefore:

- The current `use crate::live_stream::LiveBridge;` path is a **compile error** under `#[cfg(all(feature = "bevy", feature = "egui"))]` (which is the only feature set where `god_panel.rs` compiles per the `#![cfg(all(feature = "bevy", feature = "egui"))]` gate at `god_panel.rs:1`).
- A second latent issue: `god_panel.rs:47` takes `Res<LiveBridge>` even though the god panel should fire RPCs regardless of whether the binary defines a `LiveBridge` resource.

The fact that `GodPanelPlugin` is already wired in `bin/bevy_window.rs:291` (`.add_plugins(GodPanelPlugin)`) means **the binary does not build today with `--features bevy,egui`** (the features that `civ-bevy-window` requires per `Cargo.toml:170-174`).

> Cross-check: `bin/bevy_window.rs` is a separate `[[bin]]` with `required-features = ["bevy", "egui"]`. `cargo check -p civ-bevy-ref --features bevy --bin civ-bevy-window` (per `justfile:169`) compiles **without** `egui`, which is exactly the path that does NOT pull `god_panel.rs` in (the `#![cfg(all(feature = "bevy", feature = "egui"))]` gate). So CI's `cargo check` of the window bin succeeds even though the actual user-facing bin `civ-bevy-window` (which requires both features) does not.

### C. **`crates/server/src/ws_bridge.rs:1207-1294` — `DispatchEffect::GodAction` handler is duplicated three times.**

The same `match action.as_str() { "smite" => … "miracle" => … }` block appears verbatim at `ws_bridge.rs:1207-1249`, `:1251-1293`, and `:1295-…` (only the first ~50 lines of the third copy fit in the search). This is dead code from an unresolved merge, and at minimum needs deduplication — but it does NOT block the click-to-fire loop because Rust picks the first match arm that satisfies the pattern. Still, the handler is unreachable as long as (A) is unresolved.

### D. (Bonus, unrelated to god-tools but adjacent) **`clients/bevy-ref/src/ws_client.rs:78-99` — malformed `Self { … }` block and duplicated `cmd_tx`/`send_tx`/`outcome_rx` field assignments.**

`spawn_with_config` (`ws_client.rs:60-87`) compiles the trailing `Self { frame_rx, meta_rx, rtt_rx, state_rx, latest_state, send_tx, emergence_rx }` on lines 78-86, but the file also contains a second copy of the `Self { … }` literal at lines 89-99 followed by a second `send_rpc` impl block. The file has clearly been merged into an inconsistent state. **This blocks `cargo check --features bevy` of the lib** (which is needed for `civ-bevy-window` to build). Workaround for the day-1 check: cargo's incremental build may be picking one branch, but the file as written on `main` should not compile cleanly.

> Verify with `cargo check -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window` in a clean target dir before quoting this link to the PR description.

---

## 4. The shortest ordered fix list (smallest PR to first click-to-fire)

All numbers are line numbers into the current `main` commit.

### Step 1 — Make the client compile (gate god-tool wiring behind a real bridge)

**File:** `clients/bevy-ref/src/god_panel.rs:7`

Move `LiveBridge` from `bin/bevy_window.rs` into the library (or behind a thin shared resource) and update the import. The minimum-change path:

- In `clients/bevy-ref/src/live_stream.rs` (next to `LiveStreamScene` at `live_stream.rs:106`), add:
  ```rust
  #[cfg(feature = "bevy")]
  #[derive(Resource)]
  pub struct LiveBridge { pub client: crate::ws_client::WsClient }
  ```
  or, equivalently, mark the `LiveBridge` in `bin/bevy_window.rs:158` `pub` and `#[cfg(feature = "bevy")]`-gate it via a shared `pub mod god_bridge;` module.
- Update `clients/bevy-ref/src/god_panel.rs:7`:
  ```rust
  use crate::live_stream::LiveBridge;
  ```
  (the import path is already correct *if* `LiveBridge` is moved to `live_stream`).
- Update `bin/bevy_window.rs:668` from `commands.insert_resource(LiveBridge { client: ws_client });` to `commands.insert_resource(LiveBridge { client: ws_client });` (no change required once `LiveBridge` is `pub`).

This unblocks the lib + window bin compile under `bevy,egui`.

### Step 2 — Wire the JSON-RPC method

**File:** `crates/server/src/jsonrpc.rs`

In the `JsonRpcMethod` enum (`jsonrpc.rs:35-103`), add:
```rust
/// Direct god-tool intervention (`sim.god_action`, FR-CIV-GAME-002).
SimGodAction,
```

In `as_str` (`jsonrpc.rs:107-143`):
```rust
Self::SimGodAction => "sim.god_action",
```

In `parse_name` (`jsonrpc.rs:146-183`):
```rust
"sim.god_action" => Some(Self::SimGodAction),
```

In `dispatch_request` (`jsonrpc.rs:1485-2080`), add an arm alongside the existing `SimDiplomacyAction`:
```rust
JsonRpcMethod::SimGodAction => match parse_god_action_params(req.params.as_ref()) {
    Ok((action, x, y, target_faction, magnitude)) => DispatchPlan {
        response: JsonRpcResponse::success(req.id, serde_json::json!({"accepted": true, "tick": ctx.tick})),
        effect: DispatchEffect::GodAction { action, x, y, target_faction, magnitude },
    },
    Err(error) => DispatchPlan { response: JsonRpcResponse::failure(req.id, error), effect: DispatchEffect::None },
},
```

Add a small param parser near `parse_sim_command_action` (`jsonrpc.rs:412`):
```rust
pub fn parse_god_action_params(params: Option<&Value>) -> Result<(String, Option<f32>, Option<f32>, Option<u32>, Option<f32>), JsonRpcError> {
    let action = params.and_then(|p| p.get("action")).and_then(|a| a.as_str())
        .ok_or_else(|| JsonRpcError { code: error_code::INVALID_PARAMS, message: "Missing action".into(), data: None })?
        .to_owned();
    let x = params.and_then(|p| p.get("x")).and_then(|v| v.as_f64()).map(|v| v as f32);
    let y = params.and_then(|p| p.get("y")).and_then(|v| v.as_f64()).map(|v| v as f32);
    let target_faction = params.and_then(|p| p.get("target_faction")).and_then(|v| v.as_u64()).map(|v| v as u32);
    let magnitude = params.and_then(|p| p.get("magnitude")).and_then(|v| v.as_f64()).map(|v| v as f32);
    Ok((action, x, y, target_faction, magnitude))
}
```

Optionally update `crates/server/src/lib.rs` and the WS protocol surface (`docs/api/jsonrpc-surface.md`) to list `sim.god_action`. The catalog check (`scripts/check-jsonrpc-catalog.ps1`) will fail otherwise.

### Step 3 — De-dup the bridge handler

**File:** `crates/server/src/ws_bridge.rs:1207-1294`

Delete two of the three identical `DispatchEffect::GodAction` arms; keep one. (Rust pattern-matching still allows the duplicated arms today, but `clippy --all-targets -D warnings` would catch this.)

### Step 4 — Verify the click-to-fire loop end-to-end

```bash
cargo run -p civ-server
# in another shell:
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window
```

In the window:
1. wait for the connection overlay to clear (HUD shows `connected`, tick increments);
2. press `G` → "God Mode" window opens with the action list (`god_panel.rs:67-105`);
3. select `smite`, leave magnitude at 0.5, click **Invoke: smite**;
4. expect a `VoxelDeltaFrame` to arrive within ~100 ms → the chunk under the target coords repaints (`live_stream.rs:apply_voxel_delta_frame:486`).

The other four actions (`bless`, `earthquake`, `plague`, `miracle`) take the same code path; their visible side-effects differ (treasury/belief changes flow through `FactionStateFrame` + `EventFeedFrame`).

### Step 5 — Add the smallest regression test

`crates/server/tests/ws_smoke.rs` already houses WS protocol smoke tests. Add `ws_god_action_invokes_dispatch_effect`:
```rust
// build request, dispatch through DispatchContext { sim: stub, ..Default::default() },
// assert plan.effect == DispatchEffect::GodAction { action: "smite", .. }.
```

---

## 5. Out of scope for the click-to-fire loop (but adjacent)

- **HUD-side effect feedback:** `god_panel.rs:114-116` already writes `state.status = Some(...)` after the click. The server response (or the next `VoxelDeltaFrame`) is not currently surfaced back into the panel. After step 4 the panel does flip status, but it doesn't know if the server actually applied the action. A future PR can wire a request-id correlation + a `sim.god_action.applied.v1` replay-bus event (matches the existing `mod.loaded.v1` pattern in `crates/server/src/jsonrpc.rs:471-476`).
- **Operator role gating:** `SimDiplomacyAction` / `SimSpawnCivilian` check `role_allows_operator` (`jsonrpc.rs:380-391`). For a local-dev "playable" loop, `sim.god_action` should **not** be role-gated (matches the dashboard behavior). If gating is added later, mirror the existing `require_role: bool` in `WsBridgeConfig` (`crates/server/src/main.rs:34`).
- **Latency-to-frame budget:** the apply pipeline (`bin/bevy_window.rs:apply_live_frames:834-903`) is single-threaded and runs every frame; on a 1× sim speed the round trip from RPC submit to repaint is ≤ 2 ticks. 5× speed is fine; 50× would need batching but is out of scope.
- **Worldcoord conversion:** the server treats `x` / `y` as **normalized 0..1** coordinates and multiplies by `voxel.width()` / `voxel.depth()` (`ws_bridge.rs:1211-1214`). The client UI exposes them as `DragValue` sliders in `[0.0, 1.0]` (`god_panel.rs:93-95`) — no change needed.
- **`ws_client.rs:78-99` malformed file** is independent and should land in a separate cleanup PR before the god-tool PR — otherwise `cargo check -p civ-bevy-ref --features bevy` will fail and reviewers will conflate the two changes.

---

## 6. Verification gates

Once steps 1–5 land:

| Gate | Command | Expected |
|------|---------|----------|
| Compiles | `cargo check -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window` | exits 0 |
| Catalog | `just civis-3d-catalog-check` | lists `sim.god_action`; exits 0 |
| Server unit test | `cargo test -p civ-server dispatch_request` | includes new arm |
| Server smoke | `cargo test -p civ-server ws_smoke` | unchanged, still passes |
| Bevy live smoke | `just civis-3d-live-smoke` | unchanged |
| Manual click-to-fire | see step 4 above | chunk repaints within 1 tick |

The "playable-AAA last mile" is then `cargo run -p civ-bevy-ref --features bevy,egui --bin civ-bevy-window` + `G` + click → visible change.
