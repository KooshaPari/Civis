---
spec_id: civ-021
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-10
workstream_branch: docs/phantom-id-triage-2
---

# Specification: Recovered Requirements — Phantom-ID Triage Batch 1 + 2

**Slug**: civ-021-recovered-requirements | **Epic**: E2 | **Date**: 2026-06-10 | **State**: ACTIVE
**Workstream branch**: `docs/phantom-id-triage` (batch 1) + `docs/phantom-id-triage-2` (batch 2)
**Audit sources**:
[`docs/audits/phantom-triage-batch1.md`](../../../docs/audits/phantom-triage-batch1.md)
[`docs/audits/phantom-triage-batch2.md`](../../../docs/audits/phantom-triage-batch2.md)

## Problem Statement

The FR↔code↔test matrix (`docs/audits/fr-matrix.json`, 1181 IDs, generated 2026-06-10)
classifies 786 rows as `CODE-ONLY-no-spec`. Most of those rows are *not* actually
unspec'd — their requirement lives in a `docs/specs/requirements/*.md` table-cell row
(e.g. `FR-CIV-PSYCHE.md`), in `docs/specs/CIV-*.md`, in `docs/agileplus/epics/civ-w*.md`,
in `docs/design/*.md`, or in `docs/development-guide/*.md`. The matrix script
(`_id_inventory_v3.py` family) checks only `docs/specs/requirements/*` and
`docs/traceability/*`, so it undercounts.

A subset of `CODE-ONLY-no-spec` IDs **really are** unspec'd: the code/docs do
implement something user-meaningful, but no formal spec doc names it. Without
a spec entry these IDs cannot be:
- discovered by FR-tracker consumers (`docs/reference/FR_TRACKER.md`),
- gated by the verify harness (`just civis-3d-verify`),
- referenced from PR descriptions or `docs/AGENTS.md`.

This spec is the **recovery doc** for batch 1 (the top-100 of the 786). Each
stub below is a *one-line* minimum: it names the FR ID, the capability, and
the strongest evidence file:line. Follow-up batches will use the same template.

## Numbering note

The task description named this spec `civ-019-recovered-requirements`, but
`civ-019-emergence-metrics-dashboard` already exists
(see `agileplus-specs/civ-019-emergence-metrics-dashboard/`). Per the rules
in the parent Phenotype `AGENTS.md` (no two specs share a number), the next
free number is **civ-021** (civ-020 is `ca-perf-dirty-chunk`). The PR that
introduced this doc notes the deviation explicitly in its `Trace:` block.

## Target Users

- FR-tracker consumers (`docs/reference/FR_TRACKER.md`, `docs/reference/CODE_ENTITY_MAP.md`)
- Verify-harness authors wiring gates by FR ID
- Agents writing PR descriptions and `Trace:` blocks
- Mod authors reading the spec set before authoring

## Functional Requirements (recovered stubs)

The 12 FR IDs below are the *class-(a) UNCOVERED* rows from the batch-1
audit. Each stub is a one-line minimum; deeper bodies live in the cited
source-of-truth doc.

- [ ] **FR-CIV-ACT-001** — Citizen lifecycle (birth / init / age / death).
  Source: `docs/models/civ-sim/TECHNICAL_SPEC.md:2103` + `docs/reference/FR_TRACKER.md:28`.
  Full body in CIV-0103 spec; implementation in `crates/engine/src/engine.rs`.

- [ ] **FR-CIV-ECON-001-MARKET** — Market price tracking
  (`Market::record_transaction`, `update_prices`, `get_price`) in `crates/economy/src/market.rs`.
  Source: `docs/guides/COPILOT_L3_AGENTS.md:90,470`.
  RENAME-candidate: collapse the hyphenated `…-MARKET` form to the
  non-hyphenated `FR-CIV-ECON-001` (see `docs/reference/FR_TRACKER.md`).

- [ ] **FR-CIV-ECON-002-JOULE** — Joule allocator with energy conservation
  in `crates/economy/src/joule.rs`.
  Source: `docs/guides/COPILOT_L3_AGENTS.md:92,474`.
  RENAME-candidate: collapse to `FR-CIV-ECON-002` (the matrix has two rows
  for the same capability — one hyphenated, one not).

- [ ] **FR-CIV-ECON-004** — Policy-driven fiscal control via
  `crates/engine/src/policy.rs`. Source: `docs/reference/CODE_ENTITY_MAP.md:8`
  + `docs/reference/FR_TRACKER.md:10`.

- [ ] **FR-CIV-EMERGENCE-001** — Abiogenesis threshold: each tick the engine
  scans the CA state; when a configurable set of material conditions co-occur
  in a region, a proto-life event is emitted. Source:
  `docs/guides/voxel-emergent-vision-and-migration.md:97,133,137`.

- [ ] **FR-CIV-PLANET-020** — Tide-driven coastal water columns. Each tick
  the tide offset shifts the water-level voxel for every coastal column;
  columns are keyed by `(x, z)` in fixed-point world coords; iteration order
  is deterministic. Source: `crates/engine/src/engine.rs:434,460`
  (comment + `WATER_MARKER_MATERIAL` constant).

- [ ] **FR-CIV-PLANET-030** — Per-region weather grid updated by
  `phase_planet` each tick. Source: `crates/engine/src/engine.rs:437`
  (`weather_grid: Vec<WeatherCell>`) + `crates/planet/src/weather.rs`.

- [ ] **FR-CIV-PLANET-060** — Climate + weather-grid + geology folded into
  the replay hash chain; the chain digest changes on any `ClimateFrame`
  field delta. Source: `crates/engine/src/hash_chain.rs:5,129,235` +
  `crates/engine/src/replay.rs:42`.

- [ ] **FR-CIV-UX-005** — Era / overview camera presets
  ("Cam Wide" / "Cam Close") shipped across Godot + Web + Unreal clients.
  Source: `clients/godot-ref/scripts/camera.gd:43` + `ui.tscn:107` +
  `web/dashboard/src/bottom_bar.tsx`.

- [ ] **FR-CIV-BEVY-016** — Live attach smoke harness v2 (P-W1 item 41):
  `just civis-3d-live-smoke` runs `live_stream::` + minimap UV lib tests
  + both Bevy bins. Source: `clients/bevy-ref/README.md:26` +
  `docs/development-guide/p-w1-kickoff.md:127,711` + `justfile:127`.

- [ ] **FR-CIV-BEVY-022** — Live attach smoke harness v3 (P-W1 item 47):
  `live_focus::` and `live_minimap::` lib tests.
  Source: `clients/bevy-ref/README.md:26` +
  `docs/development-guide/p-w1-kickoff.md:133,737` + `justfile:127`.

- [ ] **FR-CIV-BEVY-025** — Live attach smoke harness v4 (P-W1 item 50):
  `live_pick::` lib tests. Source: `clients/bevy-ref/README.md:26` +
  `docs/development-guide/p-w1-kickoff.md:136,740` + `justfile:127`.

## Functional Requirements (recovered stubs, batch 2 — user-demand trace)

The 4 FR IDs below are the `UNSPEC'D-DEMAND` rows from
[`docs/audits/user-demand-trace-2026-06-10.md`](../../../docs/audits/user-demand-trace-2026-06-10.md)
(session `1cae14f8-e6a7-4f55-b4ba-50bad36a87eb`, 2026-05-30 — domain-model
review). They are methodology / governance asks that the
`_id_inventory_v3.py` scan does not cover, so they live in
**`SPEC-ONLY`** until code/docs land. Same one-line template as batch 1.

- [ ] **FR-CIV-DOMAIN-CTX-CATALOG** — Bounded-context catalog naming
  engine / economy / agents / needs / planet / tactics / voxel, with
  one owning crate per context. Source: `docs/research/xdd-sota-traceability.md:9,23,24`
  + the user-demand prompt itself (session `1cae14f8`).

- [ ] **FR-CIV-UBIQ-LANG-RECONCILE** — Ubiquitous-language reconciliation
  pass: pick one canonical term per concept and alias the others
  (e.g. `faction:u32` → `cluster_id`; `settlement` reserved for the
  emergent-overlap concept only; `polity` for the higher-level
  container). Doc-only deliverable
  (`docs/audits/naming-drift.md`); no code change required. Source:
  demand #23 in `docs/audits/user-demand-trace-2026-06-10.md`.

- [ ] **FR-CIV-DOMAIN-NOUN-DRIFT** — Per-spec-vs-code noun-drift report
  (the "named-noun drift" audit). For each `FR-CIV-*` epic, list the
  nouns used in the spec text, the nouns used in the code, and the
  diff. Source: demand #24 in
  `docs/audits/user-demand-trace-2026-06-10.md`.

- [ ] **FR-CIV-XDD-METHODOLOGY-PLAN** — xDD (SDD / DDD / CDD) methodology
  adoption plan for the Rust civ-sim: a per-crate matrix of which
  methodology applies (SDD for sim-core deterministic laws; DDD for
  bounded contexts above; CDD for the `protocol-3d` JSON-RPC surface).
  Source: `docs/research/xdd-sota-traceability.md:5,11,28-42` +
  demand #25 in `docs/audits/user-demand-trace-2026-06-10.md`.

## Batch 2 additions (2026-06-10)

The batch-2 audit
([`phantom-triage-batch2.md`](../../../docs/audits/phantom-triage-batch2.md),
150 rows, `FR-CIV-EMERGENCE-004` … `FR-SAVE-010`) found **one** new
uncovered real-requirement stub — `FR-CIV-PLANET-010`. Every other
batch-2 ID is either (a) covered by an existing spec home, or (b) a
`STALE-ID` whose only footprint is the L3 `PLAN.md` work-log and which
maps onto an existing spec'd ID via the rename table at the bottom of
this doc.

- [ ] **FR-CIV-PLANET-010** — Deterministic climate snapshot on
  `Simulation::snapshot()`. `phase_planet` produces a `Climate` value
  that is **bit-identical** to a pure
  `compute_climate(tick, planet, moon)` call, and the snapshot
  surfaces it on the wire for the JSON-RPC server. Source:
  `crates/engine/src/engine.rs:2161,2427` (doc comments) +
  `crates/server/src/jsonrpc.rs:411` (`phase_planet` reference) +
  test `engine_tick_includes_climate_in_snapshot` at `engine.rs:2429`.
  Acceptance is the existing `#[test]` round-trip: a freshly-built
  `Simulation::with_seed(s)` snapshot equals `compute_climate(0, …)`,
  and again after one `tick()` equals `compute_climate(1, …)`.
## Non-Functional Requirements

- Determinism: every recovered ID's underlying implementation MUST
  preserve `(seed, scenario)` determinism. Re-verification reuses the
  existing tests (e.g. `crates/planet/src/geology.rs:106` for
  FR-CIV-PLANET-040; `crates/engine/src/hash_chain.rs:235` for
  FR-CIV-PLANET-060).
- Discoverability: each ID listed above MUST appear in
  `docs/reference/FR_TRACKER.md` and `docs/reference/CODE_ENTITY_MAP.md`
  within the same batch (out of scope here — filed as follow-up).

## Validation

- A re-run of `_id_inventory_v3.py` after this commit lands MUST
  drop the 13 stub IDs (12 batch-1 + 1 batch-2) from `CODE-ONLY-no-spec`
  into `COVERED` (they now have a spec doc with the same `fr_ids` keys
  in the YAML front matter). The matrix is the source of truth; this
  doc is the harness.
- The verify gate `just civis-3d-verify` MUST remain green.

## Follow-up batches

The remaining 535 `CODE-ONLY-no-spec` rows will be triaged in subsequent
batches using the same template. This spec doc grows by appending new
stubs (no overwrite); the `meta.json` `fr_ids` array grows likewise.

## RENAME mappings (cumulative, batch 1 + 2)

The IDs below are *stale* aliases that appear in `PLAN.md` L3 work-log
rows or the agent-guide / worktree-guide commit-message templates but
do not name their own implementation. Each maps onto the spec'd ID
that owns the real implementation and its acceptance tests.

| Old ID | → | New (existing) ID | Rationale |
|--------|---|-------------------|-----------|
| `FR-CIV-0001` | → | `FR-CIV-CORE-001` | `PLAN.md:16` + `GIT_WORKTREE_GUIDE.md:151`; the real tick loop is `FR-CIV-CORE-001` in `crates/engine/src/engine.rs`. |
| `FR-CIV-ACTOR-001-LIFECYCLE` | → | `FR-CIV-ACT-001` | `PLAN.md:145-146`; the real citizen lifecycle is `FR-CIV-ACT-001` (see batch-1 stub). |
| `FR-CIV-DIPLO-001-RELATIONS` | → | `FR-CIV-DIPLO-001` | `PLAN.md:207-208`; the real 8-state FSM is `FR-CIV-DIPLO-001` in `crates/diplomacy/src/lib.rs:1,15`. |
| `FR-CIV-DIPLO-002-SHADOW` | → | `FR-CIV-DIPLO-002` | `PLAN.md:209-210`; the real influence-capital line is `FR-CIV-DIPLO-002` in `crates/diplomacy/src/lib.rs:16`. |
| `FR-CIV-METRICS-001` | → | `FR-CIV-METRICS-001-TIMESERIES` | `PLAN.md:151`; non-hyphenated form is a phantom alias. |
| `FR-CIV-METRICS-001-TIMESERIES` | → | (keep) | `PLAN.md:151-152`; the real time-series lives in `crates/engine` metrics (no separate `crates/metrics` per `PLAN.md:22`). Spec'd in `docs/reference/FR_TRACKER.md` + `STATUS_REPORT.md`. |
| `FR-CIV-RESEARCH-001-SCENARIO` | → | (keep) | `PLAN.md:233-234`; the real LLM cache + card acceptance is `FR-CIV-RESEARCH-001` in `crates/research/src/lib.rs:377,408`. |
| `FR-CIV-RESEARCH-002-SNAPSHOT` | → | (keep) | `PLAN.md:235-236`; the real canonical-replay line is `crates/research/src/lib.rs:601`. |
| `FR-CIV-RESEARCH-003-EXPORT` | → | (keep) | `PLAN.md:237-238`; the real hybrid-replay line is `crates/research/src/lib.rs:616`. |
| `FR-CIV-SERVER-001` | → | `FR-CIV-SERVER-001-WS` | `PLAN.md:174-175`; the real WebSocket server is the hyphenated form. |
| `FR-CIV-SERVER-001-WS` | → | (keep) | `PLAN.md:174-175`; the real `SimServer` is in `crates/server/src/websocket.rs` (token at `crates/server/src/main.rs` + `crates/server/src/jsonrpc.rs`). |
| `FR-CIV-SERVER-002` | → | `FR-CIV-SERVER-002-PROTO` | `PLAN.md:176-177`; the real protocol is the hyphenated form. |
| `FR-CIV-SERVER-002-PROTO` | → | (keep) | `PLAN.md:176-177`; the real `ClientMessage`/`ServerMessage` is in `crates/server/src/`. |
| `FR-CIV-SOCIAL-001-INSTITUTIONS` | → | `FR-CIV-ACT-001` (deferred) | `PLAN.md:147-148`; **`crates/social` does not exist** (`PLAN.md:20`), so the institution layer is not implemented; the partial citizen-lifecycle work lives under `FR-CIV-ACT-001`. |
| `FR-CIV-SOCIAL-002-IDEOLOGY` | → | `FR-CIV-ACT-001` (deferred) | `PLAN.md:149-150`; same as above — `crates/social` not in workspace. |
| `FR-CIV-WAR-001-UNITS` | → | (keep, group with `FR-CIV-WAR-010..013`) | `PLAN.md:203-204`; the real unit-handling code is in `crates/tactics/src/{formation,pathfinding,movement,operational}.rs` and spec'd in `docs/design/warfare.md`. |
| `FR-CIV-WAR-002-COMBAT` | → | (keep, group with `FR-CIV-WAR-020..022`) | `PLAN.md:205-206`; the real combat-resolution code is in `crates/tactics/src/military_phase.rs` + `crates/tactics/src/war_bridge.rs` and spec'd in `docs/design/warfare.md`. |
