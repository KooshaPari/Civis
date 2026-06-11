---
spec_id: civ-021
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-10
workstream_branch: docs/phantom-id-triage
---

# Specification: Recovered Requirements — Phantom-ID Triage Batch 1

**Slug**: civ-021-recovered-requirements | **Epic**: E2 | **Date**: 2026-06-10 | **State**: ACTIVE
**Workstream branch**: `docs/phantom-id-triage` (in `D:/civis-build/triage`)
**Audit source**: [`docs/audits/phantom-triage-batch1.md`](../../../docs/audits/phantom-triage-batch1.md)

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
  drop the 12 stub IDs from `CODE-ONLY-no-spec` into `COVERED` (they
  now have a spec doc with the same `fr_ids` keys in the YAML front
  matter). The matrix is the source of truth; this doc is the harness.
- The verify gate `just civis-3d-verify` MUST remain green.

## Follow-up batches

The remaining 686 `CODE-ONLY-no-spec` rows will be triaged in subsequent
batches using the same template. This spec doc grows by appending new
stubs (no overwrite); the `meta.json` `fr_ids` array grows likewise.
