# CIV-0500: Performance & Optimization Specification

**Spec ID:** CIV-0500
**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Architecture & Engine Team

**Related Specs:**
- CIV-0001: Core Simulation Loop (tick architecture, ECS model, determinism invariants)
- CIV-0100: Economy v1 (economy tick pipeline, conservation invariants)
- CIV-0200: Client Protocol (WebSocket, snapshot, replay)
- CIV-0107: Joule Economy System v1 (citizen-level allocator mechanics)

---

## Executive Summary

This specification defines the complete performance and optimization contract for CivLab: target SLOs for all simulation scenarios, the ECS memory layout required to meet cache efficiency targets, SIMD vectorization strategy for hot arithmetic loops, the parallelism architecture for multi-core tick execution, profiling and observability infrastructure, snapshot optimization for multi-client broadcast, database write throughput design, WebSocket fanout constraints, web client rendering budget, the full criterion benchmark suite, per-phase performance budget allocation, and the methodology governing how optimization work is prioritized and validated.

The guiding principle is: **measure first, then optimize, with all determinism invariants preserved.** No optimization may violate the determinism guarantees established in CIV-0001.

---

## 1. Performance Targets and Service Level Objectives

### 1.1 Simulation Tick SLOs

The following table defines the mandatory performance envelope for each simulation scenario. All percentile targets are measured over a continuous 1,000-tick window on reference hardware (AMD Ryzen 9 7950X, 64 GB DDR5-6000, NVMe SSD). Wall-clock values are for the complete tick computation excluding async DB write and excluding client socket I/O.

| Scenario   | Citizens    | Districts | Tick Budget | p50 Target | p99 Target | p999 Target |
|------------|-------------|-----------|-------------|-----------|-----------|------------|
| Small      | 1,000       | 20        | 100 ms      | 8 ms      | 16 ms     | 50 ms      |
| Medium     | 10,000      | 100       | 100 ms      | 30 ms     | 60 ms     | 100 ms     |
| Large      | 100,000     | 500       | 100 ms      | 80 ms     | 150 ms    | 300 ms     |
| Research   | 1,000,000   | 2,000     | unlimited   | < 5 s     | < 10 s    | < 30 s     |

**Notes on SLO interpretation:**

- The tick budget column is the interval between real-time ticks for interactive scenarios. Exceeding p99 once in 100 ticks is tolerated; exceeding it 3 times in 100 ticks triggers a performance alert.
- Research scenario has no real-time tick budget. The `< 5s` p50 target governs headless batch simulation throughput; 5 seconds per tick at 1M citizens enables ~720 simulated years per wall-clock day at 10 ticks/in-game-year.
- p999 targets for Large scenario are explicitly permitted to exceed the 100 ms tick budget; the simulation will run ahead of the clock during cheap ticks and absorb expensive ones without client stall because the engine maintains an internal tick queue.
- All targets assume the DB async write path is fully non-blocking (see Section 7).

### 1.2 WebSocket Latency SLO

| Measurement                        | Target   | Hard Limit |
|------------------------------------|----------|------------|
| Command → ack (client to server)   | < 5 ms   | < 20 ms    |
| Tick broadcast fanout (1 client)   | < 2 ms   | < 10 ms    |
| Tick broadcast fanout (100 clients)| < 5 ms   | < 25 ms    |
| Subscribe handshake                | < 10 ms  | < 50 ms    |

The command-to-ack target of \< 5 ms is measured from the moment the client sends a WebSocket frame to the moment the server sends the ack frame. This includes frame parsing, command queue insertion, and ack serialization, but excludes the tick in which the command actually executes (commands are deferred to the next tick boundary per CIV-0001 protocol design).

### 1.3 Snapshot Generation SLO

| Snapshot Type     | Citizens | Target   | Hard Limit |
|-------------------|----------|----------|------------|
| Delta snapshot    | any      | < 10 ms  | < 30 ms    |
| Full snapshot     | 1,000    | < 10 ms  | < 25 ms    |
| Full snapshot     | 10,000   | < 25 ms  | < 50 ms    |
| Full snapshot     | 100,000  | < 80 ms  | < 200 ms   |
| Full snapshot     | 1,000,000| < 500 ms | < 2,000 ms |

Delta snapshot targets are independent of citizen count because only changed component arrays are serialized (see Section 6).

### 1.4 Replay Seek SLO

| Operation                         | Target   | Hard Limit |
|-----------------------------------|----------|------------|
| Seek to tick within ring buffer   | < 10 ms  | < 50 ms    |
| Seek to arbitrary tick (no cache) | < 100 ms | < 500 ms   |
| Full replay verification (10k tk) | < 30 s   | < 120 s    |

---

## 2. ECS Memory Layout and Cache Optimization

### 2.1 Architecture Rationale

CivLab's ECS must satisfy two constraints simultaneously:

1. **Deterministic iteration order** (required by CIV-0001 invariant I5): iteration must be stable and reproducible across runs.
2. **Cache-efficient iteration** (required by this spec): per-entity data accessed in tight loops must reside in contiguous memory.

These constraints are compatible. Sorted dense arrays provide both: entities are stored in ID-sorted order (fulfilling I5) and all data for one component type resides in a flat `Vec\<T\>` (fulfilling cache locality).

### 2.2 Struct-of-Arrays Layout

The canonical ECS representation is **Struct of Arrays (SoA)**, not Array of Structs (AoS). The distinction is critical:

```
AoS (WRONG for hot loops):
  [Citizen0 { happiness: i16, health: i16, job_id: u32, ideology: [i16;8] }]
  [Citizen1 { happiness: i16, health: i16, job_id: u32, ideology: [i16;8] }]
  ...
  Memory layout: [hap0][hlt0][job0][ideo0][hap1][hlt1][job1][ideo1]...
  Cache miss pattern: reading happiness for 1000 citizens = 1000 * 20 bytes stride

SoA (CORRECT for hot loops):
  happiness: [hap0][hap1][hap2]...[hap999]
  health:    [hlt0][hlt1][hlt2]...[hlt999]
  job_id:    [job0][job1][job2]...[job999]
  ideology:  [ideo0_0..ideo0_7][ideo1_0..ideo1_7]...
  Cache miss pattern: reading happiness for 1000 citizens = sequential 2000-byte read
```

### 2.3 Hot/Cold Component Split

Components are divided into hot (accessed every tick in inner loops) and cold (accessed infrequently, on specific event paths only).

**Hot components** (stored in contiguous flat arrays, 32-byte aligned, near each other in memory):

| Component        | Type          | Bytes per entity | Notes                                 |
|-----------------|---------------|-----------------|---------------------------------------|
| `entity_id`     | `u32`         | 4               | Implicit index; no storage needed     |
| `happiness`     | `i16`         | 2               | -1000 to +1000 (scaled from -10..+10) |
| `health`        | `i16`         | 2               | 0 to 1000                             |
| `job_id`        | `u32`         | 4               | Points into job registry              |
| `ideology`      | `[i16; 8]`    | 16              | 8-dimensional political space         |
| **Subtotal**    |               | **24 bytes**    |                                       |

With 24 bytes per entity, approximately 2.67 citizens fit per 64-byte cache line. Rounding to 2 citizens per cache line is conservative; a 4-byte alignment pad brings citizens to 32 bytes and yields exactly 2 per cache line. At 1,000 citizens this is 500 cache lines = 32 KB, which fits in L1 data cache (typically 48 KB on Zen 4, 32 KB on older chips).

**Hot component storage declaration:**

```rust
// crates/engine/src/ecs/hot_components.rs

/// Hot citizen data -- all fields contiguous in memory.
/// Aligned to 32 bytes for AVX2 SIMD on ideology sub-slices.
#[repr(C, align(32))]
pub struct CitizenHot {
    pub happiness:  i16,
    pub health:     i16,
    pub job_id:     u32,
    pub ideology:   [i16; 8],
    // 4 bytes padding to 32-byte alignment
    _pad: [u8; 4],
}

pub struct CitizenHotArrays {
    /// Happiness for all citizens, indexed by dense citizen index.
    pub happiness:  Vec<i16>,
    /// Health for all citizens.
    pub health:     Vec<i16>,
    /// Job IDs for all citizens.
    pub job_id:     Vec<u32>,
    /// Ideology vectors: laid out as [c0_dim0, c0_dim1, ..., c0_dim7, c1_dim0, ...].
    /// Total: num_citizens * 8 * 2 bytes = num_citizens * 16 bytes.
    pub ideology:   Vec<i16>,
}
```

**Cold components** (stored in a secondary heap structure, accessed by entity handle):

| Component         | Type             | Bytes (approx) | Notes                          |
|------------------|------------------|----------------|--------------------------------|
| `birth_tick`     | `u64`            | 8              | Used for age calculation only  |
| `name`           | `String`         | 24 (ptr+len)   | Debug/UI only                  |
| `biography`      | `String`         | 24 (ptr+len)   | Research annotation only       |
| `education_hist` | `Vec\<u32\>`       | 24 (ptr+len)   | Lifetime education events      |
| `faction_hist`   | `Vec<(u32, u64)>`| 24 (ptr+len)   | Faction affiliations over time |

Cold data is stored in a `CitizenColdStore` that holds all cold fields indexed by dense citizen index. This store is accessed rarely (UI rendering, event annotation, replay export) and does not participate in tick hot loops.

### 2.4 Cache Line Analysis

Reference hardware L1 data cache: 48 KB (Zen 4). L2: 1 MB. L3: 32 MB.

| Scenario | Citizens | Hot data (24 B/citizen) | Cache fit |
|----------|----------|------------------------|-----------|
| Small    | 1,000    | 24 KB                  | L1 (48 KB)|
| Medium   | 10,000   | 240 KB                 | L2 (1 MB) |
| Large    | 100,000  | 2.4 MB                 | L3 (32 MB)|
| Research | 1,000,000| 24 MB                  | L3 (32 MB)|

For the Small scenario, the entire hot citizen dataset fits in L1, meaning the inner-loop tick performance is effectively latency-bound by compute, not memory. For Large and Research scenarios, the working set spans L3; prefetching via `_mm_prefetch` hints or Rust's `core::arch::x86_64::_mm_prefetch` is warranted for the ideology SIMD pass (see Section 3.2).

### 2.5 Archetype Pre-Filtering

ECS queries are pre-filtered by archetype before the hot loop begins. This eliminates per-entity type checks inside the loop.

```rust
// WRONG: per-entity type check in hot loop
for i in 0..world.entity_count {
    if world.entity_type[i] == EntityType::Citizen {  // branch in hot loop
        process_citizen(i, &world);
    }
}

// CORRECT: archetype range pre-computed once
let citizen_range = world.archetype_index.range(EntityType::Citizen);
for i in citizen_range {
    process_citizen(i, &world);  // no branch; all entities in range are citizens
}
```

Archetypes are stored as dense ranges `[start_idx, end_idx)` in a lookup table indexed by entity type. Entities are kept sorted by archetype. When entities migrate between archetypes (e.g., citizen gains `MilitaryRole` component), they are moved to a new archetype range at the end of the tick (never mid-tick, to preserve determinism).

### 2.6 Memory Alignment Requirements

All hot component arrays must be allocated with 32-byte alignment for AVX2 SIMD compatibility:

```rust
use std::alloc::{alloc, Layout};

fn alloc_aligned_i16(len: usize) -> Vec<i16> {
    let layout = Layout::from_size_align(len * 2, 32)
        .expect("alignment must be power of 2");
    // SAFETY: layout is non-zero and properly aligned
    let ptr = unsafe { alloc(layout) as *mut i16 };
    unsafe { Vec::from_raw_parts(ptr, len, len) }
}
```

In practice, use the `aligned-vec` crate or ensure `Vec` capacity is initialized via `Vec::with_capacity` followed by explicit alignment assertion in debug builds.

---

## 3. SIMD Optimization Targets

### 3.1 Philosophy and Boundaries

SIMD acceleration is permitted in the following contexts:
- **Permitted:** Arithmetic aggregation over citizen/district data (happiness diffusion, ideology dot products, price clearing computations, CO2 aggregation).
- **Forbidden:** Any SIMD path that would change the output value of the deterministic simulation state. Specifically: SIMD cannot be used for any computation whose output is stored in the canonical `State` struct, because SIMD floating-point is non-deterministic across ISA generations. SIMD is permitted only where: (a) all inputs and outputs are integer types, or (b) the operation is on f32 market quantities that are explicitly excluded from the determinism contract (see CIV-0001 §I1).

The approved SIMD interface is `std::simd` (Rust portable SIMD, stabilized as of Rust 1.82). Raw intrinsic calls via `core::arch` are permitted only in isolated `simd_*.rs` modules with full test coverage that verifies scalar and SIMD paths produce identical results.

### 3.2 Ideology Vector Dot Products

Citizen ideology is represented as `[i16; 8]`: an 8-dimensional political space (axes: liberty/authority, collective/individual, secular/religious, cosmopolitan/nationalist, agrarian/industrial, egalitarian/hierarchical, progressive/traditional, ecological/extractive). Ideology diffusion and compatibility scoring require dot products over these vectors.

**Scalar baseline:**
```rust
fn ideology_dot(a: &[i16; 8], b: &[i16; 8]) -> i32 {
    let mut acc: i32 = 0;
    for i in 0..8 {
        acc += a[i] as i32 * b[i] as i32;
    }
    acc
}
// Cost: 8 multiplies + 8 adds = 16 ops per pair
```

**AVX2 SIMD path (via std::simd):**
```rust
use std::simd::{i16x16, SimdInt};

fn ideology_dot_simd_pair(
    a0: &[i16; 8], a1: &[i16; 8],
    b0: &[i16; 8], b1: &[i16; 8],
) -> (i32, i32) {
    // Pack two citizen ideology vectors into one i16x16 register
    let va = i16x16::from_array([
        a0[0], a0[1], a0[2], a0[3], a0[4], a0[5], a0[6], a0[7],
        a1[0], a1[1], a1[2], a1[3], a1[4], a1[5], a1[6], a1[7],
    ]);
    let vb = i16x16::from_array([
        b0[0], b0[1], b0[2], b0[3], b0[4], b0[5], b0[6], b0[7],
        b1[0], b1[1], b1[2], b1[3], b1[4], b1[5], b1[6], b1[7],
    ]);
    let product = va * vb;  // element-wise i16 multiply (wraps on overflow; scale inputs)
    // Horizontal sum: first 8 lanes for citizen 0, second 8 for citizen 1
    let arr = product.to_array();
    let dot0 = arr[0..8].iter().map(|&x| x as i32).sum::<i32>();
    let dot1 = arr[8..16].iter().map(|&x| x as i32).sum::<i32>();
    (dot0, dot1)
}
// Cost: 1 VPMULLW + horizontal sums = ~3-4 ops per pair = ~4x speedup
```

For the diffusion phase operating on N citizens, we process citizens in pairs, achieving throughput of 2 dot products per SIMD instruction. Expected speedup vs. scalar: 3-5x on AVX2 hardware.

**Overflow handling:** Ideology values are normalized to `[-100, 100]` (scaled by 100 from the [-1.0, 1.0] conceptual range). Max product per element: 100 * 100 = 10,000. Max sum: 8 * 10,000 = 80,000. This fits in i32 without overflow. The i16 intermediate multiply can overflow for values > 181; we document the pre-condition that ideology values must be clamped to `[-128, 127]` before the SIMD path.

### 3.3 Market Price Clearing

The economy module (CIV-0100) performs market clearing for 9 goods per district, per tick. This is explicitly in the non-deterministic-float zone because market clearing computes equilibrium prices stored only as ephemeral signals (not stored in canonical simulation state directly; they feed into i64 cent prices after rounding).

**Price clearing loop (9 goods, per district):**

```rust
// SCALAR BASELINE
fn clear_prices_scalar(
    supply: &[f32; 9],
    demand: &[f32; 9],
    prev_price: &[f32; 9],
    elasticity: &[f32; 9],
) -> [f32; 9] {
    let mut new_price = [0.0f32; 9];
    for i in 0..9 {
        let excess_demand = demand[i] - supply[i];
        new_price[i] = prev_price[i] * (1.0 + elasticity[i] * excess_demand / supply[i]);
    }
    new_price
}

// SIMD PATH (f32x8 covers goods 0..7; good 8 handled scalar or with f32x1)
use std::simd::{f32x8, StdFloat};

fn clear_prices_simd(
    supply: &[f32; 9],
    demand: &[f32; 9],
    prev_price: &[f32; 9],
    elasticity: &[f32; 9],
) -> [f32; 9] {
    // Process goods 0..7 with SIMD
    let s   = f32x8::from_slice(&supply[0..8]);
    let d   = f32x8::from_slice(&demand[0..8]);
    let p   = f32x8::from_slice(&prev_price[0..8]);
    let e   = f32x8::from_slice(&elasticity[0..8]);
    let one = f32x8::splat(1.0);

    let excess      = d - s;
    let delta_ratio = e * excess / s;
    let new_p       = p * (one + delta_ratio);

    let mut result = [0.0f32; 9];
    new_p.copy_to_slice(&mut result[0..8]);
    // Good 8: scalar
    let excess8 = demand[8] - supply[8];
    result[8] = prev_price[8] * (1.0 + elasticity[8] * excess8 / supply[8]);
    result
}
```

After SIMD clearing, prices are converted to `i64` cents: `(price * 100.0).round() as i64`. The round-to-nearest provides a stable mapping from f32 price signal to integer canonical state.

### 3.4 Spatial Neighbor Queries

Hex-grid neighbor lookups use `glam::IVec2` arithmetic. The `glam` crate enables SSE2/AVX2 for 2D/3D integer vector arithmetic where available.

```rust
use glam::IVec2;

const HEX_OFFSETS: [IVec2; 6] = [
    IVec2::new(1, 0), IVec2::new(0, 1), IVec2::new(-1, 1),
    IVec2::new(-1, 0), IVec2::new(0, -1), IVec2::new(1, -1),
];

fn hex_neighbors(center: IVec2) -> [IVec2; 6] {
    HEX_OFFSETS.map(|offset| center + offset)
}
```

For radius-N neighbor queries (used in climate diffusion and social contagion), we precompute a ring-offset table at startup for all radii 1..=5 used by the simulation. Lookup is O(1) per radius.

### 3.5 Climate CO2 Aggregation

The climate module sums CO2 emissions over all districts each tick. For 2,000 districts (Research scenario), this is a horizontal integer sum over a `Vec\<i64\>` of length 2,000.

```rust
use std::simd::{i64x4, SimdInt};

fn aggregate_co2_simd(emissions: &[i64]) -> i64 {
    let mut acc = i64x4::splat(0);
    let chunks = emissions.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let v = i64x4::from_slice(chunk);
        acc += v;
    }
    // Reduce 4 lanes to 1
    let arr = acc.to_array();
    let simd_sum: i64 = arr.iter().sum();
    // Add remainder scalar
    let remainder_sum: i64 = remainder.iter().sum();
    simd_sum + remainder_sum
}
// Expected speedup: ~3x vs. scalar on large district counts
```

AVX-512 `i64x8` is preferred when available; detect at runtime via `is_x86_feature_detected!("avx512f")`. Fall back to `i64x4` (AVX2) and then scalar.

### 3.6 SIMD Benchmark Requirements

Every SIMD implementation must have a paired criterion benchmark that:
1. Measures throughput of the SIMD path.
2. Measures throughput of the scalar reference path.
3. Asserts correctness parity (SIMD output == scalar output for all test inputs).
4. Records the speedup ratio in the benchmark report.

Minimum acceptable speedup ratios before a SIMD path is considered worth keeping:

| SIMD Path                  | Minimum Speedup |
|---------------------------|----------------|
| Ideology dot product       | 2.5x           |
| Price clearing (9 goods)   | 1.5x           |
| CO2 aggregation            | 2.0x           |
| Hex neighbor generation    | 1.3x           |

If a SIMD path fails to meet its minimum speedup ratio on the CI benchmark machine, the implementation must revert to the scalar path.

---

## 4. Parallelism Architecture

### 4.1 Thread Pool Configuration

CivLab uses `rayon` for all data-parallel computation. The global Rayon thread pool is configured at server startup:

```rust
use rayon::ThreadPoolBuilder;

fn configure_thread_pool() {
    let num_cores = num_cpus::get_physical();
    // Reserve 1 core for the async I/O runtime (tokio).
    // Reserve 1 core for the simulation tick loop coordinator.
    let rayon_threads = (num_cores - 2).max(1);

    ThreadPoolBuilder::new()
        .num_threads(rayon_threads)
        .thread_name(|i| format!("civ-rayon-{}", i))
        .stack_size(4 * 1024 * 1024)  // 4 MB stack per rayon thread
        .build_global()
        .expect("failed to initialize rayon thread pool");
}
```

On a 16-core machine: 14 rayon threads + 1 tokio thread + 1 tick coordinator thread.

### 4.2 Phase-Level Parallelism

The tick phase schedule from CIV-0001 has been analyzed for data dependencies. Phases with no shared write dependencies can execute in parallel:

```
Tick N execution graph:

[Command Intake] ──────────────────────────────────────────────> [Policy Phase]
                                                                        │
                              ┌─────────────────────────────────────────┤
                              │                                          │
                    [Demographics Phase]                    [Climate Update Phase]
                    (reads: citizens)                       (reads: cells, emissions)
                    (writes: births/deaths buffer)          (writes: climate buffer)
                              │                                          │
                              └──────────────┬───────────────────────────┘
                                             │
                                    [State Merge Barrier]
                                             │
                    ┌────────────────────────┼──────────────────────────┐
                    │                        │                           │
           [Production Phase]      [Military Movement]        [Research Tick]
           (reads: buildings,      (reads: units)             (reads: tech tree)
            citizens, markets)     (writes: unit pos buffer)  (writes: research buf)
           (writes: output buffer) no shared state w/ prod     no shared state w/ prod
                    │                        │                           │
                    └────────────────────────┼──────────────────────────┘
                                             │
                                    [State Merge Barrier]
                                             │
                    ┌────────────────────────┼──────────────────────────┐
                    │                                                    │
           [Social Diffusion Phase]                          [Economy Clearing Phase]
           (reads: ideology arrays, adjacency)               (reads: output buffers)
           (writes: ideology delta buffer)                   (writes: price signals)
                    │                                                    │
                    └────────────────────────┬──────────────────────────┘
                                             │
                                    [State Merge Barrier]
                                             │
                                  [Event Collection Phase]
                                             │
                                  [Snapshot Generation]
                                             │
                                  [DB Async Write Spawn]
                                  [WebSocket Fanout]
```

**Parallelism rule:** A phase may be run in parallel with another phase if and only if:
1. Their read sets do not include any component arrays that the other phase writes to, AND
2. Neither phase calls the stochastic RNG (RNG calls are strictly serialized in Phase 4 of CIV-0001 tick schedule).

Parallel execution uses `rayon::scope`:

```rust
fn execute_parallel_phases_round1(state: &State, buffers: &mut TickBuffers) {
    rayon::scope(|s| {
        s.spawn(|_| demographics_phase(state, &mut buffers.demographics));
        s.spawn(|_| climate_update_phase(state, &mut buffers.climate));
        // Both phases complete before returning from scope
    });
}

fn execute_parallel_phases_round2(state: &State, buffers: &mut TickBuffers) {
    rayon::scope(|s| {
        s.spawn(|_| production_phase(state, &mut buffers.production));
        s.spawn(|_| military_movement_phase(state, &mut buffers.military));
        s.spawn(|_| research_tick_phase(state, &mut buffers.research));
    });
}

fn execute_parallel_phases_round3(state: &State, buffers: &mut TickBuffers) {
    rayon::scope(|s| {
        s.spawn(|_| social_diffusion_phase(state, &mut buffers.social));
        s.spawn(|_| economy_clearing_phase(state, &mut buffers.economy));
    });
}
```

### 4.3 Entity-Level Parallelism via Double Buffering

Within each phase, entity-level parallelism is safe when using the double-buffer pattern: the phase reads from `state_t` (the current tick's read-only snapshot) and writes to `state_t1_buffer` (the next tick's writable buffer). No entity reads from or writes to the same buffer array.

```rust
pub struct DoubleBuffer<T: Clone> {
    buffers: [T; 2],
    read_idx: usize,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn read(&self) -> &T {
        &self.buffers[self.read_idx]
    }

    pub fn write(&mut self) -> &mut T {
        &mut self.buffers[1 - self.read_idx]
    }

    pub fn swap(&mut self) {
        self.read_idx = 1 - self.read_idx;
    }
}
```

At the start of each tick: `read()` returns the current state. All phase writes go to `write()` buffer. After all phases complete and state merge barriers are passed: `swap()` makes the new state current.

**Entity-level par_iter example (happiness update):**

```rust
fn update_happiness_parallel(
    read: &CitizenHotArrays,
    write: &mut CitizenHotArrays,
    params: &HappinessParams,
) {
    // rayon::par_iter over citizen indices -- safe because:
    //   1. reads from `read` (immutable, shared reference OK)
    //   2. writes to `write` (non-overlapping slices per thread)
    write.happiness
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, hap)| {
            let health_contrib  = read.health[i] as i32 * params.health_weight;
            let job_contrib     = job_happiness(read.job_id[i], params);
            let new_val = (read.happiness[i] as i32
                + health_contrib / 1000
                + job_contrib / 1000
            ).clamp(-1000, 1000) as i16;
            *hap = new_val;
        });
}
```

### 4.4 Reduction Pattern for District Aggregates

Per-citizen outputs (production, CO2 emissions, social unrest) must be reduced to per-district aggregates. Use `rayon::fold` + `reduce` for cache-coherent parallel aggregation:

```rust
fn aggregate_district_production(
    citizen_production: &[i64],    // len = num_citizens
    citizen_district: &[u32],      // len = num_citizens: which district each citizen is in
    num_districts: usize,
) -> Vec<i64> {
    citizen_production
        .par_iter()
        .zip(citizen_district.par_iter())
        .fold(
            || vec![0i64; num_districts],
            |mut acc, (&prod, &dist)| {
                acc[dist as usize] += prod;
                acc
            },
        )
        .reduce(
            || vec![0i64; num_districts],
            |mut a, b| {
                a.iter_mut().zip(b.iter()).for_each(|(x, y)| *x += y);
                a
            },
        )
}
```

This pattern is correct only when district IDs are used as array indices (requires dense district ID packing, maintained by the entity manager). For sparse district IDs, use a `DashMap \< u32, i64>` reduction instead, which is lock-free but has higher constant overhead.

### 4.5 Determinism Preservation Under Parallelism

Parallelism must never affect simulation output. Enforcement rules:

1. **No parallel writes to shared state.** All parallel phases write to independent buffer arrays. Merge is sequential.
2. **Event ordering is sequential.** Events emitted by parallel phases are collected into thread-local buffers and merged in deterministic phase order (demographics events, then climate events, etc.) after all phases complete.
3. **RNG is never called from parallel workers.** The stochastic phase (CIV-0001 Phase 4) is always single-threaded with a seeded ChaCha20Rng.
4. **Reduction is commutative and associative.** Integer addition is both; floating-point reduction is not and is forbidden in the canonical state path.

---

## 5. Profiling Infrastructure

### 5.1 Tracing Instrumentation

Every tick phase is instrumented with `tracing` spans at `debug` level. The span hierarchy mirrors the phase dependency graph:

```rust
use tracing::{instrument, span, Level};

#[instrument(level = "debug", skip(state, buffers))]
fn execute_tick(state: &State, buffers: &mut TickBuffers, tick_num: u64) -> TickResult {
    let _guard = span!(Level::DEBUG, "tick", tick = tick_num).entered();

    {
        let _phase = span!(Level::DEBUG, "phase.command_intake").entered();
        command_intake_phase(state, buffers);
    }

    {
        let _phase = span!(Level::DEBUG, "phase.policy").entered();
        policy_phase(state, buffers);
    }

    {
        let _phase = span!(Level::DEBUG, "phase.parallel_round1").entered();
        execute_parallel_phases_round1(state, buffers);
    }
    // ... etc
}
```

Span attributes captured per phase:
- `entity_count`: number of entities processed
- `duration_us`: phase wall time in microseconds (post-span, via tracing subscriber)
- `event_count`: number of events emitted (for phases that emit events)

### 5.2 OpenTelemetry Export

Spans are exported via the OpenTelemetry SDK:

```toml
# Cargo.toml
[dependencies]
tracing-opentelemetry = "0.27"
opentelemetry = { version = "0.26", features = ["trace"] }
opentelemetry-otlp = { version = "0.26", features = ["tonic"] }
```

**Local dev:** Export to Jaeger at `http://localhost:4317` (OTLP gRPC). Start Jaeger with:
```bash
docker run -p 4317:4317 -p 16686:16686 jaegertracing/all-in-one:latest
```

**Production:** Export to OTLP collector (Grafana Alloy or OpenTelemetry Collector). Configure via `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable.

Sampling rate defaults to 1% in production (1 in 100 ticks traced) to bound telemetry overhead. Use `OTEL_TRACES_SAMPLER_ARG=1.0` to trace every tick for debugging.

### 5.3 Prometheus Metrics

The server exposes a `/metrics` endpoint (Prometheus scrape) via the `prometheus` crate:

```rust
use prometheus::{
    register_histogram_vec, register_gauge_vec,
    register_counter_vec, HistogramVec, GaugeVec, CounterVec
};

lazy_static! {
    static ref TICK_DURATION: HistogramVec = register_histogram_vec!(
        "civ_tick_duration_seconds",
        "Tick phase wall-clock duration",
        &["phase"],
        vec![0.001, 0.002, 0.005, 0.010, 0.020, 0.050, 0.100, 0.200, 0.500]
    ).unwrap();

    static ref ENTITY_COUNT: GaugeVec = register_gauge_vec!(
        "civ_entity_count",
        "Current count of live entities by type",
        &["entity_type"]
    ).unwrap();

    static ref EVENT_COUNT: CounterVec = register_counter_vec!(
        "civ_event_count_total",
        "Cumulative count of simulation events emitted",
        &["domain", "event_type"]
    ).unwrap();

    static ref SNAPSHOT_SIZE_BYTES: HistogramVec = register_histogram_vec!(
        "civ_snapshot_size_bytes",
        "Serialized snapshot size in bytes",
        &["snapshot_type"],
        vec![1000.0, 10_000.0, 100_000.0, 1_000_000.0, 10_000_000.0]
    ).unwrap();

    static ref WS_FANOUT_DURATION: HistogramVec = register_histogram_vec!(
        "civ_websocket_fanout_duration_seconds",
        "WebSocket tick broadcast latency",
        &["client_count_bucket"],
        vec![0.001, 0.002, 0.005, 0.010, 0.025, 0.050, 0.100]
    ).unwrap();
}
```

Required metric labels and semantics:

| Metric                         | Labels                    | Update Frequency |
|-------------------------------|---------------------------|-----------------|
| `civ_tick_duration_seconds`   | `phase`                   | Every tick       |
| `civ_entity_count`            | `entity_type`             | Every 10 ticks   |
| `civ_event_count_total`       | `domain`, `event_type`    | Every tick       |
| `civ_snapshot_size_bytes`     | `snapshot_type`           | Every snapshot   |
| `civ_ws_fanout_duration_s`    | `client_count_bucket`     | Every tick       |
| `civ_db_write_queue_depth`    | (none)                    | Every 1 s        |
| `civ_db_write_duration_s`     | (none)                    | Every DB write   |
| `civ_replay_seek_duration_s`  | (none)                    | Every seek       |

### 5.4 Flamegraph Workflow

Standard profiling workflow for identifying CPU hotspots:

```bash
# Install flamegraph tooling
cargo install flamegraph
# Requires: perf on Linux, or DTrace on macOS (sudo)

# Profile medium scenario for 1000 ticks
cargo flamegraph --release \
    --bin civ-server \
    --output flamegraph_medium.svg \
    -- --scenario scenarios/medium.yaml --ticks 1000 --headless

# Open in browser
open flamegraph_medium.svg

# For macOS without sudo DTrace, use cargo-instruments instead:
cargo install cargo-instruments
cargo instruments --release --template "Time Profiler" \
    --bin civ-server \
    -- --scenario scenarios/medium.yaml --ticks 1000 --headless
```

**Flamegraph interpretation rules:**
- Any single function consuming > 15% of tick CPU time must be investigated.
- Tick phases should appear as named frames (enforced by `#[instrument]` spans).
- Memory allocator overhead (`jemalloc::alloc`) appearing > 5% indicates hot-loop allocations that must be eliminated.

### 5.5 Cache Miss Analysis

```bash
# Linux: perf stat for cache analysis
perf stat -e cache-references,cache-misses,L1-dcache-load-misses \
    cargo run --release --bin civ-server \
    -- --scenario scenarios/medium.yaml --ticks 500 --headless

# Target cache miss rates:
# L1-dcache-load-misses < 2%  (critical: hot loop data fits in L1)
# LLC-load-misses       < 15% (acceptable for large scenario)
```

Target cache miss rates for each scenario:

| Scenario | L1 miss rate target | L2 miss rate target | L3 miss rate target |
|----------|--------------------|--------------------|---------------------|
| Small    | < 1%               | < 5%               | < 10%               |
| Medium   | < 3%               | < 10%              | < 20%               |
| Large    | < 8%               | < 20%              | < 40%               |
| Research | < 15%              | < 30%              | < 50%               |

Exceeding these targets by more than 2x triggers a cache layout investigation (verify SoA layout, alignment, and prefetch hints).

### 5.6 Automated Performance Regression Detection

Criterion benchmarks run in CI on every PR. The baseline is set from the `main` branch:

```yaml
# .github/workflows/benchmarks.yml
jobs:
  benchmark:
    runs-on: self-hosted-perf  # dedicated perf machine, no noisy neighbors
    steps:
      - uses: actions/checkout@v4
      - name: Restore baseline
        uses: actions/cache@v4
        with:
          path: target/criterion
          key: criterion-baseline-${{ github.base_ref }}
      - name: Run benchmarks
        run: cargo criterion --message-format json > bench_results.json
      - name: Check regression
        run: |
          python3 scripts/check_perf_regression.py \
            --results bench_results.json \
            --threshold-p99 1.10 \
            --threshold-mean 1.15 \
            --fail-on-regression
```

The regression check script compares p99 of each benchmark against the saved baseline. A regression of > 10% on p99 or > 15% on mean causes CI to fail and blocks merge. The script outputs a Markdown table of all changed benchmarks for the PR comment.

---

## 6. Snapshot Optimization

### 6.1 Delta Snapshot Strategy

A delta snapshot contains only the component arrays that changed during the current tick. Each component array is tracked by a dirty bit in a `ComponentDirtyMask`:

```rust
/// Bitset tracking which component arrays were written this tick.
/// One bit per component array type; 128 bits covers 128 array types.
pub struct ComponentDirtyMask {
    bits: u128,
}

impl ComponentDirtyMask {
    pub fn mark_dirty(&mut self, component_id: u8) {
        self.bits |= 1u128 << component_id;
    }

    pub fn is_dirty(&self, component_id: u8) -> bool {
        (self.bits >> component_id) & 1 == 1
    }

    pub fn dirty_components(&self) -> impl Iterator<Item = u8> {
        (0u8..128).filter(move |&i| self.is_dirty(i))
    }
}
```

At tick end, the snapshot serializer iterates only dirty component arrays:

```rust
fn serialize_delta_snapshot(
    state: &State,
    mask: &ComponentDirtyMask,
) -> DeltaSnapshot {
    let mut arrays = Vec::new();
    for component_id in mask.dirty_components() {
        let array_bytes = serialize_component_array(state, component_id);
        arrays.push(DeltaArray { component_id, data: array_bytes });
    }
    DeltaSnapshot {
        tick: state.tick,
        state_hash: state.compute_hash(),
        arrays,
    }
}
```

Typical tick modifies: happiness, health, production buffers, event log. Typical dirty set is 5-10 out of 50+ component arrays. This reduces delta snapshot size by ~80-90% vs. a full snapshot for normal ticks.

### 6.2 Full Snapshot Compression

Full snapshots are compressed with `zstd` at level 3 (fast compression, good ratio):

```rust
use zstd::stream::encode_all;

fn compress_snapshot(snapshot_bytes: &[u8]) -> Vec<u8> {
    encode_all(snapshot_bytes, 3)
        .expect("zstd compression must not fail on valid input")
}
```

**Target compressed sizes:**

| Citizens  | Uncompressed | zstd level 3 | Ratio |
|-----------|-------------|-------------|-------|
| 1,000     | ~2 MB       | ~150 KB     | 13:1  |
| 10,000    | ~20 MB      | ~1.2 MB     | 17:1  |
| 100,000   | ~200 MB     | ~10 MB      | 20:1  |
| 1,000,000 | ~2 GB       | ~90 MB      | 22:1  |

Simulation state is highly compressible because: (a) ideology vectors have low entropy (most citizens cluster near 0), (b) job_id has high repetition (few distinct jobs), (c) health and happiness values have temporal coherence (change slowly, delta-encode well within zstd's internal LZ77 window).

### 6.3 Snapshot Scheduling

```
Tick N:    Generate delta snapshot → broadcast to subscribed clients
Tick N:    Cache delta snapshot in ring buffer (capacity: 100)

Every 1000 ticks:
           Generate full snapshot
           Write full snapshot to disk at snapshots/tick_{N}.snap.zst
           This full snapshot serves as a replay anchor point.
```

Ring buffer implementation for delta snapshot cache:

```rust
pub struct DeltaSnapshotRingBuffer {
    buffer: Vec<Option<DeltaSnapshot>>,
    head: usize,
    capacity: usize,
    oldest_tick: u64,
}

impl DeltaSnapshotRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            head: 0,
            capacity,
            oldest_tick: 0,
        }
    }

    pub fn push(&mut self, snapshot: DeltaSnapshot) {
        let tick = snapshot.tick;
        self.buffer[self.head] = Some(snapshot);
        self.head = (self.head + 1) % self.capacity;
        if self.buffer[self.head].is_some() {
            // Ring buffer is full; oldest_tick advances
            if let Some(ref s) = self.buffer[self.head] {
                self.oldest_tick = s.tick + 1;
            }
        }
    }

    pub fn get(&self, tick: u64) -> Option<&DeltaSnapshot> {
        if tick < self.oldest_tick { return None; }
        let offset = (tick - self.oldest_tick) as usize;
        if offset >= self.capacity { return None; }
        let idx = (self.head + self.capacity - self.capacity + offset) % self.capacity;
        self.buffer[idx].as_ref()
    }
}
```

### 6.4 Binary Serialization Format

For clients that support binary protocol (game engines, high-frequency research scripts), snapshots are serialized with MessagePack via `rmp-serde`:

```rust
use rmp_serde::{encode, decode};

fn serialize_snapshot_msgpack(snapshot: &FullSnapshot) -> Vec<u8> {
    encode::to_vec(snapshot).expect("msgpack serialization must not fail")
}

fn deserialize_snapshot_msgpack(bytes: &[u8]) -> FullSnapshot {
    decode::from_slice(bytes).expect("msgpack deserialization must not fail on valid input")
}
```

**Serialization format comparison:**

| Format      | Size (10k citizens) | Serialize time | Deserialize time |
|-------------|-------------------|----------------|-----------------|
| JSON        | 18 MB             | 250 ms         | 180 ms          |
| MessagePack | 4 MB              | 40 ms          | 30 ms           |
| Bincode     | 3.2 MB            | 25 ms          | 20 ms           |
| JSON+zstd3  | 1.1 MB            | 280 ms         | 210 ms          |
| MsgPack+zstd| 800 KB            | 55 ms          | 45 ms           |

Default for WebSocket JSON-RPC clients: JSON. Default for binary-frame clients (Bevy, Unreal): MessagePack + zstd level 1 (fastest decompression).

### 6.5 Replay Seek Optimization

Replay seek to an arbitrary tick requires:
1. Find nearest full snapshot anchor at or before target tick.
2. Replay delta snapshots from anchor to target tick.

```rust
pub struct ReplaySeeker {
    anchors: BTreeMap<u64, FullSnapshot>,  // tick -> full snapshot
    ring_buffer: DeltaSnapshotRingBuffer,
}

impl ReplaySeeker {
    pub fn seek_to_tick(&self, target_tick: u64) -> Result<State> {
        // Find nearest anchor
        let (&anchor_tick, anchor_snapshot) = self.anchors
            .range(..=target_tick)
            .next_back()
            .ok_or(SeekError::NoAnchorFound)?;

        let mut state = anchor_snapshot.to_state();

        // Apply deltas from anchor to target
        for tick in (anchor_tick + 1)..=target_tick {
            let delta = self.ring_buffer.get(tick)
                .ok_or(SeekError::DeltaMissing { tick })?;
            state.apply_delta(delta)?;
        }

        Ok(state)
    }
}
```

With full snapshots every 1,000 ticks and 100-tick ring buffer: seeking within the last 100 ticks is O(100) delta applies (< 10 ms target). Seeking to older ticks requires loading a full snapshot from disk plus up to 999 delta replays (< 100 ms target).

---

## 7. Database Performance

### 7.1 Write Path: Batch Event Insertion

All events generated during a tick are collected and inserted in a single batch statement. Never insert events one-by-one:

```rust
// WRONG: N individual inserts (N round-trips to DB)
for event in &tick_events {
    sqlx::query!("INSERT INTO event_log (tick, domain, type, data) VALUES (?, ?, ?, ?)",
        event.tick, event.domain, event.event_type, event.data)
        .execute(&pool).await?;
}

// CORRECT: One batch insert for all tick events
async fn batch_insert_events(
    pool: &sqlx::PgPool,
    events: &[TickEvent],
) -> Result<()> {
    if events.is_empty() { return Ok(()); }

    // Build VALUES clause dynamically
    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO event_log (tick, domain, event_type, data, state_hash) "
    );
    query_builder.push_values(events, |mut b, event| {
        b.push_bind(event.tick as i64)
         .push_bind(&event.domain)
         .push_bind(&event.event_type)
         .push_bind(&event.data)
         .push_bind(&event.state_hash);
    });
    query_builder.build().execute(pool).await?;
    Ok(())
}
```

For 200 events/tick, a single batch insert takes ~2-5 ms vs. ~400 ms for 200 individual inserts (1 ms round-trip × 200).

### 7.2 Async Write: Never Block the Tick

Database writes are non-blocking with respect to the tick loop:

```rust
fn finish_tick(state: Arc<State>, events: Vec<TickEvent>, pool: Arc<sqlx::PgPool>) {
    // Tick is done. Spawn DB write as independent tokio task.
    tokio::spawn(async move {
        if let Err(e) = batch_insert_events(&pool, &events).await {
            // Log error but do not panic or stall the tick loop.
            // DB write failure is observable via Prometheus alerts.
            tracing::error!(
                tick = state.tick,
                error = %e,
                "Failed to persist tick events to database"
            );
        }
    });
    // Tick loop continues immediately; DB write happens in background.
}
```

The DB write task is spawned on the tokio runtime, not the rayon thread pool. The tokio runtime's I/O reactor handles the socket without blocking rayon workers.

### 7.3 CQRS Read Projections

Read queries (client API requests for metrics, leaderboards, history) must not touch the `event_log` table directly in hot paths. Instead, pre-aggregated projections are maintained by a background worker:

```sql
-- Pre-aggregated projection table (updated by background worker)
CREATE TABLE nation_metrics (
    tick        BIGINT NOT NULL,
    nation_id   BIGINT NOT NULL,
    population  BIGINT,
    gdp         BIGINT,       -- i64 cents
    avg_happiness INT,
    gini_x1000  INT,          -- Gini * 1000 as integer
    PRIMARY KEY (tick, nation_id)
);

-- Index for time-range queries (client API: "give me last 100 ticks of GDP")
CREATE INDEX idx_nation_metrics_nation_tick ON nation_metrics(nation_id, tick DESC);
```

The background projection worker runs every N ticks (configurable, default 10) and aggregates from the event log into the projection table. Client read queries hit only the projection table, which is small and fully indexed.

### 7.4 Connection Pool Configuration

```rust
let pool = sqlx::PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(std::time::Duration::from_secs(5))
    .idle_timeout(std::time::Duration::from_secs(300))
    .max_lifetime(std::time::Duration::from_secs(3600))
    .connect(&database_url)
    .await
    .expect("database connection pool must initialize");
```

The pool is wrapped in `Arc \< sqlx::PgPool>` and shared across all tokio tasks. Do not create per-request pools; pool overhead (connection establishment) dominates query time at this scale.

### 7.5 Vacuum Strategy

The `event_log` table receives high write volume (100-200 rows per tick = 6,000-12,000 rows per second at 10 ticks/second interactive rate). PostgreSQL autovacuum must be tuned:

```sql
ALTER TABLE event_log SET (
    autovacuum_vacuum_scale_factor = 0.01,   -- vacuum after 1% dead tuples (default: 20%)
    autovacuum_analyze_scale_factor = 0.005, -- analyze after 0.5% changes
    autovacuum_vacuum_cost_delay = 2         -- milliseconds; lower = faster vacuum
);

-- After bulk scenario imports, run manual vacuum:
VACUUM ANALYZE event_log;
```

For research scenarios running millions of ticks in batch mode, partition the `event_log` table by tick range to keep table size manageable and enable bulk partition drops after analysis:

```sql
CREATE TABLE event_log_p0000001 PARTITION OF event_log
    FOR VALUES FROM (0) TO (1000000);
CREATE TABLE event_log_p0001000 PARTITION OF event_log
    FOR VALUES FROM (1000000) TO (2000000);
-- etc.
```

---

## 8. WebSocket Throughput

### 8.1 Broadcast Channel Architecture

Tick snapshots are broadcast to all subscribed clients via a single `tokio::broadcast` channel. This avoids serializing the snapshot once per client:

```rust
use tokio::sync::broadcast;

pub struct SimServer {
    /// Tick broadcast: sender owned by tick loop, receivers cloned per client.
    tick_tx: broadcast::Sender<Arc<TickBroadcast>>,
    pool:    Arc<sqlx::PgPool>,
}

#[derive(Clone)]
pub struct TickBroadcast {
    pub tick:          u64,
    pub delta_snapshot: Arc<DeltaSnapshot>,
    pub full_snapshot:  Option<Arc<FullSnapshot>>,  // Some every 1000 ticks
    pub events:        Arc<Vec<TickEvent>>,
}

impl SimServer {
    pub fn new() -> Self {
        let (tick_tx, _) = broadcast::channel(1000);
        Self { tick_tx, pool: /* ... */ }
    }

    /// Called by tick loop after each tick completes.
    pub fn broadcast_tick(&self, broadcast: TickBroadcast) {
        // send() returns error if no active receivers; that is not an error.
        let _ = self.tick_tx.send(Arc::new(broadcast));
    }

    /// Called when a new client connects.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<TickBroadcast>> {
        self.tick_tx.subscribe()
    }
}
```

The broadcast channel capacity of 1,000 means a lagging client can be up to 1,000 ticks behind before its receiver is dropped by the channel.

### 8.2 Per-Client Subscription Filtering

Each client's WebSocket handler applies its subscription filter to the broadcast before sending:

```rust
async fn handle_client_ws(
    ws: WebSocket,
    mut rx: broadcast::Receiver<Arc<TickBroadcast>>,
    filter: SubscriptionFilter,
) {
    loop {
        match rx.recv().await {
            Ok(broadcast) => {
                // Apply filter: compute which fields this client wants
                let filtered = filter.apply(&broadcast);
                let frame = serialize_for_client(&filtered, &filter.format);
                if ws.send(frame).await.is_err() {
                    break;  // Client disconnected; exit handler
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // Client is too slow; it missed n ticks.
                if n > 100 {
                    let _ = ws.close_with_message("CLIENT_LAG_EXCEEDED").await;
                    break;
                }
                // Minor lag: log and continue
                tracing::warn!(client_lag = n, "client lagged behind tick broadcast");
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;  // Server shutdown
            }
        }
    }
}
```

The `SubscriptionFilter` is computed once at subscribe time and stored per client. Applying a filter to a broadcast is a read-only operation on the `Arc\<TickBroadcast\>`, enabling zero-copy filtering across all clients sharing the same broadcast.

### 8.3 Backpressure and Lag Eviction

Slow clients that cannot consume ticks fast enough will lag behind the broadcast ring buffer. Policy:

| Lag (ticks behind) | Action                                             |
|---------------------|---------------------------------------------------|
| 1-10               | Normal; log at trace level                         |
| 10-50              | Log at debug level; increment lag counter metric   |
| 50-100             | Log at warn level; send `sim.lag_warning` to client|
| > 100              | Disconnect with error `CLIENT_LAG_EXCEEDED`        |

Clients that fall more than 100 ticks behind are actively harming server memory (broadcast channel must hold 100+ messages). Disconnecting them is correct behavior. The client may reconnect and request a full snapshot to re-synchronize.

### 8.4 Binary Frame Protocol

For high-performance clients (Bevy, Unreal, research scripts), the binary frame protocol reduces per-tick overhead by 3-5x vs. JSON-RPC:

```
Binary frame layout:
  [1 byte:  protocol version = 0x01]
  [1 byte:  frame type: 0x01=delta, 0x02=full, 0x03=events_only]
  [2 bytes: flags: bit 0 = compressed, bit 1 = has_events]
  [8 bytes: tick number, big-endian u64]
  [4 bytes: payload_size, big-endian u32]
  [N bytes: payload (MessagePack encoded, optionally zstd compressed)]
```

**Compression:** For binary frames, zstd level 1 (fastest) is used. Level 1 achieves ~50% size reduction with minimal CPU overhead (~0.5 ms for a 1 MB frame).

### 8.5 WebSocket Extension: permessage-deflate

For JSON-RPC clients (browsers), enable `permessage-deflate` WebSocket extension:

```rust
// tokio-tungstenite server configuration
use tokio_tungstenite::tungstenite::extensions::deflate::{
    DeflateConfig, DeflateConfigBuilder
};

let deflate_config = DeflateConfigBuilder::new()
    .compression_level(flate2::Compression::fast())  // Level 1
    .server_no_context_takeover(false)  // Share context across frames (better ratio)
    .build();
```

With `permessage-deflate`, tick frames for 1k-citizen scenarios compress from ~50 KB to ~5-8 KB (10:1 ratio), reducing bandwidth from ~500 KB/s to ~50-80 KB/s per client at 10 ticks/second.

---

## 9. Map Rendering Performance (Web Client)

### 9.1 Rendering Architecture

The web client renders the simulation map using Pixi.js v8 with WebGPU backend (fallback to WebGL2). The rendering target is 60 fps with a 16 ms per-frame budget.

**Frame budget breakdown:**

| Task                           | Budget | Notes                               |
|-------------------------------|--------|-------------------------------------|
| JS logic (camera, UI events)  | 4 ms   | Input handling, state updates        |
| WebSocket parse + apply       | 2 ms   | Delta snapshot apply to render state |
| Render call (Pixi.js)         | 10 ms  | GPU command encoding + submit        |
| **Total**                     | 16 ms  | 60 fps at 16.67 ms frame budget      |

### 9.2 Pixi.js Optimization Patterns

**Unit sprites:** Use `PIXI.ParticleContainer` for all unit/citizen sprites. `ParticleContainer` uses GPU instancing, enabling up to 100,000 sprites at 60 fps on mid-range GPU.

```typescript
const unitLayer = new PIXI.ParticleContainer(100_000, {
    position: true,
    rotation: false,
    scale: false,
    uvs: true,
    tint: true,
    alpha: false,
});
```

**Terrain tilemap:** Use `PIXI.TilingSprite` or a custom tilemap shader for the terrain base layer. One draw call for the entire terrain layer at any viewport size.

**Culling:** Only render tiles within the camera viewport plus a 2-tile margin:

```typescript
function updateVisibleTiles(
    camera: Camera,
    tileSize: number,
    mapWidth: number,
    mapHeight: number,
): TileRange {
    const margin = 2;
    return {
        xMin: Math.max(0, Math.floor(camera.x / tileSize) - margin),
        xMax: Math.min(mapWidth, Math.ceil((camera.x + camera.screenW) / tileSize) + margin),
        yMin: Math.max(0, Math.floor(camera.y / tileSize) - margin),
        yMax: Math.min(mapHeight, Math.ceil((camera.y + camera.screenH) / tileSize) + margin),
    };
}
```

At 1080p with 32×32 pixel tiles, the visible range is approximately 60×34 = 2,040 tiles. With a 2-tile margin: ~2,400 tiles rendered vs. potentially millions on a large map.

### 9.3 Level of Detail (LOD)

Two LOD levels based on camera zoom factor:

| Zoom level          | Rendering mode                                    | Draw calls |
|--------------------|----------------------------------------------------|-----------|
| < 1.0 (zoomed out) | District polygon only (no individual tile sprites) | O(districts)|
| >= 1.0 (zoomed in) | Individual tile sprites + unit sprites             | O(tiles)   |

LOD transition is triggered by the camera zoom factor changing past the 1.0 threshold. At zoom \< 1.0, the map renders district-level data (population color, resource heatmap, ideology heatmap) as colored polygons. This path is extremely fast (< 2 ms render time for 500 districts).

### 9.4 Web Worker Offloading

Snapshot parsing and application to the render state must run on a Web Worker (off the main thread) to avoid blocking the render loop:

```typescript
// main.ts
const snapshotWorker = new Worker(new URL('./snapshotWorker.ts', import.meta.url));

websocket.addEventListener('message', (event) => {
    // Transfer binary ArrayBuffer to worker without copying
    snapshotWorker.postMessage({ type: 'delta', data: event.data }, [event.data]);
});

snapshotWorker.onmessage = (event) => {
    // Worker sends back the parsed render-ready state update
    const renderUpdate: RenderUpdate = event.data;
    renderState.apply(renderUpdate);  // Fast; already parsed
};

// snapshotWorker.ts
self.onmessage = (event) => {
    const { type, data } = event.data;
    if (type === 'delta') {
        const delta = decodeDeltaSnapshot(data);
        const renderUpdate = buildRenderUpdate(delta);
        self.postMessage(renderUpdate);
    }
};
```

The worker receives the raw binary frame, decodes MessagePack, and builds a render-ready state update object. The main thread receives only the lightweight render update via `postMessage`, with zero blocking on deserialization.

### 9.5 Client Performance Monitoring

The web client exposes performance metrics to the browser DevTools Performance panel via `performance.mark` and `performance.measure`:

```typescript
function renderFrame(timestamp: number): void {
    performance.mark('frame-start');

    performance.mark('ws-parse-start');
    applyPendingUpdates();
    performance.mark('ws-parse-end');
    performance.measure('ws-parse', 'ws-parse-start', 'ws-parse-end');

    performance.mark('render-start');
    app.render();
    performance.mark('render-end');
    performance.measure('render', 'render-start', 'render-end');

    performance.mark('frame-end');
    performance.measure('frame', 'frame-start', 'frame-end');

    requestAnimationFrame(renderFrame);
}
```

---

## 10. Benchmark Suite

### 10.1 Criterion Benchmark Organization

All benchmarks live in `benches/` directories within each crate. Criterion harness is configured in `Cargo.toml`:

```toml
[[bench]]
name = "engine_benchmarks"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### 10.2 civ-engine Benchmarks

```rust
// crates/engine/benches/engine_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_tick_1k_citizens(c: &mut Criterion) {
    let state = create_test_state(1_000, 20);
    let control = create_test_control();
    let seed = 42u64;

    c.bench_function("tick_1k_citizens", |b| {
        b.iter(|| {
            let (snapshot, _next_state) = tick(
                black_box(&state),
                black_box(&control),
                black_box(seed),
            );
            black_box(snapshot)
        })
    });
}

fn bench_tick_10k_citizens(c: &mut Criterion) {
    let state = create_test_state(10_000, 100);
    let control = create_test_control();
    let seed = 42u64;

    c.bench_function("tick_10k_citizens", |b| {
        b.iter(|| {
            let (snapshot, _next_state) = tick(
                black_box(&state),
                black_box(&control),
                black_box(seed),
            );
            black_box(snapshot)
        })
    });
}

fn bench_determinism_hash(c: &mut Criterion) {
    let state = create_test_state(10_000, 100);

    c.bench_function("determinism_hash_10k", |b| {
        b.iter(|| {
            let hash = state.compute_blake3_hash();
            black_box(hash)
        })
    });
}

fn bench_tick_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_scaling");
    for &citizens in &[100, 1_000, 5_000, 10_000, 50_000, 100_000] {
        let districts = (citizens / 100).max(1);
        let state = create_test_state(citizens, districts);
        let control = create_test_control();
        group.bench_with_input(
            BenchmarkId::from_parameter(citizens),
            &citizens,
            |b, _| b.iter(|| tick(black_box(&state), black_box(&control), 42)),
        );
    }
    group.finish();
}

criterion_group!(
    engine_benches,
    bench_tick_1k_citizens,
    bench_tick_10k_citizens,
    bench_determinism_hash,
    bench_tick_scaling,
);
criterion_main!(engine_benches);
```

### 10.3 civ-economy Benchmarks

```rust
// crates/economy/benches/economy_benchmarks.rs

fn bench_market_clearing_9goods(c: &mut Criterion) {
    let supply    = [1000.0f32; 9];
    let demand    = [950.0f32, 1050.0f32, 900.0f32, 1100.0f32,
                     980.0f32, 1020.0f32, 1000.0f32, 1050.0f32, 950.0f32];
    let prev_price = [100.0f32; 9];
    let elasticity = [0.1f32; 9];

    let mut group = c.benchmark_group("market_clearing");

    group.bench_function("scalar", |b| {
        b.iter(|| clear_prices_scalar(
            black_box(&supply),
            black_box(&demand),
            black_box(&prev_price),
            black_box(&elasticity),
        ))
    });

    group.bench_function("simd", |b| {
        b.iter(|| clear_prices_simd(
            black_box(&supply),
            black_box(&demand),
            black_box(&prev_price),
            black_box(&elasticity),
        ))
    });

    group.finish();
}

fn bench_joule_allocation_10k(c: &mut Criterion) {
    let actors: Vec<JouleActor> = (0..10_000)
        .map(|i| JouleActor { id: i, demand: 100 + i % 200 })
        .collect();
    let available_joules = 800_000i64;

    c.bench_function("joule_allocation_10k", |b| {
        b.iter(|| {
            let allocations = allocate_joules(
                black_box(available_joules),
                black_box(&actors),
            );
            black_box(allocations)
        })
    });
}

criterion_group!(
    economy_benches,
    bench_market_clearing_9goods,
    bench_joule_allocation_10k,
);
criterion_main!(economy_benches);
```

### 10.4 civ-actors Benchmarks

```rust
// crates/actors/benches/actors_benchmarks.rs

fn bench_citizen_tick_1k(c: &mut Criterion) {
    let citizens = create_test_citizens(1_000);
    let params = CitizenTickParams::default();

    c.bench_function("citizen_tick_1k", |b| {
        b.iter(|| {
            let mut output = CitizenTickOutput::new(1_000);
            citizen_tick_batch(
                black_box(&citizens),
                black_box(&params),
                black_box(&mut output),
            );
            black_box(output)
        })
    });
}

fn bench_demography_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("demography");

    for &count in &[1_000, 10_000, 100_000] {
        let citizens = create_test_citizens(count);
        let rng_seed = 42u64;

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, _| b.iter(|| {
                demography_update(
                    black_box(&citizens),
                    black_box(rng_seed),
                )
            }),
        );
    }
    group.finish();
}

criterion_group!(actors_benches, bench_citizen_tick_1k, bench_demography_update);
criterion_main!(actors_benches);
```

### 10.5 civ-social Benchmarks

```rust
// crates/social/benches/social_benchmarks.rs

fn bench_ideology_diffusion_1k(c: &mut Criterion) {
    let citizens = create_test_citizens_with_ideology(1_000);
    let adjacency = build_test_adjacency(1_000);
    let params = DiffusionParams { rate: 50, influence_threshold: 100 };

    let mut group = c.benchmark_group("ideology_diffusion");

    group.bench_function("scalar_1k", |b| {
        b.iter(|| ideology_diffusion_scalar(
            black_box(&citizens),
            black_box(&adjacency),
            black_box(&params),
        ))
    });

    group.bench_function("simd_1k", |b| {
        b.iter(|| ideology_diffusion_simd(
            black_box(&citizens),
            black_box(&adjacency),
            black_box(&params),
        ))
    });

    group.finish();
}

fn bench_sir_step(c: &mut Criterion) {
    let population = create_test_population(10_000);
    let disease = DiseaseParams {
        beta:  0.3,
        gamma: 0.1,
        seed:  42,
    };

    c.bench_function("sir_step_10k", |b| {
        b.iter(|| sir_step(
            black_box(&population),
            black_box(&disease),
        ))
    });
}

criterion_group!(social_benches, bench_ideology_diffusion_1k, bench_sir_step);
criterion_main!(social_benches);
```

### 10.6 civ-server Benchmarks

```rust
// crates/server/benches/server_benchmarks.rs

fn bench_snapshot_serialize_full(c: &mut Criterion) {
    let state = create_test_state(10_000, 100);
    let mut group = c.benchmark_group("snapshot_serialize_full");

    group.bench_function("json", |b| {
        b.iter(|| serialize_full_snapshot_json(black_box(&state)))
    });

    group.bench_function("msgpack", |b| {
        b.iter(|| serialize_full_snapshot_msgpack(black_box(&state)))
    });

    group.bench_function("msgpack_zstd1", |b| {
        b.iter(|| {
            let bytes = serialize_full_snapshot_msgpack(black_box(&state));
            compress_snapshot_zstd1(&bytes)
        })
    });

    group.finish();
}

fn bench_snapshot_serialize_delta(c: &mut Criterion) {
    let state = create_test_state(10_000, 100);
    // Simulate a typical tick: 8 out of 50 component arrays are dirty
    let dirty_mask = typical_delta_dirty_mask();

    c.bench_function("snapshot_serialize_delta", |b| {
        b.iter(|| serialize_delta_snapshot(
            black_box(&state),
            black_box(&dirty_mask),
        ))
    });
}

fn bench_websocket_fanout_100clients(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let broadcast = create_test_broadcast(10_000);

    c.bench_function("ws_fanout_100clients", |b| {
        b.to_async(&rt).iter(|| async {
            let server = SimServer::new_test();
            let _receivers: Vec<_> = (0..100)
                .map(|_| server.subscribe())
                .collect();
            server.broadcast_tick(broadcast.clone());
            // Wait for all receivers to receive
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
    });
}

criterion_group!(
    server_benches,
    bench_snapshot_serialize_full,
    bench_snapshot_serialize_delta,
    bench_websocket_fanout_100clients,
);
criterion_main!(server_benches);
```

### 10.7 Property-Based Performance Tests

`proptest` is used to verify O(N) and O(N log N) scaling bounds:

```rust
use proptest::prelude::*;

proptest! {
    // Tick time must be O(N) or O(N log N) in citizen count.
    // Test: doubling citizens cannot increase tick time by more than 3x.
    #[test]
    fn prop_tick_time_bounded_by_citizen_count(
        base_citizens in 100usize..=1_000,
    ) {
        let state_n  = create_test_state(base_citizens, 10);
        let state_2n = create_test_state(base_citizens * 2, 10);
        let control  = create_test_control();

        let t_n  = time_tick(&state_n, &control, 42);
        let t_2n = time_tick(&state_2n, &control, 42);

        // Doubling N should at most triple the time (O(N log N) bound)
        prop_assert!(
            t_2n < t_n * 3,
            "Tick time at 2N ({:?}) exceeds 3x time at N ({:?})",
            t_2n, t_n
        );
    }
}
```

### 10.8 CI Benchmark Workflow

```bash
# Save baseline on main branch (run in CI after merging to main)
cargo criterion --save-baseline main

# Compare on PR (run in CI for all PRs)
cargo criterion --baseline main --message-format json \
    | python3 scripts/check_perf_regression.py

# Local comparison
cargo criterion --baseline main

# Generate HTML benchmark report
cargo criterion --bench engine_benchmarks
open target/criterion/report/index.html
```

---

## 11. Performance Budget Allocation

### 11.1 Per-Phase Budget (1,000-Citizen Tick, 100 ms Budget)

The following table defines the performance budget per tick phase for the Small scenario (1,000 citizens, 20 districts). Budget column is the maximum allowed; Target column is the design goal under normal conditions. "Parallelizable" indicates whether entity-level rayon parallelism applies within the phase.

| Phase                   | Budget | Target | Parallelizable     | Notes                                   |
|------------------------|--------|--------|--------------------|-----------------------------------------|
| Command Intake          | 1 ms   | 0.1 ms | No                 | Sequential queue drain                  |
| Policy Phase            | 5 ms   | 2 ms   | Partial            | Policy evaluation, control gen          |
| Demographics            | 5 ms   | 2 ms   | Yes (rayon)        | Births, deaths, aging                   |
| Climate Update          | 5 ms   | 2 ms   | Yes (rayon)        | CO2, temperature, rainfall              |
| Production/Economy      | 20 ms  | 10 ms  | Yes (rayon)        | Output calc, market clearing            |
| Military Movement       | 10 ms  | 4 ms   | Partial            | Unit movement, pathfinding              |
| Research Tick           | 5 ms   | 2 ms   | Yes (rayon)        | Tech tree advancement                   |
| Social/Ideology         | 15 ms  | 8 ms   | Yes (rayon)        | Diffusion, SIR epidemic, grievance      |
| Stochastic Events       | 5 ms   | 3 ms   | No                 | RNG calls, event generation             |
| Metrics Computation     | 3 ms   | 1 ms   | Yes (rayon)        | Aggregations, Gini, waste               |
| Event Collection        | 2 ms   | 1 ms   | No                 | Sequential event log flush              |
| Snapshot Generation     | 10 ms  | 5 ms   | Partial            | Dirty mask scan + serialize             |
| DB Async Write Spawn    | 0.5 ms | 0.1 ms | N/A (async spawn)  | Non-blocking; DB write in background    |
| WebSocket Fanout        | 5 ms   | 3 ms   | Yes (broadcast)    | Serialize + send per subscribed client  |
| **Total Sequential**    | 91.5 ms| 43.1 ms| —                 | Sum of all phases                       |
| **Total with Parallelism** | ~55 ms | ~28 ms| —               | Parallel rounds reduce wall time        |

With three rounds of phase parallelism (demographics||climate, production||military||research, social||economy), the effective wall time for the Small scenario is approximately:

```
Round 1 (parallel): max(demographics, climate) = max(2ms, 2ms) = 2ms
Round 2 (parallel): max(production, military, research) = max(10ms, 4ms, 2ms) = 10ms
Round 3 (parallel): max(social, economy) = max(8ms, 10ms) = 10ms
Sequential sum: command(0.1) + policy(2) + stochastic(3) + metrics(1) + events(1) + snapshot(5) + ws(3) = 15.1ms
Total wall time: 2 + 10 + 10 + 15.1 = 37.1ms
```

This is well within the 100 ms tick budget for the Small scenario and achieves the p50 target of 8 ms for a single phase (the production/economy phase).

### 11.2 Per-Phase Budget (10,000-Citizen Tick, 100 ms Budget)

| Phase                   | Budget | Target | Notes                                       |
|------------------------|--------|--------|---------------------------------------------|
| Command Intake          | 1 ms   | 0.2 ms |                                             |
| Policy Phase            | 8 ms   | 4 ms   |                                             |
| Demographics (parallel) | 10 ms  | 5 ms   | 10x citizens vs. Small                     |
| Climate Update (par.)   | 5 ms   | 3 ms   | District-level: 100 districts               |
| Production (parallel)   | 30 ms  | 15 ms  | Dominant phase at 10k scale                 |
| Military (parallel)     | 15 ms  | 7 ms   |                                             |
| Research (parallel)     | 5 ms   | 2 ms   |                                             |
| Social/Ideology (par.)  | 20 ms  | 10 ms  |                                             |
| Stochastic Events       | 8 ms   | 5 ms   |                                             |
| Metrics                 | 5 ms   | 2 ms   |                                             |
| Event Collection        | 3 ms   | 1.5 ms |                                             |
| Snapshot Generation     | 20 ms  | 10 ms  | Larger state to serialize                   |
| DB Async Write Spawn    | 0.5 ms | 0.1 ms |                                             |
| WebSocket Fanout        | 8 ms   | 5 ms   |                                             |
| **Total wall (par.)**   | ~75 ms | ~40 ms |                                             |

### 11.3 Scaling Analysis

Tick time scaling with citizen count should be O(N) for all phases that are data-parallel over citizens, and O(D log D) for phases that are data-parallel over districts (where D = district count). The total tick time is dominated by the citizen-level phases:

```
T_tick(N, D) &asymp; c_citizen * N + c_district * D * log(D)

For N=1k, D=20:   T &asymp; 28ms (measured target)
For N=10k, D=100: T &asymp; 40ms (target; 10x citizens + 5x districts)
For N=100k, D=500: T &asymp; 120ms (target; 100x citizens + 25x districts)
```

The super-linear growth from N=10k to N=100k (10x citizens, but ~3x time increase rather than 10x) reflects: (a) rayon parallel scaling across 14 cores absorbing 10x work in ~2x wall time, and (b) cache effects (Medium fits in L2, Large spills to L3).

---

## 12. Optimization Methodology

### 12.1 Measure First Mandate

No optimization may be implemented without a failing benchmark or a profile trace identifying the hotspot. The following workflow is mandatory:

```
1. Identify problem
   └─ Is there a failing SLO? Which scenario? Which percentile?
   └─ Is there a criterion benchmark regressing? By how much?

2. Profile
   └─ cargo flamegraph --scenario X --ticks 1000
   └─ perf stat -e cache-misses (Linux)
   └─ Identify: which phase? which function? what % of tick time?

3. Form hypothesis
   └─ "The ideology diffusion loop has a cache miss because..."
   └─ "The market clearing is O(N²) when it could be O(N log N) because..."

4. Implement
   └─ Write the optimization in an isolated PR
   └─ Add a criterion benchmark that demonstrates the improvement

5. Benchmark
   └─ cargo criterion --baseline main
   └─ Confirm: does the target metric improve?
   └─ Confirm: does overall tick time improve?

6. Verify invariants
   └─ cargo test --all (all determinism tests must pass)
   └─ Run 1000-tick replay verification test
   └─ Confirm: output is identical to pre-optimization baseline

7. Merge
   └─ PR includes: flamegraph before/after, criterion comparison table
```

### 12.2 Forbidden Optimizations

The following optimizations are categorically forbidden regardless of performance benefit:

| Forbidden Technique               | Reason                                                       |
|----------------------------------|-------------------------------------------------------------|
| Remove BLAKE3 hash computation   | Hash is required for determinism verification (CIV-0001 E3) |
| Disable determinism replay tests | Replay tests are a correctness guarantee, not a nice-to-have|
| Use `unsafe` pointer arithmetic in hot loop | Permitted only in named `simd_*.rs` modules with full test coverage |
| Use `HashMap` in deterministic paths | Breaks CIV-0001 invariant I3 (non-deterministic iteration) |
| Merge stochastic phase into parallel execution | Breaks seeded RNG sequential contract |
| Skip event emission for "unimportant" events | Events are the audit trail; omitting any event violates E3 |
| Cache simulation state across tick boundaries without double-buffering | Breaks determinism under parallelism |
| Disable metrics aggregation in production | Metrics are required for research output correctness |

### 12.3 Approved Optimization Techniques

| Technique                          | Conditions                                             |
|-----------------------------------|-------------------------------------------------------|
| `rayon::par_iter` for entity loops | Double-buffer pattern must be in place                |
| `std::simd` for arithmetic         | Scalar path must remain as correctness reference; CI checks parity |
| `inline(always)` on hot functions | Only if profiler shows measurable benefit; document the measurement |
| Arena allocation for tick-scoped data | Use `bumpalo` crate; arena reset at tick boundary   |
| Prefetch hints (`_mm_prefetch`)    | Only in named `simd_*.rs` modules; test on target hardware |
| `zstd` compression level tuning   | Level 1-3 only; level 3 is default; document rationale for deviation |
| `#[cold]` / `#[likely]` attributes | Only on branches where branch predictor data confirms the hint |
| Precomputed lookup tables          | For O(1) neighborhood queries, sine/cosine of fixed angles |

### 12.4 Performance Review Gate

At the completion of each implementation phase (per PLAN.md), a performance review gate runs:

```bash
# Phase N completion performance gate
cargo criterion --save-baseline phase-N

# Verify all SLO targets are met:
cargo run --release --bin civ-perf-check \
    -- --scenario small --assert-p99-ms 16 \
    -- --scenario medium --assert-p99-ms 60 \
    -- --scenario large --assert-p99-ms 150

# Verify memory usage is within bounds:
cargo run --release --bin civ-mem-check \
    -- --scenario medium --assert-rss-mb 512
```

The gate fails if any SLO assertion fails. Phase cannot be marked complete until the gate passes.

### 12.5 Technical Debt Tracking

Optimization debt is tracked as first-class work items in `docs/reference/WORK_STREAM.md`. Every known performance gap that is deferred (with justification) must have a WORK_STREAM entry with:

- The failing SLO or benchmark regression it corresponds to
- The root cause (from profiler data)
- The proposed fix
- The priority tier (P0 = blocks current phase; P1 = blocks next phase; P2 = backlog)

P0 items block phase completion. P1 items must be resolved before the subsequent phase begins. P2 items are addressed during Phase 6 (Polish + Hardening).

---

## 13. Acceptance Criteria

### FR-CIV-PERF-001: Small Scenario SLO
**Spec:** 1k-citizen tick completes with p50 \< 8ms, p99 \< 16ms over 1,000-tick window.
**Test:** `bench_tick_1k_citizens` criterion benchmark; assert p99 \< 16ms on CI perf machine.
**Status:** Open

### FR-CIV-PERF-002: Medium Scenario SLO
**Spec:** 10k-citizen tick completes with p50 \< 30ms, p99 \< 60ms over 1,000-tick window.
**Test:** `bench_tick_10k_citizens`; assert p99 \< 60ms on CI perf machine.
**Status:** Open

### FR-CIV-PERF-003: Large Scenario SLO
**Spec:** 100k-citizen tick completes with p50 \< 80ms, p99 \< 150ms over 1,000-tick window.
**Test:** `bench_tick_100k_citizens`; assert p99 \< 150ms on CI perf machine.
**Status:** Open

### FR-CIV-PERF-004: WebSocket Command Latency
**Spec:** Command-to-ack latency \< 5ms p50, < 20ms p99.
**Test:** `bench_ws_command_ack`; inject 100 commands, measure ack time distribution.
**Status:** Open

### FR-CIV-PERF-005: Delta Snapshot Generation
**Spec:** Delta snapshot generated in \< 10ms for any scenario size.
**Test:** `bench_snapshot_serialize_delta`; assert p99 \< 10ms.
**Status:** Open

### FR-CIV-PERF-006: Full Snapshot at 10k Citizens
**Spec:** Full snapshot generated in \< 25ms for 10k-citizen state.
**Test:** `bench_snapshot_serialize_full`; assert mean \< 25ms at 10k citizens.
**Status:** Open

### FR-CIV-PERF-007: Replay Seek
**Spec:** Seek to any tick within last 100 ticks completes in \< 10ms; seek to arbitrary tick \< 100ms.
**Test:** `bench_replay_seek_recent`; `bench_replay_seek_arbitrary`.
**Status:** Open

### FR-CIV-PERF-008: ECS SoA Layout
**Spec:** Citizen hot data (24 bytes/citizen) laid out in SoA; 1k citizens fit in L1 cache.
**Test:** `test_ecs_hot_data_layout`; verify array strides and alignment via memory layout assertions.
**Status:** Open

### FR-CIV-PERF-009: SIMD Ideology Dot Product Speedup
**Spec:** SIMD ideology dot product achieves >= 2.5x speedup vs. scalar on AVX2 hardware.
**Test:** `bench_ideology_diffusion_1k`; criterion comparison; assert speedup >= 2.5x.
**Status:** Open

### FR-CIV-PERF-010: DB Async Write Non-Blocking
**Spec:** DB write spawn adds \< 0.5ms to tick wall time; DB write completes in background within 50ms.
**Test:** `bench_db_write_spawn`; assert tick overhead \< 0.5ms; `test_db_write_async` verifies background completion.
**Status:** Open

### FR-CIV-PERF-011: WebSocket Fanout 100 Clients
**Spec:** Broadcasting one tick snapshot to 100 connected clients completes in \< 5ms.
**Test:** `bench_websocket_fanout_100clients`.
**Status:** Open

### FR-CIV-PERF-012: Phase Parallelism
**Spec:** Parallel phase rounds (demographics||climate, production||military||research, social||economy) demonstrably reduce wall time vs. sequential execution.
**Test:** `bench_phase_parallel_vs_sequential`; assert parallel execution \< 70% of sequential time.
**Status:** Open

### FR-CIV-PERF-013: No Performance Regression on PR
**Spec:** All criterion benchmarks on a PR must not regress > 10% on p99 vs. main branch baseline.
**Test:** Automated CI check via `scripts/check_perf_regression.py`.
**Status:** Open

### FR-CIV-PERF-014: L1 Cache Miss Rate (Small Scenario)
**Spec:** L1 data cache miss rate \< 1% during Small scenario hot loop (citizens processing).
**Test:** `perf stat` measurement in CI; assert L1-dcache-load-misses \< 1%.
**Status:** Open

### FR-CIV-PERF-015: Tracing Overhead
**Spec:** Tracing spans at `debug` level add \< 2% overhead to tick wall time vs. release build without tracing.
**Test:** `bench_tick_with_tracing_vs_without`; assert overhead \< 2%.
**Status:** Open

### FR-CIV-PERF-016: Snapshot Ring Buffer Capacity
**Spec:** Delta snapshot ring buffer holds exactly 100 snapshots; oldest snapshot is evicted on push-101.
**Test:** `test_delta_ring_buffer_eviction`.
**Status:** Open

### FR-CIV-PERF-017: Batch DB Insert
**Spec:** All events from one tick inserted via one batch INSERT statement (not N individual inserts).
**Test:** `test_batch_event_insert`; mock DB and assert exactly 1 SQL statement executed per tick.
**Status:** Open

### FR-CIV-PERF-018: Client Lag Eviction
**Spec:** Client lagging > 100 ticks behind broadcast ring buffer is disconnected with `CLIENT_LAG_EXCEEDED`.
**Test:** `test_ws_client_lag_eviction`; simulate slow client, verify disconnect after 100-tick lag.
**Status:** Open

### FR-CIV-PERF-019: SIMD Correctness Parity
**Spec:** For all SIMD-accelerated functions, output equals scalar reference for all valid inputs.
**Test:** `test_ideology_dot_simd_parity`, `test_price_clearing_simd_parity`, `test_co2_aggregate_simd_parity`; proptest with random inputs.
**Status:** Open

### FR-CIV-PERF-020: Memory Alignment
**Spec:** All citizen hot component arrays are 32-byte aligned.
**Test:** `test_hot_array_alignment`; assert `array.as_ptr() as usize % 32 == 0` for all hot arrays.
**Status:** Open

---

## References

- **CIV-0001:** Core Simulation Loop — Deterministic Tick Architecture (ECS model, tick phases, determinism invariants I1-I6)
- **CIV-0100:** Economy Module v1 (economy tick pipeline, market clearing, conservation invariants)
- **CIV-0107:** Joule Economy System v1 (citizen-level joule ledger, allocation mechanics)
- **CIV-0200:** Client Protocol (WebSocket, snapshot format, binary frames)
- **PLAN.md:** Phase 0-6 task breakdown and critical path (P6.5 performance benchmarks task)
- **Rust `rayon` docs:** https://docs.rs/rayon
- **Rust `std::simd` docs:** https://doc.rust-lang.org/std/simd/index.html
- **`criterion` crate:** https://bheisler.github.io/criterion.rs/book/
- **`tracing` crate:** https://docs.rs/tracing
- **`sqlx` crate:** https://docs.rs/sqlx
- **`zstd` crate:** https://docs.rs/zstd
- **`tokio::sync::broadcast`:** https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html
- **Pixi.js v8 ParticleContainer:** https://pixijs.com/8.x/guides/components/particle-container

---

**Version History:**
- v1.0 (2026-02-21): Initial full specification. Covers SLOs, ECS memory layout, SIMD targets, parallelism architecture, profiling infrastructure, snapshot optimization, DB performance, WebSocket throughput, web client rendering, benchmark suite, phase budget allocation, and optimization methodology.
