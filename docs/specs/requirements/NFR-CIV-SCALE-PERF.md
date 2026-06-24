# NFR-CIV-SCALE-PERF — 20mi×20mi Scale, Streaming & 60fps

**Owner:** Voxel-Render + Performance Leads. **Source gap:** Feature Matrix §1/§8 (scale + perf INCOMPLETE).
**Reference bar:** Songs of Syx (100k+ agents, readable + performant at empire scale); CS2 tile streaming. Charter §"Scale target": ~20mi×20mi via SVO + dense-leaf-chunk + chunk streaming + LOD + frustum cull; **disk space is the primary bound**, not compute.
**Emergence note:** these are **NFR** constraints on the [LAW] substrate + render; they shape *how* the simulation runs, not *what* emerges. Determinism (NFR-CIV-DET) must hold across LOD transitions.

## Requirements

| ID | Requirement | Acceptance Criteria |
|---|---|---|
| NFR-CIV-SCALE-900 | The world SHALL support a ~20mi×20mi (≈32km×32km) extent at a 1–4 m base voxel via SVO + dense 16³ leaf chunks. | World of target extent instantiates; addressing covers full extent with fixed-point `WorldCoord`. |
| NFR-CIV-SCALE-901 | Only an active working set SHALL be resident in memory; the rest streams from disk on demand. | Resident chunk count bounded by a configurable budget; out-of-view chunks evict to disk; re-load on approach. Memory stays under budget across a full-map pan. |
| NFR-CIV-SCALE-902 | Disk is the primary bound: the on-disk format SHALL be compact (LOD pyramids + compressed chunks). | Documented bytes/voxel + total-disk estimate for the 20mi target; chunks compressed; LOD mips stored. |
| NFR-CIV-SCALE-910 | Agent simulation SHALL be LOD-tiered: full per-agent (Hot) near camera/active areas, statistical/aggregate (Cold) far away. | `LodTier::{Hot,Warm,Cold}` drives fidelity; Cold uses aggregate models; promotion/demotion preserves determinism contract. |
| NFR-CIV-PERF-900 | Target ≥60fps on the reference desktop (Ryzen/RTX 3090 Ti, DX12) at the active working set; ≥30fps floor under heavy load. | Frame-time budget met in a representative scenario; profiler dump shows no single system >X ms; documented. |
| NFR-CIV-PERF-901 | Rendering SHALL use GPU instancing / indirect draw for massive agent + voxel counts. | 100k+ instanced agents render within frame budget; draw-call count bounded. |
| NFR-CIV-PERF-902 | Streaming + meshing SHALL be off the critical render thread (async dirty-queue drain). | No frame hitch >X ms on chunk load/mesh; dirty queue drained on worker threads. |
| NFR-CIV-SCALE-920 | All LOD/streaming transitions SHALL preserve the determinism contract (NFR-CIV-DET). | Same seed + same camera path → bit-identical sim state regardless of which chunks were resident when. |

**Validation:** full-map pan memory-ceiling test; disk-footprint estimate doc; 60fps profiler-budget scenario (vision + telemetry verified); determinism-across-LOD test.
