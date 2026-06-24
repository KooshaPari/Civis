# Plan: CA Dirty-Chunk Performance (civ-020)

Bottleneck-fix and benchmark for the cellular-automata fluid/thermo/
percolation path on the dirty-chunk work surface.

## Phased WBS

### Phase 1: Profile (E7.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C1.1 | `bench_ca_dirty_chunk` Criterion bench (current) | — | Partial |
| C1.2 | Capture per-phase breakdown: scan, simulate, dirty, propagate | C1.1 | Partial |
| C1.3 | Profile with `cargo flamegraph` on 64×64 chunk grid, 1% writes | C1.2 | Partial |

### Phase 2: Optimisation (E7.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C2.1 | Replace per-cell scan with 16³ leaf scan, batched | C1.3 | Partial |
| C2.2 | Lazy percolation queue (drain on commit, not on read) | C1.3 | Partial |
| C2.3 | Fluid/thermo update fused (no double-iteration) | C1.3 | Partial |

### Phase 3: Gate (E7.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| C3.1 | Re-run `bench_ca_dirty_chunk`; assert P99 < 16 ms | C2.* | Planned |
| C3.2 | Determinism re-verification: same-seed rerun yields bit-identical
  voxel state | C2.* | Partial |
| C3.3 | Wire bench into `just civis-3d-verify` as a non-blocking check | C3.1, C3.2 | Partial |

## DAG Dependencies

```
C1.1 → C1.2 → C1.3
C1.3 → C2.1, C2.2, C2.3
C2.1, C2.2, C2.3 → C3.1, C3.2
C3.1, C3.2 → C3.3
```
