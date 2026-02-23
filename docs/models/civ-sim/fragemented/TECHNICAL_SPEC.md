# CivLab Civ-Sim Technical Specification

**Version:** 2.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team
**Replaces:** v1 Scaffold (32-line stub)

---

## Table of Contents

1. [System Architecture Overview](#1-system-architecture-overview)
2. [Crate Manifest — Full Library Decisions](#2-crate-manifest--full-library-decisions)
3. [Crate Structure — Full Workspace Layout](#3-crate-structure--full-workspace-layout)
4. [ECS World Design](#4-ecs-world-design)
5. [Performance Architecture](#5-performance-architecture)
6. [Determinism Architecture](#6-determinism-architecture)
7. [Server Architecture](#7-server-architecture)
8. [Build and CI Configuration](#8-build-and-ci-configuration)
9. [Python Research Bindings](#9-python-research-bindings)
10. [Non-Functional Requirements Table](#10-non-functional-requirements-table)

---

## 1. System Architecture Overview

### 1.1 Executive Summary

CivLab is a **headless, deterministic civilization simulation engine** written in Rust. The core simulation runs as an independent process with no rendering dependencies. Multiple heterogeneous clients — game engines, web browsers, research scripts — attach via WebSocket and receive tick broadcasts. The system is designed for:

- **Deterministic replay:** given `(seed, scenario, event_log)`, every run produces byte-identical output.
- **Research-first:** the Python bindings allow headless batch sweeps without any server infrastructure.
- **Multi-client real-time:** up to N simultaneous game clients observe and command the same running world.

### 1.2 Headless Core + Multi-Client Topology

```
                         ┌─────────────────────────────────────────────────┐
                         │            civ-server (axum + tokio)            │
                         │                                                  │
                         │  ┌──────────────────────────────────────────┐   │
                         │  │          Simulation Thread               │   │
                         │  │  ┌──────────────────────────────────┐   │   │
                         │  │  │        civ-engine                │   │   │
                         │  │  │  (tick loop, ECS World, phases)  │   │   │
                         │  │  └──────────┬───────────────────────┘   │   │
                         │  │             │ tick delta                 │   │
                         │  │  ┌──────────▼───────────────────────┐   │   │
                         │  │  │     Domain Crates                │   │   │
                         │  │  │  economy │ climate │ actors       │   │   │
                         │  │  │  policy  │ geo     │ social       │   │   │
                         │  │  │  war     │ metrics │ replay       │   │   │
                         │  │  └──────────────────────────────────┘   │   │
                         │  └──────────────────────────────────────────┘   │
                         │         │ mpsc::Sender<BroadcastFrame>           │
                         │  ┌──────▼──────────────────────────────────┐   │
                         │  │    Broadcast Hub                         │   │
                         │  │  tokio::broadcast::channel<Arc<Frame>>  │   │
                         │  └──────────┬──────────────────────────────┘   │
                         │             │                                    │
                         │    ┌────────┼────────────────────────────┐      │
                         │    │        │                            │      │
                         │  ┌─▼──┐  ┌─▼──┐  ┌─────┐  ┌──────┐   │      │
                         │  │WS/1│  │WS/2│  │WS/3 │  │WS/N  │   │      │
                         │  └────┘  └────┘  └─────┘  └──────┘   │      │
                         │                                         │      │
                         └─────────────────────────────────────────┘      │
                                          │                                │
           ┌────────────────┬─────────────┴──────────┬────────────────────┘
           │                │                         │
   ┌───────▼──────┐ ┌───────▼──────┐        ┌────────▼──────────┐
   │ Bevy Client  │ │ Unreal/Unity │        │ Python Research   │
   │ (Rust, WS)   │ │ (C++/C#, WS) │        │ (pyo3 FFI, direct)│
   └──────────────┘ └──────────────┘        └───────────────────┘

   ┌──────────────┐ ┌──────────────┐
   │  Web Browser │ │  Godot (GD)  │
   │  (TS, WS)    │ │  (GDScript)  │
   └──────────────┘ └──────────────┘
```

### 1.3 Crate Dependency Graph (DAG)

The inter-crate dependency graph is strictly acyclic. `tach.toml` enforces all boundary rules at compile time and in CI.

```
civ-protocol   (shared types: no simulation logic)
      │
      ├──────────────────────────────────────────────┐
      │                                               │
civ-engine  ←── civ-economy ←── civ-climate          │
      │               │              │               │
      │          civ-actors ◄────────┘               │
      │               │                              │
      │          civ-policy                          │
      │               │                              │
      │          civ-geo                             │
      │               │                              │
      │          civ-social                          │
      │               │                              │
      │          civ-war                             │
      │               │                              │
      └──────► civ-metrics ◄──────────────────────────
                    │
              civ-replay
                    │
              civ-server  (top-level integration crate)
```

**Strict rules enforced via `tach.toml`:**
- `civ-engine` depends on `civ-protocol` only. No upward domain imports.
- Domain crates (`economy`, `climate`, `actors`, `policy`, `geo`, `social`, `war`) depend on `civ-engine` and `civ-protocol`.
- `civ-metrics` aggregates from all domain crates but emits into `civ-protocol` types.
- `civ-replay` depends on `civ-metrics` and `civ-protocol`.
- `civ-server` is the only crate that depends on all others.
- **No cycles permitted.** `cargo deny` and `tach check` both enforce this.

### 1.4 Responsibility Boundaries

| Crate | Owns | Does NOT Own |
|---|---|---|
| `civ-engine` | Tick loop, phase scheduler, ECS world, determinism invariants | Any domain logic |
| `civ-economy` | Market clearing, Joule allocator, ledger, price index | Citizen behavior |
| `civ-climate` | CO2 model, weather events, Monte Carlo disaster sampling | Production math |
| `civ-actors` | Citizen lifecycle, demographics, birth/death rates | Ideology, health |
| `civ-policy` | Policy FSM, effect application, three-tier evaluation | Market clearing |
| `civ-geo` | Terrain, cell grid, pathfinding, LOD aggregation | Social dynamics |
| `civ-social` | Ideology vectors, insurgency, health model, cohesion | Economy |
| `civ-war` | Military units, combat resolution, diplomacy, shadow networks | Market prices |
| `civ-metrics` | Prometheus export, time-series storage, analytics aggregation | Simulation logic |
| `civ-server` | axum router, WebSocket upgrade, broadcast hub, HTTP API | Simulation state |
| `civ-protocol` | Shared types (Event, Command, Snapshot), schemas | Logic of any kind |
| `civ-replay` | `.civreplay` format, record, seek, verify determinism | Server I/O |

---

## 2. Crate Manifest — Full Library Decisions

This section documents every third-party dependency with version pins, rationale, and the alternatives that were rejected.

### 2.1 Cargo.toml — Workspace Dependencies Block

```toml
[workspace.dependencies]

# ---------- ECS ----------
legion = { version = "0.4.0", default-features = false }

# ---------- RNG ----------
rand_chacha = "0.3.1"
rand = { version = "0.8.5", default-features = false, features = ["std_rng"] }

# ---------- Fixed-Point Arithmetic ----------
fixed = "1.23.1"

# ---------- Serialization ----------
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.114"
rmp-serde = "1.1.2"       # MessagePack for binary frames

# ---------- Async Runtime ----------
tokio = { version = "1.36.0", features = ["full"] }

# ---------- WebSocket / HTTP ----------
axum = { version = "0.7.4", features = ["ws"] }
tokio-tungstenite = "0.21.0"

# ---------- Database ----------
sqlx = { version = "0.7.4", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }

# ---------- Parallelism ----------
rayon = "1.9.0"

# ---------- Math / Geometry ----------
glam = { version = "0.27.0", features = ["bytemuck"] }

# ---------- Metrics / Observability ----------
prometheus = "0.13.3"
opentelemetry = { version = "0.22.0", features = ["metrics", "trace"] }
opentelemetry-otlp = "0.15.0"
tracing = "0.1.40"
tracing-subscriber = { version = "0.3.18", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.23.0"

# ---------- Compression ----------
zstd = "0.13.0"

# ---------- Hashing ----------
blake3 = "1.5.1"

# ---------- CLI ----------
clap = { version = "4.5.3", features = ["derive"] }

# ---------- Config ----------
config = "0.14.0"
toml = "0.8.12"

# ---------- Testing ----------
proptest = "1.4.0"
criterion = { version = "0.5.1", features = ["html_reports"] }
insta = { version = "1.36.1", features = ["json", "yaml"] }
cargo-nextest = "0.9.68"   # parallel test executor (dev tool, not lib dep)

# ---------- Python Bindings ----------
pyo3 = { version = "0.21.0", features = ["extension-module", "abi3-py310"] }

# ---------- Utilities ----------
uuid = { version = "1.7.0", features = ["v4", "serde"] }
chrono = { version = "0.4.34", features = ["serde"] }
anyhow = "1.0.80"
thiserror = "1.0.57"
bytes = "1.5.0"
bitset-core = "0.1.0"
```

### 2.2 ECS: `legion` 0.4

**Decision:** `legion` (archetype-based ECS)

**Version pin:** `0.4.0`

**Rationale:**
- **Zero-copy archetype queries:** Components are stored in contiguous typed arrays per archetype. Iterating `(Position, Inventory, Mood)` touches exactly the memory for those three component types, in order. No indirection through `Arc<Mutex<>>` or pointer chasing.
- **No `Arc<Mutex<>>` in hot path:** `legion` worlds own component data directly. Parallel system dispatch uses safe Rust borrowing rules at compile time, not runtime locks. This is required by the determinism invariant (no non-deterministic lock ordering).
- **Cache-friendly:** Archetype layout ensures entities sharing the same component set are stored together. Iterating all Citizens touches citizen-only memory; no interleaving of unrelated component data.
- **Pure Rust:** No C dependencies. Compiles cleanly on all tier-1 targets.
- **Serialization support:** Components implement `serde::Serialize + Deserialize`. `legion` worlds can be serialized to canonical form for state hashing and snapshotting.

**Alternatives Rejected:**

| Alternative | Why Rejected |
|---|---|
| `bevy_ecs` (standalone) | Pulls in a large fraction of the Bevy dependency tree even when used headless. The `bevy_ecs` standalone crate is not officially supported as a standalone library — it is maintained as part of Bevy's monorepo and breakage is common when used outside that context. Heavier compile times. |
| `specs` | Uses `Arc<Mutex<MaskedStorage<T>>>` for component storage. Every parallel system that reads components acquires a read lock. Under high parallelism (rayon scope with 8 threads), this creates lock contention on the component storage. Also uses dynamic dispatch for system scheduling, adding runtime overhead. |
| `hecs` | Solid alternative, but lacks first-class support for parallel world access patterns. Schedules are manual; no built-in concept of system phases. Would require significant bespoke scheduling code that `legion` provides out of the box. |
| Custom entity model | Full control, but the correctness burden of a cache-friendly archetype layout is substantial. `legion` has been validated at scale; a custom solution would require the same level of validation. ADR-006 deferred this decision to the P0 prototype but `legion` was selected after benchmarking. |

### 2.3 RNG: `rand_chacha` 0.3 + `rand` 0.8

**Decision:** `rand_chacha::ChaCha20Rng` for all simulation randomness.

**Version pin:** `rand_chacha = "0.3.1"`, `rand = "0.8.5"`

**Rationale:**
- **ChaCha20 algorithm:** Stream cipher with well-studied statistical properties. Passes all PractRand and TestU01 tests. Not a cryptographic commitment — we do not need CSPRNG security — but ChaCha20's regularity ensures no unexpected period collapses or correlation issues at large tick counts.
- **Seedable, portable byte order:** `ChaCha20Rng::seed_from_u64(n)` produces identical output on x86, ARM, and WASM because ChaCha20 is defined in terms of 32-bit little-endian words. This is the critical property: the same seed on any platform produces the same byte stream. `SmallRng`, `StdRng`, and `ThreadRng` do NOT guarantee this.
- **Deterministic per-tick derivation:** Tick seed is derived as `simulation_seed XOR (tick_number * PRIME_A) XOR (phase_id * PRIME_B)`. This ensures each phase within each tick gets a unique but reproducible RNG stream, preventing phase order from affecting RNG state.
- **No `ThreadRng`:** `ThreadRng` is seeded from OS entropy. Any call to `rand::random()` or `ThreadRng::new()` in simulation code is a determinism violation caught by the custom clippy lint `clippy::float_arithmetic` combined with a `#[forbid(unsafe_code)]` policy.

**Seed Derivation:**
```rust
const PRIME_A: u64 = 0x9e37_79b9_7f4a_7c15;
const PRIME_B: u64 = 0x6c62_272e_07bb_0142;

pub fn tick_phase_seed(sim_seed: u64, tick: u64, phase: PhaseId) -> u64 {
    sim_seed
        .wrapping_mul(PRIME_A)
        .wrapping_add(tick.wrapping_mul(PRIME_A))
        .wrapping_add(phase as u64 * PRIME_B)
}

pub fn make_rng(sim_seed: u64, tick: u64, phase: PhaseId) -> ChaCha20Rng {
    ChaCha20Rng::seed_from_u64(tick_phase_seed(sim_seed, tick, phase))
}
```

### 2.4 Fixed-Point Arithmetic: `fixed` 1.23 + manual `i64 × SCALE`

**Decision:** Two-tier approach. Use the `fixed` crate (`I32F32`, `I16F16` types) for physical quantities needing fractional precision. Use manual `i64 × SCALE` (SCALE = 1_000, representing milli-units) for economic quantities where conservation must be exactly verifiable.

**Version pin:** `fixed = "1.23.1"`

**No-Float Rule:**

All simulation logic — production output, market prices, resource quantities, population counts, ideological drift — must use integer or fixed-point types. The `f32`/`f64` types are forbidden in all crates except `civ-metrics` (for Prometheus gauge export) and `civ-geo` (for SIMD spatial queries via `glam`, where only rendering-adjacent position interpolation uses float).

**Rationale:**
- **Platform determinism:** IEEE 754 floating-point arithmetic is not deterministic across platforms. The x87 FPU uses 80-bit extended precision internally; the SSE2 path uses 64-bit. ARM Cortex-A uses a different rounding mode for `fma`. A value computed as `f64` on x86 will differ at the last bit on ARM. Over thousands of ticks, these differences compound.
- **Exact conservation:** Joule economy requires `sum(allocated_joules) <= total_available_joules` exactly. With floats, rounding in the allocation loop can silently violate this invariant. With `i64` scaled integers, the invariant is `sum(allocated) <= total` checked with integer comparison.
- **Audit trail:** Fixed-point amounts can be logged, replayed, and compared exactly. A float audit trail has fuzzy equality; a fixed-point trail has exact equality.

**Type Mapping:**
```rust
// Economic quantities (milli-units, SCALE = 1_000)
type Grain   = i64;   // 1 unit = 1_000 grain-millis
type Joules  = i64;   // 1 joule = 1_000 joule-millis
type Credits = i64;   // 1 credit = 1_000 milli-credits (equivalent to "cents")

// Physical / ratio quantities (fixed-point via `fixed` crate)
use fixed::types::{I32F32, I16F16};
type FertilityRatio = I16F16;  // 0.0 to 1.0 land fertility
type IdeologyScore  = I16F16;  // -1.0 to +1.0

// Geospatial (integer grid only)
type CellX = i32;
type CellY = i32;
```

**Clippy Enforcement:**
```toml
# .cargo/config.toml in each simulation crate
[target.'cfg(all())'.rustflags]
rustflags = [
    "-D", "clippy::float_arithmetic",  # forbids f32/f64 arithmetic ops in sim crates
]
```

### 2.5 Serialization: `serde` 1.0 + `serde_json` 1.0 + `rmp-serde` 1.1

**Decision:** `serde` derive macros on all domain types. JSON for debug/research/WebSocket protocol. MessagePack (via `rmp-serde`) for binary WebSocket frames.

**Version pins:** `serde = "1.0.197"`, `serde_json = "1.0.114"`, `rmp-serde = "1.1.2"`

**Rationale:**
- **`serde`:** Zero-cost abstractions via proc-macro derive. No runtime reflection. Compile-time schema checked. The `#[serde(deny_unknown_fields)]` attribute on all protocol types ensures forward-compatibility breakage is explicit.
- **`serde_json`:** Human-readable, debuggable, compatible with every client language. Used for JSON-RPC 2.0 messages on the WebSocket transport. Not used for snapshot storage in production (MessagePack is ~3x smaller).
- **`rmp-serde`:** MessagePack is a compact binary encoding of the same serde data model. A `Snapshot` that serializes to 120 KB JSON serializes to ~35 KB MessagePack. For 60 FPS clients receiving snapshots at every tick, this is a 3.4x bandwidth reduction. `rmp-serde` shares the same `serde::Serialize` derive — no separate schema definition.
- **NOT protobuf/flatbuffers:** Protocol Buffers require a separate `.proto` schema and generated code. Changes to domain types require updating both the Rust struct and the proto definition. With `serde`, the Rust type IS the schema. FlatBuffers have zero-copy reads but require manual offset handling; the snapshot types in CivLab are read-and-process, not zero-copy accessed, so FlatBuffers' main advantage does not apply.

### 2.6 Async Runtime: `tokio` 1.36

**Decision:** `tokio` 1.x with the multi-threaded scheduler for server I/O; the simulation tick loop runs on a dedicated `std::thread`, not inside tokio.

**Version pin:** `tokio = "1.36.0"`

**Rationale:**
- **Multi-thread scheduler for server:** axum, tokio-tungstenite, and sqlx all require tokio. The multi-thread scheduler dispatches I/O tasks across all CPU cores. At 100 concurrent WebSocket clients, the I/O workload is highly concurrent; the single-thread scheduler would serialize frame dispatches and increase latency.
- **Dedicated thread for simulation:** The simulation tick loop is CPU-bound and must not be preempted by async executor task switching. Running `Simulation::tick()` inside a tokio task would make it a blocking task that stalls the executor during the 8 ms deterministic transition phase. The correct pattern is `std::thread::spawn` for the sim loop, with `tokio::sync::mpsc::channel` bridging the sim thread to the async broadcast hub. This is the standard tokio architecture for CPU-bound background workers.
- **`tokio::time::sleep`:** The server uses `tokio::time::interval` for the 100 ms tick pacing clock. Actual sleep is on the sim thread via `std::thread::sleep` (or spin-loop for sub-millisecond precision).

**Thread Architecture:**
```
Main thread (tokio runtime, multi-thread scheduler)
    │
    ├── axum HTTP/WebSocket handler tasks (async, tokio)
    ├── broadcast receiver tasks per client (async, tokio)
    └── mpsc::Receiver task (receives from sim thread)

Simulation thread (std::thread, blocking)
    └── sim_loop():
        loop {
            let frame = engine.tick();
            tx.send(frame)?;       // mpsc::Sender (non-blocking)
            sleep_until_next_tick();
        }
```

### 2.7 WebSocket / HTTP: `axum` 0.7 + `tokio-tungstenite` 0.21

**Decision:** `axum` for the HTTP server and WebSocket upgrade handler. `tokio-tungstenite` for the underlying WebSocket protocol implementation.

**Version pins:** `axum = "0.7.4"`, `tokio-tungstenite = "0.21.0"`

**Rationale:**
- **`axum`:** Tower-based, composable HTTP framework. Type-safe extractors mean malformed requests fail at the type level, not at runtime. First-class WebSocket upgrade support via `axum::extract::ws`. Middleware (tracing, metrics, auth) composes cleanly as Tower layers.
- **`tokio-tungstenite`:** The reference WebSocket implementation for tokio. Used by axum's WS handler internally. Direct use gives access to `Message::Binary` for sending MessagePack frames without re-encoding through axum's higher-level API.
- **NOT warp:** warp has a complex type-level filter system that produces confusing error messages. axum's extractor model is more ergonomic and better documented.
- **NOT actix-web:** actix-web uses a different actor-model threading approach that does not compose cleanly with tokio's multi-thread scheduler. Migration complexity is high.

### 2.8 Database: `sqlx` 0.7

**Decision:** `sqlx` with the PostgreSQL driver and compile-time query verification.

**Version pin:** `sqlx = "0.7.4"`

**Rationale:**
- **Compile-time query verification:** `sqlx::query!` macros connect to the database at compile time and verify SQL syntax, column names, and return types. Type mismatches between the query and the Rust struct are compile errors, not runtime panics. This is the primary reason for choosing `sqlx` over `diesel` (which uses a DSL) or `sea-orm` (which generates queries at runtime).
- **Async-native:** `sqlx` is built on tokio. No blocking database calls on the async executor.
- **PostgreSQL driver:** Production deployments use PostgreSQL. `sqlx`'s PostgreSQL driver supports `LISTEN/NOTIFY` for real-time event feeds, JSONB columns for schema-flexible snapshot storage, and `uuid` / `chrono` type mapping.
- **Used for:** Run metadata (scenario, seed, start time, status), snapshot archive (compressed binary blobs), event log archive, metrics time-series storage.

### 2.9 Parallelism: `rayon` 1.9

**Decision:** `rayon` for data-parallel tick phases.

**Version pin:** `rayon = "1.9.0"`

**Rationale:**
- **Work-stealing thread pool:** `rayon` maintains a pool of threads equal to the number of logical CPUs. `par_iter()` distributes work across all threads with work-stealing. For the demographics phase (iterate all citizens, apply birth/death/aging), `rayon` achieves near-linear scaling up to the CPU count.
- **Safe parallel iterators:** `rayon`'s `par_iter()` enforces Rust's borrow rules. Two parallel tasks cannot mutate the same component. This eliminates data races at compile time.
- **Scoped parallelism:** Within a tick phase, `rayon::scope` allows structured concurrency. The scope ends before the next phase begins, guaranteeing phase isolation.
- **Phases that use `rayon`:** Demographics (citizen birth/death/age), Production (per-building output), Climate event sampling (per-cell Monte Carlo), Ideology drift (per-citizen vector dot product). Phases that mutate shared market state (Trade, Command Intake) run single-threaded to preserve deterministic ordering.

**What `rayon` does NOT cover:** Cross-phase parallelism. Phases execute sequentially; within a phase, entities are processed in parallel. This is the correct tradeoff: parallel processing within a phase does not change the deterministic output because the output of each entity's computation depends only on immutable input from the previous phase.

### 2.10 Math / Geometry: `glam` 0.27

**Decision:** `glam` for spatial math (cell positions, unit vectors, pathfinding heuristics).

**Version pin:** `glam = "0.27.0"`

**Rationale:**
- **SIMD-accelerated:** `glam` uses SIMD intrinsics (SSE2/NEON) for `Vec2`, `Vec3`, `Vec4`, and `Mat4` operations. Spatial queries (nearest-neighbor, range queries) that iterate thousands of cell positions benefit from SIMD dot products and comparisons.
- **`bytemuck` feature:** The `bytemuck` feature enables zero-copy casting between `glam` types and raw byte slices. Used when serializing cell position arrays to the binary frame format.
- **Float usage in `civ-geo` only:** `glam` uses `f32`. This is acceptable in `civ-geo` because spatial computations (pathfinding, rendering-adjacent LOD) are not subject to the economic determinism requirement. Cell coordinates are integer (`i32, i32`); `glam` is used only for intermediate geometric calculations (e.g., distance heuristics in A* pathfinding) where exact integer results are read back after rounding. The clippy float-arithmetic deny is scoped to simulation-core crates, not `civ-geo`.

### 2.11 Metrics / Observability: `prometheus` 0.13 + `opentelemetry` 0.22 + `tracing` 0.1

**Decision:** Three-layer observability stack.

**Version pins:**
- `prometheus = "0.13.3"`
- `opentelemetry = "0.22.0"`
- `opentelemetry-otlp = "0.15.0"`
- `tracing = "0.1.40"`
- `tracing-subscriber = "0.3.18"`
- `tracing-opentelemetry = "0.23.0"`

**Layer Responsibilities:**

| Layer | Library | Responsibility |
|---|---|---|
| Structured logging | `tracing` + `tracing-subscriber` | Span-based event logging per tick phase; JSON output for log aggregation |
| Trace propagation | `tracing-opentelemetry` | Propagates trace context from HTTP request through sim phases |
| Metrics export | `prometheus` | Per-tick Prometheus counters and histograms; scraped by Prometheus server |
| Distributed tracing | `opentelemetry-otlp` | Exports spans to Jaeger/Tempo via OTLP gRPC; used for latency profiling |

**Key Prometheus Metrics:**
```rust
// Defined in civ-metrics/src/prometheus_registry.rs

pub static TICK_DURATION_HISTOGRAM: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "civlab_tick_duration_seconds",
        "Wall-clock duration of one simulation tick",
        vec![0.001, 0.002, 0.005, 0.008, 0.010, 0.016, 0.020, 0.050, 0.100, 0.200]
    ).unwrap()
});

pub static TICK_PHASE_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "civlab_tick_phase_duration_seconds",
        "Wall-clock duration per tick phase",
        &["phase"],
        vec![0.0001, 0.0005, 0.001, 0.002, 0.005, 0.008, 0.016]
    ).unwrap()
});

pub static CITIZEN_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("civlab_citizen_count", "Total live citizen count").unwrap()
});

pub static BROADCAST_LAG_HISTOGRAM: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "civlab_broadcast_lag_seconds",
        "Lag from tick completion to last client frame delivery",
        vec![0.0001, 0.001, 0.005, 0.010, 0.020, 0.050]
    ).unwrap()
});
```

### 2.12 Compression: `zstd` 0.13

**Decision:** `zstd` for snapshot compression in binary frames and `.civreplay` file compression.

**Version pin:** `zstd = "0.13.0"`

**Rationale:**
- **Speed-to-ratio tradeoff:** At compression level 3 (default), `zstd` achieves ~3-5x compression ratio on JSON snapshot data at >500 MB/s throughput. `gzip` achieves similar ratios at ~100 MB/s. For a 120 KB JSON snapshot compressed per tick at 10 ticks/second, `zstd` at 500 MB/s adds ~0.24 ms per tick. `gzip` would add ~1.2 ms.
- **`.civreplay` format:** Each event in the event log is individually `zstd`-compressed at level 1 (fastest). The full file header and initial state are compressed at level 9 (best ratio) since compression happens once at export time.
- **Binary frame transport:** `zstd` level 1 is applied to snapshot payloads in binary WebSocket frames when the `use_binary_frames` flag is set and `compressed` flag bit is enabled.

### 2.13 Hashing: `blake3` 1.5

**Decision:** `blake3` for state hashing (determinism verification) and event state-hash contracts.

**Version pin:** `blake3 = "1.5.1"`

**Rationale:**
- **Speed:** `blake3` is the fastest general-purpose cryptographic hash function. On AVX-512 hardware it exceeds 10 GB/s. Hashing a 120 KB serialized world state at 10 ticks/second adds less than 0.12 ms per tick.
- **SIMD native:** `blake3` uses SIMD intrinsics automatically via the `blake3` crate's build script. No manual feature flags required.
- **Canonical form input:** The state hash is computed over the canonical CBOR serialization of the full ECS world. CBOR is used (not JSON) because CBOR has a defined canonical binary encoding (no whitespace variation, deterministic key ordering in maps). The `ciborium` crate provides canonical CBOR.
- **Why not SHA-256:** SHA-256 is ~3x slower than `blake3` at the same security level. For per-tick hashing, this matters. SHA-256 provides no correctness benefit over `blake3` for this use case.

### 2.14 CLI: `clap` 4.5

**Decision:** `clap` 4 with derive API for the `civ-server` binary arguments.

**Version pin:** `clap = "4.5.3"`

**Server Binary CLI:**
```
civ-server [OPTIONS]

Options:
  --port <PORT>          WebSocket port [default: 9876]
  --seed <SEED>          Simulation seed [default: random u64]
  --scenario <PATH>      Scenario TOML file path
  --tick-rate <RATE>     Ticks per second [default: 10]
  --max-clients <N>      Maximum concurrent clients [default: 100]
  --db-url <URL>         PostgreSQL connection string
  --metrics-port <PORT>  Prometheus scrape port [default: 9090]
  --log-level <LEVEL>    Log level (trace|debug|info|warn|error) [default: info]
  --headless             Disable WebSocket server (research mode)
```

### 2.15 Config: `config` 0.14 + TOML

**Decision:** `config` crate with TOML file format for simulation configuration.

**Version pins:** `config = "0.14.0"`, `toml = "0.8.12"`

**Config layering:**
1. Default values (compiled in)
2. `config/default.toml`
3. `config/{environment}.toml` (dev, staging, prod)
4. Environment variables (`CIV_PORT`, `CIV_SEED`, etc.)
5. CLI flags (highest priority)

### 2.16 Testing: `proptest` 1.4 + `criterion` 0.5 + `insta` 1.36

**Decision:** Three-library testing stack.

**Version pins:**
- `proptest = "1.4.0"`
- `criterion = "0.5.1"`
- `insta = "1.36.1"`

**Responsibilities:**

| Library | Use Case | Example |
|---|---|---|
| `proptest` | Property-based tests on invariants | "For any seed, `sum(allocated_joules) <= total_joules`" |
| `criterion` | Microbenchmarks per tick phase | `engine::tick()` with 1k / 10k / 100k citizens |
| `insta` | Snapshot tests of serialized output | Verify `Snapshot::from_state()` produces exact known JSON |

**`proptest` usage:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn joule_conservation_holds(
        available in 0i64..=1_000_000_000,
        n_actors in 1usize..=10_000,
        seed in any::<u64>(),
    ) {
        let actors: Vec<_> = (0..n_actors).map(|i| actor_with_demand(i as i64)).collect();
        let allocations = JouleAllocator::allocate(available, &actors, seed);
        let total_allocated: i64 = allocations.iter().sum();
        prop_assert!(total_allocated <= available,
            "conservation violated: {} > {}", total_allocated, available);
    }
}
```

---

## 3. Crate Structure — Full Workspace Layout

### 3.1 Directory Tree

```
civ/
├── Cargo.toml                    (workspace manifest)
├── Cargo.lock                    (committed, deterministic builds)
├── tach.toml                     (boundary enforcement)
├── .cargo/
│   └── config.toml               (target-wide rustflags, clippy lints)
├── crates/
│   ├── civ-engine/               (tick loop, ECS world, determinism)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── simulation.rs     (Simulation struct, tick() entry point)
│   │   │   ├── phase.rs          (PhaseId enum, phase scheduler)
│   │   │   ├── world.rs          (ECS World wrapper, entity management)
│   │   │   ├── snapshot.rs       (world → Snapshot serialization)
│   │   │   ├── command.rs        (Command intake, priority queue)
│   │   │   └── hash.rs           (BLAKE3 state hash computation)
│   │   └── tests/
│   │       ├── fr_core_tick_loop.rs
│   │       ├── fr_determinism_replay.rs
│   │       ├── fr_rng_seeding.rs
│   │       └── fr_phase_schedule.rs
│   │
│   ├── civ-economy/              (market, Joule allocator, ledger)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── allocator.rs      (Allocator trait + three impls)
│   │   │   ├── market.rs         (MarketAllocator: price discovery)
│   │   │   ├── plan.rs           (PlanAllocator: central planner)
│   │   │   ├── joule.rs          (JouleAllocator: energy model)
│   │   │   ├── ledger.rs         (double-entry accounting, i64)
│   │   │   └── price_index.rs    (inflation tracking, BTreeMap)
│   │   └── tests/
│   │       ├── fr_econ_market.rs
│   │       ├── fr_econ_joule.rs
│   │       └── fr_econ_properties.rs  (proptest invariants)
│   │
│   ├── civ-climate/              (CO2 model, disasters, Monte Carlo)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── co2.rs            (CO2 concentration model, i64 ppm-millis)
│   │   │   ├── weather.rs        (temperature, rainfall per cell)
│   │   │   ├── disaster.rs       (flood, drought, storm sampling)
│   │   │   └── monte_carlo.rs    (per-cell seeded event sampling)
│   │   └── tests/
│   │
│   ├── civ-actors/               (citizen lifecycle, demographics)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── citizen.rs        (Citizen ECS components, lifecycle FSM)
│   │   │   ├── demographics.rs   (birth/death rates, migration)
│   │   │   ├── employment.rs     (job assignment, unemployment)
│   │   │   └── military.rs       (MilitaryUnit, combat_strength)
│   │   └── tests/
│   │
│   ├── civ-policy/               (policy FSM, effect application)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── policy.rs         (Policy trait, evaluation pipeline)
│   │   │   ├── tiers.rs          (baseline → constrained → optimized)
│   │   │   ├── diplomacy.rs      (DiplomaticRelation, sentiment)
│   │   │   └── shadow_networks.rs(ShadowNetwork, covert actions)
│   │   └── tests/
│   │
│   ├── civ-geo/                  (terrain, pathfinding, LOD)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── grid.rs           (CellGrid, i32 coordinates)
│   │   │   ├── terrain.rs        (TerrainType, fertility, elevation)
│   │   │   ├── pathfinding.rs    (A* with glam SIMD heuristics)
│   │   │   └── lod.rs            (LOD aggregation: cell → district → region)
│   │   └── tests/
│   │
│   ├── civ-social/               (ideology, insurgency, health)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── ideology.rs       (I16F16 ideology vectors, drift model)
│   │   │   ├── insurgency.rs     (grievance accumulation, rebellion risk)
│   │   │   ├── health.rs         (disease model, mortality)
│   │   │   └── cohesion.rs       (social cohesion metrics, Gini)
│   │   └── tests/
│   │
│   ├── civ-war/                  (military, diplomacy, shadow networks)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── combat.rs         (resolve_combat, BattleResult)
│   │   │   ├── attrition.rs      (supply lines, fatigue, morale)
│   │   │   └── occupation.rs     (territory control, resistance)
│   │   └── tests/
│   │
│   ├── civ-metrics/              (prometheus, analytics export)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── registry.rs       (Prometheus metric definitions)
│   │   │   ├── timeseries.rs     (TimeSeries<T>, append, query_range)
│   │   │   ├── snapshot.rs       (MetricsSnapshot per tick)
│   │   │   ├── export.rs         (CSV, Parquet, JSONL export)
│   │   │   └── aggregates.rs     (GDP, Gini, HDI derivation)
│   │   └── tests/
│   │
│   ├── civ-server/               (axum HTTP + WebSocket server)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs           (binary entry point, clap CLI)
│   │   │   ├── router.rs         (axum Router, middleware stack)
│   │   │   ├── ws_handler.rs     (WebSocket upgrade, session lifecycle)
│   │   │   ├── broadcast.rs      (tokio::broadcast::channel hub)
│   │   │   ├── session.rs        (ClientSession, priority, filter)
│   │   │   └── sim_bridge.rs     (mpsc bridge: sim thread → async hub)
│   │   └── tests/
│   │       ├── fr_websocket_server.rs
│   │       └── fr_server_engine.rs
│   │
│   ├── civ-protocol/             (shared types: events, commands, schemas)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          (EntityId, Cell, Tick, GoodId)
│   │   │   ├── events.rs         (SimEvent enum, 50+ variants)
│   │   │   ├── commands.rs       (Command enum, action types)
│   │   │   ├── snapshot.rs       (Snapshot, SnapshotHeader)
│   │   │   └── frame.rs          (BinaryFrame, TickBroadcast)
│   │   └── tests/
│   │
│   └── civ-replay/               (.civreplay format, seek, record)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── format.rs         (ReplayFile, magic bytes, header)
│       │   ├── writer.rs         (ReplayWriter, append events)
│       │   ├── reader.rs         (ReplayReader, seek by tick)
│       │   └── verifier.rs       (verify determinism via BLAKE3)
│       └── tests/
```

### 3.2 Per-Crate Public API Surfaces

#### `civ-engine`

```rust
// Public API surface of civ-engine

pub struct Simulation {
    pub tick: u64,
    pub seed: u64,
    world: legion::World,
    resources: legion::Resources,
    schedule: PhaseSchedule,
}

impl Simulation {
    pub fn new(seed: u64, scenario: &Scenario) -> Result<Self>;
    pub fn tick(&mut self) -> Result<TickOutput>;
    pub fn snapshot(&self) -> Result<Snapshot>;
    pub fn state_hash(&self) -> [u8; 32];
    pub fn apply_command(&mut self, cmd: Command) -> Result<CommandResult>;
}

pub struct TickOutput {
    pub tick: u64,
    pub snapshot: Snapshot,
    pub events: Vec<SimEvent>,
    pub metrics: MetricsSnapshot,
    pub state_hash: [u8; 32],
    pub phase_durations: BTreeMap<PhaseId, Duration>,
}

pub trait Phase: Send + Sync {
    fn id(&self) -> PhaseId;
    fn run(&self, world: &mut legion::World, resources: &mut legion::Resources) -> Result<Vec<SimEvent>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhaseId {
    CommandIntake   = 0,
    Policy          = 1,
    Demographics    = 2,
    Production      = 3,
    Trade           = 4,
    Climate         = 5,
    War             = 6,
    Social          = 7,
    Metrics         = 8,
    Snapshot        = 9,
}
```

#### `civ-economy`

```rust
pub trait Allocator: Send + Sync {
    fn allocate(
        &self,
        available: i64,
        demands: &[AllocationDemand],
        rng: &mut ChaCha20Rng,
    ) -> AllocationResult;

    fn verify_conservation(&self, result: &AllocationResult, available: i64) -> Result<()>;
}

pub struct MarketAllocator { /* price discovery via supply/demand */ }
pub struct PlanAllocator   { /* central quota assignment */ }
pub struct JouleAllocator  { /* work-capacity proportional allocation */ }

pub struct Ledger {
    entries: BTreeMap<EntityId, i64>,  // credit balances
}

impl Ledger {
    pub fn transfer(&mut self, from: EntityId, to: EntityId, amount: i64) -> Result<()>;
    pub fn balance(&self, entity: EntityId) -> i64;
    pub fn total_supply(&self) -> i64;  // conservation check
}
```

#### `civ-server`

```rust
pub struct SimServer {
    router: axum::Router,
    broadcast_tx: tokio::sync::broadcast::Sender<Arc<BroadcastFrame>>,
    command_tx: tokio::sync::mpsc::Sender<Command>,
}

impl SimServer {
    pub async fn serve(self, addr: SocketAddr) -> Result<()>;
}

// WebSocket handler (per-client)
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse;

pub struct ClientSession {
    pub session_id: Uuid,
    pub client_type: ClientType,
    pub priority: u32,
    pub filter: SnapshotFilter,
    pub use_binary_frames: bool,
}
```

#### `civ-protocol`

```rust
// All types in this crate derive Serialize + Deserialize

pub type Tick = u64;
pub type EntityId = u64;
pub type CellX = i32;
pub type CellY = i32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub header: SnapshotHeader,
    pub cells: Vec<CellSnapshot>,
    pub agents: Vec<AgentSnapshot>,
    pub institutions: Vec<InstitutionSnapshot>,
    pub markets: Vec<MarketSnapshot>,
    pub events: Vec<SimEvent>,
    pub metrics: MetricsSnapshot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BinaryFrame {
    pub tick: u32,
    pub event_count: u32,
    pub snapshot_size: u32,
    pub flags: u32,
    pub payload: Vec<u8>,  // zstd-compressed rmp-serde encoded Snapshot
}
```

### 3.3 Dependency Direction Summary (DAG)

```
civ-protocol      (no sim deps)
    ↑
civ-engine        (depends on: civ-protocol)
    ↑
civ-economy       (depends on: civ-engine, civ-protocol)
civ-climate       (depends on: civ-engine, civ-protocol)
civ-actors        (depends on: civ-engine, civ-economy, civ-protocol)
civ-policy        (depends on: civ-engine, civ-actors, civ-protocol)
civ-geo           (depends on: civ-engine, civ-protocol)
civ-social        (depends on: civ-engine, civ-actors, civ-protocol)
civ-war           (depends on: civ-engine, civ-actors, civ-policy, civ-protocol)
    ↑
civ-metrics       (depends on: all domain crates, civ-protocol)
    ↑
civ-replay        (depends on: civ-metrics, civ-protocol)
    ↑
civ-server        (depends on: all crates above)
```

**Tach enforcement (`tach.toml` excerpt):**
```toml
[[modules]]
path = "crates/civ-engine"
depends_on = ["crates/civ-protocol"]

[[modules]]
path = "crates/civ-economy"
depends_on = ["crates/civ-engine", "crates/civ-protocol"]

[[modules]]
path = "crates/civ-server"
depends_on = ["*"]  # server is the integration top
```

---

## 4. ECS World Design

### 4.1 Archetype Layout

`legion` organizes entities by **archetype** — the set of components they carry. All entities with the same component set are stored together in contiguous memory.

**Primary Archetypes:**

| Archetype Name | Components | Entity Count (typical) | Mutation Frequency |
|---|---|---|---|
| `Citizen` | Position, Inventory, Mood, Health, Employment, Age | 1k – 100k | Every tick (mood, health) |
| `Building` | Position, BuildingRole, Inventory, ProductionState | 100 – 10k | Every tick (production) |
| `Cell` | CellPos, TerrainType, Fertility, ClimateState | 10k – 1M | Per climate tick (~10 ticks) |
| `Institution` | InstRole, Treasury, PolicyBundle, Legitimacy | 1 – 1k | Per policy tick (~10 ticks) |
| `Market` | MarketKey, PriceHistory, OrderBook | 10 – 1k | Every tick (clearing) |
| `MilitaryUnit` | Position, UnitType, Strength, Morale, Fatigue, Faction | 0 – 10k | Per war tick |

**Component Definitions:**
```rust
// civ-protocol/src/types.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub grain:  i64,  // milli-units (SCALE=1_000)
    pub labor:  i64,  // person-hour-millis
    pub energy: i64,  // joule-millis
    pub wood:   i64,
    pub metal:  i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mood {
    pub happiness:   i16,  // -1000 to +1000 (fixed-point × 10)
    pub legitimacy:  i16,  // 0 to 1000
    pub grievance:   i16,  // 0 to 1000 (rebellion risk proxy)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub vitality:  i16,  // 0 to 1000
    pub disease:   i16,  // 0 = healthy, 1000 = terminal
    pub stress:    i16,  // 0 to 1000 (chronic stress model)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Age {
    pub ticks_lived: u64,
    pub life_stage: LifeStage,  // Child, Adult, Elder
}

// Ideology stored in civ-social (separate archetype component)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ideology {
    pub auth_lib:    i16,  // I16F16 stored as i16 (×32768 scale), -1.0 to +1.0
    pub collectivism: i16,
    pub tradition:   i16,
    pub militarism:  i16,
}
```

### 4.2 System Execution Order (Phase Schedule)

The simulation executes phases in a fixed sequential order. Within each phase, `rayon` may parallelize entity iteration, but phases do not overlap.

```
Tick N
│
├─ Phase 0: CommandIntake        [50 µs budget]
│    Systems: intake_commands, validate_commands, build_control_vector
│    Parallelism: none (single-threaded, ordered by client priority)
│    Output: Control struct (immutable for remaining phases)
│
├─ Phase 1: Policy               [2 ms budget]
│    Systems: evaluate_policies, constrain_policies, apply_fiscal_controls
│    Parallelism: none (policy FSM is stateful, sequential)
│    Output: PolicyEffect list
│
├─ Phase 2: Demographics         [1 ms budget]
│    Systems: age_citizens, apply_birth_rate, apply_death_rate, migrate_citizens
│    Parallelism: rayon par_iter over all Citizen entities
│    Output: new/dead citizen entity mutations queued (applied after phase)
│
├─ Phase 3: Production           [3 ms budget]
│    Systems: tick_buildings, compute_output, update_inventories
│    Parallelism: rayon par_iter over all Building entities (independent)
│    Output: inventory mutations queued
│
├─ Phase 4: Trade                [2 ms budget]
│    Systems: clear_markets, route_goods, apply_allocations
│    Parallelism: none (market clearing is globally stateful)
│    Output: trade events, price updates
│
├─ Phase 5: Climate              [1 ms budget]
│    Systems: tick_co2, compute_weather, sample_disasters
│    Parallelism: rayon par_iter over Cell entities for weather
│    Output: climate state mutations, disaster events
│
├─ Phase 6: War                  [2 ms budget]
│    Systems: resolve_combat, apply_casualties, update_occupation
│    Parallelism: rayon par_iter over independent battle instances
│    Output: combat result events, morale mutations
│
├─ Phase 7: Social               [1.5 ms budget]
│    Systems: drift_ideology, compute_cohesion, update_insurgency, tick_health
│    Parallelism: rayon par_iter over Citizen ideology and health
│    Output: ideology drift mutations, insurgency risk update
│
├─ Phase 8: Metrics              [0.8 ms budget]
│    Systems: aggregate_gdp, compute_gini, compute_hdi, emit_prometheus
│    Parallelism: none (aggregation is a fold, inherently sequential)
│    Output: MetricsSnapshot
│
└─ Phase 9: Snapshot + Broadcast [50 µs budget]
     Systems: serialize_snapshot, compute_state_hash, enqueue_broadcast
     Parallelism: none (serialization is a single-pass walk)
     Output: BroadcastFrame sent to mpsc channel → server broadcast hub

Total budget: ~14 ms at 1,000 citizens
```

### 4.3 Parallel System Scheduling (rayon Scope Per Phase)

Phases that use `rayon` create a scoped parallel region:

```rust
// civ-engine/src/phase.rs

pub fn run_demographics_phase(
    world: &mut legion::World,
    resources: &mut legion::Resources,
    rng_seed: u64,
    tick: u64,
) -> Vec<SimEvent> {
    use rayon::prelude::*;

    // Collect entity IDs to process (deterministic order: sorted)
    let mut citizen_ids: Vec<EntityId> = {
        let mut query = <(Entity, &Age, &Health)>::query();
        query.iter(world)
            .map(|(e, _, _)| e.id().into())
            .collect()
    };
    citizen_ids.sort_unstable();  // INVARIANT: deterministic order

    // Parallel computation (read-only, returns deltas)
    let deltas: Vec<CitizenDelta> = citizen_ids
        .par_iter()
        .map(|&id| {
            let local_seed = tick_phase_seed(rng_seed, tick, PhaseId::Demographics)
                .wrapping_add(id);
            let mut local_rng = ChaCha20Rng::seed_from_u64(local_seed);
            compute_citizen_delta(world, id, &mut local_rng, tick)
        })
        .collect();

    // Sequential application of deltas (deterministic: sorted entity order)
    let mut events = Vec::new();
    for delta in deltas {
        events.extend(apply_citizen_delta(world, delta));
    }
    events
}
```

**Key pattern:** Computation is parallel (read-only lambdas per entity); mutation is sequential (apply sorted deltas). This preserves determinism while gaining parallelism.

### 4.4 Query Types and Cache Characteristics

| Query Type | Cache Behavior | Used In |
|---|---|---|
| `<(&Position, &Inventory, &Mood)>::query()` | Sequential archetype scan, high cache hit rate on Citizen archetype | Production, Social |
| `<(&Position, &BuildingRole, &mut Inventory)>::query()` | Reads Position + BuildingRole (immutable), writes Inventory | Production phase |
| `<Entity, &Age, &Health>::query().filter(component::<Employment>())` | Filter by component presence, still contiguous | Demographics |
| `<(&MarketKey, &mut OrderBook)>::query()` | Small archetype (few markets), excellent cache locality | Trade |
| `<(&InstRole, &mut Treasury, &PolicyBundle)>::query()` | Very small archetype (few institutions), effectively L1-resident | Policy |

**SoA layout note:** `legion` uses archetype-based SoA layout. All `Position` components for Citizen entities are stored in one contiguous `Vec<Position>`. All `Mood` components are in another contiguous `Vec<Mood>`. Iterating both simultaneously is a single strided pass over two cache lines per entity pair. This is the primary performance advantage over an `AoS` layout where `struct Citizen { pos, mood, health, ... }` would interleave hot and cold fields.

**Cold data isolation:** Biography text, historical event logs, and birth-location metadata are stored outside the ECS in a `BTreeMap<EntityId, CitizenBiography>` in the engine resources. These are never accessed in hot-path phases. Keeping them out of the ECS prevents them from polluting archetype cache lines.

---

## 5. Performance Architecture

### 5.1 Tick Budget Targets

| Citizen Count | p50 Tick Time | p99 Tick Time | p999 Tick Time | Notes |
|---|---|---|---|---|
| 1,000 | ≤ 8 ms | ≤ 14 ms | ≤ 16 ms | Target for 60 FPS game clients |
| 10,000 | ≤ 30 ms | ≤ 45 ms | ≤ 50 ms | Acceptable for research mode |
| 100,000 | ≤ 150 ms | ≤ 180 ms | ≤ 200 ms | Research-only; tick rate dropped to 5/sec |

**Measurement methodology:**
- `TICK_DURATION_HISTOGRAM` Prometheus metric records wall-clock time per tick using `std::time::Instant` (wall clock only, not simulation time).
- `TICK_PHASE_DURATION` records per-phase latency.
- `criterion` benchmarks run in `crates/civ-engine/benches/tick_bench.rs` for offline regression testing.
- p50/p99/p999 percentiles exported via `opentelemetry-otlp` to Jaeger for production deployments.

### 5.2 Phase Budget Breakdown (at 1,000 Citizens)

| Phase | Budget | Parallelism | Bottleneck |
|---|---|---|---|
| CommandIntake | 50 µs | None | Command queue drain |
| Policy | 2 ms | None | Policy FSM traversal |
| Demographics | 1 ms | rayon | ChaCha20 RNG per citizen |
| Production | 3 ms | rayon | Per-building output formulas |
| Trade | 2 ms | None | Market clearing algorithm |
| Climate | 1 ms | rayon | Monte Carlo per cell |
| War | 2 ms | rayon | Combat resolution per battle |
| Social | 1.5 ms | rayon | Ideology dot products |
| Metrics | 0.8 ms | None | GDP fold, Gini sort |
| Snapshot | 50 µs | None | BLAKE3 hash + rmp-serde |
| **Total** | **~14 ms** | | |

### 5.3 SIMD Targets

**`glam` for spatial queries (`civ-geo`):**
`glam::Vec2` operations (distance, dot product) compile to SSE2 `MOVAPS/DPPS` instructions on x86. The A* pathfinding heuristic function (Euclidean distance squared) is called thousands of times per path query. SIMD reduces this from ~4 scalar multiplications + additions to a single `DPPS` instruction.

**Ideology vector dot products (`civ-social`):**
Each Citizen has a 4-component ideology vector `(auth_lib, collectivism, tradition, militarism)` stored as `[i16; 4]`. Social cohesion computation requires dot products between citizen ideology vectors and institutional ideology vectors. This is an 8×i16 → i32 sum pattern suitable for SIMD.

```rust
// civ-social/src/ideology.rs

/// Compute the alignment score between a citizen and an institution.
/// Uses a manual SIMD-friendly pattern that the compiler auto-vectorizes
/// to PMADDWD on x86 and SMLAL on ARM.
#[inline]
pub fn ideology_alignment(citizen: &Ideology, institution: &Ideology) -> i32 {
    // 4-element i16 dot product → i32
    // Compiler auto-vectorizes this to SIMD on any platform with -O2
    let a = [citizen.auth_lib, citizen.collectivism, citizen.tradition, citizen.militarism];
    let b = [institution.auth_lib, institution.collectivism, institution.tradition, institution.militarism];
    a.iter().zip(b.iter()).map(|(&x, &y)| (x as i32) * (y as i32)).sum()
}
```

Across 10,000 citizens, `rayon` dispatches this in parallel; with SIMD, per-citizen cost is ~1 ns instead of ~4 ns, reducing the social phase from ~4 ms to ~1 ms.

### 5.4 Memory Layout Strategy

**Hot path (ECS archetype arrays):** Citizen components (Position, Inventory, Mood, Health, Age) stored in contiguous `Vec<T>` per component type within each archetype. A sequential scan of 1,000 Citizen Mood components touches exactly 6 KB (1000 × 6 bytes), fitting in L1 cache (typically 32 KB).

**Cold data (off-ECS storage):** The following are stored outside the ECS in engine resources, accessed only on specific events:
- `BTreeMap<EntityId, CitizenBiography>` — name, birthplace, family history
- `BTreeMap<EntityId, Vec<HistoricalEvent>>` — per-citizen event log
- `BTreeMap<InstitutionId, PolicyHistory>` — past policy decisions

**Allocation strategy:** No per-tick heap allocations on hot paths. Entity deletion uses tombstone marking (set `alive = false` in `BitSet`) and deferred compaction every 100 ticks. Event vectors are pre-allocated with capacity `= expected_events_per_tick × 1.5` and reset each tick without deallocation.

### 5.5 Profiling Methodology

**Online profiling (`tracing` + OpenTelemetry):**
```rust
// civ-engine/src/phase.rs
use tracing::{instrument, span, Level};

#[instrument(skip_all, fields(tick = tick, phase = ?phase_id))]
pub fn run_phase(phase_id: PhaseId, tick: u64, world: &mut legion::World) {
    let _span = span!(Level::DEBUG, "phase", phase = ?phase_id).entered();
    // ... phase execution
}
```
Spans export to Jaeger via `opentelemetry-otlp`. Flamegraph-equivalent data is available in the Jaeger UI for each tick.

**Offline benchmarks (`criterion`):**
```bash
cargo bench --bench tick_bench -- --save-baseline main
# After changes:
cargo bench --bench tick_bench -- --baseline main
```
Criterion reports percent change with confidence intervals. A regression > 10% in the p99 tick time triggers a CI failure.

**Flamegraph:**
```bash
cargo flamegraph --bin civ-server -- --headless --ticks 1000 --citizens 10000
```
Uses `cargo-flamegraph` (wraps `perf record` on Linux, `dtrace` on macOS). Output is `flamegraph.svg` in the project root.

---

## 6. Determinism Architecture

### 6.1 Determinism Contract

**Formal invariant:** For all `(seed, scenario, commands)`, running the simulation twice produces:
1. Identical `state_hash` at every tick.
2. Identical `events` sequence at every tick.
3. Identical final `Snapshot` after N ticks.

This invariant is verified by the CI replay test on every PR merge.

### 6.2 Full Enumeration of Determinism Rules

#### Rule D1: No `f32`/`f64` in Simulation Logic

**Category:** Type system enforcement
**Enforced by:** Clippy lint `clippy::float_arithmetic`, scoped to simulation crates via `.cargo/config.toml`

```rust
// VIOLATION — caught at compile time:
let output: f64 = inventory.grain as f64 * 0.95;  // ERROR: float arithmetic

// CORRECT:
let output: i64 = inventory.grain * 95 / 100;  // exact, integer
```

**Clippy configuration (`.cargo/config.toml` for sim crates):**
```toml
[target.'cfg(all())'.rustflags]
rustflags = [
    "-W", "clippy::float_arithmetic",
    "-D", "clippy::float_arithmetic",  # deny = build error
]
```

**Exceptions:** `civ-geo` (SIMD spatial math with `glam`), `civ-metrics` (Prometheus gauge export). Both crates are excluded from the deny via `#[allow(clippy::float_arithmetic)]` at crate level with mandatory justification comment.

#### Rule D2: All RNG through `ChaCha20Rng` — Seeded, Not Defaulted

**Category:** API contract
**Enforced by:** CI lint scanning for `rand::random`, `rand::thread_rng`, `OsRng`, `SmallRng::from_entropy`

```rust
// VIOLATION:
let choice = rand::random::<usize>() % options.len();  // non-deterministic

// CORRECT:
let mut rng = make_rng(sim_seed, tick, PhaseId::Demographics);
let choice = rng.gen_range(0..options.len());
```

**Seed derivation for every RNG call site:**
```rust
pub fn make_rng(sim_seed: u64, tick: u64, phase: PhaseId) -> ChaCha20Rng {
    let seed = sim_seed
        .wrapping_mul(0x9e37_79b9_7f4a_7c15)
        .wrapping_add(tick.wrapping_mul(0x6c62_272e_07bb_0142))
        .wrapping_add(phase as u64 * 0xbf58_476d_1ce4_e5b9);
    ChaCha20Rng::seed_from_u64(seed)
}

// For per-entity stochastic decisions, additionally mix in entity ID:
pub fn make_entity_rng(sim_seed: u64, tick: u64, phase: PhaseId, entity_id: u64) -> ChaCha20Rng {
    let seed = make_rng(sim_seed, tick, phase)
        .next_u64()
        .wrapping_add(entity_id * 0x517c_c1b7_2722_0a95);
    ChaCha20Rng::seed_from_u64(seed)
}
```

#### Rule D3: `BTreeMap` Everywhere — Never `HashMap` in Simulation State

**Category:** Data structure discipline
**Enforced by:** Custom clippy lint `sim_hashmap_forbidden` (checked in `pre-commit` hook)

```rust
// VIOLATION:
let mut goods: HashMap<GoodId, Quantity> = HashMap::new();
for (id, qty) in &goods {
    emit_event(id, qty);  // iteration order: undefined
}

// CORRECT:
let mut goods: BTreeMap<GoodId, Quantity> = BTreeMap::new();
for (id, qty) in &goods {
    emit_event(id, qty);  // iteration order: ascending by GoodId (Ord)
}
```

**Where `HashMap` is permitted:** `civ-server` (session registry, not simulation state), `civ-metrics` (metric label indexes), build scripts.

**Where `BTreeMap` is mandatory:** All types in `civ-protocol`, all simulation state in domain crates, all command queues, all event buffers.

#### Rule D4: No `SystemTime` or `Instant` in Simulation State

**Category:** Clock discipline
**Enforced by:** Grep-based CI check for `SystemTime::now()` and `Instant::now()` in simulation crates

```rust
// VIOLATION:
let now = SystemTime::now();
let elapsed = now.elapsed().unwrap().as_secs();

// CORRECT:
let current_tick = simulation.tick;
let elapsed_ticks = current_tick - started_tick;
let sim_seconds = elapsed_ticks * 100;  // 100ms per tick
```

`Instant::now()` is allowed only in `civ-server` (for WebSocket ping/pong timing) and `civ-metrics` (for Prometheus timestamp export). Both usages are excluded from simulation state.

#### Rule D5: Sorted Entity Iteration Order

**Category:** Iteration discipline
**Enforced by:** Code review + the canonical `run_demographics_phase` pattern

All phases that iterate entities and produce events or mutations must sort entity IDs before iteration:

```rust
// The canonical parallel phase pattern (D5 + D2 combined)

let mut entity_ids: Vec<EntityId> = query.iter(world)
    .map(|(e, ..)| EntityId::from(e))
    .collect();
entity_ids.sort_unstable();  // REQUIRED: deterministic order

let deltas: Vec<Delta> = entity_ids
    .par_iter()
    .map(|&id| {
        let rng = make_entity_rng(seed, tick, phase, id);
        compute_delta(world, id, rng)
    })
    .collect();  // rayon preserves par_iter order in collect()

// Apply in sorted order
for delta in &deltas {
    apply_delta(world, *delta);
}
```

**Why `par_iter().collect()` is deterministic:** `rayon::par_iter()` followed by `.collect()` preserves input order in the output `Vec`. Tasks execute in parallel but the result vector is assembled in original order. This is a documented rayon guarantee.

#### Rule D6: Immutable Phase Input — Functional Data Flow

**Category:** Architecture discipline
**Enforced by:** Type system (`&State` vs `&mut State` parameters)

Each phase receives immutable access to prior-phase state and produces a list of mutations. Mutations are accumulated in a `Vec<Delta>` and applied sequentially after the phase completes:

```rust
// Phase signature contract
pub trait Phase {
    fn run(
        &self,
        world: &legion::World,           // IMMUTABLE read of current state
        resources: &legion::Resources,
        rng_seed: u64,
        tick: u64,
    ) -> Vec<WorldMutation>;             // OWNED output, no aliasing
}

// Engine applies mutations sequentially after each phase
fn apply_mutations(world: &mut legion::World, mutations: Vec<WorldMutation>) {
    let mut sorted = mutations;
    sorted.sort_by_key(|m| m.entity_id);  // D5: deterministic application order
    for mutation in sorted {
        mutation.apply(world);
    }
}
```

#### Rule D7: State Hash Verification

**Category:** Correctness verification
**Enforced by:** CI determinism replay test

Every tick produces a `state_hash: [u8; 32]` computed by BLAKE3 over the canonical CBOR serialization of the full ECS world state. This hash is:
- Included in every `TickOutput`
- Included in every emitted `SimEvent` (as `state_hash` field)
- Verified during `.civreplay` replay against the original run's hashes

```rust
// civ-engine/src/hash.rs

pub fn compute_state_hash(world: &legion::World, tick: u64) -> [u8; 32] {
    use ciborium::into_writer;
    use blake3::Hasher;

    // Serialize world to canonical CBOR
    let mut cbor_bytes: Vec<u8> = Vec::with_capacity(256 * 1024);
    let canonical_world = CanonicalWorldView::from(world);  // sorted, deterministic
    into_writer(&canonical_world, &mut cbor_bytes).expect("cbor serialization must not fail");

    // Include tick in hash to prevent cross-tick hash collisions
    let mut hasher = Hasher::new();
    hasher.update(&tick.to_le_bytes());
    hasher.update(&cbor_bytes);
    hasher.finalize().into()
}
```

### 6.3 CI Enforcement: Determinism Replay Test

```rust
// civ-engine/tests/fr_determinism_replay.rs

/// FR-CIV-CORE-002: Deterministic Transition
/// FR-CIV-CORE-011: Replay Determinism Verification
#[test]
fn determinism_replay_100_ticks() {
    let seed = 0xDEAD_BEEF_0BAD_CAFEu64;
    let scenario = Scenario::test_default();

    // Run 1
    let mut sim_a = Simulation::new(seed, &scenario).unwrap();
    let mut hashes_a: Vec<[u8; 32]> = Vec::new();
    for _ in 0..100 {
        let output = sim_a.tick().unwrap();
        hashes_a.push(output.state_hash);
    }

    // Run 2 (same inputs, fresh simulation)
    let mut sim_b = Simulation::new(seed, &scenario).unwrap();
    let mut hashes_b: Vec<[u8; 32]> = Vec::new();
    for _ in 0..100 {
        let output = sim_b.tick().unwrap();
        hashes_b.push(output.state_hash);
    }

    // Every tick must match
    for (i, (a, b)) in hashes_a.iter().zip(hashes_b.iter()).enumerate() {
        assert_eq!(a, b,
            "Determinism violation at tick {}: hash_a={} hash_b={}",
            i,
            hex::encode(a),
            hex::encode(b)
        );
    }
}

/// Test with multiple seeds
#[test]
fn determinism_multiple_seeds() {
    for seed in [1u64, 2, 3, 5, 7, 11, 13, 0xFFFF_FFFF_FFFF_FFFF] {
        let scenario = Scenario::test_default();
        let mut sim_a = Simulation::new(seed, &scenario).unwrap();
        let mut sim_b = Simulation::new(seed, &scenario).unwrap();
        for _ in 0..50 {
            let out_a = sim_a.tick().unwrap();
            let out_b = sim_b.tick().unwrap();
            assert_eq!(out_a.state_hash, out_b.state_hash,
                "seed={:#018x}", seed);
        }
    }
}
```

---

## 7. Server Architecture

### 7.1 axum Router Layout

```rust
// civ-server/src/router.rs

pub fn build_router(state: ServerState) -> axum::Router {
    axum::Router::new()
        // WebSocket endpoint (game clients, research clients)
        .route("/sim", get(ws_handler))

        // REST endpoints (admin, health, metrics)
        .route("/health",   get(health_handler))
        .route("/metrics",  get(metrics_handler))   // Prometheus scrape
        .route("/admin/snapshot", get(admin_snapshot_handler))
        .route("/admin/seed",     get(admin_seed_handler))

        // Tower middleware stack
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http()           // tracing spans per request
                    .make_span_with(DefaultMakeSpan::new()
                        .level(Level::DEBUG)))
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(ConcurrencyLimitLayer::new(200))     // max concurrent HTTP connections
        )
        .with_state(state)
}

#[derive(Clone)]
pub struct ServerState {
    pub broadcast_rx: BroadcastReceiver<Arc<BroadcastFrame>>,
    pub command_tx:   mpsc::Sender<Command>,
    pub session_registry: Arc<Mutex<SessionRegistry>>,  // OK: not hot path
    pub db_pool:      sqlx::PgPool,
}
```

### 7.2 WebSocket Upgrade Handler

```rust
// civ-server/src/ws_handler.rs

/// Handles WebSocket upgrade and manages the per-client session lifecycle.
/// This function is called once per client connection. It runs as a tokio task.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
    Query(params): Query<HandshakeParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_client(socket, state, params))
}

async fn handle_client(
    socket: WebSocket,
    state: ServerState,
    params: HandshakeParams,
) {
    let session = ClientSession::new(params);
    let session_id = session.id;

    state.session_registry
        .lock().await
        .insert(session_id, session.clone());

    // Subscribe to the broadcast channel for this client
    let mut rx = state.broadcast_rx.resubscribe();

    let (mut ws_tx, mut ws_rx) = socket.split();

    // Send initial handshake response
    let initial_snapshot = fetch_current_snapshot(&state).await;
    let handshake_response = serde_json::to_string(&HandshakeResponse {
        session_id,
        tick: initial_snapshot.header.tick,
        snapshot: initial_snapshot,
    }).unwrap();
    ws_tx.send(Message::Text(handshake_response)).await.ok();

    // Concurrent loops: receive commands + send broadcasts
    loop {
        tokio::select! {
            // Incoming command from client
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<Command>(&text) {
                            state.command_tx.send(cmd).await.ok();
                        }
                    }
                    None | Some(Err(_)) => break,  // client disconnected
                    _ => {}
                }
            }

            // Outgoing broadcast frame from simulation
            frame = rx.recv() => {
                match frame {
                    Ok(frame) => {
                        let filtered = frame.filter(&session.filter);
                        let msg = if session.use_binary_frames {
                            Message::Binary(filtered.to_msgpack_bytes())
                        } else {
                            Message::Text(filtered.to_json_rpc_string())
                        };
                        if ws_tx.send(msg).await.is_err() {
                            break;  // client disconnected
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("Client {} lagged {} frames", session_id, n);
                        // Client is too slow; drop frames, do not block simulation
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    state.session_registry.lock().await.remove(&session_id);
}
```

### 7.3 Multi-Client Broadcast via `tokio::broadcast::channel`

```rust
// civ-server/src/broadcast.rs

/// The broadcast channel has capacity = 256 frames (~25 seconds at 10 ticks/sec).
/// Slow clients lag but never block the simulation thread.
pub const BROADCAST_CAPACITY: usize = 256;

pub fn create_broadcast_channel() -> (
    tokio::sync::broadcast::Sender<Arc<BroadcastFrame>>,
    tokio::sync::broadcast::Receiver<Arc<BroadcastFrame>>,
) {
    tokio::sync::broadcast::channel(BROADCAST_CAPACITY)
}

/// The sim bridge task runs on the async executor and forwards frames
/// from the sim thread's mpsc channel to the broadcast channel.
pub async fn run_sim_bridge(
    mut sim_rx: mpsc::Receiver<TickOutput>,
    broadcast_tx: broadcast::Sender<Arc<BroadcastFrame>>,
) {
    while let Some(output) = sim_rx.recv().await {
        let frame = Arc::new(BroadcastFrame::from(output));
        // broadcast::send returns Err only if there are no receivers
        // (all clients disconnected). This is not an error.
        let _ = broadcast_tx.send(frame);
        BROADCAST_CLIENTS_GAUGE.set(broadcast_tx.receiver_count() as i64);
    }
}
```

**Design note:** `Arc<BroadcastFrame>` is used because `broadcast::channel` clones the value for each receiver. Cloning an `Arc` is `O(1)` (atomic increment); cloning a `BroadcastFrame` (potentially 120 KB) for each of 100 clients would be `O(n × frame_size)`. With `Arc`, all clients share the same heap allocation.

### 7.4 Command Priority Queue

Commands from clients are buffered in a `BinaryHeap` ordered by `(client_priority, tick_received)`. The `CommandIntake` phase drains this queue at the start of each tick.

```rust
// civ-server/src/sim_bridge.rs

#[derive(PartialEq, Eq)]
struct PrioritizedCommand {
    priority:      u32,       // lower = higher priority
    tick_received: u64,
    command:       Command,
}

// BinaryHeap is a max-heap; we want min-priority first,
// so we invert the Ord impl.
impl Ord for PrioritizedCommand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.priority.cmp(&self.priority)
            .then_with(|| other.tick_received.cmp(&self.tick_received))
    }
}

impl PartialOrd for PrioritizedCommand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct CommandQueue {
    heap: BinaryHeap<PrioritizedCommand>,
}

impl CommandQueue {
    /// Called by CommandIntake phase at start of each tick.
    /// Returns commands sorted by (priority asc, tick_received asc).
    pub fn drain_for_tick(&mut self) -> Vec<Command> {
        let mut cmds = Vec::new();
        while let Some(pc) = self.heap.pop() {
            cmds.push(pc.command);
        }
        cmds
    }
}
```

**Priority tier mapping:**
```rust
pub enum ClientType {
    Admin       = 0,  // override any command
    Game        = 1,  // player game clients
    AI          = 2,  // NPC / research agents
    ReadOnly    = 3,  // loggers, monitoring
}
```

### 7.5 Simulation Loop vs. Server Event Loop Separation

```
╔══════════════════════════════════════════════════════════════════════╗
║  SIMULATION THREAD (std::thread)                                     ║
║                                                                      ║
║  loop {                                                              ║
║      // 1. Drain command queue (mpsc::Receiver, try_recv)            ║
║      let commands = cmd_rx.try_recv_all();                           ║
║                                                                      ║
║      // 2. Execute tick (CPU-bound, ~14ms)                           ║
║      let output = engine.tick(commands)?;                            ║
║                                                                      ║
║      // 3. Send to async bridge (non-blocking)                       ║
║      sim_tx.try_send(output)?;                                       ║
║                                                                      ║
║      // 4. Sleep to next tick boundary (100ms)                       ║
║      sleep_until(next_tick_at);                                      ║
║  }                                                                   ║
╚════════════════════════════╤═════════════════════════════════════════╝
                             │ mpsc::channel  (sim → async)
╔════════════════════════════▼═════════════════════════════════════════╗
║  TOKIO ASYNC EXECUTOR (multi-thread scheduler)                       ║
║                                                                      ║
║  run_sim_bridge task:                                                ║
║      sim_rx.recv() → broadcast_tx.send(Arc<BroadcastFrame>)         ║
║                                                                      ║
║  Per-client ws_handler tasks:                                        ║
║      rx.recv() → ws_tx.send(filtered_frame)                         ║
║      ws_rx.next() → cmd_queue.push(command)                         ║
║                                                                      ║
║  HTTP handler tasks:                                                 ║
║      /health, /metrics, /admin/snapshot                              ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Key invariant:** The simulation thread is never `await`ed. It never uses `tokio::spawn` or any async primitive. The only bridge is `std::sync::mpsc::channel` for commands (client → sim) and `tokio::sync::mpsc::channel` for tick outputs (sim → async). This design ensures that:

1. The simulation tick is not subject to tokio task scheduling jitter.
2. Slow clients (backpressure on `broadcast::Receiver`) do not slow down the simulation.
3. The simulation can advance at any configured tick rate independently of network I/O.

---

## 8. Build and CI Configuration

### 8.1 Workspace `Cargo.toml` Profiles

```toml
# Cargo.toml (workspace root)

[profile.release]
opt-level = 3
lto = "thin"               # thin LTO: ~20% binary size reduction, faster than "fat"
codegen-units = 1          # single codegen unit for maximum inlining across crates
strip = "debuginfo"        # strip debug symbols from release binary
panic = "abort"            # no unwinding in release; smaller binary, faster

[profile.dev]
opt-level = 1              # opt-level=1: faster test compilation than 0, slow enough to debug
debug = true
overflow-checks = true     # catch integer overflows in dev/test
incremental = true

[profile.bench]
inherits = "release"
debug = true               # keep debug info for flamegraph / perf

[profile.test]
opt-level = 1
debug = true
overflow-checks = true
```

### 8.2 Feature Flags

```toml
# civ-engine/Cargo.toml

[features]
default = []

# Enables the Python research API (pyo3 bindings)
research-api = ["pyo3"]

# Enables compile-time SQL verification (requires DB at compile time)
sqlx-offline = ["sqlx/offline"]

# Enables additional tracing instrumentation (performance overhead)
trace-verbose = []
```

**Usage:**
```bash
# Build server binary (no Python bindings)
cargo build --release --bin civ-server

# Build Python extension module
cargo build --release --features research-api --lib

# Build with offline sqlx (no DB connection at compile time)
SQLX_OFFLINE=true cargo build --release
```

### 8.3 CI Pipeline

```yaml
# .github/workflows/ci.yml (representative structure)

name: CI

on: [push, pull_request]

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --all -- --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          cargo clippy --all-targets --all-features -- \
            -D warnings \
            -D clippy::float_arithmetic \
            -D clippy::unwrap_used \
            -D clippy::expect_used \
            -D clippy::panic \
            -W clippy::pedantic

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo nextest run --all --no-fail-fast
      - run: cargo test --doc --all   # doctest pass

  determinism:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo nextest run -p civ-engine --test fr_determinism_replay

  bench-regression:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - run: |
          cargo bench --bench tick_bench -- --save-baseline pr
          git checkout ${{ github.base_ref }}
          cargo bench --bench tick_bench -- --baseline pr --load-baseline main
          # criterion returns exit code 1 if regression > threshold

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo deny check licenses bans advisories

  boundary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pip install tach && tach check
```

**CI gates that must pass before merge:**
1. `cargo fmt --check` — formatting
2. `cargo clippy -D warnings -D clippy::float_arithmetic` — lints + no-float enforcement
3. `cargo nextest run --all` — all tests including determinism replay
4. `cargo deny check` — license compliance + CVE scanning
5. `tach check` — crate boundary enforcement
6. Bench regression check on PRs (p99 tick time must not increase > 10%)

### 8.4 Cargo Deny Configuration

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
    "Unicode-DFS-2016",
    "Zlib",
]
deny = ["GPL-2.0", "GPL-3.0", "LGPL-2.0", "LGPL-3.0", "AGPL-3.0"]
copyleft = "deny"

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
unsound = "deny"
```

---

## 9. Python Research Bindings

### 9.1 Design Goals

The Python research API exposes the simulation engine as a Python extension module (`.so` / `.pyd`) without requiring the WebSocket server infrastructure. A researcher can run thousands of simulations with parameter sweeps using `multiprocessing` or `concurrent.futures`.

**Target usage:**
```python
from civlab import CivLab, Scenario, SimulationResult
import multiprocessing

def run_scenario(params):
    seed, tax_rate, subsidy_level = params
    lab = CivLab()
    result = lab.run(
        seed=seed,
        scenario=Scenario.from_toml("scenarios/base.toml").with_params(
            tax_rate=tax_rate,
            subsidy_level=subsidy_level,
        ),
        n_ticks=1000,
    )
    return result.metrics_array()

# Parameter sweep: 100 seeds × 10 tax rates × 5 subsidy levels = 5,000 runs
params = [
    (seed, tax, sub)
    for seed in range(100)
    for tax in [0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55]
    for sub in [0.0, 0.1, 0.2, 0.3, 0.4]
]

with multiprocessing.Pool(processes=multiprocessing.cpu_count()) as pool:
    results = pool.map(run_scenario, params)
```

### 9.2 pyo3 FFI Layer

```rust
// civ-engine/src/python.rs  (compiled only with feature = "research-api")

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use numpy::{PyArray1, PyArray2};

/// Python class: CivLab
/// Entry point for headless simulation runs.
#[pyclass(name = "CivLab")]
pub struct PyCivLab;

#[pymethods]
impl PyCivLab {
    #[new]
    pub fn new() -> Self {
        PyCivLab
    }

    /// Run a simulation for n_ticks and return a SimulationResult.
    pub fn run(
        &self,
        py: Python<'_>,
        seed: u64,
        scenario: &PyScenario,
        n_ticks: u64,
    ) -> PyResult<PySimulationResult> {
        // Release GIL during simulation (CPU-bound, no Python objects touched)
        py.allow_threads(|| {
            let mut sim = Simulation::new(seed, &scenario.inner)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

            let mut snapshots = Vec::with_capacity(n_ticks as usize);
            let mut events = Vec::new();

            for _ in 0..n_ticks {
                let output = sim.tick()
                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
                snapshots.push(output.metrics);
                events.extend(output.events);
            }

            Ok(PySimulationResult {
                seed,
                n_ticks,
                snapshots,
                final_hash: sim.state_hash(),
            })
        })
    }
}

/// Python class: SimulationResult
#[pyclass(name = "SimulationResult")]
pub struct PySimulationResult {
    seed: u64,
    n_ticks: u64,
    snapshots: Vec<MetricsSnapshot>,
    final_hash: [u8; 32],
}

#[pymethods]
impl PySimulationResult {
    /// Return metrics as a numpy-compatible 2D array.
    /// Shape: (n_ticks, n_metrics)
    /// Columns: [tick, population, gdp, avg_happiness, gini, legitimacy, insurgency_risk, hdi]
    pub fn metrics_array<'py>(&self, py: Python<'py>) -> &'py PyArray2<f64> {
        let n_cols = 8usize;
        let mut data = vec![0.0f64; self.snapshots.len() * n_cols];
        for (i, snap) in self.snapshots.iter().enumerate() {
            let row = i * n_cols;
            data[row + 0] = snap.tick as f64;
            data[row + 1] = snap.population as f64;
            data[row + 2] = snap.gdp as f64 / 1_000.0;  // convert milli-credits
            data[row + 3] = snap.avg_happiness as f64 / 10.0;
            data[row + 4] = snap.gini as f64 / 1_000.0;
            data[row + 5] = snap.legitimacy as f64 / 10.0;
            data[row + 6] = snap.insurgency_risk as f64 / 10.0;
            data[row + 7] = snap.hdi as f64 / 1_000.0;
        }
        PyArray2::from_vec2(py, &data.chunks(n_cols)
            .map(|row| row.to_vec())
            .collect::<Vec<_>>())
            .unwrap()
    }

    /// Return the final state hash as a hex string (for reproducibility verification)
    #[getter]
    pub fn final_hash_hex(&self) -> String {
        hex::encode(self.final_hash)
    }
}

/// Python module registration
#[pymodule]
fn civlab(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCivLab>()?;
    m.add_class::<PyScenario>()?;
    m.add_class::<PySimulationResult>()?;
    Ok(())
}
```

### 9.3 GIL Release Strategy

The simulation tick loop is CPU-bound Rust code with no Python object access. `py.allow_threads()` releases the GIL for the entire `n_ticks` loop. This enables:
- True parallelism when `multiprocessing` spawns separate processes (each has its own GIL)
- Concurrent simulation runs in the same process using `concurrent.futures.ThreadPoolExecutor` (since GIL is released during simulation, threads actually run concurrently for the CPU-bound portion)

**Recommended Python usage for maximum throughput:**
```python
# Use ProcessPoolExecutor for maximum isolation + no GIL contention
from concurrent.futures import ProcessPoolExecutor

with ProcessPoolExecutor(max_workers=16) as executor:
    futures = [executor.submit(run_one, params) for params in param_list]
    results = [f.result() for f in futures]
```

### 9.4 Build and Distribution

**maturin** is used to build the Python extension module:

```bash
# Install maturin
pip install maturin

# Build the extension module (debug)
maturin develop --features research-api

# Build the wheel for distribution
maturin build --release --features research-api --strip
# Output: target/wheels/civlab-0.1.0-cp310-cp310-linux_x86_64.whl
```

**`pyproject.toml` (in civ-engine):**
```toml
[build-system]
requires = ["maturin>=1.4,<2.0"]
build-backend = "maturin"

[project]
name = "civlab"
requires-python = ">=3.10"
dependencies = ["numpy>=1.24"]

[tool.maturin]
features = ["research-api"]
python-source = "python"
module-name = "civlab._civlab"
```

---

## 10. Non-Functional Requirements Table

### 10.1 Correctness — Determinism

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-C-01 | Tick-by-tick state hash match | 100% of ticks match across two independent runs with same seed | `fr_determinism_replay` test: compare `state_hash` per tick | CI gate: test must pass on every PR |
| NFR-C-02 | Cross-platform hash match | Same hash on x86_64 Linux, ARM64 macOS, WASM | Matrix CI build + determinism test on all three targets | CI matrix: GitHub Actions with `ubuntu-latest`, `macos-latest`, `ubuntu-arm` runners |
| NFR-C-03 | Fixed-point arithmetic enforcement | Zero `f32`/`f64` operations in simulation crates | Clippy `float_arithmetic` lint | `cargo clippy -D clippy::float_arithmetic` in CI (build fails on violation) |
| NFR-C-04 | RNG seeding coverage | Zero calls to unseeded RNG in simulation crates | grep-based CI scan for `thread_rng`, `rand::random`, `OsRng` | Pre-commit hook + CI scan step |
| NFR-C-05 | BTreeMap enforcement | Zero `HashMap` in simulation state types | Custom clippy lint `sim_hashmap_forbidden` | Pre-commit hook scans `use std::collections::HashMap` in sim crates |
| NFR-C-06 | Event log completeness | Every state-mutating phase produces at least one event per tick | `fr_event_completeness` test: assert `events.len() > 0` per tick | Unit test in `civ-engine/tests/` |
| NFR-C-07 | State hash in every event | `SimEvent.state_hash` must match `state_hash` at tick of emission | Replay verifier: recompute hash, compare | `civ-replay` verifier runs on `.civreplay` files in CI |

### 10.2 Performance — Tick Latency

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-P-01 | p50 tick time (1k citizens) | ≤ 8 ms | `TICK_DURATION_HISTOGRAM` p50, scraped by Prometheus | CI bench regression: `criterion` baseline comparison on PR |
| NFR-P-02 | p99 tick time (1k citizens) | ≤ 14 ms | `TICK_DURATION_HISTOGRAM` p99 | CI bench regression gate: fail PR if p99 increases > 10% |
| NFR-P-03 | p999 tick time (1k citizens) | ≤ 16 ms | `TICK_DURATION_HISTOGRAM` p999 | Prometheus alert in production |
| NFR-P-04 | p50 tick time (10k citizens) | ≤ 30 ms | Same histogram, different scenario | Separate `bench_10k_citizens` criterion benchmark |
| NFR-P-05 | p50 tick time (100k citizens) | ≤ 150 ms | Same histogram, large scenario | Performance regression test in nightly CI only |
| NFR-P-06 | Broadcast lag | p99 ≤ 10 ms from tick completion to last client delivery | `BROADCAST_LAG_HISTOGRAM` | Prometheus alert: `civlab_broadcast_lag_seconds{quantile="0.99"} > 0.010` |
| NFR-P-07 | Snapshot serialization overhead | ≤ 1 ms per tick for 1k citizens | `TICK_PHASE_DURATION{phase="Snapshot"}` | Criterion benchmark `bench_snapshot_1k` |
| NFR-P-08 | Memory footprint | ≤ 256 MB RSS for 10k citizens | `/proc/self/status` VmRSS in health endpoint | Nightly memory regression test |

### 10.3 Scalability

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-S-01 | Max simultaneous WebSocket clients | ≥ 100 clients at 10 ticks/sec | Load test: 100 concurrent `tokio-tungstenite` clients | Load test in CI (`tests/load/100_clients.rs`) |
| NFR-S-02 | Client connection overhead | ≤ 5 ms per client connection (handshake + initial snapshot) | WebSocket upgrade + handshake response latency percentile | Integration test with timer |
| NFR-S-03 | Citizen count scaling | Tick time scales sub-linearly from 1k to 10k citizens | Ratio: `tick_time_10k / tick_time_1k ≤ 8` (expect ~5 with rayon) | Criterion comparison benchmark |
| NFR-S-04 | Command throughput | ≥ 1,000 commands/sec accepted without tick delay | Stress test: flood `command_tx` at 1k/sec, verify tick time unchanged | Load test with command flood |
| NFR-S-05 | Event log growth rate | ≤ 5 MB/minute at 1k citizens, 10 ticks/sec | Monitor `civlab_event_log_bytes_total` | Prometheus recording rule + alert |
| NFR-S-06 | WebSocket frame size | ≤ 20 KB average binary frame for 1k citizen snapshot | `FRAME_SIZE_HISTOGRAM` | Unit test on `BinaryFrame::to_msgpack_bytes()` with reference snapshot |

### 10.4 Reliability — Crash Recovery

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-R-01 | Snapshot persistence interval | Snapshot written to PostgreSQL every 100 ticks | `civlab_snapshots_written_total` counter | Integration test: run 100 ticks, verify DB has 1 snapshot row |
| NFR-R-02 | Recovery point objective (RPO) | On crash, resume from last persisted snapshot (≤ 100 ticks lost) | Kill server mid-run, restart, verify tick counter | Recovery integration test |
| NFR-R-03 | Recovery time objective (RTO) | Server restart + state load ≤ 30 seconds | Time from process start to first tick broadcast | Health check endpoint `/health` transitions from `starting` to `ready` |
| NFR-R-04 | Event log durability | Event log flushed to disk before acknowledgement | `fsync` on event log append (O_DSYNC) | Unit test: write event, kill process, verify log on restart |
| NFR-R-05 | Client reconnect | Client can reconnect and receive current snapshot within 2 seconds | Integration test: disconnect client, reconnect, measure time to first snapshot | WebSocket reconnect test |
| NFR-R-06 | Simulation panic isolation | Panic in one tick phase does not kill the server process | Inject panic via test endpoint, verify server continues | Integration test with panic injection |

### 10.5 Observability

| # | Metric | Target | Measurement Method | Enforcement Mechanism |
|---|---|---|---|---|
| NFR-O-01 | Prometheus metric coverage | 100% of tick phases have latency histograms | Count `HistogramVec` labels vs `PhaseId` enum variants | CI test: verify each `PhaseId` has a corresponding metric |
| NFR-O-02 | Structured log completeness | Every error has structured fields: `tick`, `phase`, `entity_id`, `error` | Log schema validation in CI | `tracing` instrumentation review checklist |
| NFR-O-03 | Trace propagation | Every WebSocket command is traceable from client receipt to tick application | `tracing::span` with `trace_id` propagated through command → phase | Manual trace inspection in Jaeger |
| NFR-O-04 | Metrics cardinality | Total Prometheus time series count ≤ 10,000 | Prometheus cardinality API: `count({__name__=~".+"})` | Prometheus alert: `prometheus_tsdb_head_series > 10000` |
| NFR-O-05 | Dashboard coverage | All NFR metrics visible in Grafana dashboard | Manual dashboard review | Dashboard JSON committed to repo at `ops/grafana/civ-sim.json` |
| NFR-O-06 | Alert coverage | Each p99 latency target has a Prometheus alerting rule | Count alert rules vs NFR-P-* count | CI: validate `ops/prometheus/alerts.yml` with `promtool check rules` |

---

## Appendix A: Acceptance Criteria Cross-Reference

Each section above maps to Functional Requirements. The canonical FR list is in the `docs/specs/` directory. This table provides the cross-reference for traceability.

| Spec Section | FR IDs | Source Spec |
|---|---|---|
| §4 ECS World Design | FR-CIV-CORE-019, FR-CIV-ACT-001 | CIV-0001 |
| §6 Determinism Architecture | FR-CIV-CORE-001 through 015 | CIV-0001 |
| §7 Server Architecture | FR-CIV-PROTO-001 through 015 | CIV-0200 |
| §9 Python Research Bindings | FR-CIV-RESEARCH-001 through 004 | PLAN.md P5 |
| §10 NFR Table | All NFR-* | ADR-003, ADR-004 |
| §2.2 ECS: legion | ADR-006 | ADR.md |
| §2.4 Fixed-point | ADR-007 | ADR.md |
| §2.7 WebSocket | ADR-005 | ADR.md |
| §2.1 Workspace layout | ADR-001 | ADR.md |

---

## Appendix B: Key Invariants Summary

For quick reference, the full set of hard invariants enforced at compile time or CI time:

| Invariant | Rule | Enforcement Level |
|---|---|---|
| No floats in simulation | `clippy::float_arithmetic` deny | Compile-time |
| No unseeded RNG | CI grep scan | CI gate |
| No HashMap in sim state | `sim_hashmap_forbidden` lint | Pre-commit + CI |
| No SystemTime in sim | CI grep scan | CI gate |
| No cycles in crate deps | `tach check` | CI gate |
| Sorted entity iteration | Code pattern + review | Code review |
| Joule conservation | proptest property | Test suite |
| Ledger balance non-negative | `Ledger::transfer` checks | Runtime assertion |
| Phase order fixed | `PhaseSchedule` ordering | Compile-time (enum) |
| State hash per tick | `compute_state_hash` always called | Test coverage |

---

## Appendix C: Version History

| Version | Date | Changes |
|---|---|---|
| 1.0 | (earlier) | 32-line stub scaffold |
| 2.0 | 2026-02-21 | Full engineering-grade expansion. ~2,700+ lines. All sections per spec brief. Derived from CIV-0001, CIV-0200, PLAN.md, ADR.md. |
