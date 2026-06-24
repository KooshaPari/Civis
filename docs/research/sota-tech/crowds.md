# SOTA Traffic / Crowd / Agent-Flow → Civis

**Scope:** the movement + decision layer that *consumes* the road/lane graph ([roads-lanes.md](./roads-lanes.md)) and the navmesh: local collision avoidance, large-group flow, navmesh gen, GPU-scale crowds, and agent decision architectures (utility / GOAP / BT). Maps to Civis's LOD-tiered agent sim ([emergence-charter](../../guides/emergence-charter.md): "full near the camera, statistical far away").

## Civis baseline
`crates/tactics` has hand-rolled grid pathfinding (`pathfinding.rs`, `movement.rs`, `grid_obstacles.rs`, `formation.rs`); `crates/agents/src/daily_path.rs` does per-agent daily routing. No shared navmesh, no continuous-space local avoidance, no flow-field group movement, no GPU crowd path. Decisions are bespoke per-system. Two gaps: (1) **movement substrate** (avoidance + flow + navmesh), (2) **decision substrate** (a reusable utility/GOAP/BT layer instead of ad-hoc logic).

---

## 1. Local collision avoidance — RVO / ORCA / HRVO `[adopt-now]`
For agents sharing space (sidewalks, markets, battles), velocity-obstacle methods are the standard:
- **RVO / ORCA** (Reciprocal Velocity Obstacles / Optimal Reciprocal Collision Avoidance, van den Berg) — each agent solves a small linear program each tick to pick a collision-free velocity assuming others reciprocate. O(neighbors) per agent, scales to thousands. The industry default for smooth, deadlock-resistant crowds.
- **HRVO** (Hybrid RVO) — reduces RVO's reciprocal-dance oscillation; good for symmetric head-on cases.

**Civis fit:** ORCA as the *micro* layer on top of lane/navmesh *macro* routing — the lane graph gives the route, ORCA resolves who-yields locally. **Rust:** no single dominant ORCA crate; options are wrapping `RVO2` (C++, Apache-2, the canonical lib) via `cxx`, porting the ~1k-LOC ORCA core to Rust, or `dodgy`/`dodgy_2d` (pure-Rust ORCA implementation, MIT, used in the bevy ecosystem). **Recommend `dodgy_2d` first** (pure Rust, no FFI), fall back to wrapping RVO2 if we need RVO2's exact behavior. Tag `[adopt-now]`.

## 2. Large-group flow — Flow fields & Continuum Crowds `[adopt-now / experimental]`
- **Flow-field tiles** (`[adopt-now]`): precompute, per shared goal, a vector field over a grid (Dijkstra/eikonal integration → gradient). Any number of agents share one field → **O(1) per-agent lookup**, constant cost regardless of crowd size. The RTS standard (StarCraft 2-era; the "Flow Field Tiles" GameAIPro chapter is the canonical implementation guide — tile the world, stitch fields at borders). Ideal for Civis mass events: migrations, routs, evacuations, herd/animal movement, and far-LOD statistical flow.
- **Continuum Crowds** (`[experimental]`, Treuille et al.; Supreme Commander 2 shipped a version): a *dynamic* potential field where density + speed fields make the crowd itself an obstacle — produces lane-formation and counter-flow emergently. Beautiful but **resource-heavy and historically too costly for large detailed worlds** (SupCom2's own postmortem notes the cost). Use as inspiration / for bounded set-pieces, not the default substrate. GPU continuum (hybrid GPU crowd papers) makes it tractable — a later experimental bet.

**Recommended stack:** flow-field tiles for shared-goal mass movement + ORCA for local resolution. This "macro flow-field / micro ORCA" hybrid is the modern crowd standard and matches our LOD tiers cleanly.

## 3. Navmesh — Recast/Detour in Rust `[adopt-next]`
For off-road / open-area movement (wilderness, pre-road early game, building interiors), a navmesh beats a grid.
- **`oxidized_navigation`** — tiled *runtime* Recast-style navmesh gen for Bevy, pure Rust; ingests Parry3d/Avian/Rapier colliders via the `OxidizedCollider` trait + `NavMeshAffector` component, async tile rebuild. Latest published crate tracks ~Bevy 0.15 — **version-lag is the integration cost** (port/patch to 0.18, or track the fork landscape). Tag `[adopt-next]`.
- **`rerecast` / `bevy_rerecast`** — clean-room Rust Recast (already in the engine-parity top-10). Pairs with Detour-style path queries. Prefer whichever is current on Bevy 0.18 at integration time.

Navmesh (macro for open areas) + lane graph (macro for roads) + ORCA (micro) + flow fields (mass) = the full movement substrate.

## 4. GPU crowds `[experimental]`
For 20mi×20mi with very large visible populations, push avoidance + flow integration to the GPU (compute shaders): GPU ORCA/boids, GPU flow-field integration, GPU continuum crowds (hybrid path-planning papers). Bevy 0.18 wgpu compute makes this feasible in-engine. Tag `[experimental]` — adopt only when CPU LOD-tiering proves insufficient near the camera. Keep statistical far-LOD on CPU.

## 5. Decision architectures — utility / GOAP / BT `[adopt-now]`
Civis needs a **reusable decision substrate** so agent behavior (the emergent psyche/drives layer) isn't re-hand-rolled per system. Bevy-ecosystem options:
- **`big-brain`** — **utility AI** for Bevy (scorers + actions + thinkers). Best fit for Civis: utility AI is *continuous and emergent* — agents weigh competing drives (hunger, safety, social, profit) and the highest-utility action wins, which directly serves the charter's "per-agent drives/temperament shape choices." `[adopt-now]`
- **`bevy_behave`** — modern Bevy **behavior tree** crate (actively tracking recent Bevy). BTs are better for *structured, authored* sequences (a guard patrol, a build job). Use alongside utility AI for sub-tasks. `[adopt-now]`
- **GOAP** (Goal-Oriented Action Planning, F.E.A.R. lineage): planner that chains actions to reach a goal-state via A* over action preconditions/effects. Excellent for *emergent multi-step plans* (gather→craft→trade) without authoring each sequence. Rust crates exist (`goap`, various); smaller ecosystem — port or wrap. `[adopt-next]`

**Recommended:** **utility AI (`big-brain`) as the top-level arbiter** (picks the goal from drives — most charter-aligned), **GOAP to plan the action chain** for the chosen goal, **BT (`bevy_behave`) for leaf execution** of authored sub-sequences. This is the modern layered standard (utility selects, planner sequences, BT executes) and keeps behavior emergent rather than scripted.

---

## Verdict
- **Movement substrate (adopt-now):** macro **flow-field tiles** (mass shared-goal) + micro **ORCA via `dodgy_2d`** (pure-Rust local avoidance), routed by the lane graph (roads) and **navmesh via `oxidized_navigation`/`rerecast`** (open areas, adopt-next — mind the Bevy-version lag).
- **Decision substrate (adopt-now):** **`big-brain` utility AI** (drive-weighted arbitration, most charter-fit) → **GOAP** planning (adopt-next) → **`bevy_behave` BT** leaves.
- **Experimental bets:** GPU crowds + GPU continuum crowds for near-camera mega-populations; CPU LOD-tiering stays the default.

## Sources
- [Continuum Crowds (How to RTS)](https://howtorts.github.io/2014/01/09/continuum-crowds.html) · [Basic Flow Fields (How to RTS)](https://howtorts.github.io/2014/01/04/basic-flow-fields.html)
- [Crowd Pathfinding & Steering Using Flow Field Tiles — GameAIPro Ch.23 (Emerson)](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf)
- [Advanced Techniques for Robust Efficient Crowds — GameAIPro2 Ch.17 (Pentheny)](https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter17_Advanced_Techniques_for_Robust_Efficient_Crowds.pdf)
- [Hybrid Path Planning for Massive Crowd Simulation on the GPU (Springer)](https://link.springer.com/chapter/10.1007/978-3-642-25090-3_26)
- [oxidized_navigation (crates.io / GitHub)](https://github.com/TheGrimsey/oxidized_navigation)
- dodgy_2d, big-brain, bevy_behave — crates.io / GitHub (Bevy ecosystem)
