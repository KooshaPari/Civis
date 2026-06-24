# CIV-W5 — Scale

**Status:** shipped  
**Wave:** scale  
**Primary intent:** make the world large, streamed, and fast enough to support the intended simulation envelope.

## FR Trace

- NFR-CIV-SCALE-900
- NFR-CIV-SCALE-901
- NFR-CIV-SCALE-902
- NFR-CIV-SCALE-910
- NFR-CIV-SCALE-920
- NFR-CIV-PERF-900
- NFR-CIV-PERF-901
- NFR-CIV-PERF-902

## Stories

| Story | Title | FR coverage |
|---|---|---|
| W5.1 | 20mi world extent and fixed-point addressing | NFR-CIV-SCALE-900 |
| W5.2 | Active working set streaming and eviction | NFR-CIV-SCALE-901 |
| W5.3 | Compact on-disk chunk and LOD format | NFR-CIV-SCALE-902 |
| W5.4 | LOD-tiered agent simulation | NFR-CIV-SCALE-910 |
| W5.5 | Determinism across LOD and streaming transitions | NFR-CIV-SCALE-920 |
| W5.6 | 60fps target and draw-call scaling | NFR-CIV-PERF-900, NFR-CIV-PERF-901 |
| W5.7 | Async streaming and meshing off the render thread | NFR-CIV-PERF-902 |

## Story Breakdown

### W5.1 20mi world extent and fixed-point addressing

- Support the intended large-world footprint.
- Keep coordinates stable and addressable across the full extent.

### W5.2 Active working set streaming and eviction

- Keep only the active working set resident.
- Stream the rest in and out on demand.

### W5.3 Compact on-disk chunk and LOD format

- Store the world in a compact, layered format.
- Keep compression and mips part of the contract.

### W5.4 LOD-tiered agent simulation

- Run full detail near the camera and aggregate far away.
- Preserve the sim contract when an entity changes tiers.

### W5.5 Determinism across LOD and streaming transitions

- Ensure camera path and residency changes do not alter final state.
- Keep the same seed producing the same world state.

### W5.6 60fps target and draw-call scaling

- Keep the frame budget inside the reference-performance target.
- Use instancing or indirect drawing for large counts.

### W5.7 Async streaming and meshing off the render thread

- Move chunk loading and meshing off the critical render path.
- Prevent streaming from causing frame hitches.
