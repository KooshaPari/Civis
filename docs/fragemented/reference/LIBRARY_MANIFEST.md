# CivLab Library Manifest

**Document ID:** CIVLAB-LIB-MANIFEST-001
**Version:** 1.0.0
**Status:** ACTIVE
**Date:** 2026-02-21
**Owner:** CivLab Platform Engineering
**Related Specs:**
- `PLAN.md` — Phase plan, crate structure, DAG dependencies
- `docs/specs/CIV-0001-core-simulation-loop.md` — Core tick architecture, determinism invariants
- `docs/reference/ENGINEERING_PROCESS_SUMMARY.md` — Engineering standards and process
- `docs/reference/SERVICE_CATALOG.md` — Service catalog and health contracts

---

## Table of Contents

1. [Philosophy and Governance](#1-philosophy-and-governance)
2. [Core Simulation Crates](#2-core-simulation-crates)
   - 2.1 [RNG: rand_chacha](#21-rng-rand_chacha)
   - 2.2 [ECS: legion vs. bevy_ecs vs. specs vs. hecs](#22-ecs-decision-matrix)
   - 2.3 [Fixed-Point Arithmetic: fixed vs. manual i64×SCALE](#23-fixed-point-arithmetic)
   - 2.4 [Data Parallelism: rayon](#24-data-parallelism-rayon)
   - 2.5 [Hashing: blake3](#25-hashing-blake3)
3. [Server Crates](#3-server-crates)
4. [Data Crates](#4-data-crates)
5. [Observability Crates](#5-observability-crates)
6. [Spatial and Math Crates](#6-spatial-and-math-crates)
7. [Testing Crates](#7-testing-crates)
8. [CLI and Configuration Crates](#8-cli-and-configuration-crates)
9. [Python FFI Crates](#9-python-ffi-crates)
10. [Build Tooling](#10-build-tooling)
11. [Pinned Version Lock Table](#11-pinned-version-lock-table)

---

## 1. Philosophy and Governance

### 1.1 Library-First Mandate

CivLab treats every engineering task that involves a "common" problem — retry logic, serialization, data structures, hash functions, random number generation, spatial math, HTTP networking — as a library problem first. The decision path is:

1. **Does a well-maintained crate solve 80%+ of this need?** If yes, use it.
2. **Can a thin wrapper around the library solve the remaining 20%?** If yes, wrap it. Keep wrappers under 50 LOC.
3. **Is there genuinely novel domain logic not covered by any library?** Only then is custom code acceptable.

The bar for "genuinely novel" is high. Simulation-domain logic (tick sequencing, deterministic event ordering, economy model rules, spatial hex arithmetic) qualifies. Retry loops, serialization formats, hash functions, and RNG algorithms do not.

### 1.2 ADR Requirement for Custom Implementations

If a domain area that could reasonably be served by a library is instead implemented custom, an Architecture Decision Record (ADR) is required before implementation begins. The ADR must:

- Name every library evaluated and why each was rejected.
- State the specific property that no library provides (e.g., "no crate implements lockstep deterministic tick ordering for heterogeneous ECS worlds").
- Define a clear acceptance test that the custom implementation must pass.
- Include a future-proofing note: if a library eventually covers the gap, migration is expected.

ADRs live in `docs/reference/ADR_STATUS.md`. No custom implementation of a commonly-available capability is merged without a linked ADR entry.

### 1.3 Version Pinning Policy

All dependencies are pinned to exact minor versions in `Cargo.toml` using `=` or exact lockfile commitment. Automated dependency updates (via Dependabot or Renovate) are gated by the full test suite, determinism replay tests, and a manual review for any crate touching the simulation core. Unpinned floating ranges (`*`, `^1`) are disallowed for simulation-core crates. Server and tooling crates may use caret ranges within a defined minor band.

### 1.4 Security and Supply Chain

All dependencies are checked with `cargo-deny` (license allowlist, advisory database) and `cargo-audit` (RustSec advisories) in CI. Any CRITICAL or HIGH advisory that affects a simulation-core or server crate triggers an immediate block on merging until the dependency is updated or a waiver is documented.

### 1.5 Determinism as a First-Class Constraint

The single most important non-functional requirement of the CivLab simulation core is **bitwise determinism**: given the same seed and the same sequence of input events, two separate runs of the simulation must produce identical state snapshots at every tick. This constraint directly governs library selection. Any library that:

- Uses thread-local RNG state
- Has platform-specific floating-point behavior
- Performs non-deterministic hash ordering (e.g., `HashMap` iteration without stable ordering)
- Introduces OS-level timing dependencies

...is disqualified from use in the simulation core. Each library section below includes a dedicated **Determinism Implications** subsection that explains exactly what guarantees the library provides and what constraints are imposed on its usage.

---

## 2. Core Simulation Crates

### 2.1 RNG: rand_chacha

#### Decision

**Selected:** `rand_chacha` 0.3.3 (pinned exact)
**Provides:** `ChaCha8Rng`, `ChaCha12Rng`, `ChaCha20Rng`
**Used variant:** `ChaCha8Rng` for performance-critical paths; `ChaCha20Rng` for cryptographic-strength requirements (scenario seeding, replay validation)

#### Full Comparison Matrix

| Property | rand_chacha (ChaCha20) | rand_pcg (PCG64) | rand_xoshiro (Xoshiro256++) | SmallRng (platform-specific) |
|---|---|---|---|---|
| **Cross-platform bitwise identical output** | YES — explicit byte-order spec | PARTIAL — PCG spec portable but some impls vary | YES — spec portable | NO — intentionally platform-specific |
| **Cryptographic quality** | YES — ChaCha20 is a stream cipher | NO — statistically good, not cryptographic | NO — fails BigCrush under specific seeds | NO |
| **Performance (ns/64-bit)** | ~2.1 ns (ChaCha8) | ~1.2 ns | ~0.9 ns | ~0.7 ns |
| **Deterministic from seed** | YES | YES | YES | NO |
| **Output stability across crate versions** | YES — documented API guarantee | YES | YES | NO |
| **Jumpability / stream separation** | YES — set_stream() + set_word_pos() | PARTIAL | YES — jump() | NO |
| **License** | MIT/Apache-2.0 | MIT/Apache-2.0 | MIT/Apache-2.0 | MIT/Apache-2.0 |

#### Why ChaCha Over PCG or Xoshiro

**PCG64** is faster but its Rust implementation has historically had subtle divergences between versions in the high bits of output. For a simulation that must produce bitwise-identical replays across different build environments and crate versions, "statistically good" is not sufficient — the byte-level output must be stable. ChaCha20's output is defined by an IETF RFC (RFC 7539), meaning the output for a given key+nonce is specified at the byte level and cannot change across implementations.

**Xoshiro256++** is excellent for non-cryptographic simulation work and would be acceptable for many games. However, it has known weaknesses under linear seed construction (low Hamming weight seeds produce correlated output in the low bits for the first several thousand outputs). CivLab scenario seeds are generated from user-provided strings and hashed, which can produce low-weight seeds in degenerate cases. ChaCha does not have this weakness.

**SmallRng** is explicitly disqualified. The `rand` crate documentation states: "SmallRng is not portable; its implementation may change in the future, and its output may differ between platforms." This directly violates CivLab's determinism invariant.

#### Key API Used

```rust
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

// Seeded from scenario seed (u64) — deterministic
let rng = ChaCha8Rng::seed_from_u64(scenario.seed);

// Seeded from full 256-bit key (for cryptographic-strength scenario hashing)
use rand_chacha::ChaCha20Rng;
let rng = ChaCha20Rng::from_seed(seed_bytes); // [u8; 32]

// Stream separation for parallel phases (each phase gets a unique stream)
let mut phase_rng = rng.clone();
phase_rng.set_stream(phase_id as u64);
```

#### Determinism Implications

- `ChaCha8Rng` and `ChaCha20Rng` produce bitwise-identical output for a given seed across all platforms supported by Rust's `u32` and `u64` types.
- The `rand_chacha` crate commits to output stability in its changelog; breaking changes to the output stream require a major version bump.
- **RULE:** No RNG instance may be created in simulation code without being seeded from `Simulation::rng_seed`. Thread-local or OS-seeded RNG (`rand::thread_rng()`, `OsRng`) is forbidden in simulation-phase code. It is permitted in test scaffolding and server-layer code where determinism is not required.
- **RULE:** RNG instances must not be shared across parallel rayon iterators without explicit stream separation via `set_stream()`. Each parallel worker must derive its own stream from a common seed.
- **Stream separation pattern** for parallel phases:

```rust
// In parallel tick phase, each entity gets a deterministic sub-RNG
use rayon::prelude::*;
entities.par_iter_mut().enumerate().for_each(|(idx, entity)| {
    let mut entity_rng = base_rng.clone();
    entity_rng.set_stream(idx as u64);
    entity_rng.set_word_pos(tick as u128);
    entity.apply_random_event(&mut entity_rng);
});
```

---

### 2.2 ECS Decision Matrix

#### Problem Statement

CivLab simulates potentially hundreds of thousands of entities (citizens, tiles, institutions, military units) per tick with complex component compositions. The ECS (Entity-Component-System) pattern is the standard approach for this scale in Rust game engines. Four mature options exist: `bevy_ecs`, `legion`, `specs`, and `hecs`. The correct choice has major implications for performance, parallelism, API ergonomics, and long-term maintenance.

#### Full Decision Matrix

| Criterion | bevy_ecs 0.15 | legion 0.4 | specs 0.20 | hecs 0.10 |
|---|---|---|---|---|
| **Standalone (no full engine)** | YES — `bevy_ecs` can be used without rest of Bevy | YES — standalone crate | YES | YES |
| **Archetype-based storage** | YES | YES | NO (bitset + heterogeneous) | YES |
| **Parallel system execution** | YES — `par_iter()` native | YES — `par_iter()` native | YES — via rayon | MANUAL — no system scheduler |
| **Query API ergonomics** | EXCELLENT — proc macros, filter sets | GOOD — explicit query types | ACCEPTABLE — complex setup | MINIMAL — no scheduler at all |
| **Compile-time query validation** | YES | PARTIAL | NO — runtime panics | PARTIAL |
| **Change detection** | YES — `Changed<T>` filter | NO | PARTIAL — flagged components | NO |
| **System ordering/scheduling** | YES — stages + labels | MANUAL — external scheduler | YES — via Dispatcher | NONE |
| **Bevy compatibility (if needed)** | NATIVE | NO | NO | NO |
| **Active maintenance (2026)** | HIGH — Bevy project | STAGNANT — last release 2021 | MODERATE | MODERATE |
| **Ecosystem size** | LARGE | SMALL | MEDIUM | SMALL |
| **Benchmark: 1M entity iter (ms)** | ~3.2 ms | ~2.8 ms | ~6.1 ms | ~2.1 ms |
| **Benchmark: Sparse query (ms)** | ~1.1 ms | ~0.9 ms | ~3.4 ms | ~0.7 ms |
| **Documentation quality** | EXCELLENT | POOR | GOOD | MINIMAL |
| **Migration stability** | BREAKING between minors | BREAKING (abandoned) | STABLE | STABLE |

#### Decision: bevy_ecs

**Selected:** `bevy_ecs` 0.15.x (standalone, not full Bevy engine)

**Primary reasons:**

1. **Legion is effectively abandoned.** The last release was 0.4.0 in 2021. The GitHub repository has not seen a substantive commit since early 2022. Using an abandoned ECS as the foundation of a multi-year simulation project is unacceptable regardless of its performance characteristics.

2. **bevy_ecs is actively developed and has a large ecosystem.** The Bevy project has multiple full-time contributors, releases on a regular cadence, and a large community producing plugins and extensions. If CivLab ever adds a Bevy-based reference renderer (a likely Phase 3 output), the simulation core's ECS will be directly compatible.

3. **specs has inferior archetype storage.** specs uses a bitset-based component storage model that has higher cache miss rates on dense entity queries than archetype storage. At 100,000+ entities with 8-12 components each, this translates to a measurable tick latency penalty (~2x on dense queries per benchmarks).

4. **hecs is a building block, not a framework.** hecs provides excellent raw ECS performance but no scheduling, no change detection, and no system ordering. Using hecs would require building all of this infrastructure manually — exactly the "reinventing wheels" anti-pattern the library-first mandate prohibits.

5. **bevy_ecs change detection is required.** The simulation core needs to efficiently detect which components changed in a tick to build incremental state snapshots for the WebSocket protocol. `bevy_ecs`'s `Changed<T>` and `Added<T>` query filters provide this with zero overhead on unchanged components.

#### Key API Used

```rust
use bevy_ecs::prelude::*;

// Component definitions
#[derive(Component)]
struct Position { q: i32, r: i32 }  // axial hex coords

#[derive(Component)]
struct Citizen {
    age: u32,
    health: fixed::types::I32F32,
    ideology: fixed::types::I16F16,
}

#[derive(Component)]
struct EconomicAgent {
    balance_joules: fixed::types::I64F0,
    employer: Option<Entity>,
}

// System definition
fn citizen_lifecycle_system(
    mut query: Query<(&mut Citizen, &EconomicAgent), Changed<EconomicAgent>>,
    tick: Res<SimulationTick>,
) {
    query.par_iter_mut().for_each(|(mut citizen, agent)| {
        citizen.age += 1;
        // ... lifecycle logic
    });
}

// World setup
let mut world = World::new();
let mut schedule = Schedule::default();
schedule.add_systems(citizen_lifecycle_system);
```

#### Determinism Implications

- `bevy_ecs` archetype storage provides **deterministic iteration order** within a single world given a fixed entity spawn order. Entities spawned in the same order produce the same archetype layout and the same query iteration order.
- **RULE:** Entities must be spawned in deterministic order (not from parallel threads without explicit ordering). The engine init phase is single-threaded; only the tick phases run parallel systems.
- `par_iter_mut()` in bevy_ecs does not guarantee order of execution across entities. **RULE:** No side effects between entities during parallel system execution. Entities may only mutate their own components; cross-entity writes must be staged via events and applied in a subsequent single-threaded phase.
- System execution order within a schedule is deterministic given the same system graph definition. **RULE:** All systems must be registered in the schedule's `configure_sets` block with explicit ordering dependencies.

---

### 2.3 Fixed-Point Arithmetic

#### Problem Statement

Floating-point arithmetic (`f32`, `f64`) is not suitable for deterministic simulation across platforms. IEEE 754 specifies rounding modes and fused multiply-add behavior that may differ between CPUs (x86 vs. ARM), compiler versions, and optimization levels. A simulation that uses `f64` for economic values will not produce bitwise-identical results when run on different machines.

Two options exist:
1. **`fixed` crate** — type-safe fixed-point numeric types with compile-time fractional bit specification
2. **Manual `i64 × SCALE`** — raw integer arithmetic with a constant scale factor, done by hand

#### Full Comparison

| Property | fixed 1.28 | Manual i64×SCALE |
|---|---|---|
| **Type safety** | YES — I32F32, I64F0, etc. are distinct types | NO — any i64 could be a scaled value |
| **Overflow detection** | YES — checked_add, saturating_mul | MANUAL — easy to miss |
| **Fractional precision** | Configurable at compile time | Fixed to one scale per codebase |
| **Standard ops (Add, Mul, Div)** | YES — operator overloading | MANUAL — must implement wrappers |
| **Serde support** | YES — serde feature flag | MANUAL |
| **Debug/Display** | YES — shows decimal equivalent | Shows raw integer |
| **Library ecosystem** | Self-contained | None |
| **Code volume to implement correctly** | ~50 LOC wrapper | 500-2000 LOC for safe wrapper layer |
| **ADR required** | NO (library exists) | YES |

#### Decision: fixed crate

**Selected:** `fixed` 1.28.0 (pinned exact)

The `fixed` crate provides compile-time type-safe fixed-point arithmetic. Using `I32F32` for a value means 32 integer bits and 32 fractional bits; using `I64F0` means 64 integer bits and 0 fractional bits (i.e., a regular integer with the same type system). The types implement all standard arithmetic traits, support `serde`, and implement `PartialOrd` correctly.

Manual `i64×SCALE` is not selected because:
1. It requires significant engineering effort to implement correctly (overflow, division, display).
2. It does not provide type safety — a scale mismatch between two integer values is a silent bug.
3. It would require an ADR justifying why no library was used, and no such justification exists.

#### Key API Used

```rust
use fixed::types::{I32F32, I64F0, I16F16};
use fixed::traits::Fixed;

// Economic values: high integer range, no fractional needed
type Joules = I64F0;

// Price indices: moderate range, 32-bit fractional precision
type Price = I32F32;

// Ideology: [-1.0, 1.0] range, 16-bit fractional sufficient
type IdeologyScore = I16F16;

// Arithmetic
let supply = Joules::from_num(1000);
let demand = Joules::from_num(800);
let surplus = supply.checked_sub(demand).expect("economic invariant violated");

// Conversion to f64 for display/export only (never for computation)
let display_value: f64 = surplus.to_num::<f64>();
```

#### Determinism Implications

- Fixed-point arithmetic on integers is bitwise deterministic across all platforms.
- `fixed` types do not use any floating-point hardware instructions.
- **RULE:** `f32` and `f64` are forbidden in simulation-core component values. They may only appear in rendering hints, metric export (for human readability), and Python FFI boundary conversions.
- `to_num::<f64>()` conversion is acceptable for display/export but must never feed back into simulation state.

---

### 2.4 Data Parallelism: rayon

#### Crate Details

| Property | Value |
|---|---|
| **Crate** | `rayon` |
| **Version** | `2.10.0` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Data-parallel iteration for independent simulation phases |

#### Why rayon

The CivLab tick loop runs several phases that operate over large collections of independent entities:
- Citizen lifecycle updates (100k+ entities, fully independent per entity)
- Economic clearing (per-market, parallelizable across markets)
- Climate diffusion (per-tile, parallelizable across tiles given read-only neighbor access)
- Military combat resolution (per-battle, fully independent)

Rayon provides a work-stealing thread pool that automatically distributes work across available CPU cores. The API is a direct extension of standard Rust iterators — `par_iter()` replaces `iter()` with zero algorithmic change.

Alternative: `tokio::task::spawn_blocking` — rejected because tokio is an async executor for I/O-bound concurrency, not for CPU-bound data parallelism. Mixing tokio tasks for CPU work produces suboptimal scheduling and unpredictable latency.

#### Key API Used

```rust
use rayon::prelude::*;

// Parallel citizen tick — O(N) independent operations
citizens
    .par_iter_mut()
    .for_each(|citizen| citizen.tick(&tick_state));

// Parallel market clearing — each market is independent
markets
    .par_iter_mut()
    .for_each(|market| market.clear_prices(&supply_demand));

// Parallel map-reduce for aggregate statistics
let total_population: u64 = citizens
    .par_iter()
    .filter(|c| c.is_alive())
    .map(|_| 1u64)
    .sum();
```

#### Determinism Implications

- Rayon does NOT guarantee the order in which work items are processed. **This is by design** and is acceptable because CivLab's parallel phases require independence (no entity reads another entity's state being mutated in the same phase).
- For operations that require a deterministic aggregate result (sums, sorts), use `.sum()`, `.product()`, or sort after collecting. These are deterministic regardless of processing order.
- **RULE:** No entity may read the mutable state of another entity during a parallel phase. Cross-entity dependencies must be handled via the read-phase → write-phase split in the tick loop.
- `rayon::ThreadPoolBuilder::new().num_threads(n).build()` is used in test mode to fix thread count, ensuring that test runs are reproducible in terms of scheduling behavior (though not required for correctness).

---

### 2.5 Hashing: blake3

#### Full Comparison Matrix

| Property | blake3 | sha2 (SHA-256) | xxhash (xxh3) | ahash |
|---|---|---|---|---|
| **SIMD acceleration** | YES — AVX-512, NEON, auto-detected | PARTIAL — SHA-NI instruction set | YES — vector hashing | YES |
| **Parallelizable** | YES — tree hash, parallel across chunks | NO — sequential by design | NO | NO |
| **Speed vs SHA-256** | ~3-5x faster | Baseline | ~8x faster | ~10x faster |
| **Cryptographic security** | YES — 256-bit, collision-resistant | YES | NO — not cryptographic | NO |
| **Streaming API** | YES — `Hasher::update()` | YES | YES | YES |
| **Keyed hashing** | YES — `blake3::keyed_hash()` | NO (use HMAC separately) | NO | YES |
| **Standard library compat** | YES — `std::hash::Hasher` trait | NO (via `sha2::Digest` trait) | YES | YES |
| **License** | Apache-2.0 / CC0 | MIT | BSD-2 | MIT/Apache-2.0 |

#### Decision: blake3

**Selected:** `blake3` 1.5.4 (pinned exact)

`blake3` is used for:
1. **State snapshot hashing** — deterministic hash of SimulationSnapshot for replay validation
2. **Scenario content hashing** — hash of scenario YAML for content-addressable caching
3. **Event log integrity** — chained hashes of tick events for tamper-evident replay files

SHA-256 (`sha2` crate) would also be acceptable for security properties, but blake3 is ~3-5x faster due to SIMD parallelism across the hash tree. For a simulation that hashes large state snapshots on every tick, this performance difference is meaningful.

`xxhash` (xxh3) is rejected because it is not cryptographically secure. CivLab replay files are used for research auditability; a non-cryptographic hash allows crafting collisions that produce incorrect replay validation results.

`ahash` is the correct choice for `HashMap` internal hashing (and is the default in Rust's `HashMap` via `hashbrown`), but is not suitable for content-addressable hashing where the output must be stable across processes and versions.

#### Key API Used

```rust
use blake3::{Hasher, hash, keyed_hash};

// Hash a simulation snapshot for integrity verification
fn hash_snapshot(snapshot: &SimulationSnapshot) -> [u8; 32] {
    let serialized = rmp_serde::to_vec(snapshot).expect("snapshot serialization failed");
    *blake3::hash(&serialized).as_bytes()
}

// Chained hash for replay integrity (each event hashes previous)
fn chain_hash(prev_hash: &[u8; 32], event_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(prev_hash);
    hasher.update(event_bytes);
    *hasher.finalize().as_bytes()
}

// Keyed hash for scenario identity (prevents cross-scenario collisions)
fn scenario_hash(scenario_bytes: &[u8], key: &[u8; 32]) -> blake3::Hash {
    blake3::keyed_hash(key, scenario_bytes)
}
```

#### Determinism Implications

- `blake3` output is fully deterministic and platform-independent. The BLAKE3 specification fixes the output for any given input.
- SIMD acceleration is transparent; it does not affect the hash value, only performance.
- **RULE:** All replay file integrity hashes must use `blake3`. Using a different hash function in a replay file breaks verification by clients using the standard decoder.

---

## 3. Server Crates

The CivLab server layer (Phase 3: Client Protocol) exposes the simulation engine via a WebSocket API and an optional REST API for scenario management.

### 3.1 Async Runtime: tokio

| Property | Value |
|---|---|
| **Crate** | `tokio` |
| **Version** | `1.44.0` |
| **Features** | `full` (or `rt-multi-thread, macros, net, sync, time, fs`) |
| **License** | MIT |
| **Purpose** | Async runtime for all network I/O, timers, and concurrent server tasks |

**Why tokio:** tokio is the de facto standard async runtime for production Rust services. It provides a multi-threaded work-stealing scheduler, `TcpListener`/`TcpStream`, channels (`mpsc`, `broadcast`, `watch`), timers, and filesystem I/O. The ecosystem of async crates (`axum`, `tokio-tungstenite`, `sqlx`, `redis`) is built on tokio. Using an alternative runtime (smol, async-std) would require vendoring or forking all dependencies.

**Key API:**
```rust
#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream, addr));
    }
}
```

### 3.2 HTTP Framework: axum

| Property | Value |
|---|---|
| **Crate** | `axum` |
| **Version** | `0.8.1` |
| **License** | MIT |
| **Purpose** | REST API for scenario management, health checks, metrics exposure |

**Why axum:** axum is built on `tower` and `hyper`, meaning all tower middleware (rate limiting, tracing, compression, auth) composes directly. It provides type-safe extractors, automatic JSON serialization via `serde_json`, and native tokio integration. The alternative (`actix-web`) has a different actor model that adds complexity without benefit for CivLab's simple REST surface.

**Key API:**
```rust
use axum::{routing::{get, post}, Router, Json, extract::State};

let app = Router::new()
    .route("/health", get(health_handler))
    .route("/scenarios", post(create_scenario_handler))
    .route("/scenarios/:id/run", post(run_scenario_handler))
    .with_state(app_state);
```

### 3.3 WebSocket: tokio-tungstenite

| Property | Value |
|---|---|
| **Crate** | `tokio-tungstenite` |
| **Version** | `0.24.0` |
| **License** | MIT |
| **Purpose** | WebSocket server for real-time simulation tick broadcast |

**Why tokio-tungstenite:** tungstenite is the most complete WebSocket implementation in the Rust ecosystem, supporting the full RFC 6455 spec including ping/pong, continuation frames, and close handshakes. The `tokio-tungstenite` wrapper provides async integration with no blocking. Alternative: `warp`'s built-in WebSocket — rejected because warp's API is less ergonomic and it does not support the fine-grained control needed for binary frame encoding.

**Key API:**
```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_ws(stream: tokio::net::TcpStream) {
    let mut ws = accept_async(stream).await.expect("ws handshake failed");
    loop {
        let msg = Message::Binary(snapshot_bytes.clone());
        ws.send(msg).await.expect("ws send failed");
        tokio::time::sleep(tick_duration).await;
    }
}
```

### 3.4 Middleware Stack: tower

| Property | Value |
|---|---|
| **Crate** | `tower` |
| **Version** | `0.5.2` |
| **License** | MIT |
| **Purpose** | Service abstraction, middleware composition (rate limiting, tracing, retry) |

`tower` provides the `Service` and `Layer` traits that axum, hyper, and tonic all use as their middleware interface. CivLab uses `tower::limit::RateLimitLayer` for the scenario API and `tower_http::trace::TraceLayer` for request tracing.

### 3.5 HTTP Client: hyper

| Property | Value |
|---|---|
| **Crate** | `hyper` |
| **Version** | `1.6.0` |
| **License** | MIT |
| **Purpose** | Low-level HTTP client for outbound calls (scenario import, metric export) |

`hyper` is used indirectly through `reqwest` for outbound HTTP. Direct `hyper` is used only for the WebSocket upgrade path where fine-grained control is needed.

---

## 4. Data Crates

### 4.1 Database Driver: sqlx

| Property | Value |
|---|---|
| **Crate** | `sqlx` |
| **Version** | `0.8.3` |
| **Features** | `postgres, runtime-tokio, macros, chrono, uuid` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Async PostgreSQL queries, compile-time query checking, migrations |

**Why sqlx:** sqlx provides compile-time verified SQL queries via `sqlx::query!` and `sqlx::query_as!` macros, which check queries against a live database at compile time (or against a pre-saved schema snapshot via `SQLX_OFFLINE=true`). This eliminates entire classes of runtime query errors. No ORM is used — SQL is written directly. Diesel (the alternative ORM) is rejected because its synchronous API does not integrate with tokio async code without blocking threads.

**Key API:**
```rust
use sqlx::PgPool;

// Compile-time checked query
let scenario = sqlx::query_as!(
    ScenarioRow,
    "SELECT id, name, seed, parameters FROM scenarios WHERE id = $1",
    scenario_id
)
.fetch_one(&pool)
.await?;

// Migrations
sqlx::migrate!("./migrations").run(&pool).await?;
```

### 4.2 Cache: deadpool-redis

| Property | Value |
|---|---|
| **Crate** | `deadpool-redis` |
| **Version** | `0.18.0` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Async Redis connection pool for session state, idempotency keys, hot metric caching |

`deadpool-redis` provides an async connection pool over `redis-rs`. The server uses Redis for:
- Client session state (who is connected, which scenario they are viewing)
- Idempotency keys for scenario run requests (prevent double-runs)
- Hot metric cache (last 10 ticks of snapshots for new client catch-up)

**Key API:**
```rust
use deadpool_redis::{Pool, Config};
use redis::AsyncCommands;

let cfg = Config::from_url("redis://127.0.0.1/");
let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap();

let mut conn = pool.get().await?;
conn.set_ex::<_, _, ()>("session:client-1", "scenario-42", 3600).await?;
```

### 4.3 Serialization: serde + rmp-serde + serde_json

| Crate | Version | Purpose |
|---|---|---|
| `serde` | `1.0.219` | Derive macros for all serializable types |
| `serde_json` | `1.0.135` | JSON serialization for WebSocket text frames and REST API |
| `rmp-serde` | `1.3.0` | MessagePack serialization for binary WebSocket frames and storage |

**Why MessagePack for binary frames:** The WebSocket protocol supports both text (JSON) and binary frames. For high-frequency tick broadcasts (up to 10 ticks/sec), MessagePack (`rmp-serde`) produces payloads approximately 30-50% smaller than JSON for the same data. This reduces bandwidth and parsing overhead on the client. JSON is used for the REST API (human-readable debugging) and the text-frame fallback for browser clients.

**Key API:**
```rust
// Serialize snapshot to MessagePack binary
let bytes = rmp_serde::to_vec(&snapshot)?;
ws.send(Message::Binary(bytes)).await?;

// Deserialize command from JSON text frame
let command: ClientCommand = serde_json::from_str(&text)?;
```

### 4.4 Compression: zstd

| Property | Value |
|---|---|
| **Crate** | `zstd` |
| **Version** | `0.13.3` |
| **License** | MIT/BSD |
| **Purpose** | Compression of replay files (.civreplay), snapshot archives |

`zstd` is used for compressing replay files before writing to disk. At compression level 3 (default), zstd achieves ~3-5x compression ratios on simulation state data while maintaining very fast decompression speeds (1+ GB/s). This keeps replay files manageable for long-running scenarios.

---

## 5. Observability Crates

### 5.1 Structured Logging: tracing + tracing-subscriber

| Crate | Version | Purpose |
|---|---|---|
| `tracing` | `0.1.41` | Structured spans and events throughout simulation |
| `tracing-subscriber` | `0.3.19` | Subscriber implementations (stdout JSON, file, filtering) |

All log output uses `tracing` macros (`trace!`, `debug!`, `info!`, `warn!`, `error!`). No `println!` or `log::` macros appear in production code. Structured fields attach context without string interpolation:

```rust
tracing::info!(
    tick = tick_number,
    entity_count = world.entities().len(),
    duration_ms = elapsed.as_millis(),
    "tick completed"
);
```

`tracing-subscriber` is configured with `EnvFilter` to allow runtime log level control via `RUST_LOG` environment variable.

### 5.2 OpenTelemetry Integration: opentelemetry

| Crate | Version | Purpose |
|---|---|---|
| `opentelemetry` | `0.28.0` | OTLP trace export to Jaeger/Tempo |
| `opentelemetry-otlp` | `0.28.0` | OTLP exporter |
| `tracing-opentelemetry` | `0.29.0` | Bridge between tracing spans and OTel spans |

Distributed traces span from WebSocket client connection through engine tick through snapshot broadcast. Each tick is a root span with child spans for each simulation phase (economy, actors, spatial, climate).

### 5.3 Metrics: prometheus client

| Property | Value |
|---|---|
| **Crate** | `prometheus` |
| **Version** | `0.13.4` |
| **License** | Apache-2.0 |
| **Purpose** | Expose Prometheus metrics at `/metrics` endpoint |

Key metrics exported:
- `civlab_tick_duration_seconds` (histogram) — per-tick compute time
- `civlab_entity_count` (gauge) — live entity counts by type
- `civlab_connected_clients` (gauge) — WebSocket clients
- `civlab_snapshot_size_bytes` (histogram) — snapshot payload sizes
- `civlab_economy_gdp` (gauge) — simulation GDP metric

---

## 6. Spatial and Math Crates

### 6.1 SIMD Math: glam

| Property | Value |
|---|---|
| **Crate** | `glam` |
| **Version** | `0.29.2` |
| **Features** | `scalar-math` in determinism mode; default SIMD in server mode |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Vec2, Vec3, Mat4 for spatial queries, terrain rendering hints, distance calculations |

`glam` provides SIMD-accelerated vector and matrix math. For the simulation core where determinism is required, `glam` is compiled with `scalar-math` feature which disables SIMD and uses scalar fallbacks — this ensures cross-platform determinism. For the server layer where rendering hints are computed (not fed back into simulation state), SIMD mode is used for performance.

**Determinism rule:** `glam` types with SIMD enabled are FORBIDDEN in simulation component fields. They may only appear in rendering/display code.

### 6.2 Hexagonal Grid: Manual Axial Coordinates

CivLab uses axial coordinate system for hexagonal tiles as described in Redblobgames' authoritative reference (https://www.redblobgames.com/grids/hexagons/). No external hex library is used because:

1. The required operations are simple: `(q, r)` axial coordinates, neighbor lookup (6 directions), distance calculation (`(|q| + |r| + |q+r|) / 2`), and ring/spiral traversal.
2. Available hex crates (`hexagonal`, `hex2d`) are either unmaintained or add unnecessary dependencies.
3. The implementation is fewer than 200 LOC — well below the ADR threshold.

An ADR is still filed (ADR-SPATIAL-001) documenting this decision and the specific Redblobgames formulas used, to prevent future engineers from introducing a competing implementation.

```rust
// Axial hex coordinates — deterministic, integer-only
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexPos { pub q: i32, pub r: i32 }

impl HexPos {
    pub fn neighbors(&self) -> [HexPos; 6] {
        const DIRS: [(i32, i32); 6] = [(1,0),(1,-1),(0,-1),(-1,0),(-1,1),(0,1)];
        DIRS.map(|(dq, dr)| HexPos { q: self.q + dq, r: self.r + dr })
    }

    pub fn distance(&self, other: &HexPos) -> u32 {
        let dq = self.q - other.q;
        let dr = self.r - other.r;
        ((dq.abs() + dr.abs() + (dq + dr).abs()) / 2) as u32
    }
}
```

---

## 7. Testing Crates

### 7.1 Property-Based Testing: proptest

| Property | Value |
|---|---|
| **Crate** | `proptest` |
| **Version** | `1.6.0` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Property-based tests for economic invariants, determinism properties, spatial math |

`proptest` generates random inputs to test invariants rather than specific cases. CivLab uses it to verify:
- **Joule conservation:** For any allocation, `sum(allocated) <= available`
- **Market price bounds:** For any set of supply/demand inputs, prices stay within `[0, MAX_PRICE]`
- **Determinism:** For any seed, two independent simulations with the same seed produce identical snapshots
- **Hex distance triangle inequality:** `d(a,c) <= d(a,b) + d(b,c)`

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn joule_allocation_never_exceeds_budget(
        available in 0u64..1_000_000u64,
        n_actors in 1usize..1000usize,
    ) {
        let allocations = allocate_joules(available, n_actors);
        prop_assert!(allocations.iter().sum::<u64>() <= available);
    }
}
```

### 7.2 Micro-Benchmarking: criterion

| Property | Value |
|---|---|
| **Crate** | `criterion` |
| **Version** | `0.5.1` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Statistical benchmarks for tick phases, serialization, hash operations |

`criterion` provides statistically rigorous benchmarks with warmup, outlier detection, and HTML reports. CivLab benchmarks (Phase 6) use criterion for:
- `sim.tick()` at 1,000, 10,000, and 100,000 entity counts
- `market.price_update()` at 10,000 goods
- `citizen.lifecycle_tick()` at 100,000 citizens
- Snapshot serialization: `rmp_serde::to_vec(&snapshot)`
- Snapshot hashing: `blake3::hash(bytes)`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_tick");
    for n in [1_000u32, 10_000, 100_000] {
        group.bench_with_input(BenchmarkId::new("tick", n), &n, |b, &n| {
            let mut sim = Simulation::new_with_n_citizens(n, BENCH_SEED);
            b.iter(|| sim.tick())
        });
    }
}
criterion_group!(benches, bench_tick);
criterion_main!(benches);
```

### 7.3 Snapshot Testing: insta

| Property | Value |
|---|---|
| **Crate** | `insta` |
| **Version** | `1.42.0` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Snapshot tests for serialized outputs, protocol messages, replay headers |

`insta` records the output of an expression on first run and asserts it matches on subsequent runs. Used for:
- Protocol message encoding (verify JSON/MessagePack format stability)
- Scenario YAML parsing (verify parsed structure matches expectation)
- Snapshot serialization format (detect unintended schema changes)

```rust
#[test]
fn test_server_message_encoding() {
    let msg = ServerMessage { tick: 42, state: test_snapshot() };
    let json = serde_json::to_string_pretty(&msg).unwrap();
    insta::assert_snapshot!(json);
}
```

### 7.4 Test Runner: cargo-nextest

| Property | Value |
|---|---|
| **Tool** | `cargo-nextest` |
| **Version** | `0.9.88` (installed via `cargo install`) |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Parallel test execution, per-test timeout, JUnit XML output for CI |

`cargo-nextest` runs tests significantly faster than `cargo test` by parallelizing at the test-function level rather than the binary level. It provides:
- Per-test timeout (prevents hung tests from blocking CI)
- JUnit XML output for CI systems
- Retry of flaky tests (determinism tests should never flake; retry count must be 0 for simulation-core tests)

CI command:
```bash
cargo nextest run --all-targets --failure-output=immediate --no-fail-fast \
  --retries 0 \
  --test-threads 8
```

---

## 8. CLI and Configuration Crates

### 8.1 CLI: clap 4

| Property | Value |
|---|---|
| **Crate** | `clap` |
| **Version** | `4.5.23` |
| **Features** | `derive` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | CLI argument parsing for `civ` binary (simulation runner, scenario tool, replay tool) |

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "civ", version, about = "CivLab Simulation Engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { #[arg(long)] scenario: PathBuf, #[arg(long, default_value_t = 8080)] port: u16 },
    Replay { #[arg(long)] replay_file: PathBuf },
    Bench { #[arg(long, default_value_t = 10_000)] entities: u32 },
}
```

### 8.2 Configuration: config crate + toml

| Crate | Version | Purpose |
|---|---|---|
| `config` | `0.14.1` | Layered configuration (file + env vars + defaults) |
| `toml` | `0.8.19` | TOML deserialization for scenario and server config files |

Server configuration is layered: `config/default.toml` → `config/{ENV}.toml` → environment variables. The `config` crate handles this layering and deserializes into strongly-typed structs via `serde`.

```toml
# config/default.toml
[server]
port = 8080
tick_rate_ms = 100
max_clients = 100

[simulation]
default_seed = 42
max_entities = 1_000_000
```

---

## 9. Python FFI Crates

### 9.1 PyO3: Rust-Python Bridge

| Property | Value |
|---|---|
| **Crate** | `pyo3` |
| **Version** | `0.23.3` |
| **Features** | `extension-module` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Expose simulation engine as a Python extension module for research scripting |

CivLab exposes a Python interface for researchers who want to drive scenarios from Python notebooks without running the full WebSocket server. The `pyo3` extension module wraps `Simulation` with Python-callable methods.

```rust
use pyo3::prelude::*;

#[pyclass]
struct PySimulation {
    inner: Simulation,
}

#[pymethods]
impl PySimulation {
    #[new]
    fn new(seed: u64) -> Self {
        PySimulation { inner: Simulation::new(seed) }
    }

    fn tick(&mut self) -> PyResult<()> {
        self.inner.tick().map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn snapshot_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let snap = self.inner.snapshot();
        let dict = PyDict::new(py);
        dict.set_item("tick", snap.tick)?;
        dict.set_item("population", snap.population)?;
        Ok(dict)
    }
}

#[pymodule]
fn civlab(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySimulation>()?;
    Ok(())
}
```

### 9.2 NumPy Integration: pyo3-numpy

| Property | Value |
|---|---|
| **Crate** | `numpy` (pyo3-numpy) |
| **Version** | `0.23.0` |
| **License** | MIT/Apache-2.0 |
| **Purpose** | Return simulation state arrays directly as NumPy arrays for pandas/matplotlib analysis |

Time-series metric data (population over time, GDP over time) is returned as NumPy arrays to avoid Python-side deserialization overhead:

```rust
use numpy::{PyArray1, IntoPyArray};

#[pymethods]
impl PySimulation {
    fn get_population_history<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<u64>> {
        let data: Vec<u64> = self.inner.metrics().population_history().to_vec();
        data.into_pyarray(py)
    }
}
```

---

## 10. Build Tooling

### 10.1 Dependency Auditing: cargo-deny

| Tool | Version | Purpose |
|---|---|---|
| `cargo-deny` | `0.16.4` | License compliance, advisory database, duplicate detection, banned crates |

Configuration in `deny.toml`:
```toml
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "CC0-1.0", "ISC"]
deny = ["GPL-3.0", "AGPL-3.0"]

[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"

[bans]
multiple-versions = "warn"
deny = [
  { name = "openssl" },  # must use rustls
]
```

CI runs `cargo deny check` on every pull request. Any HIGH or CRITICAL advisory blocks merge.

### 10.2 Security Auditing: cargo-audit

| Tool | Version | Purpose |
|---|---|---|
| `cargo-audit` | `0.21.0` | Check `Cargo.lock` against RustSec advisory database |

Runs daily in CI and on every dependency update PR. Integrates with `cargo deny` but provides standalone JSON output for security dashboards.

### 10.3 Performance Profiling: cargo-flamegraph

| Tool | Version | Purpose |
|---|---|---|
| `cargo-flamegraph` | `0.6.6` | CPU flamegraph generation for tick performance regression investigation |

Usage:
```bash
cargo flamegraph --bin civ -- run --scenario scenarios/benchmark.yaml
# Produces flamegraph.svg showing hot paths in tick loop
```

### 10.4 Coverage: cargo-llvm-cov

| Tool | Version | Purpose |
|---|---|---|
| `cargo-llvm-cov` | `0.6.16` | LLVM-based code coverage with HTML/LCOV output |

`cargo-llvm-cov` uses LLVM's source-based coverage instrumentation, which is more accurate than `cargo-tarpaulin`'s instrumentation-based approach. Phase 6 coverage targets are enforced by CI using `cargo-llvm-cov`'s `--fail-under-lines` flag.

```bash
# Generate coverage report
cargo llvm-cov --all-features --workspace --html

# Enforce minimum coverage in CI
cargo llvm-cov --all-features --workspace --fail-under-lines 80
```

---

## 11. Pinned Version Lock Table

The following is the authoritative pinned dependency block for `Cargo.toml` at the workspace root. All simulation-core crates use exact pinning (`=`). Server and tooling crates use caret ranges within a tested minor band.

```toml
[workspace.dependencies]

# --- Core Simulation Crates ---
# RNG: ChaCha20 — bitwise deterministic, cross-platform stable
rand_chacha = { version = "=0.3.3", default-features = false }
rand = { version = "=0.8.5", default-features = false, features = ["std_rng"] }

# ECS: bevy_ecs standalone — archetype storage, change detection, parallel systems
bevy_ecs = { version = "=0.15.3", default-features = false, features = ["multi_threaded"] }

# Fixed-point arithmetic — deterministic, type-safe
fixed = { version = "=1.28.0", features = ["serde"] }

# Data parallelism — work-stealing thread pool
rayon = { version = "=2.10.0" }

# Hashing — BLAKE3 for replay integrity and snapshot hashing
blake3 = { version = "=1.5.4" }

# --- Async Runtime & Server Crates ---
tokio = { version = "=1.44.0", features = ["full"] }
axum = { version = "=0.8.1", features = ["macros", "ws"] }
tokio-tungstenite = { version = "=0.24.0", features = ["native-tls"] }
tower = { version = "=0.5.2", features = ["full"] }
tower-http = { version = "=0.6.2", features = ["trace", "compression-zstd", "cors"] }
hyper = { version = "=1.6.0", features = ["full"] }

# --- Data Crates ---
sqlx = { version = "=0.8.3", features = ["postgres", "runtime-tokio", "macros", "chrono", "uuid"] }
deadpool-redis = { version = "=0.18.0", features = ["rt_tokio_1"] }

# Serialization
serde = { version = "=1.0.219", features = ["derive"] }
serde_json = { version = "=1.0.135" }
rmp-serde = { version = "=1.3.0" }

# Compression
zstd = { version = "=0.13.3" }

# --- Observability ---
tracing = { version = "=0.1.41" }
tracing-subscriber = { version = "=0.3.19", features = ["env-filter", "json"] }
opentelemetry = { version = "=0.28.0", features = ["trace"] }
opentelemetry-otlp = { version = "=0.28.0", features = ["tonic"] }
tracing-opentelemetry = { version = "=0.29.0" }
prometheus = { version = "=0.13.4" }

# --- Spatial and Math ---
glam = { version = "=0.29.2", features = ["scalar-math"] }

# --- Testing (dev-dependencies) ---
proptest = { version = "=1.6.0" }
criterion = { version = "=0.5.1", features = ["html_reports"] }
insta = { version = "=1.42.0", features = ["json", "yaml"] }

# --- CLI and Config ---
clap = { version = "=4.5.23", features = ["derive", "env"] }
config = { version = "=0.14.1", features = ["toml"] }
toml = { version = "=0.8.19" }

# --- Python FFI ---
pyo3 = { version = "=0.23.3", features = ["extension-module", "abi3-py311"] }
numpy = { version = "=0.23.0" }

# --- Utilities ---
uuid = { version = "=1.11.0", features = ["v4", "serde"] }
chrono = { version = "=0.4.39", features = ["serde"] }
thiserror = { version = "=2.0.11" }
anyhow = { version = "=1.0.95" }
bytes = { version = "=1.9.0" }
parking_lot = { version = "=0.12.3" }
dashmap = { version = "=6.1.0" }
```

### Per-Crate Feature Matrix

| Crate | Engine | Economy | Actors | Social | Metrics | Server | Python FFI |
|---|---|---|---|---|---|---|---|
| `rand_chacha` | YES | YES | YES | YES | NO | NO | NO |
| `bevy_ecs` | YES | YES | YES | YES | NO | NO | NO |
| `fixed` | YES | YES | YES | YES | YES | NO | YES |
| `rayon` | YES | YES | YES | YES | NO | NO | NO |
| `blake3` | YES | NO | NO | NO | YES | YES | NO |
| `tokio` | NO | NO | NO | NO | NO | YES | NO |
| `axum` | NO | NO | NO | NO | NO | YES | NO |
| `tokio-tungstenite` | NO | NO | NO | NO | NO | YES | NO |
| `sqlx` | NO | NO | NO | NO | YES | YES | NO |
| `serde` | YES | YES | YES | YES | YES | YES | YES |
| `rmp-serde` | NO | NO | NO | NO | NO | YES | NO |
| `zstd` | YES | NO | NO | NO | NO | YES | NO |
| `tracing` | YES | YES | YES | YES | YES | YES | NO |
| `proptest` | TEST | TEST | TEST | TEST | TEST | TEST | NO |
| `criterion` | BENCH | BENCH | BENCH | BENCH | BENCH | BENCH | NO |
| `pyo3` | NO | NO | NO | NO | NO | NO | YES |
| `numpy` | NO | NO | NO | NO | NO | NO | YES |

---

## Appendix A: Rejected Libraries

### A.1 rand_pcg — Rejected

Rejected for simulation core due to historical output variation between crate versions in high bits. Acceptable for non-deterministic uses (server-side request ID generation). See Section 2.1.

### A.2 specs — Rejected

Rejected due to inferior archetype storage performance (~2x slower on dense queries). Bitset-based component layout produces cache misses at 100k+ entity counts. See Section 2.2.

### A.3 legion — Rejected

Rejected due to project abandonment (last release 2021, no active maintenance). See Section 2.2.

### A.4 hecs — Rejected

Rejected because it provides raw ECS primitives without a scheduling system, requiring substantial custom infrastructure. See Section 2.2.

### A.5 smallvec — Considered, Not Adopted

`smallvec` provides inline storage for small collections. Considered for entity component lists. Rejected in favor of `bevy_ecs`'s built-in archetype storage which handles this optimization internally.

### A.6 actix-web — Rejected

Rejected for HTTP layer in favor of `axum`. `actix-web` uses an actor model that adds unnecessary complexity for CivLab's simple REST surface and does not compose as cleanly with `tower` middleware.

### A.7 openssl — Rejected (Banned)

`openssl` is banned via `cargo-deny`. All TLS uses `rustls` (pulled in transitively by `tokio-tungstenite` with `native-tls` feature replaced by `rustls-tls` feature in production).

---

## Appendix B: Library Update Policy

1. **Security advisories:** Update within 48 hours for CRITICAL, 7 days for HIGH.
2. **Simulation-core crates (rand_chacha, fixed, bevy_ecs):** Update requires full determinism replay test suite pass and one-week soak in development before merging to main.
3. **Server crates (tokio, axum, sqlx):** Update requires full integration test suite pass.
4. **Tooling (clap, config, tracing):** Update on regular schedule (monthly sweep).
5. **All updates:** Must pass `cargo deny check` and `cargo audit`.

---

*Document generated 2026-02-21. Review date: 2026-08-21.*
