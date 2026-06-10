# Civis Game-R&D — Algorithms, Bots, Systems & Hardening (the bells & whistles)

**Status:** R&D proposal (docs-only). Owner: Game-Systems R&D Lead.
**Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only physical/environmental/genomic laws are authored; everything else EMERGES. **Determinism is NOT required** (charter §2026-05-29): floats, `thread_rng`, GPU non-determinism are all welcome.
**Companions (do NOT duplicate):**
- [`docs/research/ai-rnd.md`](./ai-rnd.md) owns the **LLM/SLM/embeddings** layer (legends *prose*, naming *grammars-from-SLM*, persona chatter, meme-drift embeddings). **This doc is the NON-LLM algorithmic + systems + robustness + polish layer** — the substrate the AI narrator reads, plus the hardening and QoL that make Civis a *complete game*.
- [`docs/research/sota-tech/*`](./sota-tech/) owns the crowds/AI-backbone (`big-brain`/ORCA/flow-fields), the CA-hardening + GPU-CA path, the GI/upscaler gfx plug list, and the DF/SoS scale patterns. **This doc references those and fills the gaps they don't cover** (WFC/L-system/Voronoi procgen, the legends *data structure*, market/auction algorithms, GOAP/HTN/HPA* specifics, playtesting/fuzzing bots, save versioning/migration, perf budgets, undo/blueprints/accessibility/replay/achievements).

**Scope rule applied throughout:** *wrap > hand-roll* (per Quality Charter). Every recommendation cites a crate to fork/adopt or a published algorithm to port; nothing is "write it from scratch" where an OSS 80%-solution exists.

---

## 0. How this maps to the charter (binding)

Procgen, bots, and polish must obey "model the rule, not the outcome." Concretely:
- **Procgen is for the authored Layer-0 substrate and the *user's* tools** (worldgen strata, biome regions, road-tool snapping, scatter of flora seeds) — NOT for authoring emergent outcomes. WFC lays down *terrain/strata templates* and *user-blueprint stamps*; it must never hardcode a settlement that should emerge from agent needs. Desire-path roads still *emerge*; L-systems/road-procgen is the **user authoring tool** + the *rendering* of emergent desire-paths into geometry, not a substitute for emergence.
- **The legends engine is a measured record**, not a generator: it records causal chains over events the sim already produced (the AI doc's narrator then *renders* a window of it to prose).
- **Market algorithms are price-discovery dynamics** (a Layer-0-adjacent economic law), letting market *types* still emerge from local conditions per charter §"Markets of varying types."
- **Bots/fuzzing/hardening are dev-and-runtime infrastructure** — pure [UI/QoL]+NFR, zero charter risk.

---

## 1. SIM ALGORITHMS

### 1.1 Procedural generation beyond noise

| Technique | Civis system | Crate / source (fork-first) | Use (charter-safe) | Adopt |
|---|---|---|---|---|
| **Wave Function Collapse (WFC)** | `civ-planet` strata/aquifer templates; `civ-protocol-3d` building-graph + **user blueprint stamping**; worldgen biome-tile layout | **`wfc`** (gridbugs, arbitrary-grid, ~45k dl, mature) + **`wfc_tiled`** (Tiled CSV/TMX I/O for authoring) | Constraint-solve *authored* tile-sets: geology strata coherence, building-interior layout, user "stamp a plausible district" tool. NEVER auto-place emergent settlements. | **NOW** (worldgen strata + blueprint stamp) / **NEXT** (interiors) |
| **L-systems** | road/trail *geometry rendering* from emergent desire-paths; plant/tree growth in `civ-planet` flora; river tributary branching | port a compact L-system (no dominant Rust crate; ~200-LOC rewrite system + `kurbo`/`lyon` for the curve geometry — same geo stack `civ-traffic` already uses per roads-lanes.md) | Grow *geometry* along emergent desire-path centerlines (roads), branch rivers/plants per growth rules. The *path* emerges; the L-system only renders its visual form. | **NEXT** (flora/rivers) / **LATER** (road geometry — after desire-path substrate exists) |
| **Voronoi regions** | territory/influence cells over emergent polity clusters; biome region partition in `civ-planet`; market-catchment areas in `economy` | **`voronoi`** (Fortune's sweepline) or **`voronoice`** (faster, bounded, actively maintained — prefer); `spade` (Delaunay/CDT) for the dual | Partition space into cells for *display + spatial queries* of already-emergent clusters (territory overlay, catchment). Charter-safe: it visualizes/indexes emergent membership, doesn't define `faction:u32`. | **NOW** (biome partition + territory overlay) |
| **Poisson-disk scatter** | flora/resource/ore seeding in `civ-planet`; initial abiogenesis-site candidate scatter; settlement *candidate* points | **`fast_poisson`** (Bridson, n-D, no_std) or **`poisson-diskus`**; **`map_scatter`** (blue-noise + jittered + Halton + clustered — one crate, many distributions) | Blue-noise placement of *seeds/candidates* the sim then accepts/rejects via Layer-0 conditions. Scatter proposes; laws dispose. | **NOW** (flora/ore/abiogenesis seeding) |

**Verdict (procgen):** Adopt `wfc`+`wfc_tiled`, `voronoice`, and `map_scatter`/`fast_poisson` **now** — all mature, MIT/Apache, small. L-systems are a ~200-LOC port (`kurbo`/`lyon` already in the road stack), deferred until the desire-path substrate they render exists.

### 1.2 Emergent-history / legends ENGINE (the non-LLM substrate)

The AI doc renders legends *prose*; **this is the data structure it reads.** It is the highest-leverage charter feature with no library — it's an architecture (sota-tech sim-misc.md adopt-now #9 names the pattern; here is the concrete design).

- **Core structure: an append-only event DAG (saga graph).** `petgraph` (`StableDiGraph`) over two node kinds:
  - **Entity nodes** — stable IDs for *named* things the sim already tracks: historical figures (agents that crossed a notability threshold), lineages, settlements, polity-clusters, battles, artifacts, discovered laws. Names come from the AI doc's grammar/Markov namer (§1.2 there) — this engine only holds the ID + provenance.
  - **Event nodes** — `{epoch, region, kind, participants:[EntityId], magnitude}` emitted by existing crates: `civ-agents` lifecycle (birth/death/sickness — recent commit), `civ-tactics` battles, `civ-economy` booms/busts, `civ-engine` ideology shifts, `civ-genetics` speciation events.
  - **Causal edges** — `caused_by` / `participated_in` / `succeeded` links wire events into **causal chains** (a regicide → succession war → migration → new settlement). This is what makes it a *saga*, not a flat log: it's queryable as "why did X happen" by walking predecessors.
- **Producer:** a **DF-style coarse zero-player pre-sim** runs worldgen forward N epochs at far-LOD, logging events → instant deep backstory before the player arrives (sota-tech sim-misc §1; Toady's "history is just a record of a zero-player game"). Bound it with a time budget (DF's caveat).
- **Storage:** the graph persists to `civ-save-db` (sqlite already present); event stream already exists in `crates/watch` (SSE/snapshots). **Reuse, don't reinvent** — `watch` *is* the event source; this adds the entity-resolution + causal-linking + queryable-graph layer on top.
- **Consumer:** an egui **Legends browser** (entity → its events → causal neighbors), and the AI narrator (ai-rnd §1.1) reads epoch-windowed subgraphs as digests.
- **Loud-failure note:** notability-threshold + event emission are required producers; if a crate stops emitting lifecycle events the legends view must **show a gap with a logged warning**, not silently render an empty saga.

**Adopt:** **NOW** as architecture (`petgraph` + `crates/watch` + `civ-save-db`, all in-tree). This is the single biggest "living world becomes readable" win and the substrate the entire AI-narration plan depends on.

### 1.3 Economy / market algorithms

| Algorithm | Civis system | Approach / source | Charter fit | Adopt |
|---|---|---|---|---|
| **Tâtonnement price discovery** (Walras) | `civ-economy` allocation (CIV-0100) + joule model (CIV-0107) | Iterative: `price += k · excess_demand(price)`; converges to supply-demand equilibrium. ~40-LOC per market; no crate needed (it's a fixed-point iteration). Arxiv 2306.04890 / 2502.11449 give stable damped variants. | A *price-formation law*; market *type* (gift/barter/credit/planned) still emerges from local trust/scarcity per charter. Floats fine (determinism dropped). | **NOW** (local commodity markets) |
| **Continuous double auction (CDA)** | local marketplaces / bourse near settlements | Order-book matching (bid/ask priority queues); the canonical real-market mechanism. Port the ~150-LOC matching engine. | Emerges where institutional trust is high; tâtonnement is the cheaper default, CDA the richer near-camera variant. | **NEXT** |
| **Trade-route optimization** | inter-settlement trade over the road/lane graph | **comparative advantage** drives flow; route cost = `civ-traffic` lane graph + `pathfinding`/`fast_paths` (already the road routing stack). Min-cost-flow (`pathfinding` crate has it) for multi-source/sink surplus→deficit. | Surplus/deficit + comparative advantage are charter-named drivers; routing reuses the existing lane graph. | **NEXT** |

**Verdict (economy):** Tâtonnement **now** (tiny, high-impact, makes prices *move* legibly — feeds the info-view overlays). CDA + min-cost-flow trade routing **next**, reusing the `civ-traffic` + `pathfinding` stack.

### 1.4 Agent decision backbone hardening

sota-tech crowds.md already specifies the backbone: **`big-brain` (utility AI) → GOAP (plan) → `bevy_behave` (BT leaves)**, with **flow-field tiles (macro) + ORCA via `dodgy_2d` (micro)** for movement. This doc adds the *hardening + scale* specifics it left open:

- **GOAP at scale:** the F.E.A.R.-lineage A*-over-action-preconditions is the gap (crowds.md tags it adopt-**next**, ecosystem thin). **Recommendation:** port a compact GOAP planner (~300 LOC) keyed off `big-brain` scorers; **cache plans per `(goal, world-predicate-bitset)`** so identical situations reuse a plan (the dominant cost-saver at 100k agents). Plans are *advisory*; replanning is event-triggered, never per-tick.
- **HTN as the alternative for *authored* sub-sequences:** HTN (hierarchical task networks) beats GOAP when the decomposition is known (a build job, a patrol). Use HTN/`bevy_behave` for structured leaves, GOAP only for *emergent multi-step* chains (gather→craft→trade) where the sequence isn't authored. Don't pay GOAP's search cost where an HTN/BT suffices.
- **HPA\* (Hierarchical Pathfinding A\*) for the 20mi map:** flat A* over a 20mi voxel grid is infeasible. **HPA\*** clusters the map into a coarse abstract graph (cluster-border portals), pathfinds coarse-then-refine. Pairs with flow-fields: HPA\* for *individual* long routes, flow-fields for *mass* shared-goal movement. `pathfinding` + `fast_paths` (contraction hierarchies — already named for road routing) give the abstract-graph layer; the cluster decomposition is ~200 LOC over the chunk grid Civis already has.
- **LOD-gating the backbone (the real scale lever):** Hot tier = full utility+GOAP+ORCA; Warm = utility + flow-field + batched needs; Cold = statistical (pop-level rates, no per-agent brain). This is the SoS "aggregate the hot loops" discipline (sim-misc §2).

**Adopt:** backbone crates **now** (per crowds.md); GOAP-plan-cache + HPA\* cluster layer **next** (ports, not new deps); HTN split-of-responsibility is a design rule to apply immediately.

---

## 2. BOTS / AUTOMATED SYSTEMS

All run **offline/CI/headless**, never in the shipping sim — model-size/cost constraints relaxed, and they multiply *every other system's* iteration speed.

### 2.1 Automated playtesting bots (headless stress + imbalance/softlock/runaway detection)

- **Architecture:** a **headless sim runner** (Bevy `MinimalPlugins`, no render — already feasible since the sim crates are render-independent) drives the world for K epochs with scripted/utility god-tool inputs, dumping `civ-engine` timeseries (CIV-0103). Run **thousands of playthroughs in parallel** (the modl.ai / Unity-Game-Simulation pattern) via `rayon` across seeds.
- **Detectors (heuristic-first, no ML):**
  - **Softlock/deadlock:** no state change in N epochs (population/economy/event-rate flatlined) → flag.
  - **Runaway loop:** monotonic unbounded growth (population/resource/price) past a percentile band over a window → flag (this is also the NaN/overflow early-warning, §3).
  - **Imbalance:** one polity-cluster's share of pop/resource/territory crosses a Gini/HHI threshold → flag runaway-dominance.
  - **Starvation/collapse spirals:** need-satisfaction trending to zero across a region.
- **Search-based exploration:** MCTS / evolutionary search over god-tool action sequences to *find* the states that break the sim (arxiv 1908.01417 active-learning param tuning; the standard "let search find the bugs humans won't" approach). Reuse the **GA pattern already proven in `civ-tactics`** (doctrine_fitness) — same machinery, new fitness = "induces a failure/imbalance."
- **Output:** structured anomaly records → the AI doc's §3 SLM *triages/narrates* them into a human-readable balance report. Heuristics detect; SLM explains. Run as a nightly CI batch.

### 2.2 Balance auto-tuning (parameter sweeps)

- **Treat law/economy/drift knobs as a search space**, score generated worlds against target metrics (biome variety, settleability, economic liveliness, no-runaway). Optimize with **CMA-ES / Bayesian search** (`cmaes` crate, or `argmin` for general optimization) — *not* grid sweep (too costly at this dimensionality). Reuse the `civ-tactics` GA scaffold. (ai-rnd §3 "procgen tuning" names this; here is the concrete optimizer pick.)

### 2.3 Fuzzing the voxel CA + save/load

- **Property/fuzz the CA:** **`proptest`** (already a dev-dep) for invariants — **mass conservation** (total material in == out per tick, the charter's core CA law), **no-NaN/no-inf** cell states, **bounded growth** (no cell value escapes physical range). **`cargo-fuzz`/`libfuzzer`** on the CA step with random material/temperature/pressure fields to surface panics/overflows. The CA is the Layer-0 substrate — fuzzing it is the highest-value fuzz target.
- **Fuzz save/load round-trips:** `proptest` arbitrary world-state → serialize → deserialize → assert structural equality (round-trip invariant). Catches the migration bugs §3.2 must prevent. **`arbitrary`** crate to derive structured inputs.

### 2.4 Regression-detection bots

- **Golden-metric regression:** the nightly headless run records a metric *fingerprint* (population curve shape, economic throughput, event-rate histogram) per seed; CI diffs against a committed baseline and flags statistically-significant drift (Kolmogorov–Smirnov / simple band check). Catches "a refactor silently changed emergent behavior" — the failure mode determinism-dropping makes *harder* to catch, so this bot is the replacement safety net.
- **Perf regression:** capture frame-time / tick-time percentiles (criterion benches + the headless tick-time histogram) and fail CI on >X% regression. Ties to §3.3.

---

## 3. ROBUSTNESS / HARDENING (loud-not-silent, per repo CLAUDE.md)

The CLAUDE.md stance is explicit: **require dependencies; fail clearly, not silently; list each failing item.** Determinism is dropped, so the hardening goal shifts from "bit-identical replay" to **"no NaN/overflow/unbounded-growth crash, and every failure is loud and named."**

### 3.1 Error handling + recovery

- **`thiserror`** for typed domain errors (named failing items per CLAUDE.md), **`anyhow`** only at top-level bins. **No silent `unwrap_or_default()`** on required state — that's exactly the "graceful degradation" the stance forbids.
- **Preflight checks (loud):** on startup, verify required artifacts (law DB, model files per ai-rnd §4.4, save schema version) and fail with the *named* missing item (`civis preflight failed: missing laws/reactions.ron; civ-save-db schema v3 expected, found v1`).
- **Sim-thread panic isolation:** wrap each LOD-tier system in a catch so one region's panic **loudly quarantines that region** (logged, flagged on the event feed) rather than crashing the whole world — recovery, but *announced*, never hidden.

### 3.2 Save-file integrity / versioning / migration

- **Versioned schema with tagged-enum routing:** `#[serde(tag = "_version")]` over a `SaveEnvelope` enum (V1/V2/…) — the community-validated Rust pattern; **`serde-evolve`** does exactly this (separates wire format from domain type, deserializes any historical version, migrates forward). Adopt `serde-evolve` rather than hand-rolling `Option<T>`-everywhere.
- **Integrity:** **blake3** checksum (already in-tree via the AI cache) over the save body; on load, mismatch → **loud refusal**, not best-effort partial load.
- **Forward-only migration chain** (V1→V2→V3 migrators) per the Long-Term-Stability protocol; round-trip fuzzed (§2.3). Snapshot-based per charter (not replay-from-seed).

### 3.3 Perf budgets + frame-time guards + LOD escalation

- **Frame budget:** 60fps = **16.66ms/frame** hard budget. Instrument with Bevy's **`trace_tracy`** spans (built-in) + **`bevy_perf_hud`** for an in-game live readout.
- **Frame-time guard → automatic LOD escalation:** a system watches the rolling frame-time; when it exceeds budget for N frames, **escalate LOD** (shrink Hot tier radius, promote Warm→Cold agents, coarsen far CA) — a *visible, logged* degradation ("perf guard: LOD escalated, Hot radius 64→48"), per the loud-not-silent stance. This is the runtime resilience replacement for determinism.
- **Tick-budget for the sim** (separate from render): the sim runs on its own schedule; over-budget ticks coalesce/drop the lowest-priority work (cold-tier statistical updates) with a logged warning — never block render.

### 3.4 Determinism-OPTIONAL resilience (no NaN/overflow/unbounded growth)

Since determinism is dropped, the *new* invariants to enforce (the things that would otherwise silently corrupt a float-based sim):
- **NaN/inf guards** at CA + economy + agent-need boundaries: debug-assert finite; release **clamp + log-once** (loud sentinel, not silent swallow). A single NaN propagates through a float sim and kills it — guard early (per the search finding).
- **Overflow:** `checked_*`/`saturating_*` arithmetic on resource/population counters; saturating + a flagged-overflow event beats wrap-around silent corruption.
- **Unbounded-growth caps:** every accumulating quantity (population, resource pool, price) has a physical/economic ceiling derived from `civ-laws`; hitting it emits a balance-event (which §2.1's runaway detector also consumes). Conservation laws (mass) are the natural cap for the CA.

### 3.5 Telemetry / metrics / crash reporting

- **`tracing` + `tracing-subscriber`** structured spans (Bevy already uses tracing) → JSON sink for the headless bots (§2) to consume. The `civ-engine` timeseries (CIV-0103) is the in-sim metric stream; export it to the telemetry sink.
- **Crash reporting:** **`color-eyre`**/`human-panic` for readable panic reports with the named failing subsystem; optionally **Sentry** (the `sentry` Rust crate — but keep it opt-in/self-hostable per local-OSS-first stance; Sentry can self-host). Capture the last-N event-feed entries + LOD state in the crash context so a crash is reproducible from the snapshot.

### 3.6 Graceful (LOUD) degradation — the unifying rule

Every degradation path above is **announced**: LOD escalation logs + shows in the perf HUD; region quarantine flags on the event feed; missing-model degrades to a *named* fallback with a warning; save-checksum mismatch refuses loudly. **No path silently reduces functionality** — that is the CLAUDE.md non-negotiable.

---

## 4. POLISH / QoL — bells & whistles

Maps to the feature-matrix §6–7 BLIND cells; the competitive-benchmark names **legibility (CS2 info-views + inspect-anything) as the single biggest credibility gap** — these are mostly egui work with zero charter risk.

| Feature | Civis system | Crate / approach | Adopt |
|---|---|---|---|
| **Undo/redo** (god-tools) | god-tool palette (tool-design-directives.md) | **`undo`** (command pattern, target undo/redo) or **`undo_2`** (returns command sequences the app interprets — better for Bevy editor-style). CS2's *lack* of undo is a documented pain → this is a differentiator. | **NOW** |
| **Blueprints / copy-paste** | building/road stamps | serialize a selection region (voxel+entity tags) → re-stamp via the WFC/placement tool; reuse the save-serialization (§3.2). | **NEXT** |
| **Hotkeys + rebinding** | input layer | **`leafwing-input-manager`** (Bevy gold-standard: action-based, rebindable, gamepad+kb) — wrap, don't hand-roll a keymap. | **NOW** |
| **Tooltips everywhere + inspect-anything** | the legibility moat | **`bevy_egui`** + the inspect target = click any voxel/agent/settlement → pull its components + its legends-graph node (§1.2). `bevy-inspector-egui` for dev; bespoke player inspector for ship. **Highest-leverage polish feature.** | **NOW** |
| **Info-view overlays (~33, CS2 bar)** | the #1 credibility gap | shader/material overlay pass driven by per-voxel/per-region metric → color ramp (pollution/land-value/happiness/wealth/territory-Voronoi §1.3). egui legend + toggle. | **NOW** (first 6–8 overlays) |
| **Onboarding / tutorial** | first-run | progressive-disclosure tooltip tour (data-driven steps in RON) — no crate, ~config-driven per the "template > hardcode" rule. | **NEXT** |
| **Accessibility** | UI | colorblind-safe palettes (viridis/cividis ramps for overlays — perceptually uniform, CB-safe), UI scaling (egui native), full key-remap (leafwing). Bevy's **`accesskit`** integration for screen-reader where feasible. | **NOW** (palettes+scaling) / **NEXT** (accesskit) |
| **Camera bookmarks** | camera | store/restore camera transforms (hotkey slots) — trivial, ~30 LOC. | **NEXT** |
| **Time controls (speed/pause)** | sim schedule | sim-tick rate multiplier + pause decoupled from render (already have separate sim schedule). | **NOW** |
| **Notifications / alerts + camera-jump** | event feed | consume the legends event stream (§1.2) → routed alerts (RimWorld-letter / CS2-chirper style) with click-to-jump-camera. | **NOW** |
| **Statistics / graphs** | dashboards | **`egui_plot`** (already named available) over `civ-engine` timeseries — population/economy/ideology line+bar+box; **`egui_graphs`** (egui+petgraph) to *visualize the legends saga graph* directly. | **NOW** |
| **Photo mode** | render | hide-UI + free-cam + screenshot (screenshot-automation.md exists) + optional DoF/grade toggle. | **NEXT** |
| **Replay / timelapse** | persistence | snapshot-interval capture → playback scrubber (charter: snapshots, not seed-replay; timelapse = play snapshots fast). Pairs with photo mode for shareable clips. | **LATER** |
| **Achievements** | meta | data-driven achievement defs (RON) checked against legends events/metrics — emergent-friendly ("a lineage crossed sentience", "a market hit hyperinflation"). | **NEXT** |

---

## 5. PERF (the scale levers)

sota-tech material-physics.md owns the **GPU-CA** detail; this consolidates the perf DAG. The #1 lever per sota-tech is GPU compute for the CA.

1. **GPU compute CA** (sota-tech experimental #1): port the CA stencil → wgpu compute, dispatch only dirty chunks, render from GPU buffers; CPU CA stays for far-LOD/authoritative. **1–2 orders more simulated voxels** — *the* scale lever. Determinism-dropped clears the GPU non-determinism concern.
2. **Multithread the tick with `rayon`** (already implied by sota-tech CA-harden): chunk-parallel CA, batched needs ticks, parallel agent SoA passes. Bevy's parallel scheduler handles system-level; `rayon` for the inner data-parallel loops.
3. **ECS optimization (SoA, free from Bevy):** SoS's data-oriented discipline comes free from Bevy ECS columns (sim-misc §2). Pool pathing, batch needs, share decisions across similar agents.
4. **Spatial indexing:** a chunk-grid / `kdtree`/`rstar` (R-tree) for neighbor queries (ORCA neighbors, market catchment, inspect-pick). Voronoi (§1.3) doubles as a coarse spatial index for region queries.
5. **Chunk LOD** (already built — SVO + streaming + frustum cull): the perf-guard (§3.3) escalates it dynamically.

---

## 6. ADOPT-NOW TOP-10 (impact ÷ effort)

| # | Item | Why now (impact) | Effort | System / §ref |
|---|---|---|---|---|
| **1** | **Legends saga-graph engine** (`petgraph` over `crates/watch` events + causal links) | The substrate that makes emergence *readable* AND feeds the entire AI-narration plan. DF's actual moat. Architecture, in-tree deps. | Med | §1.2 |
| **2** | **Inspect-anything + tooltips** (`bevy_egui`, click→components+legends node) | Closes the #1 credibility gap (legibility). The highest-leverage polish feature; depth that can't be seen is worth zero. | Low–Med | §4 |
| **3** | **Info-view overlays** (first 6–8, CB-safe viridis ramps + Voronoi territory) | The CS2 ~33-overlay bar; makes emergent depth *perceptible*. Pure egui+shader, no charter risk. | Med | §4, §1.3 |
| **4** | **Frame-time guard → auto LOD escalation** (`trace_tracy`+`bevy_perf_hud`) | The single highest-leverage *robustness* fix — keeps 60fps at 20mi-scale without silent stutter; loud, logged degradation. | Low–Med | §3.3 |
| **5** | **Headless playtesting bots + heuristic detectors** (`rayon` runner, softlock/runaway/imbalance) | Multiplies every system's iteration speed; catches the failures determinism-dropping makes hard to see. Reuses `civ-tactics` GA. | Med | §2.1 |
| **6** | **Save versioning + integrity** (`serde-evolve` tagged-enum + blake3 checksum) | Prevents the inevitable "old save won't load / corrupt save" — required, loud. | Low | §3.2 |
| **7** | **Tâtonnement price discovery** in `civ-economy` | ~40 LOC; makes prices *move* legibly → directly powers the wealth/economy overlays. | Low | §1.3 |
| **8** | **NaN/overflow/unbounded-growth guards** (clamp+log-once, `checked_*`, conservation caps) | Cheapest crash-prevention for a float sim with determinism dropped; loud sentinels. | Low | §3.4 |
| **9** | **Procgen kit** (`wfc`+`wfc_tiled`, `voronoice`, `map_scatter`) for worldgen strata + biome partition + flora/ore scatter + blueprint stamp | Richer authored substrate + the user blueprint/stamp tool; all mature MIT/Apache crates. | Low–Med | §1.1 |
| **10** | **Stats/graphs + saga-graph viz** (`egui_plot` + `egui_graphs`) and **input rebinding** (`leafwing-input-manager`) | egui_plot over CIV-0103 timeseries + visual legends browser; leafwing gives rebindable hotkeys free. Table-stakes QoL, near-zero effort. | Low | §4 |

**Single highest-leverage ROBUSTNESS fix:** **#4 — the frame-time guard with automatic, *loud* LOD escalation.** At 20mi×20mi the perf cliff is the most likely "it doesn't run / it stutters" dismissal (competitive-benchmark §3 credibility gap #4), and the loud-degradation pattern is exactly the CLAUDE.md stance applied to performance. It also subsumes the determinism-dropped resilience story (graceful-but-announced under load).

**Single highest-leverage POLISH feature:** **#2 — inspect-anything + tooltips.** The competitive-benchmark verdict is unambiguous: legibility is the biggest credibility gap, outranking even the visual gap, because emergent depth that can't be read is worth zero — and DF proves a crude-looking game wins *if you can read the depth*. Click-any-voxel/agent/settlement → its components + its legends-graph node is the keystone that makes #1 and #3 pay off.

---

## 7. HARDENING CHECKLIST

- [ ] **Preflight** verifies + names every required artifact (laws RON, model files, save schema version); fails loud with the named missing item.
- [ ] **No silent fallbacks:** zero `unwrap_or_default()` on required state; every degrade path logs + surfaces a named warning.
- [ ] **NaN/inf guard** at CA, economy, and agent-need float boundaries (debug-assert finite; release clamp+log-once).
- [ ] **Overflow:** `checked_*`/`saturating_*` on all population/resource/price counters; flagged-overflow event on saturate.
- [ ] **Unbounded-growth caps** from `civ-laws` on every accumulator; mass-conservation enforced as the CA cap.
- [ ] **Save:** versioned tagged-enum envelope (`serde-evolve`), forward-only migrators, blake3 integrity check, round-trip fuzzed (`proptest`+`arbitrary`).
- [ ] **CA fuzzed** (`cargo-fuzz`) for panics; **property-tested** (`proptest`) for mass-conservation + finite-state + bounded invariants.
- [ ] **Frame-time guard** → auto LOD escalation, logged + shown in perf HUD; separate sim-tick budget with coalesce/drop + warning.
- [ ] **Sim-thread panic isolation** quarantines a failing region loudly (event-feed flag), never crashes the world silently.
- [ ] **Telemetry** (`tracing` structured spans) + crash context (last-N events + LOD state) + readable panic report (`color-eyre`/`human-panic`).
- [ ] **Regression bots** (golden-metric KS-diff + perf-percentile) gate CI; **runaway/softlock/imbalance detectors** run nightly headless.
- [ ] **Accessibility:** CB-safe perceptually-uniform overlay palettes (viridis/cividis), UI scaling, full key-remap.

---

## 8. PHASED WBS / DAG

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| **P1 Substrate + Safety** | G1 | NaN/overflow/unbounded-growth guards across CA/economy/needs (§3.4) | — |
| P1 | G2 | Save versioning (`serde-evolve`) + blake3 integrity + migrators (§3.2) | — |
| P1 | G3 | Legends saga-graph engine: entity/event/causal nodes over `crates/watch` + `petgraph`, persist to `civ-save-db` (§1.2) | — |
| P1 | G4 | Tâtonnement price discovery in `civ-economy` (§1.3) | — |
| P1 | G5 | Frame-time guard + `trace_tracy`/`bevy_perf_hud` + auto LOD escalation (§3.3) | — |
| **P2 Legibility** | H1 | Inspect-anything + tooltips (`bevy_egui`, click→components+legends node) (§4) | G3 |
| P2 | H2 | Info-view overlays (6–8, CB-safe ramps) + Voronoi territory (`voronoice`) (§4, §1.3) | G4 |
| P2 | H3 | Stats/graphs (`egui_plot`) + saga-graph browser (`egui_graphs`) (§4) | G3 |
| P2 | H4 | Notifications/alerts + camera-jump off the legends event stream (§4) | G3 |
| P2 | H5 | Undo/redo (`undo_2`) + input rebinding (`leafwing-input-manager`) + time controls (§4) | — |
| **P3 Bots + Tuning** | J1 | Headless sim runner (`rayon`, MinimalPlugins) (§2.1) | G1 |
| P3 | J2 | Heuristic detectors (softlock/runaway/imbalance/starvation) + SLM triage handoff (§2.1, ai-rnd §3) | J1, G3 |
| P3 | J3 | CA fuzzing (`cargo-fuzz`) + property tests (`proptest`: conservation/finite/bounded) (§2.3) | G1 |
| P3 | J4 | Save round-trip fuzz + regression bots (golden-metric + perf) in CI (§2.3, §2.4) | G2, J1 |
| P3 | J5 | Balance auto-tuning (CMA-ES via `cmaes`/`argmin`, reuse `civ-tactics` GA) (§2.2) | J1, J2 |
| **P4 Procgen + Movement depth** | K1 | Procgen kit: `wfc`/`wfc_tiled` worldgen strata + `map_scatter` flora/ore scatter + blueprint stamp (§1.1) | — |
| P4 | K2 | GOAP plan-cache + HTN/BT split + HPA\* cluster layer (`fast_paths`) (§1.4) | — |
| P4 | K3 | CDA + min-cost-flow trade routing over `civ-traffic` lane graph (§1.3) | G4, K2 |
| P4 | K4 | L-system flora/river/road-geometry rendering (`kurbo`/`lyon`) (§1.1) | K1 |
| **P5 Showpiece perf + polish** | L1 | GPU compute CA (wgpu, dirty-chunk dispatch) — the #1 scale lever (§5, sota-tech) | G5 |
| P5 | L2 | `rayon` inner-loop parallelism + spatial index (`rstar`) (§5) | — |
| P5 | L3 | Blueprints/copy-paste, photo mode, achievements, onboarding tour, accesskit (§4) | H1, K1 |
| P5 | L4 | Replay/timelapse snapshot scrubber (§4) | G2 |

**DAG notes:** P1 tasks are independent (parallelizable, ~5 subagents). P2 legibility depends on the P1 legends engine (G3) + economy (G4). P3 bots depend on the headless runner (J1) and guards (G1). P4/P5 are depth/showpiece layers gated on their P1 foundations. Aggressive estimate: P1 ≈ 5 parallel subagents / ~15–20 min wall; each P2/P3 item ≈ 2–3 subagents / ~5–10 min; P5 GPU-CA is the one genuinely large item (3–5 subagents / ~20 min).

---

## 9. Cross-Project Reuse Opportunities (Phenotype org)

Per the Cross-Project Reuse Protocol, candidates for extraction to a shared Phenotype crate (confirm destination with user before moving):
- **Legends saga-graph engine** (§1.2) — a generic `entity/event/causal-DAG over an event stream` is reusable by any sibling sim/world project (WSM3D, DINOForge). Candidate: `phenotype-history`.
- **Headless playtest-bot + heuristic-detector + GA-tuning harness** (§2) — generic "drive a headless sim, detect anomalies, sweep params" is project-agnostic. Pairs with the `civ-tactics` GA. Candidate: `phenotype-playtest`.
- **Frame-time-guard + loud-LOD-escalation** (§3.3) and **save-versioning/integrity** (§3.2) — both are generic Bevy/serde hardening utilities. Candidate: fold into a shared `phenotype-bevy-hardening`.
- **Procgen kit wrappers** (§1.1) — thin charter-agnostic wrappers over `wfc`/`voronoice`/`map_scatter` are reusable. Candidate: `phenotype-procgen`.

---

## 10. Sources

**Procgen (crates):**
- `wfc` / `wfc_tiled` (gridbugs WFC): https://github.com/gridbugs/wfc · https://docs.rs/wfc_tiled/ · https://www.gridbugs.org/wave-function-collapse/
- `voronoi` (Fortune's sweepline): https://crates.io/crates/voronoi · `voronoice` (bounded/maintained): https://crates.io/crates/voronoice · `spade` (Delaunay/CDT): https://crates.io/crates/spade
- Poisson-disk: `fast_poisson` / `poisson-diskus` (Bridson): https://github.com/pjohansson/poisson_diskus · `map_scatter` (multi-distribution blue-noise): https://crates.io/crates/map_scatter
- L-systems geometry stack: `kurbo` / `lyon` (per roads-lanes.md)

**Economy / markets:**
- Tâtonnement in Fisher Markets (stable damped variants): https://arxiv.org/pdf/2306.04890 · Tractable General Equilibrium: https://arxiv.org/pdf/2502.11449
- Continuous double auction / auction-as-search: https://arxiv.org/pdf/2006.00775
- Trade routing: `pathfinding` (min-cost-flow, A*) · `fast_paths` (contraction hierarchies) — per sota-tech roads-lanes.md

**Agent backbone (specifics):** GOAP (F.E.A.R. lineage) / HTN / HPA\* — `pathfinding`+`fast_paths` for the abstract graph; `big-brain`/`dodgy_2d`/`bevy_behave` per sota-tech crowds.md.

**Bots / playtesting / fuzzing:**
- AI playtest bots (scale, infinity-loop QA): https://modl.ai/ai-bots-help-game-developers-escape-the-infinity-loop-of-qa-testing/ · https://blog.unity.com/games/automate-your-playtesting-create-virtual-players-for-game-simulation
- Active-learning param tuning via playtesting: https://arxiv.org/pdf/1908.01417 · multi-agent balancing (RuleSmith): https://arxiv.org/pdf/2602.06232
- `proptest`, `cargo-fuzz`, `arbitrary` (Rust testing/fuzzing crates); `cmaes`/`argmin` (optimization)

**Robustness / save / perf:**
- `serde-evolve` (schema evolution, tagged-enum routing): https://docs.rs/serde-evolve/ · https://github.com/danieleades/serde-evolve · Rust serde versioning patterns: https://siedentop.dev/posts/rust-serde-versioning/
- Bevy profiling (`trace_tracy`): https://github.com/bevyengine/bevy/blob/main/docs/profiling.md · `bevy_perf_hud`: https://crates.io/crates/bevy_perf_hud · frame budget: https://pulsegeek.com/articles/what-is-a-frame-time-budget-in-optimization/
- `tracing`/`tracing-subscriber`, `color-eyre`/`human-panic`, `sentry` (self-hostable) — crates.io

**Legends / emergent history:**
- DF Legends mode + worldgen (the gold standard): https://dwarffortresswiki.org/index.php/DF2014:Legends · https://dwarffortresswiki.org/index.php/World_generation
- DF emergent-complexity analysis (zero-player history record): https://research.genezi.io/p/dwarf-fortress-the-nexus-of-emergent
- `petgraph` (StableDiGraph) — crates.io; `crates/watch` (in-tree event stream) + `civ-save-db` (in-tree persistence)

**Polish (crates):**
- `undo` / `undo_2`: https://crates.io/crates/undo · https://docs.rs/undo_2 · `leafwing-input-manager` (input/rebinding) · `bevy_egui`/`bevy-inspector-egui`
- `egui_plot`: https://github.com/emilk/egui_plot · `egui_graphs` (egui+petgraph viz): https://github.com/blitzarx1/egui_graphs · `accesskit` (Bevy a11y)

**Internal:** [`emergence-charter.md`](../guides/emergence-charter.md) · [`feature-matrix.md`](../specs/feature-matrix.md) · [`competitive-benchmark.md`](./competitive-benchmark.md) · [`ai-rnd.md`](./ai-rnd.md) · [`sota-tech/*`](./sota-tech/) (crowds, sim-misc, material-physics, gfx, roads-lanes)
