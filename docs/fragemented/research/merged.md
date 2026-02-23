# Merged Fragmented Markdown

## Source: research/RND-001-ecs-library-decision.md

# RND-001: ECS Library Final Decision -- legion vs bevy_ecs vs hecs vs specs

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**Recommendation: `bevy_ecs` (standalone, version 0.18.x)** as the ECS library for CivLab's
deterministic headless Rust simulation. `bevy_ecs` provides the best combination of active
maintenance, archetype-based cache-friendly iteration, standalone usability (no full Bevy engine
required), and ecosystem alignment since the CivLab web client will use Bevy's 2D renderer.
While no ECS library guarantees deterministic iteration order out of the box, `bevy_ecs` 0.15+
provides `Query::iter().sort_by_key()` which enables deterministic iteration by sorting on
`Entity` (a monotonic `u32` index + generation). The alternatives are disqualified: `legion` is
effectively archived, `specs` is unmaintained, and `hecs` is too minimal (no built-in scheduler,
no resources, no sorted iteration).

---

## Research Findings

### 1. Candidate Overview

| Crate | Version | Last Updated | Storage Model | Maintained | Notes |
|-------|---------|-------------|---------------|------------|-------|
| `bevy_ecs` | 0.18.0 | 2026-01 (active) | Archetype (table) + sparse set | Yes (Bevy team) | Can be used standalone |
| `legion` | 0.4.0 | 2022-12 (stale) | Archetype | No (archived/abandoned) | Under `amethyst/` org, Amethyst defunct |
| `hecs` | 0.10.5 | 2024-06 (low activity) | Archetype | Semi (solo maintainer) | Minimal by design |
| `specs` | 0.20.0 | 2024-06 (stale) | Sparse set (bitset-based) | No (unmaintained) | Under `amethyst/` org |

### 2. Detailed Analysis

#### 2.1 `bevy_ecs` (standalone)

**Architecture:**
- Archetype-based storage by default (table storage): entities with identical component sets
  are stored together in contiguous arrays per component type. This maximizes cache locality
  during iteration.
- Optional `SparseSet` storage per-component for components that are frequently added/removed
  (e.g., tags, markers). Configured via `#[component(storage = "SparseSet")]`.
- Entity IDs are 64-bit: 32-bit index + 32-bit generation. Monotonically allocated indices
  with generation counters for reuse detection.

**Standalone usage:**
- `bevy_ecs` is published as a separate crate on crates.io and can be depended on directly
  without pulling in the full Bevy engine (renderer, windowing, audio, etc.).
- `Cargo.toml` dependency: `bevy_ecs = "0.18"` -- no `bevy` dependency needed.
- You get `World`, `Entity`, `Component`, `Query`, `System`, `Schedule`, `Resource` --
  everything needed for a headless simulation.
- Feature flags allow disabling features not needed for headless use (e.g., `reflect` can be
  omitted).

**Determinism -- iteration order:**
- **Default iteration order is NOT deterministic.** Queries iterate entities grouped by
  archetype, and within an archetype, iteration follows insertion order. But archetype order
  itself depends on the order components were first combined, which can vary across runs if
  systems race or if entity construction order differs.
- **Solution:** Since Bevy 0.15, `QueryIter` supports `.sort_by::<Entity>(...)` and
  `.sort_by_key::<Entity, _>(...)` via `QuerySortedIter`. This sorts entities by `Entity` ID
  (or any component) before iteration, providing deterministic order.
- **Performance cost of sorting:** O(n log n) per query per frame, where n is the number of
  matching entities. For 100k entities this is ~1.7M comparisons -- roughly 50-100us on modern
  hardware. Acceptable for a tick-based simulation (not a 60fps renderer).
- **Alternative:** If sorted iteration is too expensive for hot-path queries, maintain a
  side-channel `Vec<Entity>` sorted once on insert, and iterate that instead. But for CivLab's
  tick-based model, sorting per tick is fine.

**Parallel system scheduling:**
- `bevy_ecs` schedules systems in parallel by default (via the `Schedule` executor). Systems
  that don't conflict on component access run simultaneously.
- For determinism, CivLab must use **single-threaded execution** or enforce a fixed system
  ordering. The `Schedule` can be configured to run sequentially: systems added in explicit
  order with `.chain()` or by using `SystemSet` ordering.
- Recommendation: Use `Schedule` with explicit `.before()` / `.after()` constraints and
  **disable multi-threading** in the simulation tick. Multi-threading can be used for non-
  simulation work (rendering, network IO).

**Ecosystem benefits:**
- CivLab's web client will use Bevy's 2D renderer. Using `bevy_ecs` on the server means
  the same `Component` derives work on both sides. Shared component types between client and
  server reduce serialization friction.
- `bevy_ecs` has `serde` support via `bevy_reflect` for component serialization/snapshot.
- Active community: bug fixes, performance improvements, new features every release cycle.

**Benchmarks (archetype query throughput):**
- Bevy ECS iterates ~1 billion components/sec for simple linear queries on modern hardware
  (based on community benchmarks at 100k entities with 3-component queries).
- Table storage achieves near-perfect cache utilization for iteration. Sparse set is ~2-4x
  slower for iteration but ~10x faster for add/remove.
- At CivLab's scale (10k-100k entities), query iteration is sub-millisecond.

#### 2.2 `legion`

**Status: DISQUALIFIED -- effectively archived.**

- Last meaningful commit: 2022. No releases since 0.4.0.
- Under the `amethyst/` GitHub organization, which is defunct (Amethyst game engine project
  ended).
- Multiple forks exist but none have gained traction.
- API is well-designed (archetype-based, good query ergonomics) but bitrot risk is high.
- No guarantee of compatibility with future Rust editions or toolchain changes.

**If it were maintained:**
- Legion uses archetype-based storage similar to bevy_ecs.
- Iteration order is also not guaranteed deterministic.
- Has a `Schedule` system but less mature than Bevy's.
- No Bevy ecosystem integration.

**Verdict:** Do not use. Archived project with no path forward.

#### 2.3 `hecs`

**Architecture:**
- Minimal archetype-based ECS. Entities grouped by archetype in contiguous storage.
- No built-in scheduler, no `Resource` type, no system ordering -- purely a data store.
- Designed for embedding: you bring your own game loop and system dispatch.

**Strengths:**
- Very small API surface, easy to understand and audit.
- Minimal dependencies (almost zero).
- Good for embedded or constrained environments.

**Weaknesses for CivLab:**
- **No built-in scheduler:** CivLab needs to run 20+ simulation systems per tick in a
  defined order. With `hecs`, you must build your own system runner. This is non-trivial to
  get right (dependency ordering, change detection, etc.).
- **No resources:** Global simulation state (e.g., tick counter, RNG seed, world clock) must
  be stored externally and threaded through manually.
- **No sorted iteration API:** Must collect into `Vec<(Entity, &Component)>` and sort
  manually. No built-in `sort_by_key`.
- **No change detection:** Bevy's `Changed<T>` and `Added<T>` query filters are essential
  for efficient simulation (e.g., only recalculate food for citizens whose hunger component
  changed). `hecs` has no equivalent.
- **Solo maintainer risk:** While active, the bus factor is 1.

**Verdict:** Too minimal. CivLab would need to build ~2000 LOC of infrastructure that
`bevy_ecs` provides out of the box.

#### 2.4 `specs`

**Status: DISQUALIFIED -- unmaintained.**

- Last release: 0.20.0 (2024-06 bugfix only). No active development.
- Under `amethyst/` organization (defunct).
- Uses bitset-based sparse storage: each component type has a `DenseVecStorage` or
  `HashMapStorage`, accessed via `BitSet` join operations.

**Architecture differences:**
- Sparse-set model means iteration involves bitset intersection, which is slower than
  archetype iteration for dense queries but faster for sparse queries.
- Has `SystemData` and `Dispatcher` for system scheduling.
- `Join` API for queries: `(&positions, &velocities).join()`.

**Weaknesses:**
- Bitset join is ~2-5x slower than archetype iteration for typical ECS patterns (most
  entities have the same components).
- Unmaintained -- same risk profile as `legion`.
- No Bevy integration.

**Verdict:** Do not use. Unmaintained and architecturally inferior for CivLab's dense-entity
workload.

### 3. Determinism Strategy with `bevy_ecs`

CivLab requires bit-for-bit determinism for:
1. **Replay:** Same inputs produce same outputs.
2. **Multiplayer lockstep:** All clients compute identical state.
3. **AI MCTS rollouts:** Forward simulation must be reproducible.

**Determinism threats and mitigations:**

| Threat | Mitigation |
|--------|------------|
| Query iteration order varies | Use `.sort_by_key::<Entity>(Entity::index)` on all simulation queries |
| Parallel system execution | Run simulation schedule single-threaded with explicit ordering |
| HashMap iteration order | Use `BTreeMap` or sorted `Vec` for all simulation-critical maps |
| Entity allocation order | Entities are allocated sequentially (monotonic index); deterministic if spawn order is fixed |
| Component insertion order | Insert components in the same order every time (use bundles) |
| `f32`/`f64` non-determinism | Separate concern: use fixed-point (see RND-003) |

**Entity ID determinism:**
- `bevy_ecs` allocates entities with monotonically increasing 32-bit indices. If entities are
  spawned in the same order each tick, their IDs are deterministic.
- After despawn, indices are recycled with incremented generation counters. This is also
  deterministic if despawn order is fixed.
- Sorting by `Entity` (which compares index then generation) produces a stable, reproducible
  order.

### 4. Hot-reload / Modding Considerations

- `bevy_ecs` does not natively support hot-reloading of Rust system functions at runtime.
- For CivLab modding: systems can be data-driven (e.g., a generic "apply policy" system that
  reads policy parameters from a component/resource, where policy parameters are loaded from
  mod files at startup).
- Alternatively, use `bevy_ecs`'s `DynamicBundle` and `CommandQueue` to allow mods to spawn
  entities and modify components via a scripting bridge (e.g., `wasm` or `rhai`).
- Full system hot-reload requires dynamic library loading (`libloading`) which is orthogonal
  to the ECS choice.

---

## Decision

**Use `bevy_ecs` version 0.18.x as a standalone crate.**

Rationale:
1. **Ecosystem alignment:** Bevy 2D renderer for client means shared `Component` types.
2. **Active maintenance:** Monthly releases, large contributor base, funded development.
3. **Feature completeness:** Scheduler, resources, change detection, sorted queries -- all
   required by CivLab's simulation complexity.
4. **Determinism is achievable:** With sorted iteration + single-threaded schedule +
   fixed entity spawn order, bit-for-bit determinism is tractable.
5. **Performance:** Archetype storage is optimal for CivLab's dense-entity workload
   (most entities have similar component sets: position, nation, health, resources).

---

## Implementation Contract

### Cargo.toml

```toml
[dependencies]
bevy_ecs = { version = "0.18", default-features = false, features = [
    # Include only what the headless simulation needs:
    # - No "bevy_reflect" unless serialization via reflect is needed
    # - No "multi_threaded" for simulation schedule (determinism)
] }
```

**Pin strategy:** Use `=0.18.x` for lockfile stability. Bevy has breaking changes each minor
version. Upgrade deliberately, not automatically.

### Component Definition Pattern

```rust
use bevy_ecs::prelude::*;

/// All simulation components derive Component with default (Table) storage.
/// Use SparseSet only for frequently added/removed marker components.
#[derive(Component, Clone, Debug)]
pub struct Position {
    pub hex: civlab_hex::Hex,  // axial coordinates, see RND-002
}

#[derive(Component, Clone, Debug)]
pub struct Population {
    pub count: i64,           // citizen count, plain i64
    pub food_stockpile: i64,  // joules, i64 x SCALE, see RND-003
}

/// Marker component: added/removed frequently. Use SparseSet.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsRecalculation;
```

### System Definition Pattern

```rust
use bevy_ecs::prelude::*;

/// All simulation systems MUST use sorted iteration for determinism.
/// Sort by Entity index to ensure stable processing order.
fn update_population(
    mut query: Query<(Entity, &mut Population, &FoodProduction)>,
) {
    // CRITICAL: sort_by_key for deterministic iteration
    let mut sorted: Vec<_> = query.iter_mut().collect();
    sorted.sort_by_key(|(entity, _, _)| entity.index());

    for (_entity, mut pop, food) in sorted {
        // Fixed-point arithmetic only, no f32/f64
        pop.food_stockpile += food.output_per_tick;
        // ... simulation logic
    }
}
```

### Schedule Configuration

```rust
use bevy_ecs::prelude::*;

/// Simulation schedule: single-threaded, explicit system ordering.
fn build_simulation_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    // Use SingleThreaded executor for determinism
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);

    // Explicit system ordering via .chain() or .before()/.after()
    schedule.add_systems((
        update_food_production,
        update_population,
        update_economy,
        update_military,
        update_diplomacy,
        update_research,
        update_climate,
    ).chain());  // .chain() = strictly sequential in listed order

    schedule
}
```

### Deterministic Entity Spawning

```rust
/// Entity spawning must be deterministic: same tick, same order, same result.
/// Use explicit batch spawning with a sorted input.
fn spawn_citizens(
    mut commands: Commands,
    new_citizen_events: Res<Events<NewCitizenEvent>>,
) {
    // Events are processed in insertion order (deterministic)
    for event in new_citizen_events.iter() {
        commands.spawn((
            Position { hex: event.hex },
            Population { count: 1, food_stockpile: 0 },
            Nation { id: event.nation_id },
        ));
    }
    // commands.apply() happens at end of stage -- deterministic
}
```

### World Snapshot for MCTS

```rust
/// For MCTS rollouts: extract minimal state, not full World clone.
/// See RND-011 for MCTS state representation contract.
fn snapshot_nation_state(
    world: &World,
    nation_id: NationId,
) -> NationSnapshot {
    // Query only nation-relevant components
    let mut query = world.query::<(&Nation, &Economy, &Military, &Relations)>();
    let mut snapshot = NationSnapshot::default();

    for (nation, economy, military, relations) in query.iter(world) {
        if nation.id == nation_id {
            snapshot.economy = economy.clone();
            snapshot.military = military.clone();
            snapshot.relations = relations.clone();
        }
    }
    snapshot
}
```

### Integration Test: Determinism Verification

```rust
#[cfg(test)]
mod determinism_tests {
    use super::*;

    /// Run the same simulation twice with identical inputs.
    /// Assert bit-for-bit identical output.
    #[test]
    fn simulation_is_deterministic() {
        let inputs = load_test_inputs();

        let state_a = run_simulation(&inputs, 100 /* ticks */);
        let state_b = run_simulation(&inputs, 100 /* ticks */);

        // Byte-level comparison of serialized state
        assert_eq!(
            state_a.serialize_deterministic(),
            state_b.serialize_deterministic(),
            "Simulation diverged: non-determinism detected"
        );
    }
}
```

---

## Open Questions Remaining

1. **bevy_ecs version pinning strategy:** Bevy releases break APIs every minor version.
   CivLab should pin to `=0.18.x` and schedule deliberate upgrades. Need to verify that
   `bevy_ecs` 0.18 compiles standalone without the full `bevy` crate pulling in `wgpu`,
   `winit`, etc.

2. **Sorted iteration performance at scale:** The `sort_by_key` approach is O(n log n) per
   query. At 100k entities, profiling is needed to confirm this is within budget (target:
   <1ms per sorted query). If too slow, consider maintaining pre-sorted entity lists as a
   `Resource` that gets incrementally updated on spawn/despawn.

3. **Change detection + sorted iteration interaction:** Bevy's `Changed<T>` filter narrows
   the query set before iteration. Verify that sorted iteration over `Changed<T>` results
   is also deterministic (it should be, since `Changed` is archetype-scoped and sort
   operates on the filtered set).

4. **Snapshot serialization for networking/replay:** Evaluate `bevy_reflect` vs manual
   `serde` for component serialization. Manual `serde` is more predictable for determinism
   (no reflection overhead, explicit field ordering). Preliminary recommendation: manual
   `serde` with `#[derive(Serialize, Deserialize)]` on all simulation components.

5. **WASM compatibility:** If CivLab server runs in WASM (for in-browser singleplayer),
   `bevy_ecs` compiles to WASM but the single-threaded executor must be used (no `rayon`
   in WASM). This aligns with our determinism requirement.

---

## References

- [bevy_ecs docs.rs](https://docs.rs/bevy_ecs/latest/bevy_ecs/)
- [bevy_ecs crates.io](https://crates.io/crates/bevy_ecs)
- [Bevy 0.16 release notes](https://bevy.org/news/bevy-0-16/)
- [Bevy determinism discussion #2480](https://github.com/bevyengine/bevy/discussions/2480)
- [Bevy ordered iteration issue #1470](https://github.com/bevyengine/bevy/issues/1470)
- [Bevy high-performance sorted queries #13464](https://github.com/bevyengine/bevy/issues/13464)
- [legion GitHub (archived)](https://github.com/amethyst/legion)
- [hecs GitHub](https://github.com/Ralith/hecs)
- [specs GitHub (unmaintained)](https://github.com/amethyst/specs)
- [Deterministic lockstep networking](https://gafferongames.com/post/deterministic_lockstep/)
- [Rust ECS ecosystem overview](https://arewegameyet.rs/ecosystem/ecs/)


---

## Source: research/RND-002-hexagonal-grid-decision.md

# RND-002: Hexagonal Grid Library -- hexx vs Manual Axial Implementation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**Recommendation: Use the `hexx` crate (version 0.21.x)** for CivLab's hexagonal grid system.
`hexx` is the most actively maintained Rust hex library, provides integer-based axial
coordinates (`Hex` struct with `i32` fields), includes A* pathfinding, range/ring/spiral
queries, neighbor lookups, coordinate conversions (axial/cube/offset/doubled), and has optional
Bevy integration. The core hex coordinate math is entirely integer (`i32`), meeting our
determinism requirement. The `f32` usage is confined to the `HexLayout` display layer
(hex-to-pixel conversion for rendering), which is non-simulation code. CivLab should use
`hexx` for coordinate math and pathfinding, with a thin wrapper for simulation-specific
concerns (movement cost evaluation, territory boundaries, fog of war). A manual implementation
would require ~1500-2000 LOC to replicate what `hexx` provides, with ongoing maintenance burden
and no community review.

---

## Research Findings

### 1. `hexx` Crate Analysis

**Repository:** [github.com/ManevilleF/hexx](https://github.com/ManevilleF/hexx)
**Version:** 0.21.0 (latest as of 2026-02)
**License:** MIT/Apache-2.0
**Downloads:** ~300k total on crates.io -- well-adopted in the Rust gamedev ecosystem.

#### 1.1 Core Type: `Hex`

```rust
// From hexx source:
pub struct Hex {
    /// The `x` axial coordinate (also called `q` in some references)
    pub x: i32,
    /// The `y` axial coordinate (also called `r` in some references)
    pub y: i32,
}
```

- **Coordinate system:** Axial coordinates. Cubic `z` coordinate is derived: `z = -x - y`.
- **Integer-only core:** All coordinate math (neighbors, distance, range, ring, line) operates
  on `i32`. No floating-point contamination in simulation-critical paths.
- **`Hash` implementation:** `Hex` implements `Hash`, enabling `HashMap<Hex, TileData>` storage.
- **`Ord` implementation:** `Hex` implements `Ord` (lexicographic on `(x, y)`), enabling
  `BTreeMap` and sorted iteration for determinism.
- **Serde:** Available via `serde` feature flag. Serializes as `{ "x": i32, "y": i32 }`.

#### 1.2 Coordinate Conversions

| Method | Direction | Notes |
|--------|-----------|-------|
| `to_cubic_array() -> [i32; 3]` | Axial -> Cubic | Returns `[x, y, z]` |
| `from_cubic([i32; 3]) -> Self` | Cubic -> Axial | Drops `z` |
| `to_offset_coordinates(mode) -> [i32; 2]` | Axial -> Offset | Supports Odd-R, Even-R, Odd-Q, Even-Q |
| `from_offset_coordinates([i32; 2], mode) -> Self` | Offset -> Axial | |
| `to_doubled_coordinates(mode) -> [i32; 2]` | Axial -> Doubled | |
| `from_doubled_coordinates([i32; 2], mode) -> Self` | Doubled -> Axial | |
| `to_hexmod_coordinates() -> [i32; 2]` | Axial -> HexMod | |
| `from_hexmod_coordinates([i32; 2]) -> Self` | HexMod -> Axial | |

All conversions are integer-only. No `f32` involved.

#### 1.3 Distance and Neighbor Operations

```rust
impl Hex {
    /// Manhattan distance in hex space. Returns i32.
    pub fn distance_to(self, other: Self) -> i32;

    /// Unsigned distance. Returns u32.
    pub fn unsigned_distance_to(self, other: Self) -> u32;

    /// Returns the neighbor in the given EdgeDirection.
    pub fn neighbor(self, direction: EdgeDirection) -> Self;

    /// Returns all 6 neighbors.
    pub fn all_neighbors(self) -> [Self; 6];

    /// Returns the diagonal neighbor in the given VertexDirection.
    pub fn diagonal_neighbor(self, direction: VertexDirection) -> Self;

    /// Returns all 6 diagonal neighbors.
    pub fn all_diagonals(self) -> [Self; 6];

    /// Identifies the EdgeDirection from self to a neighbor.
    pub fn neighbor_direction(self, other: Self) -> Option<EdgeDirection>;
}
```

All operations return `Hex` (i32 pairs) or primitive integers. Fully deterministic.

#### 1.4 Range, Ring, Line, and Spiral Queries

```rust
impl Hex {
    /// All hexes within `range` steps (inclusive). Returns Vec<Hex>.
    pub fn range(self, range: u32) -> Vec<Self>;

    /// Same as range but excludes center.
    pub fn xrange(self, range: u32) -> Vec<Self>;

    /// All hexes on the ring at exactly `radius` distance.
    pub fn ring(self, radius: u32) -> Vec<Self>;

    /// Hexes arranged in spiral rings from center outward.
    pub fn spiral_range(self, range: u32) -> Vec<Self>;

    /// All hexes along the line from self to other (Bresenham-style).
    pub fn line_to(self, other: Self) -> Vec<Self>;

    /// Two-segment rectilinear path.
    pub fn rectiline_to(self, other: Self) -> Vec<Self>;
}
```

**Determinism note:** `range()`, `ring()`, and `spiral_range()` return hexes in a consistent
order (rings iterate clockwise from a fixed starting direction). `line_to()` uses a
deterministic interpolation algorithm. No `f32` in any of these paths.

#### 1.5 A* Pathfinding

```rust
// In hexx::algorithms module:
pub fn a_star(
    start: Hex,
    end: Hex,
    cost_fn: impl FnMut(Hex, Hex) -> Option<f32>,  // NOTE: f32 cost!
) -> Option<Vec<Hex>>;
```

**Critical finding:** The A* implementation uses `f32` for edge costs. This is problematic
for determinism.

**Mitigation options:**
1. **Wrap with integer costs:** Write a thin wrapper that converts our `i64` movement costs
   to `f32` for `hexx`'s A*. Since A* only compares costs (not accumulates with catastrophic
   cancellation), the `f32` precision loss for typical movement costs (1-1000 range) is
   negligible and produces identical paths. However, this violates our strict "no f32 in
   simulation" rule.
2. **Reimplement A* over `hexx` coordinates:** Use `hexx` for coordinate math (neighbors,
   distance heuristic) but implement our own A* with `i64` costs. This is ~50-80 LOC and
   fully deterministic. **Recommended.**
3. **Contribute upstream:** Submit a PR to `hexx` adding a generic cost type parameter.
   Possible but not guaranteed to be accepted, and blocks our timeline.

**Recommendation:** Option 2. Reimplement A* using `hexx::Hex` for neighbor/heuristic
queries, with integer costs.

#### 1.6 Field of View and Field of Movement

```rust
pub fn field_of_view(
    origin: Hex,
    range: u32,
    blocking_fn: impl FnMut(Hex) -> bool,
) -> HashSet<Hex>;

pub fn field_of_movement(
    origin: Hex,
    budget: f32,  // NOTE: f32 budget!
    cost_fn: impl FnMut(Hex, Hex) -> Option<f32>,
) -> HashMap<Hex, f32>;
```

Same `f32` issue as A*. Same mitigation: reimplement FOV/FOM with integer costs using `hexx`
for coordinate math. FOV's `blocking_fn` is bool-returning and fully integer; only the budget
tracking needs replacement.

#### 1.7 HexLayout (Display Layer)

```rust
pub struct HexLayout {
    pub orientation: HexOrientation,  // Flat-top or Pointy-top
    pub origin: Vec2,                 // f32 pixel offset
    pub hex_size: Vec2,              // f32 pixel size
    pub invert_x: bool,
    pub invert_y: bool,
}

impl HexLayout {
    /// Converts hex coordinates to world pixel position.
    pub fn hex_to_world_pos(&self, hex: Hex) -> Vec2;  // Returns f32 Vec2

    /// Converts world pixel position to hex coordinates.
    pub fn world_pos_to_hex(&self, pos: Vec2) -> Hex;  // Accepts f32, returns i32 Hex
}
```

The `HexLayout` is the **only** part of `hexx` that uses `f32`, and it's purely for
rendering/display. The simulation never touches `HexLayout` -- only the client renderer does.
This is safe and expected.

#### 1.8 Storage Collections

`hexx` provides optimized dense storage types:

| Type | Description | Use Case |
|------|-------------|----------|
| `HexagonalMap<T>` | Dense hexagonal area storage | Fixed-size hex maps |
| `RombusMap<T>` | Dense rhombus-shaped storage | Rectangular regions |
| `HexModMap<T>` | HexMod-addressed storage | Wrapping/tiling maps |

These use array-based indexing (faster than `HashMap`) for known map bounds. All integer-
addressed.

#### 1.9 Cargo Features

```toml
[features]
default = []
serde = ["dep:serde"]        # Serde Serialize/Deserialize
bevy = ["dep:bevy_ecs", "dep:bevy_reflect", "dep:bevy_math"]  # Bevy integration
grid = []                     # Face/Vertex/Edge grid types
rayon = ["dep:rayon"]         # Parallel iteration
mesh = ["dep:glam"]           # Mesh generation (rendering)
```

For the simulation crate: use `serde` only. The `bevy` feature is for the client crate.

### 2. Manual Implementation Assessment

If we built hex grid support from scratch:

| Feature | LOC Estimate | Complexity |
|---------|-------------|------------|
| `Hex` struct + basic ops | ~100 | Low |
| Neighbor/direction lookups | ~80 | Low |
| Distance (Manhattan, unsigned) | ~30 | Low |
| Coordinate conversions (cube, offset, doubled) | ~150 | Medium |
| Range/ring/spiral queries | ~200 | Medium |
| Line drawing (Bresenham hex) | ~60 | Medium |
| A* pathfinding (integer) | ~80 | Medium |
| FOV (shadowcasting hex) | ~150 | High |
| Field of movement (Dijkstra) | ~80 | Medium |
| Dense storage collections | ~200 | Medium |
| Serde implementations | ~50 | Low |
| Tests | ~400 | Medium |
| **Total** | **~1580** | -- |

**Maintenance burden:** Every new hex algorithm (wedge queries, hex region intersection,
multi-resolution) would need to be implemented and tested from scratch. `hexx` already has
these and is community-maintained.

**Risk:** Manual implementations of hex math have subtle edge cases (negative coordinates,
boundary conditions, wrap-around). `hexx` has been battle-tested by the Bevy gamedev community.

### 3. Alternative Libraries

| Crate | Status | Notes |
|-------|--------|-------|
| `hexagonal` | Unmaintained (last update 2020) | Incomplete API, no pathfinding |
| `hex2d` | Low activity | Uses `i32` but minimal features |
| `hexing` | New (2024) | Small, missing many features |
| `hexgridspiral` | Niche | Spiral-only, not general purpose |

None are competitive with `hexx` in feature completeness or maintenance.

---

## Decision

**Use `hexx` version 0.21.x** with the following strategy:

1. **Use `hexx::Hex` as the canonical coordinate type** throughout the simulation. All tile
   positions, unit positions, and territory boundaries use `Hex`.
2. **Use `hexx`'s integer operations directly:** neighbors, distance, range, ring, spiral,
   line drawing, coordinate conversions.
3. **Reimplement pathfinding with integer costs:** Write CivLab-specific A*, FOV, and
   field-of-movement algorithms that use `hexx::Hex` for coordinate math but `i64` for cost
   accumulation. This is ~200 LOC total.
4. **Use `HexLayout` only in the client renderer**, never in the simulation crate.
5. **Use `HexagonalMap<T>` or `HashMap<Hex, T>`** for tile data storage, depending on whether
   the map has fixed bounds.

---

## Implementation Contract

### Cargo.toml (simulation crate)

```toml
[dependencies]
hexx = { version = "0.21", default-features = false, features = ["serde"] }
```

### Cargo.toml (client crate)

```toml
[dependencies]
hexx = { version = "0.21", features = ["serde", "bevy", "mesh"] }
```

### Canonical Coordinate Type

```rust
// In crates/engine/src/hex.rs or similar

/// Re-export hexx::Hex as the canonical coordinate type.
/// All simulation code uses this type for tile/entity positions.
pub use hexx::Hex;

/// Re-export direction types.
pub use hexx::{EdgeDirection, VertexDirection};

/// Re-export coordinate conversion modes.
pub use hexx::OffsetHexMode;
```

### Map Storage Pattern

```rust
use hexx::Hex;
use std::collections::HashMap;

/// Tile data for the simulation. Each hex cell has associated data.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TileData {
    pub terrain: TerrainType,
    pub elevation: i32,          // meters, integer
    pub moisture: i32,           // 0-1000 scale (fixed-point-like)
    pub movement_cost: i64,      // base movement cost to enter this tile
    pub owner: Option<NationId>,
}

/// The hex map: HashMap<Hex, TileData>.
/// For fixed-size maps, consider hexx::HexagonalMap<TileData> for better perf.
pub type HexMap = HashMap<Hex, TileData>;
```

### Integer A* Pathfinding

```rust
use hexx::Hex;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

/// A* pathfinding with integer costs. Uses hexx for coordinate math.
/// Returns None if no path exists.
pub fn a_star_integer(
    start: Hex,
    goal: Hex,
    cost_fn: impl Fn(Hex, Hex) -> Option<i64>,
) -> Option<(Vec<Hex>, i64)> {
    // Priority queue: (cost, hex). Use Reverse for min-heap.
    let mut open: BinaryHeap<Reverse<(i64, Hex)>> = BinaryHeap::new();
    let mut came_from: HashMap<Hex, Hex> = HashMap::new();
    let mut g_score: HashMap<Hex, i64> = HashMap::new();

    g_score.insert(start, 0);
    let h = heuristic(start, goal);
    open.push(Reverse((h, start)));

    while let Some(Reverse((_, current))) = open.pop() {
        if current == goal {
            return Some(reconstruct_path(&came_from, current, &g_score));
        }

        let current_g = g_score[&current];

        // Use hexx's all_neighbors() for neighbor enumeration
        for neighbor in current.all_neighbors() {
            if let Some(edge_cost) = cost_fn(current, neighbor) {
                let tentative_g = current_g + edge_cost;
                if tentative_g < *g_score.get(&neighbor).unwrap_or(&i64::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    let f = tentative_g + heuristic(neighbor, goal);
                    open.push(Reverse((f, neighbor)));
                }
            }
        }
    }
    None
}

/// Heuristic: hex Manhattan distance scaled to minimum movement cost.
/// Uses hexx's built-in distance_to (integer).
fn heuristic(a: Hex, b: Hex) -> i64 {
    a.unsigned_distance_to(b) as i64 * MIN_MOVEMENT_COST
}

const MIN_MOVEMENT_COST: i64 = 100; // Minimum possible tile cost

fn reconstruct_path(
    came_from: &HashMap<Hex, Hex>,
    mut current: Hex,
    g_score: &HashMap<Hex, i64>,
) -> (Vec<Hex>, i64) {
    let total_cost = g_score[&current];
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    (path, total_cost)
}
```

### Integer Field of View

```rust
use hexx::Hex;
use std::collections::HashSet;

/// Field of view using hexx coordinate math with boolean blocking.
/// No f32 involved -- purely geometric visibility.
pub fn field_of_view_integer(
    origin: Hex,
    range: u32,
    is_blocking: impl Fn(Hex) -> bool,
) -> HashSet<Hex> {
    let mut visible = HashSet::new();
    visible.insert(origin);

    // Ray-cast from origin to each hex on the outer ring
    for target in origin.ring(range) {
        let line = origin.line_to(target);
        for hex in line {
            if hex == origin {
                continue;
            }
            visible.insert(hex);
            if is_blocking(hex) {
                break; // Stop ray at blocking tile
            }
        }
    }
    visible
}
```

### Viewport Culling (AABB to Hex Range)

```rust
use hexx::{Hex, HexLayout};

/// Convert a screen-space AABB to the set of visible hex coordinates.
/// This is CLIENT-ONLY code (uses f32 via HexLayout).
/// Never call from simulation.
pub fn viewport_to_hex_range(
    layout: &HexLayout,
    viewport_min: glam::Vec2,  // top-left pixel
    viewport_max: glam::Vec2,  // bottom-right pixel
) -> Vec<Hex> {
    // Convert corners to hex coordinates
    let hex_min = layout.world_pos_to_hex(viewport_min);
    let hex_max = layout.world_pos_to_hex(viewport_max);

    // Compute bounding range and collect all hexes in the rectangle
    let range_x = hex_min.x.min(hex_max.x)..=hex_min.x.max(hex_max.x);
    let range_y = hex_min.y.min(hex_max.y)..=hex_min.y.max(hex_max.y);

    let mut result = Vec::new();
    for x in range_x {
        for y in range_y.clone() {
            result.push(Hex::new(x, y));
        }
    }
    result
}
```

### Determinism Guarantee

```rust
/// All hex operations in the simulation crate MUST satisfy:
/// 1. No f32/f64 in any computation
/// 2. All neighbor iterations use hexx's fixed EdgeDirection ordering
/// 3. All range/ring queries return hexes in hexx's documented order
/// 4. Pathfinding uses integer costs only
/// 5. Map iteration uses BTreeMap<Hex, _> or sorted HashMap keys
///
/// The HexLayout type is FORBIDDEN in the simulation crate.
/// It is only used in the client renderer crate.
#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn neighbor_order_is_deterministic() {
        let hex = Hex::new(5, -3);
        let neighbors_a = hex.all_neighbors();
        let neighbors_b = hex.all_neighbors();
        assert_eq!(neighbors_a, neighbors_b);
    }

    #[test]
    fn range_order_is_deterministic() {
        let hex = Hex::new(0, 0);
        let range_a = hex.range(10);
        let range_b = hex.range(10);
        assert_eq!(range_a, range_b);
    }

    #[test]
    fn pathfinding_is_deterministic() {
        let start = Hex::new(0, 0);
        let goal = Hex::new(10, -5);
        let cost_fn = |_from: Hex, _to: Hex| -> Option<i64> { Some(100) };

        let path_a = a_star_integer(start, goal, cost_fn);
        let path_b = a_star_integer(start, goal, cost_fn);
        assert_eq!(path_a, path_b);
    }
}
```

---

## Open Questions Remaining

1. **`hexx` Bevy version compatibility:** `hexx` 0.21 targets a specific Bevy version for
   its `bevy` feature. Verify compatibility with `bevy 0.18` or whatever version the client
   uses. The simulation crate doesn't use the `bevy` feature, so this only affects the client.

2. **Map wrapping / toroidal geometry:** CivLab may want a wrapping world map (cylindrical
   or toroidal). `hexx` doesn't natively support wrapping. Need to implement modular
   coordinate wrapping on top of `Hex`. Estimate: ~100 LOC for cylindrical wrapping.

3. **Multi-resolution hex grids:** `hexx` supports `to_lower_res()` and `to_higher_res()`
   for multi-resolution coordinates. Evaluate if this is useful for CivLab's zoom levels
   or strategic map view. If used, ensure the resolution conversion is deterministic (it
   should be, as it's purely integer math).

4. **Performance of `HashMap<Hex, TileData>`:** For a 100x100 hex map (10k tiles), `HashMap`
   is fine. For larger maps (1M+ tiles), `HexagonalMap` (dense array) will be significantly
   faster. Profile and decide based on actual map sizes.

5. **A* performance budget:** The integer A* implementation uses `BinaryHeap` which is
   O(n log n). For long paths across large maps (>1000 tiles), consider jump-point search
   or hierarchical pathfinding. This is an optimization concern, not a correctness concern.

---

## References

- [hexx crates.io](https://crates.io/crates/hexx)
- [hexx docs.rs](https://docs.rs/hexx/latest/hexx/)
- [hexx GitHub](https://github.com/ManevilleF/hexx)
- [Hex struct API](https://docs.rs/hexx/latest/hexx/struct.Hex.html)
- [Red Blob Games - Hexagonal Grids](https://www.redblobgames.com/grids/hexagons/) (canonical reference)
- [hexx algorithms module](https://docs.rs/hexx/latest/hexx/algorithms/index.html)


---

## Source: research/RND-003-fixed-point-decision.md

# RND-003: Fixed-Point Arithmetic -- `fixed` Crate vs Manual i64 x SCALE

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**Recommendation: Hybrid approach.** Use three numeric strategies depending on the domain:

1. **`i64` with `SCALE = 1_000_000`** for large-magnitude values (energy in Joules, population
   resources, GDP) where `I32F32` would overflow.
2. **`fixed` crate `I32F32`** (via `FixedI32<U32>` or more precisely `FixedI32<U16>` for
   range) for ratio/rate values (growth rates, efficiency percentages, tax rates, happiness
   scores) where the values stay in a bounded range and operator ergonomics matter.
3. **`cordic` crate** for trigonometric functions needed by the climate/solar angle subsystem,
   operating on `fixed` types.

The `fixed` crate (v1.30.x) provides `no_std` support, serde, and operator overloading but
**no built-in trig functions**. The `cordic` crate (v0.3.x) fills this gap with CORDIC-based
`sin`, `cos`, `atan2` for fixed-point types. For Joule-scale values, `I32F32` overflows at
~2.1 billion (2^31) which is insufficient for 100k citizens x 1 TJ = 10^17 Joules. These
values must use `i64 x SCALE` where the i64 range (+-9.2 x 10^18) accommodates even extreme
scenarios.

---

## Research Findings

### 1. The Determinism Requirement

CivLab's simulation must be bit-for-bit deterministic across:
- Different machines (x86_64, aarch64, WASM)
- Different Rust compiler versions
- Multiple runs with identical inputs

**Why no `f32`/`f64`:**
- IEEE 754 permits implementation-defined behavior for NaN payloads.
- FMA (fused multiply-add) produces different results than separate mul+add.
- x87 FPU uses 80-bit extended precision internally; SSE uses 32/64-bit. Mixing yields
  different results.
- WASM uses IEEE 754 strictly but may differ from native due to FMA availability.
- Compiler optimizations (e.g., fast-math, reassociation) can change float results.
- The Rust language does not guarantee deterministic float operations across platforms.

**Conclusion:** All simulation arithmetic must use integer or fixed-point types.

### 2. `fixed` Crate Analysis (v1.30.0)

**Repository:** [gitlab.com/tspiteri/fixed](https://gitlab.com/tspiteri/fixed)
**License:** MIT/Apache-2.0
**MSRV:** Rust 1.85.0

#### 2.1 Available Types

| Type | Bits | Signed | Fractional Bits | Integer Range | Fractional Precision |
|------|------|--------|-----------------|---------------|---------------------|
| `FixedI8<UX>` | 8 | Yes | 0-8 | Depends on X | Depends on X |
| `FixedI16<UX>` | 16 | Yes | 0-16 | Depends on X | Depends on X |
| `FixedI32<UX>` | 32 | Yes | 0-32 | Depends on X | Depends on X |
| `FixedI64<UX>` | 64 | Yes | 0-64 | Depends on X | Depends on X |
| `FixedI128<UX>` | 128 | Yes | 0-128 | Depends on X | Depends on X |
| `FixedU*` variants | * | No | * | * | * |

**Key configurations for CivLab:**

| Type Alias | Type | Integer Bits | Frac Bits | Integer Range | Precision |
|-----------|------|-------------|-----------|---------------|-----------|
| `I32F32` | Not a real type -- `FixedI32<U32>` has 0 integer bits! | 0 | 32 | -0.5..0.5 | ~2.3e-10 |
| `FixedI32<U16>` | 32-bit | 16 | 16 | -32768..32767 | ~1.5e-5 |
| `FixedI32<U8>` | 32-bit | 24 | 8 | -8388608..8388607 | ~0.004 |
| `FixedI64<U32>` | 64-bit | 32 | 32 | -2^31..2^31-1 | ~2.3e-10 |

**IMPORTANT CORRECTION:** The notation "I32F32" commonly refers to a 64-bit type with 32
integer bits and 32 fractional bits -- i.e., `FixedI64<U32>`. A `FixedI32<U32>` has zero
integer bits (range -0.5 to 0.5), which is useless for most purposes. CivLab should use:
- `FixedI64<U32>` for "I32F32" semantics (32 int + 32 frac, 64 bits total)
- `FixedI32<U16>` for "I16F16" semantics (16 int + 16 frac, 32 bits total)

#### 2.2 Overflow Analysis

**Scenario: 100k citizens x 1 TJ (10^12 Joules)**

Total energy = 100,000 x 10^12 = 10^17 Joules.

| Type | Max Value | Overflows? |
|------|-----------|------------|
| `FixedI32<U16>` (I16F16) | 32,767 | YES -- overflows at 32k |
| `FixedI64<U32>` (I32F32) | ~2.15 x 10^9 | YES -- overflows at 2.1 billion |
| `i64 x SCALE(10^6)` | ~9.2 x 10^12 | YES if SCALE=10^6, max representable = 9.2e12 |
| `i64` (raw, no scale) | ~9.2 x 10^18 | NO -- 10^17 fits comfortably |
| `FixedI128<U32>` | ~1.7 x 10^29 | NO -- but 128-bit math is slow |

**Finding:** For Joule-scale values, even `FixedI64<U32>` overflows. The only viable options
are:
1. **`i64` with reduced scale (SCALE=1000 or SCALE=100):** Max = 9.2e15 or 9.2e16 --
   sufficient for 10^17 with SCALE=100.
2. **`i64` without scale (integer Joules):** If we don't need sub-Joule precision for energy.
   Most energy calculations don't need fractional Joules.
3. **`i128`:** Overkill and slower on 32-bit / WASM targets.
4. **Domain-specific units:** Store energy in kJ or MJ instead of J. 10^17 J = 10^14 kJ =
   10^11 MJ. `FixedI64<U32>` handles 10^11 MJ fine (max ~2.1e9 with fraction... still tight).
   10^11 MJ > 2.1e9. Still overflows.
5. **`i64` with SCALE = 1_000 (milliJoules? No -- scale for sub-unit precision):**
   For energy: use raw `i64` Joules (no fractional precision needed).
   For rates: use `FixedI32<U16>` or `FixedI64<U32>`.

**Recommendation:** Energy values use plain `i64` (integer Joules or kiloJoules). No scaling
needed because sub-Joule precision is unnecessary for a civilization simulation. Rates and
ratios use `fixed` types.

#### 2.3 Arithmetic Operations

The `fixed` crate provides full operator overloading:

```rust
use fixed::types::extra::U16;
use fixed::FixedI32;

type Fix = FixedI32<U16>;

let a = Fix::from_num(3.5);
let b = Fix::from_num(2.0);
let c = a + b;          // 5.5
let d = a * b;          // 7.0
let e = a / b;          // 1.75
let f = a % b;          // 1.5
let g = -a;             // -3.5
let cmp = a > b;        // true
```

**Overflow behavior:**
- Default operations (`+`, `-`, `*`, `/`) panic on overflow in debug, wrap in release.
- `checked_*` variants return `Option<Self>`.
- `saturating_*` variants clamp to min/max.
- `wrapping_*` variants explicitly wrap.
- `strict_*` (renamed from `unwrapped_*` in v1.30) always panic on overflow.

**CivLab policy:** Use `checked_*` operations in simulation code, with explicit error
propagation. Overflow in a simulation is a logic bug that should be detected and reported,
not silently wrapped.

#### 2.4 Conversion Operations

```rust
// From integer:
let x = Fix::from_num(42);         // 42.0
let x = Fix::from_num(42_i64);     // 42.0

// From float (compile-time only for determinism):
let x = Fix::lit("3.14159");       // Exact binary representation of closest value

// To integer:
let n: i32 = x.to_num();           // Truncates toward zero
let n: i32 = x.round().to_num();   // Rounds to nearest

// To/from raw bits:
let bits: i32 = x.to_bits();       // Raw bit representation
let x = Fix::from_bits(bits);      // From raw bits (for serialization)
```

#### 2.5 Features and Compatibility

```toml
[dependencies]
fixed = { version = "1.30", features = ["serde"] }  # optional serde
```

- `no_std` by default (only `serde-str` requires `std`)
- `serde` feature: serializes as the numeric value (not raw bits)
- WASM compatible: all operations are pure integer math
- `#[repr(transparent)]` over the underlying integer type -- same layout as `i32`/`i64`

### 3. `cordic` Crate Analysis (v0.3.x)

**Repository:** [github.com/sebcrozet/cordic](https://github.com/sebcrozet/cordic)
**License:** BSD-3-Clause
**Dependency:** `fixed = "^1"` (compatible with our version)

#### 3.1 Available Functions

```rust
use cordic;
use fixed::types::extra::U16;
use fixed::FixedI32;

type Fix = FixedI32<U16>;

// Trigonometric
let angle = Fix::from_num(1.0);  // 1 radian
let s = cordic::sin(angle);       // sine
let c = cordic::cos(angle);       // cosine
let (s, c) = cordic::sin_cos(angle);  // both at once (faster)
let t = cordic::tan(angle);       // tangent

// Inverse trigonometric
let a = cordic::asin(Fix::from_num(0.5));  // arcsine
let a = cordic::acos(Fix::from_num(0.5));  // arccosine
let a = cordic::atan(Fix::from_num(1.0));  // arctangent
let a = cordic::atan2(y, x);              // atan2 with quadrant correction

// Other
let r = cordic::sqrt(Fix::from_num(2.0));  // square root
```

#### 3.2 Precision

CORDIC achieves precision proportional to the number of fractional bits:
- With 16 fractional bits (`FixedI32<U16>`): ~4-5 decimal digits of precision
- With 32 fractional bits (`FixedI64<U32>`): ~9-10 decimal digits of precision

For climate angle calculations (solar altitude, latitude effects), 4-5 digits of precision
is more than sufficient. The sun angle doesn't need sub-arcsecond precision for a civilization
game.

#### 3.3 Determinism

CORDIC algorithms are fully deterministic:
- Pure integer arithmetic internally (shift-and-add)
- Lookup table based (compile-time generated)
- No floating-point operations
- Platform-independent results for the same input type

#### 3.4 Alternative: `fixed_trigonometry` Crate

```toml
[dependencies]
fixed_trigonometry = "0.3"  # Alternative to cordic
```

- Also provides `sin`, `cos`, `tan`, `atan2` for `fixed` types
- `no_std` compatible
- Uses polynomial approximation rather than CORDIC iterations
- Similar precision (~4-5 digits for 16 frac bits)

Either `cordic` or `fixed_trigonometry` works. `cordic` is more established and has `sqrt`.

### 4. Manual `i64 x SCALE` Approach

The manual approach uses plain `i64` with a constant scale factor:

```rust
/// Scale factor: 1_000_000 = 10^6
/// This gives 6 decimal digits of fractional precision.
const SCALE: i64 = 1_000_000;

/// A "fixed-point" value is just an i64 where the real value = raw / SCALE.
type Scaled = i64;

fn from_int(n: i64) -> Scaled { n * SCALE }
fn from_frac(n: i64, d: i64) -> Scaled { n * SCALE / d }
fn to_int(s: Scaled) -> i64 { s / SCALE }
fn mul(a: Scaled, b: Scaled) -> Scaled { a * b / SCALE }
fn div(a: Scaled, b: Scaled) -> Scaled { a * SCALE / b }
```

**Pros:**
- Zero dependencies
- Full control over scale factor
- Easy to understand and debug
- `i64` range: +-9.2 x 10^18 / SCALE = +-9.2 x 10^12 representable values

**Cons:**
- **No operator overloading:** Must use `mul(a, b)` instead of `a * b`. Every arithmetic
  expression becomes verbose and error-prone.
- **Scale factor discipline:** Every multiplication must divide by SCALE, every division must
  multiply by SCALE. Forgetting produces wrong results silently.
- **Overflow risk in intermediates:** `a * b` can overflow `i64` before the `/ SCALE`
  normalization. Need `i128` intermediates or careful ordering.
- **No type safety:** A `Scaled` value and a raw `i64` are the same type. Can accidentally
  mix them (e.g., add a scaled value to an unscaled one).
- **No trig functions:** Must implement CORDIC or polynomial approximation from scratch.

### 5. Comparison Summary

| Criterion | `fixed` crate | Manual i64 x SCALE | Winner |
|-----------|---------------|---------------------|--------|
| Type safety | Strong (distinct type) | None (just i64) | `fixed` |
| Operator overloading | Yes (`+`, `-`, `*`, `/`) | No (function calls) | `fixed` |
| Overflow detection | `checked_*` variants | Manual | `fixed` |
| Range for energy | FixedI64<U32>: +-2.1e9 | i64: +-9.2e12 (SCALE=10^6) | Manual |
| Trig functions | Via `cordic` | Must implement | `fixed` |
| Serde | Built-in feature | Manual | `fixed` |
| no_std | Yes | Yes | Tie |
| WASM compat | Yes | Yes | Tie |
| Dependencies | 1 crate | 0 crates | Manual |
| Ergonomics | Excellent | Poor | `fixed` |
| Precision control | Per-type (U8, U16, U32) | Per-constant (SCALE) | `fixed` |
| Debuggability | `.to_num::<f64>()` for display | `raw / SCALE` | Tie |

---

## Decision

**Hybrid approach: domain-specific numeric types.**

### Domain 1: Large-Magnitude Values (Energy, Resources, GDP)

**Use `i64` with domain-specific units, no fractional scaling.**

- Energy: store in **kiloJoules (kJ)** as raw `i64`. Range: +-9.2e18 kJ = +-9.2e21 J.
  Even 10^17 J = 10^14 kJ, well within range.
- Resources (food, minerals): store in **grams** or **kilograms** as raw `i64`.
- GDP / currency: store in **milli-credits** as raw `i64`.

No need for `SCALE` constant -- the unit itself provides the precision. This avoids the
error-prone manual scaling math.

Newtype wrappers for type safety:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KiloJoules(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MilliCredits(pub i64);
```

### Domain 2: Ratios, Rates, Bounded Values

**Use `fixed` crate types.**

| Value | Type | Range | Precision | Justification |
|-------|------|-------|-----------|---------------|
| Growth rate | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | Rates are small numbers (0.01-0.10 typical) |
| Tax rate | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | 0.0-1.0 range |
| Happiness | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | 0.0-100.0 range |
| Efficiency | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | 0.0-1.0 multiplier |
| Temperature | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | Kelvin (200-400 typical) |
| Latitude/angle | `FixedI32<U16>` | -32768..32767 | ~1.5e-5 | Radians (-pi to pi) |

### Domain 3: Trigonometric Computations

**Use `cordic` crate with `FixedI32<U16>` inputs.**

Only needed for:
- Solar angle calculation (climate system)
- Latitude-based temperature modifiers
- Possibly orbital mechanics if the game includes planetary features

---

## Implementation Contract

### Cargo.toml

```toml
[dependencies]
fixed = { version = "1.30", features = ["serde"] }
cordic = "0.3"
```

### Type Aliases Module

```rust
// In crates/engine/src/numeric.rs

use fixed::types::extra::{U16, U32};
use fixed::{FixedI32, FixedI64};

// === Domain 1: Large Magnitudes (i64 newtypes) ===

/// Energy in kiloJoules. Range: +-9.2e18 kJ.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct KiloJoules(pub i64);

/// Currency in milli-credits (1 credit = 1000 milli-credits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct MilliCredits(pub i64);

/// Mass in grams. Range: +-9.2e18 grams = +-9.2e15 kg.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Grams(pub i64);

/// Population count. Plain integer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Population(pub i64);

// Implement basic arithmetic for newtypes via macro:
macro_rules! impl_i64_newtype_ops {
    ($T:ident) => {
        impl std::ops::Add for $T {
            type Output = Self;
            fn add(self, rhs: Self) -> Self { $T(self.0.checked_add(rhs.0).expect(concat!(stringify!($T), " overflow"))) }
        }
        impl std::ops::Sub for $T {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self { $T(self.0.checked_sub(rhs.0).expect(concat!(stringify!($T), " overflow"))) }
        }
        // Multiply by scalar (not by same type -- that changes units)
        impl std::ops::Mul<i64> for $T {
            type Output = Self;
            fn mul(self, rhs: i64) -> Self { $T(self.0.checked_mul(rhs).expect(concat!(stringify!($T), " overflow"))) }
        }
        impl std::ops::Div<i64> for $T {
            type Output = Self;
            fn div(self, rhs: i64) -> Self { $T(self.0.checked_div(rhs).expect(concat!(stringify!($T), " division"))) }
        }
    };
}

impl_i64_newtype_ops!(KiloJoules);
impl_i64_newtype_ops!(MilliCredits);
impl_i64_newtype_ops!(Grams);
impl_i64_newtype_ops!(Population);

// === Domain 2: Ratios and Rates (fixed-point) ===

/// 32-bit fixed-point with 16 integer bits and 16 fractional bits.
/// Range: -32768.0 to ~32767.99998. Precision: ~0.0000153.
/// Use for: rates, ratios, percentages, temperatures, small multipliers.
pub type Ratio = FixedI32<U16>;

/// 64-bit fixed-point with 32 integer bits and 32 fractional bits.
/// Range: -2,147,483,648 to ~2,147,483,647.999. Precision: ~2.3e-10.
/// Use for: high-precision rates where I16F16 is insufficient.
pub type HiRatio = FixedI64<U32>;

// === Domain 3: Angles (fixed-point with trig) ===

/// Angle in radians, stored as FixedI32<U16>.
/// Range: -32768..32767 radians (way more than 2*pi).
/// Precision: ~0.0000153 radians (~0.0009 degrees).
pub type Angle = FixedI32<U16>;

/// Compute sine of an angle. Fully deterministic (CORDIC algorithm).
pub fn sin(angle: Angle) -> Ratio {
    cordic::sin(angle)
}

/// Compute cosine of an angle. Fully deterministic.
pub fn cos(angle: Angle) -> Ratio {
    cordic::cos(angle)
}

/// Compute both sine and cosine. More efficient than calling each separately.
pub fn sin_cos(angle: Angle) -> (Ratio, Ratio) {
    cordic::sin_cos(angle)
}

/// Compute arctangent with quadrant correction.
pub fn atan2(y: Ratio, x: Ratio) -> Angle {
    cordic::atan2(y, x)
}

/// Compute square root. Input must be non-negative.
pub fn sqrt(value: Ratio) -> Ratio {
    cordic::sqrt(value)
}
```

### Usage Examples

#### Climate System (Trig)

```rust
use crate::numeric::{Angle, Ratio, sin, cos, sin_cos};
use fixed::traits::FromFixed;

/// Calculate solar altitude angle based on latitude and day-of-year.
/// All computations are fixed-point. No f32/f64.
pub fn solar_altitude(
    latitude: Angle,     // radians
    day_of_year: i32,    // 1-365
    hour: i32,           // 0-23
) -> Angle {
    // Earth's axial tilt: ~23.44 degrees = 0.4091 radians
    let tilt = Angle::lit("0.4091");

    // Declination angle: delta = tilt * sin(2*pi*(day-80)/365)
    let two_pi = Angle::lit("6.2832");
    let day_angle = two_pi * (day_of_year - 80) / 365;
    let declination = Ratio::from_num(tilt) * sin(day_angle);
    let declination = Angle::from_num(declination);

    // Hour angle: omega = pi/12 * (hour - 12)
    let hour_angle = Angle::lit("0.2618") * (hour - 12); // pi/12 ~= 0.2618

    // Solar altitude: sin(alt) = sin(lat)*sin(dec) + cos(lat)*cos(dec)*cos(ha)
    let (sin_lat, cos_lat) = sin_cos(latitude);
    let (sin_dec, cos_dec) = sin_cos(declination);
    let cos_ha = cos(hour_angle);

    let sin_alt = sin_lat * sin_dec + cos_lat * cos_dec * cos_ha;

    // Return altitude as angle (asin would be needed for exact angle,
    // but sin_alt as a ratio is sufficient for temperature modifiers)
    Angle::from_num(sin_alt)
}
```

#### Economy System (Large Values)

```rust
use crate::numeric::{KiloJoules, MilliCredits, Ratio};

/// Calculate food production for a tile.
/// Energy uses KiloJoules (i64), efficiency uses Ratio (fixed-point).
pub fn calculate_food_production(
    base_energy: KiloJoules,
    soil_fertility: Ratio,    // 0.0 - 1.0
    technology_bonus: Ratio,  // 1.0 = no bonus, 1.5 = 50% bonus
) -> KiloJoules {
    // Multiply base energy by fertility ratio
    // fixed * i64 -> i64: convert ratio to scaled integer multiplication
    let fertility_scaled = soil_fertility.to_num::<i64>();  // Truncates to integer
    // Better: use intermediate fixed-point for the multiplication
    let base_ratio = Ratio::from_num(base_energy.0 / 1000); // Scale down to fit Ratio
    let production_ratio = base_ratio * soil_fertility * technology_bonus;
    let production_kj = production_ratio.to_num::<i64>() * 1000; // Scale back up

    KiloJoules(production_kj)
}
```

#### Mixing Domains Safely

```rust
use crate::numeric::{KiloJoules, Ratio, Population};

/// Calculate per-capita energy consumption.
/// Returns a Ratio (energy per person per tick), not a KiloJoules.
pub fn per_capita_consumption(
    total_energy: KiloJoules,
    population: Population,
) -> Ratio {
    // i64 / i64 -> Ratio: careful to preserve precision
    // Option 1: If total_energy.0 / population.0 fits in Ratio range
    if population.0 == 0 {
        return Ratio::ZERO;
    }
    // Use i64 division with remainder for precision
    let quotient = total_energy.0 / population.0;
    let remainder = total_energy.0 % population.0;
    Ratio::from_num(quotient)
        + Ratio::from_num(remainder) / Ratio::from_num(population.0.min(i32::MAX as i64))
}
```

### Clippy Lint Configuration

```rust
// In crates/engine/src/lib.rs or build.rs:
//
// Deny all floating-point usage in the simulation crate.
#![deny(clippy::float_arithmetic)]
#![deny(clippy::float_cmp)]
#![deny(clippy::float_cmp_const)]
// These lints catch accidental f32/f64 usage at compile time.
```

### Serialization Contract

```rust
/// All numeric types serialize deterministically:
/// - i64 newtypes: serialize as JSON numbers (integer)
/// - Ratio/HiRatio/Angle: serialize as JSON numbers (decimal via serde feature)
///
/// For binary serialization (network/snapshot):
/// - i64 newtypes: little-endian i64 bytes
/// - Fixed-point: .to_bits() -> i32/i64 -> little-endian bytes
///
/// NEVER serialize fixed-point via to_num::<f64>() -- this introduces f64
/// and loses precision. Always use to_bits() for binary formats.

#[cfg(test)]
mod serde_tests {
    use super::*;

    #[test]
    fn fixed_point_roundtrips_via_bits() {
        let original = Ratio::lit("3.14159");
        let bits = original.to_bits();
        let restored = Ratio::from_bits(bits);
        assert_eq!(original, restored);
    }

    #[test]
    fn kj_roundtrips_via_json() {
        let original = KiloJoules(1_000_000_000);
        let json = serde_json::to_string(&original).unwrap();
        let restored: KiloJoules = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}
```

### Cross-Domain Conversion Safety Rules

```text
RULES FOR NUMERIC DOMAIN CROSSING:

1. i64 newtype -> Ratio:
   - Only when value fits in Ratio range (-32768..32767).
   - Scale down first if needed (e.g., divide kJ by 1000 before converting).
   - Use checked conversion: Ratio::checked_from_num(value).expect("overflow")

2. Ratio -> i64 newtype:
   - Use .to_num::<i64>() which truncates toward zero.
   - Or .round().to_num::<i64>() for rounding.
   - Wrap in appropriate newtype: KiloJoules(ratio.to_num())

3. Ratio * i64 newtype:
   - Convert the i64 to Ratio first (if in range), multiply, convert back.
   - OR: multiply i64 by ratio's numerator, divide by denominator.
   - Example: kj.0 * ratio.to_bits() as i64 / (1 << 16)

4. FORBIDDEN:
   - Never convert to f32/f64 for intermediate computation.
   - Never use .to_num::<f32>() in simulation code.
   - Never construct Ratio from runtime f32 values.
   - Compile-time float literals: use Ratio::lit("3.14") (parsed at compile time).
```

---

## Open Questions Remaining

1. **`FixedI32<U16>` vs `FixedI64<U32>` for Ratio:** The current recommendation uses
   `FixedI32<U16>` for most rates. If any rate computation involves multiplication of two
   rates (rate * rate), the intermediate product may lose significant precision with only
   16 fractional bits. Profiling needed to determine if `FixedI64<U32>` is needed for any
   hot paths. Preliminary recommendation: start with `FixedI32<U16>`, upgrade to
   `FixedI64<U32>` only for domains where precision matters (e.g., compound interest over
   many ticks).

2. **Overflow handling policy:** The contract specifies `checked_*` operations that panic
   on overflow. In production, panicking in the simulation is a hard crash. Alternative:
   use `saturating_*` for non-critical values (e.g., happiness saturates at max rather than
   crashing). Need to define per-domain overflow policy. Preliminary: panic on financial
   overflow (indicates a game balance bug), saturate on cosmetic values.

3. **Compile-time float literal precision:** `Ratio::lit("3.14159")` rounds to the nearest
   representable fixed-point value. With U16 fractional bits, `3.14159` becomes `3.14154...`
   (error ~5e-5). For game-critical constants (PI, e, tilt angle), verify that the rounded
   values are acceptable. They should be -- the game doesn't need 10-digit precision for
   any physical constant.

4. **Performance of `cordic` vs lookup table:** For trig functions called every tick for every
   tile (climate system), CORDIC's iterative algorithm may be slower than a precomputed lookup
   table. For 16-bit precision, a 65536-entry lookup table for sin/cos is only 256KB and gives
   O(1) lookup. Profile both approaches. `cordic` is the safer starting point.

5. **Interaction with MCTS rollouts:** MCTS (RND-011) will run lightweight forward simulations.
   These must use the same numeric types for determinism. Verify that the simplified rollout
   model can operate efficiently with fixed-point types (no conversion overhead in hot loops).

---

## References

- [fixed crate docs.rs](https://docs.rs/fixed/latest/fixed/)
- [fixed crate crates.io](https://crates.io/crates/fixed)
- [cordic crate docs.rs](https://docs.rs/cordic/latest/cordic/)
- [fixed_trigonometry crate](https://crates.io/crates/fixed_trigonometry)
- [Working with Fixed-Point Numbers in Rust](https://blog.implrust.com/posts/2025/12/fixed-point-crate-in-rust/)
- [CORDIC algorithm (Wikipedia)](https://en.wikipedia.org/wiki/CORDIC)
- [Deterministic lockstep networking](https://gafferongames.com/post/deterministic_lockstep/)
- [IEEE 754 cross-platform issues](https://randomascii.wordpress.com/2013/07/16/floating-point-determinism/)


---

## Source: research/RND-004-web-renderer-decision.md

# RND-004: Web Renderer Decision — Pixi.js v8 + React 19 for CivLab Web Client

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

Pixi.js v8 is the recommended 2D web renderer for CivLab. It provides WebGPU-first rendering with automatic WebGL2 fallback, native React 19 integration via `@pixi/react`, GPU-instanced particle rendering capable of 1M+ sprites at 60fps, and a mature TypeScript-first API. For the Phase 2 3D upgrade path, Babylon.js can coexist on the same page via shared WebGL context or separate canvas layering. The decision is Pixi.js v8 + React 19 for 2D, with a clean `IRenderer` interface contract enabling future Babylon.js 3D substitution.

---

## Research Findings

### 1. Pixi.js v8 Architecture and Performance

#### WebGPU-First with WebGL Fallback

Pixi.js v8 represents a ground-up rewrite that targets WebGPU as the primary rendering backend, with automatic fallback to WebGL2 on browsers that do not yet support WebGPU. This is configured via the renderer preference system:

```typescript
import { Application, autoDetectRenderer } from 'pixi.js';

// Auto-detect best available renderer
const app = new Application();
await app.init({
    preference: 'webgpu',       // Try WebGPU first
    // Falls back to WebGL2 automatically
    width: 1920,
    height: 1080,
    antialias: true,
    backgroundColor: 0x1a1a2e,
});
```

Key architectural changes in v8:
- **Reactive render loop**: v8 only updates what has changed, dramatically reducing CPU overhead for static scenes
- **Unified shader system**: Single shader language compiles to both WebGPU (WGSL) and WebGL (GLSL)
- **Scene graph optimizations**: Dirty-flag propagation means unchanged subtrees cost near-zero CPU

#### Benchmark Results (Bunnymark)

| Scenario | v7 CPU Time | v8 CPU Time | Improvement |
|----------|-------------|-------------|-------------|
| 100k sprites, all moving | ~50ms | ~15ms | **3.3x faster** |
| 100k sprites, static | ~21ms | ~0.12ms | **175x faster** |

The static scene optimization is critical for CivLab: hex map terrain tiles are largely static between turns, meaning the renderer will spend near-zero CPU time on the map background during animations and UI interactions.

#### ParticleContainer — GPU Instancing for Mass Sprites

The v8 `ParticleContainer` is completely rewritten to use GPU instancing, achieving dramatically higher throughput than v7:

| Configuration | M3 MacBook Pro @ 60fps |
|---------------|------------------------|
| Sprites in Container | 200,000 |
| Particles in ParticleContainer | **1,000,000** |

ParticleContainer uses lightweight `Particle` objects (not full `Sprite`) with a static/dynamic property split:
- **Dynamic properties** (position, rotation): uploaded to GPU every frame
- **Static properties** (texture, anchor, tint): uploaded only on explicit `update()` call

This maps well to CivLab's needs:
- Unit tokens on the map: dynamic position, static texture — use ParticleContainer
- Terrain hexes: fully static — use regular Container with reactive updates
- UI overlays: mixed — use standard Sprite/Container hierarchy

#### TilingSprite for Hex Terrain

Each hex tile can be rendered as a `TilingSprite` with the appropriate hex texture. For large maps, `@pixi/tilemap` provides optimized batch rendering:

```typescript
import { TilingSprite, Texture } from 'pixi.js';

const hexTile = new TilingSprite({
    texture: Texture.from('hex-grassland.png'),
    width: 64,
    height: 74, // Hex height for pointy-top
});
hexTile.position.set(hexToPixelX(q, r), hexToPixelY(q, r));
```

### 2. @pixi/react — React 19 Integration

#### Rebuilt for React 19

`@pixi/react` v8 was rebuilt from scratch for React 19. It uses React 19's improved reconciler and the new `extend` API pattern:

```typescript
import { Application, extend } from '@pixi/react';
import { Container, Sprite, Graphics, Text } from 'pixi.js';

// Register only the Pixi components you use
extend({ Container, Sprite, Graphics, Text });

function GameMap({ hexes }: { hexes: HexTile[] }) {
    return (
        <Application width={1920} height={1080} background={0x1a1a2e}>
            <container>
                {hexes.map(hex => (
                    <sprite
                        key={hex.id}
                        texture={hex.texture}
                        x={hex.pixelX}
                        y={hex.pixelY}
                    />
                ))}
            </container>
        </Application>
    );
}
```

Key characteristics:
- **Tree-shakeable**: Only imported Pixi components are bundled
- **TypeScript-first**: Full generic typing for Container children types
- **React 19 exclusive**: Uses React 19 internals; does not support React 18
- **Declarative**: Pixi scene graph maps to JSX component tree

#### Installation

```bash
npm install pixi.js@^8.2.6 @pixi/react
```

Or scaffold a new project:
```bash
npm create pixi.js@latest --template framework-react
```

### 3. TypeScript Strict Mode Compatibility

Pixi.js v8 is written in TypeScript and ships with full type definitions. Specific TypeScript features:

- **Generic Container typing** (v8.1.0+): `Container<Sprite>` enforces child types
- **DTS bundles**: Single definition file with all exports under `PIXI` namespace
- **Strict mode**: v8 is developed with `strict: true` in its own tsconfig; consumer projects using `strict: true` will not encounter type errors from Pixi's definitions

No known issues with TypeScript strict mode in v8. The `@pixi/react` package also ships strict-compatible types.

### 4. Tilemap Plugin for Hex Maps

#### @pixi/tilemap (Official)

- **v5.0.1+** supports Pixi.js v8 via the extension system
- Optimized for rectangular/orthogonal tile grids
- Does NOT have native hexagonal tile support
- Good for background terrain rendering with GPU-optimized batching

#### pixi-tiledmap (Community)

- Supports Tiled editor `.tmx` format including **hexagonal orientation**
- Uses the modern Assets/LoadParser extension system for v8
- Can import hex maps designed in the Tiled map editor

#### Recommended Approach for CivLab

Use a custom hex-coordinate-to-pixel mapping layer (axial coordinates) on top of standard Pixi.js Sprites/Containers. The hex math is straightforward:

```typescript
// Pointy-top hex layout
const HEX_SIZE = 32;
const SQRT3 = Math.sqrt(3);

function hexToPixel(q: number, r: number): { x: number; y: number } {
    return {
        x: HEX_SIZE * (SQRT3 * q + (SQRT3 / 2) * r),
        y: HEX_SIZE * (3 / 2) * r,
    };
}
```

This avoids depending on a tilemap plugin for hex layout, while still allowing `@pixi/tilemap` for batch-optimized rendering of the tile textures themselves.

### 5. 2D-to-3D Upgrade Path: Pixi.js + Babylon.js Coexistence

#### Architecture for Coexistence

Pixi.js (2D) and Babylon.js (3D) can coexist on the same page using two approaches:

**Approach A: Separate Canvases (Recommended)**
- Pixi renders to one `<canvas>` for 2D UI, minimaps, HUD
- Babylon renders to another `<canvas>` for the 3D game world
- CSS layering (`z-index`) composites them visually
- Each renderer owns its own WebGL/WebGPU context independently
- Simpler, avoids shared-state bugs

**Approach B: Shared WebGL Context**
- Both renderers share a single WebGL context
- Requires careful state management (save/restore GL state between renderers)
- Babylon.js has official community documentation on this pattern
- More complex, but avoids multiple-canvas compositing overhead
- Known issue: PBRMaterial in Babylon can cause rendering conflicts with Pixi

**Recommendation**: Use Approach A (separate canvases). CivLab Phase 1 is 2D-only. When Phase 2 adds 3D, the 3D canvas replaces the 2D game-world canvas while Pixi continues handling 2D UI overlays.

#### Official Integration Resources

- Pixi.js docs: "Mixing PixiJS and Three.js" guide (same principles apply to Babylon.js)
- Babylon.js docs: "Babylon.js and Pixi.js" community extension page
- Both confirm the separate-canvas approach works reliably

### 6. Comparison with Alternatives

#### Phaser 3

| Factor | Pixi.js v8 | Phaser 3 |
|--------|------------|----------|
| Rendering backend | WebGPU + WebGL2 | WebGL1 (WebGPU experimental) |
| React integration | Official `@pixi/react` | None (imperative only) |
| TypeScript | Native, strict-compatible | Bolted-on types |
| Scene management | Bring your own | Opinionated (Scenes, GameObjects) |
| Bundle size | Tree-shakeable | Monolithic (~1MB) |
| 100k sprite perf | 1M particles @ 60fps | ~50k before dropping frames |
| Hex map support | Manual + tilemap plugins | Built-in tilemap (rectangular only) |

Pixi.js v8 wins on performance, React compatibility, and TypeScript quality. Phaser's opinionated scene system adds unnecessary overhead for CivLab, which has its own ECS and game state management.

#### Raw WebGL/WebGPU

Manual WebGL/WebGPU would provide maximum control but requires implementing:
- Sprite batching
- Texture atlas management
- Scene graph and culling
- Text rendering
- Interaction/hit testing

This is thousands of lines of rendering infrastructure that Pixi provides out of the box. Not justified for CivLab.

---

## Decision

**Pixi.js v8 + React 19** via `@pixi/react` for the CivLab web client.

Rationale:
1. **Performance**: 175x improvement for static scenes; 1M particle capacity at 60fps far exceeds CivLab's hex-map rendering needs
2. **React 19**: Official first-party React 19 integration with declarative JSX scene graph
3. **TypeScript**: Strict-mode compatible, generic-typed containers
4. **WebGPU future-proofing**: WebGPU-first with automatic WebGL2 fallback
5. **3D upgrade path**: Clean separation via IRenderer interface; Babylon.js slots in for Phase 2 via separate canvas
6. **Ecosystem**: Active maintenance, 42k+ GitHub stars, professional backing

---

## Implementation Contract

### IRenderer Interface

Both the 2D (Pixi) and future 3D (Babylon) renderers must implement this interface:

```typescript
/**
 * IRenderer — Abstraction over 2D and 3D rendering backends.
 * Phase 1: Pixi2DRenderer implements this.
 * Phase 2: Babylon3DRenderer implements this.
 */
interface IRenderer {
    /** Initialize the renderer and attach to the target DOM element. */
    init(config: RendererConfig): Promise<void>;

    /** Destroy the renderer and release all GPU resources. */
    destroy(): void;

    /** Resize the rendering viewport. */
    resize(width: number, height: number): void;

    /** Set camera position and zoom for the game world view. */
    setCamera(center: WorldCoord, zoom: number): void;

    /** Get the current camera state. */
    getCamera(): CameraState;

    /** Convert screen coordinates to world coordinates. */
    screenToWorld(screenX: number, screenY: number): WorldCoord;

    /** Convert world coordinates to screen coordinates. */
    worldToScreen(worldX: number, worldY: number): ScreenCoord;

    /**
     * Render a complete frame from the given render state.
     * The renderer does NOT own game state — it receives a snapshot each frame.
     */
    renderFrame(state: RenderState): void;

    /** Register a callback for user interaction events on the game world. */
    onWorldInteraction(callback: (event: WorldInteractionEvent) => void): void;

    /** Get performance metrics from the last rendered frame. */
    getFrameMetrics(): FrameMetrics;
}

interface RendererConfig {
    /** Target DOM element to attach the canvas to. */
    container: HTMLElement;

    /** Initial viewport width. */
    width: number;

    /** Initial viewport height. */
    height: number;

    /** Preferred rendering backend. */
    preference: 'webgpu' | 'webgl2' | 'auto';

    /** Device pixel ratio override (default: window.devicePixelRatio). */
    resolution?: number;

    /** Enable antialiasing (default: true). */
    antialias?: boolean;
}

interface CameraState {
    center: WorldCoord;
    zoom: number;
    viewportBounds: { minX: number; minY: number; maxX: number; maxY: number };
}

interface WorldCoord {
    x: number;
    y: number;
}

interface ScreenCoord {
    x: number;
    y: number;
}

interface RenderState {
    /** Hex tiles visible in the current viewport. */
    visibleTiles: TileRenderData[];

    /** Units visible in the current viewport. */
    visibleUnits: UnitRenderData[];

    /** City markers visible in the current viewport. */
    visibleCities: CityRenderData[];

    /** Fog of war overlay state. */
    fogOfWar: FogOfWarData;

    /** Active animations (movement paths, combat effects). */
    animations: AnimationData[];

    /** Selection highlights and hover indicators. */
    selection: SelectionData | null;

    /** Turn counter for animation timing. */
    turnNumber: number;

    /** Frame timestamp in ms for smooth animations. */
    timestamp: number;
}

interface TileRenderData {
    q: number;
    r: number;
    terrain: string;       // texture key: 'grassland', 'desert', 'ocean', etc.
    elevation: number;     // 0-5 for terrain height shading
    resource?: string;     // optional resource overlay texture key
    improvement?: string;  // optional improvement overlay texture key
    owner?: number;        // player ID for border coloring
}

interface UnitRenderData {
    id: string;
    q: number;
    r: number;
    unitType: string;      // texture key: 'warrior', 'settler', etc.
    owner: number;         // player ID for tinting
    health: number;        // 0.0-1.0 for health bar
    facing: number;        // rotation in radians
    isMoving: boolean;
    movePath?: WorldCoord[];  // interpolation path for movement animation
}

interface CityRenderData {
    id: string;
    q: number;
    r: number;
    name: string;
    owner: number;
    population: number;
    hasWalls: boolean;
}

interface FogOfWarData {
    /** Set of "q,r" keys that are fully hidden (unexplored). */
    unexplored: Set<string>;

    /** Set of "q,r" keys that are in fog (explored but not visible). */
    fogged: Set<string>;
}

interface AnimationData {
    type: 'unit_move' | 'combat' | 'city_grow' | 'border_expand';
    progress: number;  // 0.0-1.0
    data: unknown;     // type-specific payload
}

interface SelectionData {
    selectedHex: { q: number; r: number } | null;
    selectedUnit: string | null;
    highlightedHexes: { q: number; r: number; color: number }[];
    movementRange: { q: number; r: number }[];
    attackRange: { q: number; r: number }[];
}

interface WorldInteractionEvent {
    type: 'click' | 'rightclick' | 'hover' | 'drag_start' | 'drag_move' | 'drag_end' | 'zoom';
    worldX: number;
    worldY: number;
    hexQ: number;
    hexR: number;
    button?: number;
    delta?: number;  // for zoom events
}

interface FrameMetrics {
    fps: number;
    drawCalls: number;
    triangles: number;
    cpuTimeMs: number;
    gpuTimeMs: number;
    visibleSprites: number;
}
```

### Pixi2DRenderer — Phase 1 Implementation Skeleton

```typescript
import { Application, Container, Sprite, ParticleContainer } from 'pixi.js';

class Pixi2DRenderer implements IRenderer {
    private app: Application;
    private worldContainer: Container;
    private tileLayer: Container;
    private unitLayer: ParticleContainer;
    private overlayLayer: Container;

    async init(config: RendererConfig): Promise<void> {
        this.app = new Application();
        await this.app.init({
            preference: config.preference === 'auto' ? undefined : config.preference,
            width: config.width,
            height: config.height,
            antialias: config.antialias ?? true,
            resolution: config.resolution ?? window.devicePixelRatio,
            autoDensity: true,
        });

        config.container.appendChild(this.app.canvas);

        // Layer hierarchy
        this.worldContainer = new Container();
        this.tileLayer = new Container();
        this.unitLayer = new ParticleContainer({
            dynamicProperties: { position: true, rotation: true },
            staticProperties: { texture: true, tint: true },
        });
        this.overlayLayer = new Container();

        this.worldContainer.addChild(this.tileLayer);
        this.worldContainer.addChild(this.unitLayer);
        this.worldContainer.addChild(this.overlayLayer);
        this.app.stage.addChild(this.worldContainer);
    }

    // ... remaining methods implement IRenderer contract
}
```

### Babylon3DRenderer — Phase 2 Placeholder

```typescript
import { Engine, Scene } from '@babylonjs/core';

class Babylon3DRenderer implements IRenderer {
    private engine: Engine;
    private scene: Scene;

    async init(config: RendererConfig): Promise<void> {
        const canvas = document.createElement('canvas');
        canvas.width = config.width;
        canvas.height = config.height;
        config.container.appendChild(canvas);

        this.engine = new Engine(canvas, config.antialias ?? true);
        this.scene = new Scene(this.engine);
        // Phase 2: 3D camera, hex terrain mesh, unit models
    }

    // ... remaining methods implement same IRenderer contract
}
```

### RendererFactory

```typescript
type RendererType = '2d' | '3d';

function createRenderer(type: RendererType): IRenderer {
    switch (type) {
        case '2d':
            return new Pixi2DRenderer();
        case '3d':
            return new Babylon3DRenderer();
    }
}
```

---

## Dependency Versions

| Package | Version | Purpose |
|---------|---------|---------|
| `pixi.js` | `^8.2.6` | Core 2D renderer |
| `@pixi/react` | `^8.0.0` | React 19 bindings |
| `@pixi/tilemap` | `^5.0.1` | Optimized tile batch rendering |
| `react` | `^19.0.0` | UI framework |
| `@babylonjs/core` | `^7.0.0` | Phase 2 3D renderer |

---

## Open Questions Remaining

1. **Viewport culling strategy**: Should culling be done at the application level (only passing visible tiles in `RenderState`) or at the renderer level (Pixi culling off-screen sprites)? Likely application level for consistency between 2D/3D.

2. **Texture atlas pipeline**: How will hex tile textures be packed into atlases? Pixi's `Assets` system supports spritesheet loading from TexturePacker/Aseprite output. Need to define the asset pipeline spec.

3. **Fog of war rendering**: Best approach for fog — alpha mask overlay, per-tile tinting, or shader-based? Per-tile tinting is simplest in Pixi; shader-based is more performant for large maps.

4. **WebGPU browser support timeline**: As of 2026, Chrome and Edge ship WebGPU by default. Firefox and Safari support is progressing. The automatic WebGL2 fallback handles this, but worth tracking.

5. **@pixi/tilemap hex integration**: The tilemap plugin optimizes rectangular grids. For hex grids, we need to verify whether its batch renderer still provides benefits over plain Container with Sprites, or if the custom hex layout negates the batching advantage.

---

## Sources

- [PixiJS v8 Launch Blog](https://pixijs.com/blog/pixi-v8-launches)
- [ParticleContainer v8 Blog](https://pixijs.com/blog/particlecontainer-v8)
- [PixiJS React v8 Blog](https://pixijs.com/blog/pixi-react-v8-live)
- [PixiJS Renderers Guide](https://pixijs.com/8.x/guides/components/renderers)
- [PixiJS v8 Migration Guide](https://pixijs.com/8.x/guides/migrations/v8)
- [@pixi/react GitHub](https://github.com/pixijs/pixi-react)
- [@pixi/tilemap npm](https://www.npmjs.com/package/@pixi/tilemap)
- [Babylon.js + Pixi.js Docs](https://doc.babylonjs.com/communityExtensions/Babylon.js+ExternalLibraries/BabylonJS_and_PixiJS/)
- [Mixing PixiJS and Three.js Guide](https://pixijs.com/8.x/guides/third-party/mixing-three-and-pixi)
- [PixiJS React Getting Started](https://react.pixijs.io/getting-started/)


---

## Source: research/RND-005-3d-asset-gen-tools.md

# RND-005: Agentic 3D Asset Generation — Tool Comparison and Pipeline Recommendation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

For CivLab's 3D asset needs (~200 assets across units, buildings, terrain, and resources), a hybrid pipeline is recommended: **Meshy.ai** for hero units and high-visibility assets where quality and PBR textures matter most, and **Tripo3D** for bulk terrain/building assets where throughput and cost efficiency dominate. InstantMesh serves as a local fallback for rapid prototyping and cases where API latency or cost is unacceptable. Wonder3D is not recommended due to low resolution (256x256) and limited view coverage. Estimated total cost for 200 assets: **$160-$400** depending on quality tier distribution.

---

## Research Findings

### 1. Meshy.ai

#### Overview

Meshy.ai is a commercial AI-powered 3D model generation platform offering both text-to-3D and image-to-3D pipelines. As of 2025-2026, it is one of the most mature and widely-used 3D generation APIs for game asset creation.

#### API Documentation

**Text-to-3D Endpoint:**
```
POST https://api.meshy.ai/v2/text-to-3d
```

Key parameters:
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `prompt` | string | required | Object description (max 600 chars) |
| `art_style` | string | `"realistic"` | Style: realistic, cartoon, low-poly, sculpture, pbr |
| `topology` | string | `"triangle"` | triangle or quad |
| `target_polycount` | int | 30,000 | Range: 100-300,000 |
| `enable_pbr` | bool | false | Generate metallic, roughness, normal maps |

**Image-to-3D Endpoint:**
```
POST https://api.meshy.ai/v2/image-to-3d
```

Key parameters:
| Parameter | Type | Description |
|-----------|------|-------------|
| `image_url` | string | Input reference image |
| `enable_pbr` | bool | PBR map generation |
| `texture_prompt` | string | Text guidance for texturing (max 600 chars) |
| `texture_image_url` | string | 2D image to guide texturing |
| `should_remesh` | bool | Auto-remesh output |

**Output formats:** GLB, FBX, OBJ (+MTL), USDZ

**Remesh API:**
```
POST https://api.meshy.ai/v2/remesh
```
Converts to clean quad-dominant mesh topology, useful for game-engine import.

#### Quality Assessment

| Asset Type | Quality | Fidelity | Notes |
|------------|---------|----------|-------|
| Hard-surface props (weapons, buildings) | High | 80-90% | Best-in-class for architectural/mechanical |
| Terrain objects (rocks, trees) | High | 85-90% | Good organic shapes |
| Characters/creatures | Medium | 70-80% | Needs manual cleanup for animation-ready rigs |
| Stylized/low-poly | High | 85-95% | Excellent when `art_style: "low-poly"` |

**Polygon count control:**
- `target_polycount` parameter accepts 100-300,000
- Actual output varies based on prompt complexity
- Complex prompts may yield 50k+ polys requiring manual decimation
- For game assets, target 3,000-10,000 polys per asset

**Retopology:**
- Built-in remesh API produces quad-dominant meshes
- Suitable for direct import to game engines
- Not sufficient for character animation rigging (manual work needed)

#### Pricing

| Plan | Monthly Credits | Cost | API Access |
|------|-----------------|------|------------|
| Free | 100 | $0 | No |
| Pro | 1,000 | ~$20/mo | Yes |
| Studio | 4,000/seat | ~$60/mo/seat | Yes |

Via third-party platforms (fal.ai): ~$0.80 per text-to-3D generation.

**Cost estimate for 200 assets via Meshy:**
- At $0.80/generation, assuming 1.5 attempts per asset average: 200 * 1.5 * $0.80 = **$240**
- With Pro plan credits: ~$60-100 depending on credit efficiency

#### Style Consistency

Meshy supports an `art_style` parameter and text prompts can enforce style consistency. However, achieving truly consistent style across 50+ variants requires:
1. Using the same `art_style` setting for all assets
2. Including specific style keywords in every prompt (e.g., "low-poly stylized, flat shading, vibrant colors")
3. Using image-to-3D with a reference image from an established style sheet
4. Post-processing with the Text-to-Texture API to re-skin assets with a unified style

**Verdict:** Good for hero assets. Style consistency achievable with disciplined prompting but not guaranteed out-of-the-box for 50+ variants.

---

### 2. Tripo3D

#### Overview

Tripo3D is a newer entrant (2024-2025) that has rapidly improved quality, particularly for hard-surface assets. It supports text-to-3D and image-to-3D (including multi-image input as of v2.5).

#### API Documentation

**Authentication:** Bearer token via `Authorization: Bearer <token>` header.

**Endpoints:**
```
POST https://api.tripo3d.ai/v2/openapi/task
```

Task types: `text_to_model`, `image_to_model`, `multiview_to_model`, `refine_model`

**Quality tiers:**
| Tier | Output | Credit Cost |
|------|--------|-------------|
| Draft | Base mesh only, no texture | 10 credits |
| Standard | Baked texture + PBR model | 20-25 credits |
| HD | High-res baked texture + PBR | 40-50 credits |

**Quad mesh:** Optional, +5 credits. Useful for clean topology.

**Output formats:** GLB, FBX, OBJ, USDZ

#### Quality Assessment

| Asset Type | Quality | Notes |
|------------|---------|-------|
| Hard-surface (buildings, props) | High | Improved geometry in v2.5 |
| Organic (trees, terrain) | Medium-High | Multi-image input helps |
| Characters | Medium | Better than v1 but still needs cleanup |
| Consistent textures | Medium-High | PBR output with baked textures |

**Polygon counts:**
- Tripo does not expose a direct `target_polycount` parameter
- Output typically 10k-50k triangles depending on complexity
- Quad remesh available for cleaner topology
- Post-processing decimation needed for low-poly game assets

#### Pricing

**Credit-based model:**
- Standard text-to-3D with style: 25 credits
- HD image-to-3D with PBR + quad: 50 credits
- Third-party API (fal.ai): $0.20-$0.40 per model

**Cost estimate for 200 assets via Tripo:**
- At $0.30/model average: 200 * 1.3 attempts * $0.30 = **$78**
- Significantly cheaper than Meshy for bulk generation

#### Style Consistency

Tripo v2.5's multi-image input mode helps maintain consistency by allowing style reference images alongside the target prompt. However, the system lacks a formal "style lock" feature. For 50+ consistent variants:
1. Use multi-image input with style reference
2. Apply consistent prompting templates
3. Re-texture inconsistent outputs with a separate texturing pass

**Verdict:** Best cost-to-quality ratio for bulk assets. Multi-image input is a significant advantage for consistency.

---

### 3. InstantMesh (TencentARC)

#### Overview

InstantMesh is an open-source single-image-to-3D reconstruction model from TencentARC. It uses sparse-view large reconstruction models to generate meshes from a single input image in ~10 seconds.

#### GitHub Repository

- **URL:** https://github.com/TencentARC/InstantMesh
- **License:** Apache 2.0
- **Stars:** 3k+ (as of 2025)
- **Last updated:** Active development

#### Local Inference Requirements

| Requirement | Specification |
|-------------|---------------|
| Python | >= 3.10 |
| PyTorch | >= 2.1.0 |
| CUDA | >= 12.1 |
| GPU VRAM | ~8-12GB (for large variant) |
| xformers | 0.0.22.post7 |
| Inference time | ~10s per model on A100 |

**Model variants:** 4 sparse-view reconstruction variants + customized Zero123++ UNet

**Setup:**
```bash
conda create -n instantmesh python=3.10
conda activate instantmesh
pip install torch==2.1.0 torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
pip install xformers==0.0.22.post7
pip install -r requirements.txt
```

Models auto-download on first run (~2-4GB total).

#### Quality Assessment

| Factor | Rating | Notes |
|--------|--------|-------|
| Geometry quality | Medium | Good for simple objects, struggles with thin features |
| Texture quality | Low-Medium | Single-view inference limits texture coverage |
| Polygon count | Uncontrolled | Output is raw mesh, needs decimation |
| Back-side quality | Poor | Single image means back is hallucinated |
| Processing speed | Fast | ~10s/model on A100 |
| Consistency | Low | No style control mechanism |

#### Cost

- **Infrastructure cost only**: No per-model API fee
- A100 GPU rental: ~$1-3/hour
- At 360 models/hour throughput: ~$0.003-$0.008 per model
- M3 Max (local): MPS backend may work but significantly slower (~30-60s/model)

**Verdict:** Best for rapid prototyping and bulk generation where quality requirements are low. Not suitable for final hero assets without significant manual post-processing.

---

### 4. Wonder3D (CVPR 2024)

#### Overview

Wonder3D generates textured meshes from a single image using cross-domain diffusion for consistent multi-view normal maps and color images, followed by normal fusion for 3D reconstruction.

#### GitHub Repository

- **URL:** https://github.com/xxlong0/Wonder3D
- **License:** Research/Academic
- **Published:** CVPR 2024
- **Recent update:** Wonder3D++ (Dec 2024) extends the base model

#### Pipeline

1. Input single image
2. CLIP text embedding + camera parameters
3. Cross-domain diffusion generates 6 consistent views (normal + color)
4. Normal fusion algorithm reconstructs 3D geometry
5. Output: Textured mesh (2-3 minutes)

#### Quality Assessment

| Factor | Rating | Notes |
|--------|--------|-------|
| Geometry quality | Medium-High | Leading-level geometric detail vs prior work |
| Texture quality | Medium | 256x256 resolution limit |
| View coverage | Limited | 6 views only; occluded areas poorly reconstructed |
| Front-facing bias | Strong | Front-facing images produce best results |
| Processing speed | Slow | 2-3 minutes per model |
| Resolution | Low | 256x256 view resolution |

#### Limitations

- **256x256 resolution**: Major limitation for game assets requiring detail
- **6-view limitation**: Cannot cover full 360-degree object
- **Occlusion sensitivity**: Images with occlusions produce worse results
- **Research license**: Not clearly permissive for commercial use
- **Bug history**: Cross-domain attention CFG bug (fixed Aug 2024) caused misalignment

**Verdict:** Not recommended for CivLab production. Resolution too low, view coverage insufficient, and licensing unclear. Wonder3D++ may address some issues but not yet evaluated.

---

### 5. Comparative Analysis

#### Quality Comparison Matrix

| Factor | Meshy | Tripo3D | InstantMesh | Wonder3D |
|--------|-------|---------|-------------|----------|
| Geometry quality | 8/10 | 7/10 | 5/10 | 6/10 |
| Texture/PBR quality | 9/10 | 7/10 | 3/10 | 5/10 |
| Polygon control | Yes (100-300k) | Limited | None | None |
| Style consistency | 7/10 | 6/10 | 3/10 | 4/10 |
| Output formats | GLB/FBX/OBJ/USDZ | GLB/FBX/OBJ/USDZ | OBJ/GLB | OBJ |
| API maturity | Production | Production | Self-hosted | Research |
| Speed per asset | ~60s | ~30-60s | ~10s | ~150s |
| Cost per asset | $0.50-0.80 | $0.20-0.40 | ~$0.005 | Free (self-hosted) |
| Commercial license | Yes | Yes | Apache 2.0 | Research |

#### Cost Estimate for 200 Assets

| Scenario | Meshy Only | Tripo Only | Hybrid (Recommended) | InstantMesh Only |
|----------|------------|------------|----------------------|------------------|
| 50 hero units | $60 | $26 | Meshy: $60 | $0.75 |
| 50 buildings | $60 | $26 | Tripo: $26 | $0.75 |
| 50 terrain objects | $60 | $26 | Tripo: $26 | $0.75 |
| 50 resources/props | $60 | $26 | Tripo: $26 | $0.75 |
| **Total (1.5x retries)** | **$360** | **$156** | **$207** | **$4.50 + GPU** |

The hybrid approach allocates Meshy's superior quality to high-visibility hero assets while using Tripo's cost efficiency for bulk assets that appear smaller on screen.

---

## Decision

**Hybrid pipeline: Meshy.ai (hero assets) + Tripo3D (bulk assets) + InstantMesh (prototyping)**

### Asset Tier Classification

| Tier | Tool | Assets | Quality Target | Poly Budget |
|------|------|--------|----------------|-------------|
| **S-Tier** (hero units, wonders) | Meshy.ai | ~30 | High PBR, hand-reviewed | 5k-10k tris |
| **A-Tier** (standard buildings, units) | Tripo3D | ~80 | Standard PBR | 3k-5k tris |
| **B-Tier** (terrain, resources, props) | Tripo3D | ~70 | Standard, batch-generated | 1k-3k tris |
| **Prototype** (iteration, concept art) | InstantMesh | as-needed | Draft quality | unconstrained |

### Pipeline Workflow

```
1. Concept Art (2D)
   ├── SD XL generates reference images (see RND-006)
   └── Art director approves style sheet

2. 3D Generation
   ├── S-Tier: Meshy image-to-3D with PBR + style prompt
   ├── A-Tier: Tripo multi-image-to-3D with style reference
   └── B-Tier: Tripo text-to-3D batch generation

3. Post-Processing
   ├── Polygon decimation to budget (Blender CLI or meshoptimizer)
   ├── UV unwrap validation
   ├── PBR texture baking consistency check
   └── Quality gate: screenshot comparison vs style sheet

4. Export
   ├── GLB for web client (Pixi textures or Babylon meshes)
   ├── FBX for asset archive
   └── Thumbnail renders for asset browser
```

---

## Implementation Contract

### Asset Generation Service Interface

```typescript
interface IAssetGenerator {
    /** Generate a 3D model from a text description. */
    generateFromText(request: TextTo3DRequest): Promise<Asset3DResult>;

    /** Generate a 3D model from one or more reference images. */
    generateFromImages(request: ImageTo3DRequest): Promise<Asset3DResult>;

    /** Check the status of an async generation task. */
    getTaskStatus(taskId: string): Promise<TaskStatus>;

    /** Download the generated asset files. */
    downloadAsset(taskId: string, format: AssetFormat): Promise<Buffer>;
}

interface TextTo3DRequest {
    prompt: string;
    artStyle: 'realistic' | 'cartoon' | 'low-poly' | 'stylized';
    targetPolycount: number;
    enablePBR: boolean;
    tier: 'S' | 'A' | 'B' | 'prototype';
}

interface ImageTo3DRequest {
    imageUrls: string[];         // 1-4 reference images
    texturePrompt?: string;      // Optional texture guidance
    enablePBR: boolean;
    tier: 'S' | 'A' | 'B' | 'prototype';
}

interface Asset3DResult {
    taskId: string;
    status: 'pending' | 'processing' | 'completed' | 'failed';
    downloadUrls?: {
        glb?: string;
        fbx?: string;
        obj?: string;
        usdz?: string;
    };
    metadata?: {
        polycount: number;
        hasPBR: boolean;
        generationTimeMs: number;
        provider: 'meshy' | 'tripo' | 'instantmesh';
    };
}

type AssetFormat = 'glb' | 'fbx' | 'obj' | 'usdz';

interface TaskStatus {
    taskId: string;
    status: 'pending' | 'processing' | 'completed' | 'failed';
    progress: number;      // 0.0-1.0
    estimatedTimeMs?: number;
    errorMessage?: string;
}
```

### Provider Routing Logic

```typescript
function selectProvider(request: TextTo3DRequest | ImageTo3DRequest): 'meshy' | 'tripo' | 'instantmesh' {
    switch (request.tier) {
        case 'S':
            return 'meshy';
        case 'A':
        case 'B':
            return 'tripo';
        case 'prototype':
            return 'instantmesh';
    }
}
```

---

## Open Questions Remaining

1. **Meshy v6 evaluation**: Meshy 6 was announced with WaveSpeedAI integration. Need to evaluate whether quality improvements change the tier allocation.

2. **Tripo v2.5 multi-image consistency**: How many reference images are optimal for style consistency? Need empirical testing with CivLab's art style.

3. **Animation rigging pipeline**: None of the evaluated tools produce animation-ready rigged meshes. Need a separate pipeline for character animation (Mixamo auto-rigging or manual).

4. **Texture atlas consolidation**: Generated assets each have individual textures. For web rendering performance, these need to be consolidated into shared texture atlases. Need to define the atlas packing pipeline.

5. **LOD generation**: For the web client, assets need 2-3 LOD levels (high for close-up, low for zoomed-out map view). Can meshoptimizer or Simplygon handle this automatically?

6. **InstantMesh on Apple Silicon**: MPS backend compatibility with InstantMesh's xformers dependency is uncertain. Need to test local inference on M3 Max.

---

## Sources

- [Meshy.ai Pricing](https://www.meshy.ai/pricing)
- [Meshy API Docs — Text-to-3D](https://docs.meshy.ai/en/api/text-to-3d)
- [Meshy API Docs — Image-to-3D](https://docs.meshy.ai/en/api/image-to-3d)
- [Meshy API Docs — Remesh](https://docs.meshy.ai/en/api/remesh)
- [Tripo3D Pricing](https://www.tripo3d.ai/pricing)
- [Tripo3D API Platform](https://www.tripo3d.ai/api)
- [Tripo3D API Billing](https://platform.tripo3d.ai/docs/billing)
- [Tripo3D v2.5 on fal.ai](https://fal.ai/models/tripo3d/tripo/v2.5/image-to-3d/api)
- [InstantMesh GitHub](https://github.com/TencentARC/InstantMesh)
- [Wonder3D GitHub](https://github.com/xxlong0/Wonder3D)
- [Wonder3D CVPR 2024 Paper](https://openaccess.thecvf.com/content/CVPR2024/papers/Long_Wonder3D_Single_Image_to_3D_using_Cross-Domain_Diffusion_CVPR_2024_paper.pdf)
- [Meshy Review — AIquiks](https://aiquiks.com/ai-tools/meshy)
- [Tripo AI Review 2025 — Skywork](https://skywork.ai/blog/tripo-ai-review-2025/)
- [Meshy 6 on WaveSpeedAI](https://wavespeed.ai/models/wavespeed-ai/meshy6/text-to-3d)


---

## Source: research/RND-006-sdxl-sprite-pipeline.md

# RND-006: Stable Diffusion XL + ControlNet Setup for Consistent 2D Game Sprites

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

The recommended pipeline for generating consistent 2D game sprites is **Automatic1111 WebUI API** (`/sdapi/v1/txt2img` and `/sdapi/v1/img2img`) with **ControlNet OpenPose** for consistent character poses across 8 directional facings, combined with a **custom LoRA** trained on 20-50 reference images to lock the art style. SDXL provides seed-based determinism when the same model, seed, prompt, and deterministic scheduler (Euler, DDIM) are used. ComfyUI is a viable alternative with more flexibility for complex workflows but higher integration complexity. The full pipeline: A1111 API + ControlNet OpenPose + Custom LoRA + seed determinism.

---

## Research Findings

### 1. Automatic1111 (A1111) API

#### Overview

Automatic1111's stable-diffusion-webui exposes a REST API at `/sdapi/v1/*` when launched with the `--api` flag. This is the simplest integration path for programmatic sprite generation.

#### Setup

```bash
# Launch with API enabled
python launch.py --api --listen --port 7860
```

The API documentation is auto-generated and available at `http://localhost:7860/docs` (Swagger UI).

#### Core Endpoints

**Text-to-Image:**
```
POST /sdapi/v1/txt2img
```

```json
{
    "prompt": "game character warrior, pixel art style, front-facing, white background, <lora:civlab_style:0.8>",
    "negative_prompt": "blurry, low quality, deformed, watermark, text",
    "width": 1024,
    "height": 1024,
    "steps": 30,
    "cfg_scale": 7.0,
    "sampler_name": "Euler",
    "scheduler": "Normal",
    "seed": 42,
    "batch_size": 1,
    "n_iter": 1,
    "restore_faces": false,
    "enable_hr": false,
    "alwayson_scripts": {}
}
```

**Image-to-Image:**
```
POST /sdapi/v1/img2img
```

Same parameters as txt2img plus:
- `init_images`: Array of base64-encoded input images
- `denoising_strength`: 0.0-1.0 (lower = more faithful to input)

**Response format:**
```json
{
    "images": ["base64_encoded_png_data"],
    "parameters": { ... },
    "info": "generation_info_json_string"
}
```

#### ControlNet Integration via A1111 API

ControlNet is passed through the `alwayson_scripts` parameter:

```json
{
    "prompt": "game character warrior, side view, walking pose, <lora:civlab_style:0.8>",
    "width": 1024,
    "height": 1024,
    "steps": 30,
    "cfg_scale": 7.0,
    "sampler_name": "Euler",
    "seed": 42,
    "alwayson_scripts": {
        "controlnet": {
            "args": [
                {
                    "enabled": true,
                    "module": "openpose_full",
                    "model": "control_v11p_sd15_openpose",
                    "weight": 1.0,
                    "image": "<base64_encoded_openpose_skeleton>",
                    "resize_mode": "Crop and Resize",
                    "lowvram": false,
                    "processor_res": 512,
                    "guidance_start": 0.0,
                    "guidance_end": 1.0,
                    "control_mode": "Balanced"
                }
            ]
        }
    }
}
```

#### Other Useful Endpoints

| Endpoint | Purpose |
|----------|---------|
| `GET /sdapi/v1/sd-models` | List available checkpoint models |
| `GET /sdapi/v1/loras` | List available LoRAs |
| `GET /sdapi/v1/samplers` | List available samplers |
| `POST /sdapi/v1/options` | Set/get runtime options (model swap, etc.) |
| `GET /sdapi/v1/progress` | Get generation progress |
| `POST /sdapi/v1/interrupt` | Cancel current generation |

---

### 2. ComfyUI API (Alternative)

#### Overview

ComfyUI provides a node-based workflow system with an HTTP API. Workflows are defined as JSON graphs where each node has an ID, type, and connections to other nodes.

#### API Endpoint

```
POST http://localhost:8188/prompt
```

```json
{
    "client_id": "unique-client-id",
    "prompt": {
        "3": {
            "class_type": "KSampler",
            "inputs": {
                "seed": 42,
                "steps": 30,
                "cfg": 7.0,
                "sampler_name": "euler",
                "scheduler": "normal",
                "denoise": 1.0,
                "model": ["4", 0],
                "positive": ["6", 0],
                "negative": ["7", 0],
                "latent_image": ["5", 0]
            }
        },
        "4": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": "sd_xl_base_1.0.safetensors"
            }
        },
        "5": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width": 1024,
                "height": 1024,
                "batch_size": 1
            }
        },
        "6": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": "game character warrior, pixel art style",
                "clip": ["4", 1]
            }
        },
        "7": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": "blurry, low quality, deformed",
                "clip": ["4", 1]
            }
        },
        "8": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": ["3", 0],
                "vae": ["4", 2]
            }
        },
        "9": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "warrior",
                "images": ["8", 0]
            }
        }
    }
}
```

#### ComfyUI Parameterization

Variables can be injected using handlebars syntax (`{{prompt}}`, `{{seed}}`) when using wrapper frameworks. Direct API usage requires modifying the JSON node values programmatically before submission.

#### WebSocket for Progress

ComfyUI prefers WebSocket connections for real-time progress:
```
ws://localhost:8188/ws?clientId=unique-client-id
```

Messages include `execution_start`, `executing` (per-node progress), and `executed` (completion with output paths).

#### ComfyUI vs A1111 API Comparison

| Factor | A1111 API | ComfyUI API |
|--------|-----------|-------------|
| Simplicity | Simple REST, flat JSON | Complex graph JSON |
| ControlNet | `alwayson_scripts` parameter | Dedicated ControlNet nodes |
| Workflow flexibility | Fixed pipeline | Arbitrary node graphs |
| Progress tracking | Polling (`/progress`) | WebSocket (real-time) |
| Batch operations | Built-in batch_size/n_iter | Manual graph construction |
| Documentation | Swagger auto-docs | Minimal, community-driven |
| Dynamic params | Direct JSON fields | Template injection or graph mutation |
| Setup complexity | `--api` flag | Already API-native |

**Recommendation:** A1111 API for CivLab's sprite pipeline. The use case is straightforward (txt2img + ControlNet + LoRA), and A1111's simpler REST interface reduces integration overhead. ComfyUI's graph-based approach is overkill for this pipeline.

---

### 3. ControlNet OpenPose for 8-Directional Character Sprites

#### The 8-Direction Sprite Sheet Problem

CivLab game units need sprites for 8 facing directions:

```
    N
  NW  NE
W       E
  SW  SE
    S
```

Each direction requires a consistent character in a different pose/orientation. Without ControlNet, SDXL would generate unpredictable poses, making sprite sheets inconsistent.

#### OpenPose Skeleton Control

ControlNet OpenPose uses a stick-figure skeleton to precisely control character pose and orientation. For each of the 8 directions, a pre-defined skeleton is created:

```
Direction skeletons (simplified):

N (back):     NE (3/4 back):  E (side right):  SE (3/4 front):
  O              O               O                  O
 /|\            /|\             /|                  /|\
  |              |              |                    |
 / \            / \            / \                  / \

S (front):    SW (3/4 front): W (side left):  NW (3/4 back):
  O              O                O               O
 /|\            /|\               |\              /|\
  |              |                |                |
 / \            / \              / \              / \
```

Each skeleton is saved as a 512x512 or 1024x1024 PNG with the OpenPose color-coded keypoint format:
- Red: right limbs
- Blue: left limbs
- Yellow: torso/spine
- Green: face points

#### Workflow for 8-Direction Generation

```python
import requests
import base64
import json

A1111_URL = "http://localhost:7860"

DIRECTIONS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW']

def load_skeleton(direction: str) -> str:
    """Load pre-made OpenPose skeleton for given direction as base64."""
    with open(f"assets/skeletons/{direction}.png", "rb") as f:
        return base64.b64encode(f.read()).decode()

def generate_sprite(
    unit_type: str,
    direction: str,
    seed: int,
    lora_name: str = "civlab_style",
    lora_weight: float = 0.8,
) -> bytes:
    """Generate a single sprite for a unit type and direction."""

    direction_prompts = {
        'N':  'back view, facing away',
        'NE': 'three-quarter back view, slight right turn',
        'E':  'side view, facing right, profile',
        'SE': 'three-quarter front view, slight right turn',
        'S':  'front view, facing camera',
        'SW': 'three-quarter front view, slight left turn',
        'W':  'side view, facing left, profile',
        'NW': 'three-quarter back view, slight left turn',
    }

    prompt = (
        f"game character {unit_type}, {direction_prompts[direction]}, "
        f"white background, centered, full body, "
        f"<lora:{lora_name}:{lora_weight}>"
    )

    payload = {
        "prompt": prompt,
        "negative_prompt": "blurry, low quality, deformed, watermark, text, cropped, partial body",
        "width": 1024,
        "height": 1024,
        "steps": 30,
        "cfg_scale": 7.0,
        "sampler_name": "Euler",
        "seed": seed,
        "alwayson_scripts": {
            "controlnet": {
                "args": [{
                    "enabled": True,
                    "module": "openpose_full",
                    "model": "control_v11p_sd15_openpose",
                    "weight": 1.0,
                    "image": load_skeleton(direction),
                    "resize_mode": "Crop and Resize",
                    "guidance_start": 0.0,
                    "guidance_end": 1.0,
                    "control_mode": "Balanced",
                }]
            }
        }
    }

    response = requests.post(f"{A1111_URL}/sdapi/v1/txt2img", json=payload)
    result = response.json()
    return base64.b64decode(result["images"][0])

def generate_sprite_sheet(unit_type: str, base_seed: int) -> dict[str, bytes]:
    """Generate all 8 directional sprites for a unit type."""
    sprites = {}
    for i, direction in enumerate(DIRECTIONS):
        # Use base_seed + direction offset for reproducibility
        # while maintaining different compositions per direction
        sprites[direction] = generate_sprite(
            unit_type=unit_type,
            direction=direction,
            seed=base_seed + i,
        )
    return sprites
```

#### OpenPose Model Selection for SDXL

| Model | Base | Notes |
|-------|------|-------|
| `control_v11p_sd15_openpose` | SD 1.5 | Most mature, widest adoption |
| `controlnet-openpose-sdxl-1.0` | SDXL | Native SDXL resolution (1024x1024) |
| `t2i-adapter-openpose-sdxl` | SDXL | T2I-Adapter variant, lighter weight |

For SDXL, use `controlnet-openpose-sdxl-1.0` for native resolution support.

---

### 4. LoRA Training for Art Style Consistency

#### Why LoRA?

Without a trained LoRA, SDXL generates images in a generic style that varies across prompts. A LoRA fine-tunes the model on a specific art style using a small dataset, ensuring all generated sprites share a consistent visual identity.

#### Training Dataset Requirements

| Parameter | Recommended | Notes |
|-----------|-------------|-------|
| Number of reference images | **20-50** | Sweet spot: 20-25 images with diversity |
| Minimum images | 10 | Below this, overfitting risk increases |
| Image resolution | >= 1024x1024 | Match SDXL native resolution |
| Image diversity | High | Different poses, lighting, backgrounds |
| Style consistency | Critical | All images must share target art style |
| Image format | PNG | Lossless, no compression artifacts |

**Dataset preparation:**
1. Collect 20-50 reference images in the target art style
2. Crop/resize to 1024x1024
3. Write captions for each image (auto-captioning via BLIP-2 or manual)
4. Organize as pairs: `image_001.png` + `image_001.txt`

#### Training Parameters

| Parameter | Recommended | Range | Notes |
|-----------|-------------|-------|-------|
| **Network Rank (dim)** | 32 | 8-64 | Higher = more capacity, more VRAM |
| **Network Alpha** | 16 | half of rank | Effective LR = LR * (alpha/rank) |
| **Learning Rate** | 1e-4 | 5e-5 to 2e-4 | Use Adafactor optimizer |
| **Training Steps** | 1,500-2,000 | 1,000-3,000 | More steps risk overfitting |
| **Batch Size** | 1-4 | depends on VRAM | Larger = smoother gradients |
| **Optimizer** | Adafactor | or AdamW8bit | Adafactor is memory-efficient |
| **LR Scheduler** | constant | or cosine | Constant with 0 warmup works well |
| **Loss Type** | smooth_l1 | or mse | `huber_schedule: "snr"` |
| **Resolution** | 1024x1024 | | Match SDXL native |

#### Training Infrastructure

| Hardware | Training Time | Notes |
|----------|---------------|-------|
| M3 Max (48GB) | 2-4 hours | Via mps backend, slower but usable |
| A100 (80GB) | 30-60 minutes | Fastest, recommended for iteration |
| RTX 4090 (24GB) | 1-2 hours | Good balance of cost/speed |

**Tools:**
- **kohya_ss**: Most popular LoRA trainer for SDXL, supports all parameters above
- **OneTrainer**: Alternative with GUI, good for experimentation
- **ai-toolkit**: Simplified training scripts

#### Training Command (kohya_ss)

```bash
accelerate launch train_network.py \
    --pretrained_model_name_or_path="stabilityai/stable-diffusion-xl-base-1.0" \
    --train_data_dir="./training_data" \
    --output_dir="./output_lora" \
    --output_name="civlab_style" \
    --network_module="networks.lora" \
    --network_dim=32 \
    --network_alpha=16 \
    --learning_rate=1e-4 \
    --lr_scheduler="constant" \
    --lr_warmup_steps=0 \
    --optimizer_type="Adafactor" \
    --max_train_steps=2000 \
    --resolution="1024,1024" \
    --train_batch_size=1 \
    --mixed_precision="bf16" \
    --save_every_n_steps=500 \
    --caption_extension=".txt" \
    --xformers \
    --cache_latents
```

#### LoRA Usage in Generation

Once trained, the LoRA is referenced in prompts:

```
<lora:civlab_style:0.8>
```

The weight (0.8) controls influence strength:
- **0.5-0.7**: Subtle style influence, more prompt flexibility
- **0.7-0.9**: Strong style lock, recommended for consistency
- **0.9-1.0**: Very strong, may reduce prompt adherence

---

### 5. Seed Determinism in SDXL

#### Determinism Guarantee

SDXL IS deterministic given identical:
- Model checkpoint (exact same .safetensors file)
- Seed value
- Prompt and negative prompt
- Sampler/scheduler
- Steps, CFG scale, resolution
- ControlNet inputs (if used)
- LoRA weights (if used)

#### Deterministic Schedulers

| Scheduler | Deterministic | Notes |
|-----------|---------------|-------|
| **Euler** | YES | Converges reliably, robust baseline |
| **DDIM** | YES | Deterministic, good quality |
| **DPM++ 2M** | YES | High quality, deterministic |
| **DPM++ 2M Karras** | YES | Karras noise schedule variant |
| **Euler Ancestral** | NO | Injects random noise per step |
| **DPM++ 2S a** | NO | Stochastic variant |
| **DPM++ SDE** | NO | Stochastic differential equation |

**Recommendation:** Use **Euler** as the default scheduler for CivLab sprite generation. It is deterministic, converges reliably, and produces consistent results.

#### Cross-Platform Caveats

Determinism is NOT guaranteed across:
- Different GPU hardware (NVIDIA A100 vs RTX 4090 may produce subtly different results)
- Different CUDA versions
- CPU vs GPU execution
- Different operating systems (floating-point rounding)

For CivLab, this means the sprite generation pipeline should run on a **fixed, dedicated machine** (or container with pinned CUDA version) to ensure reproducibility.

#### Seed Strategy for Sprite Sheets

```python
# Seed allocation strategy
UNIT_TYPE_SEEDS = {
    'warrior': 1000,
    'archer': 2000,
    'settler': 3000,
    'scout': 4000,
    # ... etc
}

DIRECTION_OFFSETS = {
    'N': 0, 'NE': 1, 'E': 2, 'SE': 3,
    'S': 4, 'SW': 5, 'W': 6, 'NW': 7,
}

def get_seed(unit_type: str, direction: str) -> int:
    return UNIT_TYPE_SEEDS[unit_type] + DIRECTION_OFFSETS[direction]

# warrior facing NE -> seed 1001
# warrior facing S  -> seed 1004
# archer facing E   -> seed 2002
```

This ensures:
- Same unit+direction always produces same output (reproducibility)
- Different directions use different seeds (variety)
- Different unit types use different seed ranges (no collisions)

---

### 6. Full Recommended Workflow

```
┌─────────────────────────────────────────────────────┐
│                 SPRITE GENERATION PIPELINE           │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. STYLE DEFINITION                                │
│     ├── Collect 20-50 reference images              │
│     ├── Train SDXL LoRA (rank=32, 2000 steps)       │
│     └── Validate: generate 10 test images           │
│                                                     │
│  2. POSE PREPARATION                                │
│     ├── Create 8 OpenPose skeletons (one per dir)   │
│     ├── Validate: overlay on reference images        │
│     └── Store as 1024x1024 PNGs                     │
│                                                     │
│  3. SPRITE GENERATION                               │
│     ├── A1111 API with --api flag                   │
│     ├── ControlNet OpenPose (SDXL model)            │
│     ├── LoRA: <lora:civlab_style:0.8>               │
│     ├── Scheduler: Euler (deterministic)            │
│     ├── Seed: unit_base + direction_offset          │
│     └── Resolution: 1024x1024                       │
│                                                     │
│  4. POST-PROCESSING                                 │
│     ├── Background removal (rembg or SAM)           │
│     ├── Resize to game resolution (64x64, 128x128)  │
│     ├── Pack into sprite sheet atlas                │
│     └── Generate metadata JSON                      │
│                                                     │
│  5. QUALITY GATE                                    │
│     ├── Visual consistency check across 8 dirs      │
│     ├── Silhouette comparison (shape consistency)   │
│     ├── Color palette validation                    │
│     └── Approve or regenerate with adjusted seed    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## Decision

**A1111 API + ControlNet OpenPose (SDXL) + Custom LoRA + Euler scheduler with seed determinism.**

This pipeline provides:
1. **Pose consistency**: ControlNet OpenPose enforces character orientation across 8 directions
2. **Style consistency**: Custom LoRA locks the art style across all unit types
3. **Reproducibility**: Deterministic seed + Euler scheduler = identical output for identical inputs
4. **Simplicity**: A1111 REST API is straightforward to integrate vs ComfyUI's graph-based approach
5. **Flexibility**: LoRA weight and ControlNet strength provide fine-tuning knobs

---

## Implementation Contract

### SpritePipeline Interface

```typescript
interface ISpritePipeline {
    /** Check A1111 server health and model availability. */
    healthCheck(): Promise<PipelineHealth>;

    /** Generate a single sprite for a unit type and direction. */
    generateSprite(request: SpriteRequest): Promise<SpriteResult>;

    /** Generate a complete 8-direction sprite sheet for a unit type. */
    generateSpriteSheet(request: SpriteSheetRequest): Promise<SpriteSheetResult>;

    /** Get generation progress for an active request. */
    getProgress(): Promise<GenerationProgress>;
}

interface SpriteRequest {
    unitType: string;
    direction: 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW';
    seed: number;
    loraName: string;
    loraWeight: number;          // 0.0-1.0, recommended 0.8
    controlnetWeight: number;    // 0.0-1.0, recommended 1.0
    steps: number;               // recommended 30
    cfgScale: number;            // recommended 7.0
    width: number;               // recommended 1024
    height: number;              // recommended 1024
}

interface SpriteSheetRequest {
    unitType: string;
    baseSeed: number;
    loraName: string;
    loraWeight: number;
    outputSize: { width: number; height: number };  // final sprite size (e.g., 64x64)
}

interface SpriteResult {
    imageData: Buffer;       // PNG bytes
    seed: number;            // actual seed used
    generationTimeMs: number;
    metadata: {
        prompt: string;
        negativPrompt: string;
        sampler: string;
        steps: number;
        cfgScale: number;
    };
}

interface SpriteSheetResult {
    sprites: Record<string, SpriteResult>;  // direction -> sprite
    atlasImage: Buffer;                      // packed sprite sheet PNG
    atlasMetadata: {                         // for game engine consumption
        frameWidth: number;
        frameHeight: number;
        frames: Record<string, { x: number; y: number; w: number; h: number }>;
    };
}

interface PipelineHealth {
    serverReachable: boolean;
    modelLoaded: string;
    lorasAvailable: string[];
    controlnetModelsAvailable: string[];
    gpuName: string;
    gpuVramMb: number;
}

interface GenerationProgress {
    progress: number;       // 0.0-1.0
    etaSeconds: number;
    currentStep: number;
    totalSteps: number;
    currentImage?: Buffer;  // preview of current generation
}
```

### Seed Registry

```typescript
/**
 * Deterministic seed allocation for reproducible sprite generation.
 * Each unit type gets a 1000-seed range. Directions use offsets 0-7.
 * Variants (armor upgrades, etc.) use offsets 100-199.
 */
interface ISeedRegistry {
    /** Get the deterministic seed for a specific unit + direction + variant. */
    getSeed(unitType: string, direction: string, variant?: string): number;

    /** Register a new unit type with a base seed. */
    registerUnitType(unitType: string, baseSeed: number): void;

    /** Export the full seed map for reproducibility audit. */
    exportSeedMap(): Record<string, number>;
}
```

---

## Open Questions Remaining

1. **SDXL ControlNet model maturity**: The `controlnet-openpose-sdxl-1.0` model is less battle-tested than the SD 1.5 variant. Need to validate quality at SDXL resolution for game sprite use cases.

2. **LoRA + ControlNet interaction**: High LoRA weights (>0.8) combined with strong ControlNet guidance may conflict. Need empirical testing to find the optimal balance.

3. **Background removal pipeline**: rembg vs SAM (Segment Anything Model) for clean background removal. rembg is simpler; SAM is more accurate for complex silhouettes.

4. **Animation frames**: Beyond 8-direction static sprites, CivLab may need animated sprites (walk cycle, attack, idle). This requires ControlNet temporal consistency or AnimateDiff integration.

5. **M3 Max performance**: SDXL inference on Apple Silicon via MPS is slower than CUDA. Need to benchmark: at 1024x1024 with 30 steps, how many sprites/hour? Estimate: ~2-4 sprites/minute.

6. **LoRA overfitting detection**: How to detect overfitting during training? Monitor validation loss; generate test images at each checkpoint (every 500 steps) and visually inspect for mode collapse.

7. **Sprite resolution pipeline**: Generate at 1024x1024, downscale to 64x64 or 128x128. Which downscaling algorithm preserves pixel-art crispness? Lanczos for smooth art; nearest-neighbor for pixel art.

---

## Sources

- [A1111 API Wiki](https://github.com/AUTOMATIC1111/stable-diffusion-webui/wiki/API)
- [A1111 API Discussion #3734](https://github.com/AUTOMATIC1111/stable-diffusion-webui/discussions/3734)
- [A1111 txt2img Guide](https://randombits.dev/articles/stable-diffusion/txt2img)
- [A1111 API Guide](https://randombits.dev/articles/stable-diffusion/api)
- [ComfyUI Workflow JSON Spec](https://docs.comfy.org/specs/workflow_json)
- [Hosting ComfyUI via API — 9elements](https://9elements.com/blog/hosting-a-comfyui-workflow-via-api/)
- [Building Production-Ready ComfyUI API — ViewComfy](https://www.viewcomfy.com/blog/building-a-production-ready-comfyui-api)
- [ComfyUI API Deep Wiki](https://deepwiki.com/Comfy-Org/ComfyUI/7-api-and-programmatic-usage)
- [LoRA Training 2025 Ultimate Guide — sanj.dev](https://sanj.dev/post/lora-training-2025-ultimate-guide)
- [Detailed LoRA Training Guide — ViewComfy](https://www.viewcomfy.com/blog/detailed-LoRA-training-guide-for-Stable-Diffusion)
- [SDXL LoRA Training — froehlichundfrei](https://www.froehlichundfrei.de/blog/2024-01-22-stable-diffusion-xl-lora-training/)
- [Perfect LoRA Parameters — HuggingFace](https://discuss.huggingface.co/t/perfect-lora-training-parameters-human-character/147211)
- [Ultimate SDXL LoRA Training — lilys.ai](https://lilys.ai/en/notes/training-lora-20260208/ultimate-sdxl-lora-training)
- [Reproducible Pipelines — HuggingFace Diffusers](https://huggingface.co/docs/diffusers/using-diffusers/reusing_seeds)
- [SDXL Settings Guide — Replicate](https://sdxl.replicate.dev/)
- [Sampler/Scheduler Reference — CivitAI](https://civitai.com/articles/16231/sampler-and-scheduler-reference-for-hi-dream-flux-sdxl-illustrious-and-pony)
- [OpenPose ControlNet Tutorial — NextDiffusion](https://www.nextdiffusion.ai/tutorials/how-to-use-open-pose-controlnet-in-stable-diffusion)


---

## Source: research/RND-007-adaptive-music-kira.md

# RND-007: Adaptive Music Architecture -- Kira, Howler.js, and AI Music Generation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-delta

---

## Executive Summary

This document specifies the adaptive music system for CivLab, spanning the Bevy desktop client (Kira audio engine), the web client (Howler.js), and AI music asset generation (MusicGen). The core design is a layered mixing approach: 8 pre-generated mood tracks play simultaneously with independent volume envelopes, crossfaded by game-state transitions via smooth tweens. Kira 0.12 provides the necessary primitives (TrackHandle volume automation, ClockHandle for beat-synced transitions, Tween easing curves). Howler.js mirrors this on the web with its Web Audio API gain nodes. MusicGen (Meta AudioCraft, local, free, Apache-2.0) is recommended for generating the 8 mood tracks offline during asset pipeline, avoiding runtime API costs.

---

## Research Findings

### 1. Kira Audio Engine (Rust / Bevy Client)

**Version:** Kira 0.12.0 (latest stable, docs.rs/kira/latest)
**Bevy Integration:** `bevy_kira_audio` v0.23 (supports Bevy 0.15)
**License:** MIT OR Apache-2.0

#### Core API Surface

| Struct | Role | Key Methods |
|--------|------|-------------|
| `AudioManager<DefaultBackend>` | Top-level controller. Owns the audio thread. | `::new(settings)`, `.play(sound_data)`, `.add_sub_track(settings)`, `.add_clock(settings)` |
| `StaticSoundData` | Pre-loaded audio buffer (entire file in memory). Appropriate for music tracks <60s or looping stems. | `::from_file(path)`, `.with_settings(settings)` |
| `StreamingSoundData` | Streaming from disk. Appropriate for long ambient tracks. | `::from_file(path)` |
| `TrackHandle` | Sub-mixer channel. Controls volume, panning, effects for all sounds routed to it. | `.set_volume(value, tween)`, `.set_panning(value, tween)`, `.play(sound_data)` |
| `ClockHandle` | Musical timing source. Ticks at configurable BPM. Events can be scheduled on clock ticks. | `::new(settings)`, `.set_speed(bpm, tween)` |
| `Tween` | Smooth value transition over duration with easing curve. | `Tween { start_time, duration, easing }` |
| `Easing` | Curve shape for tweens. | `Linear`, `InPowi(i32)`, `OutPowi(i32)`, `InOutPowi(i32)` |
| `Decibels` / `Volume` | Volume representation. | `Volume::Amplitude(f64)`, `Volume::Decibels(f64)` |

#### Adaptive Music Design Pattern

The standard pattern for adaptive game music in Kira:

1. **Create 8 sub-tracks** (one per mood layer) via `AudioManager::add_sub_track()`.
2. **Load 8 looping stems** as `StaticSoundData` (or `StreamingSoundData` for longer pieces).
3. **Play all 8 simultaneously** from game start, each routed to its own sub-track. Set initial volumes according to the starting game state.
4. **On game-state change**, call `track_handle.set_volume(target, tween)` on each track to crossfade between mood layers.
5. **Use ClockHandle** for beat-quantized transitions: schedule volume changes to land on the next beat boundary so crossfades sound musical rather than abrupt.

```rust
// Pseudocode: AudioPlugin setup
pub struct MusicLayer {
    track: TrackHandle,
    sound: StaticSoundHandle,
}

pub struct AdaptiveMusicPlugin;

impl Plugin for AdaptiveMusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicState>()
           .add_systems(Startup, setup_music_layers)
           .add_systems(Update, update_music_from_game_state);
    }
}

fn setup_music_layers(
    mut commands: Commands,
    mut audio_manager: ResMut<AudioManager<DefaultBackend>>,
) {
    let layers: Vec<MusicLayer> = MOOD_TRACKS.iter().map(|path| {
        let track = audio_manager.add_sub_track(TrackBuilder::default()).unwrap();
        let sound_data = StaticSoundData::from_file(path)
            .unwrap()
            .with_settings(StaticSoundSettings::new().loop_behavior(LoopBehavior::default()));
        let sound = track.play(sound_data).unwrap();
        MusicLayer { track, sound }
    }).collect();
    commands.insert_resource(MusicLayers(layers));
}

fn update_music_from_game_state(
    game_state: Res<GameState>,
    music_config: Res<MusicStateConfig>,
    layers: Res<MusicLayers>,
) {
    let volumes = music_config.volumes_for_state(&game_state);
    let tween = Tween {
        duration: Duration::from_secs(2),
        easing: Easing::InOutPowi(2),
        ..Default::default()
    };
    for (layer, &target_vol) in layers.0.iter().zip(volumes.iter()) {
        layer.track.set_volume(Volume::Amplitude(target_vol as f64), tween);
    }
}
```

#### Supported Audio Formats

Kira supports: OGG Vorbis, MP3, FLAC, WAV. For game music stems, **OGG Vorbis** is recommended (good compression, gapless looping, no patent issues).

### 2. Howler.js (Web Client)

**Version:** Howler.js 2.2.4 (latest, npm)
**License:** MIT
**Browser Support:** Chrome, Firefox, Safari, Edge, IE11 (Web Audio API primary, HTML5 Audio fallback)

#### Core API Surface

| Object | Role | Key Methods |
|--------|------|-------------|
| `Howl` | Sound instance. Loads and controls a single audio source. | `new Howl({src, loop, volume, html5})`, `.play()`, `.pause()`, `.stop()` |
| `Howl` (volume) | Per-sound volume control. | `.volume(val)`, `.fade(from, to, duration)` |
| `Howler` | Global controller. | `Howler.volume(val)`, `Howler.mute(bool)` |

#### Adaptive Music on Web

The web mirrors the Bevy pattern but uses Howler.js `Howl` instances instead of Kira tracks:

```javascript
// MusicManager.js
class AdaptiveMusicManager {
  constructor(trackPaths) {
    this.layers = trackPaths.map(path => ({
      howl: new Howl({
        src: [path],
        loop: true,
        volume: 0.0,
        html5: true,  // HTML5 audio for long music (lower memory)
      }),
    }));
  }

  start() {
    this.layers.forEach(layer => layer.howl.play());
  }

  transitionTo(targetVolumes, durationMs = 2000) {
    this.layers.forEach((layer, i) => {
      const currentVol = layer.howl.volume();
      layer.howl.fade(currentVol, targetVolumes[i], durationMs);
    });
  }
}
```

#### Web-Specific Considerations

- **Autoplay Policy:** Browsers block audio autoplay until user interaction. Music must start on first click/keypress. Use `Howler.ctx.resume()` after user gesture.
- **HTML5 vs Web Audio:** Use `html5: true` for music tracks (streaming, lower memory). Use Web Audio (default) for short SFX (lower latency).
- **Codec:** Use OGG Vorbis with MP3 fallback: `src: ['track.ogg', 'track.mp3']`.
- **Mobile:** Volume ducking on iOS when page is backgrounded. No programmatic volume control on iOS Safari for HTML5 Audio mode.

### 3. AI Music Generation (Asset Pipeline)

#### MusicGen (Meta AudioCraft) -- RECOMMENDED

**Repository:** `facebookresearch/audiocraft` (GitHub)
**License:** Apache-2.0 (code) + CC-BY-NC-4.0 (pretrained models for non-commercial) or MIT for Hydra II (commercial)
**Models:** `musicgen-small` (300M), `musicgen-medium` (1.5B), `musicgen-large` (3.3B)
**Hardware:** GPU required. `musicgen-small` runs on 8GB VRAM. `musicgen-large` needs 16GB+.
**Output:** 32kHz mono/stereo audio, up to 30s per generation.

**Strengths:**
- Local inference, no API costs, no rate limits.
- Text-conditioned: describe mood, tempo, instruments, and genre.
- Melody-conditioned: supply a reference melody to guide generation.
- Multi-band diffusion decoder available for higher quality output.

**Limitations:**
- 30s max per generation (can be extended with overlap-add stitching).
- Loopability not guaranteed; post-processing needed to create seamless loops (crossfade tail into head).
- Quality is good but not studio-grade. Adequate for game background music.

**Hydra II Alternative:** Rightsify's Hydra II is a MusicGen-based model trained entirely on licensed music. MIT license, commercially safe. Similar quality to `musicgen-medium`.

#### Suno API (Alternative, Paid)

**API:** REST, paid per generation.
**Quality:** Higher than MusicGen (closer to studio quality).
**License:** Commercial use allowed with paid plan.
**Cost:** ~$0.05/generation.

**Verdict:** MusicGen for development and MVP (free, local, good enough). Suno as upgrade path if higher quality music is needed for production release.

#### Track Generation Strategy

Generate 8 mood tracks:

| Track ID | Mood | Prompt Template |
|----------|------|-----------------|
| `base_calm` | Peaceful / Idle | "Calm ambient orchestral, gentle strings, 80 BPM, loopable" |
| `base_tense` | Tension rising | "Suspenseful orchestral, low brass, timpani rolls, 90 BPM" |
| `battle_low` | Minor skirmish | "Moderate battle music, snare drums, horns, 110 BPM" |
| `battle_high` | Major war | "Epic battle orchestral, full orchestra, choir, 130 BPM" |
| `prosperity` | Economic boom | "Triumphant fanfare, major key, brass and strings, 100 BPM" |
| `crisis` | Famine / collapse | "Dark ambient, minor key, cello solo, sparse percussion, 70 BPM" |
| `discovery` | New territory / tech | "Wonder and exploration, woodwinds, harp, gentle percussion, 85 BPM" |
| `diplomacy` | Negotiations | "Elegant courtly music, harpsichord, chamber strings, 95 BPM" |

Post-processing pipeline:
1. Generate 30s raw audio with MusicGen.
2. Normalize loudness to -14 LUFS (EBU R128).
3. Apply crossfade loop (2s fade at tail/head boundary).
4. Export as OGG Vorbis quality 6 (~160kbps).
5. Validate loop point with automated playback test.

---

## Decision

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Desktop audio engine | **Kira 0.12 via bevy_kira_audio** | Only mature Bevy audio plugin. Sub-track mixing, tweens, and clock handles provide all needed adaptive music primitives. |
| Web audio engine | **Howler.js 2.2** | De facto standard for browser game audio. Fade API, HTML5 streaming mode, broad browser support. |
| Music generation | **MusicGen (audiocraft)** for MVP | Free, local, Apache-2.0 code. Quality sufficient for game background music. Suno as paid upgrade path. |
| Track format | **OGG Vorbis** (primary) + MP3 (web fallback) | Good compression, gapless looping, patent-free. |
| Mixing architecture | **8 simultaneous looping layers** with volume automation | Industry-standard adaptive music pattern. Simple, predictable, easy to tune. |

---

## Implementation Contract

### AudioPlugin API (Bevy/Kira)

```rust
/// Resource: maps game states to per-layer volume targets.
#[derive(Resource)]
pub struct MusicStateConfig {
    /// Map from GameMood enum variant to array of 8 volume amplitudes [0.0..1.0].
    pub mood_volumes: HashMap<GameMood, [f32; 8]>,
    /// Default crossfade duration in seconds.
    pub crossfade_secs: f32,
    /// Easing curve exponent for crossfades.
    pub easing_power: i32,
}

/// The 8 mood categories that drive music mixing.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum GameMood {
    Calm,
    Tense,
    BattleLow,
    BattleHigh,
    Prosperity,
    Crisis,
    Discovery,
    Diplomacy,
}

/// Resource: holds the 8 active music layer handles.
#[derive(Resource)]
pub struct MusicLayers {
    pub layers: [MusicLayer; 8],
    pub clock: ClockHandle,
}

pub struct MusicLayer {
    pub track: TrackHandle,
    pub sound: StaticSoundHandle,
}

/// System: reads GameState, computes current GameMood, applies volume targets.
/// Runs in Update schedule at 4Hz (no need for per-frame updates).
pub fn update_music_from_game_state(
    game_state: Res<GameState>,
    config: Res<MusicStateConfig>,
    layers: Res<MusicLayers>,
) {
    // 1. Derive GameMood from GameState (war intensity, economy, diplomacy flags).
    // 2. Look up mood_volumes[mood].
    // 3. For each layer, call track.set_volume(target, tween).
    // 4. Tween uses config.crossfade_secs and config.easing_power.
}
```

### WebMusicManager API (Howler.js)

```typescript
interface MusicStateConfig {
  moodVolumes: Record<GameMood, number[]>;  // 8 volumes per mood
  crossfadeMs: number;                       // default 2000
}

type GameMood =
  | 'calm' | 'tense' | 'battle_low' | 'battle_high'
  | 'prosperity' | 'crisis' | 'discovery' | 'diplomacy';

class WebMusicManager {
  private layers: Howl[];
  private config: MusicStateConfig;

  constructor(trackUrls: string[], config: MusicStateConfig);
  start(): void;                              // Play all layers (call after user gesture)
  setMood(mood: GameMood): void;              // Crossfade to target volumes
  setMasterVolume(vol: number): void;         // 0.0..1.0
  pause(): void;
  resume(): void;
}
```

### MusicStateConfig Example

```json
{
  "mood_volumes": {
    "calm":        [1.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.2, 0.0],
    "tense":       [0.3, 1.0, 0.0, 0.0, 0.0, 0.3, 0.0, 0.0],
    "battle_low":  [0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    "battle_high": [0.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0],
    "prosperity":  [0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.3, 0.0],
    "crisis":      [0.0, 0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    "discovery":   [0.3, 0.0, 0.0, 0.0, 0.2, 0.0, 1.0, 0.0],
    "diplomacy":   [0.3, 0.0, 0.0, 0.0, 0.2, 0.0, 0.0, 1.0]
  },
  "crossfade_secs": 2.0,
  "easing_power": 2
}
```

### Asset Pipeline Contract

| Step | Tool | Input | Output | Deterministic? |
|------|------|-------|--------|----------------|
| Generate raw audio | MusicGen (`musicgen-medium`) | Text prompt + optional melody | 30s WAV @ 32kHz | No (generative AI) |
| Normalize loudness | FFmpeg `loudnorm` filter | Raw WAV | Normalized WAV @ -14 LUFS | Yes |
| Create loop | FFmpeg crossfade | Normalized WAV | Looped WAV | Yes |
| Encode OGG | FFmpeg `-c:a libvorbis -q:a 6` | Looped WAV | OGG Vorbis ~160kbps | Yes |
| Encode MP3 fallback | FFmpeg `-c:a libmp3lame -q:a 2` | Looped WAV | MP3 ~192kbps | Yes |
| Validate loop | Custom script (play 2x, check for click at boundary) | OGG file | Pass/fail | Yes |

### Acceptance Criteria

1. All 8 mood tracks load without error on both Bevy and web clients.
2. Crossfade from any mood to any other mood completes within `crossfade_secs` with no audible click or pop.
3. Music layers loop seamlessly with no audible gap at loop boundary.
4. Volume automation responds to game-state changes within 1 frame (16ms) of state change detection.
5. Web client handles browser autoplay policy: music starts only after first user interaction.
6. Total music asset size < 20MB (8 tracks * ~2.5MB OGG each).
7. CPU usage for audio mixing < 2% on reference hardware (M1 Mac, Chrome 120+).

---

## Detailed Technical Notes

### Kira ClockHandle for Beat-Synced Transitions

Kira's `ClockHandle` provides musical timing independent of wall-clock time. A clock ticks at a configurable BPM, and sounds/tweens can be scheduled relative to clock ticks rather than absolute time.

**Usage pattern for beat-quantized crossfades:**

```rust
// Create a clock at 120 BPM.
let clock_settings = ClockSettings::new().speed(ClockSpeed::TicksPerMinute(120.0));
let clock = audio_manager.add_clock(clock_settings).unwrap();

// Schedule a volume change to start on the next beat (next clock tick).
let next_tick = clock.time() + ClockTime::Ticks(1);
let tween = Tween {
    start_time: StartTime::ClockTime(next_tick),
    duration: Duration::from_secs(2),
    easing: Easing::InOutPowi(2),
};
layer.track.set_volume(Volume::Amplitude(0.8), tween);
```

This ensures crossfades align with musical beats, preventing the jarring effect of mid-phrase volume changes. The clock BPM should match the generated music BPM (stored in track metadata).

**Clock synchronization across layers:** All 8 layers should reference the same `ClockHandle` to ensure beat-aligned transitions. When transitioning from a 90 BPM track to a 130 BPM track, the clock speed itself can be tweened: `clock.set_speed(ClockSpeed::TicksPerMinute(130.0), speed_tween)`.

### Howler.js Web Audio API Gain Node Architecture

Howler.js exposes the underlying Web Audio API context via `Howler.ctx`. For advanced mixing beyond simple per-sound volume, you can tap into the gain node graph:

```javascript
// Access the master gain node.
const masterGain = Howler.masterGain;

// Create per-layer gain nodes for independent volume control.
const layerGains = trackPaths.map(() => {
  const gain = Howler.ctx.createGain();
  gain.connect(masterGain);
  return gain;
});

// Route each Howl to its corresponding gain node.
layers.forEach((layer, i) => {
  // Howler.js doesn't expose per-sound routing natively,
  // so use the Web Audio API directly for the connection.
  // This requires creating sounds with { html5: false } to use Web Audio.
});
```

For CivLab's web client, the simpler `Howl.fade()` API is sufficient for MVP. The gain node approach is documented here for future enhancement if sub-frame precision or per-layer effects (e.g., reverb on the ambient layer) are needed.

### MusicGen Generation Pipeline Details

**Model selection guidance:**

| Model | VRAM | Quality | Speed (30s generation) | Use Case |
|-------|------|---------|----------------------|----------|
| `musicgen-small` (300M) | 4GB | Acceptable | ~15s on A100 | Prototyping, iteration |
| `musicgen-medium` (1.5B) | 8GB | Good | ~30s on A100 | Production MVP |
| `musicgen-large` (3.3B) | 16GB | Best | ~60s on A100 | Final production tracks |
| Hydra II (1.5B) | 8GB | Good (commercially safe) | ~30s on A100 | Commercial release |

**Looping technique:**

MusicGen does not natively produce seamless loops. The post-processing pipeline must handle this:

```python
import numpy as np
import soundfile as sf

def create_seamless_loop(audio_path: str, output_path: str, crossfade_secs: float = 2.0):
    """Create a seamless loop from a MusicGen output."""
    audio, sr = sf.read(audio_path)
    crossfade_samples = int(crossfade_secs * sr)

    # Extract head and tail segments.
    tail = audio[-crossfade_samples:]
    head = audio[:crossfade_samples]

    # Create crossfade envelope.
    fade_out = np.linspace(1.0, 0.0, crossfade_samples)
    fade_in = np.linspace(0.0, 1.0, crossfade_samples)

    if audio.ndim == 2:  # stereo
        fade_out = fade_out[:, np.newaxis]
        fade_in = fade_in[:, np.newaxis]

    # Blend tail and head.
    crossfaded = tail * fade_out + head * fade_in

    # Construct loopable audio: crossfaded region + middle section.
    middle = audio[crossfade_samples:-crossfade_samples]
    looped = np.concatenate([crossfaded, middle])

    sf.write(output_path, looped, sr)
```

**Prompt engineering for game music:**

Effective MusicGen prompts for game music follow this structure:
```
[mood adjective] [genre] music, [instruments], [tempo] BPM, [key/tonality], loopable, game soundtrack
```

Examples:
- "Calm ambient orchestral music, gentle strings and harp, 80 BPM, C major, loopable, game soundtrack"
- "Epic battle orchestral music, full orchestra with choir and war drums, 130 BPM, D minor, loopable, game soundtrack"

The "loopable" keyword has no guaranteed effect on MusicGen's output but empirically produces outputs with more consistent endings that are easier to crossfade.

### Memory and Performance Budget

**Bevy/Kira client:**
- 8 OGG tracks at ~2.5MB each = 20MB compressed on disk.
- Decoded to PCM in memory: 8 tracks * 30s * 44.1kHz * 2ch * 4 bytes = ~42MB RAM.
- Using `StaticSoundData` (all in memory): total ~42MB for music.
- Using `StreamingSoundData` (streaming from disk): ~1MB buffer per track = ~8MB total.
- Recommendation: Use `StaticSoundData` for tracks under 30s (gapless looping guarantee). Use `StreamingSoundData` for ambient layers over 60s.

**Web client (Howler.js):**
- HTML5 Audio mode: streams from server, minimal memory footprint (~1-2MB per track buffer).
- Web Audio mode: decodes entire track into AudioBuffer, ~42MB total (same as Bevy).
- Recommendation: Use `html5: true` for music tracks on web (streaming, lower memory).

**CPU budget:**
- Kira audio mixing: <1% CPU on M1 Mac for 8 simultaneous tracks.
- Howler.js: delegated to browser's audio thread, negligible JS main thread cost.
- Volume tween calculations: ~100 float operations per frame, negligible.

## Open Questions Remaining

1. **Beat-quantized transitions:** Should crossfades snap to beat boundaries via ClockHandle, or is smooth time-based crossfading sufficient for the first implementation? Recommend starting with time-based and adding beat-sync as a polish pass.

2. **Stinger system:** Short one-shot audio stingers (e.g., "battle started" brass hit) layered on top of the adaptive mix. Not covered here; should be a separate SFX system using `StaticSoundData` on a dedicated effects track.

3. **Dynamic tempo:** Should battle intensity also affect music tempo (via `PlaybackRate`), or only volume layers? Tempo changes risk making music sound unnatural. Recommend volume-only for MVP.

4. **Per-biome ambient layers:** Forest/desert/ocean ambient soundscapes as additional non-music layers. Architectural extension of the same system (additional tracks), but needs separate mood mapping.

5. **MusicGen model licensing:** The pretrained models are CC-BY-NC-4.0 (non-commercial). For commercial release, either use Hydra II (MIT) or train on licensed music. This is a production-release blocker, not an MVP blocker.

6. **Spatial audio:** Kira supports spatial audio (3D positioning of sounds relative to a listener). This is relevant for SFX (e.g., battle sounds from the direction of conflict) but not for background music. Document spatial audio API for the SFX system research.

7. **Audio format fallback chain:** OGG is not supported in all Safari versions. The fallback chain should be: OGG -> MP3 -> WAV. Howler.js handles this natively via the `src` array. Verify Safari 17+ OGG support status before launch.

---

## Sources

- Kira 0.12 API documentation: https://docs.rs/kira/latest/kira/
- bevy_kira_audio: https://github.com/NiklasEi/bevy_kira_audio
- Howler.js: https://howlerjs.com/
- Meta AudioCraft / MusicGen: https://github.com/facebookresearch/audiocraft
- MusicGen model card: https://huggingface.co/facebook/musicgen-large
- Hydra II (Rightsify): commercially-licensed MusicGen variant
- Web Audio API autoplay policy: https://developer.chrome.com/blog/autoplay


---

## Source: research/RND-011-mcts-ai-feasibility.md

# RND-011: MCTS for Game AI -- Implementation Approach for CivLab Nation AI

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**MCTS is feasible for CivLab's difficulty 4-5 AI** with the following key design decisions:

1. **Paranoid MCTS** (not max-n or coalition) for multi-nation scenarios: treats all opponents
   as adversarial, producing robust/conservative play appropriate for a strategy game AI.
2. **Node-count bounded** (not time-bounded) for determinism: budget of N=5000-10000 nodes
   per decision, yielding consistent behavior regardless of hardware speed.
3. **Compressed state representation** (~10KB per MCTS node): extract only nation-relevant
   state (economy summary, military units, diplomatic relations, resource levels) -- not the
   full ECS World.
4. **Action space pruning** via utility-scored pre-selection: limit to top-K=20 candidate
   actions per node (from potentially 1000+ legal actions) using a fast heuristic evaluator.
5. **Simplified rollout policy**: 10-tick lookahead using a lightweight "fast-forward" model
   that approximates the full simulation with ~5 key equations (GDP growth, military strength
   delta, food balance, research progress, diplomatic tension).
6. **No parallelization for determinism**: single-threaded MCTS. The 100ms budget is met by
   limiting node count, not by parallelizing.

---

## Research Findings

### 1. MCTS Fundamentals for Strategy Games

#### 1.1 Standard MCTS (Single-Player / Two-Player)

The classic MCTS algorithm has four phases per iteration:
1. **Selection**: Traverse tree from root, picking children via UCB1 (or variant).
2. **Expansion**: Add a new child node for an unexplored action.
3. **Rollout (Simulation)**: Play random/heuristic moves from the new node to a terminal
   state or depth limit.
4. **Backpropagation**: Update win/visit statistics from the new node back to root.

UCB1 selection formula:
```
UCB1(child) = Q(child)/N(child) + C * sqrt(ln(N(parent)) / N(child))
```
Where Q = total reward, N = visit count, C = exploration constant (typically sqrt(2)).

#### 1.2 Multi-Player MCTS Variants

CivLab has 2-8 nations. Standard 2-player MCTS (minimax assumption) doesn't generalize
directly. Three main approaches:

| Variant | Description | Pros | Cons |
|---------|-------------|------|------|
| **Max-n** | Each node stores per-player rewards. Selection maximizes current player's reward. | Theoretically optimal for n-player games. | Assumes opponents play optimally for themselves. In practice, opponents may form coalitions or play suboptimally, making max-n overfit. |
| **Paranoid** | All opponents modeled as a single adversary trying to minimize the AI's reward. Reduces to 2-player minimax. | Conservative, robust play. Simple implementation (just negate reward for opponent turns). Good when opponents are threatening. | May miss cooperative/exploitative opportunities. Overly defensive in games with natural alliances. |
| **Coalition** | Dynamically models alliances. Opponents split into "with us" and "against us" groups. | More realistic for diplomacy-heavy games. | Complex implementation. Coalition detection is itself a hard problem. Unstable coalitions cause tree inconsistency. |

**Recommendation: Paranoid MCTS.**

Rationale:
- CivLab's AI difficulties 4-5 should play **competitively**, not exploitatively. Paranoid
  assumption produces an AI that defends well and doesn't take foolish risks.
- Coalition dynamics in CivLab are handled by the diplomacy system (RND-TBD), not by MCTS
  search. The AI's diplomatic decisions (ally, trade, declare war) are actions in the MCTS
  tree, not structural assumptions.
- Max-n is theoretically better but requires accurate modeling of each opponent's utility
  function. In practice, the AI doesn't know opponents' goals precisely enough for max-n
  to outperform paranoid.
- Implementation simplicity: paranoid MCTS is identical to 2-player MCTS with opponent turns
  interleaved.

Research backing: Nijssen's thesis on multi-player MCTS found that paranoid MCTS performs
comparably to max-n in most multi-player games, and significantly better when opponents have
hidden information or are modeled imprecisely.

#### 1.3 UCB1 vs PUCT

| Algorithm | Formula | Use Case |
|-----------|---------|----------|
| **UCB1** | `Q/N + C * sqrt(ln(N_parent) / N)` | No prior knowledge about action quality |
| **PUCT** | `Q/N + C * P(a) * sqrt(N_parent) / (1 + N)` | With prior probability P(a) from a heuristic/network |

**Recommendation: PUCT** (Polynomial UCT variant, as used in AlphaGo/AlphaZero).

Rationale:
- CivLab's action space is large (100-1000+ actions per turn). With UCB1, the algorithm must
  visit every action at least once before focusing -- with 1000 actions, the first 1000
  iterations are wasted on uniform exploration.
- PUCT uses a **prior probability** `P(a)` for each action, which biases exploration toward
  promising actions immediately. The prior comes from our utility-scoring heuristic (see
  Section 3), not from a neural network.
- PUCT with a good heuristic prior dramatically improves search efficiency in large action
  spaces. AlphaGo showed this; it applies equally to strategy games.

Modified PUCT for CivLab:
```
PUCT(a) = Q(a)/N(a) + C_puct * P(a) * sqrt(N_parent) / (1 + N(a))
```
Where:
- `P(a)` = prior probability from heuristic utility scoring (normalized to sum to 1.0)
- `C_puct` = exploration constant, tunable (start with 1.5, tune via self-play)
- `Q(a)` = average reward (fixed-point, see RND-003)
- `N(a)` = visit count for action a
- `N_parent` = visit count for parent node

### 2. State Representation

#### 2.1 The Full State Problem

CivLab's full simulation state includes:
- All entities in the ECS World (10k-100k entities with multiple components each)
- Terrain map (hex grid with per-tile data)
- Diplomatic relations (N x N matrix)
- Technology trees (per nation)
- Event queues, RNG state, tick counter

**Estimated size:** 1-10MB for a mid-game state. Copying this per MCTS node is infeasible
for 10k nodes.

#### 2.2 Compressed Nation State

For MCTS lookahead, the AI doesn't need full simulation fidelity. It needs to estimate
the **relative advantage** of different strategic choices. A compressed representation:

```rust
/// Compressed state for MCTS node. ~10KB total.
#[derive(Clone, Debug)]
pub struct MctsState {
    /// Which nation is making the decision
    pub acting_nation: NationId,

    /// Per-nation summaries (2-8 nations)
    pub nations: Vec<NationSummary>,

    /// Simplified diplomatic relations
    pub relations: Vec<(NationId, NationId, RelationScore)>,

    /// Current game tick
    pub tick: u64,

    /// Deterministic RNG state for rollouts
    pub rng_seed: u64,
}

/// Summary of a single nation's state. ~1KB per nation.
#[derive(Clone, Debug)]
pub struct NationSummary {
    pub id: NationId,
    pub population: i64,
    pub gdp: i64,                    // milli-credits
    pub food_balance: i64,           // kJ surplus/deficit per tick
    pub military_strength: i64,      // aggregate military power score
    pub research_progress: i64,      // total research points
    pub territory_size: i32,         // number of controlled hexes
    pub happiness: i32,              // fixed-point (Ratio bits)
    pub strategic_resources: [i64; 8], // key resource stockpiles
}

/// Diplomatic relation score between two nations.
/// Negative = hostile, positive = friendly.
pub type RelationScore = i32;
```

**Size analysis:**
- `NationSummary`: ~(8 + 8 + 8 + 8 + 8 + 8 + 4 + 4 + 64) = ~120 bytes per nation
- 8 nations: ~960 bytes
- Relations: 8*7/2 = 28 pairs * 12 bytes = ~336 bytes
- Overhead: ~200 bytes
- **Total: ~1.5 KB per MCTS node** (much better than the 10KB estimate)

At 10,000 nodes: ~15 MB total. Acceptable.

#### 2.3 State Extraction

```rust
/// Extract compressed MCTS state from the full ECS World.
/// Called once at the start of each AI decision.
pub fn extract_mcts_state(
    world: &World,
    acting_nation: NationId,
    tick: u64,
    rng_seed: u64,
) -> MctsState {
    let mut nations = Vec::new();

    // Query all nation entities and their summary components
    let mut query = world.query::<(
        &Nation,
        &Economy,
        &Military,
        &Research,
        &Territory,
        &Happiness,
        &ResourceStockpile,
    )>();

    for (nation, economy, military, research, territory, happiness, resources) in query.iter(world) {
        nations.push(NationSummary {
            id: nation.id,
            population: economy.population,
            gdp: economy.gdp,
            food_balance: economy.food_balance_per_tick,
            military_strength: military.aggregate_strength(),
            research_progress: research.total_points,
            territory_size: territory.hex_count,
            happiness: happiness.score.to_bits(),
            strategic_resources: resources.summarize(),
        });
    }

    // Sort nations by ID for determinism
    nations.sort_by_key(|n| n.id);

    // Extract diplomatic relations
    let relations = extract_relations(world);

    MctsState { acting_nation, nations, relations, tick, rng_seed }
}
```

### 3. Action Space and Pruning

#### 3.1 Action Types in CivLab

A nation's available actions per decision point include:

| Category | Example Actions | Cardinality |
|----------|-----------------|-------------|
| Military | Move unit, attack, recruit, fortify | ~50-500 (depends on army size) |
| Economic | Build improvement, set tax rate, trade | ~20-100 |
| Diplomatic | Propose alliance, declare war, trade deal | ~10-50 |
| Research | Choose tech, prioritize branch | ~5-20 |
| Domestic | Set policy, assign governor, event response | ~10-30 |
| **Total** | | **~100-700 per turn** |

With compound actions (multiple orders per turn), the space grows combinatorially.

#### 3.2 Pruning Strategy: Utility-Scored Pre-Selection

**Do not expand all actions in the MCTS tree.** Instead:

1. **Generate all legal actions** for the acting nation.
2. **Score each action** with a fast heuristic utility function (~1us per action).
3. **Select top-K actions** (K=15-25) to include in the MCTS tree.
4. The remaining actions are **pruned** -- never explored by MCTS.

This is conceptually similar to PUCT's prior probability but applied as a hard cutoff
rather than a soft bias.

```rust
/// Score an action using a fast heuristic. Higher = more promising.
/// This is NOT the MCTS reward -- it's a prior estimate for action selection.
pub fn score_action(state: &MctsState, action: &Action) -> i64 {
    match action {
        Action::Attack { target, strength } => {
            // Heuristic: attack value = expected damage - expected loss
            let target_defense = state.nation_summary(target.nation).military_strength;
            let advantage = *strength as i64 - target_defense;
            advantage * 100 // Scale to make comparable with other actions
        }
        Action::BuildImprovement { hex, improvement_type } => {
            // Heuristic: build value = expected yield increase
            improvement_type.expected_yield_increase() * 50
        }
        Action::DeclareWar { target } => {
            // Heuristic: war value based on relative power
            let us = state.nation_summary(state.acting_nation);
            let them = state.nation_summary(*target);
            (us.military_strength - them.military_strength) * 80
        }
        Action::Research { tech } => {
            // Heuristic: research value = tech's strategic weight
            tech.strategic_value() * 60
        }
        // ... other action types
        _ => 0, // Default: neutral priority
    }
}

/// Select top-K actions for MCTS expansion.
pub fn select_candidate_actions(
    state: &MctsState,
    all_actions: &[Action],
    k: usize,
) -> Vec<(Action, i64)> {
    let mut scored: Vec<_> = all_actions.iter()
        .map(|a| (a.clone(), score_action(state, a)))
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top K
    scored.truncate(k);
    scored
}
```

#### 3.3 Prior Probability for PUCT

Convert utility scores to probabilities for PUCT:

```rust
/// Convert raw utility scores to PUCT prior probabilities.
/// Uses softmax-like normalization (integer approximation).
pub fn scores_to_priors(scored_actions: &[(Action, i64)]) -> Vec<(Action, i32)> {
    if scored_actions.is_empty() {
        return Vec::new();
    }

    // Shift scores so minimum is 0 (avoid negative in "softmax")
    let min_score = scored_actions.iter().map(|(_, s)| *s).min().unwrap_or(0);
    let shifted: Vec<i64> = scored_actions.iter().map(|(_, s)| s - min_score + 1).collect();

    // Normalize to sum to 1000 (fixed-point probability, 0.1% resolution)
    let total: i64 = shifted.iter().sum();
    scored_actions.iter()
        .zip(shifted.iter())
        .map(|((action, _), &s)| {
            let prior = (s * 1000 / total) as i32; // 0-1000 range
            (action.clone(), prior.max(1)) // minimum prior of 1 (0.1%)
        })
        .collect()
}
```

### 4. Rollout Policy

#### 4.1 The Rollout Problem

Standard MCTS rollouts play to a terminal state using random moves. For CivLab:
- A game can last 1000+ ticks.
- Each tick involves the full simulation (ECS systems, climate, economy, military...).
- Running even 10 ticks of the full simulation per MCTS node is too expensive (10ms per
  tick * 10 ticks * 10,000 nodes = 1000 seconds).

#### 4.2 Fast-Forward Approximation

Instead of running the full ECS simulation, use a **simplified mathematical model** that
approximates 10 ticks of game progression in ~1-10us:

```rust
/// Fast-forward the compressed state by `ticks` steps.
/// This is a simplified model -- NOT the full simulation.
/// Accuracy is secondary to speed; the MCTS statistics average out errors.
pub fn fast_forward(state: &mut MctsState, ticks: u32, rng: &mut DeterministicRng) {
    for _ in 0..ticks {
        for nation in &mut state.nations {
            fast_forward_nation(nation, &state.relations, rng);
        }
        fast_forward_relations(&mut state.relations, rng);
        state.tick += 1;
    }
}

/// Simplified per-nation tick. ~5 key equations.
fn fast_forward_nation(
    nation: &mut NationSummary,
    relations: &[(NationId, NationId, RelationScore)],
    rng: &mut DeterministicRng,
) {
    // 1. Population growth: logistic model
    //    pop_delta = growth_rate * pop * (1 - pop/carrying_capacity)
    let carrying_capacity = nation.territory_size as i64 * 1000; // rough estimate
    let growth_rate: i64 = 5; // 0.5% per tick (scaled by 1000)
    let pop_delta = growth_rate * nation.population / 1000
        * (carrying_capacity - nation.population) / carrying_capacity;
    nation.population = (nation.population + pop_delta).max(0);

    // 2. GDP growth: proportional to population and research
    let gdp_growth_rate = 10 + nation.research_progress / 10000; // base 1% + tech bonus
    nation.gdp += nation.gdp * gdp_growth_rate / 1000;

    // 3. Food balance: territory * base_yield - population * consumption_rate
    let food_production = nation.territory_size as i64 * 500; // kJ per hex per tick
    let food_consumption = nation.population * 3; // 3 kJ per person per tick
    nation.food_balance = food_production - food_consumption;

    // 4. Military: slow decay if not at war, slow growth from GDP
    let military_budget = nation.gdp / 100; // 10% of GDP
    nation.military_strength += military_budget / 1000 - nation.military_strength / 500;

    // 5. Happiness: function of food balance and military safety
    let food_factor = (nation.food_balance / 100).clamp(-100, 100);
    nation.happiness = (nation.happiness as i64 + food_factor).clamp(0, 1000) as i32;
}

/// Simplified diplomatic evolution.
fn fast_forward_relations(
    relations: &mut [(NationId, NationId, RelationScore)],
    rng: &mut DeterministicRng,
) {
    for (_, _, score) in relations.iter_mut() {
        // Relations drift toward neutral with small random perturbation
        let drift = -(*score / 100); // mean reversion
        let noise = rng.next_range(-5, 5);
        *score = (*score + drift + noise).clamp(-1000, 1000);
    }
}
```

#### 4.3 Rollout Evaluation

After fast-forwarding, evaluate the resulting state:

```rust
/// Evaluate the state from the perspective of the acting nation.
/// Returns a score in [0, 1000] where 1000 = winning, 0 = losing.
pub fn evaluate_state(state: &MctsState) -> i32 {
    let us = state.nation_summary(state.acting_nation);
    let max_gdp = state.nations.iter().map(|n| n.gdp).max().unwrap_or(1);
    let max_mil = state.nations.iter().map(|n| n.military_strength).max().unwrap_or(1);
    let max_pop = state.nations.iter().map(|n| n.population).max().unwrap_or(1);

    // Weighted relative standing
    let gdp_score = (us.gdp * 300 / max_gdp.max(1)) as i32;        // 0-300
    let mil_score = (us.military_strength * 300 / max_mil.max(1)) as i32; // 0-300
    let pop_score = (us.population * 200 / max_pop.max(1)) as i32;  // 0-200
    let hap_score = us.happiness / 5;                                  // 0-200

    (gdp_score + mil_score + pop_score + hap_score).clamp(0, 1000)
}
```

### 5. Deterministic Bounding

#### 5.1 Why Not Wall-Clock Time

```rust
// BAD: Non-deterministic -- varies by hardware speed
while start.elapsed() < Duration::from_millis(100) {
    mcts.iterate();
}

// GOOD: Deterministic -- always same number of iterations
for _ in 0..NODE_BUDGET {
    mcts.iterate();
}
```

Using `std::time::Instant` makes the MCTS output depend on CPU speed. A fast machine
explores more nodes and makes better decisions than a slow machine -- breaking determinism
and creating unfair multiplayer.

#### 5.2 Node Budget Calibration

Target: complete MCTS within ~100ms on the reference hardware (mid-range 2024 CPU, single
thread).

**Per-node cost estimate:**
- State copy: ~1.5KB memcpy = ~100ns
- Action generation + scoring: ~5-20us (20 actions * 1us each)
- PUCT selection: ~1us (scan 20 children)
- Rollout (10 ticks fast-forward): ~10-50us
- Backpropagation: ~0.5us
- **Total per node: ~20-70us**

**Budget calculation:**
- 100ms / 50us average = ~2,000 nodes minimum
- 100ms / 20us average = ~5,000 nodes maximum
- **Recommended budget: 5,000 nodes** for difficulty 4
- **Recommended budget: 10,000 nodes** for difficulty 5

At 10,000 nodes * 50us = 500ms -- this exceeds 100ms on average hardware. Options:
1. Accept 200-500ms AI turns at difficulty 5 (strategy games are turn-based, players wait).
2. Reduce rollout depth from 10 to 5 ticks.
3. Reduce K from 20 to 10 candidate actions.
4. Optimize fast-forward model.

**Recommendation:** Start with 5,000 nodes, profile, and tune.

### 6. Parallelization Analysis

#### 6.1 Parallelization Approaches

| Method | Description | Deterministic? | Speedup |
|--------|-------------|---------------|---------|
| **Root parallelization** | Run N independent MCTS trees, merge statistics. | Yes (if merge is deterministic) | ~linear in N |
| **Leaf parallelization** | Parallelize rollouts at leaf nodes. | Yes (each rollout independent) | ~linear in batch size |
| **Tree parallelization** | Multiple threads traverse/expand same tree with locks or virtual loss. | NO -- thread scheduling affects exploration order. | Best theoretical speedup but non-deterministic. |

#### 6.2 Recommendation: No Parallelization

For CivLab, **do not parallelize MCTS**. Rationale:

1. **Determinism is the top priority.** Tree parallelization is inherently non-deterministic.
   Root parallelization can be deterministic but doubles memory usage.
2. **Budget is node-count-based, not time-based.** Parallelism doesn't help with a fixed
   node budget -- it just finishes faster. Since we're not time-bounded, there's no benefit.
3. **Simplicity.** Single-threaded MCTS is dramatically easier to debug, test, and reason
   about.
4. **MCTS runs on AI turn, not every frame.** A 200ms AI decision in a turn-based game is
   imperceptible. Parallelism solves a non-problem.

If performance becomes an issue in the future (e.g., real-time mode), root parallelization
with deterministic merging is the safe upgrade path.

### 7. Deterministic RNG

MCTS rollouts require randomness for the rollout policy. This must be deterministic:

```rust
/// Deterministic PRNG for MCTS rollouts.
/// Uses xorshift64 for speed and simplicity.
/// Seeded per MCTS search from the game's master RNG.
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) } // Avoid zero state
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Random value in [min, max] inclusive.
    pub fn next_range(&mut self, min: i64, max: i64) -> i64 {
        let range = (max - min + 1) as u64;
        min + (self.next() % range) as i64
    }
}
```

---

## Decision

Implement **Paranoid MCTS with PUCT selection, node-count bounding, compressed state,
utility-scored action pruning, and simplified rollout model.** Single-threaded, fully
deterministic. No neural network -- heuristic priors only.

---

## Implementation Contract

### Data Structures

```rust
/// MCTS tree node.
#[derive(Debug)]
pub struct MctsNode {
    /// The action that led to this node (None for root).
    pub action: Option<Action>,

    /// Visit count.
    pub visits: u32,

    /// Total reward accumulated (integer, 0-1000 scale per visit).
    pub total_reward: i64,

    /// Prior probability from heuristic (0-1000 scale, see scores_to_priors).
    pub prior: i32,

    /// Children (expanded actions).
    pub children: Vec<MctsNode>,

    /// Compressed game state at this node.
    /// Only stored for expanded nodes. None for leaf/unexpanded.
    pub state: Option<MctsState>,

    /// Whether this is the acting nation's turn (for paranoid negation).
    pub is_acting_turn: bool,
}
```

### Core MCTS Loop

```rust
/// Top-level MCTS search. Returns the best action.
pub fn mcts_search(
    initial_state: MctsState,
    node_budget: u32,
    rollout_depth: u32,
    top_k_actions: usize,
    c_puct: i32, // exploration constant, 0-1000 scale (1500 = 1.5)
) -> Action {
    let mut root = MctsNode::new_root(initial_state.clone());

    // Expand root with candidate actions
    let actions = generate_actions(&initial_state);
    let candidates = select_candidate_actions(&initial_state, &actions, top_k_actions);
    let priors = scores_to_priors(&candidates);
    root.expand(priors, &initial_state);

    // Main MCTS loop: fixed node count for determinism
    let mut rng = DeterministicRng::new(initial_state.rng_seed);

    for _ in 0..node_budget {
        // Selection: traverse tree using PUCT
        let path = select_leaf(&mut root, c_puct);

        // Expansion: add children to leaf if not terminal
        let leaf = follow_path_mut(&mut root, &path);
        if leaf.visits > 0 && leaf.children.is_empty() {
            let leaf_state = leaf.state.as_ref().unwrap();
            let leaf_actions = generate_actions(leaf_state);
            let candidates = select_candidate_actions(leaf_state, &leaf_actions, top_k_actions);
            let priors = scores_to_priors(&candidates);
            leaf.expand(priors, leaf_state);
        }

        // Rollout: fast-forward from leaf state
        let leaf = follow_path(&root, &path);
        let mut rollout_state = leaf.state.as_ref().unwrap().clone();
        fast_forward(&mut rollout_state, rollout_depth, &mut rng);
        let reward = evaluate_state(&rollout_state);

        // Backpropagation: update statistics along the path
        backpropagate(&mut root, &path, reward);
    }

    // Select best action: highest visit count (most robust)
    root.best_child_action()
}
```

### PUCT Selection

```rust
/// Select a leaf node using PUCT. Returns path of child indices.
fn select_leaf(root: &MctsNode, c_puct: i32) -> Vec<usize> {
    let mut path = Vec::new();
    let mut node = root;

    while !node.children.is_empty() {
        let parent_visits = node.visits;
        let sqrt_parent = integer_sqrt(parent_visits as i64);

        let best_idx = node.children.iter().enumerate()
            .max_by_key(|(_, child)| {
                if child.visits == 0 {
                    // Unvisited: high priority, biased by prior
                    return i64::MAX / 2 + child.prior as i64;
                }

                // Q value: average reward (0-1000 scale)
                let q = child.total_reward / child.visits as i64;

                // Paranoid: negate reward for opponent turns
                let q_adjusted = if node.is_acting_turn { q } else { 1000 - q };

                // PUCT exploration term
                let exploration = c_puct as i64 * child.prior as i64
                    * sqrt_parent / (1000 * (1 + child.visits as i64));

                q_adjusted + exploration
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        path.push(best_idx);
        node = &node.children[best_idx];
    }

    path
}

/// Integer square root (floor). Deterministic, no f64.
fn integer_sqrt(n: i64) -> i64 {
    if n <= 0 { return 0; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
```

### Backpropagation

```rust
/// Backpropagate reward along the path from leaf to root.
fn backpropagate(root: &mut MctsNode, path: &[usize], reward: i32) {
    root.visits += 1;
    root.total_reward += reward as i64;

    let mut node = root;
    for &idx in path {
        node = &mut node.children[idx];
        node.visits += 1;
        node.total_reward += reward as i64;
    }
}
```

### Best Action Selection

```rust
impl MctsNode {
    /// Select the action with the highest visit count (most robust selection).
    /// Ties broken by total reward (deterministic due to sorted comparison).
    pub fn best_child_action(&self) -> Action {
        self.children.iter()
            .max_by_key(|child| (child.visits, child.total_reward))
            .expect("MCTS root has no children")
            .action.clone()
            .expect("Root child has no action")
    }
}
```

### Integration with ECS

```rust
use bevy_ecs::prelude::*;

/// System that runs MCTS AI for nations at difficulty 4-5.
/// Called once per nation per AI decision point.
pub fn run_mcts_ai(
    world: &World,
    nation_id: NationId,
    difficulty: u8,
    tick: u64,
    rng: &mut DeterministicRng,
) -> Vec<Action> {
    // Difficulty-based parameters
    let (node_budget, rollout_depth, top_k) = match difficulty {
        4 => (5_000, 8, 15),
        5 => (10_000, 10, 20),
        _ => unreachable!("MCTS only for difficulty 4-5"),
    };

    // Extract compressed state from ECS
    let mcts_state = extract_mcts_state(world, nation_id, tick, rng.next());

    // Run MCTS
    let best_action = mcts_search(
        mcts_state,
        node_budget,
        rollout_depth,
        top_k,
        1500, // C_puct = 1.5
    );

    vec![best_action]
}
```

### Configuration Constants

```rust
/// MCTS configuration. All values are deterministic (no time-based bounds).
pub struct MctsConfig {
    /// Maximum nodes to expand per search.
    pub node_budget: u32,

    /// Number of ticks to fast-forward in rollouts.
    pub rollout_depth: u32,

    /// Number of candidate actions to consider per node.
    pub top_k_actions: usize,

    /// PUCT exploration constant (0-1000 scale; 1500 = 1.5).
    pub c_puct: i32,
}

impl MctsConfig {
    pub fn difficulty_4() -> Self {
        Self { node_budget: 5_000, rollout_depth: 8, top_k_actions: 15, c_puct: 1500 }
    }

    pub fn difficulty_5() -> Self {
        Self { node_budget: 10_000, rollout_depth: 10, top_k_actions: 20, c_puct: 1500 }
    }
}
```

### Memory Management

```rust
/// MCTS tree memory estimation:
/// - MctsNode: ~100 bytes + children Vec + Option<MctsState>
/// - MctsState: ~1.5 KB
/// - Average children per node: ~10 (after pruning)
///
/// At 10,000 nodes:
/// - Nodes: 10,000 * 100 bytes = 1 MB
/// - States: 10,000 * 1.5 KB = 15 MB (worst case; leaf nodes can drop state)
/// - Total: ~16 MB
///
/// Optimization: only store state at unexpanded leaf nodes (not internal nodes,
/// since internal node states can be reconstructed by replaying actions from root).
/// This reduces state storage to ~branching_factor * leaf_count * 1.5 KB.
///
/// For 10,000 nodes with branching factor 15:
/// - ~667 leaf nodes * 1.5 KB = ~1 MB
/// - Total with optimization: ~2 MB
///
/// MCTS memory is allocated at search start and freed at search end.
/// No persistent state between AI decisions.
```

---

## Open Questions Remaining

1. **Fast-forward model accuracy:** The simplified 5-equation model is a rough approximation.
   How much does MCTS quality degrade compared to using the full simulation? Need to validate
   by playing MCTS-with-approximate-rollout vs MCTS-with-full-rollout (expensive but
   informative one-time test). If approximate rollout is too inaccurate, consider:
   - Increasing rollout depth to compensate
   - Adding more equations (trade, climate effects)
   - Using a trained value function instead of rollouts (requires training infrastructure)

2. **C_puct tuning:** The exploration constant 1.5 is a starting point. Optimal value depends
   on the game dynamics and heuristic quality. Tune via self-play experiments. Consider
   different C_puct values for different game phases (early game: explore more; late game:
   exploit more).

3. **Action representation:** The `Action` type needs a full spec. How are compound actions
   (e.g., "build improvement AND move unit AND change tax rate") represented? Options:
   - Single-action per MCTS decision: nation makes one action per tick. Simple but slow.
   - Action sequence per turn: MCTS searches over sequences of 3-5 actions. Combinatorial
     explosion. Needs aggressive pruning.
   - Factored MCTS: separate MCTS trees for military, economic, diplomatic decisions. Results
     merged. Avoids combinatorics but misses cross-domain synergies.

4. **Multi-nation turn order:** In a single tick, nations act in order. The MCTS tree must
   model this: after our nation's action, the next nation acts, then the next, etc. With 8
   nations, depth-1 in the tree = 8 sequential actions. This makes the tree very deep with
   few visits per node. Mitigation: collapse opponent turns (assume opponents take their
   highest-utility action without MCTS search) and only search the acting nation's decisions.

5. **Difficulty 1-3 AI:** Lower difficulties use simpler AI (heuristic-only, no MCTS).
   Ensure the heuristic scoring function (used for MCTS priors) is independently useful as
   a standalone AI for difficulties 1-3. This avoids maintaining two completely separate AI
   codebases.

6. **Progressive widening:** For very large action spaces (>100 candidate actions even after
   pruning), consider progressive widening: start with K=5 actions and gradually widen to
   K=20 as the node gets more visits. This focuses early search on the most promising actions.

---

## References

- [Monte Carlo Tree Search (Wikipedia)](https://en.wikipedia.org/wiki/Monte_Carlo_tree_search)
- [Nijssen -- Monte-Carlo Tree Search for Multi-Player Games (PhD thesis)](https://project.dke.maastrichtuniversity.nl/games/files/phd/Nijssen_thesis.pdf)
- [Parallel Monte-Carlo Tree Search (Winands et al.)](https://dke.maastrichtuniversity.nl/m.winands/documents/multithreadedMCTS2.pdf)
- [MCTS review: recent modifications and applications (Springer)](https://link.springer.com/article/10.1007/s10462-022-10228-y)
- [AlphaGo/AlphaZero PUCT formula](https://www.chessprogramming.org/UCT)
- [Memory Bounded MCTS (AAAI)](https://cdn.aaai.org/ojs/12932/12932-52-16449-1-2-20201228.pdf)
- [Parametric Action Pre-Selection for MCTS in RTS Games](https://ceur-ws.org/Vol-2719/paper11.pdf)
- [Tabletop Games MCTS framework](https://tabletopgames.ai/wiki/agents/MCTS.html)
- [zxqfl/mcts -- Generic parallel MCTS in Rust](https://github.com/zxqfl/mcts)


---

## Source: research/RND-015-simulation-patterns-reference.md

# RND-015: Victoria 3 / Dwarf Fortress Simulation Patterns -- Academic Literature and Open Implementation References

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-delta

---

## Executive Summary

This document surveys simulation design patterns from three primary game references (Victoria 3, Dwarf Fortress, OpenTTD) and four foundational academic models (Sugarscape, Schelling, SIR, Hammond-Axelrod) to derive concrete design contracts for CivLab. Each source is annotated with the specific CivLab system it informs, the tunable parameters it implies, and the mathematical formulations that should be implemented. The result is a reference library of simulation patterns that CivLab's designers and implementers can draw from, with explicit contracts mapping academic theory to CivLab's domain.

---

## Research Findings

### Part I: Game Reference Systems

---

### 1. Victoria 3 -- Population and Economy Simulation

**Source:** Paradox Interactive dev diaries (2021-2025), Paradox Wiki, Mikael Andersson's GDC/Gamasutra deep dive on V3 economy.

#### 1.1 Pop System

Victoria 3's fundamental simulation unit is the "Pop" (population group). Unlike Dwarf Fortress's individual-level simulation, V3 aggregates people into groups sharing the same:
- **State** (geographic region)
- **Culture**
- **Religion**
- **Profession** (Aristocrats, Capitalists, Bureaucrats, Officers, Shopkeepers, Machinists, Laborers, Peasants, etc.)

**Key mechanics:**

| Mechanic | Description | CivLab Analog |
|----------|-------------|---------------|
| Pop growth | Birth/death rates affected by Standard of Living, healthcare laws, literacy | Population growth per cell/district |
| Migration | Pops move between states based on economic opportunity differential | Inter-cell population movement |
| Profession change | Pops shift profession based on available employment in buildings | Workforce reallocation |
| Radicalization | Pops become radical when their Standard of Living drops below expectations | Unrest / ideology shift |
| Loyalty | Pops become loyal when Standard of Living exceeds expectations | Stability bonus |
| Qualifications | Pops gain qualifications (literacy, skills) over time, enabling higher-tier employment | Technology workforce requirements |

**Standard of Living (SoL)** is the central Pop welfare metric. It is computed from:
- Wealth (income minus expenses)
- Goods consumption (which goods the Pop can afford, weighted by cultural preference)
- Political rights (laws granting or restricting franchise, education, labor rights)

**CivLab contract:** CivLab's citizen satisfaction model (CIV-0102) should derive from SoL: a weighted composite of consumption fulfillment, political freedom, and economic opportunity. The weights must be tunable per scenario.

#### 1.2 Market System

V3 uses a **closed-market equilibrium** model:

1. **Buildings** produce goods (Sell Orders) and consume goods (Buy Orders).
2. **Pops** consume goods based on wealth tier and cultural preferences (Buy Orders).
3. **Price** is set by the ratio of total Buy Orders to total Sell Orders for each good.
4. **Price range:** Base price +/- 75%. At 50% oversupply, price hits the floor. At 50% undersupply, price hits the ceiling.
5. **Market clearing:** When supply < demand, goods are rationed proportionally across all buyers.
6. **Trade routes** connect markets. Goods flow along trade routes, generating Buy/Sell orders in both markets.

**Price formula (simplified):**

```
price_ratio = buy_orders / sell_orders
if price_ratio <= 0.5:
    price = base_price * 0.25     # floor
elif price_ratio >= 2.0:
    price = base_price * 1.75     # ceiling
else:
    price = base_price * lerp(0.25, 1.75, (price_ratio - 0.5) / 1.5)
```

**Substitution:** V3 handles substitutable goods (e.g., Grain vs Fruit for food). When a preferred good is expensive, Pops substitute cheaper alternatives, reducing demand for the expensive good and increasing demand for the substitute.

**CivLab contract:** CivLab's economy (CIV-0201) should implement a similar Buy/Sell order market. Tunable parameters:
- `price_elasticity_range`: float (default 0.75, meaning +/- 75% of base price)
- `oversupply_threshold`: float (default 0.5, supply exceeds demand by this ratio to hit floor)
- `undersupply_threshold`: float (default 0.5, demand exceeds supply by this ratio to hit ceiling)
- `substitution_coefficient`: float per good-pair (how readily good A substitutes for good B)

#### 1.3 AI Decision-Making

V3's AI uses a **weighted utility system** for strategic decisions:
- Each possible action (build a factory, declare war, pass a law) is scored by a utility function.
- The utility function considers: economic impact, political feasibility, military strength, cultural alignment.
- The AI picks the highest-utility action, with some randomization for variety.
- Interest Groups (political factions) influence which actions the AI considers viable.

**CivLab contract:** CivLab's nation AI (CIV-0301) should use a weighted utility scorer with pluggable evaluation functions. The MCTS approach (RND-011) is for tactical decisions; the V3-style utility scorer is for strategic long-term planning.

---

### 2. Dwarf Fortress -- Individual Agent Simulation

**Source:** Dwarf Fortress Wiki (dwarffortresswiki.org), Tarn Adams' "Simulation Principles from Dwarf Fortress" (Game AI Pro 2, Chapter 41), Steam community discussions.

#### 2.1 Need Satisfaction System

DF's need system is the most detailed individual-agent welfare model in any game. Each dwarf has 30+ distinct needs, each with a personality-weighted priority level.

**Need categories (partial list):**

| Need | Personality Trait Driver | Satisfaction Activity |
|------|-------------------------|----------------------|
| Alcohol | Immoderation | Drink at tavern |
| Prayer | Religiosity | Pray at temple |
| Social interaction | Gregariousness | Socialize at tavern/meeting hall |
| Creativity | Creativity | Create art, craft items |
| Martial training | Martial prowess | Spar, train in barracks |
| Romance | Romance value | Seek partner |
| Nature/Animals | Nature appreciation | Visit pastures, observe animals |
| Learning | Intellectual curiosity | Read books, attend lectures |
| Acquisition | Greed | Acquire wealth objects |
| Merriment | Fun-seeking | Attend parties, performances |
| Introspection | Introspection value | Meditate |

**Need fulfillment mechanics:**

1. Each need has an internal counter ranging from 400 (Unfettered) to -100,000+ (Badly distracted).
2. When a need is satisfied, its counter resets to 400 regardless of prior value.
3. Unsatisfied needs decay over time, passing through thresholds: Unfettered (400-300) -> Level-headed (299-200) -> Untroubled (199-100) -> Not distracted (99 to -999) -> Unfocused (-1000 to -9999) -> Distracted (-10,000 to -99,999) -> Badly distracted (-100,000+).
4. Need weights are personality-driven: proposed weights are 1, 2, 5, 10 per need level (higher level = more impact on focus).

**Focus formula:**

```
numerator = sum over all needs:
    Unfettered:      6.00 * weight
    Level-headed:    5.33 * weight
    Untroubled:      4.67 * weight
    Not distracted:  4.00 * weight
    Unfocused:       3.33 * weight
    Distracted:      2.67 * weight
    Badly distracted: 2.00 * weight

denominator = 4.0 * total_need_count

focus_ratio = floor(numerator) / denominator

Focus levels:
    >= 1.40: Very focused     (+50% skill bonus)
    >= 1.20: Quite focused
    >= 1.01: Focused
    == 1.00: Untroubled       (baseline)
    >= 0.81: Unfocused
    >= 0.61: Distracted
    <  0.61: Badly distracted (-50% skill penalty)
```

**CivLab contract:** CivLab does not simulate individual dwarves, but the need-satisfaction model maps to **district-level citizen satisfaction** in CivLab. Each district has a population with aggregate need fulfillment scores. The focus formula maps to **district productivity modifier**: a well-satisfied district produces more; an unsatisfied district produces less and generates unrest.

Tunable parameters:
- `need_decay_rate`: float per tick (how fast unsatisfied needs decay)
- `need_weights`: dict mapping need -> weight (personality distribution per culture)
- `focus_to_productivity_curve`: piecewise linear mapping from focus ratio to productivity modifier
- `focus_to_unrest_threshold`: float (below this focus ratio, district generates unrest events)

#### 2.2 Stress System

DF's stress system operates on two timescales:

**Short-term stress:** Range -100,000 to 100,000. Directly modified by thoughts (happy events subtract, unhappy events add). Maps to visible mood indicators (Ecstatic to Miserable).

**Long-term stress:** Range -50,000 to 120,000. Accumulates gradually from short-term stress. Status thresholds:
- Stressed: +25,000
- Haggard: +50,000
- Harrowed: +100,000

**Rates:**
- Maximum long-term stress increase: 20,160 per year (under constant misery)
- Maximum long-term stress decrease: 43,564 per year (under constant happiness)
- Recovery is ~2x faster than accumulation, but still takes years.

**Personality modifiers:**
- **Bravery:** Controls stress accumulation rate from combat/death events.
- **Stress vulnerability:** Determines the effective threshold capacity before breakdown.
- **Anxiety propensity:** Controls natural dissipation rate.

**Breakdown cascade:** Harrowed dwarves who witness death or receive additional stress triggers enter **insanity** (permanent, removes dwarf from useful labor). This is the primary "losing is fun" cascade mechanic.

**CivLab contract:** Map to **district morale** with two timescales:
- `district_mood`: short-term, event-driven, high volatility (analogous to DF short-term stress)
- `district_stability`: long-term, slow-moving, represents accumulated civic health (analogous to DF long-term stress)
- Tunable: `mood_to_stability_transfer_rate`, `stability_recovery_rate`, `stability_collapse_threshold`
- At `stability_collapse_threshold`, district enters crisis state (analogous to DF insanity cascade): production halts, emigration spikes, revolutionary events trigger.

#### 2.3 Tarn Adams' Four Simulation Principles (Game AI Pro 2)

From Chapter 41 of Game AI Pro 2:

1. **Base simulation on reality.** Use real-world analogues as design references. When the simulation produces unrealistic results, the real-world reference tells you what is wrong. Example: V3 rain shadows on mountains producing realistic biome distribution.

2. **Embrace emergent behavior.** Do not script high-level outcomes. Define low-level rules and let macro behavior emerge. Example: DF's fortress tantrum spirals emerge from individual stress mechanics, not from a scripted "tantrum spiral" event.

3. **Make the simulation inspectable.** Every value should be visible to the player (or at least to the developer). Opaque simulations are impossible to debug and frustrating to players.

4. **Iterate on the simulation, not the content.** Build systems that generate content procedurally. Invest in simulation depth rather than hand-crafted scenarios.

**CivLab contract:** These principles should be adopted as design axioms:
- All simulation parameters must be exposed in the scenario editor and debug overlay.
- No scripted macro-events; all events emerge from agent-level or district-level rule execution.
- Prefer simulation depth (more interacting systems) over content breadth (more hand-crafted scenarios).
- Use real-world references (academic models below) for parameter calibration.

---

### 3. OpenTTD -- Transport Network Simulation

**Source:** OpenTTD Wiki (wiki.openttd.org), OpenTTD source code (GitHub, C++).

#### 3.1 Pathfinding Architecture

OpenTTD has evolved through four pathfinding systems:
1. **OPF (Old Pathfinder):** Removed due to bugs.
2. **NTP (New Train Pathfinding):** Basic A* for trains only.
3. **NPF (New Global Pathfinding):** A* for all vehicle types. Correct but slow for large maps.
4. **YAPF (Yet Another Pathfinder):** Current default. Optimized A* with caching, templated C++ for type-specific cost functions.

**YAPF design patterns:**
- **A* with infrastructure-aware cost function.** The cost of traversing a tile includes: distance, slope penalty, curve penalty, signal penalty, station penalty, depot penalty, and infrastructure maintenance cost.
- **Penalty table:** Configurable penalties per obstacle type. Example penalties (from OpenTTD settings):
  - Rail station penalty: configurable (default varies by pathfinder)
  - Slope penalty: higher cost for uphill traversal
  - Curve penalty: cost for changing direction (discourages zigzag routes)
  - Signal penalty: cost for wrong-way signals or red signals
  - Depot reverse penalty: cost for reversing in a depot

- **Segment caching:** YAPF caches the cost of previously-computed path segments to avoid recomputation. Cache invalidation on infrastructure change (track built/demolished).

**Route profitability (simplified):**
```
profit = revenue_per_unit * units_transported -
         distance_cost * distance -
         vehicle_running_cost * time -
         infrastructure_maintenance * route_tiles
```

Revenue per unit depends on: cargo type, distance transported, and time in transit (cargo loses value the longer it takes to deliver, modeled as a decay curve).

**CivLab contract:** CivLab's trade route system (CIV-0205) should use A* pathfinding with a multi-factor cost function. Tunable parameters:
- `terrain_cost_table`: dict mapping terrain type -> traversal cost
- `slope_penalty`: float (cost multiplier for elevation change)
- `infrastructure_bonus`: float (roads/rails reduce traversal cost by this factor)
- `cargo_time_decay_rate`: float (how fast cargo value decays with transit time)
- `route_maintenance_cost_per_tile`: float

---

### Part II: Academic Simulation Models

---

### 4. Epstein & Axtell (1996) -- Sugarscape: Growing Artificial Societies

**Citation:** Epstein, J.M. & Axtell, R.L. (1996). *Growing Artificial Societies: Social Science from the Bottom Up.* MIT Press. Part of the 2050 Project (Santa Fe Institute + World Resources Institute + Brookings Institution).

#### Model Description

Sugarscape is an agent-based model on a 2D grid with a single renewable resource ("sugar," later extended with "spice"):

- **Agents** are born with: vision range (how far they can see resources), metabolism (how much sugar they consume per tick), speed, and initial sugar endowment.
- **Resource landscape:** Sugar grows back at a configurable regrowth rate in fixed geographic patterns (two "sugar mountains" at opposite corners).
- **Agent rules:** Each tick, agents look in their vision range, move to the richest unoccupied cell, and consume their metabolism amount. If sugar reserves hit 0, the agent dies.
- **Emergence:** Wealth inequality emerges naturally from heterogeneous vision/metabolism. Migration waves follow resource depletion. When a second resource (spice) is introduced and agents can trade, an economic market emerges with price discovery.

**Key extensions through chapters:**
1. Basic movement + resource consumption -> wealth distribution
2. Reproduction (sexual, genetic inheritance of vision/metabolism) -> population dynamics
3. Cultural transmission (tag copying between neighbors) -> cultural clustering
4. Combat (agents can attack neighbors for resources) -> territorial behavior
5. Trade (agents exchange sugar for spice at bilateral prices) -> market emergence
6. Disease transmission (SIR-like model between neighboring agents) -> epidemic dynamics

#### CivLab Design Contracts Derived

| Sugarscape Concept | CivLab System | Contract |
|-------------------|---------------|----------|
| Heterogeneous resource landscape | Resource distribution (CIV-0103) | Resources must be geographically concentrated, not uniform. At least 2 distinct resource types with non-overlapping peaks. |
| Agent vision range | Tech-level scouting range (CIV-0301) | Higher technology increases the effective "vision" of the nation AI when evaluating expansion/settlement targets. |
| Resource regrowth rate | Renewable resource model (CIV-0104) | All renewable resources (food, timber) have a configurable regrowth rate per cell. Over-extraction should be possible (depleting faster than regrowth). |
| Wealth inequality emergence | Income distribution (CIV-0202) | The economy should produce Gini coefficient > 0 without explicit inequality rules. Inequality should emerge from agent heterogeneity and resource geography. |
| Trade price discovery | Market price model (CIV-0201) | Bilateral trade prices should emerge from supply/demand ratios, not be fixed. The V3-style market model implements this. |
| Cultural tag transmission | Ideology diffusion (CIV-0106) | Cultural/ideology values should propagate between neighboring cells via contact. See Schelling model below for the homophily coefficient. |

**Tunable parameters:**
- `resource_regrowth_rate`: float per cell type per tick
- `resource_peak_concentration`: float (how concentrated resources are at peak cells vs background)
- `agent_vision_by_tech_level`: dict mapping tech tier -> scouting range in cells

---

### 5. Schelling (1971) -- Segregation and Neighborhood Homophily

**Citation:** Schelling, T.C. (1971). "Dynamic Models of Segregation." *Journal of Mathematical Sociology*, 1(2), 143-186.

#### Model Description

Schelling's segregation model demonstrates that mild individual preferences for similar neighbors produce strong aggregate segregation:

- **Grid:** 2D grid, each cell occupied by one of two agent types (or empty).
- **Satisfaction rule:** An agent is "satisfied" if at least a fraction *t* of its 8 neighbors (Moore neighborhood) are of the same type.
- **Movement rule:** Unsatisfied agents relocate to a random empty cell.
- **Key finding:** Even with *t* as low as 0.30 (agents tolerate up to 70% different neighbors), the grid quickly segregates into large homogeneous clusters. The segregation outcome far exceeds what individual preferences would predict.

**Mathematical formulation:**

```
For agent at position (x,y) with type A:
    neighbors = cells in Moore neighborhood (8 adjacent cells)
    same_type_count = count of neighbors with type A
    total_neighbor_count = count of occupied neighbors

    similarity_ratio = same_type_count / total_neighbor_count

    satisfied = similarity_ratio >= t

    if not satisfied:
        relocate to random empty cell
```

**Parameter sensitivity:**
- *t* = 0.30: mild clustering, some mixing
- *t* = 0.50: strong segregation, clear boundaries
- *t* = 0.75: extreme segregation, nearly zero mixing
- *t* = 1.00: complete segregation (agents only happy surrounded by identical type)

**Extensions (modern research):**
- Continuous homophily preferences (not binary satisfied/unsatisfied)
- Multiple agent types (not just 2)
- Heterogeneous thresholds (different agents have different *t*)
- Network-based (not just grid neighborhoods)

#### CivLab Design Contract: Ideology Neighborhood Diffusion

CivLab's ideology system (CIV-0106) models how ideological positions spread between neighboring districts. The Schelling model directly applies:

**Contract:**
```
ideology_homophily_coefficient: float  # analogous to Schelling's 't'
    Range: [0.0, 1.0]
    Default: 0.35
    Must be tunable per scenario.

    If a district's ideology differs from > (1 - homophily_coefficient)
    fraction of its neighbors, the district experiences:
    1. Ideological pressure (drift toward neighbor majority)
    2. Internal friction (reduced stability)
    3. Migration pressure (population movement toward ideologically aligned districts)

    Higher coefficient = stronger clustering tendency = more ideological
    balkanization. Lower coefficient = more mixing = more ideological diversity.
```

**R_0 for ideology spread (from SIR model, Section 7 below):**
CivLab's `R0_civic` formula should be calibrated against Schelling dynamics. If `R0_civic > 1`, the ideology spreads; if `R0_civic < 1`, it fades. The Schelling coefficient determines the "contact rate" in the SIR analogy: higher homophily = higher effective contact rate = higher R0.

---

### 6. SIR Compartmental Model -- Epidemic / Ideology Diffusion

**Citation:** Kermack, W.O. & McKendrick, A.G. (1927). "A Contribution to the Mathematical Theory of Epidemics." *Proceedings of the Royal Society A*, 115(772), 700-721.

#### Model Description

The SIR model divides a population into three compartments:
- **S (Susceptible):** Not yet exposed.
- **I (Infected/Adopting):** Currently spreading the ideology/disease.
- **R (Recovered/Committed):** No longer actively spreading (either immune or fully committed).

**Differential equations:**

```
dS/dt = -beta * S * I / N
dI/dt = beta * S * I / N - gamma * I
dR/dt = gamma * I

where:
    N = S + I + R (total population)
    beta = transmission rate (contact rate * transmission probability per contact)
    gamma = recovery rate (1 / duration of infectious/active-spreading period)
    R0 = beta / gamma (basic reproduction number)
```

**R0 interpretation:**
- R0 > 1: epidemic grows (ideology spreads)
- R0 = 1: endemic equilibrium
- R0 < 1: epidemic dies out (ideology fades)

#### CivLab Application: R0_civic Formula

CivLab's ideology diffusion (CIV-0106) uses a SIR-inspired model where:
- **S** = population not yet exposed to an ideology
- **I** = population actively proselytizing (recently converted, enthusiastic)
- **R** = population committed but no longer actively spreading (long-term adherents)

**R0_civic formula:**

```
R0_civic = (contact_rate * conversion_probability) / fade_rate

where:
    contact_rate = f(population_density, communication_technology, schelling_homophily)
    conversion_probability = f(ideology_appeal, current_satisfaction, propaganda_investment)
    fade_rate = f(ideology_stability, counter_propaganda, time_since_conversion)
```

**Validation against epidemiology:**
- Real-world R0 values: measles ~12-18, influenza ~1.5-2.0, COVID-19 ~2.5-3.5.
- CivLab ideology R0 should range: 0.5 (fringe ideology, barely spreads) to 5.0 (revolutionary ideology in crisis conditions).
- At R0_civic > 3.0, ideology spreads explosively (revolution scenario).
- At R0_civic ~1.0, ideology reaches endemic equilibrium (stable minority).

**Tunable parameters:**
- `base_contact_rate`: float (how many neighbors a district influences per tick)
- `technology_contact_multiplier`: float per tech level (printing press, radio, internet each increase contact rate)
- `ideology_appeal_by_satisfaction`: piecewise linear curve (low satisfaction -> high appeal for revolutionary ideologies)
- `fade_rate_base`: float (how fast active spreading decays)
- `counter_propaganda_effectiveness`: float (government investment reduces R0_civic)

---

### 7. Hammond & Axelrod (2006) -- Evolution of Ethnocentrism

**Citation:** Hammond, R.A. & Axelrod, R. (2006). "The Evolution of Ethnocentrism." *Journal of Conflict Resolution*, 50(6), 926-936.

#### Model Description

An evolutionary agent-based model studying in-group favoritism:

- **Grid:** 2D toroidal grid.
- **Agents:** Each has an arbitrary "tag" (one of 4+ possible values) and two behavioral genes:
  - Cooperate with same-tag agents? (yes/no)
  - Cooperate with different-tag agents? (yes/no)
- This produces 4 strategy types:
  - **Ethnocentric:** Cooperate with same, defect with different.
  - **Humanitarian:** Cooperate with all.
  - **Selfish:** Defect with all.
  - **Traitorous:** Cooperate with different, defect with same.

- **Interaction:** Agents play one-shot Prisoner's Dilemma with all neighbors.
- **Reproduction:** Agents with positive payoff reproduce (clone with mutation) into adjacent empty cells.
- **Death:** Random death probability per tick.

**Key result:** After transient period, population distribution stabilizes at:
- Ethnocentric: ~75%
- Humanitarian: ~15%
- Selfish: ~8%
- Traitorous: ~2%

This is robust across parameter variations (doubling/halving lattice size, cycle count, tag count, cooperation cost).

**Mechanism:** Ethnocentrics form cooperating clusters that out-compete free-riders (selfish agents) in neighboring territory. Humanitarians survive as second-most-common because they also cooperate within clusters but waste cooperation on out-group defectors.

#### CivLab Design Contract: Faction Dynamics

| Hammond-Axelrod Concept | CivLab System | Contract |
|------------------------|---------------|----------|
| Tag-based cooperation | Faction alliance tendency | Nations sharing cultural/ideological tags should have higher cooperation probability. |
| Ethnocentric dominance | Default diplomatic posture | Without player intervention, AI nations should tend toward ethnocentric behavior (cooperate with similar, defect with different). |
| Humanitarian minority | Diplomatic AI variation | ~15% of AI nations should exhibit humanitarian behavior (cooperate broadly), creating natural alliance partners for diverse player strategies. |
| Selfish minority | Aggressive AI nations | ~8% of AI nations should be aggressive toward all (defect universally). |
| Cluster competition | Border dynamics | Cooperative faction clusters should expand at the expense of isolated selfish nations. |
| Tag mutation | Cultural drift | Over time, cultural tags should mutate, creating new faction alignments. |

**Tunable parameters:**
- `ethnocentric_tendency_weight`: float (how strongly cultural similarity affects cooperation decisions)
- `faction_strategy_distribution`: dict mapping strategy -> initial probability (default: {ethnocentric: 0.75, humanitarian: 0.15, selfish: 0.08, traitorous: 0.02})
- `cultural_mutation_rate`: float per tick (probability of tag change)
- `cooperation_cost`: float (cost paid by cooperator, benefit received by partner -- Prisoner's Dilemma payoff matrix)
- `cooperation_benefit_ratio`: float (ratio of partner's benefit to cooperator's cost; default ~3.0 following Axelrod)

---

### Part III: Cross-Reference and Synthesis

---

### 8. Annotated Bibliography

| # | Citation | Year | Key Contribution | CivLab System |
|---|----------|------|-----------------|---------------|
| 1 | Epstein & Axtell, *Growing Artificial Societies* | 1996 | Sugarscape: resource heterogeneity, trade emergence, wealth inequality | CIV-0103 (resources), CIV-0201 (market), CIV-0202 (inequality) |
| 2 | Schelling, "Dynamic Models of Segregation" | 1971 | Neighborhood homophily -> macro segregation | CIV-0106 (ideology diffusion) |
| 3 | Kermack & McKendrick, "Mathematical Theory of Epidemics" | 1927 | SIR compartmental model, R0 | CIV-0106 (R0_civic formula) |
| 4 | Hammond & Axelrod, "Evolution of Ethnocentrism" | 2006 | Tag-based cooperation, faction dynamics emergence | CIV-0301 (nation AI), CIV-0302 (diplomacy) |
| 5 | Adams, "Simulation Principles from Dwarf Fortress" | 2015 | 4 design principles: reality-based, emergent, inspectable, system-over-content | All CivLab systems (design axioms) |
| 6 | Paradox Interactive, Victoria 3 Dev Diaries | 2021-25 | Pop system, market system, utility-based AI | CIV-0102 (satisfaction), CIV-0201 (market), CIV-0301 (AI) |
| 7 | Bay 12 Games, Dwarf Fortress Wiki | 2006-26 | Need satisfaction, stress system, focus formula | CIV-0102 (district satisfaction), CIV-0105 (morale) |
| 8 | OpenTTD Project, Source Code and Wiki | 2004-26 | YAPF pathfinding, route profitability, infrastructure cost | CIV-0205 (trade routes) |
| 9 | Andersson, "Deep Dive: Modeling the global economy in Victoria 3" | 2022 | Closed-market equilibrium, buy/sell orders, substitution | CIV-0201 (market) |
| 10 | Hatna & Benenson, "The Schelling Model of Ethnic Residential Dynamics" | 2012 | Extended Schelling model beyond binary segregation patterns | CIV-0106 (multi-ideology diffusion) |

---

### 9. CivLab Design Contracts Summary

#### Contract 1: Resource Geography (Sugarscape-derived)

```yaml
contract_id: SIM-C001
source: Epstein & Axtell 1996 (Sugarscape)
civlab_system: CIV-0103
requirement: >
  Resource distribution must be geographically concentrated, not uniform.
  At least 2 distinct resource types with non-overlapping peak regions.
  Over-extraction (consumption > regrowth) must be possible.
parameters:
  resource_regrowth_rate:
    type: float
    range: [0.001, 1.0]
    per: cell_type
    description: "Fraction of max capacity regrown per tick"
  resource_peak_concentration:
    type: float
    range: [1.0, 20.0]
    description: "Ratio of peak cell yield to background cell yield"
```

#### Contract 2: Market Price Discovery (V3-derived)

```yaml
contract_id: SIM-C002
source: Victoria 3 market system
civlab_system: CIV-0201
requirement: >
  Goods prices emerge from aggregate buy/sell orders.
  Price range: base_price * [1 - price_elasticity, 1 + price_elasticity].
  Market clearing rations goods proportionally when supply < demand.
  Substitution reduces demand for expensive goods.
parameters:
  price_elasticity_range:
    type: float
    default: 0.75
  oversupply_floor_ratio:
    type: float
    default: 0.5
  substitution_coefficient:
    type: float
    per: good_pair
    range: [0.0, 1.0]
```

#### Contract 3: Ideology Neighborhood Diffusion (Schelling + SIR)

```yaml
contract_id: SIM-C003
source: Schelling 1971 + Kermack-McKendrick 1927
civlab_system: CIV-0106
requirement: >
  Ideology spreads between neighboring districts via SIR dynamics.
  R0_civic determines whether ideology grows or fades.
  Schelling homophily coefficient controls contact rate.
  Must be tunable per scenario.
parameters:
  ideology_homophily_coefficient:
    type: float
    default: 0.35
    range: [0.0, 1.0]
    description: "Schelling 't' -- higher = stronger clustering"
  r0_civic_range:
    type: [float, float]
    default: [0.5, 5.0]
    description: "Achievable R0 range for ideology spread"
  base_contact_rate:
    type: float
    default: 2.0
  technology_contact_multiplier:
    type: dict
    description: "tech_tier -> multiplier (e.g., printing_press: 1.5, radio: 2.0, internet: 3.0)"
```

#### Contract 4: Faction Cooperation Dynamics (Hammond-Axelrod-derived)

```yaml
contract_id: SIM-C004
source: Hammond & Axelrod 2006
civlab_system: CIV-0301, CIV-0302
requirement: >
  AI nation diplomacy uses tag-based cooperation.
  Without player intervention, ethnocentric behavior should dominate (~75%).
  Cultural similarity increases cooperation probability.
  Cultural tags mutate over time, creating new alignments.
parameters:
  ethnocentric_tendency_weight:
    type: float
    default: 0.75
    range: [0.0, 1.0]
  faction_strategy_initial_distribution:
    type: dict
    default:
      ethnocentric: 0.75
      humanitarian: 0.15
      selfish: 0.08
      traitorous: 0.02
  cultural_mutation_rate:
    type: float
    default: 0.01
    description: "Probability of cultural tag change per tick per nation"
  cooperation_benefit_ratio:
    type: float
    default: 3.0
    description: "Prisoner's Dilemma: benefit/cost ratio"
```

#### Contract 5: District Satisfaction and Morale (DF-derived)

```yaml
contract_id: SIM-C005
source: Dwarf Fortress need/stress system
civlab_system: CIV-0102, CIV-0105
requirement: >
  District satisfaction is a composite of weighted need fulfillment scores.
  Two timescales: short-term mood (volatile) and long-term stability (slow).
  Focus/productivity modifier derived from satisfaction.
  Stability collapse triggers crisis cascade.
parameters:
  need_decay_rate:
    type: float
    default: 0.01
    description: "Per-tick decay for unsatisfied needs"
  focus_to_productivity_curve:
    type: piecewise_linear
    default: [[0.6, -0.5], [0.8, -0.1], [1.0, 0.0], [1.2, 0.1], [1.4, 0.5]]
    description: "[focus_ratio, productivity_modifier] pairs"
  stability_recovery_rate:
    type: float
    default: 0.005
    description: "Max long-term stability recovery per tick under good mood"
  stability_collapse_threshold:
    type: float
    default: -0.5
    description: "Normalized stability below which crisis cascade triggers"
```

#### Contract 6: Trade Route Pathfinding (OpenTTD-derived)

```yaml
contract_id: SIM-C006
source: OpenTTD YAPF pathfinding
civlab_system: CIV-0205
requirement: >
  Trade routes use A* pathfinding with infrastructure-aware cost function.
  Cost includes: distance, terrain penalty, slope penalty, infrastructure bonus.
  Route profitability considers cargo value decay over transit time.
parameters:
  terrain_cost_table:
    type: dict
    description: "terrain_type -> base traversal cost"
    default:
      plains: 1.0
      forest: 1.5
      hills: 2.0
      mountains: 4.0
      water: 3.0
      desert: 2.5
  slope_penalty:
    type: float
    default: 1.5
    description: "Multiplier for elevation change per cell"
  infrastructure_bonus:
    type: dict
    default:
      road: 0.5
      railroad: 0.25
      highway: 0.3
    description: "Multiplier applied to base terrain cost when infrastructure exists"
  cargo_time_decay_rate:
    type: float
    default: 0.02
    description: "Fraction of cargo value lost per tick of transit time"
```

---

## Decision

1. **Adopt V3-style market model** for CivLab economy (buy/sell orders, price elasticity, substitution).
2. **Adopt DF-inspired dual-timescale satisfaction model** for district welfare (mood + stability).
3. **Implement Schelling homophily coefficient** as the core parameter for ideology neighborhood diffusion.
4. **Use SIR-derived R0_civic** for ideology spread dynamics, calibrated to range [0.5, 5.0].
5. **Apply Hammond-Axelrod faction strategy distribution** as default AI diplomatic posture distribution.
6. **Use YAPF-inspired A* with infrastructure costs** for trade route pathfinding.
7. **All parameters must be tunable per scenario** and exposed in the scenario editor.
8. **Adams' four principles** (reality-based, emergent, inspectable, system-over-content) adopted as design axioms.

---

## Open Questions Remaining

1. **Pop granularity:** V3 uses culture/profession/state aggregation. CivLab currently uses district-level aggregation. Should CivLab model individual profession groups within districts (finer granularity, closer to V3), or keep district-level aggregation (simpler, sufficient for MVP)? Recommend district-level for MVP, profession-groups as post-MVP enhancement.

2. **Multi-ideology Schelling:** The classic Schelling model uses 2 types. CivLab has 4+ ideologies. Hatna & Benenson (2012) extended Schelling to multi-type settings and found that 3+ types produce more complex boundary patterns. The homophily coefficient may need to be a matrix (per ideology-pair), not a scalar.

3. **Market simulation tick rate:** V3 runs its market simulation daily (in-game). CivLab's tick rate is not yet finalized. The market model's stability depends on tick rate -- too infrequent causes oscillations, too frequent is computationally expensive. Recommend matching the main simulation tick rate and testing for oscillation.

4. **Sugarscape wealth distribution calibration:** The Gini coefficient that emerges from CivLab's economy should be compared against real-world reference values (e.g., USA ~0.40, Nordic countries ~0.27, pre-industrial societies ~0.45). If CivLab produces unrealistic inequality levels, adjust `resource_peak_concentration` and `substitution_coefficient`.

5. **OpenTTD pathfinding scale:** OpenTTD's YAPF works on grids of ~1000x1000 tiles. CivLab's hex grid may be larger. If A* performance is insufficient, consider hierarchical A* (HPA*) or JPS+ for the hex grid. This is an implementation concern, not a design concern.

---

## Sources

### Game References

- Victoria 3 Dev Diary #1 (Pops): https://forum.paradoxplaza.com/forum/developer-diary/victoria-3-dev-diary-1-pops.1476573/
- Victoria 3 Market Wiki: https://vic3.paradoxwikis.com/Market
- Deep Dive: Modeling the global economy in Victoria 3: https://www.gamedeveloper.com/design/deep-dive-modeling-the-global-economy-in-victoria-3
- Victoria 3 Dev Diary #37 (Market Expansion): https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-37-market-expansion
- Dwarf Fortress Wiki - Need system: https://dwarffortresswiki.org/index.php/DF2014:Need
- Dwarf Fortress Wiki - Stress system: https://dwarffortresswiki.org/Stress
- Dwarf Fortress Wiki - Keeping dwarves unstressed: https://dwarffortresswiki.org/index.php/DF2014:Keeping_your_dwarves_unstressed
- Tarn Adams, "Simulation Principles from Dwarf Fortress," Game AI Pro 2, Ch. 41 (2015): http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter41_Simulation_Principles_from_Dwarf_Fortress.pdf
- OpenTTD Pathfinding documentation: https://wiki.openttd.org/en/Archive/Source/OpenTTDDevBlackBook/Simulation/Pathfinding
- OpenTTD YAPF documentation: https://wiki.openttd.org/en/Archive/Manual/Yet%20Another%20Pathfinder

### Academic References

- Epstein, J.M. & Axtell, R.L. (1996). *Growing Artificial Societies: Social Science from the Bottom Up.* MIT Press: https://mitpress.mit.edu/9780262550253/growing-artificial-societies/
- Schelling, T.C. (1971). "Dynamic Models of Segregation." *Journal of Mathematical Sociology*, 1(2): http://nifty.stanford.edu/2014/mccown-schelling-model-segregation/
- Kermack, W.O. & McKendrick, A.G. (1927). "A Contribution to the Mathematical Theory of Epidemics." *Proc. Royal Society A*, 115(772): https://en.wikipedia.org/wiki/Compartmental_models_in_epidemiology
- Hammond, R.A. & Axelrod, R. (2006). "The Evolution of Ethnocentrism." *Journal of Conflict Resolution*, 50(6): https://journals.sagepub.com/doi/10.1177/0022002706293470
- Hatna, E. & Benenson, I. (2012). "The Schelling Model of Ethnic Residential Dynamics." *JASSS*, 15(1): https://jasss.soc.surrey.ac.uk/15/1/6.html
- Mesa Schelling Model implementation: https://mesa.readthedocs.io/latest/examples/basic/schelling.html
- Evolution of ethnocentrism model (CoMSES): https://www.comses.net/codebases/2942/releases/1.1.0/


---

## Source: research/RND-016-svg-pipeline-validation.md

# RND-016: SVG Procedural Generation Pipeline — resvg Validation and DOM Edit Tooling

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-beta

---

## Executive Summary

**resvg** (pure Rust, no system dependencies) is the recommended SVG rendering library for CivLab's procedural icon/UI generation pipeline. It supports gradients, patterns, filters (including feGaussianBlur, feColorMatrix), text with embedded fonts, and clipping/masking — sufficient for game UI icons and procedural badge/emblem generation. For SVG parsing, use **roxmltree** (read-only, fastest) for analysis/inspection and **xmltree** or direct string templating for SVG mutation (attribute changes, element insertion). The alternative **librsvg** offers marginally better SVG spec coverage but introduces a C dependency (cairo, glib) that complicates cross-platform builds. Decision: **resvg + roxmltree (read) + string templating (write) for the SVG mutation pipeline**.

---

## Research Findings

### 1. resvg — Pure Rust SVG Renderer

#### Overview

resvg is an SVG rendering library written entirely in Rust. It uses tiny-skia for software rasterization and usvg for SVG parsing/simplification. The library aims to support the static SVG subset (no animations, scripting, or interactive elements).

- **Repository:** https://github.com/linebender/resvg
- **Current version:** 0.45.x (as of Feb 2026)
- **License:** MPL-2.0
- **Rendering backend:** tiny-skia (pure Rust software rasterizer)
- **SVG parser:** usvg (converts SVG to simplified render tree)

#### Architecture

resvg separates SVG processing into two distinct phases:

```
SVG File
   │
   ▼
┌──────────┐
│   usvg   │  Parse SVG → Simplified render tree
│          │  - Resolves styles/attributes
│          │  - Converts shapes to paths
│          │  - Removes invisible elements
│          │  - Resolves <use> references
│          │  - Handles CSS cascading
└────┬─────┘
     │
     ▼
┌──────────┐
│  resvg   │  Render tree → Pixel buffer
│          │  - Rasterization via tiny-skia
│          │  - Filter effects
│          │  - Compositing
│          │  - Anti-aliasing
└──────────┘
     │
     ▼
  PNG/RGBA buffer
```

This separation means:
1. usvg can be used standalone for SVG analysis/transformation without rendering
2. resvg receives a clean, simplified tree — no ambiguity in rendering
3. Cross-platform reproducibility: same SVG produces identical pixels on x86 Windows, ARM macOS, and Linux

#### SVG Feature Support Matrix

| SVG Feature | Supported | Notes |
|-------------|-----------|-------|
| **Basic shapes** (rect, circle, ellipse, line, polyline, polygon) | YES | Converted to paths by usvg |
| **Paths** (d attribute, all commands) | YES | Full path data support |
| **Gradients** (linearGradient, radialGradient) | YES | Including spreadMethod, gradientTransform |
| **Patterns** | YES | Pattern fills and strokes |
| **Clipping** (clipPath) | YES | |
| **Masking** (mask) | YES | Luminance and alpha masks |
| **Opacity** | YES | Element and group opacity |
| **Transforms** | YES | All transform types |
| **Text** | YES | With embedded font support (not system fonts) |
| **Filters** | YES | See filter details below |
| **Markers** | YES | |
| **Symbols** | YES | Resolved by usvg |
| **use** (local refs) | YES | Resolved by usvg |
| **use** (external refs) | NO | External SVG file references not supported |
| **Images** (embedded) | YES | Base64 PNG/JPEG in SVG |
| **Images** (external) | YES | File path references |
| **CSS** (inline, style element) | YES | Cascade resolution in usvg |
| **CSS** (external stylesheets) | NO | Not supported |
| **Color fonts** (Emoji) | YES | COLRv0, COLRv1 (mostly), sbix, CBDT, SVG tables |
| **Viewport/viewBox** | YES | |
| **preserveAspectRatio** | YES | |

#### Filter Support Details

| Filter Primitive | Supported | Notes |
|------------------|-----------|-------|
| **feGaussianBlur** | YES | Single-threaded IIR blur (slower than librsvg's box blur) |
| **feColorMatrix** | YES | All matrix types |
| **feComponentTransfer** | YES | |
| **feComposite** | YES | All operators |
| **feMerge** | YES | |
| **feOffset** | YES | |
| **feFlood** | YES | |
| **feBlend** | YES | All blend modes |
| **feMorphology** | YES | Erode and dilate |
| **feDisplacementMap** | YES | |
| **feTurbulence** | YES | Perlin and fractal noise |
| **feDiffuseLighting** | YES | |
| **feSpecularLighting** | YES | |
| **feImage** | YES | |
| **feTile** | YES | |
| **feConvolveMatrix** | YES | |
| **feDropShadow** | YES | SVG 2 feature |

This filter coverage is sufficient for game UI effects like:
- Drop shadows on icons (feDropShadow or feGaussianBlur + feOffset)
- Color tinting for faction-specific icons (feColorMatrix)
- Glow effects (feGaussianBlur + feComposite)
- Emboss/bevel on badges (feDiffuseLighting + feSpecularLighting)
- Noise textures for procedural backgrounds (feTurbulence)

#### Unsupported Features

The following are explicitly **not supported** in resvg:

**Elements:**
- `altGlyph`, `altGlyphDef`, `altGlyphItem` (deprecated font elements)
- `font`, `font-face`, `glyph`, `missing-glyph` (SVG fonts — use TrueType/OpenType instead)
- `color-profile` (deprecated)
- `use` with external SVG file references

**Attributes:**
- `clip` (deprecated in SVG 2)
- `color-interpolation`, `color-profile`, `color-rendering`
- `direction`, `unicode-bidi` (complex text layout)
- `font-size-adjust`, `font-stretch`
- `glyph-orientation-horizontal/vertical` (removed/deprecated in SVG 2)
- `kerning` (removed in SVG 2)

**Interactive/Dynamic:**
- Animations (SMIL `<animate>`, `<animateTransform>`, etc.)
- Scripting (`<script>`)
- Events (onclick, onload, etc.)
- Cursor (`<cursor>`)
- Links (`<a>`)

None of these unsupported features are relevant to CivLab's static icon/UI generation use case.

#### Performance

**General characteristics:**
- Pure Rust, single-threaded software rasterization (tiny-skia)
- No system library dependencies — fully self-contained
- Cross-platform identical output (bit-for-bit reproducibility)
- ~1600 regression tests in the test suite

**Benchmark data:**

From resvg-js (NAPI bindings to resvg):
- ~39.6 ops/s for SVG-to-PNG conversion (general SVG files)
- 3.6x faster than sharp for the same task

For the paris-30k.svg benchmark (30,000 layers):
- Before optimization: ~33,760ms
- After layer bounding box optimization: ~290ms (115x faster)

**Estimated performance for CivLab 64x64 icons on M3:**

resvg's bottleneck is filter-heavy SVGs (especially Gaussian blur). For simple icons at 64x64:
- Without filters: estimated **500-2000 renders/sec** (based on tiny-skia throughput for small rasters)
- With feGaussianBlur: estimated **100-500 renders/sec** (IIR blur is the bottleneck)
- With complex filter chains: estimated **50-200 renders/sec**

These are estimates extrapolated from available benchmark data. The 39.6 ops/s figure from resvg-js is for much larger, more complex SVGs. Small 64x64 icons will be orders of magnitude faster.

**Recommendation:** For CivLab's needs (generating hundreds of icons at build time, not real-time), even the conservative estimate of 50 renders/sec means a full set of 500 icons renders in 10 seconds. Performance is not a concern.

#### API Usage (Rust)

```rust
use resvg::usvg::{self, fontdb, TreeParsing, TreeTextToPath};
use resvg::tiny_skia;

fn render_svg_to_png(svg_data: &[u8], width: u32, height: u32) -> Vec<u8> {
    // Set up font database
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    // Parse SVG
    let opt = usvg::Options::default();
    let mut tree = usvg::Tree::from_data(svg_data, &opt).unwrap();
    tree.convert_text(&fontdb);

    // Create pixel buffer
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();

    // Render
    let tree = resvg::Tree::from_usvg(&tree);
    tree.render(tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Encode to PNG
    pixmap.encode_png().unwrap()
}
```

#### API Usage (Node.js via resvg-js)

```typescript
import { Resvg } from '@resvg/resvg-js';

function renderSvgToPng(svgString: string, width: number): Buffer {
    const resvg = new Resvg(svgString, {
        fitTo: { mode: 'width', value: width },
        font: {
            loadSystemFonts: false,
            fontFiles: ['./assets/fonts/game-font.ttf'],
        },
    });

    const pngData = resvg.render();
    return pngData.asPng();
}
```

---

### 2. SVG Parsing: roxmltree

#### Overview

roxmltree is a high-performance, read-only XML/SVG parser for Rust. It parses XML into an immutable tree structure optimized for fast traversal.

- **Repository:** https://github.com/RazrFalcon/roxmltree
- **License:** MIT/Apache-2.0
- **Key characteristic:** **Read-only** — no mutation of the parsed tree

#### Performance

roxmltree is the fastest XML parser in the Rust ecosystem:
- Backed by xmlparser (many times faster than xml-rs)
- Read-only design enables arena allocation and zero-copy string references
- Parent node access supported (unlike some streaming parsers)

#### Usage for SVG Analysis

```rust
use roxmltree::Document;

fn analyze_svg(svg_data: &str) {
    let doc = Document::parse(svg_data).unwrap();

    // Find all rect elements
    for node in doc.descendants() {
        if node.has_tag_name("rect") {
            let x = node.attribute("x").unwrap_or("0");
            let y = node.attribute("y").unwrap_or("0");
            let width = node.attribute("width").unwrap_or("0");
            let height = node.attribute("height").unwrap_or("0");
            let fill = node.attribute("fill").unwrap_or("none");
            println!("rect at ({x},{y}) size {width}x{height} fill={fill}");
        }
    }

    // Find elements by ID
    if let Some(icon) = doc.descendants().find(|n| n.attribute("id") == Some("icon-base")) {
        println!("Found icon-base element: {:?}", icon.tag_name().name());
    }
}
```

#### Limitations

- **Cannot modify the tree**: No `set_attribute()`, `append_child()`, `remove_child()`
- **Cannot serialize back to XML**: Read-only parsing, no writer
- For mutation, need a different approach (see section 3)

---

### 3. SVG Mutation Strategies

Since roxmltree is read-only, CivLab needs a separate approach for generating and modifying SVG documents. Three options were evaluated:

#### Option A: minidom (Rust XML library with mutation)

**Overview:** minidom is a mutable XML DOM library based on quick-xml.

```rust
use minidom::Element;

fn mutate_svg() {
    let svg: Element = "<svg xmlns='http://www.w3.org/2000/svg' width='64' height='64'>
        <rect id='bg' x='0' y='0' width='64' height='64' fill='#333'/>
    </svg>".parse().unwrap();

    // minidom supports:
    // - Element creation
    // - Attribute setting
    // - Child insertion
    // - Element removal
    // - Serialization back to XML string
}
```

**Pros:**
- Full DOM mutation (setAttribute, appendChild, removeChild)
- Based on quick-xml (faster than xml-rs)
- Serializes back to XML string

**Cons:**
- Designed primarily for XMPP, not SVG
- SVG namespace handling can be awkward
- No SVG-specific validation
- Less performant than roxmltree for read operations

#### Option B: xml-doc (Rust mutable XML tree)

**Overview:** xml-doc provides a mutable XML tree with a cleaner API than minidom.

- **Repository:** https://github.com/BlueGreenMagick/xml-doc
- Supports `set_attribute()`, element insertion, and serialization
- Slower than roxmltree for parsing but provides full mutation

#### Option C: String Templating (Recommended)

For CivLab's use case (procedural SVG generation), the SVG documents are **generated from scratch**, not parsed-and-mutated from existing files. String templating is the simplest and most performant approach:

```rust
fn generate_faction_icon(
    faction_color: &str,
    emblem_path: &str,
    border_style: &str,
    size: u32,
) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{size}" height="{size}" viewBox="0 0 64 64">
  <defs>
    <radialGradient id="bg-grad">
      <stop offset="0%" stop-color="{faction_color}" stop-opacity="0.8"/>
      <stop offset="100%" stop-color="{faction_color}" stop-opacity="0.3"/>
    </radialGradient>
    <filter id="shadow">
      <feDropShadow dx="1" dy="1" stdDeviation="1" flood-opacity="0.5"/>
    </filter>
  </defs>

  <!-- Background -->
  <circle cx="32" cy="32" r="30" fill="url(#bg-grad)"
          stroke="{faction_color}" stroke-width="2"/>

  <!-- Emblem -->
  <path d="{emblem_path}" fill="white" filter="url(#shadow)"
        transform="translate(16,16) scale(0.5)"/>

  <!-- Border decoration -->
  <circle cx="32" cy="32" r="31" fill="none"
          stroke="{border_style}" stroke-width="1" stroke-dasharray="4,2"/>
</svg>"#,
        size = size,
        faction_color = faction_color,
        emblem_path = emblem_path,
        border_style = border_style,
    )
}
```

**Pros:**
- Zero parsing overhead — SVG is generated directly as a string
- No external dependencies beyond std::fmt
- Full control over SVG structure
- Easy to parameterize any attribute or element
- Composable: build SVG fragments as functions, combine them

**Cons:**
- No structural validation (malformed SVG possible if template is wrong)
- Harder to conditionally modify existing SVGs (but CivLab generates fresh SVGs)
- String escaping needed for user-provided content (but CivLab uses known-safe values)

#### Recommendation

**String templating** for SVG generation (primary path) + **roxmltree** for any SVG analysis/inspection needed during build validation.

Rationale: CivLab generates procedural SVGs from templates with parameterized values (faction colors, emblem paths, border styles). This is fundamentally a generation problem, not a mutation problem. String templating is the simplest solution with zero overhead.

---

### 4. librsvg — Alternative Renderer

#### Overview

librsvg is the GNOME project's SVG rendering library. Originally C, it has been progressively rewritten in Rust but still depends on cairo, glib, and other C libraries.

#### Comparison with resvg

| Factor | resvg | librsvg |
|--------|-------|---------|
| Language | Pure Rust | Rust + C (cairo, glib, pango) |
| System dependencies | None | cairo, glib, pango, libxml2 |
| SVG spec coverage | Good (static subset) | Better (more edge cases) |
| Text rendering | Embedded fonts only | System fonts via pango |
| Filter performance | Single-threaded IIR blur | Box blur + multithreading |
| Cross-platform builds | Trivial (cargo build) | Complex (C toolchain + deps) |
| Cross-platform output | Bit-identical | Platform-dependent (different cairo/pango) |
| Package size | ~2MB binary | ~15MB+ with deps |
| Rust API | Native | FFI bindings |
| Maintenance | Active (linebender) | Active (GNOME) |

#### Key Differences

1. **Text rendering**: librsvg uses pango for text layout, supporting system fonts and complex text shaping (Arabic, CJK). resvg uses its own text engine with embedded fonts only. For game UI with a custom font, resvg is sufficient.

2. **Filter performance**: librsvg's Gaussian blur uses box blur approximation with multithreading, making it significantly faster for blur-heavy SVGs. resvg uses single-threaded IIR blur. For small 64x64 icons, this difference is negligible.

3. **Build complexity**: librsvg requires cairo, glib, pango, and libxml2 as build dependencies. On macOS, this means either Homebrew or a complex cross-compilation setup. On Linux, these are common but add container image size. resvg has zero non-Rust dependencies.

4. **Output reproducibility**: resvg produces bit-identical output across platforms. librsvg's output depends on the system's cairo and pango versions, which may differ between macOS and Linux.

#### Verdict

**resvg is preferred** for CivLab because:
- Pure Rust build: no C toolchain needed, trivial cross-compilation
- Bit-identical output: CI and local dev produce same results
- Feature coverage is sufficient for game UI icons
- Performance is adequate for build-time generation
- No system font dependency: game ships its own fonts

librsvg would be preferred only if:
- Complex text layout (bidirectional, CJK) were needed
- Heavy Gaussian blur performance were critical in hot paths
- Broader SVG spec coverage were required for user-supplied SVGs

None of these apply to CivLab's procedural icon pipeline.

---

### 5. SVG Mutation Contract

The following contract defines the interface for CivLab's procedural SVG generation system:

```rust
/// SVG template engine for procedural game UI generation.
/// Generates SVG strings from parameterized templates,
/// then renders them to PNG via resvg.
trait SvgPipeline {
    /// Generate an SVG string from a template and parameters.
    fn generate_svg(&self, template: &SvgTemplate, params: &SvgParams) -> String;

    /// Render an SVG string to a PNG buffer at the specified dimensions.
    fn render_to_png(&self, svg_data: &str, width: u32, height: u32) -> Vec<u8>;

    /// Batch-render multiple SVGs to PNGs.
    fn batch_render(
        &self,
        items: &[(String, u32, u32)],  // (svg_data, width, height)
    ) -> Vec<Vec<u8>>;

    /// Validate an SVG string (parse with usvg, check for errors).
    fn validate_svg(&self, svg_data: &str) -> Result<SvgValidation, SvgError>;
}

/// Template definition for procedural SVG generation.
struct SvgTemplate {
    /// Template name (e.g., "faction_icon", "resource_badge", "unit_health_bar").
    name: String,

    /// SVG template string with {{placeholder}} markers.
    template: String,

    /// Required parameters for this template.
    required_params: Vec<String>,

    /// Default values for optional parameters.
    defaults: HashMap<String, String>,
}

/// Parameters to fill an SVG template.
struct SvgParams {
    /// Key-value pairs replacing {{placeholder}} markers.
    values: HashMap<String, String>,
}

/// Validation result from SVG parsing.
struct SvgValidation {
    /// Whether the SVG is valid and renderable.
    valid: bool,

    /// Parsed dimensions.
    width: f64,
    height: f64,

    /// Number of elements in the simplified tree.
    element_count: usize,

    /// Whether filters are used (affects performance).
    uses_filters: bool,

    /// List of referenced fonts.
    fonts_used: Vec<String>,

    /// Warnings (non-fatal issues).
    warnings: Vec<String>,
}

/// SVG generation error.
enum SvgError {
    /// Template parameter missing.
    MissingParam(String),

    /// SVG parsing failed (malformed XML or unsupported features).
    ParseError(String),

    /// Rendering failed.
    RenderError(String),
}
```

### Template Examples

#### Faction Icon Template

```rust
const FACTION_ICON_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{size}}" height="{{size}}" viewBox="0 0 64 64">
  <defs>
    <radialGradient id="bg">
      <stop offset="0%" stop-color="{{primary_color}}" stop-opacity="0.9"/>
      <stop offset="100%" stop-color="{{secondary_color}}" stop-opacity="0.4"/>
    </radialGradient>
    <filter id="emblem-shadow">
      <feDropShadow dx="0.5" dy="0.5" stdDeviation="0.8" flood-color="#000" flood-opacity="0.4"/>
    </filter>
    <clipPath id="circle-clip">
      <circle cx="32" cy="32" r="29"/>
    </clipPath>
  </defs>

  <!-- Background circle with gradient -->
  <circle cx="32" cy="32" r="30" fill="url(#bg)" stroke="{{border_color}}" stroke-width="2"/>

  <!-- Emblem path (clipped to circle) -->
  <g clip-path="url(#circle-clip)" filter="url(#emblem-shadow)">
    <path d="{{emblem_path}}" fill="{{emblem_color}}"
          transform="translate({{emblem_x}},{{emblem_y}}) scale({{emblem_scale}})"/>
  </g>

  <!-- Decorative border ring -->
  <circle cx="32" cy="32" r="31" fill="none"
          stroke="{{border_color}}" stroke-width="0.5" opacity="0.6"/>
</svg>"#;
```

#### Resource Badge Template

```rust
const RESOURCE_BADGE_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{size}}" height="{{size}}" viewBox="0 0 32 32">
  <defs>
    <linearGradient id="badge-bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="{{top_color}}"/>
      <stop offset="100%" stop-color="{{bottom_color}}"/>
    </linearGradient>
  </defs>

  <!-- Rounded rectangle background -->
  <rect x="1" y="1" width="30" height="30" rx="4" ry="4"
        fill="url(#badge-bg)" stroke="{{border_color}}" stroke-width="1"/>

  <!-- Resource icon -->
  <path d="{{icon_path}}" fill="{{icon_color}}"
        transform="translate(4,4) scale(0.75)"/>

  <!-- Quantity text -->
  <text x="28" y="28" font-family="{{font_family}}" font-size="10"
        fill="white" text-anchor="end" font-weight="bold">{{quantity}}</text>
</svg>"#;
```

#### Unit Health Bar Template

```rust
const HEALTH_BAR_TEMPLATE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"
     width="{{bar_width}}" height="{{bar_height}}" viewBox="0 0 48 6">
  <!-- Background -->
  <rect x="0" y="0" width="48" height="6" rx="3" fill="#333" opacity="0.8"/>

  <!-- Health fill (width proportional to health %) -->
  <rect x="1" y="1" width="{{fill_width}}" height="4" rx="2" fill="{{health_color}}"/>

  <!-- Segment lines -->
  <line x1="12" y1="0" x2="12" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
  <line x1="24" y1="0" x2="24" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
  <line x1="36" y1="0" x2="36" y2="6" stroke="#000" stroke-width="0.5" opacity="0.3"/>
</svg>"#;
```

---

## Decision

**resvg (pure Rust) + roxmltree (read-only analysis) + string templating (SVG generation)**

This combination provides:
1. **Zero system dependencies**: Pure Rust stack, trivial to build on all platforms
2. **Bit-identical output**: Same SVG produces same PNG on macOS, Linux, and CI
3. **Sufficient feature coverage**: Gradients, filters, text, clipping — everything needed for game UI
4. **High performance**: Estimated 500+ icons/sec for simple 64x64 renders on M3
5. **Simple mutation model**: String templating is the right abstraction for procedural generation
6. **Build-time pipeline**: Icons are generated at build/deploy time, not runtime

---

## Implementation Contract

### Rust Crate Dependencies

```toml
[dependencies]
resvg = "0.45"      # SVG rendering
usvg = "0.45"       # SVG parsing/simplification (re-exported by resvg)
tiny-skia = "0.11"  # Pixel buffer (re-exported by resvg)
roxmltree = "0.20"  # Read-only SVG analysis (for validation)
```

### Build Pipeline Integration

```
Source Templates (Rust string constants)
         │
         ▼
┌──────────────────────┐
│  Template Engine      │
│  - Parameter injection│
│  - Faction colors     │
│  - Emblem paths       │
│  - Resource icons     │
└────────┬─────────────┘
         │
         ▼ SVG strings
┌──────────────────────┐
│  Validation (usvg)   │
│  - Parse check        │
│  - Font resolution    │
│  - Element count      │
└────────┬─────────────┘
         │
         ▼ validated SVG
┌──────────────────────┐
│  Rendering (resvg)   │
│  - SVG → PNG @64x64   │
│  - Batch processing   │
│  - Deterministic      │
└────────┬─────────────┘
         │
         ▼ PNG buffers
┌──────────────────────┐
│  Atlas Packing       │
│  - Combine into sheet │
│  - Generate metadata  │
│  - Output atlas.png   │
│  - Output atlas.json  │
└──────────────────────┘
```

### Performance Budget

| Operation | Target | Estimated |
|-----------|--------|-----------|
| Simple icon render (64x64, no filters) | < 2ms | ~0.5-2ms |
| Icon with drop shadow (64x64) | < 10ms | ~2-10ms |
| Icon with complex filters (64x64) | < 20ms | ~5-20ms |
| Full icon set (500 icons) | < 30s | ~5-15s |
| Validation pass (SVG parse only) | < 0.5ms | ~0.1-0.5ms |

These targets are for build-time generation on M3 hardware. Performance is not a runtime concern.

---

## Open Questions Remaining

1. **Font embedding strategy**: resvg requires fonts to be explicitly loaded (no system font fallback). Need to select and bundle a game font (e.g., Inter, Source Sans Pro, or a custom pixel font). Font must be loaded into fontdb before rendering.

2. **SVG template versioning**: As the game evolves, icon templates will change. Need a strategy for versioning templates and invalidating cached renders when templates change. Content hashing of template + params as cache key is the likely approach.

3. **Atlas packing algorithm**: For the final sprite atlas, which packing algorithm? Options: maxrects (most space-efficient), shelf (simplest), or use an existing crate like `texture-packer`. Need to evaluate based on CivLab's icon count and size distribution.

4. **resvg 0.45 vs 0.42 changes**: The task mentioned 0.42.x specifically, but resvg is now at 0.45.x. Need to verify no breaking API changes between these versions. The architecture (usvg + resvg separation) has been stable since 0.28+.

5. **SVG-in-WebGPU alternative**: For runtime rendering (not build-time), could SVGs be rendered directly in the Pixi.js WebGPU pipeline instead of pre-rasterized PNGs? Pixi supports SVG textures via the browser's built-in SVG renderer. This would avoid the build-time pipeline entirely but loses cross-platform determinism.

6. **Color space handling**: resvg supports sRGB. If CivLab's art style uses wide-gamut colors (Display P3), need to verify resvg's color handling. Likely not an issue for stylized game art.

---

## Sources

- [resvg GitHub](https://github.com/linebender/resvg)
- [resvg README](https://github.com/linebender/resvg/blob/main/README.md)
- [resvg Unsupported Features](https://github.com/linebender/resvg/blob/main/docs/unsupported.md)
- [resvg CHANGELOG](https://github.com/linebender/resvg/blob/main/CHANGELOG.md)
- [resvg SVG2 Changelog](https://github.com/linebender/resvg/blob/main/docs/svg2-changelog.md)
- [resvg docs.rs](https://docs.rs/crate/resvg/latest)
- [resvg-js GitHub](https://github.com/thx/resvg-js)
- [roxmltree GitHub](https://github.com/RazrFalcon/roxmltree)
- [roxmltree docs.rs](https://docs.rs/roxmltree/latest/roxmltree/)
- [XML Parsing in Rust — Mainmatter](https://mainmatter.com/blog/2020/12/31/xml-and-rust/)
- [xml-doc GitHub](https://github.com/BlueGreenMagick/xml-doc)
- [librsvg discussion — libvips](https://github.com/libvips/libvips/discussions/2048)
- [resvg Benchmark Issue #185](https://github.com/RazrFalcon/resvg/issues/185)
- [resvg-js sharp comparison — Issue #145](https://github.com/thx/resvg-js/issues/145)


---
