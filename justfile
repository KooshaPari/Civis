# Phenotype-org standard justfile

default:
    @just --list

# Build workspace
build:
    cargo build --workspace

# Run tests
test:
    cargo test --workspace

# Lint (clippy + fmt --check)
lint:
    cargo clippy --workspace -- -D warnings
    cargo fmt --check

# Format code
fmt:
    cargo fmt

# Security audits (cargo-deny + cargo-audit)
audit:
    cargo deny check
    cargo audit

# Find unused dependencies
unused:
    cargo machete

# Full local CI sweep (install cargo-deny for audit: cargo install cargo-deny)
ci: lint test audit unused

# Phenotype-aligned alias: Rust + optional infra note in README
quality: civis-3d-verify

# Emit `.ci/quality-manifest.json` after local gates (for cloud CI verification).
quality-manifest:
    bash scripts/quality/emit-quality-manifest.sh

# Verify committed manifest only (same as GitHub Actions quality-manifest job).
quality-manifest-verify:
    bash scripts/quality/verify-quality-manifest.sh

# Generate docs
docs:
    cargo doc --no-deps --workspace

# --- Civis 3D extension targets (feat/civis-3d-foundation) ---

# 3D verification gate: build + test + clippy --all-targets + fmt --check.
# Used by P-V0..P-U1 phase PRs before push.
civis-3d-verify:
    cargo build --workspace
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo fmt --check

# Run the Bevy reference client smoke (headless; meshes one chunk).
civis-3d-bevy-smoke:
    cargo run -p civ-bevy-ref

# Run the Bevy windowed reference client behind the optional bevy feature.
civis-3d-bevy-window:
    cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window

# Run the standalone Bevy client with in-process simulation.
civis-3d-standalone:
    cargo run -p civ-bevy-ref --features bevy --bin civ-standalone

# Run the live Bevy reference client against civ-server's WebSocket bridge.
# Requires civ-server to be running first.
civis-3d-bevy-live:
    cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window

# Run the phenotype-voxel kernel tests (sibling-repo dependency).
civis-3d-voxel-kernel:
    cd ../phenotype-voxel && cargo test

# Run the Civis 3D watch harness and dashboard together.
civis-3d-watch:
    cargo run -p civ-watch &
    cd web/dashboard && bun run dev

# Install and build the dashboard for the watch harness.
civis-3d-watch-build:
    cd web/dashboard && bun install && bun run build

# Godot GDExtension crate (excluded from workspace; test in-tree).
godot-test:
    cd clients/godot-ref/rust && cargo test

# Native infra + sim-server (postgres, dragonfly, nats, minio). Requires process-compose + sh.
infra-up:
    process-compose up

# Rust gate without cargo-deny (when deny is not installed locally).
rust-verify: lint test
