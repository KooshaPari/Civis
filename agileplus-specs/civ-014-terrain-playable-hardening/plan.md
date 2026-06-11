# Plan: Terrain Playability Hardening (civ-014)

## Phased WBS

### Phase 1: Smoke gate (E7.1)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| T1.1 | Add `playable` block to `scripts/agent-smoke.ps1`: WS + watch + Unreal preflight in series; fail-fast on first non-zero | — | Planned |
| T1.2 | Document the gate in `docs/guides/agent-smoke.md` and link from `AGENTS.md` | T1.1 | Planned |

### Phase 2: Chunk-seam + CA-dirty (E7.2, E7.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| T2.1 | `chunk_seam_watertight` integration test in `civ-voxel` | — | Planned |
| T2.2 | `bench_ca_dirty_chunk` Criterion bench; assert P99 < 16 ms in CI artifact | T2.1 | Planned |
| T2.3 | Land `wt/chunk-seam` and `perf/ca-dirty-chunk` against this spec | T2.2 | Planned |

### Phase 3: Map2D / Water / Y-axis / Spawn (E7.4–E7.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| T3.1 | `map2d_zoom_round_trip` test in `civ-watch` + dashboard | T1.1 | Planned |
| T3.2 | Land `wt/map2d-zoom` and `wt/map2d-ux-2494` | T3.1 | Planned |
| T3.3 | Water material id centralisation test (all three clients resolve the same id for "water") | T1.1 | Planned |
| T3.4 | `actor_y_persists_across_replay` test in `civ-engine` | T1.1 | Planned |
| T3.5 | Land `wt/water-placement`, `wt/actor-y-fix`, `wt/emergence-spawn`, `wt/map-seed` | T3.3, T3.4 | Planned |

## DAG Dependencies

```
T1.1 → T1.2
T1.1 → T2.1 → T2.2 → T2.3
T1.1 → T3.1 → T3.2
T1.1 → T3.3 → T3.5
T1.1 → T3.4 → T3.5
```
