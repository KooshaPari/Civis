# Civ-Sim Ops/Governance Specification

**Status:** Active
**Version:** 1.0.0
**Owner:** CivLab Platform Team
**Last Updated:** 2026-02-21
**Applies To:** civlab-sim crate, civlab-server, civlab-clients (web, mobile, desktop)

---

## Table of Contents

1. [Overview and Scope](#1-overview-and-scope)
2. [Quality Gate System](#2-quality-gate-system)
3. [CI/CD Pipeline](#3-cicd-pipeline)
4. [Spec-First Governance](#4-spec-first-governance)
5. [Versioned Policy and Metric Definitions](#5-versioned-policy-and-metric-definitions)
6. [Runtime Governance and Guardrails](#6-runtime-governance-and-guardrails)
7. [Monitoring and Observability](#7-monitoring-and-observability)
8. [Artifact Integrity](#8-artifact-integrity)
9. [Storage Governance](#9-storage-governance)
10. [Dependency Governance](#10-dependency-governance)
11. [Risk Controls](#11-risk-controls)
12. [Compliance and Auditability](#12-compliance-and-auditability)

---

## 1. Overview and Scope

### 1.1 Operational Governance Philosophy

CivLab's operational governance is built on four axioms:

1. **Determinism is a hard contract, not a best-effort property.** A single non-deterministic output anywhere in the simulation loop is a P0 incident. All governance controls flow from this axiom.
2. **Spec precedes code.** No production code is written without a corresponding functional requirement and test. Governance documents are authoritative; code is a downstream artifact.
3. **Fail loud, never silently.** All error conditions surface as structured log entries, metric counter increments, and where applicable freeze-mode triggers. No fallback paths that hide defects.
4. **Artifacts are signed and auditable.** Every simulation output, scenario config change, and runtime intervention is captured in an immutable audit log with cryptographic integrity guarantees.

### 1.2 Scope

This document governs:

| Component | Scope Included |
|---|---|
| `civlab-sim` | Rust simulation engine crate: tick loop, ECS world, RNG, determinism rules |
| `civlab-server` | JSON-RPC WebSocket server, scenario orchestration, storage layer |
| `civlab-clients` | Web (Pixi.js v8 + React 19), Mobile (TBD), Desktop (Bevy 3D): client-side governance |
| `civlab-mods` | WASM mod sandbox (wasmtime 26.x): sandboxing, resource limits |
| CI/CD pipeline | GitHub Actions workflows, artifact signing, regression suites |
| Storage | SQLite (embedded), PostgreSQL (server), backup and lifecycle policy |
| Monitoring | Prometheus metrics, Grafana dashboards, alerting rules |

Out of scope: billing infrastructure, end-user authentication flows (delegated to WorkOS/AuthKit governance).

### 1.3 Ownership Map

| Domain | Primary Owner | Secondary Owner | Escalation |
|---|---|---|---|
| Simulation engine correctness | Sim Team Lead | Rust Guild | Platform CTO |
| CI/CD pipeline | Platform Team | DevOps | Platform CTO |
| Schema governance | Sim Architect | Spec Team | Platform CTO |
| Storage and backup | Infra Team | Platform Team | Platform CTO |
| Security and dependency audit | Security Guild | Platform Team | CISO |
| Monitoring and alerting | Platform Team | Sim Team Lead | On-call engineer |
| Mod sandbox policy | Security Guild | Sim Architect | CISO |

### 1.4 Governance Layers

Governance flows through four ordered layers. A violation at any layer is a blocking defect:

```
+---------------------------------------------------------+
|  Layer 1: SPEC LAYER                                    |
|  PRD.md, FUNCTIONAL_REQUIREMENTS.md, ADR.md, this doc  |
|  Authoritative. All downstream layers must conform.     |
+---------------------------------------------------------+
|  Layer 2: CODE LAYER                                    |
|  Rust impl, JSON Schema, TOML configs, Taskfile.yml     |
|  Must satisfy all FR SHALL statements.                  |
|  Gate: clippy -D warnings, rustfmt, schema validators   |
+---------------------------------------------------------+
|  Layer 3: RUNTIME LAYER                                 |
|  Tick loop, ECS world, guardrails, freeze mode          |
|  Must satisfy determinism rules D1-D7.                  |
|  Gate: BLAKE3 hash chain, double-run, seed-sweep tests  |
+---------------------------------------------------------+
|  Layer 4: ARTIFACT LAYER                                |
|  Signed reports, replay bundles, audit logs             |
|  Must be tamper-evident, reproducible, versioned.       |
|  Gate: Ed25519 signatures, retention policy compliance  |
+---------------------------------------------------------+
```

Changes travel top-to-bottom: spec change triggers code change triggers runtime re-validation triggers re-signed artifacts. A code change without a corresponding spec change is a governance violation and will be flagged by pre-commit hooks.

### 1.5 Determinism Rules Reference (D1-D7)

The following rules are referenced throughout this document. Each quality gate maps to one or more of these rules:

| Rule ID | Name | Description |
|---|---|---|
| D1 | Pure Functions | All tick-advancing systems are pure functions of (World, Tick, Seed). No hidden state. |
| D2 | No System Time | `std::time::SystemTime`, `std::time::Instant`, and all wall-clock reads are forbidden in sim code. |
| D3 | No Float Comparison | Floating-point equality or ordering in game logic is forbidden. All rates use `FixedI32\<U16\>`; all energy/GDP use `i64` newtypes. |
| D4 | No Global Mutable State | No `static mut`, no `OnceLock` that mutates after initialization, no thread-local state in sim code. |
| D5 | Deterministic Ordering | ECS system ordering is declared explicit and total. No reliance on hash map iteration order in output-affecting code. |
| D6 | Seeded RNG Only | Only `ChaCha20Rng` seeded from the scenario config is permitted. No `rand::thread_rng()` or OS entropy sources in sim code. |
| D7 | No I/O in Tick Loop | No filesystem, network, or database access inside the 100ms tick loop. All I/O is pre-loaded or post-processed. |

---

## 2. Quality Gate System

### 2.1 Gate Overview

Quality gates are enforced at three checkpoints: pre-commit (local), CI (pull request), and nightly (regression). A gate failure at any checkpoint is a hard blocker with no bypass without a documented exception approved by the primary owner.

| Gate ID | Name | Checkpoint | Blocks |
|---|---|---|---|
| QG-01 | Schema Validation | pre-commit + CI | PR merge |
| QG-02 | Determinism Double-Run | CI | PR merge |
| QG-03 | Seed-Sweep Determinism | CI nightly | Nightly green |
| QG-04 | Cross-Platform Hash Parity | CI nightly | Nightly green |
| QG-05 | Integration Test Matrix | CI | PR merge |
| QG-06 | Replay Consistency | CI | PR merge |
| QG-07 | Tick Latency Gate | CI | PR merge |
| QG-08 | Memory Ceiling Gate | CI | PR merge |
| QG-09 | Lint and Static Analysis | pre-commit + CI | PR merge |
| QG-10 | Dependency Audit | CI weekly | Weekly green |

### 2.2 Schema Validation (QG-01)

#### 2.2.1 JSON Schema Coverage

Every external input to the simulation is validated against a versioned JSON Schema before entering the simulation layer. Schema files live in `schemas/` at the repository root:

```
schemas/
  scenario/
    v1/
      scenario.schema.json       # Top-level scenario config schema
      terrain.schema.json        # Hex grid terrain descriptor
      civilization.schema.json   # Starting civ configuration
      policy.schema.json         # Policy bundle schema
      mod-manifest.schema.json   # WASM mod manifest
  rpc/
    v1/
      request.schema.json        # JSON-RPC request envelope
      response.schema.json       # JSON-RPC response envelope
      notifications.schema.json  # Server-push notification shapes
  metrics/
    v1/
      metric-definition.schema.json  # Metric definition format
```

All schemas include `$schema`, `$id`, `title`, `description`, and `version` fields. Breaking changes to any schema require a version directory bump (`v1` to `v2`) and a corresponding ADR.

#### 2.2.2 Custom Rust Validators

JSON Schema handles structural validation. Domain invariants are enforced by a Rust `ScenarioValidator` that runs after JSON Schema passes:

```rust
// crates/civlab-sim/src/validation/scenario.rs

pub struct ScenarioValidator;

impl ScenarioValidator {
    /// Validates all domain invariants that JSON Schema cannot express.
    /// Returns a Vec<ValidationError>; empty means valid.
    /// @trace FR-VAL-001
    pub fn validate(scenario: &ScenarioConfig) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        Self::validate_seed_range(scenario, &mut errors);
        Self::validate_hex_grid_bounds(scenario, &mut errors);
        Self::validate_fixed_point_budgets(scenario, &mut errors);
        Self::validate_policy_balance(scenario, &mut errors);
        Self::validate_mod_manifest_hashes(scenario, &mut errors);
        errors
    }

    fn validate_seed_range(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Seed 0 is reserved for internal testing; production scenarios must use seed > 0
        if s.rng_seed == 0 {
            errors.push(ValidationError::new(
                "rng_seed",
                "seed 0 is reserved for testing; use seed >= 1 in production",
            ));
        }
    }

    fn validate_hex_grid_bounds(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        const MAX_RADIUS: u32 = 512;
        if s.hex_grid.radius > MAX_RADIUS {
            errors.push(ValidationError::new(
                "hex_grid.radius",
                format!("radius {} exceeds maximum {}", s.hex_grid.radius, MAX_RADIUS),
            ));
        }
    }

    fn validate_fixed_point_budgets(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Starting energy must be representable as KiloJoules (i64, non-negative)
        if s.starting_energy_kj < 0 {
            errors.push(ValidationError::new(
                "starting_energy_kj",
                "must be non-negative",
            ));
        }
        // Starting GDP must be representable as MilliCredits (i64, non-negative)
        if s.starting_gdp_mc < 0 {
            errors.push(ValidationError::new(
                "starting_gdp_mc",
                "must be non-negative",
            ));
        }
    }

    fn validate_policy_balance(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        // Sum of all policy allocation weights must equal exactly 1_000_000 ppm
        // (parts per million = fixed-point 1.0)
        let weight_sum: i64 = s.policies.iter().map(|p| p.weight_ppm).sum();
        if weight_sum != 1_000_000 {
            errors.push(ValidationError::new(
                "policies",
                format!(
                    "policy weights sum to {} ppm; must equal exactly 1_000_000",
                    weight_sum
                ),
            ));
        }
    }

    fn validate_mod_manifest_hashes(s: &ScenarioConfig, errors: &mut Vec<ValidationError>) {
        for m in &s.mods {
            if m.blake3_hash.is_empty() {
                errors.push(ValidationError::new(
                    "mods[].blake3_hash",
                    format!("mod '{}' missing required BLAKE3 hash", m.id),
                ));
            }
        }
    }
}
```

The validator is called from both the server's scenario-load path and from the `validate-scenario` Taskfile target.

#### 2.2.3 Taskfile Targets for QG-01

```yaml
# Taskfile.yml (excerpt: schema validation targets)

version: '3'

tasks:
  validate-scenario:
    desc: "Validate a scenario config file against JSON Schema and Rust domain validators"
    cmds:
      - cargo run -p civlab-cli -- validate-scenario --file {{.FILE}}
    requires:
      vars: [FILE]

  validate-schemas:
    desc: "Validate all schema files in schemas/ for JSON Schema correctness"
    cmds:
      - npx ajv-cli validate --allow-union-types -s "schemas/**/*.schema.json"
    preconditions:
      - sh: "command -v npx"
        msg: "npx must be installed"

  validate-all:
    desc: "Run all schema validation checks (schemas + sample fixtures)"
    deps: [validate-schemas]
    cmds:
      - |
        for fixture in tests/fixtures/scenarios/*.toml; do
          echo "Validating $fixture..."
          cargo run -p civlab-cli -- validate-scenario --file "$fixture"
        done
```

### 2.3 Determinism Test Harness (QG-02, QG-03, QG-04)

#### 2.3.1 Double-Run Check (QG-02)

The double-run check runs every scenario in the test fixture set twice with identical seeds and asserts byte-identical output. This catches any non-determinism introduced by OS scheduling, allocator layout, or hidden shared state.

```rust
// crates/civlab-sim/tests/determinism_double_run.rs
// @trace FR-DET-002

use civlab_sim::{SimulationEngine, ScenarioConfig};
use std::path::PathBuf;

fn load_fixture(name: &str) -> ScenarioConfig {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/scenarios")
        .join(name);
    ScenarioConfig::from_toml_file(&path).expect("fixture must parse")
}

/// Runs a scenario for n ticks and returns the final BLAKE3 hash chain root.
fn run_n_ticks(config: &ScenarioConfig, n: u64) -> [u8; 32] {
    let mut engine = SimulationEngine::new(config.clone());
    for _ in 0..n {
        engine.tick();
    }
    engine.hash_chain_root()
}

#[test]
fn determinism_double_run_standard_scenario() {
    let config = load_fixture("standard_100hex.toml");
    let run_a = run_n_ticks(&config, 1_000);
    let run_b = run_n_ticks(&config, 1_000);
    assert_eq!(run_a, run_b, "double-run mismatch: non-determinism detected");
}

#[test]
fn determinism_double_run_large_scenario() {
    let config = load_fixture("large_512hex.toml");
    let run_a = run_n_ticks(&config, 500);
    let run_b = run_n_ticks(&config, 500);
    assert_eq!(run_a, run_b, "double-run mismatch on large scenario");
}

#[test]
fn determinism_double_run_with_mods() {
    let config = load_fixture("with_mods_standard.toml");
    let run_a = run_n_ticks(&config, 200);
    let run_b = run_n_ticks(&config, 200);
    assert_eq!(run_a, run_b, "double-run mismatch with active mods");
}
```

#### 2.3.2 Seed-Sweep Test (QG-03)

The seed-sweep test runs a reduced scenario (50 ticks) across 256 distinct seeds and asserts that each seed produces a unique but internally consistent hash chain. This detects seed-contamination bugs where one run's RNG state bleeds into another.

```rust
// crates/civlab-sim/tests/determinism_seed_sweep.rs
// @trace FR-DET-006

#[test]
fn seed_sweep_256_seeds_no_cross_contamination() {
    let base_config = load_fixture("minimal_seed_sweep.toml");
    let mut results: std::collections::HashMap<[u8; 32], u64> = Default::default();

    for seed in 1u64..=256 {
        let mut config = base_config.clone();
        config.rng_seed = seed;
        let hash = run_n_ticks(&config, 50);

        if let Some(prior_seed) = results.get(&hash) {
            panic!(
                "seed {} produced same hash as seed {} — cross-contamination detected",
                seed, prior_seed
            );
        }
        results.insert(hash, seed);
    }
    assert_eq!(results.len(), 256, "expected 256 unique hashes for 256 seeds");
}

#[test]
fn seed_sweep_reproducibility_spot_check() {
    // Pick 16 seeds, run twice each, verify identical hash
    let base_config = load_fixture("minimal_seed_sweep.toml");
    for seed in [
        1u64, 7, 42, 100, 128, 200, 255, 256,
        512, 1000, 9999, 65535, 100_000, 1_000_000, u64::MAX / 2, u64::MAX - 1,
    ] {
        let mut config = base_config.clone();
        config.rng_seed = seed;
        let run_a = run_n_ticks(&config, 50);
        let run_b = run_n_ticks(&config, 50);
        assert_eq!(run_a, run_b, "seed {} not reproducible", seed);
    }
}
```

#### 2.3.3 Cross-Platform Hash Parity (QG-04)

The nightly CI matrix runs the double-run test suite on Linux x86_64, macOS arm64, and Windows x86_64. A GitHub Actions artifact uploads the hash chain output from each platform. A post-matrix comparison job asserts all three outputs are byte-identical.

```yaml
# .github/workflows/nightly-cross-platform.yml (excerpt: comparison job)

  cross-platform-hash-compare:
    needs: [test-linux, test-macos, test-windows]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: hash-chain-output-*
          merge-multiple: true

      - name: Compare hash chain outputs across platforms
        run: |
          LINUX=$(cat hash-chain-output-linux/standard_100hex_1000ticks.hex)
          MACOS=$(cat hash-chain-output-macos/standard_100hex_1000ticks.hex)
          WINDOWS=$(cat hash-chain-output-windows/standard_100hex_1000ticks.hex)
          if [ "$LINUX" != "$MACOS" ] || [ "$LINUX" != "$WINDOWS" ]; then
            echo "CROSS-PLATFORM DETERMINISM FAILURE"
            echo "Linux:   $LINUX"
            echo "macOS:   $MACOS"
            echo "Windows: $WINDOWS"
            exit 1
          fi
          echo "Cross-platform hash parity: PASS"
```

#### 2.3.4 Taskfile Targets for Determinism

```yaml
  det-double-run:
    desc: "Run determinism double-run tests (QG-02)"
    cmds:
      - cargo test -p civlab-sim --test determinism_double_run -- --nocapture

  det-seed-sweep:
    desc: "Run seed-sweep determinism tests — slow, nightly only (QG-03)"
    cmds:
      - cargo test -p civlab-sim --test determinism_seed_sweep -- --nocapture

  det-all:
    desc: "Run all local determinism tests"
    deps: [det-double-run]
    cmds:
      - echo "Seed sweep runs in CI nightly only; use det-seed-sweep to run locally"
```

### 2.4 Integration Test Matrix (QG-05)

The integration test matrix covers all D1-D7 rules with at least two targeted test cases per rule. Tests carry a `// @trace FR-DET-NNN` annotation for FR traceability.

| Rule | Test File | Test Count | FR Trace |
|---|---|---|---|
| D1 Pure Functions | `tests/integration/d1_pure_functions.rs` | 4 | FR-DET-001 |
| D2 No System Time | `tests/integration/d2_no_system_time.rs` | 3 | FR-DET-002 |
| D3 No Float Compare | `tests/integration/d3_no_float_compare.rs` | 5 | FR-DET-003 |
| D4 No Global Mut | `tests/integration/d4_no_global_mut.rs` | 3 | FR-DET-004 |
| D5 Deterministic Ordering | `tests/integration/d5_deterministic_ordering.rs` | 4 | FR-DET-005 |
| D6 Seeded RNG | `tests/integration/d6_seeded_rng.rs` | 6 | FR-DET-006 |
| D7 No I/O in Tick | `tests/integration/d7_no_io_in_tick.rs` | 3 | FR-DET-007 |

Example D7 integration test asserting the tick function does not invoke I/O syscalls:

```rust
// crates/civlab-sim/tests/integration/d7_no_io_in_tick.rs
// @trace FR-DET-007

#[cfg(target_os = "linux")]
#[test]
fn d7_tick_loop_no_io_syscalls() {
    // Install a seccomp filter that panics on any read/write/open syscall
    // during the tick execution window. This is Linux-only and CI-enforced.
    let config = load_fixture("minimal_seed_sweep.toml");
    let mut engine = SimulationEngine::new(config);

    civlab_test_utils::with_io_syscall_trap(|| {
        engine.tick(); // Must complete without triggering the trap
    });
}
```

```yaml
  test-integration-matrix:
    desc: "Run full D1-D7 integration test matrix (QG-05)"
    cmds:
      - cargo test -p civlab-sim --tests -- --nocapture 2>&1 | tee /tmp/integration-matrix.log
      - |
        if grep -q "FAILED" /tmp/integration-matrix.log; then
          echo "Integration test failures:"
          grep "FAILED" /tmp/integration-matrix.log
          exit 1
        fi
        echo "Integration matrix: PASS"
```

### 2.5 Replay Consistency (QG-06)

#### 2.5.1 BLAKE3 Hash Chain

Every tick produces a BLAKE3 hash of the serialized ECS world state. Each hash chains into the next by including the prior hash as input:

```rust
// crates/civlab-sim/src/hash_chain.rs
// @trace FR-REP-001

use blake3::Hasher;

pub struct HashChain {
    current: [u8; 32],
    tick: u64,
}

impl HashChain {
    pub fn new(seed: u64) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(b"civlab-sim-v1");
        hasher.update(&seed.to_le_bytes());
        Self {
            current: hasher.finalize().into(),
            tick: 0,
        }
    }

    /// Advance the chain by hashing the current world state snapshot.
    /// `world_snapshot` is the canonical deterministic serialization of the ECS world.
    pub fn advance(&mut self, world_snapshot: &[u8]) -> [u8; 32] {
        let mut hasher = Hasher::new();
        hasher.update(&self.current);             // chain link
        hasher.update(&self.tick.to_le_bytes());  // tick counter prevents hash reuse
        hasher.update(world_snapshot);            // world state
        let next = hasher.finalize().into();
        self.current = next;
        self.tick += 1;
        next
    }

    pub fn root(&self) -> [u8; 32] {
        self.current
    }
}
```

#### 2.5.2 Replay Verification Test

```rust
// crates/civlab-sim/tests/replay_consistency.rs
// @trace FR-REP-001

#[test]
fn replay_produces_identical_hash_chain() {
    let config = load_fixture("standard_100hex.toml");
    let tick_count = 1_000usize;

    // Initial run: capture per-tick hashes
    let mut engine = SimulationEngine::new(config.clone());
    let mut initial_hashes: Vec<[u8; 32]> = Vec::with_capacity(tick_count);
    for _ in 0..tick_count {
        engine.tick();
        initial_hashes.push(engine.current_hash());
    }

    // Replay run: reconstruct from replay bundle (tick-0 state + seed) and re-execute
    let replay_bundle = engine.export_replay_bundle();
    let mut replay_engine = SimulationEngine::from_replay_bundle(&replay_bundle);
    let mut replay_hashes: Vec<[u8; 32]> = Vec::with_capacity(tick_count);
    for _ in 0..tick_count {
        replay_engine.tick();
        replay_hashes.push(replay_engine.current_hash());
    }

    assert_eq!(
        initial_hashes, replay_hashes,
        "replay hash chain diverged from original run"
    );
}
```

#### 2.5.3 Tick-by-Tick Comparison Tool

The `civlab-cli replay-diff` command performs tick-by-tick comparison between two replay bundles and reports the first divergence point:

```bash
civlab-cli replay-diff \
  --bundle-a replays/run-001.civreplay \
  --bundle-b replays/run-002.civreplay \
  --output divergence-report.json
```

Output format:

```json
{
  "status": "diverged",
  "first_divergent_tick": 847,
  "bundle_a_hash_at_tick": "a3f7c2b1e9d045f88c2a1b3d6e9f012c3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8",
  "bundle_b_hash_at_tick": "9d4e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9da",
  "identical_ticks": 846,
  "total_ticks_compared": 1000
}
```

### 2.6 Performance Gates (QG-07, QG-08)

#### 2.6.1 Tick Latency Targets

| Scenario Size | p50 Target | p99 Target | Hard Ceiling |
|---|---|---|---|
| Small (radius <= 50) | <= 5ms | <= 15ms | 30ms |
| Standard (radius <= 150) | <= 20ms | <= 60ms | 100ms |
| Large (radius <= 300) | <= 60ms | <= 90ms | 100ms |
| Maximum (radius <= 512) | <= 80ms | <= 95ms | 100ms |

The hard ceiling of 100ms is the tick loop budget. Exceeding it triggers a `civlab_sim_tick_budget_exceeded_total` counter increment. Three consecutive violations trigger freeze mode.

#### 2.6.2 Memory Ceiling

| Resource | Per-Simulation Limit | Server-Wide Limit |
|---|---|---|
| ECS World heap | 512 MB | — |
| WASM mod memory | 64 MB per mod | 256 MB total per simulation |
| Replay buffer | 128 MB | — |
| Hash chain buffer | 4 MB | — |

Memory usage is sampled every 10 ticks and exported as `civlab_sim_memory_bytes{component="ecs_world"}`.

```yaml
  bench-tick-latency:
    desc: "Run tick latency benchmarks and fail if thresholds are missed (QG-07)"
    cmds:
      - cargo bench -p civlab-sim --bench tick_latency -- --output-format bencher | tee /tmp/bench-output.txt
      - cargo run -p civlab-cli -- check-bench-thresholds --input /tmp/bench-output.txt --thresholds bench-thresholds.toml
```

`bench-thresholds.toml`:

```toml
[small]
p50_ms = 5
p99_ms = 15
ceiling_ms = 30

[standard]
p50_ms = 20
p99_ms = 60
ceiling_ms = 100

[large]
p50_ms = 60
p99_ms = 90
ceiling_ms = 100

[maximum]
p50_ms = 80
p99_ms = 95
ceiling_ms = 100
```

---

## 3. CI/CD Pipeline

### 3.1 Pipeline Overview

```
On Pull Request (required to merge):
  fmt-check -> clippy -> test-unit -> test-integration
      -> schema-validate -> det-double-run -> replay-check
      -> bench-gate -> dep-audit -> artifact-sign-verify

Nightly (required for nightly green badge):
  full-matrix-build -> det-seed-sweep -> cross-platform-hash
      -> dep-audit -> cargo-deny -> security-scan

On Release Tag (required to publish):
  all-PR-checks -> full-matrix-build -> artifact-sign
      -> sbom-generate -> release-publish
```

### 3.2 GitHub Actions Workflow (PR)

```yaml
# .github/workflows/ci.yml

name: CI

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  fmt-check:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test-unit:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --lib -- --nocapture
      - run: cargo test -p civlab-server --lib -- --nocapture

  test-integration:
    name: Integration Tests (D1-D7)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --tests -- --nocapture 2>&1 | tee integration-results.txt
      - name: Assert no D1-D7 test failures
        run: |
          if grep -q "FAILED" integration-results.txt; then
            echo "Integration test failures detected:"
            grep "FAILED" integration-results.txt
            exit 1
          fi

  schema-validate:
    name: Schema Validation (QG-01)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with:
          node-version: '22'
      - run: npm ci
      - run: task validate-schemas
      - run: task validate-all

  det-double-run:
    name: Determinism Double-Run (QG-02)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: task det-double-run

  replay-check:
    name: Replay Consistency (QG-06)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-sim --test replay_consistency -- --nocapture

  bench-gate:
    name: Performance Gate (QG-07/08)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: task bench-tick-latency

  dep-audit:
    name: Dependency Audit (QG-10)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit --locked
      - run: cargo audit
      - run: npm audit --audit-level=high
        working-directory: clients/web

  artifact-sign-verify:
    name: Artifact Signing Self-Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test -p civlab-server --test artifact_signing -- --nocapture
```

### 3.3 Nightly Regression Suite

```yaml
# .github/workflows/nightly.yml

name: Nightly Regression

on:
  schedule:
    - cron: '0 2 * * *'   # 02:00 UTC daily
  workflow_dispatch:

jobs:
  test-linux:
    name: Full Suite Linux x86_64
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - run: task det-seed-sweep
      - name: Export hash chain output
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-linux/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-linux
          path: hash-chain-output-linux/

  test-macos:
    name: Full Suite macOS arm64
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - name: Export hash chain output
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-macos/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-macos
          path: hash-chain-output-macos/

  test-windows:
    name: Full Suite Windows x86_64
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all -- --nocapture
      - name: Export hash chain output
        shell: bash
        run: |
          cargo run -p civlab-cli -- export-hash-chain \
            --fixture tests/fixtures/scenarios/standard_100hex.toml \
            --ticks 1000 \
            --output hash-chain-output-windows/standard_100hex_1000ticks.hex
      - uses: actions/upload-artifact@v4
        with:
          name: hash-chain-output-windows
          path: hash-chain-output-windows/

  cross-platform-hash-compare:
    name: Cross-Platform Hash Parity (QG-04)
    needs: [test-linux, test-macos, test-windows]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: hash-chain-output-*
          merge-multiple: true
      - name: Compare outputs
        run: |
          LINUX=$(cat hash-chain-output-linux/standard_100hex_1000ticks.hex)
          MACOS=$(cat hash-chain-output-macos/standard_100hex_1000ticks.hex)
          WINDOWS=$(cat hash-chain-output-windows/standard_100hex_1000ticks.hex)
          echo "Linux:   $LINUX"
          echo "macOS:   $MACOS"
          echo "Windows: $WINDOWS"
          if [ "$LINUX" != "$MACOS" ] || [ "$LINUX" != "$WINDOWS" ]; then
            echo "FATAL: Cross-platform determinism failure"
            exit 1
          fi
          echo "PASS: All platforms produce identical hash chain"

  cargo-deny:
    name: License and Advisory Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v1
        with:
          command: check all

  security-scan:
    name: SAST Security Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run semgrep
        uses: semgrep/semgrep-action@v1
        with:
          config: p/rust p/secrets p/owasp-top-ten
```

### 3.4 Required Checks Before Merge

The following GitHub branch protection rules are enforced on `main` and `develop`:

| Check | Required | Dismiss Stale Reviews |
|---|---|---|
| fmt-check | yes | yes |
| clippy | yes | yes |
| test-unit | yes | yes |
| test-integration | yes | yes |
| schema-validate | yes | yes |
| det-double-run | yes | yes |
| replay-check | yes | yes |
| bench-gate | yes | yes |
| dep-audit | yes | yes |
| artifact-sign-verify | yes | yes |
| PR review (1 approver minimum) | yes | yes |

Direct pushes to `main` are forbidden. Force-push is disabled on `main` and `develop`. All changes via pull request.

### 3.5 Artifact Signing Pipeline (Release)

```yaml
# .github/workflows/release.yml (excerpt: signing)

  sign-artifacts:
    name: Sign Release Artifacts
    needs: [full-matrix-build]
    runs-on: ubuntu-latest
    permissions:
      id-token: write
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          pattern: build-*
          merge-multiple: true

      - name: Install cosign (keyless OIDC signing)
        uses: sigstore/cosign-installer@v3

      - name: Sign all artifacts
        run: |
          for artifact in dist/*; do
            cosign sign-blob \
              --yes \
              --bundle "${artifact}.cosign.bundle" \
              "$artifact"
            echo "Signed: $artifact"
          done

      - name: Generate SBOM with syft
        run: |
          curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh \
            | sh -s -- -b /usr/local/bin
          syft packages . -o spdx-json > dist/civlab-sim.sbom.spdx.json

      - uses: actions/upload-artifact@v4
        with:
          name: signed-release-artifacts
          path: dist/
```

---

## 4. Spec-First Governance

### 4.1 Spec-First Development Process

All production code changes follow this mandatory sequence. Skipping any step is a governance violation caught by pre-commit hooks:

```
Step 1: SPEC UPDATE
  Update or create spec document (PRD, FR, ADR as applicable).
  Spec must be merged to main before code begins.

Step 2: FUNCTIONAL REQUIREMENT
  FR SHALL statement added to FUNCTIONAL_REQUIREMENTS.md.
  FR ID assigned: FR-{CAT}-{NNN}
  Acceptance criteria written: testable, unambiguous.

Step 3: TEST FIRST
  Test file created with // @trace FR-{CAT}-{NNN} annotation.
  Test is failing (red) before any impl code is written.
  Test name matches FR acceptance criterion.

Step 4: IMPLEMENTATION
  Code written to make test pass.
  Code references FR ID in doc comment.
  No code beyond what is required to pass the test.

Step 5: REVIEW
  PR created with spec link and FR ID in description.
  Reviewer checks spec-to-code traceability.
  All CI checks pass.

Step 6: MERGE
  Squash merge to main (preserves atomic FR-to-code history).
  Spec tracker updated (FR status -> IMPLEMENTED).
```

### 4.2 ADR Requirement Triggers

An Architecture Decision Record is required (blocking merge) for any change meeting one or more of the following criteria:

| Trigger | Example | ADR Template |
|---|---|---|
| New external crate dependency | Adding `serde_arrow` to Cargo.toml | `docs/adr/templates/new-dependency.md` |
| Change to determinism rules D1-D7 | Relaxing D3 for a specific subsystem | `docs/adr/templates/determinism-change.md` |
| Schema breaking change | Renaming a field in `scenario.schema.json` | `docs/adr/templates/schema-change.md` |
| Storage engine change | Migrating from SQLite to DuckDB for analytics | `docs/adr/templates/storage-change.md` |
| New client platform | Adding iOS native client | `docs/adr/templates/new-client.md` |
| RPC protocol change | Adding binary framing to JSON-RPC | `docs/adr/templates/protocol-change.md` |
| Tick loop architecture change | Moving from Bevy ECS to a custom scheduler | `docs/adr/templates/arch-change.md` |
| Fixed-point type change | Switching KiloJoules from i64 to i128 | `docs/adr/templates/type-change.md` |
| Mod sandbox policy change | Increasing WASM memory limit from 64 MB | `docs/adr/templates/mod-policy-change.md` |
| New cryptographic primitive | Replacing BLAKE3 with SHA3-512 | `docs/adr/templates/crypto-change.md` |

ADR format:

```markdown
# ADR-{NNN}: {Title}

**Status:** Proposed | Accepted | Deprecated | Superseded by ADR-{NNN}
**Date:** YYYY-MM-DD
**Owner:** {Name}
**Deciders:** {Names}

## Context
What is the situation? What problem are we solving?

## Decision
What is the decision? State it concisely.

## Consequences

### Positive

### Negative

### Neutral

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|

## Implementation Notes
Key implementation constraints or gotchas.

## Traceability
- FR IDs: FR-{CAT}-{NNN}
- Related ADRs: ADR-{NNN}
```

### 4.3 Spec Versioning

#### 4.3.1 Frontmatter Fields

All spec documents include standard frontmatter:

```yaml
---
title: "Civ-Sim Ops/Governance Specification"
version: "1.0.0"           # semver: MAJOR.MINOR.PATCH
status: "active"            # draft | active | deprecated | superseded
owner: "CivLab Platform Team"
last_updated: "2026-02-21"
breaking_change: false      # true if this version breaks downstream consumers
supersedes: null            # version this replaces, if any
review_date: "2026-08-21"   # scheduled review date (6-month default)
---
```

#### 4.3.2 Semver for Breaking Changes

| Change Type | Version Bump | ADR Required | Migration Guide Required |
|---|---|---|---|
| Additive (new section, new optional field) | MINOR | no | no |
| Clarification (no behavior change) | PATCH | no | no |
| Breaking (renamed field, removed section, changed semantics) | MAJOR | yes | yes |
| Deprecation (marks for removal in next MAJOR) | MINOR | no | yes |

#### 4.3.3 Change Review Checklist

PR template for spec changes enforces the following reviewer checklist:

- [ ] Version bump matches the nature of the change
- [ ] All frontmatter fields present and valid
- [ ] ADR linked if required by Section 4.2
- [ ] Downstream impact documented in PR description
- [ ] FR tracker updated if this change closes or modifies an FR
- [ ] Migration guide written if MAJOR bump

---

## 5. Versioned Policy and Metric Definitions

### 5.1 Simulation Policy Schema

Scenario configs are versioned TOML files. The canonical schema is `schemas/scenario/v1/scenario.schema.json`. The TOML structure:

```toml
# tests/fixtures/scenarios/standard_100hex.toml

[meta]
schema_version = "1.0.0"
scenario_id = "standard-100hex-v1"
name = "Standard 100-Hex Scenario"
description = "Baseline governance fixture for CI determinism tests"
created_at = "2026-02-21"
author = "civlab-platform-team"

[simulation]
rng_seed = 42
max_ticks = 10_000
hex_grid_radius = 100
tick_budget_ms = 100

[starting_state]
civilizations = 4
starting_energy_kj = 1_000_000       # 1 GJ in KiloJoules (i64)
starting_gdp_mc = 500_000_000         # 500 kCredits in MilliCredits (i64)

[policies]
# All weights must sum to exactly 1_000_000 ppm (fixed-point 1.0)

[[policies.allocations]]
id = "military"
weight_ppm = 200_000    # 20%

[[policies.allocations]]
id = "research"
weight_ppm = 300_000    # 30%

[[policies.allocations]]
id = "infrastructure"
weight_ppm = 300_000    # 30%

[[policies.allocations]]
id = "welfare"
weight_ppm = 200_000    # 20%

[terrain]
base_seed = 42
mountain_density_ppm = 150_000    # 15%
water_density_ppm = 250_000       # 25%
forest_density_ppm = 200_000      # 20%

[mods]
enabled = false

[output]
report_every_n_ticks = 100
include_hash_chain = true
sign_output = true
```

### 5.2 Metric Definition Format

Each simulation metric is defined in `metrics/definitions/` as a TOML file:

```toml
# metrics/definitions/energy_consumption_rate.toml

[metric]
id = "FR-MET-001"
name = "energy_consumption_rate"
display_name = "Energy Consumption Rate"
description = "Rate of energy consumption across all civilizations, in KiloJoules per tick"
version = "1.0.0"
status = "active"    # active | deprecated | experimental

[formula]
expression = "sum(civ.energy_consumed_kj) / tick_count"
unit = "KiloJoules/tick"
precision = "exact"    # exact (fixed-point) | approximate (float display only)

[source]
tick_field = "world.civilizations[*].energy_consumed_kj_this_tick"
aggregation = "sum"
window = "per_tick"    # per_tick | rolling_100 | cumulative

[thresholds]
warning_low  = 1_000          # Below this: possible stall
warning_high = 10_000_000     # Above this: possible runaway consumption
critical_high = 50_000_000    # Above this: freeze-mode candidate

[prometheus]
metric_name = "civlab_sim_energy_consumption_kj_per_tick"
type = "gauge"
labels = ["scenario_id", "civilization_id", "run_id"]
help = "Energy consumed this tick by a civilization, in KiloJoules"

[traceability]
fr_id = "FR-MET-001"
prd_epic = "E2.3"
```

### 5.3 Policy Bundle Versioning

#### 5.3.1 Bundle Version Bump Rules

| Change | Bundle Version Bump | Backward Compatible |
|---|---|---|
| Adding a new optional policy allocation | MINOR | yes |
| Changing a weight without renaming IDs | MINOR | yes |
| Renaming a policy ID | MAJOR | no |
| Removing a policy ID | MAJOR | no |
| Changing weight_ppm type or unit | MAJOR | no |

#### 5.3.2 Bundle Migration Procedure

When a MAJOR policy bundle version is released:

1. Tag the old bundle in `policies/archive/v{N}/`.
2. Write a migration script in `scripts/migrate_policy_bundle_vN_to_vN+1.py`.
3. Run migration against all saved scenarios in the test fixture set and verify schema validation passes.
4. Update `schemas/scenario/v1/policy.schema.json` and bump its version field.
5. Write ADR documenting the breaking change.
6. Write migration guide at `docs/migrations/policy-bundle-vN-to-vN+1.md`.

#### 5.3.3 Backward Compatibility Rules for Saved Scenarios

- The server MUST refuse to load a scenario config whose `schema_version` major version exceeds the server's supported schema major version. Return a structured error with expected and actual versions.
- The server MUST load any scenario config whose `schema_version` major version equals the server's version and minor version is less than or equal.
- The server MUST refuse to load any scenario config with `schema_version` major version lower than `(current_major - 1)`. One previous major version is supported for one release cycle only.
- No silent at-load-time migration. If out of range, return error. Client must run the migration tool explicitly before retry.

---

## 6. Runtime Governance and Guardrails

### 6.1 Intervention Authority

Runtime interventions are tiered by authority level:

| Tier | Actor | Interventions Allowed |
|---|---|---|
| T0 Emergency | Any server process (automated) | Freeze mode trigger, emergency stop |
| T1 Operator | Server admin via `civlab-cli` | Pause/resume, tick rate throttle, entity limit adjustment |
| T2 Game Master | Authenticated GM session | Scenario parameter hot-swap within policy bundle, event injection |
| T3 Observer | Authenticated client session | Read-only: subscribe to tick events, query world state |

All interventions are written to the audit log with: actor identity, intervention type, parameters, monotonic timestamp, wall-clock timestamp, and outcome. Audit log entries are immutable.

### 6.2 Runtime Guardrails

Hard limits enforced on every tick by the `SimulationGuardrails` component:

```rust
// crates/civlab-sim/src/guardrails.rs
// @trace FR-GUARD-001

#[derive(Debug, Clone)]
pub struct SimulationGuardrails {
    /// Maximum ticks before forced termination.
    pub max_tick_count: u64,
    /// Maximum ECS entities across all archetypes.
    pub max_entity_count: u64,
    /// Maximum ECS world heap usage in bytes.
    pub max_memory_bytes: usize,
    /// Tick wall-clock budget. 3 consecutive violations trigger freeze.
    pub tick_budget_ms: u64,
    /// Maximum active WASM mod instances.
    pub max_wasm_instances: u32,
    /// Maximum total WASM memory across all instances, in bytes.
    pub max_wasm_total_memory_bytes: usize,
    /// Consecutive limit violations before freeze mode activates.
    pub freeze_threshold: u32,
}

impl SimulationGuardrails {
    pub fn production() -> Self {
        Self {
            max_tick_count: 1_000_000,
            max_entity_count: 1_000_000,
            max_memory_bytes: 512 * 1024 * 1024,        // 512 MB
            tick_budget_ms: 100,
            max_wasm_instances: 16,
            max_wasm_total_memory_bytes: 256 * 1024 * 1024,  // 256 MB
            freeze_threshold: 3,
        }
    }

    pub fn ci_test() -> Self {
        Self {
            max_tick_count: 10_000,
            max_entity_count: 100_000,
            max_memory_bytes: 256 * 1024 * 1024,
            tick_budget_ms: 500,    // Relaxed for CI debug builds
            max_wasm_instances: 4,
            max_wasm_total_memory_bytes: 64 * 1024 * 1024,
            freeze_threshold: 5,
        }
    }
}
```

### 6.3 Freeze Mode

#### 6.3.1 Triggers

| Trigger | Condition | Severity |
|---|---|---|
| Tick budget exceeded | 3 consecutive ticks over `tick_budget_ms` | P1 |
| Determinism violation | BLAKE3 hash mismatch detected in online monitoring | P0 |
| Memory ceiling exceeded | ECS heap over `max_memory_bytes` for 2 consecutive samples | P1 |
| Entity count exceeded | Entity count over `max_entity_count` | P1 |
| WASM sandbox escape | wasmtime host-call policy violation | P0 |
| Tick count limit reached | `current_tick >= max_tick_count` | P2 expected termination |
| Manual operator trigger | `civlab-cli freeze \< run-id>` | P1 |
| Critical metric threshold | Any metric with `critical_high` breach for 5 consecutive ticks | P1 |

#### 6.3.2 Freeze Mode Behavior

When freeze mode activates:

1. The tick loop halts after the current tick completes (no mid-tick halt).
2. Current world state is serialized to `snapshots/{run-id}/freeze-{tick}.civsnap`.
3. The snapshot is BLAKE3-hashed and Ed25519-signed.
4. A `FreezeEvent` notification is broadcast to all connected clients via JSON-RPC.
5. `civlab_sim_freeze_mode_active{run_id}` gauge is set to 1.
6. `civlab_sim_freeze_total{reason}` counter is incremented.
7. An audit log entry is written with: trigger reason, tick number, guardrail values, actor.
8. The server does NOT auto-restart the run. Recovery requires operator action.

```rust
// crates/civlab-server/src/freeze.rs
// @trace FR-GUARD-002

pub async fn activate_freeze_mode(
    run_id: RunId,
    trigger: FreezeTrigger,
    world_snapshot: WorldSnapshot,
    audit_log: &AuditLog,
    metrics: &SimMetrics,
    client_notifier: &ClientNotifier,
) -> Result<FreezeRecord, FreezeError> {
    let snap_bytes = world_snapshot.to_canonical_bytes();
    let snap_hash = blake3::hash(&snap_bytes);
    let snap_sig = sign_with_server_key(&snap_bytes);

    let snap_path = freeze_snapshot_path(run_id, world_snapshot.tick);
    tokio::fs::write(&snap_path, &snap_bytes).await?;

    audit_log.record(AuditEntry {
        event: AuditEvent::FreezeActivated,
        run_id,
        tick: world_snapshot.tick,
        actor: Actor::Automated(trigger.clone()),
        details: serde_json::json!({
            "trigger": trigger,
            "snapshot_hash": hex::encode(snap_hash.as_bytes()),
            "snapshot_path": snap_path.display().to_string(),
        }),
        timestamp: MonotonicTimestamp::now(),
    })?;

    metrics
        .freeze_mode_active
        .with_label_values(&[&run_id.to_string()])
        .set(1);
    metrics
        .freeze_total
        .with_label_values(&[trigger.label()])
        .inc();

    client_notifier
        .broadcast(FreezeNotification {
            run_id,
            tick: world_snapshot.tick,
            reason: trigger.user_facing_message(),
            snapshot_hash: hex::encode(snap_hash.as_bytes()),
        })
        .await;

    Ok(FreezeRecord {
        snap_path,
        snap_hash: snap_hash.into(),
        signature: snap_sig,
    })
}
```

#### 6.3.3 Freeze Recovery Procedure

```bash
# Step 1: Inspect freeze record
civlab-cli freeze-status --run-id <run-id>

# Step 2: Verify snapshot integrity
civlab-cli verify-snapshot --run-id <run-id> --tick <freeze-tick>

# Step 3: Diagnose divergence if needed
civlab-cli replay-diff \
  --bundle-a "snapshots/<run-id>/freeze-<tick>.civsnap" \
  --bundle-b "snapshots/<run-id>/pre-freeze-<tick-5>.civsnap"

# Step 4a: If safe to resume (e.g., transient tick budget spike)
civlab-cli resume \
  --run-id <run-id> \
  --justification "Transient tick budget spike; entity batch resolved"

# Step 4b: If not safe to resume (determinism violation or WASM escape)
civlab-cli abort \
  --run-id <run-id> \
  --reason "Determinism violation: BLAKE3 mismatch at tick <N>"

# Step 5: File incident (mandatory for P0 and P1 triggers)
civlab-cli incident create \
  --run-id <run-id> \
  --severity P1 \
  --trigger "tick_budget_exceeded" \
  --summary "Spike in entity creation caused 3 consecutive tick budget violations"
```

Resuming from freeze after a P0 determinism violation requires CTO sign-off in production. CI and staging environments may be resumed by the Sim Team Lead.

### 6.4 Emergency Stop

Emergency stop halts all active simulation runs immediately. Used when the server process itself must stop safely (infrastructure incident, security event).

```bash
# Emergency stop all runs
civlab-cli emergency-stop --reason "Infrastructure incident: database unreachable"

# Emergency stop a specific run
civlab-cli emergency-stop --run-id <run-id> --reason "P0: WASM sandbox escape detected"
```

Emergency stop behavior:

1. All tick loops receive cancellation signal; halt after current tick.
2. All in-progress world states serialized to emergency snapshots and signed.
3. All clients receive `EmergencyStopNotification` via JSON-RPC.
4. Server enters read-only mode; no new runs can start until operator clears the state.
5. All events recorded in the audit log.
6. `civlab_sim_emergency_stop_total` counter incremented.

To clear emergency state:

```bash
civlab-cli emergency-clear \
  --reason "Infrastructure incident resolved: database connection restored" \
  --operator-id <operator-id>
```

---

## 7. Monitoring and Observability

### 7.1 Prometheus Metrics Schema

All metrics are exported at `http://civlab-server:9090/metrics` in Prometheus text format.

#### 7.1.1 Simulation Run Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_runs_started_total` | Counter | `scenario_id`, `scenario_version` | Total runs started since server start |
| `civlab_sim_runs_completed_total` | Counter | `scenario_id`, `scenario_version`, `result` | Total runs completed; result=success,aborted,frozen |
| `civlab_sim_runs_active` | Gauge | `scenario_id` | Currently active simulation runs |
| `civlab_sim_run_duration_seconds` | Histogram | `scenario_id`, `scenario_version` | Wall-clock duration of completed runs |
| `civlab_sim_tick_total` | Counter | `run_id`, `scenario_id` | Total ticks executed |
| `civlab_sim_tick_duration_seconds` | Histogram | `run_id`, `scenario_id` | Per-tick execution time; buckets: 0.001,0.005,0.01,0.02,0.05,0.1,0.2,0.5 |
| `civlab_sim_tick_budget_exceeded_total` | Counter | `run_id`, `scenario_id` | Ticks exceeding the 100ms budget |

#### 7.1.2 Determinism Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_determinism_check_total` | Counter | `run_id`, `result` | Determinism checks; result=pass,fail |
| `civlab_sim_determinism_violations_total` | Counter | `run_id`, `scenario_id`, `d_rule` | Violations by D-rule; d_rule=D1..D7 |
| `civlab_sim_hash_chain_mismatches_total` | Counter | `run_id` | BLAKE3 hash chain mismatches in online monitoring |
| `civlab_sim_replay_consistency_failures_total` | Counter | `run_id`, `scenario_id` | Replay consistency test failures |

#### 7.1.3 Freeze Mode Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_freeze_mode_active` | Gauge | `run_id` | 1 if run is in freeze mode, 0 otherwise |
| `civlab_sim_freeze_total` | Counter | `run_id`, `reason` | Freeze activations by trigger reason |
| `civlab_sim_emergency_stop_total` | Counter | — | Emergency stops since server start |

#### 7.1.4 Resource Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_memory_bytes` | Gauge | `run_id`, `component` | Memory in bytes; component=ecs_world,wasm_total,replay_buffer,hash_chain |
| `civlab_sim_entity_count` | Gauge | `run_id`, `archetype` | ECS entity count by archetype |
| `civlab_sim_wasm_instances_active` | Gauge | `run_id` | Active WASM mod instances |
| `civlab_sim_wasm_memory_bytes` | Gauge | `run_id`, `mod_id` | WASM memory per mod instance |

#### 7.1.5 Game-Layer Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_sim_energy_consumption_kj_per_tick` | Gauge | `run_id`, `civilization_id` | Energy consumed this tick per civilization |
| `civlab_sim_gdp_mc_total` | Gauge | `run_id`, `civilization_id` | GDP of a civilization in MilliCredits |
| `civlab_sim_population_total` | Gauge | `run_id`, `civilization_id` | Population per civilization |
| `civlab_sim_territory_hexes` | Gauge | `run_id`, `civilization_id` | Hex tiles controlled |
| `civlab_sim_policy_weight_ppm` | Gauge | `run_id`, `civilization_id`, `policy_id` | Current policy allocation weight in ppm |

#### 7.1.6 Storage and I/O Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_storage_sqlite_query_duration_seconds` | Histogram | `operation`, `table` | SQLite query duration |
| `civlab_storage_postgres_query_duration_seconds` | Histogram | `operation`, `table` | PostgreSQL query duration |
| `civlab_storage_sqlite_wal_size_bytes` | Gauge | `db_path` | SQLite WAL file size |
| `civlab_storage_backup_last_success_timestamp` | Gauge | `backend` | Unix timestamp of last successful backup |
| `civlab_storage_artifact_sign_duration_seconds` | Histogram | — | Ed25519 artifact signing duration |

#### 7.1.7 Server and RPC Metrics

| Metric Name | Type | Labels | Help |
|---|---|---|---|
| `civlab_rpc_requests_total` | Counter | `method`, `status` | JSON-RPC requests; status=ok,error |
| `civlab_rpc_request_duration_seconds` | Histogram | `method` | JSON-RPC request handling duration |
| `civlab_rpc_active_connections` | Gauge | `client_type` | Active WebSocket connections; client_type=web,mobile,desktop |
| `civlab_rpc_messages_sent_total` | Counter | `notification_type` | Server-push notifications sent |

### 7.2 Grafana Dashboard Layout

The canonical dashboard is exported to `monitoring/grafana/dashboards/civlab-sim.json` and provisioned automatically. Dashboard UID: `civlab-sim-ops`.

| Row | Panel Name | Columns | Visualization | Key Query |
|---|---|---|---|---|
| 1 | Active Runs | 4 | Stat | `civlab_sim_runs_active` |
| 1 | Runs Completed (1h) | 4 | Stat | `increase(civlab_sim_runs_completed_total[1h])` |
| 1 | Freeze Mode Active | 4 | Stat (red if > 0) | `sum(civlab_sim_freeze_mode_active)` |
| 1 | Determinism Violations (24h) | 4 | Stat (red if > 0) | `increase(civlab_sim_determinism_violations_total[24h])` |
| 1 | Tick Duration p99 | 4 | Stat | `histogram_quantile(0.99, civlab_sim_tick_duration_seconds)` |
| 1 | Active WebSocket Clients | 4 | Stat | `sum(civlab_rpc_active_connections)` |
| 2 | Tick Duration Distribution | 12 | Heatmap | `civlab_sim_tick_duration_seconds_bucket` |
| 2 | Memory by Component | 12 | Time series | `civlab_sim_memory_bytes` by component |
| 3 | Runs by Result | 8 | Bar chart | `civlab_sim_runs_completed_total` by result |
| 3 | Determinism Check Pass Rate | 8 | Time series % | `rate(civlab_sim_determinism_check_total{result="pass"})` |
| 3 | RPC Request Rate | 8 | Time series | `rate(civlab_rpc_requests_total[5m])` by method |
| 4 | Energy Consumption by Civ | 12 | Time series | `civlab_sim_energy_consumption_kj_per_tick` |
| 4 | GDP by Civilization | 12 | Time series | `civlab_sim_gdp_mc_total` |
| 5 | SQLite Query Latency p99 | 12 | Time series | `histogram_quantile(0.99, civlab_storage_sqlite_query_duration_seconds)` |
| 5 | WAL Size Trend | 12 | Time series | `civlab_storage_sqlite_wal_size_bytes` |

### 7.3 Alerting Rules

```yaml
# monitoring/prometheus/alerts/civlab-sim.yml

groups:
  - name: civlab_sim_determinism
    rules:
      - alert: DeterminismViolationDetected
        expr: increase(civlab_sim_determinism_violations_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "Determinism violation in run {{ $labels.run_id }}"
          description: "D-rule {{ $labels.d_rule }} violated. Immediate investigation required."
          runbook_url: "https://docs.civlab.internal/runbooks/determinism-violation"

      - alert: HashChainMismatch
        expr: increase(civlab_sim_hash_chain_mismatches_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "BLAKE3 hash chain mismatch in run {{ $labels.run_id }}"
          description: "Online hash chain monitoring detected a mismatch. Freeze mode should have activated."
          runbook_url: "https://docs.civlab.internal/runbooks/hash-chain-mismatch"

  - name: civlab_sim_performance
    rules:
      - alert: TickBudgetExceededFrequent
        expr: rate(civlab_sim_tick_budget_exceeded_total[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "Tick budget frequently exceeded in run {{ $labels.run_id }}"
          description: "Over 10% of ticks are exceeding the 100ms budget. Freeze mode may activate."
          runbook_url: "https://docs.civlab.internal/runbooks/tick-budget-exceeded"

      - alert: TickP99HighLatency
        expr: histogram_quantile(0.99, rate(civlab_sim_tick_duration_seconds_bucket[5m])) > 0.09
        for: 5m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "Tick p99 latency above 90ms"
          description: "p99 tick duration is {{ $value | humanizeDuration }}. Budget is 100ms."
          runbook_url: "https://docs.civlab.internal/runbooks/high-tick-latency"

      - alert: SimMemoryCeilingApproaching
        expr: civlab_sim_memory_bytes{component="ecs_world"} > (450 * 1024 * 1024)
        for: 2m
        labels:
          severity: warning
          team: sim
        annotations:
          summary: "ECS world memory approaching ceiling in run {{ $labels.run_id }}"
          description: "ECS world using {{ $value | humanizeBytes }} of 512 MB limit."
          runbook_url: "https://docs.civlab.internal/runbooks/memory-ceiling"

  - name: civlab_sim_freeze
    rules:
      - alert: FreezeModeActive
        expr: sum(civlab_sim_freeze_mode_active) > 0
        for: 0m
        labels:
          severity: critical
          team: sim
        annotations:
          summary: "Simulation freeze mode is active"
          description: "{{ $value }} run(s) in freeze mode. Operator action required."
          runbook_url: "https://docs.civlab.internal/runbooks/freeze-mode"

      - alert: EmergencyStopOccurred
        expr: increase(civlab_sim_emergency_stop_total[5m]) > 0
        for: 0m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "Emergency stop triggered on civlab-server"
          description: "Server has entered emergency stop state. All runs halted."
          runbook_url: "https://docs.civlab.internal/runbooks/emergency-stop"

  - name: civlab_storage
    rules:
      - alert: BackupStaleness
        expr: time() - civlab_storage_backup_last_success_timestamp{backend="sqlite"} > 7200
        for: 10m
        labels:
          severity: warning
          team: infra
        annotations:
          summary: "SQLite backup has not succeeded in over 2 hours"
          description: "Last successful backup was {{ $value | humanizeDuration }} ago."
          runbook_url: "https://docs.civlab.internal/runbooks/backup-staleness"

      - alert: WALSizeLarge
        expr: civlab_storage_sqlite_wal_size_bytes > (512 * 1024 * 1024)
        for: 5m
        labels:
          severity: warning
          team: infra
        annotations:
          summary: "SQLite WAL file exceeds 512 MB"
          description: "WAL size is {{ $value | humanizeBytes }}. Consider checkpoint."
          runbook_url: "https://docs.civlab.internal/runbooks/wal-size"
```

### 7.4 Structured Logging

All log output uses `tracing` crate with `tracing-subscriber` JSON format. No `println!` or `eprintln!` in production code.

#### 7.4.1 Required Log Fields

| Field | Type | Description |
|---|---|---|
| `timestamp` | RFC3339 | Log time (not simulation time) |
| `level` | string | ERROR, WARN, INFO, DEBUG, or TRACE |
| `target` | string | Rust module path (e.g., `civlab_sim::tick_loop`) |
| `span.run_id` | string | Run ID if inside a simulation span |
| `span.tick` | u64 | Current tick if inside a tick span |
| `message` | string | Human-readable message |

#### 7.4.2 Log Levels and Sampling

| Level | Use | Sampling in Production |
|---|---|---|
| ERROR | Unrecoverable errors, freeze triggers, determinism violations | 100% always |
| WARN | Recoverable issues, threshold warnings | 100% |
| INFO | Run start/stop, freeze events, config changes, operator actions | 100% |
| DEBUG | Per-tick summaries every 100 ticks, RPC connections | 10% sampled |
| TRACE | Per-entity state changes, per-system timing | Off in production |

TRACE logs are never enabled in production. Enabling TRACE requires a feature flag change and a server restart, and must be approved by the Sim Team Lead.

#### 7.4.3 Log Retention

| Level | Retention | Storage Tier |
|---|---|---|
| ERROR | 2 years | Hot: 90 days; cold archive: remainder |
| WARN | 1 year | Hot: 30 days; cold archive: remainder |
| INFO | 90 days | Hot storage |
| DEBUG | 7 days | Hot storage rolling |
| TRACE | Not persisted | stdout only; never written to disk in production |

---

## 8. Artifact Integrity

### 8.1 Ed25519 Signing

All artifact outputs are signed with Ed25519 using a server-held signing key. The signing key is stored in an environment-specific secrets manager (HashiCorp Vault in production, environment variable in CI). Private keys are never written to disk.

#### 8.1.1 Signing Key Management

| Environment | Key Storage | Rotation Period | Rotation Procedure |
|---|---|---|---|
| Production | HashiCorp Vault (transit secrets engine) | 90 days | ADR-triggered rotation with 7-day overlap period |
| Staging | HashiCorp Vault | 180 days | Same as production |
| CI | GitHub Actions secret `CIVLAB_SIGNING_KEY_B64` | Per-release | Rotated with each major release |
| Development | Local file `~/.civlab/dev-signing-key.pem` (gitignored) | Developer discretion | N/A |

The public key corresponding to the production signing key is published in `public-keys/production.ed25519.pub` at the repository root. This file is included in the signed SBOM.

#### 8.1.2 What Gets Signed

| Artifact | Signed | Signature Format |
|---|---|---|
| Simulation reports (`*.civreport`) | yes | Detached `.sig` file, Ed25519 |
| Replay bundles (`*.civreplay`) | yes | Detached `.sig` file, Ed25519 |
| Freeze snapshots (`*.civsnap`) | yes | Embedded signature field |
| Audit log chunks | yes | Merkle tree root, Ed25519 |
| Release binaries | yes | cosign bundle (keyless OIDC) |
| SBOM | yes | cosign bundle (keyless OIDC) |
| Scenario config hashes | yes | Embedded in scenario metadata |

#### 8.1.3 Signing API

```rust
// crates/civlab-server/src/signing.rs
// @trace FR-INT-001

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

pub struct ArtifactSigner {
    signing_key: SigningKey,
}

impl ArtifactSigner {
    /// Sign arbitrary bytes. Returns a 64-byte Ed25519 signature.
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    /// Sign a file at `path` and write the detached signature to `path + ".sig"`.
    pub async fn sign_file(&self, path: &std::path::Path) -> Result<(), SigningError> {
        let data = tokio::fs::read(path).await?;
        let sig = self.sign(&data);
        let sig_path = path.with_extension("sig");
        tokio::fs::write(&sig_path, sig.to_bytes()).await?;
        tracing::info!(
            artifact = %path.display(),
            signature = %sig_path.display(),
            "artifact signed"
        );
        Ok(())
    }
}

pub struct ArtifactVerifier {
    verifying_key: VerifyingKey,
}

impl ArtifactVerifier {
    /// Verify a detached Ed25519 signature.
    pub fn verify(&self, data: &[u8], sig_bytes: &[u8; 64]) -> Result<(), VerificationError> {
        let sig = Signature::from_bytes(sig_bytes);
        self.verifying_key
            .verify_strict(data, &sig)
            .map_err(|_| VerificationError::InvalidSignature)
    }
}
```

#### 8.1.4 Verification CLI

```bash
# Verify a simulation report
civlab-cli verify-artifact \
  --artifact reports/run-abc123.civreport \
  --pubkey public-keys/production.ed25519.pub

# Verify a replay bundle
civlab-cli verify-artifact \
  --artifact replays/run-abc123.civreplay \
  --pubkey public-keys/production.ed25519.pub

# Batch verify all artifacts in a directory
civlab-cli verify-artifacts \
  --dir reports/ \
  --pubkey public-keys/production.ed25519.pub \
  --fail-fast
```

### 8.2 Report Package Format

A simulation report package (`*.civreport`) is a directory-format ZIP with the following structure:

```
run-abc123.civreport/
  manifest.json          # Report metadata and integrity hashes
  summary.json           # High-level run summary (tick count, outcome, final metrics)
  metrics/
    tick-metrics.csv     # Per-tick metrics (tick, energy, gdp, population per civ)
    aggregate.json       # Aggregated metric statistics (min, max, mean, p99 per metric)
  hash-chain/
    hashes.bin           # All per-tick BLAKE3 hashes (binary, 32 bytes * tick_count)
    root.hex             # Final hash chain root as hex string
  policies/
    initial-policy.toml  # Policy bundle at scenario start
    policy-changes.jsonl # Log of all policy changes during the run (one JSON per line)
  audit/
    interventions.jsonl  # All T1/T2 interventions during the run
  signature/
    manifest.sig         # Ed25519 signature over manifest.json
    report.sig           # Ed25519 signature over the entire report (all files, sorted)
```

`manifest.json` structure:

```json
{
  "format_version": "1.0.0",
  "run_id": "abc123",
  "scenario_id": "standard-100hex-v1",
  "scenario_schema_version": "1.0.0",
  "started_at": "2026-02-21T14:00:00Z",
  "completed_at": "2026-02-21T14:05:32Z",
  "tick_count": 10000,
  "outcome": "success",
  "rng_seed": 42,
  "hash_chain_root": "a3f7c2b1e9d045f8...",
  "signing_key_fingerprint": "SHA256:abc123...",
  "file_hashes": {
    "summary.json": "blake3:9d4e1f2a...",
    "metrics/tick-metrics.csv": "blake3:7c3b2a1d...",
    "metrics/aggregate.json": "blake3:5e6f7a8b...",
    "hash-chain/hashes.bin": "blake3:1a2b3c4d...",
    "hash-chain/root.hex": "blake3:2c3d4e5f...",
    "policies/initial-policy.toml": "blake3:3d4e5f6a...",
    "policies/policy-changes.jsonl": "blake3:4e5f6a7b...",
    "audit/interventions.jsonl": "blake3:5f6a7b8c..."
  }
}
```

### 8.3 Replay Bundle Signing

Replay bundles (`*.civreplay`) include:

```
run-abc123.civreplay/
  seed.bin              # 8-byte little-endian u64 seed
  tick-0-state.bin      # Canonical serialization of ECS world at tick 0
  scenario.toml         # The exact scenario config used
  hash-chain/
    root.hex            # Hash chain root after last tick
    tick-count.txt      # Number of ticks in the original run
  signature/
    bundle.sig          # Ed25519 signature over (seed.bin + tick-0-state.bin + root.hex)
    seed-proof.json     # JSON blob: {seed, scenario_id, scenario_hash, server_version}
```

The `seed-proof.json` is the tamper-detection anchor. Any replay that produces a different `hash-chain/root.hex` than the original run's root is invalid.

### 8.4 Audit Log

The audit log is an append-only JSONL file, one JSON object per line. Log chunks are signed with Ed25519 every 1000 entries. The audit log captures every event that changes simulation state or server configuration.

#### 8.4.1 Audit Event Types

| Event Type | Trigger | Mandatory Fields |
|---|---|---|
| `RunStarted` | Scenario starts | run_id, scenario_id, rng_seed, actor |
| `RunCompleted` | Scenario ends normally | run_id, tick_count, outcome, hash_chain_root |
| `RunAborted` | Operator abort | run_id, reason, tick_at_abort, actor |
| `FreezeActivated` | Freeze mode trigger | run_id, tick, trigger, snapshot_hash, actor |
| `FreezeResumed` | Operator resume after freeze | run_id, tick, justification, actor |
| `EmergencyStop` | Emergency stop | reason, active_run_ids, actor |
| `EmergencyCleared` | Emergency state cleared | reason, operator_id |
| `PolicyChanged` | T2 policy hot-swap | run_id, tick, old_policy_hash, new_policy_hash, actor |
| `EventInjected` | T2 event injection | run_id, tick, event_type, event_payload_hash, actor |
| `ConfigChanged` | Server config change | config_key, old_value_hash, new_value_hash, actor |
| `SigningKeyRotated` | Signing key rotation | new_key_fingerprint, old_key_fingerprint, actor |
| `AuditChunkSigned` | Chunk signature | chunk_start_entry, chunk_end_entry, chunk_hash, signature |

#### 8.4.2 Audit Log Entry Structure

```json
{
  "entry_id": 12345,
  "timestamp_wall": "2026-02-21T14:03:47.123456789Z",
  "timestamp_monotonic_ns": 987654321000,
  "event": "FreezeActivated",
  "run_id": "abc123",
  "tick": 847,
  "actor": {
    "type": "automated",
    "trigger": "tick_budget_exceeded",
    "consecutive_violations": 3
  },
  "details": {
    "trigger": "tick_budget_exceeded",
    "snapshot_hash": "a3f7c2b1e9d045f88c2a1b3d6e9f012c",
    "snapshot_path": "snapshots/abc123/freeze-847.civsnap"
  },
  "prev_entry_hash": "blake3:9d4e1f2a3b4c5d6e..."
}
```

The `prev_entry_hash` field chains entries together; any tampering with a prior entry invalidates all subsequent entries.

---

## 9. Storage Governance

### 9.1 SQLite Policy

SQLite is used for embedded single-node deployments and local development. All SQLite databases are opened with the following pragma configuration, applied at connection time:

```sql
-- Applied via civlab-server's SQLite connection initializer
-- @trace FR-STOR-001

PRAGMA journal_mode = WAL;          -- Write-Ahead Logging for concurrent reads
PRAGMA synchronous = NORMAL;        -- Durable without fsync on every write
PRAGMA page_size = 4096;            -- 4 KB pages (set only at DB creation time)
PRAGMA cache_size = -65536;         -- 64 MB page cache (negative = KiB)
PRAGMA foreign_keys = ON;           -- Enforce FK constraints
PRAGMA auto_vacuum = INCREMENTAL;   -- Incremental vacuuming to reclaim space
PRAGMA wal_autocheckpoint = 1000;   -- Checkpoint every 1000 WAL pages
PRAGMA mmap_size = 536870912;       -- 512 MB mmap (OS-backed memory-mapped I/O)
PRAGMA temp_store = MEMORY;         -- Temp tables in RAM
PRAGMA busy_timeout = 5000;         -- 5-second busy timeout before SQLITE_BUSY
```

Any change to pragma configuration requires an ADR and is tested against the determinism test suite (pragmas must not affect simulation output).

#### 9.1.1 Schema Migration Policy

SQLite schema migrations are managed by `sqlx` migrate with versioned SQL files in `migrations/sqlite/`:

```
migrations/sqlite/
  0001_initial_schema.sql
  0002_add_audit_log.sql
  0003_add_replay_bundles.sql
  ...
```

Rules:
- Migration files are numbered sequentially and immutable once merged to main.
- No `DROP TABLE` or `DROP COLUMN` without a corresponding data migration and a 2-week deprecation window.
- All migrations run inside a transaction. If any statement fails, the transaction rolls back and the server refuses to start.
- `sqlx migrate run` is called on server startup; the server will not start with pending migrations.
- Migrations are tested in CI against a clean database before merging.

#### 9.1.2 WAL Management

WAL checkpointing is automatic (every 1000 pages). For manual checkpointing:

```bash
# Full checkpoint and WAL reset
civlab-cli db checkpoint --backend sqlite --mode full

# Passive checkpoint (does not block readers)
civlab-cli db checkpoint --backend sqlite --mode passive
```

If the WAL exceeds 512 MB (alert threshold), a `PRAGMA wal_checkpoint(TRUNCATE)` is automatically triggered by the server and logged as an INFO event.

### 9.2 PostgreSQL Policy

PostgreSQL is used for multi-node server deployments. The `civlab-server` connects to PostgreSQL via `sqlx` with a connection pool of min 5, max 25 connections.

#### 9.2.1 Schema Migration Policy

PostgreSQL migrations use the same `sqlx` migrate tooling:

```
migrations/postgres/
  0001_initial_schema.sql
  0002_add_audit_log.sql
  ...
```

Additional PostgreSQL-specific rules:
- All tables have a `created_at TIMESTAMPTZ DEFAULT NOW()` column.
- Partitioning is used for `audit_log` and `tick_metrics` tables, partitioned by month.
- New partitions must be created at least 7 days before the month boundary. A cron job (`scripts/create-next-partition.sh`) runs daily and creates the next month's partition if it does not exist.
- Index creation must use `CREATE INDEX CONCURRENTLY` to avoid table locks.

#### 9.2.2 Partition Management

```sql
-- Partition creation template for audit_log
CREATE TABLE audit_log_2026_03
PARTITION OF audit_log
FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

-- Create index on new partition concurrently
CREATE INDEX CONCURRENTLY idx_audit_log_2026_03_run_id
ON audit_log_2026_03 (run_id);
```

Old partitions (older than the retention period in Section 9.4) are detached and dropped:

```bash
civlab-cli db drop-partition \
  --table audit_log \
  --partition audit_log_2024_01 \
  --confirm
```

### 9.3 Backup Policy

| Backend | Backup Type | Frequency | Retention | Restore Test Frequency |
|---|---|---|---|---|
| SQLite | Online backup (`.backup` API) | Every 1 hour | 30 days rolling | Weekly automated restore test |
| PostgreSQL | `pg_dump` (custom format) | Every 1 hour | 30 days rolling | Weekly automated restore test |
| PostgreSQL | WAL archiving (continuous) | Continuous | 7 days WAL | Monthly point-in-time restore test |
| Audit log | Append-only export | Every 6 hours | 2 years (cold) | Quarterly integrity check |
| Replay bundles | Cold object storage sync | Daily | Scenario TTL + 90 days | Manual on demand |

#### 9.3.1 Backup Taskfile Targets

```yaml
  backup-sqlite:
    desc: "Take an online SQLite backup to the configured backup directory"
    cmds:
      - cargo run -p civlab-cli -- db backup --backend sqlite --output {{.BACKUP_DIR}}/sqlite-{{.NOW}}.db
    vars:
      NOW:
        sh: date -u +%Y%m%dT%H%M%SZ
    requires:
      vars: [BACKUP_DIR]

  backup-postgres:
    desc: "Take a PostgreSQL backup using pg_dump"
    cmds:
      - pg_dump --format=custom --compress=9 --file={{.BACKUP_DIR}}/postgres-{{.NOW}}.pgdump {{.CIVLAB_DB_URL}}
    vars:
      NOW:
        sh: date -u +%Y%m%dT%H%M%SZ
    requires:
      vars: [BACKUP_DIR, CIVLAB_DB_URL]

  restore-test-sqlite:
    desc: "Restore latest SQLite backup to a temp DB and run validation queries"
    cmds:
      - cargo run -p civlab-cli -- db restore-test --backend sqlite --backup {{.LATEST_BACKUP}} --temp-db /tmp/civlab-restore-test.db
    requires:
      vars: [LATEST_BACKUP]

  restore-test-postgres:
    desc: "Restore latest PostgreSQL backup to a temp DB and run validation queries"
    cmds:
      - cargo run -p civlab-cli -- db restore-test --backend postgres --backup {{.LATEST_BACKUP}} --temp-db civlab_restore_test
    requires:
      vars: [LATEST_BACKUP]
```

### 9.4 Data Lifecycle and Retention

| Data Type | Active State | Archive After | Delete After | Archive Location |
|---|---|---|---|---|
| Active scenario run data | In database | Run completion | 90 days from completion | Cold object storage |
| Completed scenario reports | In database | 30 days | 2 years | Cold object storage |
| Replay bundles | On disk | 7 days after scenario end | Scenario TTL + 90 days | Cold object storage |
| Freeze snapshots | On disk | Incident resolution | 90 days after incident closed | Cold object storage |
| Audit log entries | In database | 90 days | 2 years | Cold archive (append-only export) |
| Structured logs ERROR/WARN | Hot storage | See Section 7.4.3 | Per Section 7.4.3 | Cold archive |
| Mod WASM binaries | On disk | Mod disabled | Mod removed from all scenarios | N/A (delete only) |

Deletion is performed by the `civlab-cli db gc` command, which runs the lifecycle policy and produces a deletion report before committing any deletions. Deletions are logged to the audit trail.

```bash
# Dry-run data GC (shows what would be deleted)
civlab-cli db gc --dry-run --report /tmp/gc-report.json

# Execute GC after review
civlab-cli db gc --confirm --report /tmp/gc-report.json
```

---

## 10. Dependency Governance

### 10.1 Rust Dependency Policy

#### 10.1.1 Adding a New Crate

Adding a new crate to any `Cargo.toml` in the workspace requires:

1. **ADR**: An Architecture Decision Record per Section 4.2 (trigger: "New external crate dependency").
2. **License check**: The crate's license must be on the allowed list in Section 10.3.
3. **Security audit**: `cargo audit` must pass with the new crate included.
4. **`cargo-deny` check**: `cargo deny check all` must pass.
5. **Justification**: PR description must include: crate name, version, why no existing crate suffices, license, and last release date.

The ADR is linked in the PR. The PR will not be merged without the ADR merged first.

#### 10.1.2 Pinned Crate Versions

The following crates are pinned to exact versions because minor or patch updates have historically introduced breaking behavioral changes:

| Crate | Pinned Version | Reason | Review Date |
|---|---|---|---|
| `bevy_ecs` | `=0.18.x` | ECS scheduling API surface | Per Bevy release cycle |
| `hexx` | `=0.21.x` | Hex math API; patch versions have changed coordinate conventions | Per hexx release |
| `wasmtime` | `=26.x` | Host API changes affect WASM sandbox policy | Per wasmtime major release |
| `blake3` | `>=1.5, <2` | Hash output must be stable for replay compatibility | Annual review |
| `chacha20` (via `rand_chacha`) | `>=0.3, <0.4` | RNG output stability is a determinism contract | Annual review |

Bumping any pinned crate version requires an ADR and full determinism test suite pass.

#### 10.1.3 Yanked Crate Response SLA

When `cargo audit` reports a yanked or advisory-flagged crate:

| Advisory Type | Response SLA | Action |
|---|---|---|
| RUSTSEC with CVSS >= 9.0 (Critical) | 4 hours | Immediate patching; freeze deploy pipeline until resolved |
| RUSTSEC with CVSS 7.0-8.9 (High) | 24 hours | PR within 24 hours; deploy within 48 hours |
| RUSTSEC with CVSS 4.0-6.9 (Medium) | 7 days | PR within 7 days |
| RUSTSEC with CVSS \< 4.0 (Low) | 30 days | Track and patch in next scheduled maintenance |
| Yanked crate (no advisory) | 14 days | Replace with non-yanked version |

The on-call engineer is paged for Critical advisories. High advisories create a GitHub issue assigned to the Security Guild. Medium and Low create GitHub issues labeled `security` and `dependency`.

### 10.2 JavaScript/Node Dependency Policy

The `clients/web` package uses `npm` (or `pnpm` if `pnpm-lock.yaml` is present). Rules:

- `npm audit --audit-level=high` must pass in CI on every PR.
- `package-lock.json` is committed and must be up to date.
- No `*` or `latest` version pins in `package.json`; all dependencies must be semver-pinned.
- Major version bumps of `pixi.js` or `react` require an ADR.

### 10.3 License Policy

Allowed SPDX license identifiers for Rust crates:

```
MIT
Apache-2.0
Apache-2.0 WITH LLVM-exception
BSD-2-Clause
BSD-3-Clause
ISC
MPL-2.0
Unicode-DFS-2016
CC0-1.0
Unlicense
```

Explicitly forbidden licenses:

```
GPL-2.0          (copyleft; incompatible with proprietary distribution)
GPL-3.0          (copyleft)
LGPL-2.0         (copyleft; must be reviewed case-by-case; default: forbidden)
LGPL-2.1         (same)
LGPL-3.0         (same)
AGPL-3.0         (copyleft; absolutely forbidden)
SSPL-1.0         (source-available; not open source)
Commons Clause   (source-available restriction)
```

The `deny.toml` for `cargo-deny` encodes these rules and runs in nightly CI:

```toml
# deny.toml

[licenses]
allow = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "MPL-2.0",
  "Unicode-DFS-2016",
  "CC0-1.0",
  "Unlicense",
]
deny = [
  "GPL-2.0",
  "GPL-3.0",
  "AGPL-3.0",
  "SSPL-1.0",
]
copyleft = "deny"
allow-osi-fsf-free = "neither"
confidence-threshold = 0.8

[advisories]
ignore = []   # No exemptions without ADR

[bans]
multiple-versions = "warn"    # Warn on duplicate crate versions; block if > 3 duplicates
wildcards = "deny"            # No wildcard version requirements
```

### 10.4 Security Audit Schedule

| Audit Type | Tool | Frequency | CI Integration |
|---|---|---|---|
| Rust advisory DB | `cargo audit` | Every PR + weekly nightly | Yes (blocks PR) |
| Rust license + ban | `cargo deny` | Every PR + weekly nightly | Yes (blocks PR) |
| Node.js advisory DB | `npm audit` | Every PR + weekly nightly | Yes (blocks PR) |
| SAST | `semgrep` | Every PR + nightly | Yes (blocks PR on high) |
| SBOM generation | `syft` | Every release | Yes |
| OSV database scan | `osv-scanner` | Weekly nightly | Yes (nightly) |
| Secrets detection | `gitleaks` | Pre-commit + every PR | Yes (blocks PR) |

---

## 11. Risk Controls

### 11.1 Risk Register

| Risk ID | Risk | Likelihood | Impact | Severity | Mitigation |
|---|---|---|---|---|---|
| R-01 | Determinism violation (D1-D7 breach) | Low | Critical | P0 | Double-run CI, seed-sweep, cross-platform parity, freeze mode |
| R-02 | Data loss (SQLite or PostgreSQL corruption) | Very Low | High | P1 | WAL mode, hourly backup, weekly restore test, signed artifacts |
| R-03 | Client desync (client state diverges from server) | Medium | High | P1 | Hash chain broadcast per N ticks, client-side verification |
| R-04 | WASM mod sandbox escape | Very Low | Critical | P0 | wasmtime 26.x fuel limits, memory cap, host-call allowlist, seccomp |
| R-05 | Replay bundle tampering | Very Low | High | P1 | Ed25519 signature on every bundle, verification API |
| R-06 | RNG seed leakage (seed exposed to clients) | Low | High | P1 | Seed never transmitted in RPC responses; only hash chain root |
| R-07 | Tick budget runaway (cascading slow ticks) | Low | Medium | P2 | Guardrail with 3-violation freeze threshold |
| R-08 | Dependency supply chain attack | Very Low | Critical | P0 | cargo audit, cargo deny, osv-scanner, SBOM, pinned versions |
| R-09 | Audit log tampering | Very Low | High | P1 | Per-entry hash chain, chunk Ed25519 signatures, append-only storage |
| R-10 | Schema migration failure (corrupt DB state) | Very Low | High | P1 | Transactional migrations, pre-migration backup, CI migration test |

### 11.2 Freeze Mode Runbook (Step-by-Step)

This runbook is executed whenever `FreezeModeActive` alert fires or `civlab_sim_freeze_mode_active > 0`.

```
RUNBOOK: Freeze Mode Response
Owner: On-call Engineer
Escalation: Sim Team Lead (P0/P1), Platform CTO (P0)
SLA: Acknowledge within 5 minutes; begin investigation within 15 minutes

STEP 1: ACKNOWLEDGE
  - Acknowledge the PagerDuty alert.
  - Post in #civlab-incidents Slack channel: "Investigating freeze mode on <run-id>"

STEP 2: IDENTIFY TRIGGER
  Run:
    civlab-cli freeze-status --run-id <run-id>

  Output includes: trigger type, tick number, guardrail values at freeze time.

  Trigger mapping:
    tick_budget_exceeded    -> Section 11.2.A: Tick Budget Runbook
    determinism_violation   -> Section 11.2.B: Determinism Violation Runbook (P0 ESCALATE NOW)
    memory_ceiling_exceeded -> Section 11.2.C: Memory Runbook
    entity_count_exceeded   -> Section 11.2.C: Memory Runbook
    wasm_sandbox_escape     -> Section 11.2.D: Security Incident Runbook (P0 ESCALATE NOW)
    manual_operator         -> Find the operator; ask why they froze it
    critical_metric_breach  -> Section 11.2.E: Metric Breach Runbook

STEP 3: VERIFY SNAPSHOT INTEGRITY
  Run:
    civlab-cli verify-snapshot --run-id <run-id> --tick <freeze-tick>

  If verification FAILS: this is a secondary incident (tampered snapshot).
  Escalate to Security Guild immediately.
  Do not resume the run.

STEP 4: INVESTIGATE
  See trigger-specific runbook section below.

STEP 5: DOCUMENT
  Before taking any action (resume or abort):
    civlab-cli incident create \
      --run-id <run-id> \
      --severity <P0|P1|P2> \
      --trigger <trigger-type> \
      --summary "<one-sentence description>"

STEP 6: RESOLVE
  If safe to resume:
    civlab-cli resume --run-id <run-id> --justification "<text>"

  If not safe to resume:
    civlab-cli abort --run-id <run-id> --reason "<text>"

STEP 7: POST-MORTEM
  For P0 and P1: a post-mortem is MANDATORY within 72 hours.
  Template: Section 11.4.


SECTION 11.2.A: Tick Budget Runbook
  1. Check recent tick latency metrics in Grafana.
  2. Look for entity count spikes: civlab_sim_entity_count{run_id=<run-id>}
  3. Check WASM mod memory usage: civlab_sim_wasm_memory_bytes{run_id=<run-id>}
  4. If spike is transient (< 10 ticks elevated): resume is likely safe.
  5. If sustained spike: do not resume; abort the run and investigate the scenario config.

SECTION 11.2.B: Determinism Violation Runbook (P0)
  1. ESCALATE TO SIM TEAM LEAD AND PLATFORM CTO IMMEDIATELY.
  2. Do NOT resume the run under any circumstances without CTO sign-off.
  3. Collect the freeze snapshot and the preceding snapshot (tick - 5 if available).
  4. Run: civlab-cli replay-diff \
       --bundle-a snapshots/<run-id>/freeze-<tick>.civsnap \
       --bundle-b snapshots/<run-id>/pre-freeze-<tick-5>.civsnap
  5. Identify the first divergent tick from the diff output.
  6. Check recent commits for changes to sim code that might affect D1-D7.
  7. File a GitHub issue labeled P0, determinism, with full diff output attached.
  8. Post-mortem is mandatory (Section 11.4).

SECTION 11.2.C: Memory/Entity Runbook
  1. Check entity count by archetype: civlab_sim_entity_count{run_id=<run-id>}
  2. Identify which archetype is growing unboundedly.
  3. Check the scenario config for uncapped entity-generating events.
  4. If the scenario config is the cause: abort the run; fix the config; re-run.
  5. If code is the cause: abort; file a bug; fix and release before re-running.

SECTION 11.2.D: Security Incident Runbook (P0)
  1. ESCALATE TO SECURITY GUILD AND CISO IMMEDIATELY.
  2. Emergency stop the server: civlab-cli emergency-stop --reason "WASM sandbox escape"
  3. Preserve all logs and snapshots.
  4. Do not restart the server until Security Guild clears it.
  5. Treat as a P0 security incident per the incident response process.

SECTION 11.2.E: Metric Breach Runbook
  1. Identify the metric that breached critical_high threshold.
  2. Check the metric definition in metrics/definitions/ for the expected range.
  3. Determine if the breach represents real simulation state or a metric computation bug.
  4. If real state: assess whether the scenario design is at fault; abort or resume.
  5. If metric bug: resume the run; file a bug for the metric computation.
```

### 11.3 Incident Severity Levels and Response Times

| Severity | Definition | Acknowledge | Begin Investigation | Resolve | Post-Mortem |
|---|---|---|---|---|---|
| P0 Critical | Determinism violation, WASM escape, data loss, security breach | 5 minutes | 15 minutes | Best effort / ASAP | Mandatory within 48 hours |
| P1 High | Freeze mode (non-P0 trigger), backup failure, metric breach | 15 minutes | 30 minutes | 4 hours | Mandatory within 72 hours |
| P2 Medium | Performance degradation, repeated tick budget violations | 1 hour | 2 hours | 24 hours | Optional; required if recurrence |
| P3 Low | Advisory-level issues, non-urgent dependency updates | 1 business day | 2 business days | 30 days | Not required |

On-call rotation is 24x7 for P0 and P1. P2 and P3 are handled during business hours.

### 11.4 Post-Mortem Template

Post-mortems are written as Markdown files in `docs/post-mortems/YYYY-MM-DD-<short-title>.md`:

```markdown
# Post-Mortem: <Short Title>

**Date:** YYYY-MM-DD
**Severity:** P0 | P1
**Author:** <Name>
**Reviewers:** <Names>
**Status:** Draft | Final

## Summary
One or two sentences describing what happened, impact, and duration.

## Timeline (UTC)
| Time | Event |
|---|---|
| HH:MM | Alert fired |
| HH:MM | On-call acknowledged |
| HH:MM | Root cause identified |
| HH:MM | Mitigation applied |
| HH:MM | Incident resolved |

## Root Cause
What was the fundamental cause? Be specific: which code, config, or infrastructure change caused the issue?

## Contributing Factors
What other factors made this incident possible or worse?

## Impact
- Runs affected:
- Clients affected:
- Data integrity impact:
- Duration:

## Detection
How was this detected? If detection was slow, why?

## Resolution
What steps were taken to resolve the incident?

## Action Items
| Action | Owner | Due Date | Status |
|---|---|---|---|
| Fix root cause | | | |
| Add test to prevent recurrence | | | |
| Improve detection/alerting | | | |

## Lessons Learned
What did we learn? What would we do differently?
```

---

## 12. Compliance and Auditability

### 12.1 FR Traceability

Every functional requirement must be traceable from spec to test to code. The traceability chain is:

```
FUNCTIONAL_REQUIREMENTS.md (FR-{CAT}-{NNN}: SHALL statement)
  -> Test file (// @trace FR-{CAT}-{NNN} comment)
    -> Implementation (/// @trace FR-{CAT}-{NNN} doc comment)
      -> PR (FR ID in description)
        -> Merge commit (squash: message includes FR ID)
```

The traceability check is automated in CI:

```yaml
  traceability-check:
    name: FR Traceability Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check all FR IDs have at least one test trace
        run: |
          cargo run -p civlab-cli -- check-traceability \
            --fr-file FUNCTIONAL_REQUIREMENTS.md \
            --test-dir crates/
```

The `check-traceability` command:
1. Parses all `FR-{CAT}-{NNN}` IDs from `FUNCTIONAL_REQUIREMENTS.md`.
2. Scans all test files under `crates/` for `// @trace FR-{CAT}-{NNN}` annotations.
3. Reports any FR IDs with zero test traces as errors (blocks PR merge).
4. Reports any test traces pointing to non-existent FR IDs as warnings.

### 12.2 Audit Trail Requirements

The following events MUST appear in the audit log. Missing any of these is a compliance violation:

| Event Category | Events | Required Fields |
|---|---|---|
| Simulation lifecycle | RunStarted, RunCompleted, RunAborted | run_id, scenario_id, rng_seed, tick_count, actor |
| Freeze and recovery | FreezeActivated, FreezeResumed, EmergencyStop, EmergencyCleared | run_id, tick, trigger, actor, justification |
| Runtime interventions | PolicyChanged, EventInjected | run_id, tick, actor, payload hash |
| Configuration changes | ConfigChanged, SigningKeyRotated | config_key, old_value_hash, new_value_hash, actor |
| Storage operations | DataDeleted (GC), PartitionDropped, BackupCompleted | affected_records, backup_path, operator |
| Security events | AuditChunkSigned, ArtifactVerificationFailed | chunk details, artifact path, reason |

The audit log must be queryable by `run_id`, `event_type`, `actor`, and time range:

```bash
# Query audit log for a specific run
civlab-cli audit query --run-id <run-id>

# Query all freeze events in the last 7 days
civlab-cli audit query --event FreezeActivated --since 7d

# Query all operator actions by a specific actor
civlab-cli audit query --actor <actor-id> --since 30d

# Verify audit log integrity (check hash chain and signatures)
civlab-cli audit verify --since 30d
```

### 12.3 Retention Policy for Logs and Artifacts

The following table is the single authoritative source for retention. It supersedes any conflicting statements elsewhere in this document.

| Data Category | Hot Retention | Cold Archive Retention | Total Retention | Legal Hold Override |
|---|---|---|---|---|
| Audit log entries | 90 days (database) | 2 years (cold export) | 2 years | Indefinite if legal hold flag set |
| Simulation reports | 30 days (database) | 2 years (cold storage) | 2 years | Indefinite |
| Replay bundles | 7 days (disk) | Scenario TTL + 90 days | Scenario-dependent | Indefinite |
| Freeze snapshots | Until incident resolved | 90 days after close | ~90-120 days | Indefinite |
| Structured logs ERROR | 90 days | 2 years cold | 2 years | Indefinite |
| Structured logs WARN | 30 days | 1 year cold | 1 year | Indefinite |
| Structured logs INFO | 90 days | None | 90 days | N/A |
| Signed release artifacts | 5 years (artifact store) | N/A | 5 years | Indefinite |
| SBOM files | 5 years (artifact store) | N/A | 5 years | Indefinite |

Retention automation:

```bash
# Check retention compliance (report only)
civlab-cli compliance retention-report --since 90d

# Apply retention policy (requires confirmation)
civlab-cli db gc --confirm

# Set legal hold on a run (prevents deletion)
civlab-cli compliance set-legal-hold --run-id <run-id> --reason "Active litigation"

# List all legal holds
civlab-cli compliance list-legal-holds
```

### 12.4 Governance Checkpoint Procedure

At the end of any significant implementation task, the implementing engineer runs a governance checkpoint before marking the task complete:

```bash
# Full governance checkpoint (run before closing any substantial PR)
task governance-check
```

```yaml
# Taskfile.yml: governance-check target

  governance-check:
    desc: "Run full governance checkpoint before marking a task complete"
    cmds:
      - echo "=== QG-01: Schema Validation ==="
      - task validate-all
      - echo "=== QG-02: Determinism Double-Run ==="
      - task det-double-run
      - echo "=== QG-05: Integration Matrix ==="
      - task test-integration-matrix
      - echo "=== QG-06: Replay Consistency ==="
      - cargo test -p civlab-sim --test replay_consistency -- --nocapture
      - echo "=== QG-09: Lint ==="
      - cargo fmt --all -- --check
      - cargo clippy --all-targets --all-features -- -D warnings
      - echo "=== QG-10: Dependency Audit ==="
      - cargo audit
      - echo "=== Traceability Check ==="
      - cargo run -p civlab-cli -- check-traceability --fr-file FUNCTIONAL_REQUIREMENTS.md --test-dir crates/
      - echo "=== Governance Check Complete ==="
```

The output of `task governance-check` is pasted into the PR description as evidence of compliance before requesting review.

### 12.5 Governance Review Schedule

This document and the following governance artifacts are reviewed on a scheduled basis:

| Document | Review Frequency | Reviewer | Next Review |
|---|---|---|---|
| OPS_GOVERNANCE_SPEC.md (this file) | Every 6 months | Platform CTO + Sim Team Lead | 2026-08-21 |
| FUNCTIONAL_REQUIREMENTS.md | Per release cycle | Sim Architect | Per release |
| ADR.md | Per ADR addition | Deciders named in ADR | Per ADR |
| `deny.toml` | Quarterly | Security Guild | 2026-05-21 |
| Grafana dashboards | Quarterly | Platform Team | 2026-05-21 |
| Alerting rules | Quarterly | Platform Team + On-call | 2026-05-21 |
| Backup and restore test results | Monthly | Infra Team | Monthly |
| Post-mortems action items | Monthly | Sim Team Lead | Monthly |

When a review is completed, the reviewer updates the `last_updated` frontmatter field on the document and creates a GitHub issue to schedule the next review. Reviews that surface deficiencies create follow-up issues labeled `governance`.

---

*End of Civ-Sim Ops/Governance Specification v1.0.0*

