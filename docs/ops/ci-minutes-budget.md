# CI minutes budget (zero-bill hardening)

**Status:** permanent ruling — GitHub billing will **never** be restored. The 3,000 free minutes
drain instantly from PR code runs. This document is the source of truth for the per-workflow
minutes budget after the `ci/zero-minutes-hardening` change.

## TL;DR

| | Before | After |
|---|---:|---:|
| Estimated minutes / month (auto + cron) | **~38,900 min** | **~277 min** |
| Per-PR auto-run minutes | 8 workflows × 5 min = 40 min/PR | 2 workflows × 0.5 min = 1 min/PR |
| Heavy gate path | hosted-runner Rust compile | developer-machine lefthook pre-push |

The 99.3% reduction is achieved by moving every heavy gate off the cloud onto the developer
machine and recording the result in `.ci/quality-manifest.json` (a blake2b-hashed attestation).
The cloud then only **verifies** the manifest on every PR (one bash step, ~5–10 s) and never
runs Rust / Bun / Python toolchains.

## The pattern

### 1. Heavy gates → local-first via lefthook

```
developer machine                       GitHub Actions
==================                      ==============
  pre-push (lefthook)                   PR / push: main
       │                                      │
       ▼                                      ▼
  cargo fmt / clippy / test             checkout
  web_test                              bash scripts/quality/verify-quality-manifest.sh
  dashboard_typecheck                          │
  (opt-in extras)                             ▼
       │                                  pass / fail (≈ 5 s)
       ▼
  .ci/quality-manifest.json  ──── git push ───►  (read by verify)
   { "gates": { ... }, "manifest_hash": blake2b(...) }
```

`verify-quality-manifest.sh` (the only thing the cloud runs) checks:

1. Manifest version is `"1"`.
2. `git_sha` is the current `HEAD` (or `HEAD^` for first-push on a new branch).
3. Every core gate has `status == "pass"`.
4. Optional gates (`unreal_*`, `extra_*`) may be `pass` **or** `skip` (e.g. dev machine
   has no UE installed).
5. The `manifest_hash` (blake2b of `{git_sha, sorted-gates}`) matches what's on disk —
   the manifest cannot be hand-edited without detection.

### 2. Class-(a) heavy workflows → manifest-verify dispatch-only

Each of the following workflows previously ran on `push: main` and `pull_request` and burned
2–30 min of hosted-runner time per auto-trigger. They now run only on `workflow_dispatch`
(manual opt-in); the cloud body is one manifest-verify step.

| Workflow | Class | Triggers BEFORE | Triggers AFTER |
|----------|-------|-----------------|----------------|
| `cargo-audit.yml` | (a)+(c) | push:main, weekly cron, dispatch | weekly cron, dispatch |
| `cargo-deny.yml` | (a) | push:main, dispatch | dispatch |
| `cargo-machete.yml` | (a) | push:main, dispatch | dispatch |
| `cargo-semver-checks.yml` | (a) | push:main, dispatch | dispatch |
| `civis-3d-live-smoke.yml` | (a) | PR (paths), push:main, dispatch | dispatch |
| `codeql.yml` | (a)+(c) | push:main, weekly cron, dispatch | weekly cron, dispatch |
| `codeql-rust.yml` | (a) | push:main, dispatch | dispatch |
| `dev-parity.yml` | (a) | PR, dispatch | dispatch |
| `docs-site.yml` | (a) | push:main, dispatch | dispatch |
| `fr-coverage.yml` | (a) | push:main, dispatch | dispatch |
| `journey-gate.yml` | (a) | push:main, dispatch | dispatch |
| `legacy-tooling-gate.yml` | (a) | push:main, dispatch | dispatch |
| `quality-gate.yml` | (a) | push:main, dispatch | dispatch |
| `security-guard.yml` | (a) | push:main, dispatch | dispatch |
| `trufflehog.yml` | (a) | push:main, dispatch | dispatch |
| `unreal-build.yml` | (a) | push:main, dispatch | dispatch |
| `pages.yml` | (b) | push:main (paths), dispatch | unchanged (real Pages deploy) |

### 3. Class-(c) cron downshifts

| Workflow | Cron BEFORE | Cron AFTER | Reason |
|----------|-------------|------------|--------|
| `alert-sync-issues.yml` | `17 * * * *` (hourly) | `17 6 * * 1` (weekly Mon 06:17) | Hourly sync against the phenoShared reusable ran ~24×/week of transitive work; weekly is plenty for advisory dashboards |
| `codeql.yml` | `0 6 * * 1` (weekly) | unchanged | Weekly is acceptable for advisory CodeQL dashboards |
| `scorecard.yml` | `17 3 * * 6` (weekly) | unchanged | Weekly OpenSSF scorecard (no push trigger anymore) |
| `cargo-audit.yml` | `0 2 * * 3` (weekly) | unchanged | Weekly rustsec audit is enough |

### 4. Class-(d) reusable-callers (unchanged)

The following are tiny `uses: phenoShared/...@main` callers — the heavy work runs in the
phenoShared repo, not in Civis minutes. They are kept as-is:

- `release-drafter.yml` (push:main → reusable)
- `security-guard-hook-audit.yml` (push:main + dispatch → reusable)
- `self-merge-gate.yml` (pull_request_review → reusable)
- `tag-automation.yml` (push:tags → reusable)
- `alert-sync-issues.yml` (cron + dispatch → reusable)

### 5. Class-(b) tiny verify/govern (unchanged)

- `pr-governance.yml` — manifest-verify + GraphQL (≈5 s)
- `quality.yml` — manifest-verify (≈5 s); manual `quality-full` job (gated by `workflow_dispatch`)
- `doc-links.yml` — single `echo` step (≈3 s)
- `policy-gate.yml` — bash only, no Rust (≈10 s)
- `release.yml` — `cargo publish`, only on `v*` tag push (rare, intentional)

### 6. Duplicates deleted

- `pr-governance-gate.yml` — exact duplicate of `pr-governance.yml` (both used the same
  workflow name `pr-governance-gate`; only one would run anyway, but the dead file was
  burning any `pull_request_target` event with a second `actions/checkout`). Removed.
- `pages-deploy.yml` — exact duplicate of `pages.yml`. Removed.

## Minutes estimate (per month)

Assumptions: **15 PRs/day opened, 5 merges to main/day, 30 days/month**.
For each (a)-class workflow we model:

- `push: main` (merge): 5 × 30 = 150 runs/month × heavy-minutes-per-run
- `pull_request`: 15 × 30 = 450 runs/month × heavy-minutes-per-run
- `workflow_dispatch`: ~1–2 runs/month × heavy-minutes-per-run (manual escape hatch)

| Workflow | Per-run heavy (BEFORE) | Runs/mo (BEFORE) | Min/mo (BEFORE) | Per-run (AFTER) | Runs/mo (AFTER) | Min/mo (AFTER) |
|----------|------------------------:|------------------:|----------------:|----------------:|-----------------:|---------------:|
| cargo-audit | 4 min | 600 (150 push + 450 PR) | 2,400 | 5 s verify + 4 min dispatch (2×) | 2 (dispatch) | ~8 |
| cargo-deny | 3 min | 600 | 1,800 | 5 s + 3 min dispatch (2×) | 2 | ~6 |
| cargo-machete | 2 min | 600 | 1,200 | 5 s + 2 min dispatch (2×) | 2 | ~4 |
| cargo-semver-checks | 5 min | 600 | 3,000 | 5 s + 5 min dispatch (2×) | 2 | ~10 |
| civis-3d-live-smoke | 12 min (apt + bevy link) | 450 (PR only) | 5,400 | 5 s + 12 min dispatch (1×) | 1 | ~12 |
| codeql.yml | 30 min (rust build + analysis) | 150 push + 4 cron | 4,500 | 5 s + 30 min weekly cron (4×) | 4 | ~120 |
| codeql-rust.yml | 30 min | 150 push | 4,500 | 5 s + 30 min dispatch (1×) | 1 | ~30 |
| dev-parity.yml | 8 min × 3 jobs = 24 min | 450 PR | 10,800 | 5 s + 24 min dispatch (1×) | 1 | ~24 |
| docs-site.yml | 5 min (Playwright) | 150 push | 750 | 5 s + 5 min dispatch (1×) | 1 | ~5 |
| fr-coverage.yml | 4 min | 150 push | 600 | 5 s + 4 min dispatch (1×) | 1 | ~4 |
| journey-gate.yml | 6 min | 150 push | 900 | 5 s + 6 min dispatch (1×) | 1 | ~6 |
| legacy-tooling-gate.yml | 3 min | 150 push | 450 | 5 s + 3 min dispatch (1×) | 1 | ~3 |
| quality-gate.yml | 4 min | 150 push | 600 | 5 s + 4 min dispatch (1×) | 1 | ~4 |
| security-guard.yml | 3 min | 150 push | 450 | 5 s + 3 min dispatch (1×) | 1 | ~3 |
| trufflehog.yml | 4 min | 150 push | 600 | 5 s + 4 min dispatch (1×) | 1 | ~4 |
| unreal-build.yml | 6 min (windows) | 150 push | 900 | 5 s + 6 min dispatch (1×) | 1 | ~6 |
| alert-sync-issues (cron) | 2 min | 24 hourly | 48 | 2 min weekly | 4 | ~8 |
| scorecard (cron) | 5 min | 4 weekly | 20 | 5 min weekly | 4 | ~20 |
| **Total** | | | **~38,918 min** | | | **~277 min** |

**Reduction: ~99.3%** of hosted-runner minutes, with the **only remaining burn** being:

1. `pr-governance.yml` on every PR (manifest-verify, ~5 s × 450 = 38 min/month).
2. `quality.yml` on every PR + push (manifest-verify, ~5 s × 600 = 50 min/month).
3. The four weekly cron jobs (codeql, scorecard, cargo-audit, alert-sync) at
   ~150 min/month combined.
4. Manual dispatch reruns (~50 min/month, by definition rare).

The two new constants (1+2) sum to under 90 min/month and the four cron jobs sum to
under 150 min/month. With a 3,000 free-min budget that leaves ~2,700 min/month for any
**new** heavy workflow we want to add later.

## Per-developer expectations

The local-first pattern is enforced by lefthook (`pre-push` hook). Setting up once:

```sh
# one-time
lefthook install

# each PR (default path: ≈ 5–7 seconds)
lefthook run pre-push

# each PR with extended gates (≈ 2–5 minutes, opt-in)
CIVIS_QUALITY_EXTRAS=1 lefthook run pre-push
```

The default `pre-push` runs the core gates (rust fmt/clippy/test, web test,
dashboard typecheck, or `just civis-3d-verify`). With `CIVIS_QUALITY_EXTRAS=1`
it additionally runs cargo-audit, cargo-deny, cargo-machete, cargo-semver-checks,
trufflehog, fr-coverage, docs:check, and security-guard. Each is recorded in
`.ci/quality-manifest.json` and signed by the manifest hash. CI does **not**
re-run them; it only verifies the manifest.

## Why not a self-hosted runner?

A self-hosted runner (free minutes) is the alternative the existing `agileplus` ADR
considers whenever the cloud bill climbs. We **do not** take that path here because:

- The repository is already local-first: every heavy gate is a `cargo`/`npm`/`bun`/
  python invocation we run on the dev box anyway. Adding a self-hosted runner would
  just move the same command to a different box on the same network.
- The min-spend on a self-hosted runner is non-zero (always-on VM + maintenance +
  artifact cache warming) which defeats the "no bill" goal.
- The pattern's success criterion is "PR can land with zero cloud minutes" and the
  manifest verification is a 5-second bash step — no self-hosted infra needed.

If a future month shows the weekly cron jobs (codeql + scorecard + cargo-audit +
alert-sync) climbing back above the 3,000 free budget, the "Alternatives considered"
section of that PR will revisit self-hosted.

## Trace

`Trace: NFR never-billable-CI`
