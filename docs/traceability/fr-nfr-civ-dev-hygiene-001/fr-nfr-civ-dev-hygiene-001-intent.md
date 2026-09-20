# Intent: FR-NFR-CIV-DEV-HYGIENE-001 — History-purge plan (build artifacts)

> Date: 2026-09-20
> FR: FR-NFR-CIV-DEV-HYGIENE-001
> Epic: FR-NFR-CIV-DEV-HYGIENE

## What This FR Captures

The history-purge plan documented at `docs/ops/history-purge-plan.md`
— a non-functional / hygiene requirement that captures the agreed
procedure for removing the `target-check-*`, `target-ci`, and
`.target-*` build-artifact trees from the git history using
`git-filter-repo`. PR #364 only untracked those paths; the blobs
remain in history until this plan executes.

## User Intent

Without a history purge, fresh-worktree checkouts materialize
~1.5 GB of build artifacts per checkout, leading to disk-full
incidents and over-long path failures. The plan is the agreed
mitigation, gated behind a freeze-and-mirror-backup step to
preserve recoverability.

## Acceptance Signal

- `docs/ops/history-purge-plan.md` exists and is approved.
- Post-purge, `git rev-list --objects --all | grep -cE
  'target-check|target-ci'` returns `0`.
- Fresh clones no longer materialize the artifact trees.

## Implementing Code

- `docs/ops/history-purge-plan.md:4` — `Trace:
  NFR-CIV-DEV-HYGIENE-001` line tying the plan to this FR.

## Test Coverage

> Procedural, not testable in code — validated by the post-purge
> grep above.

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-civ-dev-hygiene-001-intent.md` |
| Plan | `docs/ops/history-purge-plan.md` |