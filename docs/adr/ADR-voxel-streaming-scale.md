# ADR: Voxel Streaming Scale Target

**Status:** Accepted
**Date:** 2026-05-30

## Context

The 3D migration documents a volumetric world with SVO storage, dense 16³ leaves, chunk streaming, and LOD as the route to a world measured in tens of miles rather than a small fixed arena. The relevant references are:

- [`docs/guides/voxel-emergent-vision-and-migration.md`](../guides/voxel-emergent-vision-and-migration.md)
- [`docs/adr/ADR-005-adaptive-voxel.md`](ADR-005-adaptive-voxel.md)
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md)

## Decision

Civis will treat voxel streaming scale as a first-class architectural target. The intended substrate is the existing SVO plus dense 16³ leaf-chunk approach, with active working-set streaming, LOD, and frustum culling. The scale target is a large, real-world-equivalent world rather than a tiny fully-resident map.

## Consequences

- World representation must stay chunked and streamable.
- Memory residency is bounded by the active working set, not the full world volume.
- Renderer and simulation code must tolerate load/unload churn at chunk boundaries.
- Any alternative that cannot support the target scale with streaming should be considered off the critical path.

