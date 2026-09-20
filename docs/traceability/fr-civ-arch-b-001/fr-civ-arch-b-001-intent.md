# Intent: FR-CIV-ARCH-B-001 — Settlement cluster membership

> Date: 2026-09-19
> FR: FR-CIV-ARCH-B-001
> Epic: FR-CIV-ARCH (sub-feature B: settlement layout clustering)

## User Intent

`BuildingGraph::assign_to_cluster` must be observable: every `BuildingId`
assigned to a cluster appears in `parcels_in_cluster` for that cluster.

## Acceptance Signal

- `crates/build/src/tiers.rs:472` `fr_arch_b001_layout_cluster_membership` passes.
- `BuildingGraph::parcels_in_cluster(cluster_id)` returns the same set of
  `BuildingId`s the user assigned.
- `// Covers: FR-CIV-ARCH-B-001` on the test function and source.
