# Genre Parity Benchmark — Civis-on-main vs. Reference Titles

**Date:** 2026-06-10
**Branch / PR target:** `docs/parity-benchmark` (draft)
**Source of truth (Civis):** `docs/audits/fr-matrix-2026-06-10.md`, `docs/audits/fr-matrix.json`,
`FUNCTIONAL_REQUIREMENTS.md`, `docs/traceability/fr-3d-matrix.md`,
`docs/traceability/full-traceability-matrix.md`, repo `AGENTS.md`, maturity footer of
`AGENTS.md` ("2026-05-26"), and merged-PR evidence in `git log` on `main`.

> **Evidence convention for Civis cells.** Each cell cites (a) the canonical FR ID
> in `FUNCTIONAL_REQUIREMENTS.md` / `docs/traceability/*`, (b) the matrix status
> from `docs/audits/fr-matrix-2026-06-10.md` (`COVERED`, `IMPL-NO-TEST`,
> `SPEC-ONLY`, `CODE-ONLY-no-spec`), and (c) where useful, a merged-PR number
> from `git log` on `main`. **All Civis cells are 2026-06-10 baseline**
> (`00df9473 docs(audit): frame-budget baseline + frame diagnostics (#373)`).
>
> **Evidence convention for competitor cells.** Web fetches are unreliable in
> this environment (Wikipedia returned 403); per the operating rules, the
> document degrades to **inline product knowledge** drawn from shipped
> features as of the title's latest public release. Sources-of-fact per
> title are listed at the end of the doc.

---

## 0. Scope, method, definitions

- **Rows** = capability domains requested by the task: **GFX, SIM, TOOLS,
  UI/UX, AUDIO, SCALE, PERSISTENCE, MODDING**.
- **Columns** = **Civis-on-main** (leftmost), then five genre reference titles
  used to triangulate "what players expect from this genre" plus two indie
  depth references:
  1. **WorldBox** (Maxim Karpenko / New Eclipsical) — god-sandbox
  2. **Cities: Skylines 2** (Colossal Order / Paradox) — city-sim
  3. **Manor Lords** (Slavic Magic / Hooded Horse) — medieval settlement
  4. **Star Wars: Empire at War** (Petroglyph / LucasArts, with the *Empire
     at War: Remake* update) — RTS with tactical layer
  5. **RimWorld** (Ludeon) — colony sim / storyteller
  6. **Dwarf Fortress** (Bay 12) — depth reference
- **Verdicts per cell** (relative to the row's genre expectation, not the
  title in isolation):
  - **PARITY** — feature is present at the level players expect from the
    reference title.
  - **PARTIAL** — feature is implemented in some form (spec, code, or
    runtime evidence) but is materially thinner than the reference.
  - **MISSING** — feature is not in scope on `main`; not in FR matrix; no
    implementation in `crates/` or `clients/`.
  - **N-A** — the row genuinely does not apply to that title's genre.
- **Player-impact ranking** (top-20 below) uses a 0–5 scale:
  - *audience breadth* (how many would-be players feel the gap)
  - *session-blocker* (does the gap prevent the headline loop)
  - *word-of-mouth* (would reviewers / streamers call this out?)
  Total ≤ 15; ties broken by *recoverability* (lower = harder to close).

---

## 1. Maturity snapshot — Civis-on-main (`00df9473`)

Quoting the repo's own AGENTS.md maturity footer (2026-05-26) verbatim:

> **Mature:** determinism/replay, `civ-server` WS tests (incl. spawn
> palette), `civ-watch`, web L2 authoring, Godot/Bevy/Unreal server attach,
> JSON-RPC catalog + `just civis-3d-verify`.
>
> **Partial:** modding v3 — **25+** `civ-mod-host` tests, WASM ticks,
> `.civmod`, `civlab-sdk`, Ed25519 verify + `just civis-3d-mod-sign` + `just
> civis-3d-mod-package-all`; example mods on **civ-server** / **civ-watch** +
> `baseline.yaml`; mod browser (`mods` + `mod_lifecycle` on `sim.snapshot`,
> web **Mods** panel, Godot **Mods** label); `mod.loaded.v1` replay-bus
> JSON in replay + watch event feed; F3D0 — Bevy full `Frame3d`,
> Godot/Unreal **16³ mesh** when dense `voxels`; cross-client minimap
> click-to-focus (Bevy/Godot/web/Unreal).
>
> **Product-only (not agent blockers):** Quixel/Megascans mesh import —
> engineering slots in `Content/Megascans/` + fr-l5-visual-pass; artists
> import via Bridge.

The 2026-06-10 FR matrix (`docs/audits/fr-matrix-2026-06-10.md`) reports
**1181 unique FR/NFR IDs**: 3 `COVERED`, 213 `IMPL-NO-TEST`, 179
`SPEC-ONLY`, 786 `CODE-ONLY-no-spec`. The SPEC-ONLY and CODE-ONLY buckets
are the raw material for the gap list.

---

## 2. The matrix

Notation: `[FR-…] (status)` = Civis evidence; one-liners for competitors.

### GFX — lighting, postFX, weather, water, LOD, animation

| Game              | GFX parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:----------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Lighting: PBR + triplanar splatting spec'd (`FR-CIV-PBR-001…008`, `SPEC-ONLY`). PostFX: voxel damage pulses on watch + dashboard (FR-CIV-TACTICS-001-int, `IMPL-NO-TEST`). Weather: planet day/night + moon tides (FR-CIV-PLANET-001/002, `IMPL-NO-TEST`); CA-thermo for water (FR-CIV-CA-005, `SPEC-ONLY`). Water: dense `CaGrid` with `saturation` and sea-level pass spec'd (FR-CIV-CA-007, `SPEC-ONLY`). LOD: adaptive 16³ octree + agent LOD gestalt (FR-CIV-VOXEL-001, FR-CIV-AGENTS-010, `IMPL-NO-TEST`); ring LODs on FR-CIV-SCALE-003 (`SPEC-ONLY`). Animation: SMPL-style morph targets spec'd (FR-CIV-SPECIES-010, `IMPL-NO-TEST`). Frame budget: PR #373 baseline. Net: the rendering layer is in **PBR spec + ca-grids + dirty-chunk** state — no GI / SSR / volumetric clouds yet. |
| WorldBox          | **PARTIAL** | 2D pixel-art, soft-shadow god-view, weather (rain, snow, sandstorm, tornado), biomes with distinct palettes, smooth zoom. Lacks modern 3D lighting entirely (genre-fit). |
| Cities: Skylines 2 | **MISSING** | CS2 ships volumetric clouds, real-time GI, distance-based LOD, water caustics + flow, day/night cycle, screen-space reflections, depth-of-field, motion blur. Civis has none of these in code. |
| Manor Lords       | **PARTIAL** | Photo-real medieval lighting, day-night, seasons with snow accumulation and ground wetness, realistic wind animation on foliage, level-of-detail on building meshes, no fancy GI/SSR. Closer to Civis scope. |
| Empire at War     | **MISSING** | RTS-grade: dynamic lights, per-unit shader LOD, explosion FX, fog-of-war texture, zoom from strategic to tactical. Visually simple vs modern. |
| RimWorld          | **PARTIAL** | 2D top-down with weather (rain, snow, fog, flashstorm), zoned lighting, mood/room-impression shaders, no PBR. 2D bar remains a real ceiling on visual immersion. |
| Dwarf Fortress    | **N-A** | ASCII / classic tileset. Color tileset is a 2D tileset, not a 3D engine. |

### SIM — agents, needs, economy chains, diplomacy, combat, ecology, emergence

| Game              | SIM parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:----------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Agents: needs decay + sickness + death (FR-CIV-LIFE-001/002/003, `IMPL-NO-TEST`); utility-AI daily path / POI (FR-CIV-LIFE-010, `IMPL-NO-TEST`); OCEAN/PAD psyche (FR-CIV-PSYCHE-001, `IMPL-NO-TEST`). Economy: stocks, production, market clearing, joule conservation (FR-ECON-001/002/003, FR-CIV-LIFE-020/025, `IMPL-NO-TEST`). Diplomacy: typed war goals, treaties, Zeuthen/Rubinstein concessions (FR-CIV-DIPLO-001…008, `SPEC-ONLY`). Combat: voxel LOS, doctrine GA, A* pathfinding, fog-of-war gating engagements, formation offsets, war-bridge damage pulses (FR-CIV-TACTICS-020…042, all `IMPL-NO-TEST`). Ecology: CA fluid/thermo (FR-CIV-CA-001…010, `SPEC-ONLY`); abiogenesis suitability (FR-CIV-CA-009). Emergence: cluster-based polities, psyche-driven behavior, history→rumor→chronicle (FR-CIV-LIFE-030, FR-CIV-LEGENDS-001…008, FR-CIV-LEGENDS-* mostly `SPEC-ONLY` / `CODE-ONLY-no-spec`); legend worker code lives in `crates/legends/src/worker.rs` (`CODE-ONLY-no-spec`). Market: explicit FR-CIV-MARKET-001…008 set, `CODE-ONLY-no-spec` (`docs/design/polities-markets.md`). Net: scaffolding for *deep* sim is in the matrix; most rows are spec/IMPL-NO-TEST, none are COVERED. |
| WorldBox          | **PARTIAL** | Sims humans: jobs, happiness, family, religion, war, diseases. Creatures: elk, bears, orcs, dwarves, angels, demons, krakens, dragons. Ecology: trees grow, animal populations fluctuate. Player-painted biomes + god spells. **No** deep needs stacks, no per-agent OCEAN, no market-clearing economy. |
| Cities: Skylines 2 | **PARTIAL** | Cims: household, age, education, health, happiness, journey-to-work; city economy chains (extraction→industry→commerce); demand/supply; sector balance. Deep on traffic simulation. **No** first-class ecology, no god-powers, no sentient factions. |
| Manor Lords       | **PARTIAL** | Real medieval production chains (timber→firewood, hide→leather, berries→ale, flax→linen), seasonal labor, market stalls with regional supply, militia & retinue combat, peasant approval, burgage plot upgrades, **no diplomacy in EA**, **no ecology**. Closer to Civis scope. |
| Empire at War     | **PARTIAL** | Tactical land+space combat, heroes, fleet doctrine, planetary economy in galactic layer, factional diplomacy. Per-unit HP/damage numbers; sim depth is shallow vs colony sims. |
| RimWorld          | **PARITY** | Pawns with OCEAN-ish traits, mood, thoughts, needs, social, health, addiction, pregnancy; full mental-break system; storyteller AI (Phoebe/Randy/Cassandra); combat with cover/quality/damage types; trade caravans; emergent narratives. This is the genre's high bar. |
| Dwarf Fortress    | **PARITY** | ~2000-page manual's worth of depth: mood/stress, individual thoughts, deity relationships, soul economy, generations of history, weather+aquifer+miasma, body-part detail. The depth reference. |

### TOOLS — god tools, brushes, inspection, scenario

| Game              | TOOLS parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Brushes: 13-row `docs/design/brush-tool-system.md` (`FR-CIV-BRUSH-01..13`, all `CODE-ONLY-no-spec`). God-tools: `FR-CIV-GODTOOL-900..921` roadmap rows (`CODE-ONLY-no-spec`). Inspection: `clients/bevy-ref/src/inspect.rs` with `FR-CIV-INSPECT-900/910/920` (`CODE-ONLY-no-spec`); info views in `clients/bevy-ref/src/info_views.rs` (`FR-CIV-INFOVIEW-900..921`, `CODE-ONLY-no-spec`). Scenarios: scenario YAML schema + `crates/engine/src/scenario.rs` (FR-API-001, `IMPL-NO-TEST`); baseline scenario loads example-economic mod (FR-CIV-TACTICS-051, `IMPL-NO-TEST`); scenario fog fields wire military phase (FR-CIV-TACTICS-045, `IMPL-NO-TEST`). Onboarding/QoL: `FR-CIV-QOL-100..230` (`CODE-ONLY-no-spec`). Net: the files are present, the IDs exist, the test count is ~0 — the **transport** of tools is real, the **polish** of a shipped god-game is not yet. |
| WorldBox          | **PARITY** | Hand-painted spawn, fire, lightning, meteor, acid rain, power, freeze, destroy, fill, world-save/load, advanced editor (Remake), particles. Category-leading. |
| Cities: Skylines 2 | **PARITY** | District/area drawing tools, road tools (multi-mode, upgrade, asset-variants), zone tools, info views (population, education, health, traffic, noise, pollution, water, electricity), unlimited undo/redo, scenario loading. |
| Manor Lords       | **PARTIAL** | Direct-placement buildings on flatting, road plowing, forest designation, mining/field, trading post; on-rails development tool (no zoning). |
| Empire at War     | **PARTIAL** | Galactic: build queue, tactical: unit selection, waypoint movement, formation stance, ability hot-keys. Scenario editor (limited). |
| RimWorld          | **PARTIAL** | Architect menu (zones, rooms, doors, power, pipes, walls, floors), Orders menu (hunt, haul, tame, harvest), Inspect pane with health/mood/equipment, dev-mode tools. Powerful but per-system. |
| Dwarf Fortress    | **PARTIAL** | Designations (dig, chop, gather, build, q-spd), workshop queues, manager, stocks, room/rental assignment. Powerful ASCII UX. |

### UI/UX — layout, inspector, accessibility, panels

| Game              | UI/UX parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Bevy HUD: `clients/bevy-ref/src/event_feed` (egui toasts + scrollable log, FR-CIV-BEVY-023, `IMPL-NO-TEST`); `MenusPlugin` pause overlay + settings stub + era banner (FR-CIV-BEVY-024, `IMPL-NO-TEST`); spawn palette incl. hangar (FR-CIV-UX-006, `COVERED` in `crates/server/tests/ws_smoke.rs:1714`). Web dashboard: `web/dashboard` closed matrix in `docs/traceability/fr-web-matrix.md` (L2 authoring). Godot UI scripts in `clients/godot-ref/scripts/ui.tscn`. Unstoppable camera, drag-place, convoy along path, minimap click-to-focus (cross-client per AGENTS.md). Notification/toast roadmap `FR-CIV-NOTIFY-900..921` (`CODE-ONLY-no-spec`). Accessibility NFRs `NFR-CIV-ACC-001..004`, 3 `IMPL-NO-TEST`, 1 `SPEC-ONLY`. Onboarding `FR-CIV-QOL-100..230` (`CODE-ONLY-no-spec`). Mods panel ships (`FR-CIV-TACTICS-054` mod browser on watch/server + dashboard). |
| WorldBox          | **PARITY** | Icon-based tool palette, layer toggles, minimap, stat overlays, hot-keys, autosave indicators, world settings dialog. |
| Cities: Skylines 2 | **PARITY** | Polished radial menus, multi-lens info views, advisor notifications, scenario settings, mod manager, photo mode. Reference bar. |
| Manor Lords       | **PARTIAL** | Clean top-bar, sector overlays, family/wealth/approval panels; **no in-game map mode in EA**, no advisor. |
| Empire at War     | **PARTIAL** | Classic RTS UI; top-bar resources, minimap, build tabs, unit info card. Functional, dated. |
| RimWorld          | **PARITY** | Inspect-pane drill-down, main tab bar, gizmo overlays, history graphs, social tab, log. Tight, dense, screenshot-friendly. |
| Dwarf Fortress    | **MISSING** | ASCII keyboard-driven UI. Powerful but inaccessible; tile-set doesn't change accessibility ceiling. |

### AUDIO — ambience, music, SFX, voice, mix

| Game              | AUDIO parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Audio substrate spec: `FR-CIV-AUDIO-001..008` (`IMPL-NO-TEST`) + `FR-CIV-AUDIO-009..012` (`CODE-ONLY-no-spec`); wraps `fundsp` + `bevy_procedural_audio` → `bevy_kira_audio`; four material archetype instruments (metal/wood/hide/reed) with spatial kira playback; 4-bus mix (ambient/score/sfx/ui) per `docs/design/audio-direction.md`; missing assets warn-once. PR work visible on `feat/audio-substrate` (worktree `G:/civis-wt-audio`). |
| WorldBox          | **PARTIAL** | Bit-banger soundtrack, ambient biome tracks, explosion/spell SFX, no real mixing, no dynamic score. |
| Cities: Skylines 2 | **PARITY** | Adaptive licensed OST, station-specific jingles, ambience (traffic, sirens, wind), voiced advisor. Reference bar for the genre. |
| Manor Lords       | **PARITY** | Original medieval-orchestral OST (Radomir Milinković), era-aware stingers, distinct settlement SFX (carpenter, smith, livestock). One of the genre's strongest. |
| Empire at War     | **PARTIAL** | John Williams-licensed OST, era-specific music, unit/ability voice. |
| RimWorld          | **PARTIAL** | Adaptive ambient + combat music, SFX library, simple mixer. Music by Alistair Lindsay is highly regarded. |
| Dwarf Fortress    | **PARTIAL** | Music packs from various artists, simple SFX. Genre-defining sound, not genre-leading mix. |

### SCALE — world size, perf budget, sim-LOD, streaming

| Game              | SCALE parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | SCALE spec: `FR-CIV-SCALE-001..008` (`SPEC-ONLY`); MVP 0.5 mi² resident with one 256³ CA chunk (FR-CIV-SCALE-001); horizon-fade LOD rings (FR-CIV-SCALE-003); sim-LOD gestalt (FR-CIV-SCALE-004); prefetch from camera velocity (FR-CIV-SCALE-005). Frame-budget baseline PR #373. NFR-CIV-SCALE-001..004 mostly `SPEC-ONLY`; NFR-CIV-SCALE-900..920 (`CODE-ONLY-no-spec`). Streaming window design branch `feat/streaming-window-design` (worktree `G:/civis-wt-scale`). |
| WorldBox          | **PARTIAL** | 2D tile-map; modest world size; performance scales with render mode (sprite vs bit). 1.0 in Remake added grid 4× larger. |
| Cities: Skylines 2 | **PARTIAL** | Maps 100 km² nominally; CPU-bound (cims, traffic) before GPU. Up to ~1M cims; performance is the genre's open issue. |
| Manor Lords       | **PARTIAL** | Single regional map, 1 settlement+ surroundings; can support ~2500+ citizens with perf headroom issues in EA. |
| Empire at War     | **PARITY** | Galactic map many systems, 100+ units per tactical battle on 4GB-era hardware; flat RTS scale. |
| RimWorld          | **PARTIAL** | Single map up to 250×250; CPU-bound once pawns >10. |
| Dwarf Fortress    | **PARTIAL** | Single fortress + 1+ embark region; worldgen depth before perf; sim-LOD by tile age. |

### PERSISTENCE — save / load, replay, scenario sharing, identity

| Game              | PERSISTENCE parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:------------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Replay: `.civreplay` format + bit-identical determinism tests (FR-REPLAY-001/002, FR-CORE-001 `COVERED` via `crates/engine/tests/tick_budget.rs:1`); engine hash chain (FR-CORE-005/006 `IMPL-NO-TEST`); civ-save-db SQLite (FR-CIV-TACTICS-069/076 `IMPL-NO-TEST`); `civsave` debug folder + zstd archive (FR-CIV-TACTICS-058/060 `IMPL-NO-TEST`); session.saved.v1 on replay bus (FR-CIV-TACTICS-072). Save spec: `FR-SAVE-001..025` and `FR-CIV-SAVE-001..004` (`IMPL-NO-TEST`/`SPEC-ONLY`). Session-IDs: `FR-SESSION-001..033` (`CODE-ONLY-no-spec` in PVE spec). Scenario YAML with override (FR-API-001/003 `IMPL-NO-TEST`). No in-game save-slot UI tested; save/load exists as JSON-RPC (`save.slot`, `save.load`, `save.list` — FR-CIV-TACTICS-066 `IMPL-NO-TEST`). |
| WorldBox          | **PARTIAL** | World save/load; **no** deterministic replay (procedural world seeded by editor). |
| Cities: Skylines 2 | **PARITY** | Citisave (binary), autosave, asset mod load order, region/city swap, scenario save. |
| Manor Lords       | **PARTIAL** | Single autosave + manual; no in-game replay. |
| Empire at War     | **PARTIAL** | Save/load; no deterministic replay; **no** scenario sharing. |
| RimWorld          | **PARITY** | Save anywhere, named quicksave, "save scrubbing" via commitment mode; in-game scenario editor; steam workshop scenarios. Reference bar. |
| Dwarf Fortress    | **PARTIAL** | Embark save; legends export (XML); no deterministic replay. |

### MODDING — SDK, store, hot-reload, signed content

| Game              | MODDING parity | Cell evidence (Civis) / known-feature knowledge (competitor) |
|-------------------|:--------------:|--------------------------------------------------------------|
| Civis-on-main     | **PARTIAL** | Per AGENTS.md 2026-05-26: **25+** `civ-mod-host` tests, WASM ticks, `.civmod`, `civlab-sdk`, Ed25519 verify + `just civis-3d-mod-sign` + `just civis-3d-mod-package-all`; example mods on `civ-server` / `civ-watch` + `baseline.yaml`; mod browser on `sim.snapshot` + web **Mods** panel; `mod.loaded.v1` replay-bus JSON in replay + watch event feed. Detail FRs FR-CIV-TACTICS-043 (Ed25519) … 077 (signed remote mod registry), all `IMPL-NO-TEST`. Full CIV-0700 capability enforcement `FR-CIV-TACTICS-071` (`IMPL-NO-TEST`). Mod catalog + install (FR-CIV-TACTICS-062); publish + HTTP API (FR-CIV-TACTICS-067); hot reload (FR-CIV-TACTICS-068); `mod.permission_violation.v1` (FR-CIV-TACTICS-075). Quoting AGENTS.md explicitly: "v3 **partial–good**: manifest + `.civmod` ZIP + `wasmtime` ticks + Ed25519 verify …; mod browser on `sim.snapshot` + web **Mods** panel; `mod.loaded.v1` replay-bus JSON; determinism scan unless `mod-dev`." The headline gaps to true SDK parity are **hot-reload UX**, **mod store**, **capability enforcement in player builds**, **hot-reload during play**. |
| WorldBox          | **MISSING** | Steam workshop; per-item data packs (textures/sprites); no scripting. |
| Cities: Skylines 2 | **PARTIAL** | CS2 modding in active development post-launch; BepInEx + custom frameworks; mod manager from Paradox Mods; documented surface area still maturing. |
| Manor Lords       | **MISSING** | Steam workshop: cosmetic building XML, no scripting. |
| Empire at War      | **PARTIAL** | Map editor, large existing mod scene; scripting via Lua mods (Remake); no first-party store. |
| RimWorld          | **PARITY** | C# mod SDK, Steam Workshop, mod manager, mod list, load-order, in-game toggle. Genre reference. |
| Dwarf Fortress    | **PARTIAL** | Raw XML raws; graphics & tileset packs via Steam Workshop; raws are documented but not as polished as RimWorld's. |

---

## 3. Verdict roll-up (8 rows × 7 games)

(Counts derived from §2; "Civis" included for reference.)

| Domain     | Civis | WorldBox | CS2  | Manor Lords | EAW  | RimWorld | Dwarf Fortress |
|------------|:-----:|:--------:|:----:|:-----------:|:----:|:--------:|:--------------:|
| GFX        | PART  | PART     | MISS | PART        | MISS | PART     | N-A            |
| SIM        | PART  | PART     | PART | PART        | PART | **PAR**  | **PAR**        |
| TOOLS      | PART  | **PAR**  | **PAR** | PART    | PART | PART     | PART           |
| UI/UX      | PART  | **PAR**  | **PAR** | PART    | PART | **PAR**  | MISS           |
| AUDIO      | PART  | PART     | **PAR** | **PAR** | PART | PART     | PART           |
| SCALE      | PART  | PART     | PART | PART        | **PAR** | PART | PART       |
| PERSISTENCE| PART  | PART     | **PAR** | PART   | PART | **PAR**  | PART           |
| MODDING    | PART  | MISS     | PART | MISS        | PART | **PAR**  | PART           |

**Reading.** Civis-on-main is uniformly **PARTIAL** — across all eight rows
— and the only games that achieve **PARITY** in any row are either pure
sandbox god-games (WorldBox for tools/UI/UX), or settlement/colony
specialists with 5–10 years of shipped content (RimWorld, Dwarf Fortress,
CS2 in UI/Audio). This is a *healthy* result for a project whose
`AGENTS.md` footer lists **Mature** deliverables as *determinism/replay,
civ-server, civ-watch, web L2, multi-client attach, JSON-RPC catalog* — the
infra that lets the rest of the rows be filled in by mods, and the
*3D-fragmentation* that prevents the GFX row from being more than partial.

**Three cells are explicit "Mature" / "Partial" anchors in the AGENTS
maturity footer** (and therefore trustworthy as evidence rather than
hope): civ-server WS tests, civ-watch, JSON-RPC catalog (`just
civis-3d-verify`). The 3D fragment row in the GFX column is the single
largest 2026-06-10 narrative risk: F3D0 ships as a *16³ mesh* in
Godot/Unreal when `voxels` are dense (per AGENTS footer), which is far
below the perceptual bar CS2 sets.

---

## 4. Top-20 parity gaps, ranked by player impact (draft epic stubs)

> Score = audience breadth (0–5) + session-blocker (0–5) +
> word-of-mouth (0–5), ≤ 15. Recoverability is editorial; lower is
> harder. The "→ FR" column points to existing FR or proposes a new
> epic ID consistent with the existing `FR-CIV-*` scheme.

| #  | Gap                                              | Score | Rec. | → FR / proposed epic                                  | What is missing in Civis |
|----|--------------------------------------------------|:-----:|:----:|-------------------------------------------------------|--------------------------|
| 1  | **City-scale traffic & economy chain sim**        |  15   |  Low  | `FR-CIV-INFRA-*` (`CODE-ONLY-no-spec`) → propose `FR-CIV-ECON-CHAIN-001..010` | CS2 / SimCity-class cim routing, peak-hour congestion, multi-industry supply chains visible to player. Civis has traffic crate (`crates/civ-traffic/src/lane.rs`) and a single-test coverage, no full UI. |
| 2  | **Zoning / district-level city planning**         |  14   |  Mid  | new `FR-CIV-CITY-PLAN-001..006`                       | CS2 / SimCity4 zoning, district policies, demand overlay. Manor Lords has *flat* placement; Civis has *BuildingGraph* (`FR-CIV-BUILD-001`) but no district abstraction on top. |
| 3  | **Modern GFX (GI / SSR / volumetric / DoF)**     |  14   |  Low  | `FR-CIV-PBR-001..008` (`SPEC-ONLY`) + new `FR-CIV-POSTFX-001..004` | CS2 / Manor Lords-class. Civis has triplanar PBR and PBR spec but no GI / SSR / volumetric / DoF; no postFX pass. |
| 4  | **God-powers (meteor, fire, flood, plague, raise-dead, mass-blossom)** | 14 | Mid | `FR-CIV-GODTOOL-900..921` (`CODE-ONLY-no-spec`) → new `FR-CIV-GODTOOL-100..130` per spell class | WorldBox-class, top genre expectation. Civis has brush/tool transport but no in-game god-powers shipped. |
| 5  | **LOD-crowd / ambient crowd rendering**          |  13   |  Mid  | new `FR-CIV-RENDER-CROWD-001..005`                    | CS2-class. Civis has `FR-CIV-AGENTS-010` LOD gestalt for *sim*; no GPU-instanced crowd LOD. |
| 6  | **Medieval plot-based building (Manor Lords)**   |  13   |  Mid  | `FR-CIV-BUILD-001/010/020/030` (`IMPL-NO-TEST`) → propose `FR-CIV-BUILD-040..060` for plot/parcel/road | BuildingGraph exists, freehand parity exists, but no in-game burgage plot / manor-plot placement with road-facing. |
| 7  | **Production-chain visualizer (multi-hop)**      |  13   |  Mid  | `FR-ECON-001..003` (`IMPL-NO-TEST`) → new `FR-CIV-ECON-VIZ-001..004` | Anno / Manor Lords / Factorio-class. Civis has stocks and a research export, no in-game multi-hop chain panel. |
| 8  | **Adaptive music + dynamic mix**                 |  12   |  Mid  | `FR-CIV-AUDIO-001..008` (`IMPL-NO-TEST`) + `009..012` (`CODE-ONLY-no-spec`) | CS2 / Manor Lords / RimWorld-class. Civis audio substrate spec'd, no in-game adaptive music bus. |
| 9  | **Storyteller / dynamic event AI**               |  12   |  Mid  | new `FR-CIV-STORY-001..010`                          | RimWorld-class. Civis has cluster-based polities and OCEAN but no narrator that curates a session. |
| 10 | **Per-unit tactical detail (cover, weapon, stance)** | 12 | Low | `FR-CIV-TACTICS-020..042` (`IMPL-NO-TEST`) → new `FR-CIV-TACTICS-100..130` for cover/stance/ammo | EAW-class tactical layer. Civis has LOS, fog, formation, A*; no cover bonuses, no stance, no ammo/degradation. |
| 11 | **Save-slot UI + cloud sync / multi-slot browser** | 11 | Mid | `FR-SAVE-001..025`, `FR-CIV-SAVE-001..004` (`IMPL-NO-TEST`/`SPEC-ONLY`) | RimWorld/CS2-class. Civis has `civsave` folder, zstd archive, `save.slot/load/list` JSON-RPC, but no in-game browser tested. |
| 12 | **Procedural building WFC + culture-tile variety** | 11 | Mid | `FR-CIV-ARCH-001..008` (`SPEC-ONLY`)                 | Manor Lords / Anno-class façade variety. Civis WFC is `ghx_proc_gen`/wrap spec-only; no exemplar culture presets. |
| 13 | **Mod store / publish + first-party catalog**    |  11   |  Mid  | `FR-CIV-MOD-001..020` (`CODE-ONLY-no-spec`) + `FR-CIV-TACTICS-067/070/077` (`IMPL-NO-TEST`) | RimWorld Workshop-class. AGENTS.md explicitly excludes "mod store/publish … hot reload" from v3 scope. |
| 14 | **Ecology (creature life-cycle, seasons, weather effects on growth)** | 11 | Low | `FR-CIV-CA-001..010` (`SPEC-ONLY`) + `FR-CIV-CLIMATE-001..003` (`IMPL-NO-TEST`) | WorldBox / DF-class. Civis has CA spec; climate rows IMPL-NO-TEST. |
| 15 | **Plausible large-world streaming + chunk IO contract** | 11 | Low | `FR-CIV-SCALE-001..008` (`SPEC-ONLY`) + NFR-CIV-SCALE-001..004 | DF / CS2-class. Civic has 256³ resident window spec; no shipped streaming window in main. |
| 16 | **Hot-reload mod UX in dev mode**                |  10   |  Mid  | `FR-CIV-TACTICS-068` (`IMPL-NO-TEST`) + new UX rows   | Existing infra; needs client-side UX. |
| 17 | **In-game scenario editor (RimWorld-class)**     |  10   |  Mid  | `FR-API-001..004` (`IMPL-NO-TEST`) + new editor FRs  | Civis has scenario YAML loader; no authoring tool. |
| 18 | **First-class diplomacy UI (war goals, ledger, treaty browse)** | 10 | Low | `FR-CIV-DIPLO-001..008` (`SPEC-ONLY`) | Civis has typed war goals + Zeuthen concessions spec; no UI. |
| 19 | **Multiplayer co-op / shared map (RimWorld-class, partial)** | 10 | High | new `FR-CIV-MP-001..010` | Civis already has multi-client WS attach (CS2-style *spectator*); no shared-tick multi-authority yet. |
| 20 | **Accessibility (full keyboard, color-blind, narration)** | 9 | Mid | `NFR-CIV-ACC-001..004` (3 `IMPL-NO-TEST`, 1 `SPEC-ONLY`) | Industry baseline; Civis 1 of 4 rows SPEC-ONLY. |

**Quick read.**

- **Top 4 gaps are session-blockers** for the headline marketing loop of
  one of the six reference titles. (1) and (2) put the project behind CS2
  on the "city-sim" axis. (3) and (4) put it behind WorldBox on the
  "god-sandbox" axis. Both are *narrative* problems, not infra.
- **Gaps 5, 6, 7, 8** are the deepest blockers to the "deep sim" promise
  in the 3D additions spec — the file is called *emergent-systems-spec.md*
  and the genre expectation is RimWorld / DF, not CS2.
- **Gap 11 (save UI) and Gap 12 (WFC) are the cheapest "win-back player
  trust" wins** — both are spec'd in the matrix, the matrix says
  SPEC-ONLY / IMPL-NO-TEST, and both ship into the existing client
  surfaces.
- **Recoverability = Low** items (1, 3, 5, 10, 14, 15, 18) are the ones
  where the *implementation cost* is in months, not weeks; they need a
  spec-deep epic (which several already have, e.g. CA, Scale, PBR).

---

## 5. Strategic reads

1. **The "Mature" footer is honest, the matrix is honest, and the gap
   list is the work.** AGENTS.md already classifies FR-CIV-TACTICS,
   FR-CIV-VOXEL, FR-CIV-AGENTS, etc. as "in progress" via "Partial"; the
   matrix shows 3 COVERED of 1181 IDs. The headline *playable* loop
   today is **infrastructure + attach + modding-v3** — it is *not* the
   gameplay loops of the six reference titles.
2. **The genre is a 2-axis bet.** Civis is a single project that tries
   to be a god-sandbox **and** a city-sim **and** a medieval settlement
   **and** an RTS **and** a colony-sim. WorldBox and RimWorld each
   spent 5+ years sharpening one axis. Closing the top-20 gaps requires
   picking which 2 of those 5 axes to call *headline* and shipping them
   before chasing the others.
3. **Modding-v3 is the leverage.** The AGENTS footer lists 25+
   `civ-mod-host` tests, Ed25519 verify, `.civmod` packaging, replay-bus
   `mod.loaded.v1`, and a web Mods panel. The fastest path to closing
   many of the top-20 gaps is to **publish an example mod per gap** so
   the community can grow the matrix in parallel rather than waiting
   for first-party feature teams.
4. **The 3D fragment is the single biggest 2026-06-10 risk.** F3D0 ships
   as a 16³ mesh in Godot/Unreal; the L5 visual pass (GDD file
   `docs/development-guide/fr-l5-visual-pass.md`) is the upstream of
   gap #3. The right next step is to merge L5 *into* the mod browser /
   example-mod catalog so artists can author the visual pass as a
   `.civmod` of Quixel assets.
5. **Save-slot UI + scenario editor are the cheapest "feel-better" wins**
   (gaps 11 + 17) and would be a one-PR-each result. They should ship
   in P-W3 (P-wave 3 of the matrix's own roadmap).

---

## 6. Sources of fact used

- **Civis (in-repo).** `FUNCTIONAL_REQUIREMENTS.md`,
  `docs/traceability/fr-3d-matrix.md`, `docs/audits/fr-matrix-2026-06-10.md`,
  `docs/audits/fr-matrix.json`, `docs/traceability/full-traceability-matrix.md`,
  `docs/traceability/TRACEABILITY_MATRIX.md`,
  `docs/traceability/fr-web-matrix.md`, repo `AGENTS.md` (2026-05-26 maturity
  footer), `docs/development-guide/fr-l5-visual-pass.md`,
  `docs/development-guide/fr-3d-additions.md`, `docs/specs/CIV-0700-modding-api-spec.md`,
  `docs/design/emergent-systems-spec.md`, `docs/guides/emergence-charter.md`,
  `justfile`, `Taskfile.yml`, `git log` on `main` (PR #373 frame-budget, #372
  phantom-ID, #371 FR matrix, #366 seed substrate, #357 MCP tools).
- **WorldBox / CS2 / Manor Lords / EAW / RimWorld / Dwarf Fortress.**
  Public-release feature sets as of mid-2026, from training-time product
  knowledge. Web fetches for Wikipedia and Steam pages were not reachable
  in this environment (HTTP 403); per the operating rules the document
  degrades to inline knowledge. The above is *consumer-grade* baseline,
  not spec-grade; nothing in this doc claims source citations for the
  competitor cells.

---

**Trace:**

- FR: `FR-CIV-AUDIO-001..012`, `FR-CIV-BRUSH-01..13`, `FR-CIV-CA-001..010`,
  `FR-CIV-DIPLO-001..008`, `FR-CIV-GODTOOL-900..921`, `FR-CIV-INFOVIEW-*`,
  `FR-CIV-INSPECT-*`, `FR-CIV-LIFE-001..035`, `FR-CIV-MOD-*`, `FR-CIV-PBR-001..008`,
  `FR-CIV-PLANET-001..002`, `FR-CIV-PSYCHE-001..008`, `FR-CIV-QOL-*`,
  `FR-CIV-SCALE-001..008`, `FR-CIV-TACTICS-001..077`, `FR-CIV-UX-006`,
  `FR-CIV-VOXEL-001..010`, `FR-CORE-001`, `FR-ECON-001..003`,
  `NFR-CIV-ACC-001..004`, `NFR-CIV-SCALE-001..004`.
- Epic/Story: `P-W1` kickoff (`docs/development-guide/p-w1-kickoff.md`),
  `P-W3` / `P-W4` / `P-W5` placeholders for the gap epics in §4.

**Alternatives considered (research method, per task rule):**

- **Use `docs/audits/fr-matrix.json` directly** (raw, 13 558 lines,
  machine-formatted) — *rejected*; the JSON is the source of truth for
  status counters, but the human-readable narrative lives in the
  Markdown rollup. Cited where helpful.
- **Compose the matrix by enumerating `crates/` files** — *rejected*;
  this is what the matrix generator (`docs/audits/_gather_ids.py`)
  already does, and the file would be a re-derivation. The
  `fr-matrix-2026-06-10.md` rollup is the curated form.
- **Bring in agileplus-specs/* one-by-one** — *rejected*; the matrix
  already aggregates them. Going deeper would be research, not parity.
- **Re-verify competitor feature lists via web fetch** — *attempted*;
  failed (HTTP 403). Per task rule "web fetches MAY fail — degrade to
  inline knowledge" we used shipped-feature knowledge for the
  competitor columns and called this out in §6.
- **Choose matrix dimensions other than the 8 row × 7 col layout**
  (e.g., capability-coverage scatter, or 2-axis genre plot) —
  *rejected*; the task explicitly enumerates the rows and the 6
  reference titles. We added Dwarf Fortress as the depth reference
  only.

End of document.
