# Intent: NFR-CIV-SCALE-PERF-900 -- Chunk streaming window

> Date: 2026-09-19
> NFR: NFR-CIV-SCALE-PERF-900
> Epic: NFR-CIV-SCALE-PERF

## User Intent

For large generated worlds the engine must not load every chunk into RAM.
A moving camera focus needs explicit *load* deltas (chunks entering the
Chebyshev ball) and *unload* deltas (chunks falling outside it) so the
resident chunk count stays bounded by `load_radius^3`.

### What This FR Achieves

`crates/voxel/src/scale_stream.rs` provides a pure-logic, engine-agnostic
streaming window over a `ChunkCoord` lattice:

- `StreamingWindow { focus, load_radius, unload_radius, resident_cap, resident }`.
- `update_focus(new_focus)` — recomputes the target Chebyshev ball, emits
  deterministic [`WindowUpdate`]
  with `[LoadOp; …]` and `[UnloadOp; …]` deltas, updates `resident`.
- `WindowError::{InvalidRadii, UnloadRadiusZero}` — construction-time
  validation: `load_radius ≤ unload_radius`, and `unload_radius ≥ 1`.
- `NFR_CIV_SCALE_PERF_900` constant — canonical id for audit.

Acceptance criterion (pinned by the in-module test
`acceptance_focus_shift_loads_and_unloads_with_bounded_residency`):

1. After `update_focus`, `resident` is exactly the Chebyshev ball of
   radius `load_radius` around the new anchor — no leftovers.
2. `resident_count() ≤ load_radius^3` at all times (hard upper bound).

## Determinism

`update_focus` is order-independent given the same
`(focus, radius, resident)` triple. Loads walk the target cube in
deterministic order `(cz, cy, cx)`; unloads walk the previous resident
set in sorted order. Two windows with identical state agree bit-for-bit
on their `WindowUpdate` output.

## Acceptance Signal

- `cargo test -p voxel scale_stream::acceptance` passes.
- `cargo test -p voxel acceptance_focus_shift_loads_and_unloads_with_bounded_residency`
  passes.
- No `bevy_*` imports in the module — engine-agnostic.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/voxel/` |
| Module | `crates/voxel/src/scale_stream.rs:1` |
| Constant | `crates/voxel/src/scale_stream.rs:16` |
| Tests | `crates/voxel/src/scale_stream.rs:427`, `:479`, `:509` |
