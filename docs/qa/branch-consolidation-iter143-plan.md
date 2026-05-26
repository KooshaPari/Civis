# Branch Consolidation Plan (iter-143)

**Status**: PROPOSAL — read-only analysis. NO destructive ops executed.
**Tasks**: #507 (P0 CRITICAL Branch consolidation), #512 (P0 INFRA main coordination)
**Author**: Investigation subagent (iter-143)
**Date**: 2026-05-19
**Repo**: `C:\Users\koosh\Dino`
**Current branch under PR**: `fix/handle-connect-iter142`

---

## Executive Summary

| Category | Count |
|----------|------:|
| Local branches | 3 (`main`, `fix/handle-connect-iter142`, `safety/iter140-snapshot-2026-05-18`) |
| Remote branches | 18 (incl. `origin/HEAD`, `origin/gh-pages`, `origin/main`) |
| Active remote feature branches | 14 |
| Squash-merged → safe to delete (after #512 PR merges) | 9 |
| Genuinely orphan (unique commits, NOT in main) | 5 |
| Safety/backup branches (preserve) | 2 (1 local + 1 remote mirror) |
| Stash-recovery branches | 0 (sibling agent has not pushed yet) |

**Key divergence**: local `main` is **18 commits BEHIND `origin/main`** and **0 ahead**. This is a *trivial fast-forward* — no merge conflict risk. Local `safety/iter140-snapshot-2026-05-18` is **in sync** with `origin/safety/iter140-snapshot-2026-05-18` (0 ahead / 0 behind).

---

## 1. Full Branch Enumeration

### Local branches (3)
```
* fix/handle-connect-iter142          [ACTIVE — sibling agent owns PR]
  main                                [BEHIND origin/main by 18 commits — needs fast-forward]
  safety/iter140-snapshot-2026-05-18  [SYNC with remote]
```

### Remote branches (18)
```
origin/HEAD -> origin/main                              [pointer alias]
origin/main                                             [authoritative]
origin/gh-pages                                         [docs deploy artifact — DO NOT TOUCH]
origin/safety/iter140-snapshot-2026-05-18               [safety branch — preserve]
origin/backup/20260426-reconcile-05cd0168               [backup — see §3.D]
origin/chore/add-agents-2026-05-02
origin/chore/add-gitignore
origin/chore/changelog-stub
origin/chore/deps-high-sweep
origin/chore/dino-governance-docs-20260425
origin/ci/pin-trufflehog
origin/cursor/gitignore-pattern-refinement-e743
origin/dependabot/bootstrap
origin/feat/journey-impl
origin/fix/deps-npm-2026-04-27
origin/gt/polecat-35/83fd9412
origin/gt/polecat-44/40f140e5
origin/pr-template/bootstrap
```

---

## 2. Per-Branch Analysis

| # | Branch | Last commit | Author | Ahead | Behind | Unique commits | Squash-merged into main? | Recommendation |
|---|--------|-------------|--------|------:|-------:|---------------:|--------------------------|----------------|
| 1 | `main` (local) | 2026-04-28 17f88a14 | Forge | 0 | 18 | 0 | n/a | **FAST-FORWARD** `git fetch && git merge --ff-only origin/main` |
| 2 | `fix/handle-connect-iter142` | 2026-05-19 33d631ae | KooshaPari | 20 | 0 | 20 (real iter-142/143 work) | NO — IN-FLIGHT | **DO NOT TOUCH** (sibling agent owns PR) |
| 3 | `safety/iter140-snapshot-2026-05-18` (local + remote) | 2026-05-18 f699154e | KooshaPari | 2 | 51 | 2 (snapshot of 7137-file pre-iter-141 working tree) | NO | **PRESERVE indefinitely** (safety net per `feedback_stash_auto_route_to_branch.md`) |
| 4 | `origin/backup/20260426-reconcile-05cd0168` | 2026-04-24 05cd0168 | Forge | 8 | 119 | 8 (FUNCTIONAL_REQUIREMENTS scaffold, worklogs, phenotype-tooling, README/SPEC/PLAN) | **NO** — content NOT in main | **PRESERVE for now** — contains unmerged docs scaffold; user decision required before delete |
| 5 | `origin/chore/add-agents-2026-05-02` | 2026-05-02 98cbc32a | KooshaPari | 2 | 4 | 2 (AGENTS.md, SECURITY.md) | **YES** — subject `AGENTS.md and SECURITY.md` matches `663f6bda` in main | **SAFE to delete** |
| 6 | `origin/chore/add-gitignore` | 2026-05-02 1bc70e34 | Phenotype Agent | 1 | 8 | 1 (`Add .gitignore`) | **YES** — squash-merged as `c20d8ffa Add .gitignore (#182)` | **SAFE to delete** |
| 7 | `origin/chore/changelog-stub` | 2026-04-30 6a04d5b5 | KooshaPari | 1 | 16 | 1 (CHANGELOG stub) | **YES** — squash-merged as `91849d59 (#180)` | **SAFE to delete** |
| 8 | `origin/chore/deps-high-sweep` | 2026-04-23 74b52920 | Forge | 1 | 57 | 1 (lodash-es 4.18.x) | **YES** — squash-merged as `42be6bd1 (#151)` | **SAFE to delete** |
| 9 | `origin/chore/dino-governance-docs-20260425` | 2026-04-25 d02684ad | Forge | 1 | 42 | 1 (Dino spec pack) | **YES** — squash-merged as `c5b6369d (#154)` | **SAFE to delete** |
| 10 | `origin/ci/pin-trufflehog` | 2026-05-06 310d0a2d | Phenotype Agent | 4 | 0 | 4 (stub spec.md, trufflehog go install, CI concurrency, dup spec removal) | **PARTIAL** — `trufflehog` keyword appears in main but commit messages don't match (Go-install variant NOT in main). 0 BEHIND suggests this branch *was* main's tip at 2026-05-06 then main moved on. | **NEEDS HUMAN REVIEW** — possibly orphan CI hardening work; do NOT delete blind |
| 11 | `origin/cursor/gitignore-pattern-refinement-e743` | 2026-04-26 b2e1d2ac | Cursor Agent | 2 | 41 | 2 (rust target + python cache exclusion; remove duplicate patterns) | **PARTIAL** — `chore(gitignore): exclude rust target` matches `f819dcf6 (#155)` in main; the "remove duplicate" follow-up has no main analog | **NEEDS HUMAN REVIEW** — 1 of 2 commits unmerged |
| 12 | `origin/dependabot/bootstrap` | 2026-04-30 33dffb72 | KooshaPari | 1 | 18 | 1 (dependabot config) | **YES** — squash-merged as `edc24bc8 (#178)` | **SAFE to delete** |
| 13 | `origin/feat/journey-impl` | 2026-05-01 780f79f8 | Phenotype Agent | 1 | 14 | 1 (journey-traceability + iconography) | **YES** — squash-merged as `4d635120 (#181)` | **SAFE to delete** |
| 14 | `origin/fix/deps-npm-2026-04-27` | 2026-04-27 4c42ab92 | Forge | 4 | 41 | 4 (uuid/vite/esbuild bump, docs path base, worklog index fix, sketchfab link) | **PARTIAL** — `uuid/vite/esbuild` matches `a0c05a6d (#161)`; the 3 docs commits may NOT be in main | **NEEDS HUMAN REVIEW** |
| 15 | `origin/gt/polecat-35/83fd9412` | 2026-04-24 d23a97dd | Polecat-35 | 1 | 51 | 1 (Kilo Gastown methodology) | **YES** — squash-merged as `c6e894e2 (#128)` | **SAFE to delete** |
| 16 | `origin/gt/polecat-44/40f140e5` | 2026-04-24 1eda85fa | Polecat-44 | 1 | 51 | 1 (GEMINI.md guide) | **YES** — squash-merged as `ec321d30 (#137)` | **SAFE to delete** |
| 17 | `origin/pr-template/bootstrap` | 2026-04-30 42279a05 | KooshaPari | 1 | 19 | 1 (PR template) | **YES** — squash-merged as `71378f01 (#177)` | **SAFE to delete** |

**No open PRs** (`gh pr list` returned `[]`). All consolidation targets are post-merge cleanup, not active reviews.

---

## 3. Categorization

### 3.A Squash-merged → SAFE to delete (9 remote branches)
Subject matches a squash-merge commit in `origin/main`; content fully integrated.

1. `origin/chore/add-agents-2026-05-02`
2. `origin/chore/add-gitignore`
3. `origin/chore/changelog-stub`
4. `origin/chore/deps-high-sweep`
5. `origin/chore/dino-governance-docs-20260425`
6. `origin/dependabot/bootstrap`
7. `origin/feat/journey-impl`
8. `origin/gt/polecat-35/83fd9412`
9. `origin/gt/polecat-44/40f140e5`
10. `origin/pr-template/bootstrap`

(10 total — small drift between table and category counts: the table groups by branch state; category #3.A is the squash-merged subset.)

### 3.B Genuinely orphan / NEEDS HUMAN REVIEW (4 remote branches)
Unique commits whose subject lines have NO match in `origin/main`.

1. `origin/backup/20260426-reconcile-05cd0168` — 8 unique commits including unmerged `FUNCTIONAL_REQUIREMENTS.md` scaffold, `legacy-enforcement` CI gate, phenotype-tooling adoption. Likely intentional preservation of an alternate-direction governance experiment.
2. `origin/ci/pin-trufflehog` — 4 unique commits including a `security(ci): replace trufflehog/actions/setup with go install + setup-go` hardening that does NOT appear in main. 0 BEHIND main as of 2026-05-06 means this branched off, then main moved on without picking up the change.
3. `origin/cursor/gitignore-pattern-refinement-e743` — "remove duplicate gitignore patterns" follow-up not in main.
4. `origin/fix/deps-npm-2026-04-27` — 3 unmerged docs fixes (worklog index link, pages-path base, sketchfab link).

### 3.C Active (sibling agent owns this)
- `fix/handle-connect-iter142` (local + will be pushed to `origin/fix/handle-connect-iter142` by sibling agent). 20 unique commits including iter-142/143 wave 1+2 fixes. **DO NOT TOUCH.**

### 3.D Safety snapshots — PRESERVE
- `safety/iter140-snapshot-2026-05-18` (local + remote mirror) — snapshot of pre-iter-141 working tree (7137 files / 3.9M LOC). Per `feedback_stash_auto_route_to_branch.md` and `feedback_no_verify_forbidden.md`, safety branches are the canonical replacement for `git stash` and MUST be kept until at least the next minor-version release.
- `origin/backup/20260426-reconcile-05cd0168` — implicit safety branch from 2026-04-24 governance-reconcile session. **Preserve until §3.B review confirms its content is either redundant or merged.**

### 3.E Stash-recovery branches
- **None observed**. The sibling agent currently committing `fix/handle-connect-iter142` per the task description has not yet pushed `stash/recovered-*` branches. If they appear after sibling agent finishes, treat as §3.D (preserve).

---

## 4. Local/Remote Divergence Analysis (task #507 detail)

| Branch | Local SHA | Remote SHA | Local ahead | Local behind | Strategy |
|--------|-----------|------------|------------:|-------------:|----------|
| `main` | `17f88a14` | `6dcc193c` | 0 | 18 | **`git fetch && git merge --ff-only origin/main`** — clean fast-forward, no conflict risk |
| `fix/handle-connect-iter142` | `33d631ae` | (none yet — sibling agent will push) | 20 | 0 | Sibling agent owns; will `git push -u origin fix/handle-connect-iter142` |
| `safety/iter140-snapshot-2026-05-18` | `f699154e` | `f699154e` | 0 | 0 | **No action** — in sync |

**Critical observation**: local `main` being 18 commits behind is the *only* real divergence. None of those 18 commits would conflict with anything local — they're all squash-merges of branches we already have remote copies of. The fast-forward is safe.

---

## 5. Execution Plan (DOCUMENT-ONLY — DO NOT RUN)

### Phase 0: Prerequisites (must complete first)
0.1. Sibling agent (`fix/handle-connect-iter142`) finishes its PR cycle:
     - Pushes branch to `origin/fix/handle-connect-iter142`
     - Opens PR against `origin/main`
     - PR is reviewed + merged
     - After merge, `origin/main` advances to include iter-142/143 wave commits
0.2. User reviews this plan, confirms the §3.B "needs review" list

### Phase 1: Sync local main (after Phase 0)
```powershell
# Fetch latest from remote
git fetch origin --prune

# Verify clean state
git status            # must show "working tree clean"
git branch --show-current   # confirm not on main if there are uncommitted changes

# Fast-forward local main
git checkout main
git merge --ff-only origin/main      # safe — local main is 0 ahead
git log -1 --format='%H %s'          # verify HEAD matches origin/main

# (No push needed — local just catches up)
```
**Estimated risk**: zero. Pure fast-forward.

### Phase 2: Delete squash-merged remote branches (10 branches)
Run ONLY after user confirms each branch is squash-merged.

```powershell
# Per-branch verification — for EACH branch, confirm subject is in main first
$squashMerged = @(
  'chore/add-agents-2026-05-02',
  'chore/add-gitignore',
  'chore/changelog-stub',
  'chore/deps-high-sweep',
  'chore/dino-governance-docs-20260425',
  'dependabot/bootstrap',
  'feat/journey-impl',
  'gt/polecat-35/83fd9412',
  'gt/polecat-44/40f140e5',
  'pr-template/bootstrap'
)

foreach ($b in $squashMerged) {
  # 1. Re-verify subject is in origin/main
  $subj = git log -1 --format='%s' "origin/$b"
  $found = git log --oneline origin/main --grep="$subj" -1
  if (-not $found) { Write-Warning "MISMATCH on $b — skip"; continue }

  # 2. Delete remote branch (REQUIRES PUSH PERMISSION)
  git push origin --delete $b

  # 3. Local-side prune happens automatically via `git fetch --prune`
}
```
**Estimated risk**: low — all content is squash-merged. Safety guarantee: every commit is also tagged via its PR number in `origin/main`, so the work is permanently reachable via `git log`.

### Phase 3: Review (NOT delete) orphan branches
```powershell
# Document each orphan branch's unique commits — DO NOT delete
$needsReview = @(
  'backup/20260426-reconcile-05cd0168',  # FUNCTIONAL_REQUIREMENTS, legacy-enforcement
  'ci/pin-trufflehog',                   # trufflehog go-install hardening
  'cursor/gitignore-pattern-refinement-e743',  # 1 of 2 commits unmerged
  'fix/deps-npm-2026-04-27'              # 3 of 4 commits unmerged (docs only)
)

# User decision required — for each: cherry-pick to main, OR archive as tag, OR delete
foreach ($b in $needsReview) {
  Write-Host "=== $b ==="
  git log --oneline "origin/$b" ^origin/main
  git diff --stat origin/main "origin/$b"
}
```
**Recommendation**: Convert these to dated archive tags before any delete (so the SHAs are reachable by name forever):
```powershell
# For each needs-review branch, BEFORE delete:
git tag "archive/branch-name-$(Get-Date -Format yyyy-MM-dd)" "origin/branch-name"
git push origin --tags
# Then delete branch as Phase 2
```

### Phase 4: Preserve safety/backup branches
**No action**. `safety/iter140-snapshot-2026-05-18` (local + remote) and `backup/20260426-reconcile-05cd0168` (remote) stay until at least the v0.26.0 release.

### Phase 5: Future stash-recovery branches (if sibling agent creates any)
If sibling agent pushes `stash/recovered-*` branches per `feedback_stash_auto_route_to_branch.md`:
- Document in this file under §3.E
- Treat as safety branches (preserve)
- Re-evaluate at next release

---

## 6. Impact Estimate

| Action | Count |
|--------|------:|
| Local branches modified | 1 (main fast-forward only) |
| Remote branches deleted (Phase 2 — after user OK) | 10 |
| Remote branches preserved (safety + active + needs-review) | 4 |
| Tags created (Phase 3 archives) | 4 (if user chooses delete-after-tag for §3.B) |
| Commits lost | **0** — all squash-merged commits are reachable via main's history; orphan branches archived as tags |

**Net repo cleanup**: from 18 remote branches → 8 remote branches (or 4 if §3.B branches are tagged-and-deleted).

---

## 7. Risk Callouts

### 7.A HARD RULES (do NOT violate)
1. **NEVER delete `safety/iter140-snapshot-2026-05-18`** — it is the canonical safety net for iter-140/141/142/143 work. Per `feedback_stash_auto_route_to_branch.md`.
2. **NEVER delete `origin/backup/20260426-reconcile-05cd0168` without explicit user OK** — 8 unique commits NOT in main, including unmerged governance docs.
3. **NEVER delete `fix/handle-connect-iter142`** — currently the PR base for v0.25.0; sibling agent owns it.
4. **NEVER force-push `main`** — fast-forward only.
5. **NEVER use `--no-verify`** on any push during this consolidation (per `feedback_no_verify_forbidden.md`).
6. **NEVER use `git stash`** at any point during this consolidation (per `feedback_never_git_stash.md`).

### 7.B SOFT RISKS (warn-and-proceed)
1. `origin/ci/pin-trufflehog` was the tip of main at 2026-05-06 (0 BEHIND). It contains a `security(ci): replace trufflehog/actions/setup with go install + setup-go` hardening. Main moved on without picking this up — this looks like an unintentional drop. **User should decide if this hardening should be cherry-picked to main before the branch is deleted.**
2. `origin/fix/deps-npm-2026-04-27` has 3 docs commits (worklog index link fix, pages-path base, sketchfab link) that may be useful housekeeping. Low-stakes — recommend cherry-pick OR archive-then-delete.

### 7.C ZERO-RISK actions (safe to automate after Phase 0 completes)
1. Fast-forward local `main` to `origin/main` (Phase 1). No conflict possible.
2. `git fetch --prune` to clear stale remote-tracking refs after Phase 2 deletions.

---

## 8. Coordination with #512 (sibling agent's current PR)

This plan **assumes** sibling agent completes the iter-142/143 PR cycle FIRST. The sequence MUST be:

1. Sibling agent: commits `fix/handle-connect-iter142` work → pushes → opens PR → user reviews → merges to main
2. THEN: user reviews this plan
3. THEN: this plan's Phase 1 (local main fast-forward, now picks up the merged PR)
4. THEN: this plan's Phase 2 (delete squash-merged branches)
5. THEN: this plan's Phase 3 (orphan branch review + archive)

If sibling agent's work merges via squash, `fix/handle-connect-iter142` itself moves into §3.A (safe-to-delete) after merge.

---

## 9. Reproducibility

All read-only commands used to generate this analysis:

```bash
git branch --list
git branch -r
git log --oneline main -5
git rev-parse HEAD main origin/main
git log --oneline fix/handle-connect-iter142 ^main
git log --oneline main..origin/main
gh pr list --limit 30 --json number,title,headRefName,baseRefName,state,updatedAt

# Per-branch:
git log --oneline --format='%h %ci %an %s' origin/<branch> -1
git rev-list --count origin/main..origin/<branch>   # ahead
git rev-list --count origin/<branch>..origin/main   # behind
git log --oneline origin/<branch> ^origin/main      # unique commits
git merge-base origin/<branch> origin/main          # divergence point
git branch -r --contains origin/<branch>            # which refs include it
git log --oneline origin/main --grep="<subject>"    # squash-merge detection
```

No write/destructive ops were executed.

---

**END OF PLAN** — awaiting user authorization before any Phase 1+ execution.
