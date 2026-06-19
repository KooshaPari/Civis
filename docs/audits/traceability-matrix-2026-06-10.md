# Traceability Matrix — 2026-06-10 Workstream Audit

**Date:** 2026-06-10
**Scope:** Map the 2026-06-10 workstream surface (wave-1 PR stack
`#333/#334/#335/#336` + hygiene `#341/#342/#343/#344` + wave-1
foundation merge `af913fb2` + terrain fragment PR pending + verify
harness extension worktrees + CA dirty-chunk perf
`perf/ca-dirty-chunk-v3` + emergence dashboard
`feat/emergence-dashboard` + CI billing guard worktree + dep sweep
`chore/dependabot-sweep-20260610` + README worktrees
`docs/readme-workstate-*`) to the `FR-*` / `NFR-*` IDs in
`agileplus-specs/civ-001..civ-020`.
**Sources of truth:**

- `agileplus-specs/civ-001..civ-020/*/meta.json` — FR-ID catalogues
- `docs/reference/non-functional-requirements.md` (af913fb2) — NFR-ID catalogue
- `docs/traceability/fr-3d-matrix.md` (af913fb2) — partial 3D matrix
- `docs/traceability/full-traceability-matrix.md` (af913fb2) — full matrix
- `docs/reports/wave-1-pr-stack-2026-06-05.md` (af913fb2) — CI blocker report

**Companion docs:**

- `docs/audits/traceability-matrix-2026-06-10.md` (this file)
- `agileplus-specs/civ-014..civ-020/{meta.json,plan.md,spec.md}` (spec triples)

**Notation:**

- **Spec** = `agileplus-specs/civ-XXX-<slug>` directory; triple = `meta.json + plan.md + spec.md`
- **FR** = `FR-<DOMAIN>-<NNN>` (e.g. `FR-CIV-VERIFY-001`); one FR maps to one spec
- **NFR** = `NFR-CIV-<DOMAIN>-<NNN>` (e.g. `NFR-CIV-PERF-003`); from the NFR doc
- **Status:** `merged` (in main, full); `pending` (PR open or worktree ready, not yet merged); `planned` (spec exists, implementation not started)

---

## Section 1 — Workstream → Spec → FR/NFR Cross-Walk

The workstreams are listed in the order they appear in the 2026-06-10
agent dispatch. Each workstream row points to the spec(s) that own
the work and the FR/NFR IDs that gate it.

| # | Workstream | Branch / PR | State | Spec (civ-XXX) | FR IDs | NFR IDs | Notes |
|---|---|---|---|---|---|---|---|
| 1 | **Wave-1 foundation** | `af913fb2` (merged) | merged | civ-001..civ-013, civ-014..civ-017 (this PR) | FR-CORE-001/002/003/005, FR-ECON-001/002/003/005, FR-PROTO-001/002, FR-CLIENT-001/003, FR-REPLAY-001/002, FR-API-001, FR-METRICS-001/002/003, FR-CIV-VOXEL-000/001/002/003/004/010, FR-CIV-BUILD-000/001/010/020/030, FR-CIV-GENETICS-000/001/002/010, FR-CIV-SPECIES-000/001, FR-CIV-AGENTS-000/001/010, FR-CIV-DIFFUSION-000/001, FR-CIV-LAWS-000/001/002, FR-CIV-RESEARCH-000/001/002/003, FR-CIV-TACTICS-000..077, FR-CIV-PLANET-000/001/002, FR-CIV-PROTO3D-000/001/002, FR-CIV-UX-000/001/004/006, FR-CIV-BEVY-002/009/012/014/017/023/024/026, FR-CIV-CA-001..005, FR-CIV-TERRAIN-001..006, FR-CIV-TACTICS-100/101/102, FR-CIV-FOG-001..005, FR-CIV-VERIFY-001..006, FR-CIV-MCP-001..006 | NFR-CIV-PERF-001..007, NFR-CIV-DET-001..004, NFR-CIV-SCALE-001..003, NFR-CIV-REL-001..004, NFR-CIV-SEC-001..004, NFR-CIV-ACC-001..004, NFR-CIV-PORT-001..003, NFR-CIV-MAINT-001..006 | 476-commit foundation; 516 files / +61,230 lines; fast-forward landed on `origin/main` 2026-06-10. |
| 2 | **Wave-1 PR stack** (CI follow-up) | PRs `#333`, `#334`, `#335`, `#336` | merged (foundation) + pending (CI gates) | civ-016, civ-018 (this PR) | FR-CIV-VERIFY-001..010, FR-CIV-MCP-001..006 | NFR-CIV-MAINT-001..006, NFR-CIV-SEC-004 | 3 remaining CI blockers per `docs/reports/wave-1-pr-stack-2026-06-05.md`: (1) pr-governance-gate runner cache, (2) GitGuardian on PR #333, (3) `rust` test flakiness on `#334`/`#335`. **CI pending billing fix** for pr-governance-gate; cache-expiry forward-fix preferred over revert. |
| 3 | **CI governance** | PRs `#345`, `#347` | pending | civ-016, civ-018 (this PR) | FR-CIV-VERIFY-003/004/008/010 | NFR-CIV-MAINT-001, NFR-CIV-SEC-004 | `#345` is the pr-governance-gate rename + `@v7` tag fix landed in `af913fb2`; `#347` is the GitGuardian skip-condition fix (security-guard.sh: `exit 1` now propagates). `agent-smoke.ps1` is the green-gate. |
| 4 | **README workstate** | PR `#348` | pending | civ-016 (cross-link from spec), civ-001 (overview) | FR-CIV-VERIFY-001, FR-CORE-001 | — | Three README worktrees (`civis-wt-readme3/4/5`) on `docs/readme-workstate-20260610*` branches; one PR lands; two are scratch (locked). |
| 5 | **Dep sweep** | branch `chore/dependabot-sweep-20260610` (worktree `E:/civis-wt-depsweep`) | pending | civ-001, civ-016 (verify) | FR-CIV-VERIFY-003, FR-CIV-MAINT-005 | NFR-CIV-SEC-004, NFR-CIV-MAINT-005 | Dependabot weekly; gated by `cargo audit` + `cargo deny` + license check; failures block merge. Replaces the wave-1-era `chore/dependabot-{frontend,rust}-2026-06-05` PRs (`#334`/`#335`) which were part of the wave-1 stack. |
| 6 | **Verify harness extension** | worktree `E:/civis-wt-verify` (branch `feat/verify-harness`) | planned | **civ-018** (this PR, new) | FR-CIV-VERIFY-007/008/009/010 | NFR-CIV-MAINT-001, NFR-CIV-REL-004 | Adds `scripts/ci/audit-pr-queue.sh`, `scripts/ci/audit-worktrees.sh`, `scripts/ci/with-cargo-target.sh` — closes the civ-016 E9.7–E9.9 spec gap. |
| 7 | **CA dirty-chunk perf** | branch `perf/ca-dirty-chunk-v3` (worktree `C:/Users/koosh/Dev/Civis` + `E:/civis-wt-ca-dirty` + `E:/civis-wt-ca-dirty-tmp`) | planned | **civ-020** (this PR, new), civ-014 | **FR-CIV-CA-001..005** (this PR), FR-CIV-TERRAIN-003, FR-CIV-VOXEL-001/002/003 | NFR-CIV-PERF-003/005, NFR-CIV-DET-001 | Bottleneck fix for the wave-1 CA fluid/thermo/percolation upgrade; target P99 < 16 ms on 64×64 grid, 1% writes. Determ. invariant must hold (`ca_dirty_chunk_optimisation_preserves_determinism`). |
| 8 | **Emergence dashboard** | branch `feat/emergence-dashboard` (worktree `E:/civis-wt-emergence-dash`) | planned | **civ-019** (this PR, new), civ-013, civ-009 | **FR-CIV-EMERG-001..005** (this PR), FR-METRICS-001/002/003, FR-CIV-AGENTS-001, FR-CIV-DIFFUSION-001, FR-CIV-CULT-001 | NFR-CIV-PERF-003, NFR-CIV-DET-001 | Read-only panel: ClusterEntropy, IdeologyHomophilyIndex, SentienceFraction, PsycheStability, DiplomacyTensionIndex. Web `EmergencePanel` + Bevy `live_emergence_overlay` (E). |
| 9 | **Terrain fragment ship** | branch `fix/terrain-fragmentation-ship` (worktree `E:/civis-wt-terrain-ship`) | pending | civ-014 (terrain playability), civ-020 (CA-perf cross-link) | FR-CIV-TERRAIN-001/002/003, FR-CIV-CA-001..005 | NFR-CIV-PERF-005, NFR-CIV-DET-001 | Terrain PR pending; the chunk-seam + CA-dirty-chunk work co-land with civ-020. |
| 10 | **CI billing guard** | branch `fix/ci-billing-guard-alert-sync` (worktree `E:/civis-wt-ci-billing-guard-fresh`) | planned | civ-016, civ-017 | FR-CIV-VERIFY-001/003, FR-CIV-MCP-001/005 | NFR-CIV-SEC-002 (env-only secrets) | The "CI pending billing fix" the tasking flags: the pr-governance-gate has been failing because of the `actions/github-script` SHA cache (per `docs/reports/wave-1-pr-stack-2026-06-05.md`, Blocker 1). The fresh worktree is the forward-fix; cache-expiry or admin re-registration required. |

---

## Section 2 — FR-ID → Spec Cross-Reference

The catalogue of FR IDs in the spec triples, grouped by domain. Use
this section to look up "which spec owns this FR ID" in O(1) mental
lookup.

### 2.1 Strategic CivLab core (FR-CORE / FR-ECON / FR-PROTO / FR-CLIENT / FR-REPLAY / FR-API / FR-METRICS)

| FR ID | Spec | Title |
|---|---|---|
| FR-CORE-001..007 | civ-001 | Core Simulation Engine |
| FR-ECON-001..005 | civ-002 | Economy and Joule System |
| FR-METRICS-001..003 | civ-002 | (lives with economy; metrics are engine-side) |
| FR-CIV-ACTOR-001/002, FR-CIV-SOCIAL-001/002 | civ-003 | Actor and Citizen Lifecycle |
| FR-CIV-BUILD-001/002/003 | civ-004 | Building Tiers and Production Chains |
| FR-CIV-CLIMATE-001/002/003 | civ-005 | Climate, Disasters, and Seasons |
| FR-CIV-WAR-001..004 | civ-006 | Deep Combat System |
| FR-CIV-DIPLO-001/002/003, FR-CIV-GOV-001/002 | civ-007 | Diplomacy, Laws, and Government |
| FR-CIV-BIO-001/002/003 | civ-008 | Genetics and Species Diversity |
| FR-CIV-CULT-001/002/003 | civ-009 | Culture Diffusion and Ideology Spread |
| FR-PROTO-001..005, FR-CLIENT-003 | civ-010 | Multi-Client Protocol (WebSocket + Binary Frames) |
| FR-CLIENT-001, FR-CIV-HUD-001..005 | civ-011 | Bevy Primary Client (3D, DX12/DLSS) |
| FR-CIV-CLIENT-GODOT-001/002 | civ-012 | Godot Secondary Client |
| FR-API-001..004, FR-REPLAY-001/002 | civ-013 | Research API and Scenario System |

### 2.2 3D extension (FR-CIV-*)

| FR ID prefix | Spec | Title |
|---|---|---|
| FR-CIV-TERRAIN-001..006 | civ-014 | Terrain Playability Hardening |
| FR-CIV-TACTICS-100/101/102, FR-CIV-FOG-001..005 | civ-015 | Tactics, Fog-of-War & Combat Pipeline |
| FR-CIV-VERIFY-001..010 | civ-016 | Developer-Experience Verify Harness & Worktree Hygiene |
| FR-CIV-MCP-001..006 | civ-017 | Civis MCP Server |
| FR-CIV-VERIFY-007/008/009/010 (extension) | **civ-018** (new) | Verify Harness Extension |
| FR-CIV-EMERG-001..005 (new domain) | **civ-019** (new) | Emergence Metrics Dashboard |
| FR-CIV-CA-001..005 (new domain) | **civ-020** (new) | CA Dirty-Chunk Performance |

> The `FR-CIV-VERIFY-007/008/009/010` rows are also catalogued in
> civ-016 (E9.6 / E9.7 / E9.8 / E9.9); civ-018 is the **authoritative
> spec for the scripts**, and civ-016 is the **umbrella spec** for
> the harness as a whole. See "Missing spec entries" below for the
> rationale.

### 2.3 3D matrix rows (FR-CIV-VOXEL/BUILD/GENETICS/SPECIES/AGENTS/DIFFUSION/LAWS/RESEARCH/TACTICS/PLANET/PROTO3D/UX/CA/EMERG/TERRAIN/FOG/MCP/VERIFY/BEVY)

The 3D matrix in `docs/traceability/fr-3d-matrix.md` enumerates 160+
rows. The cross-walk above pins each **FR-ID prefix** to its spec
home; per-row status (Done / Partial / Planned) lives in
`docs/traceability/fr-3d-matrix.md` and is **not** duplicated here.

---

## Section 3 — NFR-ID → Workstream Cross-Reference

The NFR catalogue (`docs/reference/non-functional-requirements.md`,
shipped in `af913fb2`) defines 35 NFRs across 8 categories. The
2026-06-10 workstreams bind to NFRs as follows:

| Category | NFR IDs | Workstream binding |
|---|---|---|
| **PERFORMANCE** | NFR-CIV-PERF-001..007 | `#1` (wave-1: 60 FPS at 1k entities, M1/Metal, tick budget, scaling, mesh budget, memory, bandwidth) → `#7` (CA-perf: P99 < 16 ms on dirty chunk) → `#8` (emergence: ≤ 200 µs P99 added to diffusion) |
| **DETERMINISM** | NFR-CIV-DET-001..004 | `#1` (wave-1: cross-run, cross-platform, fixed-point, RNG logging) → `#7` (CA-perf: optimisation must not break determinism) → `#8` (emergence: replay-bus event must not change hash chain) |
| **SCALE** | NFR-CIV-SCALE-001..003 | `#1` (wave-1: 1M agent roadmap gates, chunk streaming, simultaneous clients) |
| **RELIABILITY** | NFR-CIV-REL-001..004 | `#1` (wave-1: no panics, loud failure for deps, autosave cadence, no silent corruption) → `#6` (verify harness: auditable scripts) |
| **SECURITY** | NFR-CIV-SEC-001..004 | `#1` (wave-1: WASM sandbox, env-only secrets, no network egress, zero high/critical) → `#5` (dep sweep: cargo audit + deny) → `#10` (CI billing guard: env-only secret scan) |
| **ACCESSIBILITY** | NFR-CIV-ACC-001..004 | `#8` (emergence dashboard: colorblind-safe threshold chips, keybind for overlay) |
| **PORTABILITY** | NFR-CIV-PORT-001..003 | `#1` (wave-1: target platform matrix, backend selection docs, headless server) |
| **MAINTAINABILITY** | NFR-CIV-MAINT-001..006 | `#1` (wave-1: coverage threshold, complexity caps, duplication, docstring coverage, boundary enforcement, zero lint suppressions) → `#5` (dep sweep) → `#6` (verify harness) |

---

## Section 4 — Missing Spec Entries (closes the spec gap)

Three workstreams have **no spec home** as of the 2026-06-10 audit.
This PR adds the spec triples in canonical format (meta.json +
plan.md + spec.md).

| Spec (new) | FR-ID prefix | Workstream | Why it was missing |
|---|---|---|---|
| **civ-018-verify-harness-extension** | FR-CIV-VERIFY-007/008/009/010 (authoritative rows) | `feat/verify-harness` (worktree `E:/civis-wt-verify`) | The civ-016 umbrella spec tracks E9.7–E9.9 as "Planned" but does not author the scripts. The scripts (`audit-pr-queue.sh`, `audit-worktrees.sh`, `with-cargo-target.sh`) need their own spec to be auditable. |
| **civ-019-emergence-metrics-dashboard** | FR-CIV-EMERG-001..005 (new domain) | `feat/emergence-dashboard` (worktree `E:/civis-wt-emergence-dash`) | The wave-1 emergence foundation shipped 5+ emergent behaviours but no engine-level metrics block, no JSON-RPC `sim.snapshot.emergence` field, no web `EmergencePanel`, no Bevy `live_emergence_overlay`. The FR-CIV-EMERG-* family is the spec home. |
| **civ-020-ca-perf-dirty-chunk** | FR-CIV-CA-001..005 (new domain) | `perf/ca-dirty-chunk-v3` (worktree `C:/Users/koosh/Dev/Civis` + `E:/civis-wt-ca-dirty`) | The wave-1 CA fluid/thermo/percolation upgrade landed in `af913fb2` but the perf gate (`bench_ca_dirty_chunk` P99 < 16 ms) is not yet wired and the optimisation plan is not spec'd. The FR-CIV-CA-* family is the spec home. |

### 4.1 Canonical spec triple format

Each new spec directory contains exactly three files matching the
wave-1-era format (see `agileplus-specs/civ-001-core-simulation-engine/`
as the reference):

```
agileplus-specs/civ-XXX-<slug>/
  meta.json   # spec_id, slug, title, status, type, epic, fr_ids[], priority, target_release, created_at, updated_at
  plan.md     # Phased WBS (Phase 1..N with Task table + DAG dependencies)
  spec.md     # YAML frontmatter + Problem Statement + FR/NFR/AC + Status
```

Field-level conformance verified against `civ-001` meta.json:

| Field | Type | Required | Example (civ-018) |
|---|---|---|---|
| `spec_id` | string | yes | `"civ-018"` |
| `slug` | string | yes | `"civ-018-verify-harness-extension"` |
| `title` | string | yes | `"Verify Harness Extension ..."` |
| `status` | enum | yes | `"active"` |
| `type` | enum | yes | `"feature"` |
| `epic` | string | yes | `"E9"` |
| `fr_ids` | string[] | yes | `["FR-CIV-VERIFY-007", ...]` |
| `created_at` | ISO-8601 | yes | `"2026-06-10T00:00:00Z"` |
| `updated_at` | ISO-8601 | yes | `"2026-06-10T00:00:00Z"` |
| `priority` | enum | yes | `"SHALL"` |
| `target_release` | string | yes | `"MVP"` |

---

## Section 5 — Open Gaps & Forward-Fixes

| Gap | Owner | Forward-fix | NFR binding |
|---|---|---|---|
| pr-governance-gate runner SHA cache | CI infra (cannot fix from CLI) | Wait for cache expiry (typically <24h) or disable+re-enable the workflow; or replace `actions/github-script@v7` with a pure node script (last resort). | NFR-CIV-MAINT-001, NFR-CIV-SEC-004 |
| GitGuardian flagged string in wave-1 history | Repo owner (dashboard access) | Review `https://dashboard.gitguardian.com`, add `.gitguardian.yaml` allow-list entry or fix the flagged string. | NFR-CIV-SEC-004 |
| `cargo test` flakiness on `#334`/`#335` (curl 8.0 → `proxy.golang.org`) | CI infra | Add `nick-fields/retry@v3` step with `max_attempts: 3`. | NFR-CIV-REL-001 |
| **CI billing guard** (this PR's headline "CI pending billing fix" note) | Repo owner (billing dashboard) | The pr-governance-gate is in a broken state because the runner can't resolve `actions/github-script`; this is a **GitHub Actions billing / minutes** issue, not a code issue. Forward-fix: re-pin `@v7` after cache expires, or register a new SHA. | NFR-CIV-SEC-002, NFR-CIV-MAINT-001 |
| Wave-1 era specs (civ-014..civ-017) carry-over | This PR | The predecessor's 014..017 spec triples are added in this PR so the spec catalogue is complete on `main` (these were present in the predecessor worktree but never committed). | NFR-CIV-MAINT-004 |

---

## Section 6 — Audit Notes

- This matrix is the **2026-06-10 snapshot**. Re-run on every wave to
  keep spec → workstream → FR/NFR coverage current.
- The 3D matrix in `docs/traceability/fr-3d-matrix.md` and the full
  matrix in `docs/traceability/full-traceability-matrix.md` (both
  shipped in `af913fb2`) remain the **per-row authoritative** sources;
  this file is the **workstream-level** cross-walk.
- NFR cells in the full matrix read "see NFR doc" (per the
  af913fb2-era convention). The NFR doc
  (`docs/reference/non-functional-requirements.md`) IS present on
  `main` as of `af913fb2`; future audits can replace "see NFR doc"
  with the concrete `NFR-CIV-*-NNN` IDs.
- The `FR-CIV-VERIFY-007/008/009/010` rows are intentionally listed
  under BOTH civ-016 (umbrella) AND civ-018 (authoritative). The
  civ-016 rows are the **policy** ("a worktree convention SHALL exist
  with PR-queue + worktree audits and a shared `CARGO_TARGET_DIR`");
  the civ-018 rows are the **implementation contract** for the three
  audit scripts. Same FR-ID, two specs, one rule.
- The `FR-CIV-CA-*` family is new with this PR. The wave-1 commit
  message mentioned "FR-CIV-CA-*" but the spec triple was never
  authored; this PR is the spec home.
- The `FR-CIV-EMERG-*` family is new with this PR. The wave-1
  foundation has 5+ emergent behaviours (sentience threshold,
  inter-cluster diplomacy, culture + language drift, psyche + social
  graph, insurgency pressure) but no engine-level metrics block; the
  spec defines the metrics, the snapshot field, and the dashboard +
  Bevy overlays.

---

**Last updated:** 2026-06-10
**Auditor:** trace-2026-06-10 worktree (forge agent, autonomous)
**Branch:** `docs/traceability-20260610` (DRAFT PR pending)
