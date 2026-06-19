# History Purge Plan — build-artifact trees (follow-up to #364)

Status: PLANNED — execute only AFTER the current PR merge wave lands.
Trace: NFR-CIV-DEV-HYGIENE-001.

## Problem
PR #364 untracked 23,161 build-artifact paths (`target-check-build/`, `target-check-test2/`,
`target-check-clippy3/`, `target-ci/`, `.target-*`) from the index, but every one of those
blobs remains in **history**, inflating clones/fetches and slowing object enumeration.
Observed costs (2026-06-10 session): fresh-worktree checkouts materialized ~1.5GB of
artifacts each; three disk-full incidents; checkout failures on over-long artifact paths.

## Procedure (git-filter-repo)
1. **Mirror backup first**: `git clone --mirror https://github.com/KooshaPari/Civis.git civis-backup.git`
   and archive it off-machine.
2. Freeze merges (announce; pause the loop's merge lanes).
3. Fresh mirror clone, then:
   ```bash
   printf 'target-check-build/\ntarget-check-test2/\ntarget-check-clippy3/\ntarget-ci/\nglob:.target-*\n' > /tmp/purge-paths.txt
   git filter-repo --invert-paths --paths-from-file /tmp/purge-paths.txt
   ```
4. Temporarily disable branch protection on `main`; `git push --force --mirror origin`; re-enable protection.
5. Every machine/worktree re-clones (worktrees against the old object store are invalid).
   Update: `C:/Users/koosh/Dev/Civis`, `civis-game`, all `G:/civis-wt-*`.
6. Verify: `git rev-list --objects --all | grep -cE 'target-check|target-ci'` → 0; repo size delta recorded here.

## Sequencing / blast radius
- All open PRs must be MERGED or intentionally closed first — filter-repo rewrites their base SHAs.
- Run within one sitting; CI (post-billing-fix) re-runs on the rewritten main.

## Alternatives considered
- **BFG Repo-Cleaner**: faster but path-glob support is weaker for this mixed pattern set — rejected.
- **Partial clone (`--filter=blob:none`)** as mitigation only: helps new clones, leaves history dirty — complementary, not sufficient.
- **Do nothing**: clone cost stays ~GBs and grows — rejected.
- **git-filter-repo (chosen)**: canonical, scriptable, preserves authorship.
