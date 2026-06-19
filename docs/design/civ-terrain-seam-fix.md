# Terrain LOD-Seam Fragmentation — Root Cause + Fix Plan

**Status:** Design
**Date:** 2026-06-15
**Issue:** #98
**Scope:** `clients/bevy-ref/src/voxel_sim.rs`, `clients/bevy-ref/src/voxel_smooth_mesher.rs`,
`clients/bevy-ref/src/voxel_stream.rs`, `crates/voxel/src/window/mod.rs`
**Requirements addressed:** `FR-CIV-VOXEL-010` (watertight terrain), `NFR-SCALE-PERF`
**Complements:** `docs/design/civ-perf-dirty-incremental.md` (dirty-flag remesh),
`docs/design/streaming-window.md` (ring policy + seam band)

---

## 1. Symptom

Players see visible fragmentation at chunk boundaries: gaps, dark seam walls,
and "floating island" fragments where adjacent terrain chunks fail to meet.
The bug manifests in two distinct code paths:

1. **Dense `voxel_sim` path** — smooth mesher chunks that are *mostly* seamless
   but fragment under specific boundary conditions (solid-over-air edge, or after
   CA remesh despawn/respawn).
2. **Streaming `voxel_stream` path** — the cubic mesher produces hard seam walls
   at every chunk boundary, and LOD-level transitions create T-junctions.

---

## 2. Root-Cause Analysis

### RC-1: Cubic mesher has no neighbor apron (PRIMARY — dense path)

The cubic `CubicMesher::mesh_cubic` operates on a single `CHUNK_EDGE³` (32³)
window. It has **no visibility** into neighboring chunks. At a chunk boundary
where the neighbor is solid, the cubic mesher treats the out-of-bounds side as
phantom air and emits boundary faces — creating a visible "wall" of faces
pointing into the interior of the adjacent chunk.

| Evidence | Location |
|----------|----------|
| Cubic mesher called with bare `ChunkView` (no apron) | `voxel_sim.rs:954-961` (`spawn_chunk_meshes` cubic branch) |
| Smooth mesher called with padded `slice_chunk_with_apron` | `voxel_sim.rs:942-951` (`spawn_chunk_meshes` smooth branch) |
| `should_use_smooth_mesh` previously had a distance gate | `voxel_sim.rs:830-844` — old code returned `false` for far chunks, routing them to cubic |
| Test confirms fix: Smooth mode always uses smooth mesher | `voxel_sim.rs:159-169` (`terrain_fragmentation_tests::smooth_mode_always_uses_smooth_mesher_regardless_of_distance`) |
| Regression guard: `SMOOTH_MESH_PADDED_EDGE > CHUNK_EDGE` | `voxel_sim.rs:207-214` (`padded_edge_larger_than_chunk_edge`) |

**Status:** Partially fixed. The `should_use_smooth_mesh` distance gate was removed
(commit for #98), so Smooth mode now always uses the smooth mesher. However, the
cubic fallback still exists in the `spawn_chunk_meshes` dead branch (`voxel_sim.rs:969-981`,
`if false`) and in `compute_chunk_mesh` (`voxel_sim.rs:1178-1181`). The fix is
structural but incomplete — see RC-2 for why smooth-mode chunks can still fragment.

### RC-2: Surface Nets mesh boundary is chunk-local — no shared vertices

The smooth mesher (`voxel_smooth_mesher.rs:163-193`, `build_surface_nets`) runs
Surface Nets over the padded 38³ grid and then **subtracts APRON from positions**
(`:173-176`). The resulting mesh is a standalone `MeshBuffer` positioned at the
chunk origin. Two adjacent chunks each independently extract their isosurface.
At the shared boundary:

- Chunk A places a vertex at position (31.7, 12.3, 5.1) on the boundary face.
- Chunk B places a vertex at position (0.2, 12.5, 5.3) on its opposite boundary face.
- These vertices do **not** share an edge — they are two separate triangles.

The result: a sub-voxel gap (typically 0.1–0.5 units) that reads as a dark line
or crack under lighting.

| Evidence | Location |
|----------|----------|
| Position offset by APRON subtraction | `voxel_smooth_mesher.rs:173-176` |
| Surface Nets extraction is chunk-local | `voxel_smooth_mesher.rs:167` (`surface_net(CHUNK_EDGE_PADDED, &density, true)`) |
| Single-chunk seam test only tests ONE chunk | `voxel_smooth_mesher.rs:564-599` (`seam_chunk_produces_continuous_boundary_surface`) — proves vertices exist near boundary but does NOT test two-chunk continuity |
| No cross-chunk stitching in codebase | Zero matches for "stitch", "skirt", or "seam_verts" in any `.rs` file |

### RC-3: Streaming path uses cubic mesher exclusively — no apron, no smooth

`voxel_stream.rs:246` calls `CubicMesher::mesh_cubic(view, lod)` directly. The
streaming path has **no apron**, **no smooth mesher**, and **no stitching**.
Every chunk boundary produces a hard seam wall.

| Evidence | Location |
|----------|----------|
| Streaming `mesh_chunk` uses cubic mesher only | `voxel_stream.rs:246` |
| No padded apron slice in streaming path | `voxel_stream.rs:241-245` — bare `ChunkView` from `state.world.get(coord)` |
| No smooth mesher import | `voxel_stream.rs:25-28` — imports `CubicMesher` but not `build_smooth_meshes` |

### RC-4: LOD-level transitions produce T-junctions (streaming path)

`select_lod` (`crates/voxel/src/lod.rs:24-30`) assigns different `LodLevel`
values to chunks based on camera distance. At an LOD boundary, chunk A meshes at
LOD 0 (full 16³ resolution) while adjacent chunk B meshes at LOD 1 (8³ resolution).
The LOD 0 mesh has vertices along the shared edge that have no corresponding
vertex on the LOD 1 mesh — a classic T-junction that produces cracks.

| Evidence | Location |
|----------|----------|
| LOD selection by distance | `voxel_stream.rs:194-195` (`select_lod(dist, lod_scale, lod_policy)`) |
| `LodLevel` passed to `mesh_cubic` | `voxel_stream.rs:246` |
| No T-junction stitching in cubic mesher | `CubicMesher` re-exported from `phenotype_voxel` (`crates/voxel/src/lib.rs:24`) — no stitching logic |
| `WindowPolicy.seam_chunks` exists but is not consumed by streaming | `crates/voxel/src/window/mod.rs:138-139,232-252` — the seam band is modeled but `voxel_stream.rs` doesn't use `WindowPolicy` |

### RC-5: Async mesh race creates frame-gap seams (dense path)

`dispatch_chunk_mesh_tasks` (`voxel_sim.rs:1235-1254`) spawns per-chunk async
tasks. `apply_chunk_mesh_tasks` (`voxel_sim.rs:1264-1299`) drains them one at a
time via `block_on(poll_once)`. Tasks complete in arbitrary order — chunk (2,0,1)
may spawn before chunk (1,0,0). During the gap, the player sees chunk A with
open boundary edges and no neighbor mesh.

| Evidence | Location |
|----------|----------|
| Tasks complete in arbitrary order | `voxel_sim.rs:1274-1278` (`poll_once` drains one completed task per frame) |
| Entities spawned immediately on completion | `voxel_sim.rs:1282-1298` — no batching, no "wait for neighbors" gate |
| No generation counter to invalidate stale tasks | `ChunkMeshJob` carries no generation/epoch field (`voxel_sim.rs:1140-1145`) |

### RC-6: No skirts generated by either mesher

Neither the smooth mesher nor the cubic mesher generates "skirt" geometry — vertical
quads extending downward from boundary edges. Skirts are the standard technique for
hiding sub-voxel gaps at chunk boundaries: they fill the visual crack with a
continuation of the surface material, making the seam invisible even when two
chunks don't share exact boundary vertices.

| Evidence | Location |
|----------|----------|
| `build_surface_nets` returns raw SN output | `voxel_smooth_mesher.rs:163-193` — no skirt pass |
| `mesh_cubic_split` returns raw cubic faces | `voxel_sim.rs:1196-1204` — no skirt pass |
| Zero matches for "skirt" in entire codebase | N/A |

---

## 3. Fix Design

### Fix 1: Enable smooth mesher in streaming path (RC-3)

**Change:** `voxel_stream.rs` should use `build_smooth_meshes` instead of
`CubicMesher::mesh_cubic`.

**Implementation:**
- Import `build_smooth_meshes`, `SMOOTH_MESH_PADDED_EDGE`, and the apron slice helpers.
- `mesh_chunk` slices a padded apron from `state.world.get(coord)` (same pattern as
  `voxel_sim::slice_chunk_with_apron`).
- Routes through `should_use_smooth_mesh` (or the streaming analogue) instead of
  always using cubic.
- When smooth mesher returns empty buffers, falls back to cubic for the chunk.

**Files changed:** `clients/bevy-ref/src/voxel_stream.rs`

### Fix 2: Boundary vertex snapping (RC-2)

**Change:** After Surface Nets extraction, snap vertices within ε of chunk boundary
planes to the exact boundary coordinate.

**Implementation:** In `build_surface_nets` (`voxel_smooth_mesher.rs:163-193`),
after APRON offset subtraction, clamp boundary-plane vertices:
```
for each vertex where position[d] is within ε of 0.0 or CHUNK_EDGE as f32:
    position[d] = 0.0 or CHUNK_EDGE as f32  (whichever is closer)
```
This ensures two adjacent chunks produce vertices at the **same** boundary
coordinate, closing the sub-voxel gap. ε = 0.5 (half a voxel) is safe because
Surface Nets interpolation places boundary vertices within ~0.3 voxels of the
grid edge.

**Files changed:** `clients/bevy-ref/src/voxel_smooth_mesher.rs:163-193`

### Fix 3: Skirt generation (RC-6, complements Fix 2)

**Change:** Generate a 1-voxel-height skirt around the mesh boundary.

**Implementation:** After mesh extraction, walk all boundary-edge triangles
(those with one vertex on a chunk boundary plane). Emit duplicate vertices
offset by `-SKIRT_DEPTH` (1.0 world unit downward along the surface normal)
and connect them with new triangles. The skirt fills the visual crack even
when Fix 2's snapping isn't perfect.

**Files changed:** `clients/bevy-ref/src/voxel_smooth_mesher.rs` (new `add_skirts`
pass), `clients/bevy-ref/src/voxel_stream.rs` (same pass for streaming path)

### Fix 4: LOD-boundary stitching via `WindowPolicy.seam_chunks` (RC-4)

**Change:** Wire `WindowPolicy.seam_chunks` into the streaming path so chunks
in the seam band always mesh at the same LOD as their inner neighbor.

**Implementation:**
- `voxel_stream.rs:mesh_chunk` checks `WindowPolicy::classify(coord, anchor)`.
  If the chunk is in the `Fading` (seam) band, override its LOD to match the
  `mesh_ring` LOD (LOD 0).
- This eliminates the T-junction: chunks on both sides of the LOD boundary
  use the same resolution.
- `WindowPolicy.seam_chunks` (default `1`) means one chunk-width of uniform
  LOD at each ring transition.

**Files changed:** `clients/bevy-ref/src/voxel_stream.rs:194-207`

### Fix 5: Async mesh ordering via generation counter (RC-5)

**Change:** Add a generation counter to `ChunkMeshInput`/`ChunkMeshJob` so stale
tasks from a previous CA tick are discarded instead of applied.

**Implementation:**
- Add `generation: u64` to `ChunkMeshInput` (`voxel_sim.rs:1129-1137`) and
  `ChunkMeshJob` (`voxel_sim.rs:1140-1145`).
- On dispatch (`dispatch_chunk_mesh_tasks`), stamp `generation = state.tick`.
- On apply (`apply_chunk_mesh_tasks`), compare `job.generation` with the
  current `state.tick`. If `job.generation < state.tick`, despawn the carrier
  entity and skip the mesh spawn — the chunk will be re-dispatched on the next
  tick.
- This prevents the frame-gap visual artifact: a stale task's mesh is never
  shown if a newer CA step has already happened.

**Files changed:** `clients/bevy-ref/src/voxel_sim.rs:1129-1145,1235-1299`

---

## 4. Minimal Fix Priority

For **immediate visual improvement** (lowest code change, highest impact):

| Priority | Fix | Impact | Files |
|----------|-----|--------|-------|
| P0 | Fix 2: Boundary snapping | Closes sub-voxel gap for all smooth-mode dense chunks | `voxel_smooth_mesher.rs` |
| P1 | Fix 3: Skirt generation | Fills visual crack even without perfect snapping | `voxel_smooth_mesher.rs` |
| P2 | Fix 5: Generation counter | Eliminates frame-gap race artifact | `voxel_sim.rs` |
| P3 | Fix 4: Seam-band LOD override | Eliminates T-junctions in streaming path | `voxel_stream.rs` |
| P4 | Fix 1: Smooth mesher in streaming | Brings smooth terrain to streaming path | `voxel_stream.rs` |

Fixes 2+3 together close ~90% of visible seam artifacts in the dense path.
Fix 5 eliminates the intermittent flash. Fix 4+5 address the streaming path.

---

## 5. Programmatic Verify Plan

### 5.1 CIVIS_DUMP mesh vert count probe

Extend `scene_dump.rs` to emit per-chunk mesh statistics:

```rust
// New field in the "meshes" section of CIVIS_DUMP JSON:
"per_chunk": [
  { "chunk_id": 42, "vert_count": 1847, "tri_count": 3214, "origin": [64.0, 0.0, 96.0] },
  ...
]
```

**Verification criteria:**
- All non-empty chunks have `vert_count > 0`.
- Chunks at the world boundary (x=0, z=0, x=max, z=max) have boundary vertices
  at the expected coordinates (snapped to 0.0 or `CHUNK_EDGE`).
- No chunk has `vert_count == 0` when its voxel data contains non-AIR material
  (the "dissolved terrain" check).

**Command:**
```powershell
CIVIS_DUMP=terrain-probe.json CIVIS_DUMP_WARMUP=8.0 cargo run -p civ-bevy-window
# Parse terrain-probe.json → assert mesh count == expected chunk count
```

### 5.2 Seam continuity test (unit)

Add to `voxel_smooth_mesher.rs::tests`:

```rust
#[test]
fn two_adjacent_chunks_share_boundary_vertices() {
    // Build two 32³ chunks that are both solid, with identical boundary voxels.
    // Mesh both, then check that boundary vertices (position[d] ≈ 0 or 32)
    // are at the EXACT same coordinates (within ε = 1e-6).
    let chunk_a = [MaterialId(1); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
    let chunk_b = [MaterialId(1); CHUNK_EDGE * CHUNK_EDGE * CHUNK_EDGE];
    let padded_a = /* solid padded apron */;
    let padded_b = /* solid padded apron with correct neighbor context */;
    let bufs_a = build_smooth_meshes(&chunk_a, &padded_a, None, &registry);
    let bufs_b = build_smooth_meshes(&chunk_b, &padded_b, None, &registry);
    // Extract boundary vertices from A (position[0] ≈ CHUNK_EDGE as f32)
    // and from B (position[0] ≈ 0.0).
    // Assert: for each boundary vertex in A at (32, y, z),
    //   there exists a vertex in B at (0, y', z') where
    //   |y - y'| < 0.01 and |z - z'| < 0.01.
}
```

### 5.3 Generation counter staleness test

Add to `voxel_sim.rs::tests`:

```rust
#[test]
fn stale_mesh_task_is_discarded() {
    // Simulate: dispatch a mesh task at tick=1.
    // Advance state.tick to 3.
    // Apply the old task → assert it is NOT spawned (generation mismatch).
}
```

### 5.4 Agent smoke extension

Add a seam check to `scripts/agent-smoke.ps1`:
- Run `CIVIS_DUMP` with terrain probe.
- Parse JSON, assert `meshes.count >= (WORLD_DIMS[0] / 32) * (WORLD_DIMS[2] / 32) * 0.8`
  (at least 80% of expected chunks meshed — allows for all-air chunks).
- Assert `per_chunk` entries have no zero-vert chunks where voxel non-air is present.

---

## 6. Open Questions

1. **Skirt depth tuning.** 1.0 unit is conservative; a depth tied to the Surface Nets
   interpolation error (~0.3 voxels) may be tighter. Defer to visual testing.
2. **Cross-chunk vertex welding.** Snapping (Fix 2) and skirts (Fix 3) are
   approximations. True vertex welding requires a跨-chunk Surface Nets pass, which
   means running the mesher over the full neighborhood set in one pass. This is the
   "correct" fix but costs 8× more density evaluations per chunk. Defer to a
   follow-up if snapping + skirts prove insufficient.
3. **Streaming smooth mesher perf.** The smooth mesher's 38³ padded grid + 5³ blur
   is ~3× more expensive than the cubic mesher per chunk. In the streaming path,
   which meshes hundreds of chunks per frame during camera movement, this may cause
   frame drops. Benchmark `bench_chunk_mesh_smooth` vs `bench_chunk_mesh_cubic`
   before enabling by default; consider smooth-only for `mesh_ring` inner ring
   and cubic for outer rings.
4. **Godot/Unreal parity.** The fixes above are Bevy-specific. Godot and Unreal
   clients consume `Frame3d` wire chunks and mesh independently. The same skirt
   generation and snapping logic should be ported to their mesher shims. File as
   follow-up tasks.
