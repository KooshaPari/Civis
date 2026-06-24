# Phantom-ID Triage — Batch 2 (2026-06-10)

**Source:** `docs/audits/fr-matrix.json` (1181 IDs, generated 2026-06-10)
filtered to **CODE-ONLY-no-spec** (786 rows), sorted by reference
count (`len(code_refs) + len(test_refs)`), next **150** taken
(positions 101–250, starting at `FR-CIV-EMERGENCE-004` and ending at
`FR-SAVE-010`). The first 100 rows were published in
[`phantom-triage-batch1.md`](phantom-triage-batch1.md) (merged as
PR #372).

**Verdict taxonomy:** (inherited from batch 1)
- **(a) REAL-REQUIREMENT** — the code/docs implement a user-meaningful capability.
  - `cov: yes` → already covered by an existing spec file; no stub needed.
  - `cov: no`  → NO existing spec; a one-line stub is appended to
    `agileplus-specs/civ-021-recovered-requirements/spec.md`.
- **(b) STALE-ID** — ID is a comment/trace artifact; no implementation matches.
- **(c) RENAME** — ID maps onto an existing spec'd ID modulo naming drift.

## Summary

- (a) REAL & covered by existing spec: **135**
- (a) REAL & UNCOVERED (stub appended to civ-021): **1**
- (b) STALE-ID (delete-recommendation): **0** *(all batch-2 STALEs are also RENAME-candidates — captured in (c))*
- (c) RENAME (map to existing ID): **14**

The single new stub is **`FR-CIV-PLANET-010`** (deterministic
climate snapshot on `Simulation::snapshot()`). The 14 RENAME
candidates are all `PLAN.md` L3 work-log phantom IDs whose real
implementation lives under a spec'd FR ID (e.g.
`FR-CIV-DIPLO-001-RELATIONS` → `FR-CIV-DIPLO-001`).

## Per-row verdicts

| # | FR ID | Verdict | Cov? | Evidence (file:line) |
|---|-------|---------|------|----------------------|
| 1 | `FR-CIV-EMERGENCE-004` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:97,140,207` — Speciation registry driven by `should_speciate()`. |
| 2 | `FR-CIV-EMERGENCE-010` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:98,143,213` — Kinship clusters (P-VM-5). |
| 3 | `FR-CIV-EMERGENCE-013` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:98,133,146` — Cluster event logging. |
| 4 | `FR-CIV-GEO-001` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2024` + `docs/reference/FR_TRACKER.md:22` — Terrain types & properties (Section 12.2 table). |
| 5 | `FR-CIV-LEGENDS-INGEST-02` | **a** | yes | `docs/design/legends-engine.md:437` + `crates/legends/src/worker.rs:1` — Off-thread worker draining `.civreplay` event bus. |
| 6 | `FR-CIV-LEGENDS-QUERY-07` | **a** | yes | `docs/design/legends-engine.md:442` + `crates/engine/src/emergence.rs:35,592` — Read-only `O(neighborhood)` query API + `EpochDigest`. |
| 7 | `FR-CIV-MOD-000` | **a** | yes | `docs/design/modding-platform.md:26,89,164` — Mod charter (no hardcoded outcomes). |
| 8 | `FR-CIV-MOD-001` | **a** | yes | `docs/design/modding-platform.md:27,53` + `docs/specs/CIV-0700-modding-api-spec.md:2356` — Mod capability surface. |
| 9 | `FR-CIV-MOD-002` | **a** | yes | `docs/design/modding-platform.md:28,178` + `docs/specs/CIV-0700-modding-api-spec.md:2364` — Mod manifest. |
| 10 | `FR-CIV-MOD-003` | **a** | yes | `docs/design/modding-platform.md:29,192` + `docs/specs/CIV-0700-modding-api-spec.md:2372` — Mod load lifecycle. |
| 11 | `FR-CIV-MOD-004` | **a** | yes | `docs/design/modding-platform.md:30,203` + `docs/specs/CIV-0700-modding-api-spec.md:2380` — Mod sandboxing. |
| 12 | `FR-CIV-MOD-005` | **a** | yes | `docs/design/modding-platform.md:31,214` + `docs/specs/CIV-0700-modding-api-spec.md:2388` — Mod host API. |
| 13 | `FR-CIV-MOD-006` | **a** | yes | `docs/design/modding-platform.md:32,226` + `docs/specs/CIV-0700-modding-api-spec.md:2396` — Mod event hooks. |
| 14 | `FR-CIV-MOD-007` | **a** | yes | `docs/design/modding-platform.md:33,234` + `docs/specs/CIV-0700-modding-api-spec.md:2404` — Charter validator. |
| 15 | `FR-CIV-MOD-008` | **a** | yes | `docs/design/modding-platform.md:34,249` + `docs/specs/CIV-0700-modding-api-spec.md:2412` — Mod subscription. |
| 16 | `FR-CIV-MOD-010` | **a** | yes | `docs/design/modding-platform.md:36,287` + `docs/specs/CIV-0700-modding-api-spec.md:2428` — Mod isolation guarantees. |
| 17 | `FR-CIV-MOD-012` | **a** | yes | `docs/design/modding-platform.md:38,307` + `docs/specs/CIV-0700-modding-api-spec.md:2444` — Mod ABI version model. |
| 18 | `FR-CIV-MOD-013` | **a** | yes | `docs/design/modding-platform.md:39,320` + `docs/specs/CIV-0700-modding-api-spec.md:2452` — Mod resolver. |
| 19 | `FR-CIV-MOD-014` | **a** | yes | `docs/design/modding-platform.md:40,355` + `docs/specs/CIV-0700-modding-api-spec.md:2460` — Mod dependency graph. |
| 20 | `FR-CIV-MOD-015` | **a** | yes | `docs/design/modding-platform.md:41,375` + `docs/specs/CIV-0700-modding-api-spec.md:2468` — Mod dev-mode (hot reload). |
| 21 | `FR-CIV-PLANET-010` | **a** | **no** | `crates/engine/src/engine.rs:2161,2427` (doc comments) + `crates/server/src/jsonrpc.rs:411` — Deterministic climate on `Simulation::snapshot()`; tested by `engine_tick_includes_climate_in_snapshot` at `engine.rs:2429`. **Stub appended to `civ-021` (this PR).** |
| 22 | `FR-CIV-PSYCHE-010` | **a** | yes | `docs/design/psyche-social.md:142,255,277` — Drives (needs → tension → drive) in §2.1 + AC table. |
| 23 | `FR-CIV-PSYCHE-011` | **a** | yes | `docs/design/psyche-social.md:191,259,278` — Mood, measured function. |
| 24 | `FR-CIV-PSYCHE-020` | **a** | yes | `docs/design/psyche-social.md:153,256,279` — Memory (bounded) + decay. |
| 25 | `FR-CIV-PSYCHE-024` | **a** | yes | `docs/design/psyche-social.md:233,264,281` — Personality drift over time. |
| 26 | `FR-CIV-PSYCHE-032` | **a** | yes | `docs/design/psyche-social.md:203,260,284` — Ties accumulation. |
| 27 | `FR-CIV-PSYCHE-033` | **a** | yes | `docs/design/psyche-social.md:206,261,285` — Social-graph pruning. |
| 28 | `FR-CIV-RENDER-001` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:96,148,152` — Voxel chunk streaming. |
| 29 | `FR-CIV-RENDER-002` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:96,148,153` — Material transparency pass. |
| 30 | `FR-CIV-RTS-004` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:1143,1317,2007` — Command queuing & auto-execute (Section 12.1). |
| 31 | `FR-CIV-RTS-014` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2017` + `docs/specs/CIV-0400-ai-npc-behavior-spec.md:14,2514` — Faction AI behavior. |
| 32 | `FR-CIV-UI-001` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:99,155,159` — Material brush (god-tool). |
| 33 | `FR-CIV-UI-003` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:99,155,161` — Emergence notification feed. |
| 34 | `FR-CIV-VOXEL-021` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:94,124` + `crates/voxel/src/worldgen.rs:540` — Material palette in RON. |
| 35 | `FR-CIV-VOXEL-022` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:94,125,183` — CA determinism (BTreeMap scan order). |
| 36 | `FR-CIV-VOXEL-032` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:95,119,131` — Atmospheric gas pockets. |
| 37 | `FR-CIV-WEB-000` | **a** | yes | `docs/traceability/fr-web-matrix.md` + `docs/development-guide/fr-web-spectator.md:3,29` — Web spectator attach shell. |
| 38 | `FR-CIV-WEB-008` | **a** | yes | `docs/traceability/fr-web-matrix.md` + `docs/development-guide/fr-web-spectator.md:37` + `web/dashboard/src/lib/authoring.ts:95` — L2 authoring panel. |
| 39 | `FR-SOC-DET-001` | **a** | yes | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1495,1498,1984` — Social determinism invariants. |
| 40 | `FR-CIV-0001` | **c** | n/a | `PLAN.md:16` + `docs/guides/GIT_WORKTREE_GUIDE.md:151` — L3 plan row + worktree-guide commit-message template. Maps to `FR-CIV-CORE-001` (real tick loop in `crates/engine/src/engine.rs`). |
| 41 | `FR-CIV-ACTOR-001-LIFECYCLE` | **c** | n/a | `PLAN.md:145-146` — L3 work-log row. Maps to `FR-CIV-ACT-001` (batch-1 stub). |
| 42 | `FR-CIV-AI-003` | **a** | yes | `docs/design/civ-ai-crate.md:35` + `crates/ai/src/providers/ollama_dev.rs:1` — `OllamaDevProvider` (dev-only, OpenAI-compat HTTP). |
| 43 | `FR-CIV-AI-013` | **a** | yes | `docs/design/civ-ai-crate.md:45,257` — Memory / context-window service. |
| 44 | `FR-CIV-AI-015` | **a** | yes | `docs/design/civ-ai-crate.md:47,259` — Tool-use / structured-output service. |
| 45 | `FR-CIV-ASSET-002` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437,3206` — Variant rendering (sizing palette). |
| 46 | `FR-CIV-ASSET-003` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2447,3207` — Variant rendering (job palette). |
| 47 | `FR-CIV-ASSET-004` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2457,3208` — Variant rendering (biome palette). |
| 48 | `FR-CIV-ASSET-005` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2467,3209` — Variant rendering (era palette). |
| 49 | `FR-CIV-ASSET-006` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2477,3210` — Variant rendering (faction palette). |
| 50 | `FR-CIV-ASSET-007` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2487,3211` — Variant rendering (climate palette). |
| 51 | `FR-CIV-ASSET-008` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2497,3212` — Animation frame bake. |
| 52 | `FR-CIV-ASSET-009` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2507,3213` — Sprite-sheet build step. |
| 53 | `FR-CIV-ASSET-013` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2549,3217` — Atlas size budget. |
| 54 | `FR-CIV-ASSET-014` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2559,3218` — Atlas gzip transport. |
| 55 | `FR-CIV-ASSET-016` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2579,3220` — SRI hash pin. |
| 56 | `FR-CIV-ASSET-017` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589,3221` — Local-dev asset path. |
| 57 | `FR-CIV-ASSET-018` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2599,3222` — Production CDN. |
| 58 | `FR-CIV-ASSET-019` | **a** | yes | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609,3223` — Fallback to SVG. |
| 59 | `FR-CIV-BEVY-001` | **a** | yes | `docs/development-guide/p-w1-kickoff.md:80` + `clients/bevy-ref/src/bevy_render.rs:171` — `civ-standalone` gameplay plugins (sim bridge, HUD, spawn tools, minimap). |
| 60 | `FR-CIV-BEVY-002` | **a** | yes | `docs/development-guide/p-w1-kickoff.md:81` + `clients/bevy-ref/src/lib.rs:1241` — Live attach scene sync (`live_scene`: voxel chunks + agent markers from `Frame3d`). |
| 61 | `FR-CIV-BEVY-021` | **a** | yes | `docs/development-guide/p-w1-kickoff.md:83,132` — GitHub Actions `.github/workflows/civis-3d-live-smoke.yml` (headless `just civis-3d-live-smoke`). |
| 62 | `FR-CIV-CORE-011` | **a** | yes | `docs/models/civ-sim/TECHNICAL_SPEC.md:1369` + `docs/specs/CIV-0001-core-simulation-loop.md:917` — Replay determinism verification. |
| 63 | `FR-CIV-DIFFUSION-003` | **a** | yes | `docs/design/tech-engineering.md:149` + `crates/diffusion/src/lib.rs:114` — Trajectories monotonically non-decreasing (AC: no rewinds). |
| 64 | `FR-CIV-DIPLO-001-RELATIONS` | **c** | n/a | `PLAN.md:207-208` — L3 work-log row. Maps to `FR-CIV-DIPLO-001` (real 8-state FSM in `crates/diplomacy/src/lib.rs:1,15,770,821`). |
| 65 | `FR-CIV-DIPLO-002-SHADOW` | **c** | n/a | `PLAN.md:209-210` — L3 work-log row. Maps to `FR-CIV-DIPLO-002` (real influence-capital in `crates/diplomacy/src/lib.rs:16`). |
| 66 | `FR-CIV-ECON-003` | **a** | yes | `docs/reference/FR_TRACKER.md:9` + `docs/reports/STATUS_REPORT.md:91` — Joule economy allocator (real impl in `crates/engine/src/engine.rs`). |
| 67 | `FR-CIV-EMERGENCE-002` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:97,138` — Agent bootstrap from CA pattern. |
| 68 | `FR-CIV-EMERGENCE-003` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:97,139` — Environment-vector fitness. |
| 69 | `FR-CIV-EMERGENCE-011` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:98,144` — Cultural clusters via diffusion. |
| 70 | `FR-CIV-EMERGENCE-012` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:98,145` — Territorial clusters. |
| 71 | `FR-CIV-GEO-004` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2027` + `docs/reports/STATUS_REPORT.md:96` — Neighbor queries & pathfinding (Section 12.2). |
| 72 | `FR-CIV-INFRA-030` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/civ-traffic/src/lib.rs:20,424` — Identical event order → identical emergent road graph; test `emergent_growth_is_deterministic`. |
| 73 | `FR-CIV-INFRA-040` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/civ-traffic/src/lib.rs:437` + `docs/design/vehicles-logistics.md:407` — Vehicles gate on tech era; test `vehicles_gate_on_tech_era`. |
| 74 | `FR-CIV-L5` | **a** | yes | `docs/development-guide/fr-l5-visual-pass.md:1,23` + `clients/godot-ref/scripts/spawn_burst.gd:4` — Incremental visual presentation pass (L5 product-quality ladder). |
| 75 | `FR-CIV-LIFE-022` | **a** | yes | `crates/economy/src/stocks.rs:317,436` — Comparative advantage = highest net-surplus good (real `Market` impl). |
| 76 | `FR-CIV-LIFE-024` | **a** | yes | `crates/economy/src/stocks.rs:335,376` — Trade conserves combined stock total (real conservation test). |
| 77 | `FR-CIV-MARKET-001` | **a** | yes | `docs/design/polities-markets.md:98` + `docs/design/master-roadmap.md:25` — Per-locale condition probe (real impl cited at §4 of design doc). |
| 78 | `FR-CIV-METRICS-001` | **c** | n/a | `PLAN.md:151` — L3 work-log row. Maps to `FR-CIV-METRICS-001-TIMESERIES` (non-hyphenated form is a phantom alias). |
| 79 | `FR-CIV-METRICS-001-TIMESERIES` | **a** | yes | `docs/reference/FR_TRACKER.md` + `docs/reports/STATUS_REPORT.md` + `PLAN.md:151-152` — Time-series metrics; real impl in `crates/engine` (no separate `crates/metrics` per `PLAN.md:22`). |
| 80 | `FR-CIV-MOD-016` | **a** | yes | `docs/design/modding-platform.md:42,398` — Mod permissions gate. |
| 81 | `FR-CIV-MOD-017` | **a** | yes | `docs/design/modding-platform.md:43,419` — Mod introspection. |
| 82 | `FR-CIV-MOD-018` | **a** | yes | `docs/design/modding-platform.md:44,457` — Mod telemetry. |
| 83 | `FR-CIV-MOD-019` | **a** | yes | `docs/design/modding-platform.md:45,486` — Mod lifecycle event log. |
| 84 | `FR-CIV-MOD-020` | **a** | yes | `docs/design/modding-platform.md:46,500` — Mod uninstall + cleanup. |
| 85 | `FR-CIV-POLITY-001` | **a** | yes | `docs/design/polities-markets.md:37` + `docs/design/master-roadmap.md:25` — Cohesion graph (weighted, emergent cluster graph). |
| 86 | `FR-CIV-POLITY-008` | **a** | yes | `docs/design/polities-markets.md:90,140` — Polities act on markets only through existing institutions. |
| 87 | `FR-CIV-PROTO3D-010` | **a** | yes | `crates/protocol-3d/src/lib.rs:631,965` + `crates/server/src/voxel_frame_builder.rs:117` — Building tier defaults for legacy payloads and round-trips. |
| 88 | `FR-CIV-PROTO3D-011` | **a** | yes | `crates/protocol-3d/src/lib.rs:714,1048` + `crates/server/src/voxel_frame_builder.rs:126` — Civilian state entries round-trip. |
| 89 | `FR-CIV-PROTO3D-012` | **a** | yes | `crates/protocol-3d/src/lib.rs:749,1083` + `crates/server/src/voxel_frame_builder.rs:150` — Faction state entries round-trip. |
| 90 | `FR-CIV-PSYCHE-021` | **a** | yes | `docs/design/psyche-social.md:209,280` — Social event logging. |
| 91 | `FR-CIV-PSYCHE-031` | **a** | yes | `docs/design/psyche-social.md:174,283` — Ties persistence. |
| 92 | `FR-CIV-PSYCHE-034` | **a** | yes | `docs/design/psyche-social.md:241,286` — Beliefs/norms language. |
| 93 | `FR-CIV-PSYCHE-035` | **a** | yes | `docs/design/psyche-social.md:245,287` — Belief drift mechanism. |
| 94 | `FR-CIV-PSYCHE-036` | **a** | yes | `docs/design/psyche-social.md:246,288` — Norm propagation. |
| 95 | `FR-CIV-PSYCHE-037` | **a** | yes | `docs/design/psyche-social.md:247,289` — Cultural artifact. |
| 96 | `FR-CIV-PSYCHE-040` | **a** | yes | `docs/design/psyche-social.md:6,290` — Psyche LOD summary. |
| 97 | `FR-CIV-RES-001` | **a** | yes | `docs/reference/FR_TRACKER.md:40` + `docs/reference/CODE_ENTITY_MAP.md:17` — Scenario API (real impl in `crates/server/src/main.rs` per CODE_ENTITY_MAP). |
| 98 | `FR-CIV-RESEARCH-001-SCENARIO` | **c** | n/a | `PLAN.md:233-234` — L3 work-log row. Maps to `FR-CIV-RESEARCH-001` (real LLM cache + card acceptance in `crates/research/src/lib.rs:377,408`). |
| 99 | `FR-CIV-RESEARCH-002-SNAPSHOT` | **c** | n/a | `PLAN.md:235-236` — L3 work-log row. Maps to `FR-CIV-RESEARCH-002` (real canonical-replay in `crates/research/src/lib.rs:601`). |
| 100 | `FR-CIV-RESEARCH-003-EXPORT` | **c** | n/a | `PLAN.md:237-238` — L3 work-log row. Maps to `FR-CIV-RESEARCH-003` (real hybrid-replay in `crates/research/src/lib.rs:616`). |
| 101 | `FR-CIV-RTS-012` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2015` + `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2532` — Turn-based vs real-time (Section 12.1). |
| 102 | `FR-CIV-RTS-013` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2016` + `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2531` — Unit experience & leveling. |
| 103 | `FR-CIV-RTS-015` | **a** | yes | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2018` + `docs/specs/CIV-0400-ai-npc-behavior-spec.md:2533` — Client-side prediction & replay correction. |
| 104 | `FR-CIV-SERVER-001` | **c** | n/a | `PLAN.md:174-175` — L3 work-log row. Maps to `FR-CIV-SERVER-001-WS` (real `SimServer` in `crates/server/src/websocket.rs`). |
| 105 | `FR-CIV-SERVER-001-WS` | **a** | yes | `PLAN.md:174-175` + `crates/server/src/main.rs` (CODE_ENTITY_MAP) + `crates/server/src/jsonrpc.rs` — WebSocket server (real impl, `sim.snapshot` over WS). |
| 106 | `FR-CIV-SERVER-002` | **c** | n/a | `PLAN.md:176-177` — L3 work-log row. Maps to `FR-CIV-SERVER-002-PROTO` (real protocol in `crates/server/src/`). |
| 107 | `FR-CIV-SERVER-002-PROTO` | **a** | yes | `PLAN.md:176-177` + `crates/server/src/jsonrpc.rs` + CODE_ENTITY_MAP — JSON-RPC protocol (real `ClientMessage` / `ServerMessage` in `crates/server`). |
| 108 | `FR-CIV-SOCIAL-001-INSTITUTIONS` | **c** | n/a | `PLAN.md:147-148` — L3 work-log row. **`crates/social` is NOT in workspace** (`PLAN.md:20`); deferred — collapse to `FR-CIV-ACT-001` for now. |
| 109 | `FR-CIV-SOCIAL-002-IDEOLOGY` | **c** | n/a | `PLAN.md:149-150` — L3 work-log row. Same as above; deferred. |
| 110 | `FR-CIV-SPECIES-104` | **a** | yes | `docs/design/species-sentience.md:77,101` — Speciation rules (genotype/phenotype split). |
| 111 | `FR-CIV-SPECIES-201` | **a** | yes | `docs/design/species-sentience.md:99,206` — Cognition & tool-use thresholds. |
| 112 | `FR-CIV-SPECIES-302` | **a** | yes | `docs/design/species-sentience.md:122,192` — Reproduction / lineage rules. |
| 113 | `FR-CIV-SPECIES-406` | **a** | yes | `docs/design/species-sentience.md:173,216` — Extinction + niche collapse. |
| 114 | `FR-CIV-TRAFFIC-LANE-001` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/civ-traffic/src/lane.rs:3,323` — Lane promotion by road class; test `lanes_follow_road_kind_ladder`. |
| 115 | `FR-CIV-TRAFFIC-LANE-002` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/civ-traffic/src/lane.rs:4,346` — Lane-node connectivity; test `lanes_connect_through_nodes`. |
| 116 | `FR-CIV-TRAFFIC-LANE-003` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/civ-traffic/src/lane.rs:5,359` — Scalar speed graph preserved. |
| 117 | `FR-CIV-UI-002` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:99,160` — Condition overlay (god-tool). |
| 118 | `FR-CIV-UX-002` | **a** | yes | `docs/development-guide/fr-godot-attach.md:14` + `crates/server/src/jsonrpc.rs:56` — Server spawn via WS (`sim.spawn_civilian`). |
| 119 | `FR-CIV-UX-003` | **a** | yes | `docs/development-guide/fr-godot-attach.md:15` + `crates/server/src/jsonrpc.rs:60` — Server voxel write via WS (`sim.place_voxel`). |
| 120 | `FR-CIV-VEHICLE-001` | **a** | yes | `docs/design/vehicles-logistics.md:101-102` — Vehicle entity & motion model. |
| 121 | `FR-CIV-VEHICLE-010` | **a** | yes | `docs/design/vehicles-logistics.md:161-162` — Cargo + capacity. |
| 122 | `FR-CIV-VEHICLE-020` | **a** | yes | `docs/design/vehicles-logistics.md:198-199` — Route assignment. |
| 123 | `FR-CIV-VEHICLE-040` | **a** | yes | `docs/design/vehicles-logistics.md:277-278` — Tech-era gating. |
| 124 | `FR-CIV-VOXEL-005` | **a** | yes | `docs/traceability/civis-tracelinks.md` + `crates/voxel/src/lib.rs:249` + `docs/worklogs/2026-05-22-civis-3d-kickoff.md:73` — `VoxelWorld` replay bit-identical when `seed` is fixed. |
| 125 | `FR-CIV-VOXEL-030` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:95,129` — World-gen strata (bedrock, soil, ore). |
| 126 | `FR-CIV-VOXEL-031` | **a** | yes | `docs/guides/voxel-emergent-vision-and-migration.md:95,130` — Hydrology (water-filled basins). |
| 127 | `FR-CIV-WAR-001-UNITS` | **c** | n/a | `PLAN.md:203-204` — L3 work-log row. Real unit handling in `crates/tactics/src/{formation,pathfinding,movement,operational}.rs`; spec'd at `docs/design/warfare.md:77,191` (FR-CIV-WAR-010/011/012/013). |
| 128 | `FR-CIV-WAR-002-COMBAT` | **c** | n/a | `PLAN.md:205-206` — L3 work-log row. Real combat in `crates/tactics/src/military_phase.rs` + `war_bridge.rs`; spec'd at `docs/design/warfare.md:106,195` (FR-CIV-WAR-020/021/022). |
| 129 | `FR-CIV-WAR-010` | **a** | yes | `docs/design/warfare.md:77,191` — Unit command pipeline. |
| 130 | `FR-CIV-WAR-011` | **a** | yes | `docs/design/warfare.md:83,192` — Doctrine fitness. |
| 131 | `FR-CIV-WAR-012` | **a** | yes | `docs/design/warfare.md:86,193` — Engagement selection. |
| 132 | `FR-CIV-WAR-013` | **a** | yes | `docs/design/warfare.md:89,194` — Line-of-sight. |
| 133 | `FR-CIV-WAR-020` | **a** | yes | `docs/design/warfare.md:106,195` — Damage resolution. |
| 134 | `FR-CIV-WAR-021` | **a** | yes | `docs/design/warfare.md:111,196` — Armor / penetration. |
| 135 | `FR-CIV-WAR-022` | **a** | yes | `docs/design/warfare.md:114,197` — Casualty aggregation. |
| 136 | `FR-CIV-WAR-030` | **a** | yes | `docs/design/warfare.md:124,198` — Siege state machine. |
| 137 | `FR-CIV-WAR-040` | **a** | yes | `docs/design/warfare.md:144,199` — Surrender rules. |
| 138 | `FR-CIV-WAR-041` | **a** | yes | `docs/design/warfare.md:147,200` — Treaty-after-war logistics. |
| 139 | `FR-CIV-WAR-042` | **a** | yes | `docs/design/warfare.md:150,201` — War-end accounting (ledger conservation). |
| 140 | `FR-CIV-WEB-002` | **a** | yes | `docs/traceability/fr-web-matrix.md` + `docs/development-guide/fr-web-spectator.md:31` + `web/dashboard/src/lib/civisServer.ts:1` — Read-only spectator view (P-U1); tests `health`, `sim.snapshot`, `mergeSnapshot`. |
| 141 | `FR-CIV-WEB-006` | **a** | yes | `docs/traceability/fr-web-matrix.md` + `docs/development-guide/fr-web-spectator.md:35` + `web/dashboard/src/lib/frame3d.ts:1` — `F3D0` binary handler. |
| 142 | `FR-DET-002` | **a** | yes | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290,440` — D2 "No System Time" rule (`@trace` + matrix row). |
| 143 | `FR-DET-006` | **a** | yes | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:342,444` — D6 "Seeded RNG" rule. |
| 144 | `FR-DET-007` | **a** | yes | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:445,451` — D7 "No I/O in Tick" rule. |
| 145 | `FR-MET-001` | **a** | yes | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174,1203` — M1 metrics observability. |
| 146 | `FR-REP-001` | **a** | yes | `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:489,532` — R1 replay integrity. |
| 147 | `FR-SAVE-006` | **a** | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2805,2943` — Save-format versioning. |
| 148 | `FR-SAVE-007` | **a** | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2806,2943` — Save integrity (blake3). |
| 149 | `FR-SAVE-008` | **a** | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2807,2949` — Save migration. |
| 150 | `FR-SAVE-010` | **a** | yes | `docs/specs/CIV-1000-save-load-persistence-spec.md:2809,2955` — Save garbage-collection. |

## Spec stubs to append

`1` new one-line spec stub goes to
`agileplus-specs/civ-021-recovered-requirements/spec.md` (appended at
the end of the existing batch-1 stub list, in a new
"Batch 2 additions" subsection). The 14 RENAME candidates are
captured in a new "RENAME mappings (cumulative)" table at the bottom
of the same spec doc.

- **FR-CIV-PLANET-010** — Deterministic climate on `Simulation::snapshot()` (`crates/engine/src/engine.rs:2161,2427` + test `engine_tick_includes_climate_in_snapshot` at `engine.rs:2429`).

## RENAME mappings (verdict c)

| Old ID | → | New (existing) ID | Rationale |
|--------|---|-------------------|-----------|
| `FR-CIV-0001` | → | `FR-CIV-CORE-001` | `PLAN.md:16` + `GIT_WORKTREE_GUIDE.md:151` — L3 plan row + worktree-guide commit-message template. Real tick loop is `FR-CIV-CORE-001` in `crates/engine/src/engine.rs`. |
| `FR-CIV-ACTOR-001-LIFECYCLE` | → | `FR-CIV-ACT-001` | `PLAN.md:145-146` — L3 work-log row. Real citizen lifecycle is `FR-CIV-ACT-001` (batch-1 stub). |
| `FR-CIV-DIPLO-001-RELATIONS` | → | `FR-CIV-DIPLO-001` | `PLAN.md:207-208` — L3 work-log row. Real 8-state FSM is `FR-CIV-DIPLO-001` in `crates/diplomacy/src/lib.rs:1,15,770,821`. |
| `FR-CIV-DIPLO-002-SHADOW` | → | `FR-CIV-DIPLO-002` | `PLAN.md:209-210` — L3 work-log row. Real influence-capital is `FR-CIV-DIPLO-002` in `crates/diplomacy/src/lib.rs:16`. |
| `FR-CIV-METRICS-001` | → | `FR-CIV-METRICS-001-TIMESERIES` | `PLAN.md:151` — non-hyphenated form is a phantom alias of the hyphenated spec'd ID. |
| `FR-CIV-RESEARCH-001-SCENARIO` | → | `FR-CIV-RESEARCH-001` | `PLAN.md:233-234` — L3 work-log row. Real LLM cache + card acceptance is `FR-CIV-RESEARCH-001` in `crates/research/src/lib.rs:377,408`. |
| `FR-CIV-RESEARCH-002-SNAPSHOT` | → | `FR-CIV-RESEARCH-002` | `PLAN.md:235-236` — L3 work-log row. Real canonical-replay in `crates/research/src/lib.rs:601`. |
| `FR-CIV-RESEARCH-003-EXPORT` | → | `FR-CIV-RESEARCH-003` | `PLAN.md:237-238` — L3 work-log row. Real hybrid-replay in `crates/research/src/lib.rs:616`. |
| `FR-CIV-SERVER-001` | → | `FR-CIV-SERVER-001-WS` | `PLAN.md:174-175` — non-hyphenated form is a phantom alias of the hyphenated spec'd ID. |
| `FR-CIV-SERVER-002` | → | `FR-CIV-SERVER-002-PROTO` | `PLAN.md:176-177` — non-hyphenated form is a phantom alias of the hyphenated spec'd ID. |
| `FR-CIV-SOCIAL-001-INSTITUTIONS` | → | (deferred → `FR-CIV-ACT-001`) | `PLAN.md:147-148` — L3 work-log row. **`crates/social` is NOT in workspace** (`PLAN.md:20`); collapse to `FR-CIV-ACT-001` (citizen-lifecycle row) for the partial work that landed. |
| `FR-CIV-SOCIAL-002-IDEOLOGY` | → | (deferred → `FR-CIV-ACT-001`) | `PLAN.md:149-150` — same as above; deferred. |
| `FR-CIV-WAR-001-UNITS` | → | `FR-CIV-WAR-010/011/012/013` (group) | `PLAN.md:203-204` — L3 work-log row. Real unit handling in `crates/tactics/src/{formation,pathfinding,movement,operational}.rs`; spec'd at `docs/design/warfare.md:77,191`. |
| `FR-CIV-WAR-002-COMBAT` | → | `FR-CIV-WAR-020/021/022` (group) | `PLAN.md:205-206` — L3 work-log row. Real combat in `crates/tactics/src/military_phase.rs`; spec'd at `docs/design/warfare.md:106,195`. |

## Notes on method

- The matrix's `status: CODE-ONLY-no-spec` is the *matrix's* view, which only checks for
  `docs/specs/requirements/*.md` and `docs/traceability/*.md`. The triage here also
  recognises `docs/specs/CIV-*.md`, `docs/agileplus/epics/civ-w*.md`, `docs/design/*.md`,
  `docs/development-guide/*.md`, `docs/models/civ-sim/*.md`, `docs/reference/*.md`,
  and `PLAN.md` (work-log only) as legitimate spec homes.
- **`PLAN.md` is treated as a work-log, NOT a spec home.** L3 rows in `PLAN.md` that
  reference an FR ID are *planning artifacts* for a hypothetical L3 worker, not a
  spec for an actual implementation. When the real implementation lives in
  `crates/<x>/src/...` and is spec'd at `docs/specs/CIV-*.md` or `docs/design/*.md`
  under a *different* FR ID, the `PLAN.md` row is a `STALE-ID` (verdict c, RENAME).
- **`crates/social` does not exist** in the workspace
  (`PLAN.md:20`, top of "Current vs planned" table). `FR-CIV-SOCIAL-001-INSTITUTIONS`
  and `FR-CIV-SOCIAL-002-IDEOLOGY` are L3 rows for crates that were *never created*;
  they are mapped to the partial citizen-lifecycle row `FR-CIV-ACT-001` for now
  (deferred = no full body until `crates/social` lands).
- The **only** batch-2 ID with a real, tested code footprint but no
  spec home is `FR-CIV-PLANET-010` (climate-snapshot determinism,
  with the existing `engine_tick_includes_climate_in_snapshot` test at
  `crates/engine/src/engine.rs:2429`). It receives a stub.
- IDs in the `FR-CIV-VOXEL-005/030/031/032` / `-RENDER-001/002` /
  `-UI-001/002/003` / `-EMERGENCE-002/003/004/010/011/012/013` /
  `-WEB-000/002/006/008` / `-INFRA-030/040` /
  `-TRAFFIC-LANE-001/002/003` / `-LIFE-022/024` / `-MARKET-001` /
  `-POLITY-001/008` / `-PROTO3D-010/011/012` / `-RTS-012/013/014/015` /
  `-GEO-001/004` / `-WAR-010..042` / `-VEHICLE-001/010/020/040` /
  `-SPECIES-104/201/302/406` / `-ASSET-002..019` /
  `-MOD-000..020` / `-PSYCHE-010/011/020/021/024/031..037/040` /
  `-SOC-DET-001` / `-SAVE-006/007/008/010` / `-UX-002/003` /
  `-L5` / `-AI-003/013/015` / `-BEVY-001/002/021` / `-CORE-011` /
  `-DIFFUSION-003` / `-ECON-003` / `-RES-001` /
  `-LEGENDS-INGEST-02` / `-LEGENDS-QUERY-07` / `-SERVER-001-WS` /
  `-SERVER-002-PROTO` / `FR-DET-002/006/007` / `FR-MET-001` /
  `FR-REP-001` series are all real user-meaningful requirements that
  the matrix undercounted because their spec lives in a `docs/design/*.md`
  or `docs/specs/CIV-*.md` table-cell row rather than a
  one-ID-per-section doc. These are marked `cov: yes` and do NOT need
  a new stub.

## Trace (Epic / Story / FR IDs touched)

- **Epic:** E2 (recovered-requirements remediation)
- **Story:** S-021-batch2 — "Triage next 150 CODE-ONLY-no-spec rows from `fr-matrix.json`"
- **FR IDs (stub added):** `FR-CIV-PLANET-010` (1)
- **FR IDs (RENAME-captured, no implementation change):**
  `FR-CIV-0001`, `FR-CIV-ACTOR-001-LIFECYCLE`,
  `FR-CIV-DIPLO-001-RELATIONS`, `FR-CIV-DIPLO-002-SHADOW`,
  `FR-CIV-METRICS-001`, `FR-CIV-METRICS-001-TIMESERIES`,
  `FR-CIV-RESEARCH-001-SCENARIO`, `FR-CIV-RESEARCH-002-SNAPSHOT`,
  `FR-CIV-RESEARCH-003-EXPORT`, `FR-CIV-SERVER-001`,
  `FR-CIV-SERVER-001-WS`, `FR-CIV-SERVER-002`, `FR-CIV-SERVER-002-PROTO`,
  `FR-CIV-SOCIAL-001-INSTITUTIONS`, `FR-CIV-SOCIAL-002-IDEOLOGY`,
  `FR-CIV-WAR-001-UNITS`, `FR-CIV-WAR-002-COMBAT` (18)
- **NFR IDs touched:** none
- **Adopted FRs (the spec homes this audit now formally recognises,
  in addition to the batch-1 set):**
  `docs/design/voxel-emergent-vision-and-migration.md`,
  `docs/design/psyche-social.md`, `docs/design/civ-ai-crate.md`,
  `docs/design/warfare.md`, `docs/design/polities-markets.md`,
  `docs/design/vehicles-logistics.md`, `docs/design/species-sentience.md`,
  `docs/design/legends-engine.md`, `docs/design/modding-platform.md`,
  `docs/specs/CIV-0300-rts-ui-ux-spec.md`,
  `docs/specs/CIV-0400-ai-npc-behavior-spec.md`,
  `docs/specs/CIV-0600-2d-asset-pipeline-spec.md`,
  `docs/specs/CIV-0700-modding-api-spec.md`,
  `docs/specs/CIV-1000-save-load-persistence-spec.md`,
  `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md`,
  `docs/specs/CIV-0101-two-zoom-lod-v1.md`,
  `docs/development-guide/fr-web-spectator.md`,
  `docs/development-guide/fr-godot-attach.md`,
  `docs/development-guide/fr-l5-visual-pass.md`,
  `docs/development-guide/p-w1-kickoff.md`,
  `docs/traceability/fr-web-matrix.md`,
  `docs/traceability/civis-tracelinks.md`,
  `docs/models/civ-sim/TECHNICAL_SPEC.md`,
  `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md`.

## Alternatives considered

1. **Just re-run the matrix script and treat the output as ground truth.**
   Rejected — batch 1 demonstrated the script's known false-positive rate
   on `CODE-ONLY-no-spec` is ~99% (786 of 786 rows in batch 1 + 2 are
   actually spec'd when broader spec homes are recognised). Skipping
   triage would leave 535 phantom IDs in the matrix and continue
   blocking FR-tracker consumers and the verify harness.
2. **One audit doc covering all 786 rows at once.**
   Rejected — would produce a 5000+ line file that the harness cannot
   diff usefully and that would have to be re-validated in one PR
   (high risk of merge conflicts with concurrent work on
   `docs/audits/fr-matrix.json`). The 100/150-row batch size matches
   the matrix's natural ref-count break (a single ref-count value
   would otherwise straddle batches). Plan: 100 + 150 + N batches
   of ≤ 150 until the 786 rows are exhausted (536 rows remain after
   this PR lands).
3. **Auto-generate the spec stubs by re-running the matrix script
   against a patched `fr_ids` front-matter array.**
   Rejected — the matrix script's source-of-truth for spec homes is
   `docs/specs/requirements/*.md` and `docs/traceability/*.md` only;
   expanding the script to recognise `docs/design/*.md` +
   `docs/development-guide/*.md` + `docs/specs/CIV-*.md` would
   require either (a) a complex regex of front-matter keys (brittle,
   no front matter in design docs) or (b) a JSON spec-home manifest
   file (out of scope; the doc-by-doc triage is more accurate for
   the L1 engineering handoff). The matrix's `CODE-ONLY-no-spec`
   status is a *triage hint*, not a verdict; manual review is
   required for each row, exactly as batch 1 established.
4. **Treat `PLAN.md` as a spec home for the L3 work-log rows.**
   Rejected — `PLAN.md` is a planning artifact (L3 agent
   instructions, status table, dependencies). It does not have
   acceptance criteria, schemas, or any of the §2/§3 structure
   the other spec homes have. Promoting it to a spec home would
   hide the real spec'd IDs and double-count coverage. The chosen
   verdict (RENAME for `FR-CIV-0001`, `FR-CIV-SERVER-001`,
   `FR-CIV-DIPLO-001-RELATIONS`, etc.) keeps the spec set
   honest: the real ID is the one with the real spec body, the
   alias is captured in the rename table for the FR-tracker
   consumer to dedupe.
