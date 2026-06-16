# Civis Engine — Per-Tick Hot Path Audit

**Scope:** `crates/engine/src/engine.rs` tick loop (`tick_with_emergence_source`) and delegated phase bodies (`engine.rs`, `emergence.rs`, `disasters.rs`, `emergence_metrics.rs`, `civ-agents`, `civ-tactics`, `civ-planet`).

**Method:** Static read-only review (no `cargo` profiling). Costs are expressed as asymptotic work per tick at default scale (~128 civilians = 4 factions × 32, 16-region weather grid) and how they scale.

**Tick order (27 phases + optional emergence sample every 50 ticks):**

```
production → citizen_lifecycle → military → economy → planet → diplomacy → tactics →
voxel → compact → buildings → diffusion → disasters → life → settlement_consumption →
emergence → research → tech → belief → unrest → faction_unrest → cohesion →
social_mood → stratification → institutions → economic_focus → chronicle →
[sample_emergence @ tick % 50]
```

---

## Top 5 Most Expensive Per-Tick Operations

| Rank | Phase | Hot operation | Dominant cost | Optimization lever |
|------|-------|---------------|---------------|-------------------|
| 1 | `life` | `cluster_by_colocation` + multi-pass ECS rebuild | **O(N²)** pairwise distance + **4× O(N)** scans + HashMap/BTreeMap rebuild | Spatial index + dirty clustering |
| 2 | `emergence` | `emergence_psyche` + `agent_entity` | **O(N²)**–**O(N×T)** linear scans per social tie | `agent_id → Entity` cache |
| 3 | `military` | `tick_war_bridge` + `line_of_sight` | **O(U² × L)** unit pairs × Bresenham ray length | Spatial bucketing + LOS cache |
| 4 | `diffusion` | `propagate_cohort_*_with_lod` | **~12× O(N)** redundant full-world queries | Single-pass cohort scan |
| 5 | `citizen_lifecycle` | `attach_citizen_to_agents` | **O(N) clone** of every agent every tick | Spawn/despawn dirty gate |

*N = civilian count, U = military unit count, T = average social ties per agent, L = LOS ray length in voxels.*

---

### 1. `phase_life` — full recomputation of settlements every tick

**Where:** `engine.rs` `phase_life` (steps 2–6): `build_poi_registry`, per-agent needs/pathing loop, `cluster_by_colocation`, `id_to_entity` HashMap, `cluster_stocks` BTreeMap replace.

**Cost:**

- **`build_poi_registry`:** O(B) building query every tick; POI set is stable unless buildings change.
- **Per-agent loop:** O(N) with multiple `world.get`/`get_mut` per entity; **`civ.clone()`** on the pathing path copies `AgentCivilian` even when only `id` is needed.
- **`cluster_by_colocation`:** O(N²) all-pairs distance checks (union-find over sorted agents). At N=128 ≈ 8k pairs; at N=10k ≈ 50M pairs — dominates quickly.
- **Rebuilds:** fresh `Vec<Entity>`, `Vec<(u64, Position3d)>`, `HashMap<u64, Entity>`, and `cluster_stocks = next_stocks` with **`.cloned()`** per cluster every tick regardless of membership stability.

**Optimization (incremental / dirty-flag / cache):**

1. **Clustering:** maintain a uniform-grid or hash spatial index; only re-run connected-components for agents that moved beyond `cluster_radius` or joined/left (dirty set from pathing).
2. **POI registry:** cache `PoiRegistry`; invalidate on building spawn/despawn or `phase_buildings` allocation.
3. **`id_to_entity`:** persistent `HashMap<u64, Entity>` updated on spawn/despawn instead of rebuilt from scratch.
4. **`cluster_stocks`:** update counts in place when cluster membership changes; avoid full BTreeMap clone.
5. **Pathing:** pass `civilian.id` by reference; avoid `civ.clone()` in the hot loop.

---

### 2. `phase_emergence` — linear `agent_entity` scans inside per-agent psyche work

**Where:** `emergence.rs` `phase_emergence` → `emergence_psyche` (primary), plus `emergence_culture` / `emergence_genetics_sentience`.

**Cost:**

- **`agent_entity`:** each call runs `world.query::<&Civilian>().iter().find(|(_, c)| c.id == agent_id)` — **O(N)** per lookup.
- **`emergence_psyche`:** for each agent, iterates social-graph ties and calls `agent_entity(other_id)` for exposure — **O(N × T × N)** worst case.
- **`emergence_genetics_sentience`:** collects all agents with **`d.clone()`** (full `Dna` clone) every tick.
- **`emergence_culture`:** clones all `CultureProfile` values (`values().cloned().collect()`), builds **O(K²)** contact edges for K clusters, re-inserts entire map.
- **`psych_profile.clone()`** at start of psyche phase copies config that is tick-invariant.

**Optimization:**

1. **`HashMap<u64, Entity>` side index** (or `Vec` indexed by dense id if ids are compact): O(1) `agent_entity`; rebuild only on spawn/despawn.
2. **Genetics:** evaluate sentience from `&Dna` without cloning; skip agents already in `sentient_agents`.
3. **Culture:** drift only clusters whose population or neighbor set changed; avoid cloning unchanged profiles.
4. **Store `&PsychProfile` / `Arc<PsychProfile>`** instead of per-tick clone.

---

### 3. `phase_military` — O(U²) war bridge with per-pair voxel LOS

**Where:** `engine.rs` `phase_military` → `civ_tactics::build_fog_for_units`, `tick_war_bridge`, `tick_operational_movement`; nested `query_mut` scans to apply grid moves and HP loss.

**Cost:**

- **`tick_war_bridge`:** nested loop over all unit pairs in engage range; each candidate runs **`line_of_sight`** (3D Bresenham, one `voxel.read` per cell along ray). Complexity **O(U² × L)** on war cadence ticks (`tick % cadence_ticks == 0`).
- **`build_fog_for_units`:** allocates fog grid up to 256² and calls `fog.update(units, world)` — voxel-scoped work every military tick when fog is enabled.
- **`engagements.clone()`** duplicates engagement vec after bridge.
- **Movement apply:** for each grid move, scans **all** `MilitaryUnit` entities via `query_mut` to find one match — O(moves × U).
- **`damaged_targets.contains(&j)`** inside shooter loop is O(U) per inner iteration (linear vec search).

**Optimization:**

1. **Spatial hash** on grid coords: only consider targets within `engage_range_grid` Manhattan ball.
2. **LOS cache** keyed by `(shooter_cell, target_cell)` for the tick (invalidated on voxel writes).
3. **Entity index:** `Vec<Entity>` already built — use direct index for HP/move updates; drop inner `query_mut` scans.
4. **`HashSet` for `damaged_targets`** instead of `Vec::contains`.
5. **Fog:** incremental update from unit delta positions rather than full `fog.update` over world.

---

### 4. `phase_diffusion` — six full-world wardrobe/tools scans per tick

**Where:** `engine.rs` `phase_diffusion` → `propagate_cohort_wardrobe_with_lod` + `propagate_cohort_tools_with_lod` (each: `count_civilians` + two filter-count queries + mutation pass + recount).

**Cost:**

- Per cohort pass: **`count_civilians`** (full `Civilian` query), **two** `query::<&Wardrobe/Tools>().iter().filter(...).count()`, one `query_mut` promotion loop, then **another** count query.
- Wardrobe + tools ⇒ **~6 full-world iterations** over overlapping component sets every tick, even when `current_fraction` changes by at most a handful of promotions.
- LOD (`should_tick_entity_with_policy`) reduces mutations but **not** the counting passes.

**Optimization:**

1. **Single fused pass:** one `query_mut::<(&mut Wardrobe, &LodTier)>` that counts `at_target` and promotes in one walk; same for tools.
2. **Cached `CohortStats`:** recompute `current_fraction` only when a promotion occurred or on spawn/despawn (dirty flag).
3. **Share `total_civilians`** between wardrobe and tools (one count per tick).

---

### 5. `phase_citizen_lifecycle` — redundant `attach_citizen_to_agents` clone sweep

**Where:** `engine.rs` `phase_citizen_lifecycle` line 1 calls `attach_citizen_to_agents`; `engine.rs` helper clones every `AgentCivilian` into a `Vec` before checking for existing `Citizen` component.

**Cost:**

- **Every tick:** `world.query::<&AgentCivilian>().iter().map(|...| civilian.clone()).collect()` — O(N) allocations and copies even when all agents were attached ticks ago.
- Followed by **`query_mut` over all `(AgentCivilian, Position3d, Needs)`** for aging, food, birth/death — necessary O(N) but compounded with attach overhead.

**Optimization:**

1. **Dirty gate:** call `attach_citizen_to_agents` only after spawns (`spawn_child_near`, scenario load) — track `agents_needing_citizen_attach: HashSet<Entity>`.
2. **In attach:** iterate without clone; use `world.get::<&Citizen>(entity).is_err()` inline.
3. **Birth window:** `tick % 200` already limits births; ensure attach runs on birth list only, not whole world.

---

## Honorable Mentions (lower steady-state cost or amortized)

| Item | Phase | Note |
|------|-------|------|
| `compute_weather` full `Vec` rebuild | `planet` | O(R), R=16 default — cheap; cache cells whose inputs unchanged if R grows. |
| `phase_disasters` weather scan | `disasters` | O(R) over 16 cells — negligible vs agent work. |
| `sample_from_voxel_world` | post-`chronicle` (every 50 ticks) | Full dense-chunk voxel histogram + optional 4096-cell clone — **spike**, not steady per-tick. |
| `voxel.compact()` | `compact` | Every 64 ticks — compaction cost amortized; watch if chunk count is large. |
| `phase_voxel_ca` 16³ per dirty chunk | external CA path | Full-grid scan when Bevy CA grid attached; not in default `tick()` without caller grid. |
| `emergence_civ_ai` / `civ_ai_sync_generate` | `emergence` | Sync LLM call when legend/sentience feed events exist — unbounded latency, not CPU-bound. |
| `mod_host.tick` / `economy_tick` / `military_tick` | `economy`, `military` | WASM mod execution — environment-dependent; not analyzed here. |

---

## Suggested Profiling Order (when `cargo` is allowed)

1. `phase_life` — flamegraph around `cluster_by_colocation` and POI build.
2. `phase_emergence` — count `agent_entity` invocations per tick.
3. `phase_military` on war cadence — `line_of_sight` + `tick_war_bridge`.
4. `phase_diffusion` — ECS query count per tick.
5. `attach_citizen_to_agents` — allocation rate in `citizen_lifecycle`.

Instrument with `tracing` spans per phase (already partially present for mod phases) or `#[cfg(feature = "perf")]` counters for query iterations.

---

## Cross-Phase Dependency Notes

- **`life` → `emergence`:** clustering in `phase_life` feeds `ClusterMember` used by `emergence_social` / `emergence_psyche` — incremental clustering benefits both.
- **`life` → `settlement_consumption`:** recomputes `cluster_sizes` from `ClusterMember` again — share cached sizes from `phase_life`.
- **`planet` → `disasters`:** weather grid is small today; if region count scales with map size, pair incremental weather with incremental disaster threshold scan (only cells whose temp/precip changed).

---

*Generated by static audit of `engine.rs` phase decomposition. No source changes, no `cargo` runs.*
