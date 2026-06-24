# Perf — Dirty-Flag Stepping + Incremental Remesh (Engine Tick + Mesher)

**Status:** Design (Wave 1 slice)
**Date:** 2026-06-14
**Scope:** `crates/engine/src/engine.rs` (14-phase tick), `crates/voxel` (CA
stepper + kernel), `crates/server/src/voxel_frame_builder.rs` (wire layer),
`clients/bevy-ref/src/voxel_sim.rs` (renderer), `clients/godot-ref/`,
`clients/unreal-show/`
**Requirements addressed:** `NFR-CIV-PERF-003` (tick budget at 200 civilians),
`NFR-CIV-PERF-005` (mesh / terrain budget), `FR-CIV-CA-*` (CA perf
contract), `FR-CIV-VOXEL-002` (deterministic dirty queue)
**Complements:** `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md` (the
CA-side v3 contract; this doc is the **engine + mesher** side that the CA
contract feeds into, plus the wider tick-side hotspots that aren't CA at all)

---

## 1. Context

`Simulation::tick` (`crates/engine/src/engine.rs:1175-1236`) runs **14 ordered
phases per tick** (see `PHASE_ORDER` at `crates/engine/src/engine.rs:61-77`).
For a 256³ world with full agent population and a per-tick CA pass, profiling
on the reference RTX 3090 Ti host (per `agileplus-specs/civ-020`) shows the
**CA freeze class** dominates the per-tick budget:

| Symptom (player-observable) | Where it hurts |
|-----------------------------|----------------|
| Bevy sim throttled to **0.25 Hz** (was 2 Hz) | `clients/bevy-ref/src/voxel_sim.rs:44` |
| "12 Hz froze the frame loop" with 256³ | `clients/bevy-ref/src/voxel_sim.rs:40-43` |
| `parallel_for`-less CA sweeps freeze the render thread | `crates/voxel/src/fluid_ca.rs:1148-1163` |
| Per-frame `commands.entity(e).despawn()` loop on full chunk set | `clients/bevy-ref/src/voxel_sim.rs:632-716` |
| `Despawn all + respawn all` on world re-entry | `clients/bevy-ref/src/voxel_sim.rs:376-395` |
| Game-tick phase iterate-everything walks (compact, climate, faction-loops) | `crates/engine/src/engine.rs:1318-1326, 1752-1840, 1959-2161` |

The known fix pattern is well understood (dirty-chunk stepping +
incremental remesh + `parallel_for`), and pieces of it are already in code
(`CaGrid::dirty_chunks` and `last_changed_chunks`,
`VoxelWorld::drain_dirty` → `DirtyChunkEvent`, the
`step_world_with_config` write-back gate). What's missing is the **uniform
dirty-flag contract across all 14 phases and all 4 clients** + the
**per-tick return signal that lets the renderer skip static remesh**.

This doc locks down that contract and the phased plan to land it. The first
slice (Wave 1) lands the data model + the engine-side `tick() -> TickOutcome`
return; later slices wire the renderer / mesher parallelism and
`Parallel.For` (Godot/Unreal) call sites.

---

## 2. Goals & non-goals

### Goals

- **Working-set bounded per tick.** Each phase's cost is a function of
  `(dirty_set, active_set)` — not the full ECS / voxel / population. A
  static world (no dirty chunks) pays **zero** for the CA + remesh phases.
- **Cheap "did anything change?" return.** `Simulation::step()` /
  `Bevy step_and_remesh` exposes a `bool`/`TickOutcome::changed` that
  the renderer checks before issuing any `despawn`/`respawn` / GPU
  upload. Skipping the static path is the single biggest win.
- **Incremental mesh rebuild.** Only chunks whose kernel dirty event
  surfaces (`DirtyChunkEvent`) are despawned + re-spawned. No "tear down
  the world, rebuild it" path on every CA tick.
- **Parallelisable within a phase.** Phases that already walk an
  independent dirty-set (CA rule passes, agent clusters, building-graph
  frontier) expose a `Parallel.For`-friendly body so the
  Bevy/Godot/Unreal renderer bridge can fan out across cores.
- **Deterministic.** Two same-seed runs of `tick` produce a bit-identical
  `(phase_outputs, dirty_sets, replay hash chain)` — see
  `crates/engine/src/engine.rs:1231-1235` (the existing
  `check_integrity` debug gate) and ADR-004.

### Non-goals (this slice)

- **No Quixel/Megascans import path** — already deferred to engineering
  slots (`Content/Megascans/`) per `fr-l5-visual-pass.md` and the
  Civis 3D AGENTS.md.
- **No multi-camera / split-screen** — single anchor policy only.
- **No shader-side remesh blend** — the mesher still cuts/cleans
  surfaces; cross-chunk visual seam handling stays in
  `streaming-window.md §3.7`.
- **No new modding surface** — dirty flags are not exposed on the
  `civ-mod-host` API (CIV-0700 v3 partial-good only, per AGENTS.md).
- **No worldgen rewrite** — `worldgen::generate` (the 33 s load stall
  in `clients/bevy-ref/src/voxel_sim.rs:459`) is its own workstream;
  the dirty-flag contract only touches the *steady-state* tick path.

---

## 3. Hotspot table

Each row is a real call site in the current main branch. The
"expected impact" is per-tick wall-clock on the reference 256³
+ 200-civilian fixture.

| # | Location | Pattern (current) | Fix | Expected impact |
|---|----------|-------------------|-----|-----------------|
| H1 | `crates/voxel/src/fluid_ca.rs:1148-1163` | Full-grid `for y in 0..dims[1] { for z in 0..dims[2] { for x in 0..dims[0] } }` sweep in `step_with_parity`, then a second full-grid sweep at `:1166-1178` for the gas phase | Scope each triple-loop to `dirty_chunks` + 1-cell halo (already factored via `chunk_is_active` at `:1116-1129`); fan the chunk list across `rayon::iter` or `Parallel.For` so each chunk runs on its own core | **Largest single win.** 256³ × 2 sweeps → ~16 M cell visits → ~active-chunk count. Static world: 0. |
| H2 | `crates/voxel/src/fluid_ca.rs:1139-1142` | `if grid.dirty_chunks.is_empty() { return false; }` — the cheap `changed` bool | Plumb as `StepOutcome { changed, changed_chunks }` (already exists at `:72-79`); make `step_world_with_config` *return* the per-chunk `ChunkId` list (already does at `:1424-1440`) and surface it on `Simulation::last_tick_voxel_events` | Pure signal — no algorithmic change, but unlocks H5. |
| H3 | `crates/voxel/src/fluid_ca.rs:436-482` (`chunks_changed_from`) | Per-cell diff walk over every chunk to compute `last_changed_chunks` (4096 cells/chunk worst case, breaks out at first cell flip) | Keep as-is — the early-out makes cost ∝ *changed* chunks, not total. Add a parallel-prefix break so a chunk that diverges at cell 1 still uses a vectorised cell-comparison loop body | Cheap; mostly correctness / bench. |
| H4 | `crates/voxel/src/fluid_ca.rs:1142, 1165` | Two `grid.clone()` calls per tick (one for `before`, one for `prev`) — the full `cells`/`temperatures`/`saturation` triple | Replace `before` with the existing `scratch` (`scratch_view`/restore_scratch at `:276-296`) — `before` is only read after the rule passes finish, so a single `mem::take` of scratch (with the existing swap-back) eliminates one of the two clones | Halves the per-tick allocation cost on the CA path. |
| H5 | `clients/bevy-ref/src/voxel_sim.rs:632-716` (`step_and_remesh`) | After every CA tick, walk `last_changed_chunks`, despawn the old mesh entities for those chunks, dispatch a new async mesh task for each, await the result | Gate the whole despawn/respawn block on `outcome.changed` (currently `step()` is called, but `if !outcome.changed { return; }` is *not* short-circuiting the next CA tick correctly when the static world is initialised with non-zero `dirty_chunks` from `mark_mobile_chunks`); use `last_changed_chunks` (already a `HashSet<ChunkId>` at `:667-680`) as the *exclusive* set for the despawn loop (already done at `:689-701`) — the gap is the *initial-frame* path that spawns 100% of the world mesh on world build | Static world: despawn 0, respawn 0, async tasks 0. |
| H6 | `clients/bevy-ref/src/voxel_sim.rs:40-44` | `CA_TICK_HZ = 0.25` — the renderer is throttled to 1 step / 4 s because the underlying CA is still expensive | With H1 + H4, the CA cost drops ~2–4× on the dirty-chunk subset; raise `CA_TICK_HZ` back toward 4–8 Hz in steps gated on `bench_ca_dirty_chunk` P99 | Visible only — lets the player see flowing water again. |
| H7 | `clients/bevy-ref/src/voxel_sim.rs:376-395` (`despawn_world_and_pending`) | On world re-entry, the renderer **despawns every chunk entity** (`chunk_entities.drain()` flattens the whole map) and re-spawns from scratch — this is a "despawn all + rebuild all" freeze on every menu round-trip | Replace with **incremental world-build**: keep `chunk_entities` populated across the menu trip, only despawn chunks the new world is going to overwrite (compare seeds, compare dims — if identical, keep the meshes; if different, only despawn the dirty-diff). The current `build_voxel_world` at `:445-540` always re-dispatches mesh tasks even when the input didn't change | Eliminates the menu round-trip freeze; static world re-entry: 0 mesh tasks. |
| H8 | `crates/engine/src/engine.rs:1570-1749` (`phase_life`) | Full ECS walk: collect all agents (`world.query::<&AgentCivilian>().iter()` at `:1598-1603`), then loop over them, then build a `HashMap<u64, Entity>` (`id_to_entity` at `:1716-1721`), then `cluster_by_colocation(&positions, …)` (the 2-D O(N²) cluster pass at `:1713`) | Scope the per-entity loop to a **dirty-agent set** that `phase_life` itself produces: a civilian only needs re-evaluating when their position/cluster/need/health actually changed this tick. Drop the `id_to_entity` HashMap rebuild by maintaining an `agent_id -> entity` `BTreeMap` in `Simulation` and only invalidating on despawn (already done partially via `last_deaths`) | Agent iteration O(N) → O(ΔN) where ΔN is the per-tick movement fraction. At 200 civilians with 5 % movement: ~10× reduction. |
| H9 | `crates/engine/src/engine.rs:1752-1776` (`phase_production`) | Full ECS walk over all buildings every tick (`world.query::<&Building>().iter()` at `:1758`) | Cache a `BuildingByType: HashMap<BuildingType, Vec<Entity>>` in `Simulation` and rebuild it on building-graph delta (the `building_graph` already at `:401`); `phase_production` reads from the cache | 6 type buckets vs N walk — small but cumulative. |
| H10 | `crates/engine/src/engine.rs:1779-1840` (`phase_citizen_lifecycle`) | Full ECS walk + spawn `births` + despawn `dead` for every citizen tick (line 1791) | Combine with H8: lifecycle only re-evaluates agents in the dirty set (the LOD policy at `:1779-1780` already gates Warm/Cold tiers — the same gate should feed the dirty set) | LOD-tier agents skip outright; birth/death logic runs on the Hot tier only. |
| H11 | `crates/engine/src/engine.rs:1959-2161` (`phase_diplomacy`) | O(F²) faction pair loop at `:2013-2022` with 6 gradient signals each, plus a `HashSet<ClusterId>` rebuild for proximity at `:2067-2088` | Stable — F (factions) is small (3 default, < 10 realistic). The cluster rebuild is O(N) over agents and can be cached & invalidated by H8. **Not on the critical path** for the 200-civ budget; defer | Low priority; cluster cache piggybacks on H8. |
| H12 | `crates/engine/src/engine.rs:1318-1326` (`phase_planet`) | Recomputes climate + weather grid *every tick* (`compute_climate(tick, …)` and `compute_weather(&climate, tick, n)` at `:1319-1324`) — pure functions of `(tick, planet, moon)` | Cache the `(climate, weather_grid)` keyed on `(tick // weather_cadence, planet.axial_tilt_deg)`; recompute only on `tick % cadence == 0` (cadence = 16 mirrors the existing `phase_buildings` cadence at `:1502`) | ~5–10 % of the tick budget on a quiet world. |
| H13 | `crates/engine/src/engine.rs:1489-1491` (`phase_voxel`) | Drains the kernel's dirty queue every tick (`self.voxel.drain_dirty()`); the queue can be empty | Already correct; document the `is_empty()` short-circuit and have downstream consumers (Bevy `step_and_remesh`, server `voxel_frame_builder`) check it | Free. |
| H14 | `crates/engine/src/engine.rs:1194-1236` (`run_tick`) | 14 phases in series; no phase parallelism (determinism-friendly) | Keep sequential, but each phase returns a `PhaseOutcome { changed: bool, dirty_count: usize }`; `run_tick` aggregates them into `TickOutcome { changed, changed_phases, dirty_set_hashes }` so callers see *which* phase did work without re-deriving | Foundation for everything below. |
| H15 | `crates/server/src/voxel_frame_builder.rs:42-94` (`build_voxel_delta_frame`) | Walks `events` once; builds one `VoxelChunkDelta` per `chunk_id` (sorted by `(chunk_id, write_seq)`). The `build_chunk_delta` helper at `:81-94` returns a **zero-payload** delta as a TODO | When `VoxelWorld::chunk(coord) -> Option<&Chunk>` lands (referenced in the comment at `:85-88`), swap the zero-payload delta for the dense leaf payload; the wire protocol stays identical | Saves a `read()` per cell on the consumer side; the GPU upload can skip the per-cell re-derivation. |
| H16 | `clients/godot-ref/`, `clients/unreal-show/` | Godot and Unreal clients mirror the Bevy `step_and_remesh` + `despawn_world_and_pending` patterns (per AGENTS.md "Bevy/Godot/Unreal server attach") | Same contract — the Godot/Unreal mesher consumes `last_changed_chunks` as its exclusive set; `Parallel.For` on the mesher job (Godot's `WorkerThreadPool`, Unreal's `ParallelFor`) for the chunk-mesh compute (the smooth mesher per-chunk cost is the inner loop) | Cross-client perf contract — locked here so the server-side wire format and the dirty-set semantics stay uniform. |

**Reference numbers** (from `civ-020` bench target): wave-1 baseline
~45 ms / 64×64 grid, 1 % writes; target P99 < 16 ms post-fix.
The table above extends that to the engine + mesher side, where the
biggest absolute cost is H1 (the CA full-grid sweep) and H7 (the menu
round-trip freeze).

---

## 4. Architecture

### 4.1 The dirty-flag data model

```rust
// crates/engine/src/engine.rs

/// Outcome of a single phase. `changed` is the cheap scalar check; the
/// optional `dirty_set` is the per-phase *active set* the renderer /
/// downstream phase should walk (vs the full ECS / voxel substrate).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhaseOutcome {
    /// True when at least one observable side effect (voxel write, agent
    /// spawn/despawn, faction event, etc.) happened in this phase.
    pub changed: bool,
    /// Phase-specific dirty set. `VoxelDirty { chunks }` for the voxel
    /// phase; `AgentDirty { entities }` for the life / citizen / military
    /// phases; `Empty` for phases that have no per-entity fan-out
    /// (climate, diplomacy, economy).
    pub dirty_set: DirtySet,
    /// Wall-clock duration of the phase in microseconds (FR-CORE-007 —
    /// mirrors the existing `TickProfile`).
    pub duration_micros: u64,
}

/// Aggregate dirty-set across all phases for one tick. The renderer
/// checks `changed` first (cheap); if true, it walks `dirty_set` (still
/// bounded — never the full world).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DirtySet {
    /// No observable side effect; downstream consumers MUST skip their
    /// remesh / rebuild work.
    #[default]
    Empty,
    /// One or more voxel chunks changed. The renderer walks `chunks`
    /// and remeshes exactly those.
    VoxelDirty { chunks: Vec<ChunkId> },
    /// One or more agent entities need a re-skin / re-spawn (used by
    /// the cross-client bridge; not the chunk mesher).
    AgentDirty { entities: Vec<u64> },
    /// Multiple kinds changed in the same tick (rare — economy + life).
    Mixed(Box<MixedDirty>),
}

/// Bundle for `DirtySet::Mixed`. Lives in its own struct so the enum
/// stays small in the common `Empty` / `VoxelDirty` cases.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MixedDirty {
    pub voxel: Vec<ChunkId>,
    pub agents: Vec<u64>,
    pub combat_pulses: u32, // FR-CIV-TACTICS-024
    pub diplomacy_events: u32, // CIV-007
}

/// Aggregate per-tick result. Returned by `Simulation::step()` (new)
/// and `Simulation::tick_profiled()` (extended). The deterministic
/// production path stays `tick() -> ()`; the observed path returns
/// `TickOutcome`. Same phase list, same world output, no drift.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TickOutcome {
    pub tick: u64,
    /// True when ANY phase reported `changed`. The renderer's master
    /// gate: when false, despawn 0, respawn 0, GPU upload 0.
    pub changed: bool,
    /// Per-phase outcomes in `PHASE_ORDER`. The renderer / replay log
    /// / perf HUD can read this without re-deriving.
    pub phases: Vec<(PhaseId, PhaseOutcome)>,
    /// Replay-bus JSON lines (`mod.loaded.v1`, `combat_pulse.v1`, etc.)
    /// — already produced; lifted to the aggregate level so callers
    /// don't have to reach into the replay log for the common case.
    pub bus_events: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhaseId {
    Production,
    CitizenLifecycle,
    Military,
    Economy,
    Planet,
    Diplomacy,
    Tactics,
    Voxel,
    Compact,
    Buildings,
    Diffusion,
    Disasters,
    Life,
    Emergence,
}
```

The enum is **stable** (14 variants, fixed order = `PHASE_ORDER`); adding a
phase is a non-event (add a variant, a phase, and a test in
`phase_order_matches_tick_sequence` at `crates/engine/src/engine.rs:2613-2633`).
The `DirtySet` is **intentionally a flat enum**, not a trait object, so the
`Empty` case is a single byte and the `VoxelDirty` case is a
`Vec<ChunkId>`-only struct (matches the wire format that
`voxel_frame_builder.rs` already produces).

### 4.2 `step()` returns `TickOutcome`

The current `Simulation::tick()` at `crates/engine/src/engine.rs:1175-1177`
returns `()`. The new contract is:

```rust
impl Simulation {
    /// Advance the simulation by one tick. Returns the per-tick
    /// `TickOutcome` so the renderer / bridge can skip work when the
    /// world is static.
    ///
    /// The deterministic production path is `tick()` (returns `()`)
    /// and wraps `step()` ignoring the result. Replay log + wire
    /// format are unchanged.
    pub fn step(&mut self) -> TickOutcome {
        self.run_step(None)
    }

    fn run_step(&mut self, mut profile: Option<&mut TickProfile>) -> TickOutcome {
        let mut outcome = TickOutcome { tick: self.state.tick, ..Default::default() };
        macro_rules! phase {
            ($id:expr, $name:literal, $body:expr) => {{
                let start = Instant::now();
                let phase_outcome: PhaseOutcome = $body;
                let elapsed = start.elapsed().as_micros() as u64;
                if let Some(p) = profile.as_deref_mut() {
                    p.record($name, elapsed);
                }
                if phase_outcome.changed { outcome.changed = true; }
                outcome.phases.push(($id, phase_outcome));
            }};
        }
        // Phase bodies return PhaseOutcome instead of ().
        phase!(PhaseId::Production, "production", self.phase_production());
        phase!(PhaseId::CitizenLifecycle, "citizen_lifecycle", self.phase_citizen_lifecycle());
        // … etc, all 14 phases …
        self.replay_log.record_tick(self.state.tick);
        outcome
    }
}
```

Each `phase_*` method is updated to return `PhaseOutcome` (semantically a
trivial refactor — the phase already knows whether it changed anything;
the only change is the return type and the explicit
`PhaseOutcome { changed: <bool>, dirty_set: <enum> }` at the end of
each function body).

`Simulation::tick()` continues to return `()` and discards the
`TickOutcome`. This is the **determinism-preserving compatibility
shim** — the production path is byte-identical for a given seed; the
new `step()` is the **observed path** the renderer consumes.

### 4.3 `Simulation::voxel_dirty_chunks()` accessor

`Simulation::last_tick_voxel_events()` at `crates/engine/src/engine.rs:1071-1073`
already returns the kernel's `DirtyChunkEvent` slice. The new
**typed** accessor wraps it:

```rust
impl Simulation {
    /// Per-chunk `ChunkId`s whose cells actually changed on the most
    /// recent tick. Empty when no phase produced a voxel write.
    /// Mirrors `last_tick_voxel_events()` but with a `ChunkId`-shaped
    /// type the mesher can dispatch on directly.
    #[must_use]
    pub fn last_tick_voxel_chunk_ids(&self) -> Vec<ChunkId> {
        let mut out: Vec<ChunkId> = self
            .last_tick_voxel_events
            .iter()
            .map(|e| e.chunk_id)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}
```

The Bevy client currently re-derives this from `CaGrid::last_changed_chunks`
at `clients/bevy-ref/src/voxel_sim.rs:667-680`. After this slice, the
canonical source of truth is `Simulation::last_tick_voxel_chunk_ids()`;
the Bevy accessor is deprecated to a thin shim that calls it.

### 4.4 Renderer-side: `step_and_remesh` short-circuit

```rust
// clients/bevy-ref/src/voxel_sim.rs (updated)
pub fn step_and_remesh(/* … */) {
    // (unchanged fixed-step accumulator)
    state.accumulator += time.delta_secs();
    let step_dt = 1.0 / CA_TICK_HZ;
    if state.accumulator < step_dt { return; }
    state.accumulator = (state.accumulator - step_dt).min(step_dt);

    // CA step — local, no engine coupling.
    let outcome = step(&mut state.grid, MaterialRegistry::standard());
    if !outcome.changed {
        // Static world: zero remesh, zero GPU upload. Single check.
        return;
    }
    state.tick = state.tick.wrapping_add(1);

    // `outcome.changed_chunks` is the *exclusive* remesh set (already
    // sorted, already dedupped at crates/voxel/src/fluid_ca.rs:1352-1353).
    let changed_set: HashSet<ChunkId> =
        outcome.changed_chunks.iter().copied().collect();
    if changed_set.is_empty() { return; } // defensive: `changed` implies non-empty

    // Despawn ONLY the chunks in `changed_set` (already gated at
    // voxel_sim.rs:689-701 — kept).
    // …
    // Re-spawn ONLY the chunks in `changed_set` (already done at
    // voxel_sim.rs:703-715 — kept).
}
```

The `outcome.changed` short-circuit is the **single most impactful
change** in this slice. Combined with `last_changed_chunks` being
empty for a static world (`crates/voxel/src/fluid_ca.rs:1399-1406`),
the renderer does zero work for a static world.

### 4.5 Parallelism: `Parallel.For` on the chunk loop

The CA triple-loop at `crates/voxel/src/fluid_ca.rs:1148-1163` is
**embarrassingly parallel**: each chunk reads from the `before` grid
(snapshot), writes to `grid`, and the only cross-chunk reads are the
1-cell halo (already cloned into the `cells` index list at `:393-425`).

The phased plan replaces the serial triple-loop with:

```rust
// crates/voxel/src/fluid_ca.rs (sketch — first slice, Bevy/rayon)
use rayon::prelude::*;

let active_chunks: Vec<usize> = before.dirty_chunks.iter().copied().collect();
let per_chunk = |chunk: usize| {
    // Returns a Vec<(linear_index, new_material, new_temp)> of writes
    // for this chunk. No cross-chunk writes.
    run_chunk_simulation(&before, chunk, reg, tick, sea_level, parity)
};
let writes: Vec<ChunkWrites> = active_chunks.par_iter().map(|&c| per_chunk(c)).collect();
// Commit phase — serial, but cheap.
for w in writes { commit_chunk_writes(grid, w); }
```

The **commit phase** is a single linear pass over the write set,
deterministic in (cx, cy, cz) order — the same total order the
existing serial loop produces. Replay equality holds.

For Godot (`clients/godot-ref/`) and Unreal
(`clients/unreal-show/`), the same pattern uses
`WorkerThreadPool.add_native_task` /
`ParallelFor(chunk_count).Body(...)` respectively. The
`crates/voxel` body is identical; only the *fan-out* differs.

### 4.6 Incremental world build (menu round-trip fix)

`despawn_world_and_pending` at `clients/bevy-ref/src/voxel_sim.rs:376-395`
**drains `chunk_entities` entirely** and re-spawns the world from
scratch. The fix is a **seeded incremental build**:

```rust
pub fn build_world_on_play(/* … */) {
    // …
    let prev_seed = state.last_build_seed;
    let prev_dims = state.last_build_dims;
    if built.0 && prev_seed == Some(params.seed) && prev_dims == Some(dims) {
        // Same world — keep the meshes. Just re-arm the dirty set
        // (worldgen may have changed since last build; the
        // `last_build_seed` was set on the previous build).
        return;
    }
    // Different world — despawn only the chunks the new world will
    // not produce. The full drain stays as the worst-case fallback.
    despawn_world_and_pending(/* … */);
    // …
    state.last_build_seed = Some(params.seed);
    state.last_build_dims = Some(dims);
}
```

For the **first build** (`!built.0`), the despawn is a no-op; the
world re-entry cost is **1 mesh dispatch per chunk** (already async
via `dispatch_chunk_mesh_tasks`), not "despawn all + respawn all".

---

## 5. Phased plan

Each task is named after the **real function name** it lands in; the
slice deliverable is the version that compiles + the tests that
guard the contract.

### Phase 0: Foundation (this slice, Wave 1)

| Task | Function(s) | Deliverable | Test |
|------|-------------|-------------|------|
| F1 | `crates/engine/src/engine.rs` — add `PhaseOutcome`, `DirtySet`, `MixedDirty`, `TickOutcome`, `PhaseId` types (no body change yet) | Types compile, `phase_order_matches_tick_sequence` still passes | New unit test in `engine` asserting enum sizes (e.g. `assert!(std::mem::size_of::<DirtySet>() <= 32)`) |
| F2 | `crates/engine/src/engine.rs:1175-1236` — add `pub fn step(&mut self) -> TickOutcome` alongside `tick()`; rewire `run_tick` to `run_step` and return `PhaseOutcome` from each phase fn body | `step()` compiles, `tick()` unchanged, replay equality holds | Extend `test_tick_advances` (`engine.rs:2565-2570`) to assert `step().tick == 1` |
| F3 | `crates/engine/src/engine.rs` — make each `phase_*` return `PhaseOutcome` (mechanical: 14 functions, ~14 lines of body change each — `return PhaseOutcome { changed: <expr>, dirty_set: <enum> }` at the end) | All phases compile, no body change beyond return type | Extend `tick_profiled_records_all_phases` (`engine.rs:2574-2589`) to assert 14 `PhaseOutcome` entries |
| F4 | `crates/engine/src/engine.rs` — add `last_tick_voxel_chunk_ids()` (per §4.3) | New accessor | New unit test asserting dedup + sort |
| F5 | `clients/bevy-ref/src/voxel_sim.rs:652-655` — replace `let outcome = step(...); if !outcome.changed { return; }` with **the new `outcome.changed` short-circuit** (it's already there; the change is to ensure the **next** CA tick isn't issued when the world is truly static — i.e. the dirty-set should drain to empty after the first tick on a static world) | Static world: zero remesh, zero mesh dispatch | New Criterion bench `bench_bevy_step_static` asserting zero `despawn` calls |
| F6 | `clients/bevy-ref/src/voxel_sim.rs:376-395` — incremental world build per §4.6 | Menu round-trip = zero mesh dispatch for unchanged world | New unit test in `chunk_exposure_tests` style (already in voxel_sim.rs:100-141) |

**Wave 1 deliverable:** the engine returns a `TickOutcome`, the Bevy
client short-circuits on `outcome.changed`, and the menu round-trip
is incremental. Static-world remesh cost drops to **0 ms/tick** (the
gate). CA path is still serial — Phase 1 is the parallelism slice.

### Phase 1: CA parallelism (Wave 2)

| Task | Function(s) | Deliverable | Test |
|------|-------------|-------------|------|
| C1 | `crates/voxel/src/fluid_ca.rs:1148-1163, 1166-1178` — replace serial triple-loops with `per_chunk` closure + `rayon::iter::ParallelIterator` for Bevy/native; feature-gate on `parallel` so the wasm/embedded build keeps the serial path | `bench_ca_dirty_chunk` P99 < 16 ms (per `civ-020` acceptance) | Existing `civ-020` bench wired into `just civis-3d-verify` |
| C2 | `crates/voxel/src/fluid_ca.rs:1142, 1165` — replace the two `grid.clone()` (the `before` and `prev` snapshots) with the existing scratch `mem::take` / `restore_scratch` (`:276-296`) | One fewer full-grid allocation per tick | New Criterion micro-bench asserting CA allocation count == 1 (down from 2) |
| C3 | `clients/godot-ref/rust/` — port the Bevy `step_and_remesh` short-circuit to Godot's `WorkerThreadPool`; the `per_chunk` closure body is the same as C1 | Godot client ticks at 4 Hz on a static world | Smoke test in `just godot-test` |
| C4 | `clients/unreal-show/` — port the same short-circuit to Unreal's `ParallelFor`; the body is the same | Unreal CivShow ticks at 4 Hz on a static world | Manual PIE run + `scripts/agent-smoke.ps1 -FullUnreal` |

**Wave 2 deliverable:** CA path is parallel + allocation-light;
all 3 clients tick at 4 Hz on the same dirty set.

### Phase 2: Engine-side hot loops (Wave 3)

| Task | Function(s) | Deliverable | Test |
|------|-------------|-------------|------|
| E1 | `crates/engine/src/engine.rs:1570-1749` — refactor `phase_life` to walk only the dirty-agent set (the LOD policy at `:1779-1780` is the source); cache `id_to_entity` as a `BTreeMap<u64, Entity>` in `Simulation`; only invalidate on despawn (H8) | Agent loop ∝ ΔN, not N | New bench `bench_phase_life_movement_fraction` |
| E2 | `crates/engine/src/engine.rs:1752-1776` — `phase_production` reads from `BuildingByType` cache (H9) | Walk goes from O(N buildings) to O(6 type buckets) | Existing `test_tick_advances` covers correctness |
| E3 | `crates/engine/src/engine.rs:1779-1840` — `phase_citizen_lifecycle` shares E1's dirty-agent set (H10) | LOD-tier agents skip outright | New unit test asserting Cold-tier agents do not appear in `last_births` / `last_deaths` |
| E4 | `crates/engine/src/engine.rs:1318-1326` — `phase_planet` cadence cache (H12) | Climate recomputes on `tick % 16 == 0` | New unit test asserting the cache hit/miss boundary |

**Wave 3 deliverable:** the engine-side hot loops scale with
`(active_set, dirty_set)`, not the full ECS. The 200-civ tick budget
fits the target frame time on the reference host.

### Phase 3: Wire + GPU (Wave 4)

| Task | Function(s) | Deliverable | Test |
|------|-------------|-------------|------|
| W1 | `crates/server/src/voxel_frame_builder.rs:81-94` — replace the zero-payload `build_chunk_delta` with the dense leaf payload from `VoxelWorld::chunk(coord) -> Option<&Chunk>` (lands in `civ-voxel` first) | Wire payload is dense, not zero | New test asserting `delta.voxels.len() == 4096` for non-air chunks |
| W2 | `clients/bevy-ref/src/voxel_sim.rs:1053-1224` — make the chunk-mesh async task **conditional on `changed_chunks`** (currently `dispatch_chunk_mesh_tasks` dispatches all visible chunks — restrict to `changed` set) | Mesh compute cost ∝ |changed|, not |visible| | New unit test asserting task count == |changed| |
| W3 | `clients/bevy-ref/src/voxel_smooth_mesher.rs` — expose a `build_smooth_meshes_chunk(&chunk, &apron) -> MeshBuffer` for a single chunk (currently a `Vec<MeshBuffer>` over the whole world) | Single-chunk remesh is a pure function | New test in `voxel_smooth_mesher` |

**Wave 4 deliverable:** the wire format and the GPU upload both
respect the dirty-set contract. Cross-client parity is uniform
(Bevy / Godot / Unreal all consume the same `ChunkId` list).

---

## 6. Determinism contract

The dirty-flag contract is **replay-equivalent** to the current code:

- **Same `(seed, scenario, snapshot)` → same `TickOutcome`.** Two
  same-seed runs of `step()` produce bit-identical `TickOutcome`
  (the `chunks` field is sorted + dedupped; `changed` is a function
  of `(phase_outputs, dirty_sets)`).
- **Same replay hash chain.** `replay_log.record_tick` is unchanged;
  the BLAKE3 root at `crates/engine/src/engine.rs:1297-1299` is the
  same byte-for-byte.
- **`step()` and `tick()` produce the same world.** `tick()` is
  literally `let _ = self.step();` — the phase list, the RNG draws,
  the voxel writes, the agent spawns/despawns are unchanged.

The only **non-deterministic** piece is wall-clock (the
`TickProfile` durations). That's already documented at
`crates/engine/src/engine.rs:1180-1183` ("the timing path is
observability only and never touches simulation state").

The CA parallelism in Phase 1 is **also deterministic** because the
per-chunk closure writes into a fresh `Vec<ChunkWrites>` and the
commit phase walks in `(cx, cy, cz)` order — the same order the
serial loop produces. The `rayon::iter` join doesn't change the
output.

---

## 7. Cross-references

- `FUNCTIONAL_REQUIREMENTS.md` § NFR-CIV-PERF-003, NFR-CIV-PERF-005
- `agileplus-specs/civ-020-ca-perf-dirty-chunk/spec.md` — the
  CA-side v3 contract; this doc is the engine + mesher extension
- `docs/design/streaming-window.md` — the chunk-LOD + ring policy
  (the window decides *which* chunks are in the active set; this
  doc decides *what to do* with the active set on a given tick)
- `docs/adr/ADR-004-deterministic-replay.md`,
  `docs/adr/ADR-005-adaptive-voxel.md` — the kernel guarantees
  the dirty contract inherits
- `crates/engine/src/engine.rs:1175-1236` — `run_tick` (the phase
  loop; becomes `run_step` returning `TickOutcome`)
- `crates/voxel/src/fluid_ca.rs:72-79` — `StepOutcome { changed,
  changed_chunks }` (the CA-side analogue; the engine's
  `TickOutcome` aggregates per-phase `PhaseOutcome`s)
- `crates/voxel/src/fluid_ca.rs:1399-1406` — the static-world
  zero-work contract (already in code; the renderer now respects it)
- `crates/server/src/voxel_frame_builder.rs:42-94` — the wire layer
  (Phase 3 lifts the zero-payload TODO)
- `clients/bevy-ref/src/voxel_sim.rs:40-44` — the 0.25 Hz throttle
  (Phase 1 raises it back)
- `clients/bevy-ref/src/voxel_sim.rs:376-395` — the
  despawn-all + respawn-all freeze (Phase 0 fixes the
  round-trip; the per-tick despawn is already incremental at
  `:689-701`)

---

## 8. Open questions (follow-up slices)

1. **Dirty-set propagation across phases.** Some phases (e.g. `life`)
   produce an agent dirty set that downstream phases (e.g. `military`)
   consume. The current `PhaseOutcome.dirty_set` is per-phase; a
   `cumulative_dirty_set()` accessor may be needed for the cross-phase
   optimisation in Phase 2 (E1 → E3). Defer to Wave 3 review.
2. **Bevy ECS component-level dirty flags.** The current contract is
   *phase-level*; per-component dirty flags would let `phase_military`
   skip a unit that hasn't moved this tick. Bigger refactor; defer.
3. **GPU upload streaming.** The Bevy client currently uploads the
   full mesh; the dirty chunk list should drive a partial upload. W2
   is the first slice; the actual GPU side (a `BufferUsage::Dynamic`
   + `set_buffer_sub_data` per chunk) is a renderer change beyond
   this doc.
4. **Cluster cache across `phase_life` and `phase_diplomacy`.** H11
   already notes the proximity `HashSet<ClusterId>` rebuild; sharing
   a single `ClusterIndex` between the two phases is a clean follow-up.
5. **Replay determinism under parallelism.** Phase 1's rayon commit
   phase is order-stable, but `cargo test` parallelism (separate
   worktrees, separate `CARGO_TARGET_DIR`) may surface a flaky
   `bench_ca_dirty_chunk` — the bench needs a fixed-thread
   `rayon::ThreadPoolBuilder` for CI parity. Defer to `civ-020`
   bench-gate workstream.
