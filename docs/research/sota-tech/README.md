# SOTA Tech Survey — Sim / Physics / Graphics for Civis

Broad SOTA + experimental survey across simulation, physics, and graphics, mapped to Civis gaps + Bevy-0.18 integration paths. **Complements** (does not duplicate) the [engine-parity README](../engine-parity/README.md) AAA-plug list — that doc owns the GI/upscaler/meshlet/navmesh *plug* table; this set is the broader sim/physics/gfx *technique* survey. Grounded in the [emergence charter](../../guides/emergence-charter.md).

## Documents
- [roads-lanes.md](./roads-lanes.md) — City-builder road/lane systems → civ-traffic lane-graph upgrade
- [crowds.md](./crowds.md) — Traffic/crowd/agent-flow (ORCA, flow fields, navmesh, utility/GOAP/BT)
- [material-physics.md](./material-physics.md) — Physics/material sim (CA → GPU CA → MPM/FLIP) + rigid/destruction
- [gfx.md](./gfx.md) — SOTA + experimental graphics (Solari, VXGI, clouds, virtual texturing, splatting)
- [sim-misc.md](./sim-misc.md) — DF world-history gen, Songs-of-Syx scale, ML behavior, GPU ECS

Tag legend: `[adopt-now]` in-tree/mature, integrate now · `[adopt-next]` mature, next wave · `[experimental]` promising bet · `[avoid]` superseded/poor fit. All recommendations OSS (MIT/Apache/BSD/zlib/MPL) + Bevy-0.18 compatible unless noted.

---

## ADOPT-NOW — Top 10 (ranked by impact ÷ effort)

| # | Adopt | Area | Gives Civis | Effort | Why ranked here |
|---|---|---|---|---|---|
| 1 | **Node→Segment→Lane→LaneConnection model** (C:S) under civ-traffic | roads | Lane-level traffic, turn/junction logic, curved/elevated roads — keeping emergence+dual-authoring | Med | The single biggest *sim-fidelity* leap; ECS-native; evolves (not rewrites) our edge graph. See roads-lanes.md migration ladder. |
| 2 | **`bevy_solari` ReSTIR GI + specular** (in-tree, 0.19) | gfx | Lumen-class GI **and** RT reflections in one system; DXR (we target it) | Low–Med | Biggest visual leap per effort; maintained in-tree; already engine-parity #2. |
| 3 | **Harden CA: dirty-rect sleeping chunks + 3D update-order + rayon + temp/pressure fields + data-table reactions** (Noita+Powder-Toy) | physics | Scales our voxel CA substrate; unlocks fire/steam/ice/stress emergently | Low–Med | No new deps; foundation for all later material work; pure win on the Layer-0 substrate. |
| 4 | **`big-brain` utility AI** (+`bevy_behave` BT leaves) | crowds/AI | Reusable drive-weighted decision substrate (charter psyche layer) — emergent, not scripted | Low–Med | Stops per-system hand-rolled AI; utility-arbitration is the most charter-aligned decision model. |
| 5 | **Flow-field tiles + ORCA (`dodgy_2d`)** macro/micro crowds | crowds | O(1)-per-agent mass movement (migrations/routs/herds) + smooth local avoidance | Med | Modern crowd standard; matches LOD tiers; pure-Rust micro layer, no FFI. |
| 6 | **GPU-driven rendering + procedural atmosphere + volumetric fog** (in-tree) | gfx | AAA sky/day-night (ties to `crates/planet`) + draw scaling, cheap | Low | Already in-tree (0.16+); turn on now; high polish per effort. |
| 7 | **DLSS / DLSS-RR (`dlss_wgpu`) + TAA + AgX tonemapping** | gfx | Upscaling (our NVIDIA target) + filmic HDR; DLSS-RR denoises Solari | Low | First-party 0.17+; keep default tonemapping LUTs (project black-PBR pitfall). engine-parity #1. |
| 8 | **`Avian` rigid bodies + CA structural-stress fracture** | physics | Destruction (Chaos analog) — CA fractures → rigid debris | Med | Pure-Rust ECS-native; destruction loop the voxel world wants; no Chaos port. |
| 9 | **DF-style coarse pre-sim → event-log → Legends view** | sim | Emergent deep history before play; browsable living-world record | Low–Med | Architecture not library; we already have the event stream (`crates/watch`); huge living-world payoff. |
| 10 | **SoS-style LOD aggregation + pooled/batched hot loops** on ECS SoA | sim | 40k+ individually-simulated agents at framerate | Med | The concrete scale discipline for 20mi×20mi; ECS gives SoA free; benchmark target. |

## EXPERIMENTAL BETS — Top 5

| # | Bet | Area | Payoff | Risk / status |
|---|---|---|---|---|
| 1 | **GPU cellular automata** (CA stencil → wgpu compute, dirty-chunk dispatch, render from GPU buffers) | physics | 1–2 orders more simulated voxels at framerate — *the* scale lever for the substrate | Med — determinism-not-required clears it; keep CPU CA for far-LOD/authoritative. Strongest bet; near adopt-next. |
| 2 | **MLS-MPM premium-material solver** (snow/sand/mud/lava/elastic unified, grid-coupled, bounded near-camera) | physics | AAA hero-material sim coupled to the CA world | Med–High — no mature Rust crate; port the compact MLS-MPM core to a compute shader or wrap CUDA/wgpu kernel. |
| 3 | **GPU crowds / GPU continuum crowds** (avoidance+flow on compute) for near-camera mega-populations | crowds | SupCom2-style emergent lane-formation at huge scale | Med–High — profile CPU LOD-tiering first; continuum historically costly. |
| 4 | **3D Gaussian Splatting** for static photoreal backdrops / hero landmarks | gfx | Photoreal set-dressing at 100+ FPS, no render-time NN | Med — static only; CANNOT be CA-simulated/destroyed → never the world substrate. Narrow but real. |
| 5 | **ML/LLM agent flavor** (dialogue/culture/naming via Firepass/Kimi; small local policies) over the utility/GOAP base | sim/AI | Richer emergent culture/language flavor | Med — prefer emergent cultural-evolution *rules* first; ML for flavor, not core sim control. |

**Avoid:** DDGI/surfels (superseded by in-tree Solari) · VXGI as *primary* GI (prev-gen; only a non-DXR fallback, where our voxels make voxelization cheap) · SPH for bulk fluid (prefer grid-coupled MPM/FLIP) · full virtual texturing unless decal/terrain-paint detail demands it (triplanar+material-array covers voxels).

---

## Recommended ROAD/LANE model for civ-traffic

Evolve today's single `RoadKind` edge graph (`None→Trail→Road→Highway`, scalar speed) into the **Cities-Skylines two-tier graph**, keeping our emergence + dual-authoring:

```
TIER 1 (authoring/emergence — mostly exists):
  Node{pos,junction} · Segment{a,b, class:RoadClass, spline:Bézier, height, provenance, traffic_weight}
  RoadClass replaces flat RoadKind → {tier, capacity, default_lanes, routing_penalty, allows_zoning}
TIER 2 (routing — NEW, generated from Tier 1 at promotion):
  Lane{segment, index, class, direction, offset_spline}
  LaneConnection{from_lane, to_lane, node, turn, allowed_classes, priority}   ← THE routing graph
```
Pathfinding runs over **LaneConnection** (A* / `fast_paths` contraction hierarchies), cost = `lane_length × congestion × class_penalty`, with **multi-node lookahead** for lane selection (avoids the documented C:S2 per-node pile-up bug). Emergent edges generate lanes at promotion (Trail=1 ped lane, Road=1/dir, Highway=2+/dir). **TM:PE proves** all advanced traffic (turn restrictions, bus lanes, priorities) = policy flags on `LaneConnection` — no bespoke systems. **Rust:** `kurbo`+`lyon`+`bevy_math` curves (geometry); `petgraph`+`pathfinding`+`fast_paths` (routing). Forward-only migration ladder in [roads-lanes.md](./roads-lanes.md).

## Recommended MATERIAL-SIM upgrade path (CA → GPU → MPM)

```
NOW   Harden CPU CA   dirty-rect sleeping chunks + 3D bottom-up/checkerboard order + rayon chunk-parallel
                      + temperature & pressure fields + data-table reactions (Powder-Toy model, in crates/laws)
NEXT  GPU CA          port stencil to wgpu compute, dispatch only dirty chunks, render from GPU buffers;
                      CPU CA stays for far-LOD/authoritative regions
NEXT  Destruction     Avian rigid debris + CA structural-stress fracture loop
LATER MLS-MPM/FLIP    bounded near-camera regions (snow/sand/mud/lava unified via MLS-MPM; hero water via FLIP),
                      coupled to the CA at region boundaries (CA supplies/absorbs mass). Experimental bet #1–2.
```
CA stays the charter Layer-0 substrate everywhere; GPU CA scales it; MPM/FLIP upgrade only the hero materials the camera lingers on. Determinism-not-required (charter, 2026-05-29) clears floats/GPU/non-deterministic ordering across this entire path.

---

## Sources
Per-topic citations in each doc. Key anchors: [Solari 0.19 (Apr 2026)](https://jms55.github.io/posts/2026-04-12-solari-bevy-0-19/) · [C:S node/segment/lane net](https://cslmodding.info/asset/network/) · [Noita CA tech (80.lv)](https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation) · [Powder Toy](https://powdertoy.co.uk/) · [MLS-MPM / CK-MPM (arXiv 2412.10399)](https://arxiv.org/html/2412.10399) · [Flow Field Tiles — GameAIPro Ch.23](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf) · [DF Legends/worldgen](https://dwarffortresswiki.org/index.php/World_generation) · [Songs of Syx scale (PC Gamer)](https://www.pcgamer.com/songs-of-syx-is-a-base-building-game-with-massive-scale-battles/)
