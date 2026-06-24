# Songs of Syx Teardown for Civis Spec Map

## Overview
Songs of Syx is a city-builder / grand-strategy hybrid whose entire pitch is **scale**: simulate a settlement growing into an empire of **tens of thousands of individual citizens** (40,000+), each a real agent with race, needs, job, and class — not an abstract number. It is Civis's best reference for **LOD-tiered agent simulation at empire scale**, **performance under huge populations**, and **statistics UI that stays readable** when the population is enormous. Directly relevant to Civis's 20mi×20mi + LOD-tiered-agents target.

## Feature & Systems Teardown
### Massive-scale population sim
The sim models thousands-to-tens-of-thousands of citizens individually, down to race/class divides and warfare ([rpgcodex thread](https://rpgcodex.net/forums/threads/songs-of-syx-city-builder-grand-strategy-game-with-emphasis-on-scale.132581/); [Citizens wiki](https://songsofsyx.com/wiki/index.php/Citizens)). Population is the core resource and constraint.

### Needs, services, happiness at scale
Citizens have needs (food variety, housing, services, religion, entertainment) and a happiness/loyalty model; unmet needs → unrest. The challenge is keeping per-agent need satisfaction tractable across 10k+ agents — the game leans on **aggregate service-coverage fields** + sampled per-agent checks rather than fully simulating every interaction every tick.

### Race/species diversity
Multiple playable/managed races with different needs, productivity, and tolerances; emergent inter-race tension. Maps to Civis's emergent multi-species substrate.

### Room/zone designation
Players paint rooms (housing, farms, workshops, services) rather than place fixed buildings; the sim fills and staffs them. Designation-as-hint, not rigid placement.

### Performance characteristics (the key lesson)
Songs of Syx is **heavily CPU-bound**; late-game slowdowns are CPU (deep per-agent sim), not GPU. Players lower **Unit Detail** and shadow quality to keep sim speed at 10k+ pop ([performance guide](https://www.gamehelper.io/games/songs-of-syx/articles/songs-of-syx-performance-optimization-guide-best-settings-for-maximum-fps)). The render LODs aggressively (units become dots/blobs when zoomed out) while the sim continues — a direct analog to Civis Hot/Warm/Cold tiers.

### Statistics / graphs UI
Extensive empire-level statistics screens: population graphs, resource flows, happiness, military — aggregate dashboards with drill-down, essential for steering a 40k-pop empire you cannot inspect agent-by-agent.

## What it NAILS
- **Believable scale** — tens of thousands of *individual* agents, not abstractions.
- **Aggregate-coverage + sampled-agent** hybrid keeping needs tractable at scale.
- **Aggressive render LOD** decoupled from sim LOD.
- **Empire-scale stats dashboards** with drill-down.
- **Room/zone designation** as a hint the sim fills.

## What to ADOPT for Civis
- **LOD-tiered agent sim** (full near camera, statistical far) — already Civis's `LodTier`. Validate against SoS's CPU-bound reality. `[EMERGENT]`+NFR → NFR-CIV-SCALE-910, FR-CIV-PSYCHE-921.
- **Aggregate service-coverage fields** feeding overlays + cheap far-sim. `[UI/QoL]`+`[EMERGENT]` → FR-CIV-INFOVIEW-912.
- **Empire-scale statistics dashboards** with aggregate+drill-down. `[UI/QoL]` → FR-CIV-NOTIFY-910/911.
- **Render LOD decoupled from sim LOD** (dots when zoomed out). `[UI/QoL]`+NFR → NFR-CIV-PERF-901.
- **Designation-as-hint** (rooms/zones bias agent behavior, don't force it). `[UI/QoL]` → FR-CIV-ROAD-921.

## What to AVOID
- SoS is **CPU-bound and single-threaded-ish** in its hot loops — Civis must parallelize (off-thread meshing/streaming, NFR-CIV-PERF-902) to avoid the same late-game wall.
- Its 2D top-down render dodges Civis's 3D-voxel cost; don't assume its pop ceilings translate without GPU-driven rendering.
- SoS races are semi-hardcoded; Civis species must remain genomically emergent.

## Bevy / Rust ecosystem notes
GPU instancing / indirect draw is mandatory to beat SoS's CPU wall at Civis scale; `egui_plot` for the stats dashboards; `big_space` for the 20mi extent. See [bevy-ecosystem-reference](./bevy-ecosystem-reference.md).

## Sources
- Songs of Syx scale thread (rpgcodex) — https://rpgcodex.net/forums/threads/songs-of-syx-city-builder-grand-strategy-game-with-emphasis-on-scale.132581/
- Citizens — https://songsofsyx.com/wiki/index.php/Citizens
- Administration — https://songsofsyx.com/wiki/index.php/Administration
- Performance / 60fps settings guide — https://www.gamehelper.io/games/songs-of-syx/articles/songs-of-syx-performance-optimization-guide-best-settings-for-maximum-fps
- "How it feels to rule" review (scale feel) — https://proc3ss.com/reviews/songs-of-syx-how-it-feels-to-rule
- Steam page — https://store.steampowered.com/app/1162750/Songs_of_Syx/
