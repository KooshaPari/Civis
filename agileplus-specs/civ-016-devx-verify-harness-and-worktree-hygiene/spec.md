---
spec_id: civ-016
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-09
---

# Specification: Developer-Experience Verify Harness & Worktree Hygiene

**Slug**: civ-016-devx-verify-harness-and-worktree-hygiene | **Epic**: E9 | **Date**: 2026-06-09 | **State**: ACTIVE

## Problem Statement

Civis has accumulated multiple parallel developer-experience surfaces that
have no single spec home: `scripts/agent-smoke.ps1` (AX-04, fast + FullUnreal),
`just civis-3d-verify` (catalog + scenario + web + mod-host + check/clippy/
fmt), `just civis-3d-catalog-check` (JSON-RPC drift), `just civis-3d-scenario-
check` (YAML schema), `just civis-3d-mod-{wasm,package,sign}`, and the
worktree convention `git worktree add E:/civis-wt-SHORTNAME BRANCH` (or
`C:/Users/koosh/Dev/civis-wt-SHORTNAME`). These are operational gates that
already exist; this spec is the **contract** that ties them to specific FRs
so that future contributors can extend the harness without regressing the
gates, and so that worktree naming/queue hygiene is auditable.

## Target Users

- Agent workers (L3 + dispatch) running preflight and full sweeps
- PR reviewers evaluating whether a PR meets the verify gate
- Maintainers auditing worktree sprawl and PR queue hygiene
- New contributors onboarding to the `civis-3d-*` verify surface

## Functional Requirements

- [ ] **FR-CIV-VERIFY-001**: `scripts/agent-smoke.ps1` (default, no flags) SHALL
  exit 0 on a clean `origin/main` checkout, with the offline Unreal preflight
  enabled. Required: `ws_smoke`, catalog/scenario/mod-host/godot gates,
  civ-watch boot, Unreal preflight.
- [ ] **FR-CIV-VERIFY-002**: `scripts/agent-smoke.ps1 -FullUnreal` SHALL pass when
  UE 5.7 + VS 2026 are present (full `clients/unreal-show/scripts/build.ps1`).
  The default invocation SHALL be the offline preflight only.
- [ ] **FR-CIV-VERIFY-003**: `just civis-3d-verify` SHALL be the single entry point
  for catalog + scenario + web + mod-host + `cargo check/test/clippy --all-
  targets -- -D warnings` + `cargo fmt --check` and SHALL exit 0 on `main`.
- [ ] **FR-CIV-VERIFY-004**: `just civis-3d-catalog-check` SHALL fail CI on any
  drift between `crates/server/src/jsonrpc.rs` method surface and
  `docs/api/jsonrpc-surface.md`. 14 methods are tracked today.
- [ ] **FR-CIV-VERIFY-005**: `just civis-3d-scenario-check` SHALL pass for
  `scenarios/baseline.yaml` (run on Windows with `-j 1` per
  `docs/guides/agent-smoke.md`).
- [ ] **FR-CIV-VERIFY-006**: `just civis-3d-mod-{wasm,package,sign}` SHALL
  round-trip an example mod through `wasm32-unknown-unknown` build,
  `example-policy.civmod` and `example-economic.civmod` packaging, and
  Ed25519 signing, printing `author_pubkey_hex` for downstream verification.
- [ ] **FR-CIV-VERIFY-007**: Every new feature worktree SHALL be created with
  the path `E:/civis-wt-<SHORTNAME>` (or the
  `C:/Users/koosh/Dev/civis-wt-<SHORTNAME>` equivalent on the local
  dev box) and SHALL use a branch name `feat/<topic>`, `fix/<topic>`,
  `chore/<topic>`, or `wt/<topic>`; canonical `C:/Users/koosh/Dev/Civis`
  SHALL stay on `main`. No git stash; no branch checkout inside a shared
  working tree.
- [ ] **FR-CIV-VERIFY-008**: A PR that is open against `main` for more than 14
  days without rebase SHALL be flagged in the PR queue audit; the audit
  SHALL be runnable as `bash scripts/ci/audit-pr-queue.sh` and SHALL
  output a markdown table to stdout (no network calls required).
- [ ] **FR-CIV-VERIFY-009**: A worktree whose underlying branch has no
  commits ahead of `origin/main` for more than 30 days SHALL be
  considered stale and SHALL be listed (not auto-pruned) by
  `bash scripts/ci/audit-worktrees.sh`.
- [ ] **FR-CIV-VERIFY-010**: Cargo builds SHALL run at most one cargo
  process at a time, always with `CARGO_TARGET_DIR=E:/civis-target`,
  and SHALL be skipped entirely when the task does not require a build.
  `scripts/ci/with-cargo-target.sh` SHALL export the env var; CI agents
  SHALL source it before `cargo` invocations.

## Non-Functional Requirements

- The harness scripts SHALL be POSIX-shell or PowerShell-Core compatible
  (no bash-isms on the Windows path; no pwsh-isms on the Linux CI path)
- The harness SHALL be idempotent: re-running on a clean tree yields the
  same exit code as the first run
- `CARGO_TARGET_DIR` SHALL be a shared, fast-disk location
  (E:/civis-target on Windows)
- Worktree naming SHALL NOT collide with `target-check-*` debug dirs
  (already present in the repo root — see `AGENTS.md` and `CLAUDE.md`)

## Constraints and Dependencies

- Depends on FR-CORE-001 (deterministic tick) for any `ws_smoke` variant
- Depends on FR-PROTO-001/002 (WS + JSON-RPC) for `sim.snapshot` smoke
- Depends on FR-CIV-TACTICS-042 (fog-of-war) for the unified smoke
- The worktree rule is a *convention*, not a hook; CI does not enforce it,
  but the audit script reports drift

## Acceptance Criteria

- [ ] `.\scripts\agent-smoke.ps1` exits 0 on `main`
- [ ] `just civis-3d-verify` exits 0 on `main`
- [ ] `just civis-3d-catalog-check` exits 0 (14 methods, no drift)
- [ ] `just civis-3d-scenario-check` exits 0 with `scenarios/baseline.yaml`
- [ ] `just civis-3d-mod-wasm`, `civis-3d-mod-package`, `civis-3d-mod-sign`
  succeed for `mods/example-policy` and `mods/example-economic`
- [ ] `bash scripts/ci/audit-pr-queue.sh` produces a markdown table; an
  empty list is acceptable
- [ ] `bash scripts/ci/audit-worktrees.sh` produces a markdown table
- [ ] `scripts/ci/with-cargo-target.sh` exports `CARGO_TARGET_DIR`

## Implementation Notes

- The worktree convention is already documented in `AGENTS.md`,
  `CLAUDE.md`, and this repo's `CLAUDE.md`. This spec makes it
  auditable via the `audit-worktrees.sh` script.
- The PR queue audit and worktree audit are intentionally **read-only**;
  the harness never deletes branches, worktrees, or repos. (OPERATING
  RULES: "Never delete repos, worktrees, or branches with unmerged work.")
- `E:/civis-target` is the shared target dir; the harness MUST source
  `scripts/ci/with-cargo-target.sh` before any `cargo` call (to avoid
  re-compilation per-worktree).

## Status

| Story | Status |
|-------|--------|
| E9.1 `agent-smoke.ps1` default + FullUnreal | implemented |
| E9.2 `civis-3d-verify` (catalog + scenario + web + mod-host + check/clippy/fmt) | implemented |
| E9.3 `civis-3d-catalog-check` JSON-RPC drift | implemented |
| E9.4 `civis-3d-scenario-check` baseline.yaml | implemented |
| E9.5 `civis-3d-mod-wasm` / `mod-package` / `mod-sign` | implemented |
| E9.6 Worktree convention (`<disk>:/civis-wt-<SHORT>`) | implemented (documented) |
| E9.7 PR queue audit script | Planned (this PR) |
| E9.8 Worktree stale-branch audit script | Planned (this PR) |
| E9.9 Shared `CARGO_TARGET_DIR` wrapper | Planned (this PR) |
