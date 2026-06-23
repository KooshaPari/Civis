# Watch Crate Traceability Audit
**Date:** 2026-06-17  
**Crate:** `crates/watch/`  
**Purpose:** Identify untagged public API items and propose FR/NFR mappings  

---

## Executive Summary

| Metric | Count |
|--------|-------|
| Total public items (pub fn/struct/enum/const) | 14 |
| Items with FR/NFR tags in doc comments | 2 |
| **Untagged items** | **12** |
| Coverage | **14.3%** |

The watch crate is a lightweight HTTP server for the Civis 3D sandbox dashboard with procedural terrain generation and simulation worker orchestration. Most public items lack explicit FR traceability tags in their documentation.

---

## Gap Map: Untagged Public Items

| Name | File:Line | Item Kind | Proposed FR | Reasoning |
|------|-----------|-----------|-------------|-----------|
| `run()` | server.rs:77 | async fn | FR-PROTO-001 | HTTP server bootstrap and WebSocket setup for client connections |
| `SIZE` | terrain.rs:14 | const usize | FR-API-001 | Terrain grid dimensions; part of scenario configuration/generation API |
| `Biome` | terrain.rs:19 | enum | FR-API-001 | Biome classification system; maps to procedural generation and terrain data model |
| `Biome::rgb()` | terrain.rs:39 | pub fn | (Internal) | Helper for serialization; consider tag as NFR-RENDERING or leave untagged |
| `Biome::from_height()` | terrain.rs:52 | pub fn | FR-CORE-003 / FR-API-001 | Deterministic biome assignment from height; part of replay-safe terrain generation |
| `Terrain` | terrain.rs:73 | struct | FR-CORE-003 | Represents persistent terrain state; must be bit-identical under replay |
| `Terrain::heights_fingerprint()` | terrain.rs:84 | pub fn | FR-REPLAY-002 | State hash for determinism verification; directly enables replay verification |
| `Terrain::generate()` | terrain.rs:96 | pub fn | FR-CORE-003 | Deterministic terrain generation; seeded RNG for replay safety |
| `Terrain::cell_index()` | terrain.rs:137 | pub fn | (Internal) | Spatial indexing helper; consider as NFR-PERF or leave untagged |
| `Terrain::biome_at()` | terrain.rs:147 | pub fn | (Internal) | Query helper for spatial operations; possibly NFR-PERF |
| `Terrain::is_walkable()` | terrain.rs:152 | pub fn | FR-CORE-002 | Entity pathfinding/placement constraint; relates to ECS entity spatial validation |
| `pub mod terrain` (re-export) | lib.rs:17 | mod | FR-API-001 | Public module export for terrain API surface |

---

## Tagged Items (Existing Traceability)

| Name | File:Line | FR Tag | Context |
|------|-----------|--------|---------|
| `AppState::mods` | app.rs:300 | FR-CIV-TACTICS-054 | Loaded mods for dashboard browser (custom Civis domain tag) |
| Airport/Port/Hangar placement | snapshot.rs:663 | FR-CIV-UX-006 | Placed infrastructure; ECS authoring (custom Civis domain tag) |

**Note:** Existing tags use `FR-CIV-*` domain (Civis-specific extensions); standard FUNCTIONAL_REQUIREMENTS.md uses `FR-{CATEGORY}-{NUMBER}` (CORE, ECON, PROTO, REPLAY, API, CLIENT).

---

## Proposed Traceability Strategy

### Primary Mapping Rules

1. **Terrain Generation & Determinism** → `FR-CORE-003` (Deterministic Transition Phase)
   - `Terrain::generate()`, `Terrain::heights_fingerprint()`, `Biome::from_height()`
   - Justification: Seeded RNG + bit-identical output required for replay safety

2. **Terrain Query API** → `FR-API-001` (Scenario YAML Format and Validation)
   - `Terrain`, `SIZE`, `Biome`, `Terrain::is_walkable()`
   - Justification: Terrain is part of the scenario definition and runtime data model

3. **Replay Verification** → `FR-REPLAY-002` (Bit-Identical Determinism Verification)
   - `Terrain::heights_fingerprint()`
   - Justification: State hash directly supports replay determinism CI gate

4. **HTTP Server Bootstrap** → `FR-PROTO-001` (RFC 6455 WebSocket Server)
   - `run()` in server.rs
   - Justification: Initializes socket, router, and client connection handling

5. **Internal/Untagged Helpers** (do not tag)
   - `Terrain::cell_index()`, `Terrain::biome_at()`, `Biome::rgb()`
   - Justification: Private implementation details; consumers care about higher-level APIs

---

## Recommended Documentation Updates

### 1. server.rs — `pub async fn run()`

**Current:**
```rust
pub async fn run() {
    // ...
}
```

**Proposed:**
```rust
/// Bootstrap the Civis watch HTTP server.
///
/// Initializes the simulation worker, terrain cache, save database, and
/// Axum router, then listens for WebSocket clients on the configured port.
/// Handles all watch APIs: SSE snapshots, terrain, control routes, mods, saves.
///
/// **FR-PROTO-001**: Provides RFC 6455 WebSocket server entry point.
pub async fn run() {
    // ...
}
```

### 2. terrain.rs — `pub const SIZE` and `pub enum Biome`

**Current:**
```rust
pub const SIZE: usize = 256;

pub enum Biome { ... }
```

**Proposed:**
```rust
/// Side length of the generated terrain grid.
///
/// **FR-API-001**: Scenario definition includes map dimensions.
pub const SIZE: usize = 256;

/// One terrain biome. Maps to a colour in the web dashboard.
///
/// **FR-API-001** / **FR-CORE-003**: Deterministic biome classification
/// from height ensures replay-safe terrain state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Biome {
    // ...
}
```

### 3. terrain.rs — `impl Terrain`

**Current:**
```rust
pub fn heights_fingerprint(&self) -> u64 { ... }
pub fn generate(seed: u64) -> Self { ... }
pub fn is_walkable(&self, x: f32, y: f32) -> bool { ... }
```

**Proposed:**
```rust
/// FNV-1a digest of all height samples (bit-exact, replay-safe).
///
/// **FR-REPLAY-002**: State hash enables bit-identical determinism verification.
/// Used by replay CI gate to confirm state at every tick.
pub fn heights_fingerprint(&self) -> u64 { ... }

/// Generate a new heightmap from `seed`. Deterministic.
///
/// **FR-CORE-003**: Seeded RNG ensures bit-identical terrain under replay.
/// Multi-octave value noise with radial/ridge falloff for natural coastlines.
pub fn generate(seed: u64) -> Self { ... }

/// Return whether the position is walkable terrain.
///
/// **FR-CORE-002**: ECS entity placement validation.
/// Non-walkable biomes: DeepWater, Water.
pub fn is_walkable(&self, x: f32, y: f32) -> bool { ... }
```

### 4. lib.rs — Module Re-exports

**Current:**
```rust
pub mod terrain;
pub use server::run;
```

**Proposed:**
```rust
/// Procedural terrain generation (256×256 heightmap + biomes).
///
/// **FR-API-001** / **FR-CORE-003**: Deterministic, seeded terrain for
/// scenario initialization and replay safety.
pub mod terrain;

/// Bootstrap the watch server.
///
/// **FR-PROTO-001**: WebSocket server entry point.
pub use server::run;
```

---

## Implementation Notes

### Deferred Items (No Tag Recommended)

- `Biome::rgb()` — Serialization helper for web dashboard; not a FR requirement
- `Terrain::cell_index()` — Spatial indexing utility; internal to `biome_at()` and `is_walkable()`
- `Terrain::biome_at()` — Query helper; consumers use `is_walkable()` or direct iteration

### Domain Tag Alignment

The FUNCTIONAL_REQUIREMENTS.md uses standard categories:

| Category | Scope |
|----------|-------|
| CORE | Simulation engine, tick loop, ECS |
| ECON | Production, markets, taxation, allocation |
| PROTO | WebSocket, JSON-RPC, client protocol |
| REPLAY | Determinism, replay files, audit trails |
| API | Scenario YAML, Python SDK, parameter override |
| CLIENT | Client implementations, integration |

Watch crate aligns primarily with **PROTO**, **CORE** (determinism), and **API** (terrain in scenario context).

### Cross-Repo Reuse Opportunity

Terrain generation and determinism verification are candidates for extraction to a shared `phenotype-terrain` module if other projects (Civis, DINOForge, etc.) reuse procedural terrain. Current strategy: keep in-tree pending consolidation planning.

---

## Files and Paths

- **Audit source:** `/crates/watch/src/`
- **FUNCTIONAL_REQUIREMENTS.md:** `FUNCTIONAL_REQUIREMENTS.md` (repo root)
- **Spec docs:** `agileplus-specs/` (if present)
- **Output:** This file at `docs/audits/watch-trace-gaps-2026-06-17.md`

---

## Next Steps (No Code Changes)

1. **Review** this audit for accuracy; confirm proposed FR mappings with domain owners
2. **Plan** a PR to add FR tags to doc comments per the "Recommended Documentation Updates" section
3. **Monitor** future public API additions to include FR tags at definition time
4. **Consider** adding a lint rule to enforce FR/NFR tags on public items (optional, future enhancement)

---

## Audit Metadata

- **Auditor:** Claude (read-only analysis)
- **Method:** Grep + Read for pub items + doc comment inspection
- **Scan scope:** All `.rs` files in `crates/watch/src/`
- **Date generated:** 2026-06-17
- **Status:** Informational (audit only; no code modified)
