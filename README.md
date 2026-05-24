> **Pinned references (Phenotype-org)**
> - MSRV: see rust-toolchain.toml
> - cargo-deny config: see deny.toml
> - cargo-audit: rustsec/audit-check@v2 weekly
> - Branch protection: 1 reviewer required, no force-push
> - Authority: phenotype-org-governance/SUPERSEDED.md

# Civis — CivLab

[![Build](https://img.shields.io/github/actions/workflow/status/KooshaPari/Civis/ci.yml?branch=main&label=build)](https://github.com/KooshaPari/Civis/actions)
[![Release](https://img.shields.io/github/v/release/KooshaPari/Civis?include_prereleases&sort=semver)](https://github.com/KooshaPari/Civis/releases)
[![License](https://img.shields.io/github/license/KooshaPari/Civis)](LICENSE)
[![Phenotype](https://img.shields.io/badge/Phenotype-org-blueviolet)](https://github.com/KooshaPari)


**Civis** is the canonical workspace for **CivLab**, a headless, deterministic civilization simulation engine.

CivLab decouples simulation logic from rendering: a Rust simulation core runs headlessly and exposes a client-agnostic protocol (WebSocket JSON-RPC + binary frames) so Bevy, Unreal, Unity, Godot, web, and research clients can attach to the same world timeline simultaneously.

> **Status:** Pre-MVP. See [`PRD.md`](./PRD.md), [`ADR.md`](./ADR.md), and [`PLAN.md`](./PLAN.md) for current scope and phasing.
> **Target:** MVP Q3 2026 → v1 Q4 2026.

---

## What CivLab Is

| Dimension | Choice |
|---|---|
| **Language** | Rust (edition 2024) |
| **Architecture** | ECS via [`hecs`](https://crates.io/crates/hecs) |
| **Determinism** | Fixed-point `i64` @ 10^6 scale; `ChaCha8Rng` seeded once per run; `BTreeMap` for ordered iteration |
| **Tick loop** | Fixed-timestep, 100 ms/tick, sub-16 ms target budget |
| **Protocol** | WebSocket JSON-RPC + binary frames (multi-client) |
| **Rendering clients** | Godot (game UX), Bevy (CI/reference), Unreal (visuals), **Web (spectator/ops only)** — see [ADR-009](docs/adr/ADR-009-web-client-strategy.md) |
| **Replay** | Full event log → bit-identical replay (`.civreplay`) |

CivLab is simultaneously a **game** (RTS-style city/nation building), a **research sandbox** (deterministic, scriptable, full event logs), and a **platform** (multiple renderers attach to one simulation).

See [`COMPARISON.md`](./COMPARISON.md) for how CivLab differs from Dwarf Fortress, Victoria 3, CK3, and Factorio.

---

## Repository Structure

- `src/` — simulation core (Rust workspace)
- `Cargo.toml` — Rust 2024 workspace manifest
- `docs/` — VitePress docs and specification corpus
  - `docs/wiki/` — concept and architecture knowledge
  - `docs/development-guide/` — contributor and implementation guides
  - `docs/roadmap/` — planning and sequencing artifacts
  - `docs/api/` — API and contract documentation
- Root specs: [`PRD.md`](./PRD.md), [`ADR.md`](./ADR.md), [`PLAN.md`](./PLAN.md), [`FUNCTIONAL_REQUIREMENTS.md`](./FUNCTIONAL_REQUIREMENTS.md), [`COMPARISON.md`](./COMPARISON.md)

---

## Quick Start

**Prerequisites:** Rust (edition in `Cargo.toml`), [Bun](https://bun.sh) (docs only; no npm/yarn/pnpm), [Task](https://taskfile.dev).

```bash
git clone https://github.com/KooshaPari/Civis.git && cd Civis
cargo build --workspace && cargo test --workspace
task quality
cargo run -p civ-server   # http://127.0.0.1:3000  (override with CIVIS_WS_ADDR)
```

**`civ-server` protocol** — HTTP on the bind address; WebSocket JSON-RPC at `/ws`.

| Kind | Methods / routes |
|------|------------------|
| HTTP | `GET /healthz` → `{ "tick": <u64> }` · `GET /replay/export` → `.civreplay` (`application/octet-stream`) · `POST /replay/import` → load `.civreplay` bytes into the bridge |
| WS JSON-RPC | `health` · `sim.status` · `sim.snapshot` · `sim.command` (`params.action`: `noop` \| `tick`) · `sim.spawn_civilian` (`x`, `y` normalized 0–1, `faction`) · `sim.place_voxel` (`x`, `y`, `z`, `material`) · `sim.save_replay` · `sim.load_replay` (`params.path`) · `sim.reset` (`params.seed`) · `sim.set_policy` · `sim.set_speed` · `sim.get_speed` |

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
| Run all Rust tests | `cargo test --workspace` |
| FR-CORE-001 tick budget (10k ticks, release) | `cargo test -p civ-engine --release ten_thousand_ticks_under_budget -- --ignored` |
| Lint (deny warnings) | `cargo clippy --workspace -- -D warnings` |
| Format check | `cargo fmt --check` |
| Local quality gate | `task quality` |
| Build docs | `cd docs && bun run docs:build` |
| Preview docs | `cd docs && bun run docs:dev` |
| Web spectator (ADR-009) | `cargo run -p civ-server` then `cd web && npm install && npm run dev` → http://127.0.0.1:5173 |
| Web tests | `cd web && npm test` |
| Screenshot assets | [`docs/guides/screenshot-automation.md`](docs/guides/screenshot-automation.md) |

All tests must reference a Functional Requirement (FR) per `FUNCTIONAL_REQUIREMENTS.md`.

---

## License

Dual-licensed under MIT ([`LICENSE-MIT`](./LICENSE-MIT)) or Apache 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE)) at your option.
