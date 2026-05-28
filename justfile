# Phenotype-org standard justfile

# On Windows use PowerShell so cargo/.cargo/bin is on PATH without extra setup.
set windows-shell := ["powershell", "-NoProfile", "-Command"]

default:
    @just --list

# Build workspace
build:
    cargo build --workspace

# Compile-only check for the workspace.
check:
    cargo check --workspace

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

# Lint + audit + format check.
quality: lint audit
    cargo fmt --check

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

# JSON-RPC method catalog must match jsonrpc.rs (docs/api/jsonrpc-surface.md).
civis-3d-catalog-check:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-jsonrpc-catalog.ps1

# Scenario YAML + mods validation (civ-engine scenario::* tests).
civis-3d-scenario-check:
    # Single link job avoids intermittent LNK1104 on Windows when other cargo builds run.
    cargo test -p civ-engine scenario --quiet -j 1

civis-3d-web-check:
    node --test web/tests/*.test.mjs

civis-3d-mod-check:
    cargo test -p civ-mod-host -p civlab-sdk --quiet

# Build example-policy WASM guest (wasm32-unknown-unknown).
civis-3d-mod-wasm:
    rustup target add wasm32-unknown-unknown
    cargo rustc -p civlab-sdk --release --target wasm32-unknown-unknown --crate-type cdylib
    cp target/wasm32-unknown-unknown/release/civlab_sdk.wasm mods/example-policy/mod.wasm
    cp target/wasm32-unknown-unknown/release/civlab_sdk.wasm mods/example-economic/mod.wasm

# Package example-policy as example-policy.civmod.
civis-3d-mod-package: civis-3d-mod-wasm
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-policy

# Sign example mod.wasm (prints author_pubkey_hex for manifest.toml).
civis-3d-mod-sign MOD="example-policy":
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sign-example-mod.ps1 -ModId {{MOD}}

# Package both example mods for distribution (FR-CIV-TACTICS-059).
civis-3d-mod-package-all: civis-3d-mod-wasm
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-policy
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-economic

# 3D verification gate: check + test + clippy --all-targets + fmt --check.
# Uses cargo check (not build) so the gate works when service binaries are
# held open by the running dev stack (Windows exe-lock).
# Used by P-V0..P-U1 phase PRs before push.
civis-3d-verify: civis-3d-catalog-check civis-3d-scenario-check civis-3d-web-check civis-3d-mod-check
    # cargo check avoids exe-lock issues on Windows (service binaries stay open).
    # Targeted tests are already run by sub-recipes above.
    cargo check --workspace
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
    cargo run -p civ-bevy-ref --features bevy,egui --bin civ-standalone

# Standalone client attached to civ-server (requires server running on :3000).
civis-3d-standalone-live:
    powershell -Command "$env:CIVIS_ATTACH='server'; cargo run -p civ-bevy-ref --features bevy,egui --bin civ-standalone"

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
    cargo test --manifest-path clients/godot-ref/rust/Cargo.toml

# Full local dev stack: infra + civ-watch.
dev:
    process-compose up

# Tear down the local dev stack.
dev-stop:
    process-compose down

# Build and run the Bevy desktop client.
play:
    cargo run -p civ-bevy-ref --features bevy --bin civ-bevy-window

# Build all clients.
build-all:
    cargo build -p civ-bevy-ref
    cargo build --manifest-path clients/godot-ref/rust/Cargo.toml
    powershell -NoProfile -ExecutionPolicy Bypass -File .\clients\unreal-show\scripts\build.ps1

# Run all available tests.
test-all:
    cargo test --workspace
    cargo test --manifest-path clients/godot-ref/rust/Cargo.toml
    cd web && npm test

# Lint, audit, and format checks.
quality: lint audit
    cargo fmt --check

# Release build + signing + packaging.
deploy:
    cargo build --release --workspace
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sign-example-mod.ps1 -ModId example-policy
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-policy
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-economic

# Criterion benchmarks.
bench:
    cargo bench --workspace

# Native infra + civ-watch stack.
infra-up:
    process-compose up

# Rust gate without cargo-deny (when deny is not installed locally).
rust-verify: lint test
