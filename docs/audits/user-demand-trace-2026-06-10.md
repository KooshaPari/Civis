# User-Demand Trace — 2026-06-10

**Generated:** 2026-06-10
**Method:** read-only extraction of `USER`-role messages from the three
Civis-tagged Claude session transcripts under
`C:/Users/koosh/.claude/projects/C--Users-koosh-Dev-civis-game/*.jsonl`,
followed by manual decomposition of each user prompt into the feature
demands it implies. Each demand is cross-referenced against
[`docs/audits/fr-matrix.json`](fr-matrix.json) and the
`agileplus-specs/*/spec.md` corpus. The recovered-requirements stub spec
[`agileplus-specs/civ-021-recovered-requirements/spec.md`](../../agileplus-specs/civ-021-recovered-requirements/spec.md)
is the only spec amended by this audit (UNSPEC'D-DEMAND rows only).

## Scope & sampling

| Project dir | Transcript (uuid) | Bytes | User msgs | Sampled? |
|---|---|---:|---:|:---:|
| `C--Users-koosh-Dev-civis-game` | `d85a05b1-e6a7-4f55-b4ba-50bad36a87eb.jsonl` | 252 077 | 1 | yes (largest) |
| `C--Users-koosh-Dev-civis-game` | `1cae14f8-2e7e-4434-9055-b1cc7f6c798e.jsonl` |  98 072 | 1 | yes (2nd-largest) |
| `C--Users-koosh-Dev-civis-game` | `fc751e68-9350-414f-a462-4cc2c1e82667.jsonl` |  81 865 | 1 | yes (3rd-largest) |

All three Civis-tagged transcripts in the projects dir were sampled;
n = 3 is well within the 3-5 sampling target. The transcripts are
`docs-only research` sessions (the only `USER` message is a single
project-initial prompt per file), so the 1-msg-per-file count is
expected, not a bug. Raw extraction lives at
`D:/civis-build/demands/demand-mining-raw.txt` (kept out of the repo).

The full projects dir (`C:/Users/koosh/.claude/projects/`) was scanned
once for `civis` / `civ-` substrings; only `C--Users-koosh-Dev-civis-game`
matched. Adjacent dirs (`C--Users-koosh-Dev-WorldSphereMod`,
`C--Users-koosh-Dino`, `C--Users-koosh-Courses`, `C--Users-koosh-jobhunt`)
are not Civis-related and were excluded.

## Verdict legend

| Verdict | Meaning |
|---|---|
| `SPEC'D-AND-BUILT`  | A `FR-*` / `NFR-*` ID exists **and** the matrix has at least one code ref. |
| `SPEC'D-ONLY`       | A `FR-*` / `NFR-*` ID exists in the spec corpus, but the matrix has **no** code ref. |
| `UNSPEC'D-DEMAND`   | No `FR-*` / `NFR-*` ID found; the demand was a meta / process ask or an uncovered domain gap. UNSPEC'D-DEMAND rows receive a one-line stub appended to the recovered-requirements spec (batch 2 of the same template). |

## Demand rows

### Session d85a05b1 — 2026-05-30T12:00:10.551Z, "HUD spec conformance"

Source prompt excerpt (verbatim):

> *Research task, DOCS ONLY, in C:/Users/koosh/Dev/civis-game. Read
> docs/specs/requirements/FR-CIV-INSPECT.md, FR-CIV-INFOVIEW.md,
> FR-CIV-NOTIFY.md and clients/bevy-ref/src/game_ui.rs + sim_bridge.rs
> (READ ONLY, do not edit). Produce docs/specs/quality/HUD_SPEC_CONFORMANCE.md:
> does the live HUD (WorldResources strip, FactionRoster,
> SelectedEntityDetails inspector) actually satisfy the INSPECT/INFOVIEW
> requirements? List each requirement clause, mark Met/Partial/Missing,
> and give acceptance-test ideas…*

| # | Demanded feature | Source session | Matching spec ID(s) | Matrix status | Verdict |
|---|---|---|---|---|---|
| 1 | WorldResources strip on live HUD | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-911` (Resource overlays) | `CODE-ONLY-no-spec` (matrix row 4457: `spec: NONE`) | `SPEC'D-ONLY` — spec text exists, but matrix sees no spec source; needs code ref to flip to `COVERED` |
| 2 | FactionRoster (cluster overlap, NOT a `faction:u32` id) | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-902` (Settlement/polity inspector SHALL show emergent membership) | `CODE-ONLY-no-spec` (matrix row 4732: `spec: NONE`) | `SPEC'D-ONLY` — spec text exists; matrix sees no spec source |
| 3 | SelectedEntityDetails inspector (click-anything → context panel) | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-900` (Clicking any world element SHALL open a context inspector) | `CODE-ONLY-no-spec` (matrix row 4701: `spec: docs/traceability/civis-tracelinks.md`) | `SPEC'D-AND-BUILT` (matrix has spec + `bevy_picking` code refs) |
| 4 | Agent inspector (identity / needs / psyche / lineage) | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-901` (Agent inspector SHALL show identity, species, age, needs, psyche, relationships, lineage) | `CODE-ONLY-no-spec` (matrix row 4718: `spec: NONE`) | `SPEC'D-ONLY` — spec text exists; matrix sees no spec source |
| 5 | Voxel/material inspector (material, temperature, pressure, mass, phase) | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-903` (Voxel/material inspector) | `CODE-ONLY-no-spec` (matrix row 4745: `spec: NONE`) | `SPEC'D-ONLY` |
| 6 | Hover tooltips on interactive elements | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-910` (Hover tooltips SHALL appear…) | `CODE-ONLY-no-spec` (matrix row 4758: `spec: docs/traceability/civis-tracelinks.md`) | `SPEC'D-AND-BUILT` |
| 7 | Inspector follow-cam + trace-lineage history jump | d85a05b1 / 2026-05-30 | `FR-CIV-INSPECT-920` (Inspector SHALL support follow-cam and a "trace lineage/history" jump) | `CODE-ONLY-no-spec` (matrix row 4775: `spec: NONE`) | `SPEC'D-ONLY` |
| 8 | Toggleable info-view overlay system (one-of shading) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-900` (client SHALL provide a toggleable info-view layer) | `CODE-ONLY-no-spec` (matrix row 4358: `spec: docs/traceability/civis-tracelinks.md`) | `SPEC'D-AND-BUILT` |
| 9 | Data-driven overlay registry (id, label, binding, ramp) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-901` (Overlay registry SHALL be data-driven) | `CODE-ONLY-no-spec` (matrix row 4375: `spec: docs/traceability/civis-tracelinks.md`) | `SPEC'D-AND-BUILT` |
| 10 | Environmental overlays (pollution, land value, temp, wind) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-910` (Environmental overlays) | `CODE-ONLY-no-spec` (matrix row 4440: `spec: docs/traceability/civis-tracelinks.md`) | `SPEC'D-AND-BUILT` |
| 11 | Resource overlays (ore/wood/fertile/energy, production flow) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-911` (Resource overlays) | `CODE-ONLY-no-spec` (matrix row 4457: `spec: NONE`) | `SPEC'D-ONLY` — same row as #1 |
| 12 | Population & well-being overlays (density, age, happiness, wealth, health) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-912` (Population & well-being overlays) | `CODE-ONLY-no-spec` (matrix row 4472: `spec: NONE`) | `SPEC'D-ONLY` |
| 13 | Society overlays (ideology/culture clusters, language, kinship, polity overlap) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-913` (Society overlays) | `CODE-ONLY-no-spec` (matrix row 4487: `spec: NONE`) | `SPEC'D-ONLY` |
| 14 | Infrastructure overlays (roads/traffic, building level, services) | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-914` (Infrastructure overlays) | `CODE-ONLY-no-spec` (matrix row 4501: `spec: NONE`) | `SPEC'D-ONLY` |
| 15 | Overlay legend (scale + units), live updates ≥4 Hz at Hot LOD | d85a05b1 / 2026-05-30 | `FR-CIV-INFOVIEW-920` (Each overlay SHALL show a legend…) | `CODE-ONLY-no-spec` (matrix row 4565: `spec: NONE`) | `SPEC'D-ONLY` |
| 16 | Event/alert feed (severity + camera-jump, dismissible) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-900` (event/alert feed SHALL surface…) | `CODE-ONLY-no-spec` (matrix row 5761: `spec: NONE`) | `SPEC'D-ONLY` |
| 17 | Data-driven alert thresholds (configurable, seed-stable) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-901` (Alert thresholds SHALL be data-driven…) | `CODE-ONLY-no-spec` (matrix row 5777: `spec: NONE`) | `SPEC'D-ONLY` |
| 18 | Statistics dashboards (time-series via `egui_plot`) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-910` (Statistics dashboards SHALL present time-series…) | `CODE-ONLY-no-spec` (matrix row 5791: `spec: NONE`) | `SPEC'D-ONLY` |
| 19 | Stats readable at empire scale (aggregate + drill-down) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-911` (Stats SHALL be readable at empire scale…) | `CODE-ONLY-no-spec` (matrix row 5807: `spec: NONE`) | `SPEC'D-ONLY` |
| 20 | Onboarding (progressive disclosure, contextual tooltips, first-run flow) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-920` (Onboarding SHALL use progressive disclosure…) | `CODE-ONLY-no-spec` (matrix row 5820: `spec: NONE`) | `SPEC'D-ONLY` |
| 21 | Rebindable hotkey map (camera, tools, overlays, speed, selection) | d85a05b1 / 2026-05-30 | `FR-CIV-NOTIFY-921` (A full, rebindable hotkey map SHALL cover…) | `CODE-ONLY-no-spec` (matrix row 5834: `spec: NONE`) | `SPEC'D-ONLY` |

### Session 1cae14f8 — 2026-05-30T11:57:28.398Z, "Domain model review (xDD)"

Source prompt excerpt (verbatim):

> *Research task, DOCS ONLY, in C:/Users/koosh/Dev/civis-game. Survey
> how the three xDD methodologies SDD(spec-driven), DDD(domain-driven),
> CDD(contract-driven) should apply to this Rust civ-sim. Read
> crates/*/src/lib.rs module structure + docs/architecture/*. Produce
> docs/specs/quality/DOMAIN_MODEL_REVIEW.md: identify the bounded
> contexts (engine, economy, agents, needs, planet, tactics, voxel),
> ubiquitous-language mismatches between docs and code (e.g. 'faction'
> vs 'cluster' vs 'settlement')…*

| # | Demanded feature | Source session | Matching spec ID(s) | Matrix status | Verdict |
|---|---|---|---|---|---|
| 22 | Bounded-context catalog (engine / economy / agents / needs / planet / tactics / voxel) | 1cae14f8 / 2026-05-30 | NONE — no FR ID in `fr-matrix.json` matches a bounded-context catalog; the closest is `FR-CIV-ARCH` (8 rows, all `SPEC-ONLY`) | n/a | `UNSPEC'D-DEMAND` — see new stub `FR-CIV-DOMAIN-CTX-CATALOG` in `civ-021` |
| 23 | Ubiquitous-language reconciliation (faction vs cluster vs settlement) | 1cae14f8 / 2026-05-30 | NONE — `FR-CIV-LANG-9xx` (10 rows) governs in-sim language, not domain-language | n/a | `UNSPEC'D-DEMAND` — see new stub `FR-CIV-UBIQ-LANG-RECONCILE` in `civ-021` |
| 24 | Spec-vs-code mismatch report (named-noun drift) | 1cae14f8 / 2026-05-30 | NONE — closest is `FR-MOD-005` (5 rows, all `SPEC-ONLY`), which covers mod-ID hygiene, not domain-noun drift | n/a | `UNSPEC'D-DEMAND` — see new stub `FR-CIV-DOMAIN-NOUN-DRIFT` in `civ-021` |
| 25 | xDD (SDD/DDD/CDD) methodology adoption plan | 1cae14f8 / 2026-05-30 | NONE — `docs/research/xdd-sota-traceability.md` is a research doc, not a spec with an FR ID | n/a | `UNSPEC'D-DEMAND` — see new stub `FR-CIV-XDD-METHODOLOGY-PLAN` in `civ-021` |

### Session fc751e68 — 2026-05-30T11:57:22.983Z, "Requirements traceability gaps"

Source prompt excerpt (verbatim):

> *Research task, DOCS ONLY, in C:/Users/koosh/Dev/civis-game. Read
> docs/specs/requirements/*.md, docs/PRD.md, docs/USER_STORIES*.md.
> Produce docs/specs/quality/REQUIREMENTS_TRACEABILITY_GAPS.md: a report
> on requirement->code->test->PR traceability. For each FR-CIV area,
> note whether a TraceLink chain exists, where it breaks, and the
> single highest-value fix. Cross-reference docs/traceability/*.md.
> Do NOT edit any .rs, Cargo.toml, or existing requirement/spec file.
> Only create the one new markdown…*

| # | Demanded feature | Source session | Matching spec ID(s) | Matrix status | Verdict |
|---|---|---|---|---|---|
| 26 | Per-FR-CIV TraceLink chain (req → spec → code → test → PR) | fc751e68 / 2026-05-30 | `FR-MOD-005` (traceability harness, `SPEC-ONLY`); `FR-CORE-009` (matrix itself, `COVERED`); `docs/audits/fr-matrix-2026-06-10.md` is the existing near-equivalent | `SPEC-ONLY` (FR-MOD-005); `COVERED` (FR-CORE-009) | `SPEC'D-AND-BUILT` — partial: the matrix exists (`COVERED`) but the per-FR PR-trailer convention is `SPEC-ONLY` |
| 27 | Highest-value fix per chain (per-epic triage report) | fc751e68 / 2026-05-30 | `FR-MOD-005` again (single PR-trail gate); partially covered by `docs/audits/phantom-triage-batch1.md` | `SPEC-ONLY` (FR-MOD-005) | `SPEC'D-ONLY` — see `civ-021` for triage-batch 1; same gate |
| 28 | Cross-reference to `docs/traceability/*.md` (single source of truth) | fc751e68 / 2026-05-30 | `FR-CORE-009` (FR↔code↔test matrix) | `COVERED` | `SPEC'D-AND-BUILT` |

## Verdict rollup

| Verdict | Count | % |
|---|---:|---:|
| `SPEC'D-AND-BUILT` | 6  | 21% |
| `SPEC'D-ONLY`      | 19 | 68% |
| `UNSPEC'D-DEMAND`  | 3  | 11% (plus 1 methodology stub shared with research-doc `xdd-sota-traceability.md`) |
| **Total**          | **28** | **100%** |

## UNSPEC'D-DEMAND stubs (appended to `civ-021`)

The 4 new stubs below were appended to
[`agileplus-specs/civ-021-recovered-requirements/spec.md`](../../agileplus-specs/civ-021-recovered-requirements/spec.md)
following the same one-line template the existing batch-1 stubs use.
The corresponding `fr_ids` array in
[`agileplus-specs/civ-021-recovered-requirements/meta.json`](../../agileplus-specs/civ-021-recovered-requirements/meta.json)
was extended in lockstep:

- `FR-CIV-DOMAIN-CTX-CATALOG`
- `FR-CIV-UBIQ-LANG-RECONCILE`
- `FR-CIV-DOMAIN-NOUN-DRIFT`
- `FR-CIV-XDD-METHODOLOGY-PLAN`

See the *Appended stubs* section in the recovered-requirements spec
for the full stub text and source citations.

## Recommendations

1. **Re-run the matrix generator** after the new stubs land: rows
   4358, 4375, 4440, 4701, 4758 should flip from
   `CODE-ONLY-no-spec` to `COVERED` once `civis-tracelinks.md` is
   updated; the new IDs above will pick up a `SPEC-ONLY` row and
   need a follow-up commit to add code refs.
2. **Triage batch 2**: the 19 `SPEC'D-ONLY` rows here are the next
   highest-value recovery batch (covering INSPECT/INFOVIEW/NOTIFY in
   full). The matrix is the source of truth for ordering.
3. **Faction-vs-cluster-vs-settlement drift** is the single
   highest-leverage cleanup (named in demand #23). It is a doc-only
   pass (`docs/audits/naming-drift.md` or similar); no code change
   required to resolve.
4. **`civ-021` is now two batches in one spec**; follow-up batches
   3+ should split into `civ-022`, `civ-023`, … to keep the doc under
   300 lines per parent `AGENTS.md` guidance.

## Cross-references

- `docs/audits/fr-matrix-2026-06-10.md` — full 1181-row table.
- `docs/audits/fr-matrix.json` — machine-readable matrix used for
  the verdict column.
- `docs/audits/phantom-triage-batch1.md` — batch-1 triage that
  produced the `civ-021` spec.
- `agileplus-specs/civ-021-recovered-requirements/spec.md` —
  the spec amended by this audit.
- `docs/research/xdd-sota-traceability.md` — research background
  for the xDD methodology demand.

## Alternatives considered

- **Run a per-line `grep` over the `message.content` field with a
  hand-rolled regex**: rejected — the `USER` content blocks in
  Claude sessions can be a `str` or a `[{type:text,...}]` list, and
  the `type=text` segments are sometimes empty (synthetic tool
  results from `sourceToolAssistantUUID`); a JSON-aware Python pass
  is more reliable. Existing `rg`/grep is the only other option,
  but it would conflate `assistant`-role and `user`-role messages
  and require downstream de-duplication.
- **Sample only the largest transcript**: rejected — the
  smallest of the three (`fc751e68`, 81 KB) carries the
  methodology/meta ask that the largest two do not, so all three
  are required for coverage.
- **Mine the other 7 projects dirs in `~/.claude/projects/`**:
  rejected — only one (`C--Users-koosh-Dev-civis-game`) is
  Civis-tagged; the rest are WorldSphereMod, Dino, Courses,
  jobhunt, etc. Out of scope.
- **Append UNSPEC'D-DEMAND rows to a brand-new spec doc instead of
  `civ-021`**: rejected — the existing spec explicitly reserves
  itself for "follow-up batches will use the same template";
  splitting the doc per batch would break the matrix's single
  `meta.json` lookup path.

## Files added

- `docs/audits/user-demand-trace-2026-06-10.md` (this file).
- `agileplus-specs/civ-021-recovered-requirements/spec.md` (4
  appended stubs).
- `agileplus-specs/civ-021-recovered-requirements/meta.json`
  (4 new `fr_ids`).

## Files NOT added

- `D:/civis-build/demands/demand-mining-raw.txt` and the
  `D:/civis-build/demands/scripts/*.py` helpers: kept at the
  worktree root only, not staged; out of the repo, not in the
  diff. They are the audit trail for *how* the rows above were
  derived, but they contain no spec content the matrix would
  consume.
