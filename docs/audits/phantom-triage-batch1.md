# Phantom-ID Triage — Batch 1 (2026-06-10)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs) — filtered to **CODE-ONLY-no-spec** (786 rows),
sorted by reference count, top **100** taken.

**Verdict taxonomy:**
- **(a) REAL-REQUIREMENT** — the code/docs implement a user-meaningful capability.
  - `cov: yes` → already covered by an existing spec file; no stub needed.
  - `cov: no`  → NO existing spec; a one-line stub is appended to
    `agileplus-specs/civ-021-recovered-requirements/spec.md` (civ-019 was already
    taken by `civ-019-emergence-metrics-dashboard`, so we use civ-021 — next free).
- **(b) STALE-ID** — ID is a comment/trace artifact; no implementation matches.
- **(c) RENAME** — ID maps onto an existing spec'd ID modulo naming drift.

## Summary

- (a) REAL & covered by existing spec: **86**
- (a) REAL & UNCOVERED (stub appended to civ-021): **12**
- (b) STALE-ID: **1**
- (c) RENAME (map to existing ID): **1**

## Per-row verdicts

| # | FR ID | Verdict | Cov? | Evidence (file:line) |
|---|-------|---------|------|----------------------|
| 1 | `FR-CIV-0001-TICK` | **b** | n/a | `docs/guides/COPILOT_L3_AGENTS.md:203,246` + `GIT_WORKTREE_GUIDE.md:148,168,189` — example commit-message template only; no code/tests implement this specific ID. The real tick loop is `FR-CIV-CORE-001` in `crates/engine/src/engine.rs`. Rec |
| 2 | `FR-CIV-GODTOOL-920` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:24` — Time controls and god-hand interactions. |
| 3 | `FR-CIV-INFOVIEW-900` | **a** | yes | `clients/bevy-ref/src/info_views.rs:17,380` — overlay registry + active-overlay toggle; real impl in info_views.rs. |
| 4 | `FR-CIV-INFOVIEW-910` | **a** | yes | `clients/bevy-ref/src/info_views.rs:19,268` — colour functions for high-value overlays (pure). |
| 5 | `FR-CIV-INSPECT-900` | **a** | yes | `clients/bevy-ref/src/inspect.rs:12,351,363` — raycast pick → classify → populate inspector. |
| 6 | `FR-CIV-INSPECT-910` | **a** | yes | `clients/bevy-ref/src/inspect.rs:13,15` — hover tooltip + god-hand cursor readout. |
| 7 | `FR-CIV-PLANET-040` | **a** | yes | `crates/planet/src/geology.rs:1,106` — deterministic geology seed layer (`GeologyMap::seed`); bit-identical for same config. |
| 8 | `FR-CIV-VOXEL-020` | **a** | yes | `clients/bevy-ref/src/bin/standalone.rs:189` + `voxel_stream.rs:13,352,364` — camera-driven chunk streaming; tests verify round-trip and bounded disc. |
| 9 | `FR-CIV-WEB-007` | **a** | yes | `web/dashboard/src/babylon_scene.tsx`, `rendererMode.ts` — Babylon.js viewer module via `?renderer=babylon`. |
| 10 | `FR-CIV-AI-007` | **a** | yes | `crates/ai/src/cache.rs:3` + `provenance.rs` + `lib.rs:13,205` — blake3 hash-keyed cache + `AiEvent` provenance. |
| 11 | `FR-CIV-CORE-002` | **a** | yes | `docs/models/civ-sim/TECHNICAL_SPEC.md:1368` + `docs/AGILE_WORKSTREAM.md:445` — deterministic transition (test: 100-tick replay). |
| 12 | `FR-CIV-CORE-003` | **a** | yes | `docs/AGILE_WORKSTREAM.md:446` + `docs/reference/CODE_ENTITY_MAP.md:17` — seeded RNG in stochastic phase; maps to WebSocket JSON-RPC server. |
| 13 | `FR-CIV-ECON-001-MARKET` | **a** | no | `docs/guides/COPILOT_L3_AGENTS.md:90,470` — Market price tracking (`crates/economy/src/market.rs`); ID is a hyphenated alias of `FR-CIV-ECON-001` (see `docs/reference/FR_TRACKER.md`). |
| 14 | `FR-CIV-ECON-002` | **c** | n/a | `docs/guides/COPILOT_L3_AGENTS.md:92,271,474` — Joule allocator (energy conservation). Maps to `FR-CIV-ECON-002-JOULE` (the hyphenated form has its own row in the matrix). |
| 15 | `FR-CIV-NOTIFY-900` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:26` — Event and alert feed (W6.4). |
| 16 | `FR-CIV-NOTIFY-910` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:27` — Statistics dashboards (W6.5). |
| 17 | `FR-CIV-PLANET-020` | **a** | no | `crates/engine/src/engine.rs:434,460` — coastal water columns shifting with tide offset; `WATER_MARKER_MATERIAL` constant. Real engine state, no formal spec doc. |
| 18 | `FR-CIV-RTS-003` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:1116,1318,1344` — formation types (wedge/line/column/circle). |
| 19 | `FR-CIV-UX-005` | **a** | no | `clients/godot-ref/scripts/camera.gd:43` + `ui.tscn:107` + `web/dashboard/src/bottom_bar.tsx` — era / overview camera presets (Cam Wide/Cam Close). Real cross-client UI. |
| 20 | `FR-CIV-AI-001` | **a** | yes | `crates/ai/src/lib.rs:7,55` — civ-ai crate = provider port + capability-based routing; consumers are feature services. |
| 21 | `FR-CIV-GODTOOL-921` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:25` — Undo and blueprint reuse in the UI. |
| 22 | `FR-CIV-INFOVIEW-901` | **a** | yes | `clients/bevy-ref/src/info_views.rs:18` + legend ramp test at L782 — legend / colour-scale. |
| 23 | `FR-CIV-INFOVIEW-911` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:30` — Resource overlays (W4.3). |
| 24 | `FR-CIV-INFOVIEW-912` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:31` — Population and well-being overlays (W4.4). |
| 25 | `FR-CIV-PSYCHE-920` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:16` — queryable chronicle of emergent events (legends mode). |
| 26 | `NFR-CIV-PERF-901` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:27` — 60fps target + draw-call scaling (W5.6 second half). |
| 27 | `NFR-CIV-SCALE-900` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:22` — 20mi world extent and fixed-point addressing (W5.1). |
| 28 | `FR-CIV-ACT-001` | **a** | no | `docs/reference/FR_TRACKER.md:28` + `docs/models/civ-sim/TECHNICAL_SPEC.md:2103` — Citizen lifecycle; maps to CIV-0103 citizen lifecycle spec (which is closed, not in `docs/specs/requirements/`). |
| 29 | `FR-CIV-AI-006` | **a** | yes | `crates/ai/src/providers/dummy.rs:1` — `DummyAiProvider` (deterministic, test-only). |
| 30 | `FR-CIV-ASSET-010` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80,2517` — Build-time reproducibility (content-hash stability). |
| 31 | `FR-CIV-ASSET-011` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81,2529` — Render batch performance (<30s on 4-core CI). |
| 32 | `FR-CIV-GEO-010` | **a** | yes | `docs/specs/CIV-0101-two-zoom-lod-v1.md:1580,1584` — two-zoom LOD traceability (L2RegionSnapshot, L1DistrictSnapshot). |
| 33 | `FR-CIV-GODTOOL-900` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:23` — Data-driven god-tool palette. |
| 34 | `FR-CIV-GODTOOL-910` | **a** | yes | `docs/agileplus/epics/civ-w1-voxel-render.md:19` — Material brush writes the voxel field. |
| 35 | `FR-CIV-GODTOOL-911` | **a** | yes | `docs/agileplus/epics/civ-w1-voxel-render.md:20` — Spawn brush seeds DNA-bearing life. |
| 36 | `FR-CIV-GODTOOL-912` | **a** | yes | `docs/agileplus/epics/civ-w1-voxel-render.md:21` — Disaster brush injects physical initial conditions. |
| 37 | `FR-CIV-INFOVIEW-913` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:32` — Society overlays (W4.5). |
| 38 | `FR-CIV-INFOVIEW-914` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:33` — Infrastructure overlays (W4.6). |
| 39 | `FR-CIV-INFOVIEW-920` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:34` — Live legends and update cadence (W4.7). |
| 40 | `FR-CIV-INSPECT-901` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:36` — Agent, settlement, material inspector fields (W4.9). |
| 41 | `FR-CIV-INSPECT-920` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:37` — Hover tooltips and follow-cam history jump (W4.10). |
| 42 | `FR-CIV-NOTIFY-901` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:26` — Event and alert feed (W6.4 second half). |
| 43 | `FR-CIV-NOTIFY-920` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:28` — Progressive onboarding and tool discovery (W6.6). |
| 44 | `FR-CIV-NOTIFY-921` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:29` — Rebindable hotkey map (W6.7). |
| 45 | `FR-CIV-PLANET-030` | **a** | no | `crates/engine/src/engine.rs:437` (weather_grid) + `crates/planet/src/weather.rs` — per-region weather grid updated by `phase_planet` each tick. Real engine state, no formal spec doc. |
| 46 | `FR-CIV-PLANET-060` | **a** | no | `crates/engine/src/hash_chain.rs:5,129,235` + `replay.rs:42` — hash chain folds in climate + weather-grid + geology; replay digest changes on any ClimateFrame delta. |
| 47 | `FR-CIV-PROTO-002` | **a** | yes | `docs/specs/CIV-0200-client-protocol.md:265` — server architecture FR-CIV-PROTO-001..015. |
| 48 | `FR-CIV-PSYCHE-911` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:14` — beliefs/norms drift and diffuse across social graph. |
| 49 | `FR-CIV-PSYCHE-921` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:17` — psyche/social/history LOD-tiered (Hot/Cold). |
| 50 | `FR-CIV-ROAD-921` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:16` — district designation as lens/hint over emergent land-use. |
| 51 | `FR-CIV-RTS-001` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:1313,1316` + `docs/reference/FR_TRACKER.md:16` — unit movement command (Q/Rally). |
| 52 | `NFR-CIV-PERF-902` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:28` — async streaming + meshing off render thread (W5.7). |
| 53 | `NFR-CIV-SCALE-910` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:25` — LOD-tiered agent simulation (W5.4). |
| 54 | `FR-CIV-AI-002` | **a** | yes | `crates/ai/src/providers/local_slm.rs:1` — `LocalSlmProvider` (mistral.rs, GGUF Q4_K_M). |
| 55 | `FR-CIV-AI-008` | **a** | yes | `crates/ai/src/pool.rs:1` — async worker pool, dedicated tokio runtime; sim never awaits. |
| 56 | `FR-CIV-AI-009` | **a** | yes | `crates/ai/src/preflight.rs:1` — loud-failure preflight for required model artifacts. |
| 57 | `FR-CIV-ASSET-001` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:80,2427` — SVG template rendering pipeline. |
| 58 | `FR-CIV-ASSET-020` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:81,2619` — Template validation pre-commit hook. |
| 59 | `FR-CIV-CORE-019` | **a** | yes | `docs/specs/CIV-0001-core-simulation-loop.md:957` + `AGILE_WORKSTREAM.md:196` — ECS entity model (dense arrays, zero-copy queries). |
| 60 | `FR-CIV-ECON-002-JOULE` | **a** | no | `docs/guides/COPILOT_L3_AGENTS.md:92,474` — Joule allocator (`crates/economy/src/joule.rs`); energy conservation test required. |
| 61 | `FR-CIV-INSPECT-902` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:36` — inspector fields (W4.9 second bucket). |
| 62 | `FR-CIV-INSPECT-903` | **a** | yes | `docs/agileplus/epics/civ-w4-perception.md:36` — inspector fields (W4.9 third bucket). |
| 63 | `FR-CIV-LEGENDS-GRAPH-01` | **a** | yes | `crates/legends/src/lib.rs:16` + `legends/tests/saga_graph.rs:2` + `docs/design/legends-engine.md:436` — `petgraph::StableDiGraph` saga graph with entity/event nodes + typed edges. |
| 64 | `FR-CIV-MOD-009` | **a** | yes | `docs/design/modding-platform.md:35,73,260` — charter validator rejects hardcoded-outcome mods (faction/market/city-scripts). |
| 65 | `FR-CIV-MOD-011` | **a** | yes | `docs/design/modding-platform.md:37,120,299` — semver dependency + version model; resolver builds graph; loud-fail on missing. |
| 66 | `FR-CIV-NOTIFY-911` | **a** | yes | `docs/agileplus/epics/civ-w6-ui.md:27` — Statistics dashboards (W6.5 second half). |
| 67 | `FR-CIV-PSYCHE-030` | **a** | yes | `docs/design/psyche-social.md:162,257,282` — directed weighted ties accumulate from `Cooperated`/`Defected` events; AC-4 + AC-5 spec it. |
| 68 | `FR-CIV-PSYCHE-900` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:11` — emergent psyche state (drives, temperament, mood, bounded memory). |
| 69 | `FR-CIV-PSYCHE-901` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:12` — mood is measured function of need+memory+env+social. |
| 70 | `FR-CIV-PSYCHE-910` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:13` — kinship + contact social graph emerge from co-location/repro/interaction. |
| 71 | `FR-CIV-PSYCHE-912` | **a** | yes | `docs/specs/requirements/FR-CIV-PSYCHE.md:15` — language emerge and drift over contact networks. |
| 72 | `FR-CIV-ROAD-900` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:11` — desire-path emergence (Manor Lords model). |
| 73 | `FR-CIV-ROAD-901` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:12` — self-organizing structure construction (needs+resources+labor). |
| 74 | `FR-CIV-ROAD-902` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:13` — shared data tags across authored/emergent structures. |
| 75 | `FR-CIV-ROAD-910` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:14` — vehicles/transport emerge on road network. |
| 76 | `FR-CIV-ROAD-920` | **a** | yes | `docs/specs/requirements/FR-CIV-ROAD.md:15` — manual road tools (place/curve/snap/upgrade). |
| 77 | `FR-CIV-RTS-002` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:1314,1315` — unit combat & attack orders (W/Fortify). |
| 78 | `FR-CIV-WEB-003` | **a** | yes | `crates/engine/src/spectator.rs:1` — Read-only spectator view (P-U1); `web/src/snapshotView.mjs:2` derives scene counts. |
| 79 | `NFR-CIV-AI-001` | **a** | yes | `crates/ai/src/pool.rs:1` — pool isolation property: sim never awaits. |
| 80 | `NFR-CIV-AI-003` | **a** | yes | `crates/ai/src/lib.rs:13,205` — cache mandatory for cost/latency; `cached_generate` wrapper. |
| 81 | `NFR-CIV-PERF-900` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:27` — 60fps target + draw-call scaling (W5.6). |
| 82 | `NFR-CIV-SCALE-901` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:23` — active working set streaming + eviction (W5.2). |
| 83 | `NFR-CIV-SCALE-902` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:24` — compact on-disk chunk + LOD format (W5.3). |
| 84 | `NFR-CIV-SCALE-920` | **a** | yes | `docs/agileplus/epics/civ-w5-scale.md:26` — determinism across LOD/streaming transitions (W5.5). |
| 85 | `FR-CIV-AI-004` | **a** | yes | `crates/ai/src/providers/firepass_kimi.rs:1` — `FirepassKimiProvider` wraps `civ-research::FirepassKimiClient`. |
| 86 | `FR-CIV-AI-005` | **a** | yes | `crates/ai/src/providers/embed.rs:1` — `EmbedProvider` (fastembed-rs/ort, MiniLM 384-dim). |
| 87 | `FR-CIV-AI-010` | **a** | yes | `crates/ai/src/config.rs:1` — `.env`-driven config (no hardcoded paths/keys). |
| 88 | `FR-CIV-AI-011` | **a** | yes | `docs/design/civ-ai-crate.md:43` — naming service (grammar+Markov inline; SLM batch-seed). |
| 89 | `FR-CIV-AI-012` | **a** | yes | `docs/design/civ-ai-crate.md:44` — legends narration service (epoch-digest → SLM prose, hash-cached). |
| 90 | `FR-CIV-AI-014` | **a** | yes | `docs/design/civ-ai-crate.md:46` — chatter/headlines service (fixed-persona SLM, event-triggered, rate-limited). |
| 91 | `FR-CIV-ASSET-012` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2539,3216` — Atlas load time (<500ms). |
| 92 | `FR-CIV-ASSET-015` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:1714,2569,3219` — Atlas `Cache-Control: immutable` headers. |
| 93 | `FR-CIV-BEVY-016` | **a** | no | `clients/bevy-ref/README.md:26` + `docs/development-guide/p-w1-kickoff.md:127,711` + `justfile:127` — Live attach smoke harness v2 (item 41): runs `live_stream::` + minimap UV lib tests + both Bevy bins. |
| 94 | `FR-CIV-BEVY-022` | **a** | no | `clients/bevy-ref/README.md:26` + `docs/development-guide/p-w1-kickoff.md:133,737` + `justfile:127` — Live attach smoke harness v3 (item 47): `live_focus::` and `live_minimap::` lib tests. |
| 95 | `FR-CIV-BEVY-025` | **a** | no | `clients/bevy-ref/README.md:26` + `docs/development-guide/p-w1-kickoff.md:136,740` + `justfile:127` — Live attach smoke harness v4 (item 50): `live_pick::` lib tests. |
| 96 | `FR-CIV-CORE-004` | **a** | yes | `docs/reference/CODE_ENTITY_MAP.md:9` + `docs/specs/CIV-0001-core-simulation-loop.md:882` — Sub-16ms tick time + I/O event logging. |
| 97 | `FR-CIV-CORE-005` | **a** | yes | `docs/specs/CIV-0001-core-simulation-loop.md:887` — BTreeMap ordered iteration (no HashMap in critical paths). |
| 98 | `FR-CIV-CORE-013` | **a** | yes | `docs/specs/CIV-0001-core-simulation-loop.md:927` — phase schedule integrity (Command → Policy → Transition → Stochastic → Metrics → Broadcast). |
| 99 | `FR-CIV-ECON-004` | **a** | no | `docs/reference/CODE_ENTITY_MAP.md:8` + `docs/reference/FR_TRACKER.md:10` — Policy-driven fiscal control (`crates/engine/src/policy.rs`). Real code, no formal spec doc yet. |
| 100 | `FR-CIV-EMERGENCE-001` | **a** | no | `docs/guides/voxel-emergent-vision-and-migration.md:97,133,137` — Abiogenesis threshold: engine scans CA state each tick; proto-life event on co-occurring material conditions. Code path is in `civ-diffusion`+`civ-engine`; no formal spec doc |

## Spec stubs to append

`12` new one-line spec stubs go to
`agileplus-specs/civ-021-recovered-requirements/spec.md`. Each stub names the FR ID,
a one-line description, and the strongest evidence file:line.

- **FR-CIV-ECON-001-MARKET** — Market price tracking (Market::record_transaction, update_prices, get_price) in civ-economy. RENAME-suggest: collapse to FR-CIV-ECON-001.
- **FR-CIV-PLANET-020** — Tide-driven coastal water columns (engine.rs:434).
- **FR-CIV-UX-005** — Era / overview camera presets (Cam Wide / Cam Close) across Godot + Web + Unreal.
- **FR-CIV-ACT-001** — Citizen lifecycle (birth/init/age/death) — see CIV-0103 spec for full body; covered by `crates/engine/src/engine.rs`.
- **FR-CIV-PLANET-030** — Per-region weather grid (engine.rs:437).
- **FR-CIV-PLANET-060** — Climate+weather+geology folded into replay hash chain.
- **FR-CIV-ECON-002-JOULE** — Joule allocator with energy conservation in civ-economy/src/joule.rs.
- **FR-CIV-BEVY-016** — Live attach smoke harness v2 — live_stream + minimap UV + both Bevy bins.
- **FR-CIV-BEVY-022** — Live attach smoke harness v3 — live_focus + live_minimap lib tests.
- **FR-CIV-BEVY-025** — Live attach smoke harness v4 — live_pick lib tests.
- **FR-CIV-ECON-004** — Policy-driven fiscal control via crates/engine/src/policy.rs.
- **FR-CIV-EMERGENCE-001** — Abiogenesis threshold + proto-life event.

## RENAME mappings (verdict c)

| Old ID | → | New (existing) ID | Rationale |
|--------|---|-------------------|-----------|
| `FR-CIV-ECON-002` | → | `FR-CIV-ECON-002-JOULE` | `docs/guides/COPILOT_L3_AGENTS.md:92,271,474` — Joule allocator (energy conservation). Maps to `FR-CIV-ECON-002-JOULE` (the hyphenated form has its own row in t |

## Notes on method

- The matrix's `status: CODE-ONLY-no-spec` is the *matrix's* view, which only checks for
  `docs/specs/requirements/*.md` and `docs/traceability/*.md`. The triage here also
  recognises `docs/specs/CIV-*.md`, `docs/agileplus/epics/civ-w*.md`, `docs/design/*.md`,
  and `docs/development-guide/*.md` as real spec homes.
- IDs in the FR-CIV-INFOVIEW-9xx / -INSPECT-9xx / -NOTIFY-9xx / -PSYCHE-9xx / -ROAD-9xx /
  NFR-CIV-SCALE-9xx / NFR-CIV-PERF-9xx / FR-CIV-GODTOOL-9xx series are all real user-
  meaningful requirements that the matrix undercounted because their spec lives in a
  table-cell row (e.g. `docs/specs/requirements/FR-CIV-PSYCHE.md` line 11) rather than a
  one-ID-per-section doc. These are marked `cov: yes` and do NOT need a new stub.
- FR-CIV-0001-TICK is the lone STALE candidate with a non-trivial ref count: it appears
  only in agent-guide commit-message templates and the worktree guide, never in actual
  code. It is mapped to FR-CIV-CORE-001 (tick monotonicity, real impl in `engine.rs`).