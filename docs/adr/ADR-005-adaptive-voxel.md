# ADR-005: Adaptive Hybrid Voxel Substrate with Deterministic Mesh-Dirty Queue

**Date:** 2026-05-22
**Status:** PROPOSED
**Author:** Civis 3D Extension

---

## Context

The Civis 3D extension (`docs/roadmap/civis-3d-extension.md`) requires a voxel substrate
that supports:

1. **Determinism + replay** (ADR-004): every voxel write must produce a deterministic
   event ordering so chunk-mesh rebuilds are bit-identical across machines and replays.
2. **Wide range of detail** — coarse, far-from-camera terrain at planet scale; fine,
   near-camera detail for buildings + tactical combat + Teardown-style damage.
3. **Multiple renderers in parallel** (Bevy / Godot / Unreal) — same chunks, three
   meshers. See ADR-007.
4. **WSM3D-lineage scale lessons** — `VoxelScaleMultiplier`, LOD threshold consistency,
   2D-position-vs-3D-frustum culling, JSON-settings staleness sanity checks. The new
   substrate must absorb these as first-class invariants, not afterthoughts.

Naive choices fail:
- **Flat chunked arrays (Minecraft-style)** waste memory and force uniform resolution
  across the whole planet.
- **Pure octree** is memory-efficient but hot-loop unfriendly for meshing and neighbor
  queries.
- **OpenVDB** is excellent for film/VFX sparse fields but heavier than needed for
  game-authoritative per-chunk dirty tracking.
- **Pure RLE** struggles with localized edits and meshing across 3D boundaries.

## Decision

Adopt a **hybrid sparse voxel octree (SVO) + dense 16³ leaf chunks** as the primary
voxel storage. Implementation lives in a new top-level Phenotype-org-shared crate
`phenotype-voxel`, consumed by Civis (`crates/voxel`) as a path / git dependency, and
by WorldSphereMod3D via a C ABI generated through `ffi-core`/`cbindgen`.

Core guarantees:

1. **Deterministic dirty queue.** Every write emits a `DirtyChunkEvent{chunk_id, write_seq, bounds}`.
   Consumers drain events in `(chunk_id, write_seq)` order. No HashMap-iteration ordering
   leaks into the public API.
2. **World coordinates are fixed-point `i64` at `10^6` scale**, matching `civ-engine`'s
   `Fixed`. No raw float positions cross crate boundaries.
3. **`VoxelScaleMultiplier` is a first-class semantic** with default 8.0 (the WSM3D
   visible default). LOD thresholds must compose with it; the substrate exposes a helper
   so consumers cannot accidentally desynchronise.
4. **Per-engine mesher trait** (`Mesher`) — `phenotype-voxel` ships the trait + Bevy
   implementation. Godot + Unreal implementations live in `clients/godot-ref` and
   `clients/unreal-show` respectively. Vertex / index buffers are mesh-neutral.

## Consequences

- **Replay-safe by construction.** Chunk-mesh rebuild order is part of `.civreplay`
  contract.
- **Memory locality where it matters.** Dense 16³ leaves keep hot voxel data
  cache-friendly; the SVO trims far-field memory.
- **Cross-repo discipline.** A standalone `phenotype-voxel` repo forces stable API
  boundaries; WSM3D and Civis both consume the same kernel.
- **Per-engine mesher work is bounded.** Each renderer ships its own adapter; the
  substrate stays engine-agnostic.
- **No accidental float leakage.** World coords are fixed-point everywhere; engines
  convert at the renderer boundary.

## Alternatives Considered

- **`bevy_voxel_world` only.** Tied to Bevy; can't serve Godot or Unreal cleanly.
- **OpenVDB FFI.** Heavyweight; build chain pain on Windows; over-featured for
  per-chunk dirty tracking.
- **Civis-internal crate (no shared repo).** Forfeits WSM3D consolidation and the
  Phenotype-org reuse protocol.

## Cross-references

- Detailed design: `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md`
- ADR-004 (deterministic replay) — this ADR extends its scope into voxel writes.
- ADR-007 (three renderers) — defines the consumer set of the mesher trait.
