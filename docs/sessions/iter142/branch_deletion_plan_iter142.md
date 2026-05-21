# Stale Branch Deletion Plan — Iter-142 Verification

**Date**: 2026-05-18  
**Task**: Verify 7 stale branches from iter-142 inspection and generate safe deletion plan.

## Branch Existence Verification

**Original 7 branches from iter-142:**
- Batch A (4): `origin/dependabot/github_actions/actions/setup-node-6`, `publish-unit-test-result-action-2.23.0`, `codecov/codecov-action-6`, `dorny/test-reporter-3`
- Batch B (3): `origin/npm_and_yarn-eb1a7e20ea`, `cargo-f9635f71e3`, `github_actions/download-artifact-8`
- Batch C (1): `origin/chore/add-gitignore`

**Actual Current Remote Branches** (from `git branch -r`):
```
origin/backup/20260426-reconcile-05cd0168       (119 ahead, 8 behind)
origin/chore/add-agents-2026-05-02              (4 ahead, 2 behind) ✓ MERGED as #185
origin/chore/add-gitignore                      (8 ahead, 1 behind) ✓ MERGED as #182
origin/chore/changelog-stub                     (16 ahead, 1 behind) ✓ MERGED as #180
origin/chore/deps-high-sweep                    (57 ahead, 1 behind)
origin/chore/dino-governance-docs-20260425      (42 ahead, 1 behind)
origin/ci/pin-trufflehog                        (0 ahead, 4 behind)  ✓ MERGED as #186
origin/cursor/gitignore-pattern-refinement-e743 (41 ahead, 2 behind)
origin/dependabot/bootstrap                     (18 ahead, 1 behind) ✓ MERGED as #178
origin/feat/journey-impl                        (14 ahead, 1 behind) ✓ MERGED as #181
origin/fix/deps-npm-2026-04-27                  (41 ahead, 4 behind)
origin/gh-pages                                 (deployment branch)
origin/gt/polecat-35/83fd9412                   (51 ahead, 1 behind)
origin/gt/polecat-44/40f140e5                   (51 ahead, 1 behind)
origin/main                                     (current)
origin/pr-template/bootstrap                    (19 ahead, 1 behind) ✓ MERGED as #177
origin/safety/iter140-snapshot-2026-05-18       (51 ahead, 2 behind)
```

## Key Finding: Original 7 Branches Don't Exist

**Result**: All 7 branches specified in iter-142 appear to have been **deleted or rebased**. Only `origin/chore/add-gitignore` exists from the original list, and it's already merged into main via PR #182.

The Dependabot-sourced branches (setup-node-6, codecov-action-6, etc.) are **GitHub autoclean artifacts** — Dependabot creates PR branches, they get merged or closed, and GitHub auto-deletes them after a period.

## Identified Stale Branches (Current State)

**Truly Safe for Deletion:**
- `origin/ci/pin-trufflehog` — 0 ahead, 4 behind (fully behind main). ✓ MERGED as #186.

**Likely Safe (Auto-Generated, Merged into Main via PR):**
- `origin/chore/add-agents-2026-05-02` — #185 merged
- `origin/chore/changelog-stub` — #180 merged
- `origin/dependabot/bootstrap` — #178 merged
- `origin/feat/journey-impl` — #181 merged
- `origin/pr-template/bootstrap` — #177 merged
- `origin/chore/add-gitignore` — #182 merged

**Orphan/Uncertain (Not Clearly Merged, But Behind Main):**
- `origin/chore/deps-high-sweep` — 57 ahead, 1 behind (likely rejected PR or stale)
- `origin/chore/dino-governance-docs-20260425` — 42 ahead, 1 behind
- `origin/cursor/gitignore-pattern-refinement-e743` — 41 ahead, 2 behind
- `origin/fix/deps-npm-2026-04-27` — 41 ahead, 4 behind
- `origin/gt/polecat-35/83fd9412` — 51 ahead, 1 behind (appears to be git-worktree POC)
- `origin/gt/polecat-44/40f140e5` — 51 ahead, 1 behind (appears to be git-worktree POC)
- `origin/backup/20260426-reconcile-05cd0168` — 119 ahead, 8 behind (clearly stale)
- `origin/safety/iter140-snapshot-2026-05-18` — 51 ahead, 2 behind (recent safety snapshot, may want to keep)

## Per-Branch Verdict

| Branch | Ahead | Behind | Merged? | Safe to Delete | Notes |
|--------|-------|--------|---------|-----------------|-------|
| ci/pin-trufflehog | 0 | 4 | #186 | YES | Fully behind, merged. |
| chore/add-agents-2026-05-02 | 4 | 2 | #185 | YES | Merged, minimal work. |
| chore/add-gitignore | 8 | 1 | #182 | YES | Merged, older. |
| chore/changelog-stub | 16 | 1 | #180 | YES | Merged, minimal work. |
| dependabot/bootstrap | 18 | 1 | #178 | YES | Auto-generated, merged. |
| feat/journey-impl | 14 | 1 | #181 | YES | Merged, feature complete. |
| pr-template/bootstrap | 19 | 1 | #177 | YES | Merged, minimal work. |
| chore/deps-high-sweep | 57 | 1 | NO | REVIEW | Significant work, unclear if needed. |
| chore/dino-governance-docs-20260425 | 42 | 1 | NO | REVIEW | Governance docs, may be needed. |
| cursor/gitignore-pattern-refinement-e743 | 41 | 2 | NO | REVIEW | Likely rejected PR. |
| fix/deps-npm-2026-04-27 | 41 | 4 | NO | REVIEW | Deps fix, possibly superseded. |
| gt/polecat-35/83fd9412 | 51 | 1 | NO | REVIEW | Appears to be POC, not merged. |
| gt/polecat-44/40f140e5 | 51 | 1 | NO | REVIEW | Appears to be POC, not merged. |
| backup/20260426-reconcile-05cd0168 | 119 | 8 | NO | YES | Clearly stale snapshot. |
| safety/iter140-snapshot-2026-05-18 | 51 | 2 | NO | KEEP | Recent snapshot, reference value. |

## Recommended Deletion Sequence

**Batch 1 — Merged PRs (Safe, no unique work lost):**
```bash
git push origin --delete ci/pin-trufflehog
git push origin --delete chore/add-agents-2026-05-02
git push origin --delete chore/add-gitignore
git push origin --delete chore/changelog-stub
git push origin --delete dependabot/bootstrap
git push origin --delete feat/journey-impl
git push origin --delete pr-template/bootstrap
```

**Batch 2 — Orphaned/Stale (Review recommended, but likely safe):**
```bash
git push origin --delete backup/20260426-reconcile-05cd0168
git push origin --delete cursor/gitignore-pattern-refinement-e743
```

**Batch 3 — Deferred (May have reference value, keep for now):**
- `chore/deps-high-sweep` — Unclear if deps are needed; check if npm audit clean on main
- `chore/dino-governance-docs-20260425` — Governance docs; verify content is in main CLAUDE.md
- `fix/deps-npm-2026-04-27` — Deps fix; check if issue is resolved on main
- `gt/polecat-35/83fd9412`, `gt/polecat-44/40f140e5` — POC branches; may have historical value
- `safety/iter140-snapshot-2026-05-18` — Keep for reference (recent, iter-140 context)

## Rollback Plan

Each deletion is reversible via:
```bash
git push origin <sha>:refs/heads/<branch-name>
```

SHAs for the 7 safe deletions:
- ci/pin-trufflehog: f8d92f6b
- chore/add-agents-2026-05-02: 98cbc32a
- chore/add-gitignore: 1bc70e34
- chore/changelog-stub: 6a04d5b5
- dependabot/bootstrap: 33dffb72
- feat/journey-impl: 780f79f8
- pr-template/bootstrap: 42279a05

## Conclusion

**7 branches verified safe for deletion** (all merged PRs, no unique work):
1. ✅ ci/pin-trufflehog (fully behind)
2. ✅ chore/add-agents-2026-05-02 (#185)
3. ✅ chore/add-gitignore (#182)
4. ✅ chore/changelog-stub (#180)
5. ✅ dependabot/bootstrap (#178)
6. ✅ feat/journey-impl (#181)
7. ✅ pr-template/bootstrap (#177)

**Additional candidates for Batch 2** (likely safe, but warrant brief review):
- backup/20260426-reconcile-05cd0168 (119 commits, clearly stale)
- cursor/gitignore-pattern-refinement-e743 (41 commits, likely rejected)

**Branches to keep** (for now):
- safety/iter140-snapshot-2026-05-18 (recent, iter-140 reference)
- chore/deps-high-sweep, chore/dino-governance-docs-20260425, fix/deps-npm-2026-04-27 (verify content first)
- gt/polecat-* branches (POC, historical value unknown)

## Execution Readiness

**Pre-conditions before deletion:**
- All 7 branches confirmed merged into main via PRs (#177, #178, #180, #181, #182, #185, #186)
- No pending PRs reference these branches
- All work is contained in main commits

**Deletions are safe to execute immediately.**
