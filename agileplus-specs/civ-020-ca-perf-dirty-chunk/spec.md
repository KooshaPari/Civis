---
spec_id: civ-020
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-10
workstream_branch: perf/ca-dirty-chunk-v3
---

# Specification: CA Dirty-Chunk Performance

**Slug**: civ-020-ca-perf-dirty-chunk | **Epic**: E7 | **Date**: 2026-06-10 | **State**: ACTIVE
**Workstream branch**: `perf/ca-dirty-chunk-v3` (in
`C:/Users/koosh/Dev/Civis`, also `E:/civis-wt-ca-dirty` and
`E:/civis-wt-ca-dirty-tmp`)

## Problem Statement

The wave-1 emergence foundation (af913fb2) introduced a
**CA fluid + thermo + percolation upgrade** (FR-CIV-CA-*) on top of the
existing `civ-voxel` dirty-chunk queue. The new CA path iterates
per-cell on dirty chunks and re-scans the full chunk grid for
percolation + thermo propagation each tick. On a 64×64 chunk grid
with 1% random writes, the per-tick CA cost is the dominant
`phase_voxel` contributor, threatening NFR-CIV-PERF-005 (mesh /
terrain budget) and NFR-CIV-PERF-003 (tick budget at 200 civilians).
This spec is the **bottleneck fix + bench gate** for the
`perf/ca-dirty-chunk-v3` workstream; the workstream ships from
`C:/Users/koosh/Dev/Civis` (the canonical primary worktree) and
is mirrored in `E:/civis-wt-ca-dirty` for fast-iteration.

## Target Users

- Voxel / CA engineers optimising the dirty-chunk scan path
- Engine / perf authors wiring the bench into the verify gate
- QA / agent-smoke authors extending `scripts/agent-smoke.ps1`
- Modding authors who rely on the determinism invariant (FR-CIV-DET-001)

## Functional Requirements

- [ ] **FR-CIV-CA-001**: The CA fluid update SHALL be fused with the
  thermo update (single pass per 16³ leaf), eliminating the
  double-iteration that the wave-1 implementation introduced.
- [ ] **FR-CIV-CA-002**: The CA dirty-chunk scan SHALL be batched per
  `chunk_id` (not per-cell) and SHALL use the existing
  `(chunk_id, write_seq)` ordering from FR-CIV-VOXEL-002.
- [ ] **FR-CIV-CA-003**: The percolation queue SHALL be lazy — drain on
  commit, not on read; the queue size is bounded by the number of
  dirty chunks for the current tick.
- [ ] **FR-CIV-CA-004**: `bench_ca_dirty_chunk` (Criterion) SHALL run
  in CI as a non-blocking check and SHALL assert P99 < 16 ms on the
  reference grid (64×64, 1% writes) on the RTX 3090 Ti host CPU.
- [ ] **FR-CIV-CA-005**: The CA path SHALL preserve determinism
  (FR-CIV-DET-001): two same-seed runs of the bench yield bit-identical
  voxel state and bit-identical replay hash chain.

## Non-Functional Requirements

- Determinism: the optimisation MUST NOT change the per-tick voxel
  state for a given `(seed, scenario, snapshot)`; verified by
  `ca_dirty_chunk_optimisation_preserves_determinism`
- Performance: the optimised CA path is a 2–4× speedup over the
  wave-1 baseline (target: P99 < 16 ms vs the wave-1 baseline ~45 ms)
- Test surface: `crates/voxel/`, `crates/engine/src/engine.rs`
  (`phase_voxel`), `benches/ca_dirty_chunk.rs`
- The bench runs with `CARGO_TARGET_DIR=E:/civis-target` per
  civ-018 (FR-CIV-VERIFY-010) so multiple agents don't trash the
  cache

## Constraints and Dependencies

- Depends on FR-CIV-VOXEL-001 (adaptive storage) and FR-CIV-VOXEL-002
  (deterministic dirty queue) — the CA path sits on top of these
- Depends on FR-CIV-DET-001 (cross-run bit-identical replay) for
  the determinism invariant
- Depends on civ-018 (FR-CIV-VERIFY-010) for the shared
  `CARGO_TARGET_DIR` wrapper
- Conflicts-of-record: `perf/ca-dirty-chunk` (legacy), `wt/voxel-fluid` —
  this spec is the v3 contract that closes the perf gap

## Acceptance Criteria

- [ ] `cargo bench --bench ca_dirty_chunk` reports P99 < 16 ms on the
  reference grid
- [ ] `cargo test -p civ-voxel ca_dirty_chunk_optimisation_preserves_
  determinism` exits 0
- [ ] `cargo test -p civ-engine replay_ca_dirty_chunk_bit_identical`
  exits 0
- [ ] `just civis-3d-verify` exits 0 with the bench wired in
  (non-blocking, but the bench artifact is uploaded)

## Implementation Notes

- The bench lives at `benches/ca_dirty_chunk.rs` and uses the same
  64×64 grid + 1% writes pattern as the existing
  `bench_voxel_adaptive_storage` for comparability
- The perf artifact is uploaded via `actions/upload-artifact@v4` keyed
  on `criterion-ca-dirty-chunk` so the PR review can see the
  P99 trend vs the wave-1 baseline
- The workstream branch `perf/ca-dirty-chunk-v3` is **not** the
  canonical branch — the canonical branch is `main`; the v3 branch
  is a perf-test branch that gets squashed on merge

## Status

| Story | Status |
|-------|--------|
| E7.1 Profile + per-phase breakdown | Complete |
| E7.2 Batch scan + lazy percolation + fused fluid/thermo | Complete |
| E7.3 Bench gate + determinism re-verification | Partial |
