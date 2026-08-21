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

# Standalone cargo-deny gate (mirrors .github/workflows/cargo-deny.yml).
# Runs `cargo deny check`; current cargo-deny discovers workspace members from
# Cargo metadata and does not accept a `--workspace` flag.
deny:
    cargo deny check

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
    python3 scripts/check-jsonrpc-catalog.py

# Scenario YAML + mods validation (civ-engine scenario::* tests).
civis-3d-scenario-check:
    # Single link job avoids intermittent LNK1104 on Windows when other cargo builds run.
    cargo test -p civ-engine scenario --quiet -j 1

civis-3d-web-check:
    # Bound worker creation so the local attestation is stable under macOS
    # process pressure; test semantics and file coverage are unchanged.
    node --test --test-concurrency=1 web/tests/*.test.mjs

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

# 3D verification gate: check + test + clippy --all-targets + voxel fmt --check.
# Uses cargo check (not build) so the gate works when service binaries are
# held open by the running dev stack (Windows exe-lock).
# Used by P-V0..P-U1 phase PRs before push.
civis-3d-verify: civis-3d-catalog-check civis-3d-scenario-check civis-3d-web-check civis-3d-mod-check
    # cargo check avoids exe-lock issues on Windows (service binaries stay open).
    # Targeted tests are already run by sub-recipes above.
    cargo check --workspace
    # Benchmarks stay explicit (`cargo bench --bench ca_dirty_chunk`) and are
    # not part of this bounded attestation gate.
    cargo clippy --workspace --all-targets -- -D warnings
    # Keep this extension gate scoped to the voxel crate; workspace-wide fmt
    # remains a separate parity check and currently includes unrelated client drift.
    cargo fmt --package civ-voxel -- --check

# Programmatic verification harness (verify/pixels/census subcommands).
# `verify` requires the `bevy` feature so the windowed renderer can run.
# `pixels` and `census` run with default features and are safe in headless CI.
# KNOWN-GOOD launch facts (do not hardcode here):
#   * civ-standalone needs `civ-bevy-ref --features bevy,egui` and
#     `BEVY_ASSET_ROOT=<repo>/clients/bevy-ref` (see clients/bevy-ref/README.md).
#   * `civis-census` targets civ-server at ws://$CIV_WS_HOST:$CIV_SERVER_PORT$/$CIV_WS_PATH
#     (defaults: 127.0.0.1:3000/ws) and calls `sim.status` over JSON-RPC.
#
# `with_bevy=1` adds the heavier `cargo check --features bevy` step (the
# `civis-verify` bin pulls in bevy_ecs/wgpu; expect several minutes on a cold
# cache). Default is fast (default features only).
civis-verify with_bevy="":
    @echo "==> civis-cli: cargo check (default features)"
    cargo check -p civis-cli
    @echo "==> civis-cli: cargo test (lib + bins)"
    cargo test -p civis-cli
    powershell -NoProfile -Command "if ('{{with_bevy}}' -eq '1') { Write-Host '==> civis-cli: cargo check --features bevy (verify bin)'; & cargo check -p civis-cli --features bevy --bin civis-verify; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE } } else { Write-Host '==> Skipping bevy check (pass with_bevy=1 to include it).' }"
    @echo "==> Hint: 'cargo run -p civis-cli --bin civis-census' against a live civ-server,"
    @echo "        or 'cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone' with"
    @echo "        BEVY_ASSET_ROOT=clients/bevy-ref for the windowed reference client."

# Feature-gated Bevy client compile oracle (GAP-BEVY-COMPILE-001 / WBS P2.3).
# Default `cargo check --workspace` does not enable optional `bevy`/`egui` features.
bevy-egui-check:
    cargo check -p civ-bevy-ref --features bevy,egui

# SHELL-FEATURES-001 — fuller local play fingerprint: release civ-standalone + audio.
# Builds only (interactive window is awkward to auto-run from just on Windows).
# See clients/bevy-ref/README.md "Local play fingerprints" for run commands.
# Optional live attach env (PowerShell examples echoed after build):
#   CIVIS_ATTACH=server
#   CIV_WS_URL=ws://127.0.0.1:3010/ws?tick_format=binary
#   CIV_SERVER_PORT=3010   (used when CIV_WS_URL unset; default 3000)
civis-bevy-play:
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}" \
        BEVY_ASSET_ROOT="${BEVY_ASSET_ROOT:-$(pwd)/clients/bevy-ref}" \
        cargo build --release -p civ-bevy-ref --features bevy,egui,audio,client-bins --bin civ-standalone
    @echo ""
    @echo "==> civis-bevy-play: release civ-standalone built (bevy,egui,audio)."
    @echo "    Set BEVY_ASSET_ROOT and run the binary from repo root (PowerShell):"
    @echo "      $$env:BEVY_ASSET_ROOT='$(pwd)/clients/bevy-ref'"
    @echo "      $$env:CARGO_TARGET_DIR='${CARGO_TARGET_DIR:-$(pwd)/target}'"
    @echo "      & \"$$env:CARGO_TARGET_DIR/release/civ-standalone.exe\""
    @echo "    Local sandbox: omit attach env vars."
    @echo "    Live attach (civ-server on :3010):"
    @echo "      $$env:CIVIS_ATTACH='server'; $$env:CIV_SERVER_PORT='3010'"
    @echo "      & \"$$env:CARGO_TARGET_DIR/release/civ-standalone.exe\""
    @echo "    Or explicit URL:"
    @echo "      $$env:CIVIS_ATTACH='server'"
    @echo "      $$env:CIV_WS_URL='ws://127.0.0.1:3010/ws?tick_format=binary'"
    @echo "      & \"$$env:CARGO_TARGET_DIR/release/civ-standalone.exe\""

# SHELL-ATTEST-001 — headless AppState / menu smoke (no GPU window).
# Complements bevy-egui-check compile gate with playability wiring proof.
bevy-shell-smoke:
    cargo test -p civ-bevy-ref --features bevy,egui --test shell_attest
    cargo test -p civ-bevy-ref --features bevy,egui menus::tests::
    cargo test -p civ-bevy-ref --features bevy,egui outcome_overlay::tests::

# Run the Bevy reference client smoke (headless; meshes one chunk).
civis-3d-bevy-smoke:
    cargo run -p civ-bevy-ref --bin civ-bevy-ref

# Run the Bevy windowed reference client behind the optional bevy feature.
civis-3d-bevy-window:
    cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window

# Run the standalone Bevy client with in-process simulation.
civis-3d-standalone:
    cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone

# Bounded native launch smoke: build + run civ-standalone for N Update frames
# (default 5), assert preflight + clean AppExit. Needs a GPU adapter.
# Override frames: `just civis-3d-standalone-smoke 12`
civis-3d-standalone-smoke FRAMES="5":
    cargo build -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/native-standalone-smoke.ps1 -Frames {{FRAMES}}

# Standalone client attached to civ-server (requires server running on :3000).
civis-3d-standalone-live:
    powershell -Command "$env:CIVIS_ATTACH='server'; cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone"

# Standalone live attach with explicit WS URL (Tailscale / remote civ-server).
civis-3d-standalone-live-url URL:
    powershell -Command "$env:CIVIS_ATTACH='server'; $env:CIV_WS_URL='{{URL}}'; cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone"

# Headless live-attach protocol smoke (F3D0 + voxel ground; no GPU window).
# P-W1 kickoff item 41 / FR-CIV-BEVY-016; item 47 / FR-CIV-BEVY-022; item 50 / FR-CIV-BEVY-025.
civis-3d-live-smoke:
    cargo test -p civ-server frame_triple
    cargo test -p civ-server --test ws_smoke ws_client_receives_binary_frame3d_after_tick
    cargo test -p civ-bevy-ref --features bevy,egui --lib live_ground::
    cargo test -p civ-bevy-ref --features bevy,egui --lib live_stream::
    cargo test -p civ-bevy-ref --features bevy,egui --lib live_focus::
    cargo test -p civ-bevy-ref --features bevy,egui --lib live_minimap::
    cargo test -p civ-bevy-ref --features bevy,egui --lib live_pick::
    cargo test -p civ-bevy-ref --lib chunk_to_minimap
    cargo test -p civ-bevy-ref --lib minimap_uv_to_chunk
    cargo check -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone
    cargo check -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window

# Run the live Bevy reference client against civ-server's WebSocket bridge.
# Requires civ-server to be running first.
civis-3d-bevy-live:
    cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window

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
    powershell -NoProfile -ExecutionPolicy Bypass -Command 'Set-Item Env:CARGO_TARGET_DIR target-godot-smoke; cargo test --manifest-path clients/godot-ref/rust/Cargo.toml -j 1'

# Full local dev stack: infra + civ-watch.
dev:
    process-compose up

# Tear down the local dev stack.
dev-stop:
    process-compose down

# --- Fast Bevy dev loop (incremental + asset hot-reload + watch) ---
# See docs/development-guide/dev-loop.md for measured compile-time deltas.
# NOTE: `dev`/`dev-stop` above own the infra stack (process-compose); the fast
# Bevy iteration loop lives under `run`/`run-voxel`/`dev-fast`/`dev-fast-voxel`.

# One-shot launch of the standalone sandbox (incremental, no watcher).
run:
    cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-standalone

# One-shot launch of the live voxel/windowed client.
run-voxel:
    cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window

# Install the dev-loop watch tool (cargo-watch) if missing. Idempotent.
dev-tools:
    cargo watch --version > $null 2>&1; if ($LASTEXITCODE -ne 0) { cargo install cargo-watch --locked }

# Fast dev loop: watch sources, rebuild incrementally, asset hot-reload on.
# `hot` feature = dynamic_linking (engine linked as a shared lib) for subsecond
# warm rebuilds. Edit a system -> save -> cargo-watch relinks only our crate.
# Assets (PNG/.glb/WGSL) hot-reload live inside the running process (no rebuild).
dev-fast: dev-tools
    cargo watch -x "run -p civ-bevy-ref --features hot,egui,client-bins --bin civ-standalone"

# Same loop for the live windowed/voxel client.
dev-fast-voxel: dev-tools
    cargo watch -x "run -p civ-bevy-ref --features hot,client-bins --bin civ-bevy-window"

# Build + run the standalone Bevy sandbox (release). Encodes the verified
# boot incantation: playable `bevy,egui,audio` features, a caller-overridable target directory,
# and BEVY_ASSET_ROOT=clients/bevy-ref so the bin
# finds its assets when launched from the workspace root (Bevy 0.18
# `AssetPlugin::file_path` defaults to "./assets" relative to CWD, which is
# the workspace root, not the crate). Both the script and the recipe set the
# env, so callers can use either path and still get the correct asset root.
# Override the target dir by exporting `CARGO_TARGET_DIR` before invoking
# `just play` (the recipe's default is just a default — caller wins).
play:
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}" \
        BEVY_ASSET_ROOT="${BEVY_ASSET_ROOT:-$(pwd)/clients/bevy-ref}" \
        powershell -NoProfile -ExecutionPolicy Bypass -File Tools/play.ps1

# Same as `play` with RUST_LOG=info,civ_bevy_ref=debug,wgpu=warn.
play-debug:
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}" \
        BEVY_ASSET_ROOT="${BEVY_ASSET_ROOT:-$(pwd)/clients/bevy-ref}" \
        powershell -NoProfile -ExecutionPolicy Bypass -File Tools/play.ps1 -LogLevel 'info,civ_bevy_ref=debug,wgpu=warn'

# Same as `play` with RUST_LOG=info,civ_bevy_ref=debug,wgpu=warn and RUST_BACKTRACE=full.
play-trace:
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}" \
        BEVY_ASSET_ROOT="${BEVY_ASSET_ROOT:-$(pwd)/clients/bevy-ref}" \
        powershell -NoProfile -ExecutionPolicy Bypass -File Tools/play.ps1 -LogLevel 'info,civ_bevy_ref=debug,wgpu=warn' -Backtrace full

# Build + run the live windowed Bevy client (civ-bevy-window, F3D0 binary frame
# attach). Mirrors the `play` verified incantation: `bevy,egui` features,
# caller-overridable CARGO_TARGET_DIR and BEVY_ASSET_ROOT=clients/bevy-ref.
# The window client reads the same asset dir as the standalone (sandbox
# terrain fallback + sky HDR + UI panel textures) so the root must be the
# bevy-ref crate.
play-window:
    {{ if os_family() == "windows" { "$env:CARGO_TARGET_DIR = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { \"$(Get-Location)\\target\" }; $env:BEVY_ASSET_ROOT = if ($env:BEVY_ASSET_ROOT) { $env:BEVY_ASSET_ROOT } else { \"$(Get-Location)\\clients\\bevy-ref\" }; cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window" } else { "CARGO_TARGET_DIR=\"${CARGO_TARGET_DIR:-$(pwd)/target}\" BEVY_ASSET_ROOT=\"${BEVY_ASSET_ROOT:-$(pwd)/clients/bevy-ref}\" cargo run -p civ-bevy-ref --features bevy,egui,client-bins --bin civ-bevy-window" } }}

# Kill a running civ-standalone game process.
stop:
    powershell -NoProfile -Command "Get-Process -Name civ-standalone -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue; Write-Host '[stop] civ-standalone stopped.' -ForegroundColor Green"

# Tail the civ-standalone game log (live follow).
logs:
    powershell -NoProfile -Command "Get-Content -LiteralPath '.process-compose/logs/civ-standalone.log' -Wait -Tail 50"

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

# Release build + signing + packaging.
deploy:
    cargo build --release --workspace
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sign-example-mod.ps1 -ModId example-policy
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-policy
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-example-mod.ps1 -ModId example-economic

# Criterion benchmarks.
bench:
    cargo bench --workspace

# CA dirty-chunk benchmark.
ca-bench:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-dirty-chunk-bench.ps1

# CA dirty-chunk profiling.
ca-flamegraph:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-flamegraph.ps1

# CA dirty-chunk perf sweep.
ca-perf:
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ca-perf.ps1

# Rust gate without cargo-deny (when deny is not installed locally).
rust-verify: lint test

# Register/refresh Civis in %APPDATA%/.../Start Menu/Programs/Phenotype Apps/.
# Call after packaging dist/Civis.exe (native launchType in phenotype-tooling apps.json).
register-startmenu:
    pwsh -NoProfile -File C:/Users/koosh/Dev/phenotype-tooling/Tools/Register-StartMenuApps.ps1 -App Civis

# Autograder loop for engine build-green: compile + acceptance tests must pass.
build-green-engine:
    cargo check -p civ-engine
    cargo build --release -p civ-bevy-ref --features "bevy egui client-bins" --bin civ-bevy-window  # REAL playable gate (release build, not check)
