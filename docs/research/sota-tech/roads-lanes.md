# SOTA City-Builder Road / Lane Systems → Civis civ-traffic Upgrade Path

**Scope:** the network *data model* + per-lane pathfinding used by city-builders, mapped onto our current `civ-traffic` crate (`crates/civ-traffic/src/lib.rs`), with a concrete target model and Rust integration path. Companion docs: [crowds.md](./crowds.md) (the agent-flow layer that *consumes* the graph), [engine-parity README](../engine-parity/README.md) (rendering plugs — not duplicated here).

## Where Civis is today (baseline)

`civ-traffic` is a **single shared edge graph** with two authoring channels (emergent desire-paths + user freehand) feeding one `TrafficGraph`. The promotion ladder is `None → Trail → Road → Highway (+ Bridge)`; each `RoadKind` exposes a scalar `speed_multiplier`. This is the **WorldBox / Manor Lords desire-path tier** — edges carry a *kind* and a *speed*, but there is **no lane structure, no per-lane connectivity, no turn/junction model, no geometry/spline**. Pathing reads a per-edge scalar cost only.

That is correct and good *for an emergent civ sandbox* — but it caps us below city-builder traffic fidelity (no lane-level congestion, no turn restrictions, no realistic junction behavior, no curved/elevated roads). The target below is an **incremental evolution**, not a rewrite: keep the dual-authoring + emergence, layer a lane graph *underneath* the existing `RoadKind` edges.

---

## SOTA reference models

### 1. Cities: Skylines 1 — Node → Segment → Lane (the canonical model) `[adopt-now]`
The C:S1 network is the de-facto standard for moddable city-builder roads:

- **Node** = a junction/endpoint (a point where segments meet). Owns the intersection logic.
- **Segment** = a directed-or-bidirectional stretch of road between two nodes; carries a road *prefab* (the road type/hierarchy) and a Bézier curve for geometry.
- **Lane** = the actual travel channel inside a segment. Each lane has a type (car/pedestrian/parking/public-transport), a direction, a position offset, and a **Bézier spline** of its own.
- **Lane connections** = at each node, lanes connect to lanes across segments. *This* is the routing graph: agents pathfind **lane-to-lane**, not segment-to-segment. Turn restrictions, lane arrows, and forced merges are all expressed as which lane-connections exist.

Pathfinding runs over the **lane-connection graph** (A*/Dijkstra with per-lane cost = length × congestion × type penalty). The node/segment layer is for *geometry + authoring*; the lane-connection layer is for *routing*. **This separation is the single most important idea to adopt.**

### 2. TM:PE (Traffic Manager: President Edition) — lane routing as data `[adopt-now, study]`
TM:PE is the most-installed C:S1 mod and is the **best open-source reference for lane logic** (MIT-ish, C#, decompiled-friendly). It demonstrates that the *entire* advanced traffic behavior set is just **edits to the lane-connection graph + per-lane policy flags**:
- Lane connectors (manually rewire which lane→which lane at a node).
- Lane-arrows / forced turns, priority signs, junction restrictions (no u-turn, no lane-change near junction).
- Per-lane speed limits, vehicle-type restrictions (bus lanes), parking AI, timed traffic lights.

**Takeaway for Civis:** every "advanced traffic" feature is a *policy annotation on a lane-connection edge*, not new core code. If our lane graph stores `LaneConnection { from_lane, to_lane, allowed_classes, turn, priority }`, we get the whole TM:PE feature surface emergently/declaratively. Study TM:PE's `LaneConnectionManager`, `LaneArrowManager`, `JunctionRestrictionsManager`.

### 3. Cities: Skylines 2 — ECS lane net (validates our architecture, cautionary on pitfalls) `[adopt-now model, avoid the bug]`
C:S2 rebuilt the same node/segment/lane net on **Unity DOTS ECS** — directly analogous to our Bevy ECS. Modders decompiled it into an [interactive ECS component/system explorer](https://news.ycombinator.com/item?id=38700552). Confirms: *the lane graph is the right ECS-native shape* (nodes, segments, lanes, lane-connections as entities/components, pathfinding as a system).

**Cautionary tale (well-documented community failure):** C:S2's launch pathfinding was widely panned. Root cause per community analysis: **CIMs make routing decisions per-node and choose the target lane at the node nearest the intersection** — if that node is too close, vehicles can't merge in time, causing pile-ups and "every agent treats a node as identical, never sees a turn ahead." Lesson: **lane-selection must look *ahead* of the immediate node** (multi-node lookahead / reserve target lane N segments early), and junction node spacing matters. Bake lookahead into the router from day one.

### 4. Road hierarchy + zoning snap `[adopt-next]`
- **Hierarchy** (alley < street < avenue < arterial < highway): not just a speed multiplier — it drives *routing preference* (penalize routing local traffic onto highways and vice-versa), *capacity*, and *zoning eligibility*. Our `RoadKind` ladder is a 1-D proxy; promote it to a `RoadClass` with `{capacity, lane_count_default, routing_penalty, allows_zoning}`.
- **Zoning snap** (C:S): zones attach to road *segments* with a depth offset; cells snap to the segment's spline. For Civis this maps to settlements/structures binding to nearby segments — emergent buildings already want a road adjacency; make the segment the anchor.

### 5. Spline / curved / elevated roads `[adopt-next]`
City-builders represent road geometry as **cubic Bézier per segment** (C:S) with per-lane parallel-offset splines. Elevation/bridge/tunnel/slope are *the same segment with a height profile* (C:S2 treats elevated/bridge/slope/tunnel as sub-networks bundled with the basic net). For Civis-on-voxels, the spline drives (a) the voxel stamp footprint and (b) the agent travel curve; elevation = a per-segment height curve sampled into the SVO.

### 6. Manor Lords — organic desire-paths (we already do this; keep it) `[adopt-now, have it]`
Manor Lords roads emerge from accumulated foot traffic (no grid, no zoning), exactly our `Emergent` channel. Validation that our desire-path accumulation is SOTA for *organic* settlements. The upgrade is: when an emergent edge promotes past `Trail`, **lift it into the lane graph** (a Trail = 1 bidirectional pedestrian lane; a Road = 1 lane/direction; a Highway = 2+ lanes/direction). Emergence stays; lanes are generated at promotion time.

### 7. OSM-style procedural network gen `[experimental]`
For large worldgen seeds, OSM road-network generators (and academic procedural-city road grammars: tensor-field / template-based growth, e.g. CityEngine-style, Parish-Müller) produce realistic arterial+local hierarchies. Useful as a *worldgen prior* (pre-seed major arterials between settlement sites) rather than runtime. Tag experimental — only if we want pre-built civilizations rather than pure ground-up emergence.

---

## Concrete target model for civ-traffic

Evolve the single-edge graph into a **two-tier graph**: keep the existing authoring/promotion tier, add a lane tier beneath it. ECS-native (Bevy entities or `BTreeMap`-keyed for order-determinism, matching the current crate style).

```text
TIER 1 — AUTHORING / EMERGENCE (mostly exists today)
  Node     { pos: WorldCoord, junction_kind }
  Segment  { a: Node, b: Node, class: RoadClass, spline: Bezier, height: HeightCurve,
             provenance: InfraProvenance, traffic_weight }   // promotion still lives here
  RoadClass (replaces flat RoadKind) { tier, capacity, default_lanes, routing_penalty, allows_zoning }

TIER 2 — ROUTING (new; generated from Tier 1 at promotion time)
  Lane           { segment: Segment, index, class: LaneClass(Foot/Cart/Vehicle/...),
                   direction, offset_spline: Bezier }
  LaneConnection { from: Lane, to: Lane, node: Node, turn: Turn,
                   allowed_classes, priority }                // ← THE routing graph
```

**Pathfinding** runs over `LaneConnection` (A* / contraction-hierarchy), cost = `lane_length × congestion(lane) × class_penalty`. Multi-node **lookahead** for lane selection (avoid the C:S2 bug). The existing `speed_multiplier_at` becomes a thin shim over per-lane cost so the life-sim keeps working during migration.

**Migration ladder (forward-only, non-destructive):**
1. Rename `RoadKind`→`RoadClass` carrying `{capacity, default_lanes, routing_penalty}` (keep the enum-like tiers; just enrich them). Existing promotion logic unchanged.
2. Add Bézier `spline` + `HeightCurve` to segments (geometry only; pathing still scalar). Unlocks curved/elevated rendering + voxel stamping.
3. Auto-generate Tier-2 lanes from segment `class` at promotion (`default_lanes`). Add lane-to-lane connections at nodes with default turn-all.
4. Switch the router to the lane-connection graph with congestion + lookahead.
5. (Optional) Expose TM:PE-style `LaneConnection` policy edits to the user spawn-tools (turn restrictions, class restrictions) — emergent OR authored, same as roads today.

This preserves the **dual-authoring + emergence charter invariant** (both channels feed one graph) while reaching C:S-class lane fidelity.

---

## Rust / Bevy crates for the geometry + routing layer

| Need | Crate | Tag | Notes |
|---|---|---|---|
| Bézier/spline math for segments + lane offsets | **`kurbo`** | `[adopt-now]` | de-facto Rust 2D curve lib (Linebender); flatten, offset, arclen, split — exactly the lane-offset-spline + zoning-snap math. |
| Curve/path tessellation (render road ribbons) | **`lyon`** | `[adopt-now]` | path tessellation to meshes; pairs with kurbo for the road surface ribbon + lane markings. |
| Bevy-native splines / curves | **`bevy_math` cubic curves** (in-tree) + **`bevy_spline`/`splines` crate** | `[adopt-now]` | Bevy 0.18 has first-class `CubicBezier`/`CubicCurve` in `bevy_math` — use in-tree before pulling an external spline crate. |
| Routing graph / A* / contraction hierarchies | **`petgraph`** + **`pathfinding`** crate | `[adopt-now]` | lane-connection graph storage + A*/Dijkstra. For 20mi×20mi scale, add **contraction hierarchies** (`fast_paths` crate) for precomputed long-haul queries. |
| Navmesh for off-road / pedestrian areas | **`rerecast`/`oxidized_navigation`** | `[adopt-next]` | see crowds.md; complements lane graph for open-area movement. |

---

## Verdict

- **Adopt the C:S Node→Segment→Lane→LaneConnection model now** as the target; it is ECS-native and matches our crate's determinism style. Lane-connection graph = the routing layer; node/segment = authoring/geometry.
- **Keep our emergence + dual-authoring**; generate lanes at promotion time so desire-paths still drive the world. Manor Lords parity already achieved.
- **Study TM:PE** as the proof that all advanced traffic = policy flags on lane-connections (no bespoke systems).
- **Avoid the C:S2 per-node lane-selection bug**: build multi-node lookahead into the router from the start.
- **Rust path:** `kurbo`+`lyon`+`bevy_math` curves for geometry; `petgraph`+`pathfinding`(+`fast_paths`) for the lane-routing graph. All MIT/Apache, Bevy-0.18 compatible, wrap-over-handroll compliant.

## Sources
- [C:S2 ECS Systems/Components Explorer (HN)](https://news.ycombinator.com/item?id=38700552)
- [C:S Network Asset Creation — node/segment/lane/elevation](https://cslmodding.info/asset/network/)
- [C:S2 lane / pathfinding per-node decision discussion](https://steamcommunity.com/app/949230/discussions/0/4349988612566900418/)
- [C:S2 path AI critique (community root-cause)](https://steamcommunity.com/app/949230/discussions/0/4406291330030559509/)
- kurbo (Linebender), lyon, petgraph, pathfinding, fast_paths — crates.io
