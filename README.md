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
| **Rendering clients** | Bevy (reference), Unreal, Unity, Godot, Web, Research API |
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

**Prerequisites:**
- Rust 2024 edition (`rustup` recent stable; the workspace `Cargo.toml` pins the edition — install a current `rustup` if `cargo build` complains about edition)
- [Bun](https://bun.sh) for the docs site (`bun --version` ≥ 1.0). **Use Bun only — do not use npm/yarn/pnpm.**
- [Task](https://taskfile.dev) for the local quality gate

```bash
# Clone
git clone https://github.com/KooshaPari/Civis.git
cd Civis

# Build & test simulation core
cargo build --workspace
cargo test --workspace

# Local quality gate (clippy, fmt, tests)
task quality

# Docs site (Bun only)
cd docs
bun install
bun run docs:dev      # local preview
bun run docs:build    # static build
bun run docs:index    # regenerate docs/.generated/doc-index.json
```

> **Note on case:** the canonical repo name is **`Civis`** (capital C). Some legacy docs and scripts reference `civ` or `civis`; treat `Civis` as authoritative.

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
| Lint (deny warnings) | `cargo clippy --workspace -- -D warnings` |
| Format check | `cargo fmt --check` |
| Local quality gate | `task quality` |
| Build docs | `cd docs && bun run docs:build` |
| Preview docs | `cd docs && bun run docs:dev` |

All tests must reference a Functional Requirement (FR) per `FUNCTIONAL_REQUIREMENTS.md`.

---

## License

Dual-licensed under MIT ([`LICENSE-MIT`](./LICENSE-MIT)) or Apache 2.0 ([`LICENSE-APACHE`](./LICENSE-APACHE)) at your option.
