# Plan: Verify Harness Extension (civ-018)

Refines `civ-016` (E9.7–E9.9) — adds the auditable scripts that close
the spec gap.

## Phased WBS

### Phase 1: PR-queue audit (E9.7)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V1.1 | Author `scripts/ci/audit-pr-queue.sh` (POSIX shell, no `gh api`) | — | Planned |
| V1.2 | Test cases: empty repo, fresh open PR, 14-day-stale PR | V1.1 | Planned |
| V1.3 | Wire into `just civis-3d-verify` as a non-blocking check | V1.2 | Planned |

### Phase 2: Worktree stale-branch audit (E9.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V2.1 | Author `scripts/ci/audit-worktrees.sh` (POSIX shell, reads `git worktree list`) | — | Planned |
| V2.2 | Test cases: empty, single fresh worktree, 30-day-stale branch | V2.1 | Planned |
| V2.3 | Output is markdown table; never auto-prunes | V2.2 | Planned |

### Phase 3: Shared cargo target wrapper (E9.9)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V3.1 | Author `scripts/ci/with-cargo-target.sh` exporting `CARGO_TARGET_DIR=E:/civis-target` | — | Planned |
| V3.2 | Source wrapper from `just civis-3d-verify` and `scripts/agent-smoke.ps1` | V3.1 | Planned |
| V3.3 | Document in `docs/guides/agent-smoke.md` | V3.2 | Planned |

## DAG Dependencies

```
V1.1 → V1.2 → V1.3
V2.1 → V2.2 → V2.3
V3.1 → V3.2 → V3.3
V1.3, V2.3, V3.3 → civ-016 (umbrella spec acceptance)
```
