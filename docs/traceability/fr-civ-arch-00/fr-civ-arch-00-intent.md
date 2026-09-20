# Intent: FR-CIV-ARCH-00 -- Architectural cluster (001+002) oracle

> Date: 2026-09-19
> FR: FR-CIV-ARCH-00
> Epic: FR-CIV-ARCH

## User Intent

`FR-CIV-ARCH-00` is the audit-cluster alias for the shared behavioural
oracle that simultaneously verifies `FR-CIV-ARCH-001` (3D tiled WFC solver
wraps an external crate) and `FR-CIV-ARCH-002` (`BuildingGraph` stays the
authoritative structural schema). The two are exercised by the same
deterministic fixture because a regression in either requirement breaks the
exact same on-disk golden.

### What This FR Achieves

Provides a single, replay-safe test entry point that pins both WFC and
graph-as-source-of-truth invariants for `civ_build::ArchitectureMode ::
{Canonical,Primitive}`. The cluster oracle makes it impossible to ship a
hand-rolled WFC core or to demote `BuildingGraph` to a side table without
tripping CI.

## Acceptance Signal

- `cargo test -p engine fr_civ_act_arch_cluster` passes.
- The cluster still pins the same `BuildingProvenance`/tile-set pair across
  re-runs.
- No hand-rolled WFC re-introduction (audit script `fr-coverage-*` reports
  zero `hand_rolled_wfc_core` mentions).

## Traceability

| Artifact | Path |
|----------|------|
| Covers: FR-CIV-ARCH-001 | `docs/traceability/fr-civ-arch-001/fr-civ-arch-001-intent.md` |
| Covers: FR-CIV-ARCH-002 | `docs/traceability/fr-civ-arch-002/fr-civ-arch-002-intent.md` |
| Cluster oracle | `crates/engine/tests/fr_civ_act_arch_cluster.rs` |
| Implementing crate | `crates/build/src/` |
