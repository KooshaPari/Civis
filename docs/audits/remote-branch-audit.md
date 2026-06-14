# Remote Branch Audit

Generated: 2026-06-13
Main tip: `2276ed98` — `chore(cliff): upgrade to v2 template (BREAKING suffix) (#450)`

## Audit Method

For each `origin/*` branch except `main`:

1. **Commits-not-in-main**: `git log --oneline origin/main..origin/<branch>` — commits unique to the branch.
2. **Code-diff vs main**: `git diff --stat origin/main...origin/<branch>` — diff from merge-base to branch tip.
3. **Merged PR match**: `gh pr list --state merged` queried for a PR whose `headRefName` equals the branch name.

## Important Caveat

Many remote branches have **no merge base** with current `origin/main`. The repository appears to have been rebased or force-pushed at some point, so `git diff origin/main...origin/<branch>` fails with `fatal: no merge base` for the majority of branches. The `CommitsNotInMain` counts are therefore high (the branch history is completely divergent from current main), but the actual code changes may already be present in main via a merged PR.

Classification rule:
- **MERGED-RESIDUE** — a merged PR exists with this exact branch name (work is in main; branch is stale remote residue).
- **NEEDS-RECOVERY** — no merged PR found; the branch may contain unmerged work or may be an abandoned stale branch.

## MERGED-RESIDUE (9 branches)

| Branch | CommitsNotInMain | HasDiff | Merged PR | Merged At |
|--------|-----------------|---------|-----------|-----------|
| `chore/expand-codeowners` | 79 | false | #246 — chore(codeowners): expand placeholder to real ownership rules | 2026-04-24 |
| `chore/integrate-phenotype-docs` | 69 | false | #233 — chore: integrate @phenotype/docs | 2026-03-31 |
| `chore/regen-quality-manifest-2026-06-05` | 469 | false | #336 — chore(ci): regenerate quality-manifest for wave-1 HEAD | 2026-06-05 |
| `ci/add-release-workflow` | 83 | false | #250 — ci: bootstrap release workflow | 2026-04-24 |
| `ci/cargo-deny-scheduled-scan` | 90 | false | #258 — ci: add cargo-deny scheduled scan | 2026-04-27 |
| `docs/phantom-id-triage-3` | 2 | true | #386 — docs(audit): phantom-ID triage batch 3 (75 IDs) | 2026-06-11 |
| `feat/civis-3d-foundation` | 189 | false | #296 — feat: Civis 3D foundation — scaffold 11 new crates for WorldBox-class extension | 2026-05-25 |
| `feat/journey-impl` | 117 | false | #286 — docs: journey-traceability + iconography implementation | 2026-05-02 |
| `feature/civis-trufflehog` | 123 | false | #295 — chore: bootstrap trufflehog.yml | 2026-05-02 |

## NEEDS-RECOVERY (31 branches)

| Branch | CommitsNotInMain | HasDiff | Merged PR |
|--------|-----------------|---------|-----------|
| `chore/codeql-pin-actions-2026-04-27` | 88 | false | — |
| `chore/dependabot-frontend-2026-06-05` | 472 | false | — |
| `chore/parallel-session-sync` | 146 | false | — |
| `chore/tech-debt-sweep` | 147 | false | — |
| `chore/trufflehog-20260502` | 121 | false | — |
| `chore/trufflehog-pending` | 126 | false | — |
| `chore/worklog-seed-Civis` | 123 | false | — |
| `ci/local-first-manifest-verify` | 2 | true | — |
| `ci/pin-trufflehog` | 134 | false | — |
| `cursor/codeowners-infra-path-02ee` | 80 | false | — |
| `cursor/dev-parity-bevy-dependencies-9f32` | 476 | false | — |
| `cursor/ggshield-exit-code-handling-6368` | 227 | false | — |
| `cursor/ggshield-secret-detection-d180` | 244 | false | — |
| `cursor/multiple-application-issues-aa51` | 147 | false | — |
| `cursor/security-guard-exit-codes-fb54` | 218 | false | — |
| `cursor/security-guard-governance-gate-3c4a` | 242 | false | — |
| `cursor/security-manifest-bevy-bugs-585d` | 163 | false | — |
| `cursor/security-script-material-cache-9917` | 148 | false | — |
| `cursor/workflow-input-sanitization-81bc` | 84 | false | — |
| `docs/p-p1-kickoff` | 243 | false | — |
| `docs/sync-status-2026-05-28` | 241 | false | — |
| `feat/astar-obstacle-pathfinding` | 135 | false | — |
| `feat/frecon005-allocation` | 546 | false | — |
| `feat/p-l1-kickoff` | 254 | true | — |
| `feat/p-w1-bevy-gameplay-026` | 232 | false | — |
| `feat/p-w1-bevy-item-027` | 285 | true | — |
| `feat/process-compose` | 162 | false | — |
| `fix/clippy-warnings` | 217 | false | — |
| `fix/governance-gate-cache-bypass` | 488 | false | — |
| `fix/justfile-check` | 226 | false | — |
| `fix/lockfile-init` | 93 | false | — |

## Recommendations

1. **MERGED-RESIDUE branches** should be deleted from the remote to reduce clutter. Their work is already in main.
2. **NEEDS-RECOVERY branches** with `HasDiff = true` (`ci/local-first-manifest-verify`, `docs/phantom-id-triage-3` — already merged, `feat/p-l1-kickoff`, `feat/p-w1-bevy-item-027`) should be reviewed manually; they may have actual unmerged code changes despite the divergent history.
3. **NEEDS-RECOVERY branches with no merged PR and high commit counts** may represent abandoned work or branches that were superseded by other PRs. Consider opening a tracking issue or reviewing with the authors before deleting.
4. **No merge base anomaly**: A repository-wide history rewrite appears to have occurred. The commit counts above reflect complete history divergence rather than typical ahead/behind metrics. If a full history reconciliation is needed, consider a `--allow-unrelated-histories` merge or rebase of the stale branches onto current main.
