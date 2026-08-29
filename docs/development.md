---
title: Development
description: Workspace setup, build, test loop, code style, and contribution workflow for Civis.
---

# Development

## Overview

This page describes the day-to-day development loop for Civis: setting up the workspace, building, running tests, linting, and submitting changes. The stack is Rust + Bun (for the VitePress docs) + a small set of supporting tools (just, trunk, cargo).

The repository is a Cargo workspace rooted at the top-level `Cargo.toml`. The Bevy client lives under `clients/bevy-ref`. Mods are loaded at runtime by `crates/mod-host`.

## Prerequisites

- **Rust** — pinned by `rust-toolchain.toml`. Install with [rustup](https://rustup.rs).
- **Bun** — used for the docs site (`docs/`). Install with `npm install -g bun` or via the official installer.
- **just** — task runner used by the included `justfile`. Install with `cargo install just`.
- **trunk** — used by the Bevy client for asset bundling. Install with `cargo install trunk`.
- **Git** — for the contribution workflow.

## Quick Start

```bash
# Clone
git clone https://github.com/KooshaPari/Civis.git
cd Civis

# Build the engine + server + watch + CLI
cargo build --workspace

# Run tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Run the docs site locally
cd docs
bun install
bun run dev
```

The docs site listens on `http://localhost:5173` by default.

## Running the Stack Locally

```bash
# Terminal 1: server
cargo run -p civ-server

# Terminal 2: watch
cargo run -p civ-watch

# Terminal 3: client
cd clients/bevy-ref
trunk serve
```

The client expects `civ-server` on `ws://127.0.0.1:7777/ws` by default and `civ-watch` on `http://127.0.0.1:9090`. Both ports can be overridden via environment variables (`CIVIS_WS_PORT`, `CIV_WATCH_PORT`).

## Workspace Tasks

The included `justfile` defines common targets:

| Task | Description |
|------|-------------|
| `just build` | `cargo build --workspace` |
| `just test` | `cargo test --workspace` |
| `just lint` | `cargo clippy --workspace --all-targets -- -D warnings` |
| `just fmt` | `cargo fmt --all` |
| `just fmt-check` | `cargo fmt --all -- --check` |
| `just docs-dev` | `cd docs && bun run dev` |
| `just docs-build` | `cd docs && bun run build` |

## Code Style

- `cargo fmt` with the project `rustfmt.toml`.
- `cargo clippy -- -D warnings` is enforced in CI.
- Public APIs require doc comments; private items use `//` comments.
- Module organization follows the `crates/<name>/src/lib.rs` convention; submodules live in `crates/<name>/src/<sub>.rs`.
- Prefer `Fixed` arithmetic for any numeric work that crosses the engine boundary.

## Testing

```bash
cargo test --workspace                 # all crates
cargo test -p civ-engine               # engine only
cargo test -p civ-server --test ws_smoke  # WebSocket integration tests
```

Snapshot tests live in `crates/save-db/tests/` and require a checked-in baseline. Update baselines with `UPDATE_SNAPSHOTS=1 cargo test -p save-db`.

## Contribution Workflow

1. Create a feature branch from `main`: `git checkout -b feat/<short-name>`
2. Make focused commits with conventional-commit-style messages:
   - `feat(scope): short summary`
   - `fix(scope): short summary`
   - `docs(scope): short summary`
   - `chore(scope): short summary`
3. Run `just fmt && just lint && just test` before pushing.
4. Open a PR with a body describing the change, the test coverage, and any spec/ADR updates.
5. Address review comments; CI must be green before merge.

## Mod Authoring

Mods are `.civmod` archives (zip + manifest) loaded by `crates/mod-host`. To start authoring:

```bash
# Scaffold a new mod
cargo run -p civis-cli -- mod init --path ./mods/my-mod

# Validate
cargo run -p civis-cli -- mod validate --path ./mods/my-mod

# Publish
curl -X POST http://127.0.0.1:9090/control/mods/publish \
     -H 'Content-Type: application/json' \
     -d '{"path": "./mods/my-mod/dist/my-mod.civmod"}'
```

See the mod-host ABI documentation in `crates/mod-host/README.md` for hook signatures.

## See Also

- [Architecture](/architecture/) — workspace layout and crate boundaries.
- [Simulation](/simulation/) — engine internals for engine contributors.
- [API](/api/) — public APIs when extending server or watch.
- [Deployment](/deployment/) — bringing local changes to a deployed environment.