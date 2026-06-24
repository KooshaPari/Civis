---
spec_id: civ-018
state: ACTIVE
plan_status: PLANNED
last_audit: 2026-06-10
parent_spec: civ-016
---

# Specification: Verify Harness Extension

**Slug**: civ-018-verify-harness-extension | **Epic**: E9 | **Date**: 2026-06-10 | **State**: ACTIVE
**Parent spec**: `civ-016-devx-verify-harness-and-worktree-hygiene` (E9.7–E9.9)

## Problem Statement

`civ-016` (E9.7–E9.9) flags three operational deltas that the umbrella
spec tracks but does not author:
1. A **read-only** PR-queue audit that flags PRs open > 14 days without rebase
   (`scripts/ci/audit-pr-queue.sh`)
2. A **read-only** worktree stale-branch audit that lists worktrees whose
   branch has no commits ahead of `origin/main` for > 30 days
   (`scripts/ci/audit-worktrees.sh`)
3. A shared `CARGO_TARGET_DIR=E:/civis-target` wrapper
   (`scripts/ci/with-cargo-target.sh`) so the rule "ONE cargo, always
   `E:/civis-target`" is enforced at the shell boundary

These three scripts close the spec gap that civ-016 leaves as "Planned"
in rows E9.7–E9.9. Without them, the harness is enforceable by hand
only. This spec is the auditable contract for the scripts themselves
(FR-CIV-VERIFY-008/009/010) and inherits the worktree-convention
contract from civ-016 (FR-CIV-VERIFY-007).

## Target Users

- Agent workers (L3 + dispatch) running preflight sweeps
- Maintainers auditing worktree sprawl and PR queue hygiene
- New contributors onboarding to the `civis-3d-*` verify surface
- CI / GitHub Actions authors wiring the scripts into verify gates

## Functional Requirements

- [ ] **FR-CIV-VERIFY-007** *(inherited from civ-016)*: Every new feature
  worktree SHALL be created at `E:/civis-wt-<SHORTNAME>` (or
  `C:/Users/koosh/Dev/civis-wt-<SHORTNAME>`) on a `feat/<topic>`,
  `fix/<topic>`, `chore/<topic>`, or `wt/<topic>` branch.
- [ ] **FR-CIV-VERIFY-008**: `scripts/ci/audit-pr-queue.sh` SHALL print a
  markdown table of PRs open > 14 days without rebase; SHALL NOT make
  network calls; SHALL be runnable as `bash scripts/ci/audit-pr-queue.sh`
  and produce stdout-only output.
- [ ] **FR-CIV-VERIFY-009**: `scripts/ci/audit-worktrees.sh` SHALL print
  a markdown table of worktrees whose branch has no commits ahead of
  `origin/main` for > 30 days; SHALL NOT prune or delete anything; SHALL
  be runnable as `bash scripts/ci/audit-worktrees.sh` with stdout-only
  output.
- [ ] **FR-CIV-VERIFY-010**: `scripts/ci/with-cargo-target.sh` SHALL
  export `CARGO_TARGET_DIR=E:/civis-target` (overridable via
  `CIVIS_CARGO_TARGET_DIR`); agents and CI SHALL source it before any
  `cargo` invocation; `cargo` invocations without the wrapper SHALL be
  flagged by `just civis-3d-verify` as a warning, not a hard error
  (forward-fix, not gate).

## Non-Functional Requirements

- The scripts SHALL be POSIX-shell compatible (no bash-isms on Windows;
  no pwsh-isms on Linux CI)
- The audit scripts SHALL be idempotent: re-running yields the same
  output as the first run
- `CARGO_TARGET_DIR` SHALL be a shared, fast-disk location
  (E:/civis-target on Windows; CI can override via env)
- The scripts SHALL never delete branches, worktrees, or repos
  (OPERATING RULES invariant)

## Constraints and Dependencies

- Depends on civ-016 (E9.6 worktree convention) — this spec is the
  auditable follow-on
- Depends on FR-CORE-001 (deterministic tick) for any verify variant
- The PR-queue audit reads `git log origin/main..origin/<branch>`;
  requires a fetched `origin/main`

## Acceptance Criteria

- [ ] `bash scripts/ci/audit-pr-queue.sh` exits 0 and produces a
  markdown table; an empty list is acceptable
- [ ] `bash scripts/ci/audit-worktrees.sh` exits 0 and produces a
  markdown table
- [ ] `bash scripts/ci/with-cargo-target.sh && echo "$CARGO_TARGET_DIR"`
  prints `E:/civis-target` (or `$CIVIS_CARGO_TARGET_DIR` if set)
- [ ] `just civis-3d-verify` exits 0 with the new scripts wired in

## Implementation Notes

- The scripts are intentionally **read-only**; the harness never deletes
  branches, worktrees, or repos
- `E:/civis-target` is the shared target dir; the wrapper MUST be
  sourced before any `cargo` call to avoid per-worktree recompilation
- The wrapper does NOT override `CARGO_TARGET_DIR` if the caller has
  already exported it (forward-fix, not gate)

## Status

| Story | Status |
|-------|--------|
| E9.7 PR-queue audit script | Planned (this spec) |
| E9.8 Worktree stale-branch audit script | Planned (this spec) |
| E9.9 Shared `CARGO_TARGET_DIR` wrapper | Planned (this spec) |
| E9.6 Worktree convention (inherited) | implemented (civ-016) |
