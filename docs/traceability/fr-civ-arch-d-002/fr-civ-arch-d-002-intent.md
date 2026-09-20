# Intent: FR-CIV-ARCH-D-002 — Clustered parcel offset determinism

> Date: 2026-09-20
> FR: FR-CIV-ARCH-D-002
> Epic: FR-CIV-ARCH-D

## What This FR Captures

Determinism guarantee for `clustered_parcel_offset(cluster, idx, cfg)`
— the per-cluster spatial offset used when scattering parcels inside a
cluster. The function must return the same `(x, y)` offset for the
same `(cluster, idx, cfg)` pair across calls and across replays.

## User Intent

Clustering drives how emergent parcels spread within a settlement
zone. If offsets drifted between runs, replays would diverge
visually and the determinism guarantees of the engine would
break. This test sweeps a small set of `(cluster, idx)` pairs to
pin the contract.

## Acceptance Signal

- `cargo test -p build fr_arch_d002_clustered_offset_is_deterministic`
  passes.

## Implementing Code

- `crates/build/src/tiers.rs:618` — test
  `fr_arch_d002_clustered_offset_is_deterministic`
- Production code: `clustered_parcel_offset` in the same file.

## Test Coverage

- `crates/build/src/tiers.rs:618` (dedicated test, swept 4 clusters
  × 16 indices)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-arch-d-002-intent.md` |
| Implementing crate | `crates/build/src/` |