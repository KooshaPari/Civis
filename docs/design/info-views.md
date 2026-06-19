# Civis Info-View Overlay Suite — Full CS2-Class Spec

> **Status:** Design spec (2026-05-30). Owned by Design (Planner stance — specs/AC/pseudocode only, no implementation code).
> Companion to [`docs/research/competitive-benchmark.md`](../research/competitive-benchmark.md) (legibility is the #1 credibility gap), [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) (only laws authored; everything else emerges → overlays *measure*, never *define*), and the SOTA survey [`docs/research/sota-tech/`](../research/sota-tech/).
> Implements against the existing data-driven registry in [`clients/bevy-ref/src/info_views.rs`](../../clients/bevy-ref/src/info_views.rs) (7 overlays today).

---

## 1. Why this exists (the credibility thesis)

Per the competitive benchmark §5, **legibility — the CS2-style info-view overlay suite + inspect-anything — is the single biggest credibility gap, outranking even the visual gap.** Civis's entire pitch is *emergent depth*; emergence that cannot be *seen* is worth zero. Cities: Skylines 2 ships **~33 info-view overlays** as the legibility gold standard. This spec catalogs a **CS2-class suite of 31 overlays** (matching CS2's bar) across six groups, and specifies the data-driven registry so each new overlay is a *registration*, not a code fork.

**Charter alignment (non-negotiable):** Overlays are **read-only measurements** over the emergent substrate. An overlay may *visualize* a faction, a culture, a market type, or a language — but it must **derive that classification at render time from emergent cluster/field data**, never from an authored enum. Where the underlying field is BLIND/INCOMPLETE today (per the feature matrix), the overlay is specified now and gated behind a data-availability flag (§7), so it lights up the moment the producing crate surfaces the field — no overlay rework.

---

## 2. Design principles

1. **Data-driven registry.** Every overlay is an `InfoOverlay` value (id, name, group, legend, data-source descriptor, sampler, color function). Adding "soil fertility" = appending one registration + one sampler. This pattern already exists in `info_views.rs`; this spec extends it (§6) with `group`, `data_source`, `render_kind`, `availability`, and `interactions`.
2. **Separation of sample → classify → color.** The renderer samples a field per lattice cell (or per entity), the overlay's pure function maps the sample to a legend position, the legend maps position to color. The color function is the *only* thing that differs between overlays.
3. **Legend is mandatory.** Every overlay (except categorical/distinct-color ones) ships an ordered `&[LegendStop]` ramp with human labels. Categorical overlays (territory, culture, language) declare `LegendKind::Categorical` and render a "distinct color per cluster" note plus a top-N swatch key.
4. **Three render kinds, one registry.** `RenderKind::LatticeRecolor` (heatmap over the terrain lattice — the default, reuses today's gizmo grid), `RenderKind::Gizmo` (lines/arrows/borders — roads, traffic flow, territory borders, trade routes), `RenderKind::EntityTint` (recolor agents/structures directly — happiness, health). The renderer dispatches on `render_kind`; new render kinds are additive.
5. **Charter-safe classification.** Categorical overlays hash an *emergent cluster id* to a stable color (`cluster_color`, already implemented). They never read an authored taxonomy.
6. **Wrap, don't hand-roll.** Color ramps reuse the existing `ramp_color`; plots/time-series (where an overlay carries a trend) use `egui_plot` (already a dependency per the benchmark). No bespoke charting.

---

## 3. The overlay catalog (31 overlays, 6 groups)

Each row: **data source** (producing crate + field), **legend/ramp**, **render kind**, **interactions**. "Field" names are the *contract* the producing crate must expose to the overlay sampler (a thin read-only accessor), not new sim state.

Availability legend: **LIVE** = data exists today (terrain/sim already computed) · **NEAR** = producing crate exists, field needs a read accessor · **BLIND** = field is INCOMPLETE/absent in the producing crate; overlay specified + gated (§7).

### Group A — Terrain / Environment (8)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| A1 | **Elevation** | `terrain` · `terrain_height` → `height_norm` | RAMP_ELEVATION (deep→coast→lowland→highland→peak) | LatticeRecolor | hover = exact metres above sea level | LIVE |
| A2 | **Water / Hydrology** | `terrain` · `WATER_LEVEL` vs height; `planet` · flow accumulation | RAMP_WATER (dry→submerged); add `RAMP_FLOW` for river strength | LatticeRecolor | hover = depth / flow m³s⁻¹; click cell = watershed highlight | LIVE (depth) / NEAR (flow) |
| A3 | **Temperature** | `planet` · insolation+lapse (today: terrain proxy lapse+latitude) | RAMP_TEMPERATURE (cold→temperate→hot) | LatticeRecolor | hover = °C; time-of-day/season scrub if `planet` exposes clock | LIVE (proxy) / NEAR (planet) |
| A4 | **Precipitation / Humidity** | `planet` · weather (rainfall, humidity field) | `RAMP_MOISTURE` (arid tan → wet blue-green) | LatticeRecolor | hover = mm/yr; toggle rain-now vs annual-mean | NEAR |
| A5 | **Biome** | `species`+`planet` · emergent biome = phenotype-cluster × climate band (NOT an enum) | `LegendKind::Categorical` (distinct per emergent biome cluster) + top-N key | LatticeRecolor | hover = dominant flora/fauna species ids; click = biome stat card | NEAR (derive from climate+species clusters) |
| A6 | **Material / Surface** | `terrain`/`voxel` · top-voxel material band | RAMP_MATERIAL (water→sand→grass→rock→snow) | LatticeRecolor | hover = material name from `laws` material DB | LIVE |
| A7 | **Soil Fertility** | `planet`+`laws` · moisture × material × insolation → growth potential | `RAMP_FERTILITY` (barren grey → fertile green) | LatticeRecolor | hover = fertility index; click = supported crop/forage | NEAR |
| A8 | **Resource Deposits** | `voxel`/`laws` · ore/mineral/forest density in column | `LegendKind::Categorical` per resource (from `laws` material rows) + intensity alpha | LatticeRecolor | hover = resource + abundance; click = nearest extractor | NEAR |

### Group B — Population / Society (6)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| B1 | **Population Density** | `agents` · `Civilian` count per cell (already aggregated) | RAMP_DENSITY (empty→settled→crowded) | LatticeRecolor | hover = head count; click = settlement summary | LIVE |
| B2 | **Needs Pressure** | `needs` · `Needs{food,shelter,safety,belonging}` mean | RAMP_NEEDS (content→strained→critical) | LatticeRecolor | hover = per-need breakdown bars; sub-toggle per need | LIVE |
| B3 | **Health** | `needs`/`agents` · health/condition scalar | `RAMP_HEALTH` (green→amber→red) | EntityTint + LatticeRecolor agg | hover = afflictions; click = agent inspector | NEAR |
| B4 | **Happiness / Mood** | `agents` psyche · mood/temperament (BLIND today — psyche layer) | `RAMP_MOOD` (despairing→content→joyful) | EntityTint | hover = mood drivers; click = psyche card | BLIND |
| B5 | **Age / Lineage** | `genetics`+`agents` · age, generation depth | `RAMP_AGE` (youth→elder) ; lineage = Categorical by founding ancestor | EntityTint | hover = age+lineage; click = family tree (ties to Legends) | NEAR (age) / BLIND (lineage graph) |
| B6 | **Migration Flow** | `agents` · `daily_path`/movement vectors aggregated (flow-field, crowds.md) | arrow gizmos colored by volume `RAMP_FLOWVOL` | Gizmo (arrows) | hover = net flow + destination cluster; click = trace path | NEAR |

### Group C — Economy (6)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| C1 | **Resource Stock** | `economy` · settlement inventory per resource | `RAMP_STOCK` (scarce red → surplus green), per selected resource | LatticeRecolor | resource dropdown; hover = stock units; click = stores | NEAR |
| C2 | **Production** | `economy` · output rate per cell/settlement | `RAMP_PRODUCTION` (idle→busy) | LatticeRecolor | hover = goods produced + rate; click = production chain | NEAR |
| C3 | **Trade Routes** | `economy`+`civ-traffic` · flow of goods along lane graph | route gizmos sized/colored by throughput `RAMP_TRADE` | Gizmo (curves) | hover = goods+volume; click = both endpoints; filter by good | NEAR |
| C4 | **Wealth / Prosperity** | `economy` · accumulated value per agent/settlement | `RAMP_WEALTH` (poor→affluent, diverging) | EntityTint + LatticeRecolor agg | hover = wealth index; click = balance sheet | NEAR |
| C5 | **Market Type** | `economy` · emergent market mode (gift/barter/commodity/credit — DERIVED from local trust/scarcity, NOT an enum) | `LegendKind::Categorical` per emergent market mode + key | LatticeRecolor | hover = market mode + drivers; click = market card | BLIND |
| C6 | **Supply / Demand Imbalance** | `economy` · surplus−deficit per resource per cell | diverging `RAMP_BALANCE` (deficit red ↔ neutral ↔ surplus blue) | LatticeRecolor | resource dropdown; hover = signed imbalance; click = trade suggestion | NEAR |

### Group D — Territory / Culture / Polity (5)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| D1 | **Territory** | `agents`/`diffusion` · dominant emergent cluster per cell (cluster overlap, NOT `faction:u32`) | Categorical (`cluster_color`) + filled regions | LatticeRecolor + Gizmo borders | hover = cluster id + cohesion; click = polity card | LIVE (cluster) → richer NEAR |
| D2 | **Culture** | `diffusion` · cultural-trait field (norms/beliefs drift over kinship/contact net) | Categorical per emergent culture cluster; blend at frontiers | LatticeRecolor | hover = dominant cultural traits; click = culture card | BLIND |
| D3 | **Language / Dialect** | `diffusion` · language field (dialect/creole emergence over contact) | Categorical per emergent language cluster; gradient = dialect distance | LatticeRecolor | hover = language id + mutual-intelligibility %; click = lexicon sample | BLIND |
| D4 | **Ideology** | `diffusion` · belief-system field | Categorical per emergent ideology + intensity alpha | LatticeRecolor | hover = ideology + adherence %; click = tenets | BLIND |
| D5 | **Diplomacy / Relations** | `agents`/`diplomacy_ui` data · inter-cluster relation scores | diverging `RAMP_RELATION` (hostile red ↔ neutral ↔ allied green), relative to selected cluster | LatticeRecolor + Gizmo links | select focus cluster; hover = relation score+history; click = relation card | BLIND (relations graph) |

### Group E — Infrastructure / Traffic (3)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| E1 | **Roads / Network** | `civ-traffic` · `TrafficGraph` edges by `RoadClass` | Categorical per road tier (trail/road/highway/bridge) | Gizmo (ribbons) | hover = class+provenance (emergent vs authored); click = segment | NEAR |
| E2 | **Traffic / Congestion** | `civ-traffic`+`agents` · per-lane occupancy/flow (lane graph, roads-lanes.md) | `RAMP_CONGESTION` (free green → jam red) | Gizmo (colored lanes) | hover = throughput vs capacity; click = lane detail | NEAR (lane graph in migration) |
| E3 | **Service Coverage** | `agents`/`economy` · reach radius of emergent service providers (well/market/healer) | `RAMP_COVERAGE` (uncovered grey → well-served green); service dropdown | LatticeRecolor | service dropdown; hover = nearest provider distance; click = provider | BLIND |

### Group F — Hazards / Environment Stress (3)

| # | Overlay | Data source (crate · field) | Legend / ramp | Render kind | Interactions | Avail |
|---|---|---|---|---|---|---|
| F1 | **Disasters / Active Hazards** | `voxel`/`planet` · active fire/flood/quake/storm cells (CA fields: temp/pressure/water) | Categorical per hazard type + intensity alpha; pulsing | LatticeRecolor + Gizmo | hover = hazard + severity; click = affected agents | NEAR (CA fields LIVE) |
| F2 | **Pollution / Contamination** | `voxel`/`laws` · pollutant concentration in air/water/soil CA field | `RAMP_POLLUTION` (clean → toxic), per medium (air/water/soil) | LatticeRecolor | medium dropdown; hover = ppm + source; click = source trace | BLIND (pollutant field) |
| F3 | **Danger / Safety** | `agents`/`tactics` · threat field (combat, predators, crime from psyche) | diverging `RAMP_DANGER` (safe green ↔ neutral ↔ deadly red) | LatticeRecolor | hover = threat sources; click = recent incidents (Legends) | BLIND (threat field) |

**Catalog count: 31 overlays** (8 terrain + 6 population + 6 economy + 5 territory + 3 infrastructure + 3 hazard), squarely in CS2's ~33 band and covering every group the prompt enumerates.

---

## 4. Priority-12 — ship first

Ranked by **(legibility payoff × data-availability) ÷ effort**, biased to overlays whose producing data is **LIVE or NEAR** so they light up immediately and convert the benchmark's #1 gap into a visible win. The first seven are the registry's current overlays (re-grouped + legend-hardened); the next five are the highest-value additions over already-computed data.

| Rank | Overlay | Group | Why first | Avail | FR |
|---|---|---|---|---|---|
| 1 | Elevation (A1) | Terrain | already live; baseline legibility + legend exemplar | LIVE | FR-CIV-INFOVIEW-910 |
| 2 | Water / Hydrology (A2) | Terrain | live submersion; flow accessor is a thin add | LIVE/NEAR | FR-CIV-INFOVIEW-911 |
| 3 | Material / Surface (A6) | Terrain | live band; ties legend to `laws` material DB names | LIVE | FR-CIV-INFOVIEW-912 |
| 4 | Population Density (B1) | Population | live agent aggregate; the "where is the civ" view | LIVE | FR-CIV-INFOVIEW-913 |
| 5 | Needs Pressure (B2) | Population | live; the single most charter-demonstrating overlay (shows emergent strain) | LIVE | FR-CIV-INFOVIEW-914 |
| 6 | Territory (D1) | Territory | live emergent cluster; proves "polities emerge, not enums" | LIVE | FR-CIV-INFOVIEW-915 |
| 7 | Temperature (A3) | Terrain | live proxy now; upgrades to `planet` with no overlay change | LIVE/NEAR | FR-CIV-INFOVIEW-916 |
| 8 | Resource Deposits (A8) | Terrain | NEAR; turns the static world into a *strategic* map | NEAR | FR-CIV-INFOVIEW-917 |
| 9 | Roads / Network (E1) | Infra | NEAR; `TrafficGraph` exists; first Gizmo render-kind exemplar | NEAR | FR-CIV-INFOVIEW-918 |
| 10 | Wealth / Prosperity (C4) | Economy | NEAR; first EntityTint exemplar; high "is there depth" payoff | NEAR | FR-CIV-INFOVIEW-919 |
| 11 | Disasters / Hazards (F1) | Hazard | NEAR; CA temp/pressure/water fields are LIVE; high drama/legibility | NEAR | FR-CIV-INFOVIEW-920 |
| 12 | Migration Flow (B6) | Population | NEAR; Gizmo arrows; makes invisible agent movement *readable* | NEAR | FR-CIV-INFOVIEW-921 |

**Rationale:** ranks 1–7 are zero-new-data (live or already-aggregated) and immediately close the "I can't see what's happening" dismissal; ranks 8–12 each need only a thin read accessor on an existing crate and unlock the three new render kinds (Gizmo, EntityTint) so the remaining 19 BLIND/NEAR overlays become pure registrations once their producing crates surface fields. Every BLIND overlay (psyche, culture, language, ideology, market type, pollution) is specified now and gated (§7) so it ships the moment its emergent field exists — no overlay rework, satisfying the charter's "specify the measurement, let the data emerge."

---

## 5. Toggle UX (CS2 info-view panel + hotkeys)

### 5.1 Info-view panel (extends `draw_info_view_panel`)
- **Grouped accordion.** Replace the flat wrapped button row with six collapsible group headers (Terrain, Population, Economy, Territory, Infrastructure, Hazards) — mirrors CS2's grouped info-view menu. Each group lists its overlays as `selectable_label`s with the description as hover text. Exactly one overlay active at a time (CS2 semantics); an **Off** entry at top.
- **Legend dock.** When an overlay is active, the legend renders beneath the panel via the existing `draw_legend`, extended for `LegendKind::Categorical` (top-N swatch key with cluster labels) and for diverging ramps (centered neutral).
- **Sub-controls.** Overlays with a selector (resource dropdown for C1/C6, medium dropdown for F2, service dropdown for E3, focus-cluster picker for D5, per-need sub-toggle for B2) render an inline control below the legend. Driven by an optional `controls: &[OverlayControl]` on the registration — additive, data-driven.
- **Availability affordance.** BLIND overlays render disabled-greyed with a "data not yet surfaced" tooltip (charter-honest; no silent fake data — aligns with repo "fail clearly, not silently" stance).

### 5.2 Hotkeys
- **`Tab`** — cycle off → each → off (existing `cycle()`; keep).
- **`Shift+Tab`** — cycle backward.
- **`` ` `` (backtick)** — toggle the info-view panel open/closed.
- **`1`–`6`** — jump to that group's *first* overlay (CS2-style group hotkeys); repeat press cycles within the group.
- **`Esc`** — deactivate (overlay off, terrain shows through) via `deactivate()`.
- **`F`** — when a categorical overlay (territory/culture/language/diplomacy) is active, set the *focus cluster* to the cluster under the cursor (drives relative ramps like D5).

### 5.3 Acceptance criteria (UX)
- AC-UX-1: Exactly one overlay active at any time; selecting another deselects the prior (no stacking).
- AC-UX-2: Every active non-categorical overlay shows a labeled legend; every categorical overlay shows a distinct-color note + top-N key.
- AC-UX-3: BLIND overlays are visibly disabled with an explanatory tooltip; they never render placeholder/fake data.
- AC-UX-4: Hotkeys `Tab`/`Shift+Tab`/`` ` ``/`1`–`6`/`Esc` behave as specified; no hotkey both cycles and toggles.
- AC-UX-5: Sub-controls (dropdowns/sub-toggles) only appear for overlays that declare them.

---

## 6. Data-driven registry (extension spec)

The registry already exists (`InfoOverlay`, `InfoViewRegistry`, `default_overlays`, `ramp_color`, `cluster_color`). This spec **extends** it forward-only (no v2, no parallel registry) so new overlays remain a single registration. Pseudocode contract (planner stance — not implementation):

```text
# Extend InfoOverlay with the fields needed for the full suite:
InfoOverlay {
  id, name, description, legend           # exists today
  group: OverlayGroup                     # NEW: Terrain|Population|Economy|Territory|Infrastructure|Hazard
  data_source: DataSource                 # NEW: { crate_name, field, availability }  (doc/telemetry, drives §5.1 greying)
  legend_kind: LegendKind                 # NEW: Sequential | Diverging | Categorical
  render_kind: RenderKind                 # NEW: LatticeRecolor | Gizmo | EntityTint
  controls: &[OverlayControl]             # NEW (optional): resource/medium/service dropdowns, focus-cluster, sub-toggles
  sampler: fn(&SampleCtx, cell) -> OverlaySample   # NEW: fills the sample from the right crate field(s)
  color_fn: fn(&OverlaySample, &OverlayParams) -> OverlayColor   # exists; gains params for selected-resource etc.
}

DataSource.availability ∈ { Live, Near, Blind }   # Blind ⇒ overlay greyed in panel (§5.1), never fabricated
```

- **OverlaySample** grows beyond today's terrain+density fields to a tagged union / option-bag of per-domain scalars (temperature, moisture, fertility, stock, wealth, congestion, relation, hazard, …) so one sampler signature serves all overlays. Unfilled fields default to `None` → the overlay's `color_fn` returns `None` (cell shows terrain through), exactly as `color_population` skips empty cells today.
- **Renderer dispatch** on `render_kind`: `LatticeRecolor` reuses today's gizmo grid; `Gizmo` draws lines/arrows/curves/borders (roads, traffic, trade, migration, territory borders); `EntityTint` recolors agent/structure entities directly. Each render kind is one system; adding a kind is additive, not a fork.
- **Registration is the only extension surface.** Adding "soil fertility" (A7) = one `InfoOverlay{ group: Terrain, data_source:{planet+laws,...}, render_kind: LatticeRecolor, sampler: sample_fertility, color_fn: color_fertility }` appended to `default_overlays()`, plus the `sample_fertility` accessor on the `planet` field. No new system, no panel change (panel is generated from the grouped registry).

### 6.1 Acceptance criteria (registry)
- AC-REG-1: Adding an overlay requires only (a) one `InfoOverlay` registration and (b) one sampler accessor; no edit to the panel, hotkey, or render systems.
- AC-REG-2: The panel, legend, hotkey groups, and availability-greying are all *derived from* the registry (no hardcoded overlay lists outside `default_overlays`).
- AC-REG-3: Categorical overlays derive color via `cluster_color(emergent_cluster_id)` — never from an authored taxonomy (charter gate).
- AC-REG-4: A BLIND overlay registered with `availability: Blind` renders greyed and its `color_fn` is never invoked until its `data_source` reports data present.
- AC-REG-5: `RenderKind` dispatch covers LatticeRecolor, Gizmo, EntityTint; an unknown kind is a compile error (exhaustive match), not a silent no-op.

---

## 7. Charter & availability gating

Per the emergence charter, overlays **measure emergent patterns; they never define them.** Concretely:
- **Categorical overlays** (Biome, Resource, Territory, Culture, Language, Ideology, Market Type, Diplomacy, Roads-by-class, Hazard-by-type) read an *emergent cluster/class id* computed by the producing crate at sample time and hash it to a stable color. Civis must never gain a `Culture` or `MarketType` enum to serve an overlay; the overlay consumes whatever cluster the `diffusion`/`economy` crate measured.
- **BLIND fields** (psyche/mood B4, lineage graph B5, market type C5, culture D2, language D3, ideology D4, diplomacy D5, service coverage E3, pollution F2, danger F3) are **specified and registered now**, gated by `availability: Blind`. They appear greyed in the panel with a "data not yet surfaced" tooltip and activate automatically when the producing crate exposes the field — no overlay code change. This satisfies the repo "fail clearly, not silently" stance (no fake data) and the charter ("specify the measurement, let the substrate emerge").
- **Forward-only:** as `planet` (weather/clock), the `civ-traffic` lane graph (roads-lanes.md), the psyche/social-graph layer, and the `diffusion` culture/language fields land, their overlays flip Blind→Near→Live with zero overlay rework.

---

## 8. WBS (phased, DAG)

| Phase | Task ID | Description | Depends On | Effort (agent) |
|---|---|---|---|---|
| **P1 Registry extension** | T1.1 | Extend `InfoOverlay`/`OverlaySample` with group, data_source, legend_kind, render_kind, controls, sampler (§6) | — | 1 subagent, ~5 min |
| | T1.2 | Grouped accordion panel + legend dock + categorical/diverging legend rendering (§5.1) | T1.1 | 1 subagent, ~5 min |
| | T1.3 | Hotkeys: Shift+Tab, backtick toggle, 1–6 groups, Esc, F focus-cluster (§5.2) | T1.1 | ~3 tool calls, ~2 min |
| **P2 Render kinds** | T2.1 | `Gizmo` render system (lines/arrows/curves/borders) | T1.1 | 1 subagent, ~5 min |
| | T2.2 | `EntityTint` render system (recolor agents/structures) | T1.1 | 1 subagent, ~5 min |
| **P3 Priority-12 (LIVE)** | T3.1 | Re-group + legend-harden the 7 existing overlays (ranks 1–7) into the new registry | T1.2 | 1 subagent, ~5 min |
| **P4 Priority-12 (NEAR)** | T4.1 | Resource Deposits (A8) + sampler on `voxel`/`laws` | T3.1 | parallel subagent |
| | T4.2 | Roads/Network (E1) sampler on `civ-traffic` `TrafficGraph` | T2.1, T3.1 | parallel subagent |
| | T4.3 | Wealth (C4) sampler on `economy` | T2.2, T3.1 | parallel subagent |
| | T4.4 | Disasters/Hazards (F1) sampler on `voxel`/`planet` CA fields | T2.1, T3.1 | parallel subagent |
| | T4.5 | Migration Flow (B6) sampler on `agents` movement aggregate | T2.1, T3.1 | parallel subagent |
| **P5 Gated suite** | T5.1 | Register the remaining 19 NEAR/BLIND overlays as availability-gated stubs (§7) so they light up on data | T3.1, T4.* | 2 parallel subagents, ~10 min |
| | T5.2 | Sub-controls (dropdowns/focus-cluster/sub-toggles) for overlays that declare them | T1.2 | 1 subagent, ~5 min |
| **P6 Validate** | T6.1 | Tests: AC-REG-1..5 + AC-UX-1..5 (registry-derived panel, single-active, categorical color, blind-greying, render-kind exhaustiveness) | all | 1 subagent, ~8 min |

DAG summary: T1.1 → {T1.2, T1.3, T2.1, T2.2}; T1.2 → T3.1 → {T4.1..T4.5} → T5.*; everything → T6.1. P4 tasks parallelize (disjoint crates/files). No cycles.

---

## 9. Requirements traceability (FR-CIV-INFOVIEW-*)

Existing (in `info_views.rs`): `FR-CIV-INFOVIEW-900` (registry + active-overlay toggle), `-901` (legend/color-scale), `-910` (high-value overlays over computed data). This spec adds:

| FR | Requirement |
|---|---|
| FR-CIV-INFOVIEW-902 | Overlays are grouped into the six CS2-class groups (Terrain/Population/Economy/Territory/Infrastructure/Hazard); the panel is generated from the grouped registry. |
| FR-CIV-INFOVIEW-903 | Three render kinds (LatticeRecolor, Gizmo, EntityTint) dispatched from the registry; new render kinds are additive. |
| FR-CIV-INFOVIEW-904 | Categorical overlays derive color from emergent cluster ids only (no authored taxonomy) — charter gate. |
| FR-CIV-INFOVIEW-905 | BLIND overlays are registered and availability-gated; greyed in panel, never fabricate data, auto-activate on data presence. |
| FR-CIV-INFOVIEW-906 | Toggle UX: grouped accordion, legend dock, sub-controls, and hotkeys (Tab/Shift+Tab/backtick/1–6/Esc/F) per §5. |
| FR-CIV-INFOVIEW-911..921 | The Priority-12 overlays (ranks 2–12; rank 1 is `-910`) — one FR per overlay per §4. |
| FR-CIV-INFOVIEW-930 | The full 31-overlay catalog is specified and registrable; each overlay traces to a producing crate field (§3). |

Each FR should trace requirement→code→test→PR in Tracera and to an AgilePlus Epic ("CS2-class legibility") / Story (per group), per the repo traceability protocol.

---

## 10. Cross-project reuse opportunities

- **`ramp_color` + `LegendStop` + `cluster_color`** (the legend/color-ramp + stable-hash-to-color primitives) are generic and already DINOForge-adjacent; candidate for extraction into a shared `phenotype-ui` overlay/legend module reused by WSM3D info overlays and any Phenotype map-viz client. Target: a shared crate; impacted repos: WSM3D, DINOForge. Confirm destination with the user before extracting (cross-repo move).
- **The grouped data-driven overlay registry pattern** (sample→classify→color, render-kind dispatch) is itself reusable for any map-overlay UI across the org; document as the reference pattern.

---

## 11. Verdict

A **31-overlay CS2-class suite** is specified across six groups, each tied to a concrete producing-crate field and one of three render kinds, all expressed as data-driven registrations over the existing `InfoOverlay` registry. **Ship the Priority-12 first** (7 live re-groupings + 5 thin-accessor additions) to close the benchmark's #1 credibility gap immediately; register the remaining 19 as availability-gated stubs that light up automatically as the emergent fields (psyche, culture, language, ideology, markets, pollution) surface — no overlay rework, fully charter-compliant.
