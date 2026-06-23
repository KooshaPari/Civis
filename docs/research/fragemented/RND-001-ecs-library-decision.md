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
- **Solution:** Since Bevy 0.15, `QueryIter` supports `.sort_by::\<Entity\>(...)` and
  `.sort_by_key::<Entity, _>(...)` via `QuerySortedIter`. This sorts entities by `Entity` ID
  (or any component) before iteration, providing deterministic order.
- **Performance cost of sorting:** O(n log n) per query per frame, where n is the number of
  matching entities. For 100k entities this is ~1.7M comparisons -- roughly 50-100us on modern
  hardware. Acceptable for a tick-based simulation (not a 60fps renderer).
- **Alternative:** If sorted iteration is too expensive for hot-path queries, maintain a
  side-channel `Vec\<Entity\>` sorted once on insert, and iterate that instead. But for CivLab's
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
- **No change detection:** Bevy's `Changed\<T\>` and `Added\<T\>` query filters are essential
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
| Query iteration order varies | Use `.sort_by_key::\<Entity\>(Entity::index)` on all simulation queries |
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

3. **Change detection + sorted iteration interaction:** Bevy's `Changed\<T\>` filter narrows
   the query set before iteration. Verify that sorted iteration over `Changed\<T\>` results
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
