[STATUS: PLAYABLE ALPHA] [██████████] 95% complete

---

[![Build](https://img.shields.io/github/actions/workflow/status/KooshaPari/Civis/ci.yml?branch=main&label=build)](https://github.com/KooshaPari/Civis/actions)
[![Release](https://img.shields.io/github/v/release/KooshaPari/Civis?include_prereleases&sort=semver)](https://github.com/KooshaPari/Civis/releases)
[![License](https://img.shields.io/github/license/KooshaPari/Civis)](LICENSE)
[![Phenotype](https://img.shields.io/badge/Phenotype-org-blueviolet)](https://github.com/KooshaPari)

# Civis — Emergent Civilization Godgame

**Civis** is a Bevy 0.18 desktop godgame (DX12/Windows primary) built on a headless, deterministic civilization simulation engine. Life, society, language, culture, economy, law, and politics emerge from physical and genomic laws — you play the role of a god watching (and intervening in) a world that writes its own history.

The simulation core runs headlessly over WebSocket JSON-RPC + binary frames. Multiple renderer clients (Bevy reference, Godot, Unreal, Web) can attach to the same world timeline simultaneously.

---

## What is Civis?

| Dimension | Value |
|---|---|
| **Engine** | Rust + hecs ECS; deterministic fixed-point (i64 @ 10^6 scale); ChaCha8Rng |
| **Renderer** | Bevy 0.18 (DX12 Ultimate + DXR + DLSS); Godot secondary; Unreal showcase |
| **Protocol** | WebSocket JSON-RPC + binary voxel frames |
| **Emergence** | Life / sentience / psyche / ideology / culture / language / markets / polities / architecture — all emergent |
| **Hardcoded only** | Physical laws, environmental laws, genomic floor |
| **Tick rate** | 100 ms/tick; fixed-timestep; sub-16 ms target budget |
| **Target dir** | E:\civis-target (off-tree; C: fills otherwise) |

---

## Current Features

### Core Simulation
- Deterministic world simulation with emergence metrics (entropy, power-law, novelty, MI)
- Voxel terrain (SVO + dense leaf chunks), climate, disasters, genomic substrate
- 12-technology research tree with tier-gated unlocks
- Faction system: diplomacy, war, trade, treasury, cultural drift
- Civilization era progression: Prehistoric → Ancient → Classical → Medieval → Renaissance → Modern

### Gameplay HUD (Bevy client)
| Key | Panel |
|-----|-------|
| **F** | Player faction HUD — population, treasury, era, government type |
| **N** | Event feed — 50-entry rolling log, color-coded by kind |
| **T** | Tech tree — 12 technologies, research queue |
| **D** | Diplomacy — propose treaty, declare war, offer trade |
| **G** | God-mode actions — smite, bless, earthquake, plague, miracle |
| **E** | Emergence dashboard — 6 criticality metrics (entropy, power-law α, novelty, MI, …) |
| **M** | Minimap — terrain / faction / population overlays, right-click inspect |
| **Y** | History charts — population / treasury / factions / entropy sparklines (200-sample ring buffers) |
| **P** | Performance HUD — FPS / frame-ms / sim tick / tick_ms / civilian / faction counts |
| **F5** | Save — 5 named slots |
| **Esc** | Settings menu |
| **K** | Toggle mute |
| **?** | Controls reference |
| **H** | Replay tutorial onboarding (6-step hint cards) |
| Space / , / . | Pause / slow / fast |

### God-Mode Actions (sim.god_action RPC)
- **smite** — meteor strike at (x,y); terrain damage + belief spike
- **bless** — boost target faction treasury + 500 belief
- **earthquake** — ground quake at (x,y); rubble + infrastructure damage
- **plague** — trigger disease + treasury debit on target faction
- **miracle** — +2000 belief + treasury boost across all factions

### World Presets
- Ardani, Velthari, Grundak, Felmar — each with distinct biome seeds

### Victory / Defeat Conditions
- Tech victory (all 12 researched), population victory, extinction defeat

---

## Running

Requires Rust (see ust-toolchain.toml) and a separate terminal for each process.

**Server:**
`powershell
D:/civis-build/target = "E:\civis-target"
cargo run -p civ-server
`

**Bevy client:**
`powershell
D:/civis-build/target = "E:\civis-target"
 = "C:\Users\koosh\Dev\Civis\clients\bevy-ref"
cargo run -p civ-bevy-ref --features bevy,egui --bin bevy_window
`

Or use the ergonomic launcher:
`ash
just play          # release build + detached launch + log tail
just play-debug    # with RUST_LOG=info,civ_bevy_ref=debug
`

---
## Repository Structure

- `crates/` — simulation core (Rust workspace, 28 members)
- `Cargo.toml` — Rust 2024 workspace manifest
- `docs/` — VitePress docs and specification corpus
  - `docs/wiki/` — concept and architecture knowledge
  - `docs/development-guide/` — contributor and implementation guides
  - `docs/roadmap/` — planning and sequencing artifacts
  - `docs/api/` — API and contract documentation
- Root specs: [`PRD.md`](./PRD.md), [`ADR.md`](./ADR.md), [`PLAN.md`](./PLAN.md), [`FUNCTIONAL_REQUIREMENTS.md`](./FUNCTIONAL_REQUIREMENTS.md), [`COMPARISON.md`](./COMPARISON.md)

---

## Quick Start

**Prerequisites:** Rust (edition in `Cargo.toml`), [Bun](https://bun.sh) (docs only; no npm/yarn/pnpm), [Task](https://taskfile.dev), [lefthook](https://github.com/evilmartians/lefthook) (local git hooks).

```bash
git clone https://github.com/KooshaPari/Civis.git && cd Civis
lefthook install
cargo build --workspace && cargo test --workspace
just civis-3d-verify          # or: lefthook run pre-push (emits manifest + runs gates)
cargo run -p civ-server       # http://127.0.0.1:3000  (override with CIVIS_WS_ADDR)
```

### Launch the standalone game (Bevy)

The Bevy reference client needs **both** the `bevy,egui` feature set and
a `BEVY_ASSET_ROOT` env var. Bevy 0.18 `AssetPlugin::file_path` defaults
to `"./assets"` relative to CWD — from the workspace root that resolves
to the wrong directory and produces 6 phantom module errors + ~10 asset
404s. Use the ergonomic launcher (it defaults `BEVY_ASSET_ROOT` and
`CARGO_TARGET_DIR=G:/civis-target-gate` for you):

```bash
just play          # release build + detached launch + log tail
just play-debug    # RUST_LOG=info,civ_bevy_ref=debug,wgpu=warn
just play-trace    # + RUST_BACKTRACE=full
just play-window   # live F3D0 binary-frame client (civ-bevy-window)
```

Manual incantation if you don't have `just` (Windows PowerShell):

```powershell
$env:BEVY_ASSET_ROOT = "$PWD/clients/bevy-ref"
$env:CARGO_TARGET_DIR = "G:/civis-target-gate"   # any out-of-tree dir
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-standalone
```

Manual incantation (POSIX / WSL):

```bash
BEVY_ASSET_ROOT="$PWD/clients/bevy-ref" \
CARGO_TARGET_DIR="$PWD/target" \
cargo run -p civ-bevy-ref --features bevy,egui --bin civ-standalone
```

The `just play*` recipes call into `Tools/play.ps1` (Windows) or
`Tools/play.sh` (POSIX) — both scripts honor a pre-set
`CARGO_TARGET_DIR` and default `BEVY_ASSET_ROOT` to
`clients/bevy-ref` when the env var is unset, so direct script
invocation is also safe. A runtime asset-root fallback in
`clients/bevy-ref/src/bin/standalone.rs` is planned but **deferred** to
a follow-up PR; for now, set `BEVY_ASSET_ROOT` (or use `just play*`,
which sets it for you).

### Local-first CI (avoid billable runners)

Heavy quality runs **on your machine** via lefthook; GitHub Actions only **verifies** the committed attestation in `.ci/quality-manifest.json` (no `cargo` on the runner — same pattern as phenotype-journey `manifest.verified.json`).

| Step | Command |
|------|---------|
| Install hooks | `lefthook install` |
| Run before push | `lefthook run pre-push` → runs fmt/clippy/test/web/dashboard checks, writes `.ci/quality-manifest.json` |
| Commit manifest | `git add .ci/quality-manifest.json` (staged automatically when hooks pass) |
| Cloud verify (CI) | `just quality-manifest-verify` or workflow job `quality-manifest` |

Optional full sweep on Actions (manual only): **Actions → Quality → Run workflow** (`workflow_dispatch` → `quality-full`).

**PR merge without billable runners:** only `quality-manifest` + `pr-governance-gate` run on pull requests. Legacy workflows (`cargo-deny`, CodeQL, `quality-gate`, etc.) run on `main` push or `workflow_dispatch` only. Add label `local-first-ci` or `ci-billing-exception` on the PR to ignore stale red checks from before this policy.

**`civ-server` protocol** — HTTP on the bind address; WebSocket JSON-RPC at `/ws`.

| Kind | Methods / routes |
|------|------------------|
| HTTP | `GET /healthz` → `{ "tick": <u64> }` · `GET /replay/export` → `.civreplay` (`application/octet-stream`) · `POST /replay/import` → load `.civreplay` bytes into the bridge |
| WS JSON-RPC | `health` · `sim.status` · `sim.snapshot` · `sim.command` (`noop` \| `tick`) · `sim.spawn_civilian` · `sim.spawn_entity` (`kind`: civilian \| vehicle \| airport) · `sim.place_voxel` · `sim.damage` (immediate voxel apply) · replay/policy/speed methods |

**`POST /replay/import`** — replace the live bridge simulation from a raw `.civreplay` body (no filesystem path). Request: `Content-Type: application/octet-stream`. Success: `{ "ok": true, "tick": <u64> }`; invalid bytes → `400`. Updates both the in-memory sim and the bridge tick counter (same state as `GET /healthz`).

**`sim.snapshot`** — read-only view of the in-memory simulation. Params: `{}`. When the bridge can read the sim, result includes `tick`, `population`, `building_count`, and `market_prices`; `energy_budget` and `hash_chain_root` (64-char lowercase hex from the replay hash chain) are included when set. Otherwise returns `{ "tick": <u64> }` only.

**`sim.reset`** — replace the bridge simulation with `Simulation::with_seed`. Params: `{ "seed": <u64> }` (required). Result: `{ "seed": <u64>, "tick": 0 }`. Resets the live world and tick counter.

**`sim.set_policy`** — update `Simulation::economy_policy`. Params: `{ "scarcity_multiplier": <f64> }` (required, ≥ 0); optional `{ "base_consumption_joules": <u64> }`. Result: `{ "updated": true, "scarcity_multiplier": <f64>, "base_consumption_joules": <f64> }` (joules reflect the live policy after apply).

**`sim.set_speed`** — store bridge tick speed multiplier. Params: `{ "multiplier": <u32> }` (0, 1, 2, 4, or 8). Result: `{ "accepted": true, "multiplier": <u32> }`.

**`sim.get_speed`** — read stored multiplier. Params: omit or `{}`. Result: `{ "multiplier": <u32> }`.

**Tick broadcast (10 Hz push)** — not JSON-RPC; the bridge pushes three `Frame3d` values each tick (`VoxelDelta`, `BuildingDiff`, `AgentAppearance`). Wire encoding is set by `WsBridgeConfig::tick_broadcast_format` (`TickBroadcastFormat`):

| Mode | WebSocket frames per tick |
|------|---------------------------|
| `Text` | 3 JSON text frames |
| `Binary` | 3 `F3D0` binary frames |
| `Both` (default) | 3 text frames, then 3 matching binary frames |

Binary layout: `F3D0` magic (4) · kind tag (1: voxel / building / agent) · payload length BE (4) · JSON body (`civ-protocol-3d`). `cargo run -p civ-server` reads `CIVIS_TICK_BROADCAST` (`text` | `binary` | `both`, default `both`). Bevy clients that prefer binary-only tick frames should start the server with `CIVIS_TICK_BROADCAST=binary`. When embedding `run_ws_bridge`, set `tick_broadcast_format` on `WsBridgeConfig` directly.

Examples (send as WebSocket text frames after connecting to `ws://127.0.0.1:3000/ws`):

```json
{"jsonrpc":"2.0","id":1,"method":"health","params":{}}
{"jsonrpc":"2.0","id":2,"method":"sim.snapshot","params":{}}
{"jsonrpc":"2.0","id":3,"method":"sim.reset","params":{"seed":4242}}
{"jsonrpc":"2.0","id":4,"method":"sim.set_policy","params":{"scarcity_multiplier":0.0}}
{"jsonrpc":"2.0","id":5,"method":"sim.set_speed","params":{"multiplier":2}}
{"jsonrpc":"2.0","id":6,"method":"sim.get_speed"}
```

Docs: `cd docs && bun install && bun run docs:dev` · build: `bun run docs:build` · index: `bun run docs:index`

> **Note on case:** the canonical repo name is **`Civis`** (capital C). Some legacy docs reference `civ` or `civis`; treat `Civis` as authoritative.

---

## Where to Start

Read these in order:

1. **[`PRD.md`](./PRD.md)** — product vision, target users, MVP/v1/v2 feature matrix, success criteria.
2. **[`ADR.md`](./ADR.md)** — architectural decisions (Rust + Hecs ECS, fixed-point arithmetic, ChaCha8Rng, WebSocket protocol).
3. **[`PLAN.md`](./PLAN.md)** — phased engineering plan (Phase 0 → 6), DAG dependencies, parallel tracks.
4. **[`FUNCTIONAL_REQUIREMENTS.md`](./FUNCTIONAL_REQUIREMENTS.md)** — FR-traceable requirements; every test references an FR.
5. **[`COMPARISON.md`](./COMPARISON.md)** — how CivLab compares to existing civilization games and engines.
6. **`docs/wiki/`** — concept and architecture deep-dives.
7. **`docs/roadmap/`** — current sequencing and milestones.

For contributors: [`CONTRIBUTING.md`](./CONTRIBUTING.md), [`AGENTS.md`](./AGENTS.md), [`CLAUDE.md`](./CLAUDE.md).

---

## Development

| Task | Command |
|---|---|
| Bevy live-attach smoke (headless, no GPU) | `just civis-3d-live-smoke` — F3D0 WS, `live_*`, `event_feed` / `menus`, protocol extended frames, optional `gpu_features` / `pbr-textures` (`materials`); see `clients/bevy-ref/README.md` |
| Run all Rust tests | `cargo test --workspace` |
| FR-CORE-001 tick budget (10k ticks, release) | `cargo test -p civ-engine --release ten_thousand_ticks_under_budget -- --ignored` |
| Lint (deny warnings) | `cargo clippy --workspace -- -D warnings` |
| Format check | `cargo fmt --check` |
| Local quality gate | `task quality` |
| Build docs | `cd docs && bun run docs:build` |
| Preview docs | `cd docs && bun run docs:dev` |
| Web dashboard (L2 authoring default) | `cargo run -p civ-server` + `cargo run -p civ-watch` (terrain) → `cd web/dashboard && npm run dev` → http://127.0.0.1:5173 — `?spectator=1` for read-only |
| Web tests | `cd web && npm test` |
| CA dirty-chunk benchmark | `just ca-bench` |
| CA dirty-chunk flamegraph | `just ca-flamegraph` |
| CA dirty-chunk perf sweep | `just ca-perf` |
| Screenshot assets | [`docs/guides/screenshot-automation.md`](docs/guides/screenshot-automation.md) |

All tests must reference a Functional Requirement (FR) per `FUNCTIONAL_REQUIREMENTS.md`.

---

## License

Dual-licensed under MIT ([`LICENSE-MIT`](./LICENSE-MIT)) or Apache 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE)) at your option.