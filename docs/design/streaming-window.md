# Streaming Window — Architecture for the Unbounded World

**Status:** Design (Wave 1 slice)
**Date:** 2026-06-10
**Scope:** Civis 3D extension (`crates/voxel`, `crates/engine`, `clients/bevy-ref`)
**Requirements addressed:** `FR-CIV-SCALE-001..008` (HW-bounded streaming)

---

## 1. Context

Civis' volumetric world is no longer a small diorama. The MVP target is a
**256³ resident** working set (≈ 0.5 mi²); the final target imposes **no fixed
cap on world extent** — only the hardware working-set budget and disk bound
how much of the world is materialised. Memory and tick time MUST scale with
the **active working set** (what the camera + sim can interact with this
moment), not the total world volume.

Main already provides:

- 4-tier fixed world sizes (`WORLD_DIMS_SMALL..WORLD_DIMS_HUGE`) in
  `clients/bevy-ref/src/voxel_sim.rs:148-151` — a stopgap for small builds.
- A chunk-streaming layer (`crates/voxel/src/stream.rs`) with:
  - seeded per-chunk regen, dirty-chunk disk cache, LRU active set
    ([`StreamingWorld`](crates/voxel/src/stream.rs:156));
  - LOD-by-distance via the kernel's `select_lod`
    ([`lod_for`](crates/voxel/src/stream.rs:243));
  - back-face / pre-frustum skip + camera-anchored page-in
    (`clients/bevy-ref/src/voxel_stream.rs`).
- `DirtyChunkEvent` (kernel) for deterministic mesh rebuild.

What's missing for "no fixed cap":

1. A **named policy** that decides which chunks belong to the working set, in
   what state, with what sim fidelity — not ad-hoc LRU.
2. **Rings** (concentric AABBs) so a chunk's *role* (mesh now, sim every tick,
   sim every 4th tick, frozen, evicting) is a first-class concept derived
   from ring distance, not a scattered `if radius < 4` check.
3. **Sim-LOD cohorts** so far chunks don't run a per-voxel CA tick (mass
   conservation already forces this) and the sim cost grows with the **hot
   cohort**, not the resident set.
4. **Horizon-fade seams** between adjacent LOD rings so popping is hidden in a
   blend band, not on a hard boundary.
5. **Prefetch** driven by camera + sim-interest velocity so the inner ring
   warms up before the camera arrives.

This doc locks down the architecture and the chosen approach; the first
slice of code lives in `crates/voxel/src/window.rs` (see §6).

---

## 2. Goals & non-goals

### Goals

- **Working-set bounded.** RAM residency, mesh cost, and sim cost are a
  function of `(active_ring_radius, vband, sim_lod_radius, sim_lod_step)`.
  Adding world extent does not change the budget.
- **Deterministic.** Two clients with the same seed and the same
  `(camera, policy)` see the same ring assignment, the same `ChunkState`
  transitions, and the same eviction order.
- **Replay-safe.** Ring transitions are functions of `(camera_pos, tick)`,
  not of side-effectful load order, so a `.civreplay` can reconstruct the
  working set bit-identically.
- **Pluggable eviction.** The default is ring-distance; the policy is a
  trait so LRU / cost-weighted / age-bounded variants can replace it
  without touching the streaming layer.

### Non-goals (this slice)

- The actual mesh-blend shader (horizon-fade alpha bands): a future
  renderer pass reads `RingIndex` per chunk; the seam policy is locked
  here, the shader is not.
- Multi-camera / split-screen / portal cameras: the window is anchored on
  one anchor (`WindowPolicy::anchor`). Multi-anchor is `policy.anchors()`
  in a later slice.
- Save-format changes (`FR-CIV-SCALE-007`): the existing `ChunkStore`
  (regen-on-demand for clean chunks, disk-persist for dirty) already
  satisfies the streaming side of scale persistence; save-format
  materialisation is a follow-up.
- Mod / sim-dev fast-path shortcuts: those are layered on top, not
  rewritten.

---

## 3. Architecture

### 3.1 The window

A **window** is a camera-anchored AABB with concentric **rings**:

```text
                    ring 4 (evict candidates)    —  sim: FROZEN
                 ┌────────────────────────────┐
                /  ring 3 (coarse sim)        /   —  sim: COARSE_SIM
               /  ┌──────────────────────┐   /
              /  /  ring 2 (full mesh)   /  /
             /  /  ┌──────────────┐     / /
            /  /  │ ring 1 hot   │     / /   —  sim: FULL_SIM
           /  /  │  (camera)     │    / /
          /  /  └──────────────┘    / /
         /  /  ┌──────────────────┐/ /       —  render: RING 2 only,
        /  /  │ ring 0 (anchor)  │/ /           but with HORIZON_FADE
       /  /  │  cell + halo     │/ /             blend over ring 2-3
      /  /  └──────────────────┘/ /
     /  └────────────────────────┘/
    └────────────────────────────┘
```

The **anchor** is the cell containing the camera (a single chunk in MVP).
Each chunk is classified by its **ring distance** — the Chebyshev distance
in chunk units from the anchor. Vertical distance is folded into the
classification (see §3.4).

### 3.2 Chunk lifecycle (`ChunkState`)

Every chunk in the world is in exactly one of:

| State            | Meaning                                                         |
|------------------|-----------------------------------------------------------------|
| `Unloaded`       | Not in `resident`; will regen from seed if requested.           |
| `Resident`       | In `resident` (warm) but not yet meshed.                        |
| `Meshed`         | In `resident` and a mesh is alive (Bevy entity spawned).        |
| `Fading`         | Mesh is alive but alpha is being lowered for a ring shrink.     |
| `Evicting`       | Marked for eviction; mesh despawn scheduled.                    |
| `Evicted`        | Removed from `resident`; persisted to disk if dirty.            |

Transitions are driven by `WindowPolicy::classify(coord, anchor, tick)` and
the streaming layer's load/evict path. The `Fading` state exists so a ring
shrink doesn't pop meshes — the chunk stays `Resident`+meshed for one extra
`fade_ticks` while its alpha ramps from 1.0 to 0.0 (the actual ramp is the
renderer's job; the policy just owns the time window).

### 3.3 Sim-LOD cohorts

`RingIndex` maps to a **sim cohort**:

| Ring (dist)  | Sim cohort    | Tick behaviour                                  |
|--------------|---------------|-------------------------------------------------|
| 0 (anchor)   | `FullSim`     | Every tick, per-voxel CA, full agent tick.       |
| 1            | `FullSim`     | Every tick, per-voxel CA, full agent tick.       |
| 2            | `CoarseSim`   | Every Nth tick, statistical gestalt only.        |
| 3            | `CoarseSim`   | Every 4×Nth tick.                                |
| ≥ 4          | `Frozen`      | No sim tick; mass is conserved trivially.        |

N is `WindowPolicy::sim_lod_step`. The cohorts are **derived from ring
distance**, not stored, so changing the policy recomputes the cohort on
the next `classify()` call.

This satisfies `FR-CIV-SCALE-004` (gestalt without state divergence): the
cohort multiplier is a *tick rate* change, not a state-fidelity drop —
frozen chunks conserve mass by construction (no writes, no decay), and
coarse-sim chunks use the same diffusion / agent state vectors, just
aggregated.

### 3.4 Ring distance (Chebyshev, vertical weighted)

`ring_distance(coord, anchor) = max(|Δx|, |Δy|*vy_w, |Δz|)`

with `vy_w` (default 2) so a single step up/down "costs" two horizontal
steps. Worlds are mostly flat heightfields — the vertical weight reflects
that fact (most action is on the surface) and keeps the inner ring from
filling the entire Y axis. The function is `pure` and `const`-callable so
it can be evaluated from `const fn`s in render code and from policy
classifiers.

### 3.5 Prefetch policy

The streaming layer's `desired_set` becomes:

```text
want(rng, prefetch_ring) =
  for each chunk in ring ≤ rng:
      include;
  for each chunk in (rng, prefetch_ring] that lies in the
  camera's forward half-cone (or sim-interest direction):
      include with priority "prefetch";
```

`prefetch_ring` defaults to `rng + 1`. The forward cone uses
`dot(forward, to_chunk) > 0` (cheap backface pre-pass; the same one
already in `clients/bevy-ref/src/voxel_stream.rs:215`). The **policy** is
where the cone half-angle lives (`forward_cone_cos_theta`, default 0.0 =
hemisphere), and **sim-interest** is an optional second anchor (e.g. a
selected agent) that gets its own prefetch cone, unioned with the camera
cone.

### 3.6 Eviction policy (default = ring-distance, not LRU)

**Decision:** the default eviction policy is **ring-distance**, not LRU.

The existing `StreamingWorld` evicts via LRU (oldest touch first). For the
unbounded window, LRU and ring-distance often agree (the camera's
movement makes LRU order track ring order), but they diverge in two cases
the user actually sees:

1. **Idle frames.** The camera is still; LRU churns the inner ring
   around (a chunk touched in frame N is "hot" relative to one touched
   in frame N-2). Ring-distance does not churn.
2. **Long pause + sudden jump.** LRU keeps a far-flung chunk alive
   (recently touched) over a close chunk the camera just arrived at.
   Ring-distance evicts the far one.

Ring-distance eviction is the default. The `EvictionPolicy` trait lets
LRU win where the working set is constrained (e.g. the existing small
diorama MVP) — LRU is a *policy option*, not the default. LRU is still
available as a tie-breaker inside a ring (see §5).

### 3.7 Horizon-fade seams

Two adjacent rings must not pop. The `WindowPolicy` owns the seam
width — `seam_chunks: u8`, default 1 — meaning the last chunk of the
inner ring is in **seam** mode: it renders at the **next ring's** LOD
and a blend weight `< 1.0`. The next-ring-out chunk is the **seam
target** (its mesh is the fallback if the seam mesh hasn't built yet).
Concretely:

- ring `r` chunks at distance `r * step - seam_chunks ≤ d < r * step`
  are `Seam(r)`: they mesh at LOD for distance `r+1`, with alpha
  `(r*step - d) / seam_chunks`.
- The seam exists only in **render-LOD space**, not sim-space. Sim
  cohort is still `classify(ring)`. Sim gets the same fidelity; render
  gets the cross-fade.

This satisfies `FR-CIV-SCALE-003` ("LOD rings with horizon-fade seams
between chunk resolutions").

---

## 4. Alternatives considered

### 4.1 Fixed-grid tiers (`WORLD_DIMS_SMALL..HUGE`)

**What it is:** the current MVP. A combo box picks a tier; the world is
fully resident at that tier. No streaming, no horizon fade, no sim LOD.

**Why it fails the final target:** the user picks a tier once and is
stuck — extending the world requires a save/load boundary. The
`Perf` HUD lies at tier 3 (`384³` ≈ 56k chunks ≈ 22 GB dense).

**Where it stays:** tier 0/1 (`SMALL`, `MEDIUM`) remain the
zero-config dev fallback. The window policy defaults are tuned so
`active_budget = SMALL * CHUNK_EDGE³` matches the SMALL tier's working
set, so dev loops don't change.

### 4.2 Clipmap rings (a la `block-mesh-rs`, `building-blocks`)

**What it is:** N concentric horizontal rings with a fixed Y extent
(the camera's height band). Far rings are coarser LOD voxel grids (2×,
4×) rather than chunks. Meshing is per-ring-grid, not per-chunk.

**Pros:** one mesh per ring (small draw call count); well-known
horizon fade.

**Why we don't pick it:**

- **Grid-relative, not chunk-relative.** A clipmap stores voxels at
  multiples of the base voxel size per ring. Our `phenotype-voxel`
  kernel is **chunk-keyed** (deterministic `DirtyChunkEvent` per
  `(chunk_id, write_seq)`). Clipmap would force a per-ring re-tile on
  the kernel side, breaking the dirty-queue contract.
- **Asymmetric scaling.** A clipmap is square in XZ; our worlds are
  not (vertical extent is much smaller). The seam handling for
  non-square clipmap is fiddly.
- **Edit path.** A 1-voxel user edit in a far ring would have to
  re-mesh an entire 4× grid cell. Chunk rings edit at the chunk
  granularity that already drives `DirtyChunkEvent`.

We **borrow** the clipmap concept of "ring index drives LOD step" and
"seam blend in the last chunks of a ring" — that part is canonical.
We reject the per-ring re-tile.

### 4.3 Pure octree cut (SVO root split)

**What it is:** the kernel's SVO is split on-camera so the active
octree is camera-anchored. Far octree branches are unloaded as a unit
(an octree node = N³ chunks).

**Why we don't pick it:**

- The kernel already keeps a dense 16³ leaf; an octree-branch
  eviction would unceremoniously swap the leaf representation.
- Octree branches don't align with mesh-dirty events. A branch cut
  is a structural operation; a `DirtyChunkEvent` is at leaf
  granularity.
- "Branch radius" is hard to map to "user-visible ring distance"
  (octree depth is uneven for heightfield worlds — many leaves are
  near the surface, few far from it).

We **keep** the SVO as the substrate; the window policy is purely an
**access pattern** on top of the SVO + dense leaves, not a
restructuring of it.

### 4.4 Eviction: LRU vs ring-distance vs cost-weighted

| Policy          | Pros                                       | Cons                                          |
|-----------------|--------------------------------------------|-----------------------------------------------|
| **LRU**         | Simple; works for stationary camera.       | Churns on idle frames; bad for sudden jumps.  |
| **Ring-distance** (chosen) | Stable on idle; matches user intent. | Can pin a far chunk that was once visited.    |
| **Cost-weighted** | Evicts the largest mesh first under pressure. | Not user-visible; surprise evictions.         |

Ring-distance wins on **determinism** (replay produces the same eviction
order without needing a touch counter) and on **predictability** (a
user who understands "rings" can reason about what stays). LRU is kept
as a tie-breaker for chunks at the *same* ring distance (the order is
then `(ring, lru_pos)`).

---

## 5. Data shapes (first slice)

```rust
// crates/voxel/src/window.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    Unloaded,
    Resident,
    Meshed,
    Fading { ticks_remaining: u8 },
    Evicting,
    Evicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimCohort {
    FullSim,
    CoarseSim { step_multiplier: u8 },
    Frozen,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowPolicy {
    pub mesh_ring: u8,           // inner ring fully meshed
    pub sim_ring:  u8,           // inner ring on full sim cadence
    pub coarse_ring: u8,         // ring 0..coarse_ring on coarse sim
    pub seam_chunks: u8,         // see §3.7
    pub vy_weight: u8,           // see §3.4
    pub sim_lod_step: u8,        // see §3.3
    pub prefetch_ring: u8,       // see §3.5
    pub forward_cone_cos_theta: i8, // Q0.7 dot threshold
}

#[must_use]
pub const fn ring_distance(
    coord: ChunkCoord,
    anchor: ChunkCoord,
    vy_weight: u8,
) -> u32 {
    let dx = (coord.cx - anchor.cx).unsigned_abs();
    let dz = (coord.cz - anchor.cz).unsigned_abs();
    let dy = (coord.cy - anchor.cy).unsigned_abs() * (vy_weight as u32).max(1);
    dx.max(dz).max(dy)
}

impl WindowPolicy {
    #[must_use]
    pub const fn classify(&self, coord: ChunkCoord, anchor: ChunkCoord) -> ChunkState {
        // ring N→(N+1) is the seam band; classify as Meshed but with a
        // seam-aware LOD hook (renderer reads `RingIndex`).
        let ring = ring_distance(coord, anchor, self.vy_weight) as u8;
        if ring <= self.mesh_ring { ChunkState::Meshed }
        else { ChunkState::Unloaded } // etc.
    }

    #[must_use]
    pub const fn sim_cohort(&self, coord: ChunkCoord, anchor: ChunkCoord) -> SimCohort {
        let ring = ring_distance(coord, anchor, self.vy_weight) as u8;
        if ring <= self.sim_ring { SimCohort::FullSim }
        else if ring <= self.coarse_ring { SimCohort::CoarseSim { step_multiplier: 2 } }
        else { SimCohort::Frozen }
    }
}
```

The full type is in `crates/voxel/src/window.rs`. The slice lands a
**ring-distance function** + **chunk-state enum** + **WindowPolicy
config + classify() + sim_cohort()** + **eviction comparator** —
all behind `pub mod window;` (no feature flag — the module is small,
pure, and testable in isolation, so it always compiles). Wiring it
into `StreamingWorld` and `voxel_stream.rs` is a follow-up slice
(FR-CIV-SCALE-001 / 002 acceptance).

---

## 6. Cross-references

- `FUNCTIONAL_REQUIREMENTS.md` § FR-CIV-SCALE-001..008
- `docs/adr/ADR-005-adaptive-voxel.md` (kernel guarantees)
- `docs/adr/ADR-voxel-streaming-scale.md` (scale target is a first-class
  architectural concern; this design is its implementation strategy)
- `crates/voxel/src/stream.rs` (current LRU streaming — WindowPolicy
  layers on top)
- `crates/voxel/src/lod.rs` (per-chunk `ChunkRenderPlan` — WindowPolicy
  feeds the `LodLevel` field)
- `clients/bevy-ref/src/voxel_stream.rs` (camera-anchored `desired_set`
  — WindowPolicy replaces the inline `STREAM_RADIUS` constant)
- `clients/bevy-ref/src/voxel_sim.rs:148-151` (`WORLD_DIMS_*` — kept
  for tier 0/1 dev fallback; the window policy defaults match SMALL)

---

## 7. Open questions (follow-up slices)

1. **Multi-anchor** (split-screen, cinematic cameras, sim-interest as a
   second anchor): add `WindowPolicy::anchors() -> &[Anchor]`,
   take the union of per-anchor rings.
2. **Eviction hysteresis** — a `keep_ring` (`< mesh_ring`) so a chunk
   that drops out of the mesh ring is held in `Fading` for one extra
   cycle. The first slice implements this as a constant; the next
   slice makes it a `WindowPolicy` field.
3. **Sim cohort step multiplier per ring** — first slice uses a single
   `sim_lod_step`; a real-world horizon probably wants
   `step * 2^(ring - coarse_ring)`.
4. **Disk prefetch warm** — when a chunk on the prefetch ring is
   resident on disk from a prior session, prioritize its load (it's
   near-zero cost vs regen). Deferred.
5. **Clipmap-style 2× LOD grid in the seam** — first slice uses the
   next ring's `LodLevel`; the renderer can later replace it with a
   2× downsampled mesh from the parent ring. Deferred.
