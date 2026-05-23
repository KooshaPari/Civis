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

# Full local CI sweep
ci: lint test audit unused

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

# Run the phenotype-voxel kernel tests (sibling-repo dependency).
civis-3d-voxel-kernel:
    cd ../phenotype-voxel && cargo test
