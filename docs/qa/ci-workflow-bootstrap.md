# CI workflow bootstrap on `main`

**Date:** 2026-05-24 (updated 2026-05-29)  
**Status:** **Superseded.** `origin/main` @ `125d44ad` now has full `.github/workflows/ci.yml` (build/test/Sonar). This doc remains as historical context for the #163 workflow deletion and bootstrap path. PR #189 no longer adds a stub `ci.yml`; merge with `main` keeps the full workflow.

## Findings

### What is on `origin/main` today

```bash
git ls-tree -r --name-only origin/main -- .github/workflows/
```

| File | Purpose |
|------|---------|
| `.github/workflows/scorecard.yml` | OpenSSF Scorecard (push to `main`, weekly schedule) |
| `.github/workflows/trufflehog.yml` | Secret scanning |

```bash
git show origin/main:.github/workflows/ci.yml
# fatal: path '.github/workflows/ci.yml' exists on disk, but not in 'origin/main'
```

### Why `ci.yml` is missing

1. **PR #163** (`9fbe0813` — *Add root Taskfile.yml*) deleted **24** workflow files in one commit, including `ci.yml`, `lint.yml`, `codeql.yml`, `polyglot-build.yml`, and the rest of the CI stack (~3103 lines removed).
2. **`main` was later rebuilt minimally** — governance and security-only workflows:
   - `f8d87bdd` — trufflehog
   - `0944b02f` — scorecard
   - `f0c02791` — bootstrap FUNDING + trufflehog (#186)
3. **Full CI never landed back on `main`.** The complete workflow set (40+ files) lives on long-lived branches such as `safety/iter145-recovery-20260523-0432` (~4490 lines ahead of `origin/main` under `.github/workflows/`).

### GitHub Actions behavior (why this matters)

| Scenario | Behavior |
|----------|----------|
| PR adds `ci.yml` only on the PR branch | For **same-repo** PRs, `pull_request` workflows from the PR head **can** run. They do **not** run on `push` to `main` until merged. |
| Branch protection / required checks named `CI` / `build` | Checks that reference workflow names/jobs that **do not exist on `main`** will stay missing or stale until `main` has the workflow file. |
| `workflow_run`, `workflow_call`, reusable workflows | Depend on the callee existing on the **default branch** (or the ref that triggered the chain). |
| Follow-up PRs that only touch code | Expect green CI on `main`; without bootstrap, only scorecard/trufflehog run on `main` pushes. |

**Conclusion:** Treat restoring `.github/workflows/ci.yml` on `main` as a **bootstrap PR**, then follow with a **full CI restoration PR** (or merge the safety/recovery branch workflow tree in slices).

## Recommended merge path

### Option A — Minimal bootstrap first (this branch)

1. Merge a **trigger-only** `ci.yml` stub to `main` (see `.github/workflows/ci.yml` on branch `docs/ci-workflow-bootstrap`).
2. Unblocks branch-protection check names and proves Actions wiring on `main`.
3. Open a second PR to replace the stub with the full workflow from `safety/iter145-recovery-20260523-0432` (or cherry-pick `9355a058` / latest `ci.yml`).

### Option B — Full CI in one PR

1. Cherry-pick or merge the entire `.github/workflows/` diff from the recovery branch:

   ```bash
   git fetch origin
   git checkout -b ci/restore-workflows origin/main
   git checkout safety/iter145-recovery-20260523-0432 -- .github/workflows/
   # Resolve conflicts (trufflehog.yml differs); keep main's trufflehog if needed
   git commit -m "ci: restore workflow tree removed in #163"
   ```

2. Expect first `main` run to surface real failures (secrets, Sonar token, lockfiles, etc.) — fix in follow-ups.

### Option C — Document-only (no workflow merge yet)

Use this doc in PR descriptions until a human approves merging workflows to `main`. PR CI may still run from the PR branch for same-repo PRs, but **`main` push CI stays absent**.

## Step-by-step: bootstrap PR (Option A)

```bash
# From a clean tree based on origin/main
git fetch origin
git checkout -b docs/ci-workflow-bootstrap origin/main

# Ensure stub exists (or copy from this branch)
# .github/workflows/ci.yml  — trigger-only job

git add docs/qa/ci-workflow-bootstrap.md .github/workflows/ci.yml
git commit -m "docs(qa): document CI workflow bootstrap; add ci.yml stub on main"
git push -u origin docs/ci-workflow-bootstrap
```

Open PR → **base: `main`** → title: `ci: bootstrap ci.yml on main (stub + docs)`.

After merge:

```bash
git checkout main && git pull
gh run list --workflow=ci.yml --limit 3
```

## Step-by-step: full restoration (after bootstrap)

```bash
git fetch origin
git checkout -b ci/restore-full origin/main
git show safety/iter145-recovery-20260523-0432:.github/workflows/ci.yml > .github/workflows/ci.yml
# Or restore entire directory:
git checkout safety/iter145-recovery-20260523-0432 -- .github/workflows/
git add .github/workflows/
git commit -m "ci: restore full CI workflow (post-#163)"
git push -u origin ci/restore-full
```

Verify required secrets in repo settings: `SONAR_TOKEN`, any deploy keys, codecov, etc.

## Verification commands

```bash
# List workflows on remote main
git ls-tree -r --name-only origin/main -- .github/workflows/

# Confirm ci.yml absent on main
git cat-file -e origin/main:.github/workflows/ci.yml 2>&1 || echo "missing (expected today)"

# Compare workflow tree vs recovery branch
git diff origin/main safety/iter145-recovery-20260523-0432 --stat -- .github/workflows/

# Last commit that deleted ci.yml on main ancestry
git log origin/main --oneline --diff-filter=D -- .github/workflows/ci.yml
# → 9fbe0813 Add root Taskfile.yml. (#163)
```

## References

| Item | Value |
|------|-------|
| Deletion commit | `9fbe0813` (PR #163) |
| Current `main` tip (2026-05-24) | `6dcc193c` |
| Recovery branch with full CI | `safety/iter145-recovery-20260523-0432` @ `7cc4df4e` |
| Workflow path audit | `docs/qa/workflow_path_audit_iter142.md` |
| Bootstrap branch | `docs/ci-workflow-bootstrap` |

## Trufflehog CI failure (PR #189, 2026-05-25)

The failing check was **not** a verified secret hit. The PR branch used `trufflehog/actions/setup@main`, which GitHub Actions cannot resolve (`repository not found`). `origin/main` pins `trufflesecurity/trufflehog@…` in `.github/workflows/trufflehog.yml`. Merging `main` into `docs/ci-workflow-bootstrap` fixes trufflehog for subsequent runs.

## Do not merge to `main` without explicit owner approval

Agent/automation branches should stop at **push + PR link**. Merging workflow changes to `main` changes org-wide CI behavior and branch protection.
