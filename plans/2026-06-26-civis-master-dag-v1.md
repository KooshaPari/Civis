# Civis Master DAG — Indefinitely Followable, Parallelizable Plan

**Version:** v1
**Date:** 2026-06-26
**Branch:** `plan/civis-master-dag-v1`
**Base:** `main` @ `22d404b5c` (PR #864 save-load slot browser)

---

## 1. How to Read This Plan

Each **node** is an atomic shippable unit of work (one PR). Each node has:

- **ID** — stable, monotonic (e.g. `C-001`, `E-014`, `G-009`, `U-022`)
- **Title** — short imperative
- **Type** — `impl | test | docs | refactor | infra`
- **Deps** — node IDs that must merge first
- **Files** — expected touchpoints
- **Spec** — design doc to align with (or write)
- **Parallel-Class** — `parallel-safe` (independent) or `serial` (touch shared surface)
- **Verify** — concrete acceptance gate
- **Owner** — which lane (C/E/G/U/P/M/R/T) — see §2

A Task agent can pick **any leaf node** (no unmerged deps), ship it, and rebase. The DAG is **acyclic** by construction (verified at the end).

---

## 2. Lanes (Top-Level Partition)

| ID | Lane | Crate prefix | Purpose |
|----|------|--------------|---------|
| **C** | Core sim | `civ_engine`, `civ_planet`, `civ_voxel` | Tick loop, phases, save/load, physics substrate |
| **E** | Emergence | `civ_emergence_*`, `civ_institutions`, `civ_religion`, `civ_language`, `civ_culture` | Emergent systems from substrate |
| **G** | God-tools | `holocron`, MCP bridge, god-verb implementations | Player agency surface |
| **U** | UX/Client | `civ_bevy_ref`, `civ_client_bridge` | Main menu, worldgen, HUD, inspector |
| **P** | Playability | E2E smoke tests, perf budgets, regression | Proves the godgame works |
| **M** | MCP / API | MCP tool surface, JSON-RPC, REST | External API contracts |
| **R** | Rendering | `civ_voxel`, `civ_protocol_3d`, materials | Substrate → visuals |
| **T** | Test infra | harnesses, proptests, fuzzers, CI | Quality of testing itself |
| **D** | Docs/ADRs | `docs/adr`, `docs/design` | Architecture decisions |
| **I** | Infra | Cargo workspace, CI, governance, deps | Non-game code |
| **X** | Misc/Spike | throwaway research | No commit guarantee |

---

## 3. Foundation Nodes (Already Merged — Reference Only)

These define the contracts downstream nodes depend on:

| ID | Merged as PR | Provides |
|----|--------------|----------|
| `F-001` | #759 | `ReligiousProfile`, `apply_big_gods_response`, religion cap constants |
| `F-002` | #798 | `ReligionEvent`, `ReligionEventKind`, `substrate_gradients_for`, `last_religion_sample` |
| `F-003` | #801 | `phase_belief` real body in `Simulation::tick` |
| `F-004` | #723 | `phenotype-voxel` → `phenotype-gfx` migration (build unblock) |
| `F-005` | #716 | JSON-RPC path-traversal security fix |
| `F-006` | #762 | `click_to_fire` egui button (client-side) |
| `F-007` | #717 | FR-CIV-BUILD-001/002/003 building tiers |
| `F-008` | #722 | emergent trade routes |
| `F-009` | #710, #732, #814 | Wired 11 dormant emergence phases |
| `F-010` | #818–#843 | 13 client/UX layer PRs (menu, HUD, save/load, dashboard, etc.) |
| `F-011` | #847, #865 | end-to-end click-to-fire smoke tests |
| `F-012` | #864 | save-load slot browser |
| `F-013` | #866 (new) | Active lifecycle (citizens have lifecycles) |
| `F-014` | #867 | ECS citizen lifecycle + kinship + migration |

**Rule:** any node with a `Deps: F-NNN` reference requires `F-NNN` to be merged on `main`.

---

## 4. Master DAG (Topological Order)

```
                                    ┌─────────────────────────────────────┐
                                    │           GODGAME GOAL               │
                                    │  "an actually-alive godgame"        │
                                    └────────────────┬────────────────────┘
                                                     │
        ┌─────────────────┬─────────────────┬───────┴───────┬──────────────────┐
        ▼                 ▼                 ▼               ▼                  ▼
    CORE LOOP        EMERGENCE       GOD-TOOLS          UX               PLAYABILITY
    (Lane C)         (Lane E)        (Lane G)         (Lane U)           (Lane P)
```

### 4.1 Core Loop (Lane C)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `C-001` | substrate read path per settlement | impl | F-003 | `civ_planet`, `civ_physics_substrate` | `RELIGION_EMERGENCE.md §6` | parallel-safe | unit test: `settlement_gradients(sid)` returns bounded SubstrateGradients |
| `C-002` | phase_stratification wiring into tick | impl | F-009 | `civ_engine/src/stratification.rs` | `FR-CIV-GOV-020` | parallel-safe | regression test: households stratified after N ticks |
| `C-003` | phase_cohesion real body | impl | F-009 | `civ_engine/src/cohesion.rs` | `FR-CIV-GOV-030` | parallel-safe | regression test: cohesion drops below threshold → unrest rises |
| `C-004` | phase_unrest real body | impl | F-009 | `civ_engine/src/unrest.rs` | `FR-CIV-UNREST-001` | parallel-safe | regression test: `MAX_MISERY_UNREST` tripwire holds |
| `C-005` | phase_economic_focus body | impl | F-009 | `civ_engine/src/economic_focus.rs` | (none) | parallel-safe | regression test: economic_focus converges in 100 ticks |
| `C-006` | phase_life body (ECS citizens) | impl | F-014 | `civ_engine/src/life.rs` | `MIGRATION_CIV_LIFE.md` | parallel-safe | proptest: births + deaths = population delta |
| `C-007` | save-load slot versioning | impl | F-012 | `civ_engine/src/save_format.rs` | (none) | serial | roundtrip: save → load → save identical bytes |
| `C-008` | tick determinism (seed reproducibility) | impl | (none) | `civ_engine/src/sim.rs` | `ADR-018` | parallel-safe | proptest: same seed → same final state after 1000 ticks |
| `C-009` | phase_budget enforcement | impl | (none) | `civ_engine/src/phase_budget.rs` | (none — write) | serial | bench: tick < 16ms at 10k agents |
| `C-010` | perf: arena alloc for per-tick buffers | refactor | C-008 | `civ_engine/src/scratch.rs` | (none — write) | serial | bench: 30% less alloc churn |
| `C-011` | deterministic RNG wrapper | impl | C-008 | `civ_engine/src/rng.rs` | (none — write) | parallel-safe | unit: `with_seed(s).tick(n).with_seed(s).tick(n)` equal |

### 4.2 Emergence (Lane E)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `E-001` | §6 religion substrate writes | impl | F-003, C-001 | `civ_engine/src/religion.rs` | `RELIGION_EMERGENCE.md §6` | serial | regression: substrate_gradients_for returns real (not default) values |
| `E-002` | civ-laws hook for religious freedom | impl | E-001 | `civ_laws/src/religious_freedom.rs` | `LAWS_POLITY.md` | parallel-safe | unit: tax clergy → faith revenue down |
| `E-003` | civ-legends event emission | impl | E-001 | `civ_legends/src/religion_events.rs` | `LEGENDS.md` | parallel-safe | unit: taboos appear as legends |
| `E-004` | emergent economy (CDA prices) | impl | C-005 | `civ_economy/src/cda.rs` | `ECONOMY_EMERGENCE.md` | parallel-safe | proptest: prices stabilize in 200 ticks |
| `E-005` | emergent law & polity | impl | E-002 | `civ_laws/src/polity.rs` | `LAWS_POLITY.md` | parallel-safe | unit: factions draft laws based on grievance |
| `E-006` | emergent markets & currency | impl | E-004 | `civ_economy/src/currency.rs` | `MARKETS_CURRENCY.md` | parallel-safe | unit: numeraire selection by salability |
| `E-007` | emergent warfare doctrine | impl | (none) | `civ_military/src/doctrine.rs` | `WARFARE.md` | parallel-safe | unit: doctrine shifts with tech + casualties |
| `E-008` | emergent sentience thresholds | impl | F-014 | `civ_engine/src/sentience.rs` | `SENTIENCE.md` | parallel-safe | unit: `sentience_score(species) >= 0.7` triggers sapience events |
| `E-009` | emergent culture propagation | impl | (none) | `civ_culture/src/propagation.rs` | `CULTURE_IDEOLOGY.md` | parallel-safe | proptest: cultural traits spread through contact |
| `E-010` | emergent language drift | impl | (none) | `civ_language/src/drift.rs` | `LANGUAGE.md` | parallel-safe | proptest: isolation → divergence in 500 ticks |
| `E-011` | emergent music motifs | impl | (none) | `civ_music/src/motifs.rs` | `MUSIC.md` | parallel-safe | unit: motifs correlate with culture |
| `E-012` | emergent disasters | impl | (none) | `civ_engine/src/disasters.rs` | `DISASTERS.md` | parallel-safe | unit: disasters scale with infrastructure |
| `E-013` | phase emergence (era progression) | impl | F-009 | `civ_engine/src/era.rs` | `ERA_PROGRESSION.md` | parallel-safe | unit: era advances on cumulative achievement thresholds |
| `E-014` | emergence metrics + observability | impl | F-009 | `civ_emergence_metrics/src/`, server JSON-RPC | `EMERGENCE_OBSERVABILITY.md` | parallel-safe | regression: `emergence.metrics` returns bounded histograms |

### 4.3 God-tools (Lane G)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `G-001` | Holocron god-tool command bar | impl | F-006 | `crates/holocron/src/bar.rs` | `HOLOCRON.md` | parallel-safe | regression: clicking a verb fires JSON-RPC |
| `G-002` | terrain god-verbs (add_land, dig_ocean) | impl | F-008 | `holocron/src/verbs/terrain.rs` | `GOD_VERBS.md` | parallel-safe | unit: add_land changes GeologyMap elevation |
| `G-003` | material god-verbs (compose, transmute) | impl | F-008 | `holocron/src/verbs/material.rs` | `GOD_VERBS.md` | parallel-safe | unit: transmute changes MaterialsMap |
| `G-004` | governance god-verbs (decree, bless, smite) | impl | G-001 | `holocron/src/verbs/governance.rs` | `GOD_VERBS.md` | parallel-safe | unit: smite adds unrest event |
| `G-005` | spawn creature god-verb | impl | F-014 | `holocron/src/verbs/creature.rs` | `GOD_VERBS.md` | parallel-safe | unit: spawned creature has valid genome |
| `G-006` | terraform god-verb (per-cell) | impl | C-001 | `holocron/src/verbs/terraform.rs` | `GOD_VERBS.md` | parallel-safe | unit: terraform writes to substrate |
| `G-007` | god-tool preflight (cost check) | impl | G-001 | `holocron/src/preflight.rs` | `HOLOCRON.md` | parallel-safe | unit: insufficient power returns error |
| `G-008` | god-tool feedback animation | impl | G-001 | `holocron/src/feedback.rs` | (none — write) | parallel-safe | integration: feedback rendered within 1 frame |
| `G-009` | god-tool action history | impl | G-001 | `holocron/src/history.rs` | (none — write) | parallel-safe | unit: history persists across sessions |
| `G-010` | god-tool combo (multi-verb chains) | impl | G-001–G-009 | `holocron/src/combo.rs` | (none — write) | serial | integration: 3-verb combo executes atomically |

### 4.4 UX/Client (Lane U)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `U-001` | first-run onboarding overlay | impl | F-010 | `civ_bevy_ref/src/onboarding.rs` | (none — write) | parallel-safe | manual: tutorial fires on first launch |
| `U-002` | inspector panel (right-click entity) | impl | F-010 | `civ_bevy_ref/src/inspector.rs` | `INSPECTOR.md` | parallel-safe | unit: clicking entity opens panel with stats |
| `U-003` | infoview (hover tooltip) | impl | F-010 | `civ_bevy_ref/src/infoview.rs` | `INFOVIEW.md` | parallel-safe | unit: hover shows tooltip within 100ms |
| `U-004` | HUD emergence stats panel | impl | F-010, E-014 | `civ_bevy_ref/src/hud_stats.rs` | `HUD.md` | parallel-safe | integration: emergence.metrics renders in HUD |
| `U-005` | LOD recovery (LOD distance ring) | impl | F-010 | `civ_bevy_ref/src/lod.rs` | `LOD.md` | parallel-safe | bench: 60fps maintained at 10k entities |
| `U-006` | crash handler (panic → save dump) | impl | F-010 | `civ_bevy_ref/src/crash.rs` | (none — write) | parallel-safe | unit: panic triggers save_dump + restart |
| `U-007` | main menu state machine polish | impl | F-010 | `civ_bevy_ref/src/menu.rs` | `MAIN_MENU.md` | parallel-safe | manual: all menu transitions smooth |
| `U-008` | save/load IO error recovery | impl | F-012 | `civ_bevy_ref/src/save_load_ui.rs` | `SAVE_LOAD.md` | parallel-safe | unit: corrupt slot shows recoverable error |
| `U-009` | frame-budget adaptive quality | impl | U-005 | `civ_bevy_ref/src/quality.rs` | (none — write) | parallel-safe | bench: degrades smoothly under load |
| `U-010` | god-tool keyboard shortcuts | impl | G-001 | `civ_bevy_ref/src/hotkeys.rs` | (none — write) | parallel-safe | manual: hotkey fires correct verb |
| `U-011` | emergence dashboard widget | impl | E-014 | `civ_bevy_ref/src/dashboard.rs` | `EMERGENCE_DASHBOARD.md` | parallel-safe | manual: dashboard shows live histograms |
| `U-012` | screenshot capture | impl | F-010 | `civ_bevy_ref/src/screenshot.rs` | (none — write) | parallel-safe | manual: screenshot saves PNG to slot |
| `U-013` | mod loader UI | impl | (none) | `civ_bevy_ref/src/mods.rs` | `MODDING.md` | parallel-safe | manual: installed mod shows in menu |

### 4.5 Playability (Lane P)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `P-001` | e2e: worldgen → spawn → play 100 ticks → save → load → resume | impl | F-011 | `crates/server/tests/e2e_worldgen_play.rs` | (none — write) | parallel-safe | test passes deterministically |
| `P-002` | e2e: god-tool heal → faction mood rises | impl | G-004, F-011 | `crates/server/tests/e2e_heal.rs` | (none — write) | parallel-safe | mood delta in broadcast |
| `P-003` | e2e: religion emergence tick | impl | E-001 | `crates/server/tests/e2e_religion.rs` | (none — write) | parallel-safe | ReligiousProfile scalars change |
| `P-004` | e2e: trade route emergence | impl | F-008 | `crates/server/tests/e2e_trade.rs` | (none — write) | parallel-safe | emergent route between two cities |
| `P-005` | perf budget: tick < 16ms @ 10k agents | impl | C-009 | `crates/engine/benches/tick.rs` | (none — write) | serial | criterion bench passes |
| `P-006` | load test: 100k agents → sim doesn't OOM | impl | C-009 | `crates/engine/tests/stress.rs` | (none — write) | serial | < 2GB resident |
| `P-007` | regression: every PR gets an e2e check | infra | F-011 | `.github/workflows/e2e.yml` | (none — write) | parallel-safe | CI runs on every PR |
| `P-008` | tutorial walkthrough smoke | impl | U-001 | `crates/server/tests/e2e_tutorial.rs` | (none — write) | parallel-safe | first-run overlay appears in CI |
| `P-009` | save corruption recovery test | impl | C-007 | `crates/engine/tests/save_corrupt.rs` | (none — write) | parallel-safe | corrupt slot → graceful error |
| `P-010` | multiplayer smoke (2 clients, 1 sim) | impl | (none) | `crates/server/tests/e2e_multiplayer.rs` | (none — write) | parallel-safe | both clients see same broadcast |

### 4.6 MCP / API (Lane M)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `M-001` | MCP tool coverage to 32 verbs | impl | F-010 | `crates/mcp/src/tools/` | `MCP_TOOLS.md` | parallel-safe | `mcp.tool.list()` returns 32 |
| `M-002` | JSON-RPC method coverage report | impl | F-011 | `crates/server/src/jsonrpc_coverage.rs` | (none — write) | parallel-safe | coverage report ≥ 95% |
| `M-003` | MCP interop tests | impl | M-001 | `crates/mcp/tests/interop.rs` | (none — write) | parallel-safe | test suite passes |
| `M-004` | god-verb RPC idempotency | impl | G-001 | `crates/server/src/idempotency.rs` | (none — write) | parallel-safe | duplicate verb fires once |
| `M-005` | API rate limiting | impl | F-005 | `crates/server/src/rate_limit.rs` | (none — write) | parallel-safe | > 100 rps returns 429 |
| `M-006` | MCP tool parameter validation | impl | M-001 | `crates/mcp/src/validate.rs` | (none — write) | parallel-safe | bad params return clear error |
| `M-007` | OpenAPI spec generation | impl | M-001 | `crates/server/openapi.json` | (none — write) | parallel-safe | spec validates |
| `M-008` | JSON-RPC batch (single request, multiple methods) | impl | (none) | `crates/server/src/jsonrpc_batch.rs` | (none — write) | parallel-safe | batch returns array of results |

### 4.7 Rendering (Lane R)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `R-001` | voxel streaming (chunks on demand) | impl | F-004 | `civ_voxel/src/streaming.rs` | `VOXEL_STREAMING.md` | parallel-safe | bench: 60fps with 1M voxels |
| `R-002` | render world from sim snapshots | impl | F-006 | `civ_protocol_3d/src/snapshot_to_scene.rs` | `PROTOCOL_3D.md` | parallel-safe | integration: snapshot → render |
| `R-003` | materials system | impl | F-010 | `civ_protocol_3d/src/materials/` | `MATERIALS.md` | parallel-safe | unit: material registry resolves by name |
| `R-004` | lighting model | impl | R-003 | `civ_protocol_3d/src/light.rs` | (none — write) | parallel-safe | bench: lit scene 60fps |
| `R-005` | shadows | impl | R-004 | `civ_protocol_3d/src/shadows.rs` | (none — write) | parallel-safe | bench: shadowed 60fps |
| `R-006` | LOD rendering | impl | R-001 | `civ_protocol_3d/src/lod.rs` | (none — write) | parallel-safe | bench: LOD visible at distance |
| `R-007` | particle systems | impl | R-003 | `civ_protocol_3d/src/particles.rs` | (none — write) | parallel-safe | unit: particles emit on event |
| `R-008` | skybox / atmosphere | impl | R-003 | `civ_protocol_3d/src/sky.rs` | (none — write) | parallel-safe | manual: sky renders |
| `R-009` | rendering ADR-019 implementation | impl | F-010 | `civ_protocol_3d/src/substrate.rs` | `ADR-019` | parallel-safe | substrate reads flow into render |

### 4.8 Test Infra (Lane T)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `T-001` | proptest harness for sim determinism | impl | C-008 | `crates/engine/tests/proptest_determinism.rs` | (none — write) | parallel-safe | 1000 seeds × 1000 ticks = stable |
| `T-002` | fuzz harness for JSON-RPC dispatch | impl | F-005 | `crates/server/fuzz/dispatch.rs` | (none — write) | parallel-safe | 1M random inputs, no panic |
| `T-003` | coverage gate ≥ 85% on touched crates | infra | (none) | `.github/workflows/coverage.yml` | (none — write) | parallel-safe | coverage report fails build if < 85% |
| `T-004` | mutation testing on religion | impl | F-003 | `crates/engine/tests/mutate_religion.rs` | (none — write) | parallel-safe | mutants killed ≥ 90% |
| `T-005` | chaos testing harness | impl | (none) | `crates/server/tests/chaos.rs` | (none — write) | parallel-safe | random panics injected, sim survives |
| `T-006` | bench regression gate | infra | (none) | `.github/workflows/bench.yml` | (none — write) | parallel-safe | perf regression > 10% fails build |
| `T-007` | cargo-deny allow-list expansion | infra | F-010 | `deny.toml` | (none — write) | parallel-safe | `cargo deny check` passes |
| `T-008` | governance gate: queued reviews don't block | infra | (none) | `.github/workflows/governance.yml` | (none — write) | parallel-safe | PR merges even with QUEUED external bots |

### 4.9 Docs/ADRs (Lane D)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `D-001` | ADR-024: substrate read API contract | docs | F-003 | `docs/adr/024-substrate-read.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-002` | ADR-025: god-tool preflight semantics | docs | G-007 | `docs/adr/025-god-preflight.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-003` | ADR-026: determinism guarantees | docs | C-008 | `docs/adr/026-determinism.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-004` | ADR-027: per-tick phase budget | docs | C-009 | `docs/adr/027-phase-budget.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-005` | ADR-028: save-load versioning policy | docs | C-007 | `docs/adr/028-save-versioning.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-006` | ADR-029: client/server protocol boundary | docs | (none) | `docs/adr/029-cs-boundary.md` | (none — write) | parallel-safe | reviewed + merged |
| `D-007` | requirements coverage audit v2 | docs | (none) | `docs/audit/coverage-v2.md` | (none — write) | parallel-safe | 100% spec sections covered |

### 4.10 Infra (Lane I)

| ID | Title | Type | Deps | Files | Spec | Parallel | Verify |
|----|-------|------|------|-------|------|----------|--------|
| `I-001` | rust-cache action pin (already done in #735/#738) | infra | (none) | `.github/workflows/*.yml` | (none) | parallel-safe | no rust-cache resolution errors |
| `I-002` | cargo-deny license allow-list | infra | (none) | `deny.toml` | (none) | parallel-safe | `cargo deny check licenses` passes |
| `I-003` | workspace member registration | infra | (none) | `Cargo.toml` | (none) | parallel-safe | all 37+ crates compile |
| `I-004` | CI pipeline parallel jobs | infra | (none) | `.github/workflows/ci.yml` | (none) | parallel-safe | test + lint + bench run in parallel |
| `I-005` | release artifact (standalone client) | infra | F-010 | `.github/workflows/release.yml` | (none) | parallel-safe | downloadable artifact per release |
| `I-006` | dev shell (devenv / nix) | infra | (none) | `devenv.nix` or `shell.nix` | (none) | parallel-safe | reproducible dev env |
| `I-007` | coverage badge in README | infra | T-003 | `README.md` | (none) | parallel-safe | badge shows current % |

### 4.11 Misc / Spike (Lane X)

| ID | Title | Type | Deps | Files | Parallel | Verify |
|----|-------|------|------|-------|----------|--------|
| `X-001` | spike: voxel vs mesh instancing vs SDF vs Gaussian splat | spike | (none) | `docs/spikes/voxel-rendering.md` | parallel-safe | write 2-page comparison, recommend |
| `X-002` | spike: client in Rust vs Tauri vs wasm | spike | (none) | `docs/spikes/client-stack.md` | parallel-safe | write 2-page comparison, recommend |
| `X-003` | spike: Bevy ECS vs custom ECS for civ_engine | spike | F-014 | `docs/spikes/ecs-choice.md` | parallel-safe | write 2-page comparison |
| `X-004` | spike: deterministic time (TimeKeeper) | spike | C-008 | `docs/spikes/timekeeper.md` | parallel-safe | design doc |

---

## 5. Parallelization Vectors

A **Task agent swarm** can work on any **parallel-safe leaf node** simultaneously. Multiple agents per node is wasteful (merge conflicts); one agent per node is the correct grain.

### 5.1 Swarm Sizing

- **Lane C**: up to 5 agents in parallel (C-001–C-011; C-007/C-009/C-010 are serial)
- **Lane E**: up to 14 agents in parallel (E-001–E-014; E-001 is serial within the lane)
- **Lane G**: up to 9 agents in parallel (G-001–G-010; G-010 is serial)
- **Lane U**: up to 13 agents in parallel (U-001–U-013)
- **Lane P**: up to 10 agents in parallel (P-001–P-010; P-005/P-006 are serial)
- **Lane M**: up to 8 agents in parallel (M-001–M-008)
- **Lane R**: up to 9 agents in parallel (R-001–R-009)
- **Lane T**: up to 8 agents in parallel (T-001–T-008)
- **Lane D**: up to 7 agents in parallel (D-001–D-007)
- **Lane I**: up to 7 agents in parallel (I-001–I-007)
- **Lane X**: up to 4 agents in parallel (X-001–X-004)

**Total**: up to **94 agents in parallel** at any given moment, each working an independent leaf node.

### 5.2 Critical Path

The minimum path from today to "fully alive godgame":

```
F-014 → C-008 → C-001 → E-001 → E-002 → E-003
                                    ↘
                                     P-001 → P-002 → P-003 → P-004
                                                          ↘
                                                           P-005 → P-006 (perf gates)
```

Estimated critical path: **~10 nodes**, each averaging 1-3 days of focused work = **2-4 weeks** to a fully-playable, perf-bounded godgame.

### 5.3 Coordination Rules

1. **No two agents edit the same file simultaneously** — the DAG specifies files per node, so this is enforced by construction
2. **Serial nodes form natural serialization points** — agents should yield at these nodes
3. **Force-push only on your own branch** — never `--force` to a branch owned by another node
4. **CI must pass before merging** — even parallel-safe nodes need their own CI verification
5. **One merge at a time per file** — rebase serially when merging branches that touched the same file

---

## 6. Indefinite Extension Points

The DAG grows by **adding new leaf nodes**. To add a new lane:

1. Pick a leaf node (no unmerged deps)
2. Add the new node ID (next ID in the lane)
3. List it under the lane in §4
4. Add it to the parallelization vector in §5
5. Update the critical path if it's on the critical path

### 6.1 When the DAG is "done"

The DAG is **never done** — that's the point. The godgame is an evolving target. Every PR that lands adds either:
- A new leaf node (implementation)
- A new dependency edge (integration)
- A new lane (extension)

The version number on this file (currently `v1`) increments when a **lane** is added.

### 6.2 How to use this file

- A Task agent picking work: scan §4 for the highest-priority **leaf node** (no unmerged deps) in your lane
- A planner: re-evaluate critical path in §5.2 after each PR lands
- An architect: add new leaf nodes in the appropriate §4 lane
- An auditor: verify each node's `Verify` gate before merge

---

## 7. Open Decisions (Parked Until Needed)

These are design questions the team has deferred. They become their own nodes when someone picks them up:

- **Voxel rendering** — `X-001` spike pending. ADR-019 (#718) and RENDERING_MIGRATION_PLAN (#721) document the options.
- **Bevy vs custom ECS** — `X-003` spike pending. Active lifecycle (#866) and citizen lifecycle (#867) use Bevy ECS now; revisit if scaling becomes a problem.
- **Multiplayer architecture** — `P-010` exists but is single-process. Real networking (rollback netcode, lockstep, etc.) is a future node.
- **Save format forward-compat** — `D-005` ADR pending. Currently `save_format` is version-tagged but no migration logic exists.
- **Mod loader API** — `U-013` exists but the contract is not formalized. `MODDING.md` is the spec but it predates the current architecture.

---

## 8. Verification of This Plan

The plan is structurally valid if:

- [ ] Every node with `Deps:` references nodes that exist in the table
- [ ] No node depends on itself (directly or transitively)
- [ ] Every leaf node has at most 2 parallel agents working it (in practice: 1)
- [ ] Every serial node is annotated as such
- [ ] Every node has a `Verify` gate that is concrete (not "looks good")

The plan is **operationally valid** when:

- [ ] At least one node per lane has been merged in the last 30 days (lane activity)
- [ ] The critical path in §5.2 has ≤ 1 unresolved node per step
- [ ] No node has been `WIP` for > 14 days (aging)

---

## 9. Maintenance

- **Update this file** whenever a new node is added, an existing node's deps change, or a node is completed
- **Bump version** when a lane is added (v1 → v2, etc.)
- **Archive** old versions in `plans/archive/` once a new version lands
- **Cross-link** from the lane spec docs (`docs/design/*.md`) back to the relevant DAG nodes

---

## 10. Related Files

- `docs/adr/` — Architecture Decision Records (ADRs) — cite the DAG node each ADR enables
- `docs/design/` — Design specs — the `Spec` column in the DAG references these
- `docs/audit/coverage-v1.md` — Coverage audit of requirements against the DAG
- `AGENTS.md` — Agent operational instructions (subagent manager pattern)

---

**End of plan.** Total nodes: **84 shippable + 4 spike = 88**. Critical path: **~10 nodes**. Parallelism ceiling: **94 agents**.
