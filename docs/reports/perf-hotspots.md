# Performance Hotspots Audit

> Branch: `feat/frecon005-allocation`  
> Scope: `crates/engine/src/engine.rs` tick phases + `clients/bevy-ref/src/voxel_sim.rs`  
> Generated: 2026-06-13

---

## 1. Engine Tick Phases — `crates/engine/src/engine.rs`

### 1.1 `phase_diplomacy` — O(n²) pairwise faction loop + nested full-population scans

**Location:** `crates/engine/src/engine.rs:1940-2141`

- **Pattern:** `pairs` builds an O(f²) vector over faction IDs. For every pair, `phase_diplomacy` iterates the entire `trade_routes` vector to compute `trade_volume`, and then iterates **all living civilians** with `ClusterMember` to build proximity sets (`clusters_a`, `clusters_b`).
- **Impact:** With `f` factions and `n` civilians, each tick does ~O(f² × (r + n)) work. For 128 civilians and 3 factions this is negligible, but if factions scale to 10+ and civilians to 1,000+, this becomes the dominant tick cost.
- **Allocations in hot loop:** `HashSet<ClusterId>` is allocated fresh for every pair on every tick. `HashSet::with_capacity` is never used, so the sets reallocate repeatedly as they grow.
- **Dirty-flag opportunity:**
  - `trade_routes` rarely change (only on war→trade suppression or scenario load). Cache a `BTreeMap<(u32,u32), Fixed>` of pre-computed trade volumes and invalidate only when routes mutate.
  - `clusters_a` / `clusters_b` only change when civilians move clusters. The life phase already computes `cluster_by_colocation` results; cache faction→cluster-set membership and reuse it here instead of re-scanning the world.

### 1.2 `phase_tactics` — O(n²) pulse dedup + engagement loops

**Location:** `crates/engine/src/engine.rs:1391-1463`

- **Pattern:** `self.last_tick_combat_pulses.iter().any(|pulse| ...)` is executed for every pending `DamageEvent`. If multiple damage events hit the same area, this is O(d × p) where `d` is damage events and `p` is pulses. In burst-fire or explosion-heavy ticks this can become quadratic.
- **Pattern:** The doctrine-evolve branch (every 64 ticks) builds `vec![FactionEngagementStats::default(); self.faction_doctrines.len()]` from scratch, then iterates `last_tick_engagements` twice (once to accumulate shooter/target stats, once to distribute per-shooter voxel damage).
- **Allocations:** `faction_stats` Vec is allocated every 64 ticks. `last_tick_engagements.clone()` is called in `phase_military` (`crates/engine/src/engine.rs:1890`) before tactics even runs, so engagements are cloned twice per tick.
- **Dirty-flag opportunity:**
  - Pulse dedup should use a `HashSet<(u32, u32)>` of quantized grid coordinates instead of a linear scan.
  - Doctrine fitness only needs to be recomputed when engagements actually change (war-bridge cadence is already 16 ticks; doctrine evolution is 64 ticks). Track a `bool: engagements_changed_since_last_doctrine_tick` and skip the entire block when false.

### 1.3 `phase_life` — Six full-world scans + O(n²) clustering

**Location:** `crates/engine/src/engine.rs:1551-1730`

- **Pattern:** This phase performs the following full passes over the entity population every single tick:
  1. `missing` scan — all `AgentCivilian` without `LifeNeeds` (`world.query::<&AgentCivilian>().iter().filter(...)`)
  2. `build_poi_registry` — all `Building` entities (full pass, rebuilds registry every tick)
  3. `entities` collection — all `AgentCivilian` into a `Vec<Entity>`
  4. Per-entity inner loop — multiple `world.get::<&...>` calls (ECS random access, not a scan, but still scattered)
  5. `positions` collection — all `(AgentCivilian, Position3d)` into a `Vec`
  6. `id_to_entity` HashMap — all `AgentCivilian` into a map
  7. `cluster_sizes` BTreeMap — rebuilt from scratch
  8. `next_stocks` BTreeMap — rebuilt from scratch
- **Pattern:** `cluster_by_colocation` (called at line 1694) is an O(n²) density-based clustering algorithm over all positions. With 128 agents this is ~16k distance checks; with 1,000 agents it is ~1M checks per tick.
- **Allocations in hot loop:** `Vec::with_capacity` is never used for the large intermediate collections. `next_stocks` is a brand-new `BTreeMap` every tick; `cluster_stocks` is dropped and replaced. Each agent's daily path may allocate a `ChaCha8Rng` (seeded per-entity) on every `Wander` or `SeekNeed` branch.
- **Dirty-flag opportunity:**
  - `build_poi_registry` should be cached. Buildings only change when `phase_buildings` allocates new parcels (every 16 ticks). Store a `poi_registry_dirty: bool` and rebuild only when true.
  - `cluster_by_colocation` results can be cached with a `positions_changed_since_last_cluster` flag. Agent positions only move when they are actively pathing (a subset of the population each tick). Most agents idle.
  - `next_stocks` can be updated in-place: iterate `cluster_stocks` and `add` the new production, rather than rebuilding the entire map.
  - `id_to_entity` can be maintained incrementally: a `BTreeMap<u64, Entity>` on `Simulation` that is updated whenever `spawn_civilian_at` or `spawn_child_near` runs, and on entity despawn.

### 1.4 `phase_diffusion` — Double-counting full population twice per phase

**Location:** `crates/engine/src/engine.rs:1515-1540` (delegates to `propagate_cohort_wardrobe_with_lod` and `propagate_cohort_tools_with_lod` at lines 548-638)

- **Pattern:** Each helper function:
  1. Counts total civilians via `count_civilians(world)` (full pass)
  2. Counts `currently_at_target` via `world.query::<&Wardrobe>().iter().filter(...).count()` (full pass)
  3. Iterates `world.query_mut::<(&mut Wardrobe, &LodTier)>()` (full pass)
  4. Re-counts `currently_at_target` again (full pass)
- **Impact:** For wardrobe + tools, this is **6 complete entity scans** every tick. At 128 entities this is cheap; at scale it dominates.
- **Dirty-flag opportunity:**
  - `count_civilians` and the target-era counts can be maintained as counters on `Simulation` that are incremented on spawn and decremented on despawn. This removes all 4 counting passes.
  - `current_fraction` only needs to be recomputed once per tick, not twice per helper. Compute it once in `phase_diffusion` and pass it down.

### 1.5 `phase_military` — O(n²) unit-update loops

**Location:** `crates/engine/src/engine.rs:1824-1929`

- **Pattern:** After `tick_operational_movement` returns a list of `grid_move`s, the code does:
  ```rust
  for grid_move in tick_operational_movement(...) {
      if let Some(target_entity) = entities.get(grid_move.unit_index).copied() {
          for (entity, unit) in self.world.query_mut::<&mut MilitaryUnit>() {
              if entity == target_entity { ... break; }
          }
      }
  }
  ```
  This is O(m × u) where `m` is movement pulses and `u` is unit count. The inner `query_mut` iterates every military unit for every movement pulse. A `HashMap<Entity, &mut MilitaryUnit>` would make this O(u + m).
- **Pattern:** The same anti-pattern appears for applying engagement damage (lines 1894-1917): for each engagement, a full `query_mut::<&mut MilitaryUnit>()` is run to find the target entity.
- **Allocations:** `entities` and `samples` Vecs are collected from scratch every tick. `engagements.clone()` is called before the loop (line 1890).
- **Dirty-flag opportunity:**
  - Build a `HashMap<u32, Entity>` (or `hecs` entity index) from `unit_id` → `Entity` once per tick, then use direct `get::<&mut MilitaryUnit>(entity)` lookups instead of scanning the world.
  - `tick_operational_movement` already returns unit indices; if it also returned the entity handle, the inner loop could be eliminated entirely.

### 1.6 `phase_economy` + `tiered_demand` — Full-needs scan every tick

**Location:** `crates/engine/src/engine.rs:2168-2232`

- **Pattern:** `tiered_demand` iterates `self.world.query::<&LifeNeeds>().iter()` to compute aggregate unmet pressure across all citizens. This is a full pass every tick, even though needs only change in `phase_life` (which runs earlier in the same tick). The result is immediately consumed and never cached.
- **Allocations:** `subsist_p`, `basic_p`, `comfort_p` accumulate in local `i64` variables, but the pressure computation uses a closure (`unmet`) called for every need field of every citizen — 5 fields × n citizens = 5n closure calls per tick.
- **Dirty-flag opportunity:**
  - `tiered_demand` should be computed **once** in `phase_life` immediately after needs are ticked, and stored on `Simulation`. `phase_economy` then reads the cached value. Since needs only mutate in `phase_life`, this is safe across the tick.
  - The pressure closure should be inlined or replaced with a small helper function to avoid per-call closure overhead.

### 1.7 `phase_production` — Linear building scan (acceptable, but note)

**Location:** `crates/engine/src/engine.rs:1733-1757`

- This is a simple O(b) iteration over buildings. Not a hotspot at current scale, but if building count grows to 10,000+ this will matter. The `Fixed` arithmetic (`food += Fixed::from_num(1)`) is currently cheap.

### 1.8 `phase_planet` — Weather grid rebuilt every tick

**Location:** `crates/engine/src/engine.rs:1299-1307`

- **Pattern:** `compute_weather(&self.climate, self.state.tick, self.weather_grid.len().max(1) as u32)` rebuilds the entire `weather_grid` Vec from scratch every tick.
- **Allocations:** `self.weather_grid` is fully replaced (a new `Vec<WeatherCell>` is allocated and the old one is dropped).
- **Dirty-flag opportunity:**
  - `weather_grid` only needs to change when `climate` changes meaningfully (temperature, precipitation). The grid is deterministic from `tick` + `planet` parameters. If the grid size is fixed for the scenario, the same grid could be incrementally updated or cached with a tick key. However, because `compute_weather` is deterministic and cheap for a 16-cell grid, this is only a hotspot if the grid size increases (e.g. to 64×64 or 256×256 for regional weather).

---

## 2. Bevy Voxel Renderer — `clients/bevy-ref/src/voxel_sim.rs`

### 2.1 `step_and_remesh` — Full-grid CA step + per-chunk mesh rebuild

**Location:** `clients/bevy-ref/src/voxel_sim.rs:631-702`

- **Pattern:** `step(&mut state.grid, MaterialRegistry::standard())` performs a full-grid cellular-automata sweep. At 256³ this is ~16M cell updates. The code throttles to 0.25 Hz to avoid frame drops, but the fundamental cost is a full-grid pass.
- **Pattern:** After a CA step, `changed_chunks` is computed from `dirty_chunks`, then every changed chunk is **despawned and fully remeshed** via `spawn_chunk_meshes`. Even if only 1 voxel changed, the entire 32³ chunk is sliced, padded, meshed, and re-spawned.
- **Allocations:** `changed_chunks` HashSet is allocated fresh. `stale` Vec is allocated. `spawn_chunk_meshes` internally allocates many Vecs.
- **Dirty-flag opportunity:**
  - The CA step itself should operate on **dirty chunks only** (the `dirty_chunks` HashSet already exists in `CaGrid`). A dirty-chunk CA step would restrict the update to chunks that contain mobile cells (water, lava, etc.), reducing the sweep from the whole grid to O(dirty_chunks) per tick.
  - Mesh rebuild should use a **dirty-voxel mask** within the chunk: only re-slice and re-mesh if the chunk's dirty voxels intersect the surface (affect the mesh silhouette). Interior voxels that change material but do not create/destroy surface faces should not trigger remesh.

### 2.2 `spawn_chunk_meshes` — Per-chunk triple-copy + per-frame full-grid iteration

**Location:** `clients/bevy-ref/src/voxel_sim.rs:892-987`

- **Pattern:** For every chunk in the grid (triple nested loop `cz/cy/cx`), the function:
  1. Calls `slice_chunk` — copies 32³ = 32,768 voxels into a stack array
  2. Calls `slice_chunk_with_apron` — copies ~36³ = ~46,656 voxels into a padded stack array
  3. Calls `chunk_saturation_with_apron` — copies another ~46,656 bytes
  4. Calls `build_smooth_meshes` or `CubicMesher::mesh_cubic`
  5. Calls `split_by_material` — allocates a `BTreeMap<MaterialId, MeshBuffer>` and re-indexes every triangle
- **Impact:** For a 96×64×96 world (18 chunks wide × 2 high × 18 deep = 648 chunks), this is **~648 × (32k + 47k + 47k) ≈ 82 MB of memory copied per remesh frame**, even when only a handful of chunks actually changed. The function is called with a `filter: Option<&HashSet<ChunkId>>`, but the triple loop still iterates over **all** chunks to test the filter.
- **Allocations:** `slice_chunk` returns a 32,768-element array by value. `slice_chunk_with_apron` returns a ~46,656-element array by value. `chunk_saturation` returns a `Vec<u8>` with `with_capacity(32,768)` but the capacity is often wasted when chunks are empty or all-water. `split_by_material` allocates a fresh `BTreeMap` and fresh `MeshBuffer` per material group.
- **Dirty-flag opportunity:**
  - The `filter` branch should **skip the outer loop entirely** when `filter` is present: iterate the `HashSet` directly rather than scanning all chunk coordinates. This changes O(total_chunks) to O(dirty_chunks).
  - `slice_chunk` and `slice_chunk_with_apron` should be replaced with a **borrowed view** into the grid when the grid is dense and contiguous. The current dense array layout (`CaGrid.cells: Vec<MaterialId>`) supports linear indexing; a `ChunkView` that indexes directly into the grid without copying would eliminate the copy entirely. This requires the mesher to accept a getter closure instead of a fixed-size array.
  - `split_by_material` should reuse a pre-allocated `BTreeMap` or `Vec` (cleared each chunk) rather than allocating a new one per chunk.

### 2.3 `dispatch_chunk_mesh_tasks` — Double-copy on async dispatch

**Location:** `clients/bevy-ref/src/voxel_sim.rs:1221-1240`

- **Pattern:** For every chunk, `chunk_mesh_input` calls both `slice_chunk` and `slice_chunk_with_apron` on the main thread, then the resulting `ChunkMeshInput` (which itself contains two large arrays) is moved into an async task. The task then calls `compute_chunk_mesh`, which may call `build_smooth_meshes`.
- **Impact:** The main thread does the full copy before the async task even starts. At 648 chunks, the main thread copies ~648 × 80k = ~52 MB just to stage data for the task pool. The async task then re-reads the same data.
- **Dirty-flag opportunity:**
  - The chunk data should be **shared via an `Arc<CaGrid>` or `RwLock<CaGrid>`** so the async task can read directly from the grid without copying. The task only needs the chunk coordinates (cx, cy, cz) and a shared grid reference; the mesher can then slice on the worker thread, or better yet, index into the grid without copying.
  - Alternatively, dispatch a `ChunkId` + grid reference, and let the worker thread do the slicing. This keeps the main thread cost to O(chunks) metadata only.

### 2.4 `log_mesher_diagnostic` — Diagnostic-only full-grid remesh

**Location:** `clients/bevy-ref/src/voxel_sim.rs:594-628`

- **Pattern:** On every world build, this function walks every chunk, slices it, pads it, meshes it, and counts off-grid vertices — all to log a single diagnostic line.
- **Impact:** This is a **complete extra remesh pass** on the main thread during world load. For 648 chunks, this adds ~50-100ms of pure diagnostic overhead to the already expensive initial load.
- **Dirty-flag opportunity:**
  - Gate this behind a `debug_assertions` or `CIVIS_MESHER_DIAG=1` env flag. It should never run in production builds.
  - If kept, sample **one** chunk deterministically (e.g. the first non-empty chunk) rather than scanning all chunks.

### 2.5 `chunk_has_exposed_face` — 6-neighbor scan per voxel

**Location:** `clients/bevy-ref/src/voxel_sim.rs:993-1036`

- **Pattern:** For every voxel in a 32³ chunk, checks all 6 neighbors. This is 6 × 32,768 = ~196k boundary checks per chunk.
- **Impact:** Currently only used in the disabled cubic fallback branch (`if false`), but if re-enabled, it would add ~196k × dirty_chunks checks per remesh frame.
- **Dirty-flag opportunity:**
  - This function is only relevant for the cubic fallback path. If the cubic path stays disabled, delete it to reduce code size and avoid accidental re-enable.
  - If re-enabled, replace with a surface-voxel mask computed during the mesher: the mesher already knows which voxels produce faces; reuse that mask.

### 2.6 `build_voxel_world` — Full-grid census on load

**Location:** `clients/bevy-ref/src/voxel_sim.rs:444-538`

- **Pattern:** After worldgen, a triple nested loop (`z/y/x`) iterates every cell in the grid to count non-air voxels and find `max_solid_y`.
- **Impact:** For a 256³ world, this is 16.7M cell reads. This is O(n³) and runs on the main thread during load.
- **Dirty-flag opportunity:**
  - `worldgen::generate` already produces the grid; the generator can export `non_air_count` and `max_solid_y` as metadata, eliminating the census loop.
  - If the generator cannot be modified, the census loop can be fused with `mark_mobile_chunks` to avoid a second pass.

---

## 3. Summary Table

| File | Location | Pattern | Severity | Dirty-flag / Fix |
|------|----------|---------|----------|------------------|
| `engine.rs` | `phase_diplomacy:1940-2141` | O(f²) pairs + O(n) nested scan per pair | **High** at scale | Cache trade-vol map; reuse cluster membership from `phase_life` |
| `engine.rs` | `phase_tactics:1391-1463` | O(d × p) pulse dedup + O(e²) engagement loops | **High** in burst ticks | Quantized `HashSet` dedup; skip doctrine when engagements unchanged |
| `engine.rs` | `phase_life:1551-1730` | 6 full scans + O(n²) clustering | **Critical** | Incremental `PoiRegistry`, `id_to_entity`, `cluster_stocks`; cache clusters |
| `engine.rs` | `phase_diffusion:1515-1540` | 6 full-population scans | **Medium** | Maintain spawn/despawn counters; compute fraction once |
| `engine.rs` | `phase_military:1824-1929` | O(m × u) nested unit queries | **High** | `HashMap<Entity, &mut MilitaryUnit>` or direct entity lookup |
| `engine.rs` | `tiered_demand:2168-2232` | Full needs scan every tick | **Medium** | Compute in `phase_life`, cache on `Simulation` |
| `engine.rs` | `phase_planet:1299-1307` | Weather grid rebuilt every tick | **Low** (16 cells) | Only rebuild if grid size changes; cache keyed by tick |
| `voxel_sim.rs` | `step_and_remesh:631-702` | Full-grid CA step + full chunk remesh | **Critical** | Dirty-chunk CA only; surface-intersection test before remesh |
| `voxel_sim.rs` | `spawn_chunk_meshes:892-987` | Triple-copy per chunk + full-grid loop | **Critical** | Iterate `filter` directly; borrow grid instead of copy |
| `voxel_sim.rs` | `dispatch_chunk_mesh_tasks:1221-1240` | Double-copy to async tasks | **High** | Dispatch `ChunkId` + shared grid; slice on worker |
| `voxel_sim.rs` | `log_mesher_diagnostic:594-628` | Extra full-grid remesh for logging | **Medium** | Gate behind env flag; sample one chunk |
| `voxel_sim.rs` | `build_voxel_world:485-496` | O(n³) census after load | **Low** | Export stats from `worldgen::generate` |

---

## 4. Recommended Priority Order

1. **Fix `phase_life` incremental caching** — removes the biggest engine-side cost and enables scaling past 1,000 agents.
2. **Fix `spawn_chunk_meshes` filter iteration + copy** — changes the Bevy remesh from O(total_chunks) to O(dirty_chunks), and removes the per-chunk memory copy.
3. **Dirty-chunk CA step** — restrict `step()` to `dirty_chunks` so the CA throttle can be lifted from 0.25 Hz back to 2-12 Hz.
4. **Fix `phase_military` nested queries** — direct entity lookup instead of world scan per movement pulse.
5. **Fix `phase_diplomacy` nested scans** — reuse cached cluster sets and trade-route volumes.
6. **Fix `phase_diffusion` counter maintenance** — maintain population counters on spawn/despawn.
7. **Gate / remove `log_mesher_diagnostic`** — immediate load-time win.
