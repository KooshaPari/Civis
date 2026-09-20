# Intent: FR-CIV-ARCH-B-002 — Clustered parcel offset spread

> Date: 2026-09-19
> FR: FR-CIV-ARCH-B-002
> Epic: FR-CIV-ARCH (sub-feature B: settlement layout clustering)

## User Intent

`clustered_parcel_offset(cluster, slot, ring)` must diverge from a single
point so consecutive parcel slots occupy distinct `(x, z)` offsets inside
the cluster ring.

## Acceptance Signal

- `crates/build/src/tiers.rs:491` `fr_arch_b002_cluster_offsets_diverge_from_centre` passes.
- Eight consecutive slot indices return more than one unique `(x, z)` pair.
- `// Covers: FR-CIV-ARCH-B-002` on the test function and source.
