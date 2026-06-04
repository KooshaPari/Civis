# Cross-Repo Quality/Acceptance Tooling Scan — 2026-06-03

Read-only survey to find EXISTING tooling for acceptance contracts, quality gates, SLA/perf
guards, bloat detection, SOTA-semantic guarding, and DAG/deployment orchestration — to decide
what to LIFT into a shared layer and where to PUSH new shared infra UP.

## TL;DR

- **The canonical shared-infra repo already exists:** `KooshaPari/phenotype-tooling`
  (local clone: `C:\Temp\phenotype-tooling`). It is a consolidated Rust workspace explicitly
  built to "replace duplicated shell/Python scripts scattered across repos." Most of what we
  want to build ALREADY EXISTS there as released crates.
- **Consumption is via reusable GitHub workflows** (`uses: KooshaPari/phenotype-tooling/.github/workflows/reusable/...@main`)
  and **git submodules** (Dino already submodules `phenotype-journeys`). This is the push-up mechanism.
- **Agent DAG/lane orchestration exists twice:** `phenotype-tooling/crates/agent-orchestrator`
  (lane-based, non-overlapping file scopes) and the daemon-backed `agent-runner`
  (`C:\Users\koosh\.claude\tools\agent-runner`, persistent `codex app-server` WS daemon).

---

## 1. What EXISTS and WHERE (lift-from targets + file paths)

### Priority target: `C:\Temp\phenotype-tooling` (KooshaPari/phenotype-tooling)
Rust workspace of clap-based CLIs. Top-level: `crates/ governance/ hooks/ packages/ Tools/ bin/ scripts/ .github/`.
README states each crate is "independently usable and can be adopted by other Phenotype repos
without copying implementation logic." Target consumers named in README:
`AgilePlus, HexaKit, PhenoKits, heliosApp, Civis, PolicyStack, thegent, portage, phenodocs, Pyron, phenoDesign`.

| Concern | Crate / file (lift-from path) | What it does |
|---|---|---|
| **Quality gate (aggregate)** | `crates/quality-gate/` | Aggregates `cargo fmt`/clippy/test/bench pass-fail into ONE gate. Replaces "30+ duplicated `scripts/quality-gate.sh`". |
| **Perf / SLA budget** | `crates/bench-guard/` | Benchmark regression detection + thresholds (from FocalPoint `tooling/bench-guard/`). |
| **Bloat / anti-pattern** | `crates/legacy-scan/` | Detects shell/Python anti-patterns + deprecated library usage per scripting policy. |
| **Acceptance / FR traceability** | `crates/fr-trace/` | FR-NNN → test traceability scanner (AgilePlus FR-* convention). Replaces `traceability-check.py` (+3 dupes). |
| **Acceptance / FR coverage** | `crates/fr-coverage/` | Functional-Requirement test-coverage analyzer (FocalPoint `tooling/fr-coverage/`). |
| **Policy / contracts** | `crates/policystack/` | Absorbed `KooshaPari/PolicyStack` (TS + Python policy-federation) — policy-as-code engine. |
| **Resilience SLA primitives** | `crates/phenotype-resilience/` | Shared rate limiter / circuit breaker / bulkhead for org services. |
| **Diff/patch** | `crates/phenotype-diff/` | Line-level unified diff + patch-apply (wraps `similar`). |
| **DAG / agent orchestration** | `crates/agent-orchestrator/` | Lane-based parallel-agent dispatcher; non-overlapping glob scopes; validates scope overlap; `orchestration.toml`. |
| **Worktree DAG/isolation** | `crates/worktree-manager/` (`wtm`) | Git worktree automation. |
| **Release/deploy** | `crates/release-cut/`, `crates/sbom-gen/` | Semver release + changelog; CycloneDX/SPDX SBOM. |
| **Commit/contract linkage** | `crates/commit-msg-check/` | Conventional commits + linkage rules. |
| **Docs health** | `crates/docs-health/`, `crates/doc-link-check/` | markdownlint/vale + broken-link scan. |
| **Forecast/usage** | `crates/agent-forecast/`, `crates/anthropic-usage-poll/` | Agent forecasting + Anthropic usage polling. |
| **Service registry / temporal** | `crates/phenotype-service-registry/`, `crates/temporal-grounding/` | Service registry; temporal grounding. |

Other reusable assets in the repo:
- `governance/ci-journey-gate.yml` — canonical wrapper that consuming repos copy; calls
  `uses: KooshaPari/phenotype-tooling/.github/workflows/reusable/journey-gate.yml@main`.
- `.github/workflows/reusable-*.yml` — `reusable-journey-gate.yml`, `reusable-cargo-deny.yml`,
  `reusable-trufflehog.yml` (the actual reusable workflow library).
- `.github/workflows/quality-gate.yml`, `fr-coverage.yml`, `scorecard.yml`, `codeql.yml`.
- `hooks/` (+ `bin/hook-entry`) — shared git/claude hook entrypoint.
- `scripts/adopt-tooling.sh` — the adoption shim for consumer repos.
- `Tools/Register-StartMenuApps.ps1` + `apps.json`, `renovate.json5`, `deny.toml`,
  `trufflehog.yml`, `cliff.toml`, `CODEOWNERS` (`* @KooshaPari`).

### Daemon-backed agent runner: `C:\Users\koosh\.claude\tools\agent-runner`
Single-binary Rust CLI wrapping `codex`. v0.2+ manages ONE persistent `codex app-server` WS
daemon (no per-dispatch process sprawl). Subcommands: `daemon start/stop/status`, `dispatch`,
`resume`. WS JSON-RPC flow `initialize → thread/start → turn/start → turn/completed`; auto-accepts
approval requests (sandbox-bypassed). Built exe present: `agent-runner.exe`; `jobs/`, `schema/`, `src/`.
This is the existing execution substrate for a DAG engine (the `/dispatch` skill is wired to it).

### Dino (this repo) — already a heavy consumer / pattern source
`.github/workflows/` (50 files) already contains many gate workflows worth generalizing UP:
`benchmark-regression-gate.yml`, `journey-quality-gates.yml`, `policy-gate.yml`, `proof-gate.yml`,
`pattern-gates.yml`, `schema-drift.yml`, `mutation-test.yml`, `scorecard.yml`, `sbom.yml`,
`framework-version.yml`, `unbounded-constraints.yml`, plus the whole Pattern-Catalog CI-script
gate family (#99–#235 in CLAUDE.md). Dino consumes shared infra via submodule
(`.gitmodules` → `tools/phenotype-journeys`).

---

## 2. PUSH-UP TARGETS (canonical org home + consumption mechanism)

- **Canonical org home for shared dev-infra: `KooshaPari/phenotype-tooling`.** It is purpose-built
  for this ("centralizes build verification, code-quality checks, documentation validation, release
  support, and SBOM into a single Rust workspace … adopted by other Phenotype repos without copying").
  Owner: `@KooshaPari` (CODEOWNERS). No `KooshaPari/.github` org-default repo and no `phenoShared`
  repo were found — phenotype-tooling IS the shared home.
- **Consumption mechanisms (use these to push up, don't reinvent):**
  1. **Reusable GitHub workflows** — `uses: KooshaPari/phenotype-tooling/.github/workflows/reusable/<x>.yml@main`,
     with a thin local wrapper (pattern shown in `governance/ci-journey-gate.yml`). This is the primary path.
  2. **Released crate binaries** — consumer repos replace local `scripts/quality-gate.sh` etc. with a
     one-line shim invoking the released crate binary (README migration plan step 3); `scripts/adopt-tooling.sh` automates it.
  3. **Git submodules** — for code/content that must be vendored in-tree (Dino ↔ `phenotype-journeys`).
- **Where the NEW tools should live:**
  - (a) **Daemon-backed agent-runner** — already lives at `C:\Users\koosh\.claude\tools\agent-runner`
    (user-global Claude tools). Keep it there as the execution daemon; it's already the `/dispatch`
    backend. If it needs org distribution, publish it as a crate under `phenotype-tooling/crates/`.
  - (b) **Acceptance-contract + DAG engine** — belongs in `phenotype-tooling/crates/` next to and
    composed with `agent-orchestrator` (lane/DAG config), `fr-trace`/`fr-coverage` (acceptance),
    `quality-gate`/`bench-guard`/`legacy-scan` (gates), driven by `agent-runner` as the executor.
    Do NOT build it inside Dino — Dino should consume it via reusable workflow + crate shim.

---

## 3. Recommendation: LIFT-FROM vs BUILD-NEW

| Capability | Verdict | Rationale / path |
|---|---|---|
| **Hard contracts / acceptance gates** | **LIFT** `fr-trace` + `fr-coverage` + `quality-gate` | FR-* → test traceability + coverage already implemented; wire as the hard-gate. |
| **Soft contracts / advisory** | **LIFT + thin extend** | Same crates run in non-blocking/scorecard mode (Dino's `journey-quality-gates.yml` is prior art for soft tiers). Add a severity/threshold flag rather than a new tool. |
| **Bloat / SOTA-semantic guards** | **LIFT** `legacy-scan`; **BUILD-NEW small** for SOTA-semantic | `legacy-scan` covers deprecated-lib/anti-pattern + size-policy bloat. Semantic "is-this-still-SOTA" guarding is NOT present anywhere — net-new, but build it as a `phenotype-tooling` crate, not in Dino. |
| **Perf / SLA budgets** | **LIFT** `bench-guard` (+ `phenotype-resilience` for runtime SLA) | Regression detection + thresholds done. Dino's `benchmark-regression-gate.yml` is the consumer pattern to generalize. |
| **DAG / deployment orchestration** | **LIFT + compose, BUILD-NEW thin DAG layer** | `agent-orchestrator` (lane scoping) + `agent-runner` daemon (execution) + `worktree-manager` (isolation) already exist. Missing piece is a true **dependency-DAG scheduler** over lanes (current orchestrator is parallel-lane, scope-validated, not dependency-ordered). Build that thin DAG layer as a new crate composing the three. |
| **Acceptance-contract engine (declarative spec → gates)** | **BUILD-NEW (in phenotype-tooling), compose existing** | No single declarative "contract → {hard gate, perf budget, bloat guard} → DAG dispatch" engine exists. Compose `fr-*`, `quality-gate`, `bench-guard`, `legacy-scan`, `policystack` behind one contract schema. Home: `phenotype-tooling/crates/`. |

### Bottom line
Almost nothing here is greenfield. The lift-from layer (`phenotype-tooling`) and the execution
substrate (`agent-runner`) both already exist with the right consumption mechanisms
(reusable workflows + crate shims + submodules). New work = (1) a declarative acceptance-contract
schema, (2) a dependency-DAG scheduler composing the existing orchestrator/runner/worktree crates,
and (3) a SOTA-semantic guard — all authored as `phenotype-tooling` crates and pushed up via
reusable workflows, NOT built inside Dino.
