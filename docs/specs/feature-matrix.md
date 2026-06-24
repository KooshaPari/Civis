# Civis Feature Matrix — Master Gap Map

**Status:** Living document. Owned by Research & Spec Lead.
**Purpose:** Map every capability area against (a) the reference-game bar, (b) current Civis state (honest), and (c) the target. This is the master gap map every domain Lead works against.
**Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only physical/environmental/genomic laws are authored; everything else EMERGES. Tagging below:
- **[LAW]** authored Layer-0 rule (physics/chemistry/climate/genomics).
- **[EMERGENT]** must arise from laws, never hardcoded as an enum/script.
- **[UI/QoL]** presentation/tooling affordance — always allowed, not simulation.

## State legend
- **BLIND** — capability absent or unmeasured; we cannot even see/inspect it.
- **INCOMPLETE** — partial substrate exists; key pieces missing.
- **UNPOLISHED** — works mechanically but lacks the QoL/bells-and-whistles bar.
- **SOLID** — at or near the reference bar.

Current-state assessments derive from the crate set (`crates/{voxel,planet,laws,genetics,species,agents,economy,tactics,engine,watch,protocol-3d,...}`) and existing `docs/specs/CIV-0001..1000`. Where a domain Lead disputes an assessment, update the cell and cite the crate/spec.

---

## 1. World & Substrate

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Worldgen (terrain/biomes/hydrology) | CS2 real-world heightmaps; WorldBox procedural biomes; DF mineral/aquifer strata | CS, WorldBox, DF | `civ-planet` geology+biomes+weather; `civ-watch` procedural heightmap | INCOMPLETE | Deterministic procedural 20mi×20mi worldgen w/ strata, aquifers, climate bands | [LAW] |
| Voxel material/fluid CA | PowderToy element/reaction depth; Teardown destructibility | PowderToy, Teardown | `phenotype-voxel` SVO+dense leaf chunks + dirty queue; `civ-voxel` adapter | INCOMPLETE | Full gravity liquids/powders/gases/solids, mass-conserving, thermal/pressure | [LAW] |
| Climate/weather/day-night/tides | CS2 (wind/water flow overlays); Civ seasons | CS2 | `civ-planet` weather + day/night + tides | UNPOLISHED | Insolation-driven weather visible in overlays; seasonal economy hooks | [LAW] |
| Chemistry/energy/materials DB | PowderToy reactions; Factorio recipe graph | PowderToy | `civ-laws` RON law DB + validator; `civ-economy` joule model (CIV-0107) | SOLID | Conservation-complete; mod-friendly RON | [LAW] |
| Scale: 20mi×20mi streaming/LOD | Songs of Syx (empire scale); CS2 (tile streaming); big_space | SoS, CS2 | SVO + chunk streaming + LOD + frustum cull (built); two-zoom LOD (CIV-0101) | INCOMPLETE | Disk-bound active-set streaming, 60fps, LOD-tiered far sim | [LAW]+NFR |

## 2. Life, Mind & Society

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Genomics (DNA/mutation/speciation) | Spore creature DNA; real GA | Spore | `civ-genetics` ChaCha8 deterministic DNA/mutation/recomb/fitness/speciation | SOLID | Hamming-distance speciation thresholds | [LAW] |
| Species/phenotype expression | Spore creature creator; WorldBox races | Spore, WorldBox | `civ-species` deterministic DNA→Morphology/BehaviorWeights | SOLID | Many viable body/mind plans, no humanoid enum | [EMERGENT] |
| Paths to sentience | Spore stage gates (scripted — AVOID); B&W creature learning | Spore, B&W | none explicit | BLIND | Sentience as crossable cognitive/genomic threshold, measured not given | [EMERGENT] |
| Per-agent needs | RimWorld (food/rest/recreation/comfort); SoS at scale; CS2 cims | RimWorld, SoS | `civ-agents` Needs (food/shelter/safety/belonging) utility-AI; emergent needs decay/sickness/death (recent commit) | INCOMPLETE | Full Maslow-ish need stack, LOD-tiered | [EMERGENT] |
| Psyche (drives/temperament/memory/mood) | RimWorld thoughts/moods/mental breaks; DF | RimWorld, DF | BehaviorWeights only | BLIND | Per-agent temperament+memory+belief shaping divergence | [EMERGENT] |
| Social networks (kinship/relationships/grudges) | DF relationships+histories; RimWorld social | DF, RimWorld | none explicit | BLIND | Emergent kinship/contact graph; grudges/bonds | [EMERGENT] |
| Ideology/culture/language drift | Civ culture/religion spread; DF myths/legends | Civ, DF | `civ-engine` ideology metrics (CIV-0106) | INCOMPLETE | Belief/norm/language drift over contact networks; dialects/creoles | [EMERGENT] |
| Emergent histories/legends/myths | DF legends mode (gold standard) | DF | event log only | BLIND | Recorded emergent history queryable as legends/chronicle | [EMERGENT]+[UI/QoL] |

## 3. Economy, Polity & Civilization

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Markets (multiple types) | Civ trade; CS2 supply chains; Manor Lords regional trade | Civ, CS2, ML | `civ-economy` double-entry ledger + allocation (CIV-0100) | INCOMPLETE | Gift/barter/commodity/credit/planned emerging from local conditions | [EMERGENT] |
| Production/supply chains | Factorio; CS2 industry; Anno | Factorio, CS2 | `civ-economy` district production + joule budget (CIV-0107) | UNPOLISHED | Conservation-complete chains visible in overlays | [EMERGENT] |
| Polities/states (decentralized) | Civ (hardcoded — AVOID); Old World dynasties; WorldBox kingdoms | Old World, WorldBox | war/diplomacy shadow (CIV-0105); institutions (CIV-0103) | INCOMPLETE | Emergent cluster-overlap membership, NOT faction:u32; anarchic/networked/tributary forms valid | [EMERGENT] |
| Diplomacy/AI agendas | Civ VI agendas; Old World | Civ, Old World | shadow diplomacy (CIV-0105) | INCOMPLETE | Emergent stances from payoff/kinship/culture | [EMERGENT] |
| Taxation/budget/policies | CS2 budget panel + district policies | CS2 | economy ledger | UNPOLISHED | Player-set + emergent fiscal levers w/ budget UI | [UI/QoL]+[EMERGENT] |
| Tech/research progression | Civ tech tree (hardcoded — AVOID); discovered laws | Civ, Old World | `civ-laws` era-unlock prereqs + `civ-research` card validator (ADR-006) | INCOMPLETE | Tech = discovered laws gated by measurable prereqs, NOT a fixed tree | [EMERGENT]+[LAW] |

## 4. Architecture, Infrastructure & Engineering

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Architecture (buildings emerge + placeable) | Manor Lords burgage plots (organic); CS2 zoning | ML, CS2 | building graph (procedural+freehand) in `civ-protocol-3d` | INCOMPLETE | Agents build from need+resource; user can place; shared data tags | [EMERGENT]+[UI/QoL] |
| Roads/desire-paths | Manor Lords desire-lines; CS2 road tools (gold for tooling) | ML, CS2 | none explicit | BLIND | Roads form along desire-paths; user road tools w/ curves/snap/upgrade | [EMERGENT]+[UI/QoL] |
| Vehicles/transport | CS2 traffic AI; CtA vehicle crews | CS2, CtA | none explicit | BLIND | Emergent vehicles/agents on roads; transport overlays | [EMERGENT] |
| Zoning/districts | CS2 zoning paint + districts+policies | CS2 | none explicit | BLIND | District designation (UI) over emergent land-use | [UI/QoL] |

## 5. Warfare (Strategic / Operational / Tactical)

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Strategic layer | EAW:FoC galactic map; Civ war | EAW, Civ | war/diplomacy shadow (CIV-0105) | INCOMPLETE | Strategic posture emerging; map UI | [EMERGENT]+[UI/QoL] |
| Operational maneuver | EAW reinforcement; CtA operational | EAW, CtA | none | BLIND | Supply/logistics-aware maneuver | [EMERGENT] |
| Tactical (per-soldier, destructible) | Call to Arms direct-control + cover; Men of War | CtA | `civ-tactics` per-soldier voxel-destructible combat + doctrine GA + fog-of-war | INCOMPLETE | RTS command + optional direct control; logistics/ammo | [EMERGENT]+[UI/QoL] |
| Doctrine evolution | — (novel) | — | `civ-tactics` doctrine_fitness GA | SOLID | Doctrines evolve via GA | [EMERGENT] |

## 6. God-Tools & Interaction

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| God-tool palette | WorldBox ~374 powers / 8 tabs (gold); B&W hand | WorldBox, B&W | terraform + material tools (tool-design-directives.md, recent) | INCOMPLETE | Rich power palette: spawn-anything, terraform, disasters, material brush | [UI/QoL] |
| Spawn-anything sandbox feel | WorldBox; Garry's Mod | WorldBox | partial (place_voxel/spawn_civilian/damage via civ-watch control) | INCOMPLETE | Poke-the-world loop w/ instant feedback | [UI/QoL] |
| Inspect-anything | WorldBox entity inspect; DF unit screen | WorldBox, DF | none explicit | BLIND | Click any voxel/agent/settlement → full inspector | [UI/QoL] |
| God-hand interaction metaphor | B&W hand gestures | B&W | none | BLIND | Direct manipulation cursor/hand affordance | [UI/QoL] |

## 7. Camera, Controls, UI/UX & Info-Views

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Strategic↔tactical camera | EAW two-layer zoom; CS2 pan/zoom/tilt | EAW, CS2 | two-zoom LOD (CIV-0101) | UNPOLISHED | Seamless god↔city↔tactical zoom, smooth | [UI/QoL] |
| Info-view overlays | **CS2 ~33 overlays** (gold standard) | CS2 Info views wiki | minimal HUD (bevy-ref game_ui) | **BLIND** | Full overlay suite: pollution/land-value/happiness/wealth/services/traffic/resources/etc. | [UI/QoL] |
| Selection/picking | CS2; RTS standard | CS2 | partial | INCOMPLETE | Click/drag/box-select agents+buildings+voxels | [UI/QoL] |
| Tooltips/inspector panels | RimWorld clarity; CS2 | RimWorld, CS2 | minimal | BLIND | Rich contextual tooltips + detail panels | [UI/QoL] |
| Undo/blueprints/copy | CS2 (lacks undo — pain); Manor Lords | CS2 | none | BLIND | Undo, blueprint copy/paste for god-tools | [UI/QoL] |
| Notifications/alerts | CS2 chirper/alerts; RimWorld letters | CS2, RimWorld | none | BLIND | Event feed + alert routing + camera-jump | [UI/QoL] |
| Stats/graphs panels | SoS graphs; CS2 statistics; Old World | SoS, CS2 | timeseries (CIV-0103); egui_plot available | INCOMPLETE | Population/economy/ideology time-series dashboards | [UI/QoL] |
| Onboarding/tutorial | CS2; RimWorld scenario | CS2 | none | BLIND | Progressive disclosure, tooltips, first-run flow | [UI/QoL] |
| Hotkeys/control scheme | RTS standard; CS2 | CS2 | none documented | BLIND | Full rebindable hotkey map | [UI/QoL] |

## 8. Audio, Persistence, Modding, Performance

| Area | Reference bar | Reference source | Civis now | State | Target | Tag |
|---|---|---|---|---|---|---|
| Audio (adaptive) | B&W; CS ambient; kira | B&W | adaptive music via kira (RND-007); `civ-audio` (CIV-0800) | INCOMPLETE | Adaptive score + spatial SFX + UI juice | [UI/QoL] |
| Save/load/replay | CS2 saves; deterministic replay | CS2 | `civ-save-db`; persistence (CIV-1000); determinism guide | INCOMPLETE | Deterministic save + bit-identical replay | [LAW]+[UI/QoL] |
| Modding API | CS Workshop (gold); WorldBox mods; RimWorld XML | CS, RimWorld | `civ-mod-host` sandbox; modding (CIV-0700); RON laws mod-friendly | INCOMPLETE | Safe sandboxed code+data mods; law/material/asset mods | [UI/QoL] |
| Performance/LOD/scale | SoS 100k+ agents; CS2 | SoS | LOD tiers (Hot/Warm/Cold); perf spec (CIV-0500) | INCOMPLETE | 60fps @ 20mi, 100k+ LOD-tiered agents | NFR |

---

## Headline gaps (most BLIND / unpolished)
1. **Info-view overlay suite — BLIND.** The single biggest "feels blind" gap. CS2 ships ~33 overlays; Civis has a minimal HUD. Highest-leverage QoL fill.
2. **Inspect-anything + tooltips + god-hand — BLIND.** Cannot click the world to understand it. WorldBox/DF/RimWorld all nail this.
3. **Psyche + social networks + emergent histories — BLIND.** Substrate (genetics/species/needs) exists but mind/society/legends layer is unmeasured.
4. **Roads/desire-paths + vehicles + architecture emergence — BLIND/INCOMPLETE.** The "city-builder" core verbs.
5. **Notifications/undo/blueprints/onboarding — BLIND.** Standard bells-and-whistles entirely absent.

See [`requirements/`](./requirements/) for FR/NFR derived from these gaps and [`backlog.md`](./backlog.md) for the prioritized waves.
