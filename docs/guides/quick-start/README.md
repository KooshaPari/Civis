# Quick Start

This guide brings up the current Civis workspace from a fresh checkout. The
implemented runtime today is a Rust engine plus a smoke server binary; the
WebSocket protocol and long-running service surfaces are still planned.

## Prerequisites

- Rust toolchain compatible with the workspace `rust-toolchain.toml`
- [Bun](https://bun.sh/) for documentation dependencies and VitePress
- [Task](https://taskfile.dev/) for repo quality shortcuts

## Clone

```bash
git clone https://github.com/KooshaPari/Civis.git
cd Civis
```

## Build And Test

```bash
cargo build --workspace
cargo test --workspace
```

## Run The Smoke Server

`civ-server` currently executes one deterministic simulation step and prints the
resulting metrics. It is useful for verifying the engine/server crate wiring.

```bash
cargo run -p civ-server
```

Expected output shape:

```text
tick=1 energy=995000000000 waste=500000000 surplus=990000000000 tyranny=0.005025 legitimacy=0.994975
```

## Run Quality Checks

```bash
task quality
```

`task quality` runs the configured linter and documentation build. If you only
need docs:

```bash
task docs:build
```

## Work On Docs

Use Bun for all docs package work.

```bash
cd docs
bun install
bun run docs:dev
bun run docs:build
```

## Read Next

- [API Reference](/api/) for the currently implemented Rust APIs
- [CIV Specifications](/specs/) for planned simulation and protocol contracts
- [Development Guide](/development-guide/) for contributor workflows
