# WorldBox Teardown for Civis Spec Map

## Overview
WorldBox is a 2D pixel sandbox god-simulator: the player is a deity poking a tile world with an enormous palette of powers and watching civilizations, wars, religions, and ecosystems emerge. It is the single best reference for Civis's **god-tool palette**, **spawn-anything sandbox feel**, and **inspect-anything** loop. Its whole appeal is "set conditions, then watch the world react" — exactly Civis's emergence thesis in 2D.

## Feature & Systems Teardown
### God-tool palette (the gold standard)
WorldBox exposes ~**374 powers** organized into **8 tabs**: **Main**, **Unit**, **World Shaping**, **Noosphere and Life**, **Animals/Creatures/Monsters**, **Nature and Disasters**, **Destruction and Chaos**, and **Other various powers** ([Powers wiki](https://the-official-worldbox-wiki.fandom.com/wiki/Powers)). Categories include:
- **World shaping / terrain & biomes** — raise/lower land, mountains, sand, soil, grass/biome seeds, water/lava brushes. Some powers are *agents* themselves (Langton-style ants: Blue Ant→sand+ocean, Green Ant→soil, Black Ant→mountains) — i.e., a power that seeds a process, not a one-shot edit ([Other powers](https://worldbox-sandbox-god-simulator.fandom.com/wiki/Other_various_powers)).
- **Life / spawn** — drop creatures, kingdoms, plants; beehive→bees→pollination chains.
- **Nature & disasters** — lightning (ignites wood, heats pixels), lava (cools to rock), tornado, meteor, plague.
- **Destruction & chaos** — Gray Goo (devours land→deep ocean), Flame/Ice Towers (spawn demons/cold-ones), explosions.
- **Time** — pause/resume the world.

### Spawn-anything sandbox feel & the poke-the-world loop
Everything is a brush. Instant, mass-of-pixels feedback; no build menus or tech gating between the player and the world. Consequences are simulated, not scripted — set fire and it spreads by heat rules; spawn a kingdom and it wars/allies/schisms on its own.

### Emergent kingdoms, wars, religions
Civilizations form, expand, found cities, declare wars, spread religions and languages, and rise/collapse with zero authored plot. The world keeps a history/relations layer the player can browse.

### Inspect-anything
Click any unit/tile/kingdom for a detail panel: stats, traits, relations, history. This readability is what makes the emergence legible and fun.

## What it NAILS
- The **brush-everything palette** with categorized tabs and search.
- **Powers-as-processes** (ants, goo, beehives) — seeding a rule, not painting a result.
- **Instant tactile feedback** at the pixel/CA level.
- **Inspect-any-entity** turning emergence into a story you can read.
- **Pause/speed** as first-class god verbs.

## What to ADOPT for Civis
- Categorized, searchable, **data-driven god-tool palette** with tabs mirroring WorldBox's taxonomy. `[UI/QoL]` → FR-CIV-GODTOOL-900/901.
- **Powers that seed Layer-0 processes** (spawn DNA, ignite, seed biome) whose outcome emerges. `[UI/QoL]` input → `[EMERGENT]` result. FR-CIV-GODTOOL-911/912. Tension check: fine — the tool sets initial conditions; physics/genomics do the rest.
- **Inspect-anything + history browse.** `[UI/QoL]` → FR-CIV-INSPECT-900..903, FR-CIV-PSYCHE-920 (chronicle/legends).
- **Pause/speed + god-hand grab.** `[UI/QoL]` → FR-CIV-GODTOOL-920.
- Terrain/material brushes as superset of CS + WorldBox (already in `tool-design-directives.md`). `[UI/QoL]` → FR-CIV-GODTOOL-910.

## What to AVOID
- WorldBox **hardcodes "kingdoms/religions" as discrete objects**; Civis must keep these EMERGENT (cluster overlap, not `faction:u32`). Adopt the *feel*, not the data model.
- Don't let powers become scripted outcomes (e.g., a "win war" button) — every power must be a physical/biological initial condition.
- 2D pixel CA does not impose Civis's 3D-voxel/LOD constraints; don't copy its unbounded per-pixel sim assumptions at 20mi scale.

## Bevy / Rust ecosystem notes
Brush application maps to the `phenotype-voxel` dirty-queue; picking for inspect via `bevy_picking`; palette UI via egui/bevy_ui. See [bevy-ecosystem-reference](./bevy-ecosystem-reference.md).

## Sources
- WorldBox Powers — https://the-official-worldbox-wiki.fandom.com/wiki/Powers
- Other various powers — https://worldbox-sandbox-god-simulator.fandom.com/wiki/Other_various_powers
- WorldBox (NamuWiki overview) — https://en.namu.wiki/w/WorldBox%20-%20God%20Simulator
- PowerBox mod (palette extensibility) — https://www.curseforge.com/world-box/mods/powerbox
- Steam store page — https://store.steampowered.com/app/1206560/WorldBox__God_Simulator/
