# Intent: FR-CIV-ARCH-B-003 — Settlement centroid within bounds

> Date: 2026-09-19
> FR: FR-CIV-ARCH-B-003
> Epic: FR-CIV-ARCH (sub-feature B: settlement layout clustering)

## User Intent

`settlement_cluster_centroid` must return a position that lies within the
axis-aligned bounding box of the input member positions. Drift outside the
bounds indicates a centroid-math regression.

## Acceptance Signal

- `crates/build/src/tiers.rs:505` `fr_arch_b003_cluster_centroid_within_bounds` passes.
- Centroid `x` ∈ `[min_x, max_x]` and `z` ∈ `[min_z, max_z]` for the input set.
- `// Covers: FR-CIV-ARCH-B-003` on the test function and source.
