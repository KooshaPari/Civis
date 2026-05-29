# DINOForge Branch/PR/Stash Consolidation — Execution Plan

Generated: 2026-05-28  
Source audits: `branch-consolidation-audit.md`, `pr-consolidation-audit.md`, `local-git-state-audit.md`

---

## Summary Counts

| Category | Count |
|---|---:|
| Remote branches to delete (ALREADY-MERGED, clean) | 3 |
| Remote branches to delete (STALE — safe-delete list) | 17 |
| Remote branches to delete (STALE — needs-human-review) | 3 |
| Remote branches needing-human-review before delete | 1 (NEEDS-INVESTIGATION: `origin/origin`) |
| Open PRs — READY-TO-MERGE | 5 |
| Open PRs — NEEDS-REBASE then merge | 4 |
| Active-unmerged branches needing a PR | 7 |
| Active-unmerged branches — dependabot (auto-handled) | 5 |
| Local branches to delete (already in main) | 8 |
| Local stashes needing human decision | 2 |
| Items needing human review | 6 |

---

## CRITICAL SEQUENCING RULE

> **Land `feat/unityexplorer-devtools-20260528` into `main` FIRST — before any other merge, PR creation, or rebase. Every branch in this plan that touches overlapping files (CI, SDK, docs, packs, lockfiles) must be rebased onto the post-merge `main` tip, not the pre-merge tip. Doing it in any other order guarantees conflicts that require force-resolution.**

---

## Phase A — Delete ALREADY-MERGED remote branches

These three branches have 0 commits ahead of `origin/main` and are fully merged. Run immediately — no risk.

```bash
# 3 already-merged remote branches
git push origin --delete docs/sonar-pr188-hotspots
git push origin --delete fix/governance-pr188
git push origin --delete fix/sonar-pr188-blockers
```

> Hooks fire on push --delete (lefthook pre-push typically no-ops on deletions, but verify once). If a hook blocks: investigate the hook, do NOT pass --no-verify.

---

## Phase B — Merge READY-TO-MERGE open PRs

**Sequence these AFTER `feat/unityexplorer-devtools-20260528` lands.** All five are green and MERGEABLE today, but each touches lockfiles or dependency manifests that `feat/unityexplorer-devtools-20260528` also modifies (see Conflict Map below). Merging before that branch lands guarantees the big branch will need an extra rebase pass.

**Merge order (no dependency between them; merge in any order after the big branch):**

```bash
# PR #193 — ndarray Rust bump (no C# overlap, lowest-risk first)
gh pr merge 193 --squash --delete-branch

# PR #206 — nalgebra Rust bump
gh pr merge 206 --squash --delete-branch

# PR #209 — Playwright dev bump
gh pr merge 209 --squash --delete-branch

# PR #215 — @remotion/cli scripts/video bump
gh pr merge 215 --squash --delete-branch

# PR #230 — uuid npm bump
gh pr merge 230 --squash --delete-branch
```

**Notes:**
- `--squash` keeps main history clean for bot PRs. Use `--merge` if you prefer commit granularity.
- `--delete-branch` removes the head branch from origin automatically.
- If any check goes red between now and merge, investigate before merging.

---

## Phase C — Rebase-then-merge NEEDS-REBASE PRs

These four PRs were created before the current wave of commits and now conflict with `origin/main`. The correct sequence:

1. Wait for `feat/unityexplorer-devtools-20260528` to land.
2. For each PR: rebase its head branch onto the new `main` tip.
3. Force-push the rebased branch.
4. Let CI run.
5. Merge once green.

**Do NOT rebase until `feat/unityexplorer-devtools-20260528` is in main — otherwise you will rebase twice.**

```bash
# --- After the big feature branch is in main ---

# PR #216 — Snyk mermaid bump (docs/package-lock only, very small diff)
git fetch origin
git checkout snyk-fix-9edfd94ce8b34257abbe93fcae9e822c
git rebase origin/main
git push origin snyk-fix-9edfd94ce8b34257abbe93fcae9e822c --force-with-lease
# Let CI run, then:
gh pr merge 216 --squash --delete-branch

# PR #217 — Snyk @remotion/cli bump (scripts/video/package.json only)
git checkout snyk-fix-7389a3c591b5d9eb5726479c717e9955
git rebase origin/main
git push origin snyk-fix-7389a3c591b5d9eb5726479c717e9955 --force-with-lease
gh pr merge 217 --squash --delete-branch

# PR #218 — Snyk idna bump (src/Tests/e2e/requirements.txt only)
git checkout snyk-fix-80168d908625f6d971bf41969dd61351
git rebase origin/main
git push origin snyk-fix-80168d908625f6d971bf41969dd61351 --force-with-lease
gh pr merge 218 --squash --delete-branch

# PR #189 — docs/ci-workflow-bootstrap (2-file docs stub)
git checkout docs/ci-workflow-bootstrap
git rebase origin/main
git push origin docs/ci-workflow-bootstrap --force-with-lease
gh pr merge 189 --squash --delete-branch
```

**Conflict expectations:**
- PRs #216, #217, #218 are single-file security bumps — rebase conflicts should be trivial (accept theirs or ours on lockfiles).
- PR #189 touches CI docs — moderate conflict risk if `feat/unityexplorer-devtools-20260528` also modified `.github/workflows/`. Inspect `git diff --name-only origin/main...docs/ci-workflow-bootstrap` after rebasing.

---

## Phase D — Active-unmerged branches: open PRs or rebase-and-PR

These seven branches have unique work not in `origin/main`. They are not currently open as PRs (except dependabot ones which are handled in Phase B/C above).

**All must be rebased onto `main` AFTER the big feature branch lands.**

### D1 — Dependabot branches (already have open PRs — handled above)

These five already have PRs (#193, #206, #209, #215, #230). Do NOT open duplicate PRs. They are addressed in Phase B.

```
origin/dependabot/cargo/src/Tools/AssetPipelineRust/nalgebra-0.35.0   → PR #206
origin/dependabot/cargo/src/Tools/AssetPipelineRust/ndarray-0.17.2    → PR #193
origin/dependabot/npm_and_yarn/npm_and_yarn-e9ce4f7be9                 → PR #230
origin/dependabot/npm_and_yarn/playwright-1.60.0                       → PR #209
origin/dependabot/npm_and_yarn/scripts/video/remotion/cli-4.0.467      → PR #215
```

### D2 — Human-authored active branches (open PRs for these after big branch lands)

For each branch: rebase onto new `main`, push, open PR, let CI run, then merge.

```bash
# --- After feat/unityexplorer-devtools-20260528 is in main ---

# 1. agent/coderabbit-main-config (1 commit, 1 file — CodeRabbit config on main)
git fetch origin
git checkout agent/coderabbit-main-config
git rebase origin/main
git push origin agent/coderabbit-main-config --force-with-lease
gh pr create \
  --head agent/coderabbit-main-config \
  --base main \
  --title "chore(ci): add CodeRabbit main-branch approval config" \
  --body "Adds the `.coderabbit.yaml` or equivalent config for main-branch review automation."

# 2. ci/pin-trufflehog (6 commits — CI hardening and security tooling pinning)
git checkout ci/pin-trufflehog
git rebase origin/main
git push origin ci/pin-trufflehog --force-with-lease
gh pr create \
  --head ci/pin-trufflehog \
  --base main \
  --title "ci: pin trufflehog action SHA and harden CI security tooling" \
  --body "Pins the trufflehog action to a verified SHA and adds related CI hardening changes."

# 3. cursor/agent-merge-workflow-issues-8376 (1 commit — merge workflow/logging fixes)
git checkout cursor/agent-merge-workflow-issues-8376
git rebase origin/main
git push origin cursor/agent-merge-workflow-issues-8376 --force-with-lease
gh pr create \
  --head cursor/agent-merge-workflow-issues-8376 \
  --base main \
  --title "fix(ci): agent merge workflow and logging fixes" \
  --body "Merge workflow and Actions automation logging fixes (Cursor agent contribution)."

# 4. cursor/bridge-and-security-issues-6930 (1 commit — Bridge/runtime + security bug fixes)
git checkout cursor/bridge-and-security-issues-6930
git rebase origin/main
git push origin cursor/bridge-and-security-issues-6930 --force-with-lease
gh pr create \
  --head cursor/bridge-and-security-issues-6930 \
  --base main \
  --title "fix(bridge,security): bridge/runtime and security bug fixes" \
  --body "Bridge runtime reliability and security fixes (Cursor agent contribution)."

# 5. cursor/docs-mermaid-lockfile-a19a (2 commits — docs lockfile refresh)
git checkout cursor/docs-mermaid-lockfile-a19a
git rebase origin/main
git push origin cursor/docs-mermaid-lockfile-a19a --force-with-lease
gh pr create \
  --head cursor/docs-mermaid-lockfile-a19a \
  --base main \
  --title "chore(docs): refresh lockfile for Mermaid dependency update" \
  --body "Updates the docs lockfile after the mermaid security bump."

# 6. cursor/security-bypass-and-code-duplication-9748 (1 commit — security bypass + dedupe)
git checkout cursor/security-bypass-and-code-duplication-9748
git rebase origin/main
git push origin cursor/security-bypass-and-code-duplication-9748 --force-with-lease
gh pr create \
  --head cursor/security-bypass-and-code-duplication-9748 \
  --base main \
  --title "fix(security): add bypass guard and remove code duplication" \
  --body "Security bypass guard and code duplication cleanup (Cursor agent contribution)."

# 7. feat/journey-impl (1 commit — journey traceability)
git checkout feat/journey-impl
git rebase origin/main
git push origin feat/journey-impl --force-with-lease
gh pr create \
  --head feat/journey-impl \
  --base main \
  --title "feat(traceability): journey implementation and iconography" \
  --body "Adds journey traceability tooling and iconography to the docs/build pipeline."
```

**NOTE on `cursor/docs-mermaid-lockfile-a19a`:** this branch likely overlaps with PR #216 (Snyk mermaid bump). Merge PR #216 first; if `cursor/docs-mermaid-lockfile-a19a` becomes redundant after that, close it instead of creating a duplicate PR.

---

## Phase E — Local stash and dirty-state handling

**CRITICAL: Never `git stash pop`, `git stash apply`, `git stash drop`, or `git stash branch`. The user explicitly forbids all stash operations.** Instead, the plan below routes stash contents to human inspection without touching the stash itself.

### stash@{0} — `lefthook auto backup` (NEEDS-INVESTIGATION)

**Recommendation: leave in place; flag for human review.**

This stash was created automatically by Lefthook's backup mechanism and spans a wide surface (CDN/lazy-load architecture, runtime/SDK asset CDN code, CLI cache/telemetry, PackCompiler validation, lockfiles). It may partially duplicate work already on `feat/unityexplorer-devtools-20260528` or may contain unique unreduced work.

**Human action required:**
1. Run `git stash show -p stash@{0} > docs/qa/stash-0-diff.txt` to capture the diff as a reviewable artifact — this does NOT pop the stash.
2. Compare the diff against the tip of `feat/unityexplorer-devtools-20260528` to find any unique hunks.
3. If unique work is found: manually copy those hunks into a new commit on a dated branch (e.g., `chore/stash-0-recovery-20260528`), push it, and open a PR.
4. Only after the recovery branch is merged should anyone consider dropping the stash.

### stash@{1} — `WIP on feat/unityexplorer-devtools-20260528` (PRESERVE-AND-MERGE)

**Recommendation: leave in place; inspect against current branch tip before the big PR merges.**

This stash is tied to the current feature branch by its message (`feat(devtools): bundle UnityExplorer as optional dev tool`). The work is likely either already on the branch tip or was left behind when `lefthook auto backup` fired.

**Human action required:**
1. Run `git stash show -p stash@{1} > docs/qa/stash-1-diff.txt` — capture without applying.
2. Compare against `git diff origin/main...HEAD` on `feat/unityexplorer-devtools-20260528`.
3. Any unique hunks should be manually committed to the current branch as a new commit before the PR is opened.
4. Do NOT drop the stash until a human has confirmed the recovered content is in a merged commit.

### Dirty working tree (14 modified tracked files + untracked files)

The dirty state on `feat/unityexplorer-devtools-20260528` includes:
- `lefthook.yml` (staged)
- 13 `packages.lock.json` files (modified)
- Untracked: `tools/phenotype-journeys/`, several new `docs/design/*.md`, `docs/research/*.md`, `docs/specs/v0.27.0*`, `docs/qa/assetswap-real-bundles-spec.md`, `telemetry-viewer-task.txt`

**Recommendation:** commit the staged and untracked files into the current branch before opening the PR for `feat/unityexplorer-devtools-20260528`. Group them logically:
- Commit 1: `lefthook.yml` + any related hook changes
- Commit 2: all 13 `packages.lock.json` regenerations (single chore commit)
- Commit 3: new docs design/research/specs files (if they belong in this branch)
- Commit 4: `tools/phenotype-journeys/` if it is part of this feature

`telemetry-viewer-task.txt` in the repo root looks like an agent scratch file — verify before committing; it may belong in `docs/sessions/` per Desktop contamination rules.

---

## Phase F — Delete STALE remote branches

Run AFTER the big feature branch merges and AFTER you have confirmed no stale branch contains unique work referenced by any currently-open PR.

### Safe-delete list (no open PRs, no unique unreduced work)

```bash
# Backup/snapshot branches — safely obsolete
git push origin --delete backup/20260426-reconcile-05cd0168
git push origin --delete safety/iter140-snapshot-2026-05-18
git push origin --delete safety/iter145-recovery-20260523-0432

# Stash-recovery branches — content should already be in main or the current feature branch
git push origin --delete stash/recovered-2026-05-19-1
git push origin --delete stash/recovered-2026-05-19-2
git push origin --delete stash/recovered-2026-05-19-3

# Chore stubs — superseded by main commits
git push origin --delete chore/add-agents-2026-05-02
git push origin --delete chore/add-gitignore
git push origin --delete chore/changelog-stub
git push origin --delete chore/deps-high-sweep
git push origin --delete chore/dino-governance-docs-20260425
git push origin --delete cursor/gitignore-pattern-refinement-e743
git push origin --delete dependabot/bootstrap
git push origin --delete pr-template/bootstrap

# Snyk branches covered by open PRs (PRs #216, #217, #218)
# WAIT until those PRs merge, then delete:
git push origin --delete snyk-fix-7389a3c591b5d9eb5726479c717e9955
git push origin --delete snyk-fix-80168d908625f6d971bf41969dd61351
git push origin --delete snyk-fix-9edfd94ce8b34257abbe93fcae9e822c

# Old dependency fix — likely superseded
git push origin --delete fix/deps-npm-2026-04-27
```

### Branches needing human review BEFORE delete

```bash
# gt/polecat-* — Gastown methodology artifacts; confirm no spec content needed in docs/
# Do NOT delete until a human reviews the commit content.
# git push origin --delete gt/polecat-35/83fd9412   # HOLD
# git push origin --delete gt/polecat-44/40f140e5   # HOLD

# gh-pages — the active VitePress deploy branch; NEVER delete
# git push origin --delete gh-pages   # FORBIDDEN
```

---

## Phase G — Delete local-only already-merged branches

After `feat/unityexplorer-devtools-20260528` is merged and you are on `main`:

```bash
# Local branches whose tips are already in origin/main
git branch -d feat/v0.26.0-fireworks-kimi-judge
git branch -d feat/v0.26.0-implementation-wave-1
git branch -d refactor/mcp-sonar-cpd-dedupe
git branch -d worktree-agent-a3846aabec020ba7d
git branch -d worktree-agent-a846f691378b3c472
git branch -d worktree-agent-ab5df1fe8f361ca51
git branch -d worktree-agent-ad246c04efa32c334
git branch -d worktree-agent-ae83614d2361217ad
```

Use `-d` (not `-D`) — it will refuse to delete if the branch is not yet merged, protecting you against mistakes.

---

## Conflict Map

The following branches and PRs will conflict with `feat/unityexplorer-devtools-20260528` because that branch carries 52+ commits touching the same file domains. All must be rebased AFTER it lands.

| Branch / PR | Conflict surface | Action |
|---|---|---|
| PR #189 `docs/ci-workflow-bootstrap` | `.github/workflows/`, docs CI stubs | Rebase after big branch; inspect diff carefully |
| PR #216 `snyk-fix-9edfd94ce8b34257abbe93fcae9e822c` | `docs/package-lock.json` / lockfiles | Rebase; lockfile conflict — accept "theirs" on lock entries |
| PR #217 `snyk-fix-7389a3c591b5d9eb5726479c717e9955` | `scripts/video/package.json` | Rebase; likely trivial version bump conflict |
| PR #218 `snyk-fix-80168d908625f6d971bf41969dd61351` | `src/Tests/e2e/requirements.txt` | Rebase; single-line pin, low conflict risk |
| `cursor/docs-mermaid-lockfile-a19a` | lockfiles, Mermaid dep | May become redundant after PR #216; compare before creating PR |
| `cursor/bridge-and-security-issues-6930` | Bridge/runtime source files | Rebase mandatory; inspect for C# merge conflicts |
| `ci/pin-trufflehog` (6 commits) | `.github/workflows/`, CI config | Highest conflict risk among active branches; rebase carefully |
| stash@{0} | Broad: lockfiles, runtime, SDK, CLI, docs | Must compare against post-merge `HEAD` before any recovery commit |
| stash@{1} | `feat/unityexplorer-devtools-20260528` surface | Compare against branch tip before PR opens |
| All 13 dirty `packages.lock.json` files | Lock regeneration | Commit onto current branch before PR; will conflict if merged to main separately |

**Branches unlikely to conflict** (isolated file domains):
- `agent/coderabbit-main-config` — `.coderabbit.yaml` only
- `cursor/agent-merge-workflow-issues-8376` — Actions YAML only (minor)
- `cursor/security-bypass-and-code-duplication-9748` — isolated security guard
- `feat/journey-impl` — docs/traceability only
- All dependabot bumps (Cargo/ndarray/nalgebra) — `Cargo.lock` only

---

## Risk Callouts

### HIGH RISK — Human decision required

1. **stash@{0} scope**: The `lefthook auto backup` stash is the single highest-risk item. It spans CDN architecture, SDK, CLI, and test changes that may or may not be on the feature branch. Do NOT discard it. Capture the diff to a file and review before the big PR merges.

2. **`origin/origin` pseudo-ref**: An unknown ref appeared in `git branch -r` output. It is 0 commits ahead of `origin/main` but is NOT a normal branch name. It could be a corrupted remote tracking entry. Run `git remote prune origin` to clean dangling refs after confirming it maps to nothing real — but do NOT delete it blindly.

3. **`gh-pages` branch**: Listed as STALE in the audit (9 commits ahead of main, older date). It is the live VitePress deployment target. Do NOT delete it. Any "stale" classification is an artifact of divergent history, not mergeability.

4. **`gt/polecat-*` branches**: These 2-branch Gastown methodology artifacts (1 commit each) are ahead of `origin/main` and not merged. They may contain spec or doc content that was never integrated. A human must read the commit diffs before deleting.

5. **`ci/pin-trufflehog` (6 commits ahead)**: This is the largest non-dependabot active branch after the current feature branch. It has 6 commits of CI hardening. It will have a real rebase conflict with the big feature branch's CI changes. Assign this a dedicated rebase session — do not batch it with the single-commit branches.

6. **Dirty tree / `telemetry-viewer-task.txt`**: A plain-text task file sitting at the repo root is a governance violation (should be in `docs/sessions/`). Verify its contents before committing; move it to `docs/sessions/telemetry-viewer-task.md` if it is a session note.

### MEDIUM RISK — Proceed with care

7. **`cursor/bridge-and-security-issues-6930`**: Touches Bridge/runtime C# source. After the big branch lands with 52+ commits of SDK/runtime changes, a rebase here may surface semantic conflicts that do not show as textual conflicts. Review the diff manually after rebase.

8. **13 `packages.lock.json` files in dirty tree**: These were modified by the current branch's builds. They must be committed to `feat/unityexplorer-devtools-20260528` before the PR opens — or the PR diff will be polluted with unrelated lock regenerations.

### LOW RISK — Routine

9. All five dependabot PRs are green and MERGEABLE. They are safe to merge in batch after the big branch lands. No manual review needed beyond confirming CI stays green.

10. The three ALREADY-MERGED remote branches can be deleted immediately without waiting for any sequencing.

---

## Safe-Delete List

These can be deleted with confidence once the big feature branch is in `main`. No unique work.

**Remote (run after feat/unityexplorer-devtools-20260528 merges):**
- `backup/20260426-reconcile-05cd0168`
- `chore/add-agents-2026-05-02`
- `chore/add-gitignore`
- `chore/changelog-stub`
- `chore/deps-high-sweep`
- `chore/dino-governance-docs-20260425`
- `cursor/gitignore-pattern-refinement-e743`
- `dependabot/bootstrap`
- `docs/sonar-pr188-hotspots` ← can delete NOW (already merged)
- `fix/deps-npm-2026-04-27`
- `fix/governance-pr188` ← can delete NOW (already merged)
- `fix/sonar-pr188-blockers` ← can delete NOW (already merged)
- `pr-template/bootstrap`
- `safety/iter140-snapshot-2026-05-18`
- `safety/iter145-recovery-20260523-0432`
- `snyk-fix-7389a3c591b5d9eb5726479c717e9955` ← after PR #217 merges
- `snyk-fix-80168d908625f6d971bf41969dd61351` ← after PR #218 merges
- `snyk-fix-9edfd94ce8b34257abbe93fcae9e822c` ← after PR #216 merges
- `stash/recovered-2026-05-19-1`
- `stash/recovered-2026-05-19-2`
- `stash/recovered-2026-05-19-3`

**Local (run after feat/unityexplorer-devtools-20260528 merges):**
- `feat/v0.26.0-fireworks-kimi-judge`
- `feat/v0.26.0-implementation-wave-1`
- `refactor/mcp-sonar-cpd-dedupe`
- `worktree-agent-a3846aabec020ba7d`
- `worktree-agent-a846f691378b3c472`
- `worktree-agent-ab5df1fe8f361ca51`
- `worktree-agent-ad246c04efa32c334`
- `worktree-agent-ae83614d2361217ad`

---

## Needs-Human-Review List

Do NOT touch these without human sign-off:

| Item | Why |
|---|---|
| `stash@{0}` — lefthook auto backup | Unknown unique work; may duplicate or extend the feature branch. Capture diff first. |
| `stash@{1}` — WIP on feat/unityexplorer-devtools-20260528 | Tied to current branch; review against branch tip before PR opens. |
| `origin/origin` pseudo-ref | Unknown nature; run `git remote prune origin` after human inspection. |
| `gt/polecat-35/83fd9412` | May contain spec/doc content; read commit diff before deleting. |
| `gt/polecat-44/40f140e5` | Same as above. |
| `gh-pages` | NEVER delete — live deployment branch. |

---

## Execution Order Summary

```
NOW (safe immediately):
  Phase A — delete 3 already-merged remote branches

BEFORE feat/unityexplorer-devtools-20260528 PR:
  Phase E (stash inspection) — capture stash diffs to docs/qa/
  Commit dirty tree onto current branch (lefthook.yml + packages.lock.json files + new docs)
  Move telemetry-viewer-task.txt to docs/sessions/

OPEN PR for feat/unityexplorer-devtools-20260528:
  Open the PR, get CI green, merge to main

AFTER feat/unityexplorer-devtools-20260528 lands:
  Phase B — merge 5 ready dependabot PRs (gh pr merge 193 206 209 215 230)
  Phase C — rebase 4 conflicting PRs (#189 #216 #217 #218) onto new main, re-CI, merge
  Phase D — rebase 7 human-authored active branches, open PRs for each, CI, merge
  Phase F — delete 17 stale remote branches (safe-delete list)
  Phase G — delete 8 local already-merged branches

DEFERRED (human decision required):
  Stash@{0} recovery decision
  gt/polecat-* branch content review
  origin/origin pseudo-ref cleanup (git remote prune origin)
```
