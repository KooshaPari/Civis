# Intent: FR-CIV-ARCH-B-004 — Independent cluster tracking

> Date: 2026-09-19
> FR: FR-CIV-ARCH-B-004
> Epic: FR-CIV-ARCH (sub-feature B: settlement layout clustering)

## User Intent

`BuildingGraph` must track multiple clusters independently: assignments to
different cluster ids never bleed into each other, and
`settlement_cluster_count` reflects every distinct cluster the user created.

## Acceptance Signal

- `crates/build/src/tiers.rs:514` `fr_arch_b004_distinct_clusters_stay_separate` passes.
- `parcels_in_cluster(1).len() == 1`, `parcels_in_cluster(2).len() == 2`.
- `settlement_cluster_count() == 2` after two distinct assignments.
- `// Covers: FR-CIV-ARCH-B-004` on the test function and source.
