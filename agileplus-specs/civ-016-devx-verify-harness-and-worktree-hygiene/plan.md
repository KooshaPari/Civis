# Plan: Developer-Experience Verify Harness & Worktree Hygiene (civ-016)

## Phased WBS

### Phase 1: Audit scripts (E9.7, E9.8)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V1.1 | Author `scripts/ci/audit-pr-queue.sh` (POSIX shell, `gh api` not required; reads `origin/<branch>` via `git log`) | — | Planned |
| V1.2 | Author `scripts/ci/audit-worktrees.sh` (POSIX shell, reads `git worktree list` + `git log -1`) | — | Planned |
| V1.3 | Unit tests for both: empty repo, single fresh worktree, stale branch (mock via `--git-dir` env) | V1.1, V1.2 | Planned |

### Phase 2: Shared target-dir wrapper (E9.9)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V2.1 | Author `scripts/ci/with-cargo-target.sh` that exports `CARGO_TARGET_DIR=E:/civis-target` (overridable via `CIVIS_CARGO_TARGET_DIR`) | — | Planned |
| V2.2 | Update `docs/guides/agent-smoke.md` to call out the wrapper | V2.1 | Planned |

### Phase 3: Spec publication + AGENTS.md cross-link
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| V3.1 | Add row to `AGENTS.md` "Verify before you claim done" table pointing at this spec | V1.*, V2.* | Planned |
| V3.2 | Cross-link from `docs/development-guide/fr-ax-dx-ux-maturity-audit.md` | V3.1 | Planned |

## DAG Dependencies

```
V1.1 → V1.3
V1.2 → V1.3
V2.1 → V2.2
V1.3, V2.2 → V3.1 → V3.2
```
