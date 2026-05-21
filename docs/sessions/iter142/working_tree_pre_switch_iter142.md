# Working Tree Pre-Switch Analysis (safety/iter140-snapshot → fix/handle-connect-iter142)

**Date**: 2026-05-18
**Current Branch**: `fix/handle-connect-iter142` (ced0dccf)
**Working Tree State**: 1 modified file + 14 untracked iter-142 audit documents

## Summary
- **(a) Total Uncommitted**: 15 items (1 modified `.claude/settings.json` + 14 untracked docs)
- **(b) Files Already on fix/handle-connect-iter142**: 0 (none of the current untracked docs exist on destination)
- **(c) Files NEW to Working Tree (iter-142 docs)**: 14 documents (all audit/session records from current session)
- **(d) Recommended Action**: **Commit new iter-142 docs to current branch BEFORE any switch**, OR copy them to a safety extension branch
- **(e) Risk of Data Loss on Switch**: **YES if switching without preserving untracked files** — all 14 iter-142 audit documents will be orphaned (not lost, but invisible on the destination branch)

---

## Detailed Findings

### (a) Uncommitted File Inventory

| Type | Count | Details |
|------|-------|---------|
| Modified staged/unstaged | 1 | `.claude/settings.json` (NEW-ONLY, not on fix/handle-connect) |
| Untracked | 14 | All `*_iter142.md` files (audit docs, branch inventory, governance notes, retrospective) |
| **TOTAL** | **15** | |

### (b) Files Already on fix/handle-connect-iter142

**Count: 0**

None of the 14 untracked iter-142 documents exist on the destination branch `fix/handle-connect-iter142`. Two similar iter-142 docs exist on that branch but with different names:
- `docs/sessions/asset_swap_investigation_iter142.md` (exists on fix/handle-connect, different from current work)
- `docs/sessions/branch_consolidation_state_iter142.md` (exists on fix/handle-connect, different from current work)

### (c) NEW Untracked Files (Iter-142 Audit Documents)

All 14 files are NEW to this session and do NOT exist on fix/handle-connect-iter142:

**docs/qa/** (7 files):
1. `benchmark_state_audit_iter142.md`
2. `build_errors_iter142.md`
3. `claude_commands_audit_iter142.md`
4. `git_push_diagnosis_iter142.md`
5. `governance_hardening_iter142.md`
6. `mcp_server_cpu_diagnosis_iter142.md`
7. `workflow_path_audit_iter142.md`

**docs/sessions/** (6 files):
8. `CONSOLIDATION_PR_DESCRIPTION_iter142.md`
9. `branch_consolidation_playbook_iter142.md`
10. `branch_inventory_local_iter142.md`
11. `branch_protection_audit_iter142.md`
12. `branch_provenance_audit_iter142.md`
13. `iter-142-retrospective.md`

**scripts/game/** (1 file):
14. `README-deploy-handle-connect-fix.md`

### (d) Recommended Action

**Option 1 (PREFERRED)**: Commit new iter-142 docs to current branch (`fix/handle-connect-iter142`) before any further work:
```bash
git add docs/qa/*_iter142.md docs/sessions/*_iter142.md scripts/game/README-deploy-handle-connect-fix.md
git commit -m "docs(iter-142): checkpoint audit documents for handle-connect investigation"
```
Then branch switches become safe — the files are tracked and available on all branches that pull this commit forward.

**Option 2**: Copy untracked files to a temporary safety location (`$env:TEMP\DINOForge_iter142_backup\`) before switching, then re-add them manually to the destination branch.

**Option 3**: Use `git stash branch` to preserve untracked + modified state into a new temporary branch, then cherry-pick the stashed documents onto fix/handle-connect-iter142 after switching.

### (e) Risk Assessment

**CURRENT BRANCH**: `fix/handle-connect-iter142` (ced0dccf)  
**UNCOMMITTED STATE**: 1 modified + 14 untracked = 15 items

**Risk Level**: **MEDIUM** if switching without preservation
- **.claude/settings.json**: Will be UNTRACKED on the destination (git checkout doesn't remove modified files; they're preserved in the working tree but may conflict with destination HEAD state)
- **14 iter-142 docs**: Will become ORPHANED if not committed before switching. They remain on disk but are invisible to git on the destination branch. Risk of accidental deletion during cleanup.

**Mitigation**: Commit or stash the iter-142 audit documents BEFORE switching branches. `.claude/settings.json` is safe to leave — it won't interfere with a checkout.

---

## Files at Risk of Loss

**If switching without preservation**:
- `docs/qa/benchmark_state_audit_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/build_errors_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/claude_commands_audit_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/git_push_diagnosis_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/governance_hardening_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/mcp_server_cpu_diagnosis_iter142.md` ← LOST (orphaned untracked)
- `docs/qa/workflow_path_audit_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/CONSOLIDATION_PR_DESCRIPTION_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/branch_consolidation_playbook_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/branch_inventory_local_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/branch_protection_audit_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/branch_provenance_audit_iter142.md` ← LOST (orphaned untracked)
- `docs/sessions/iter-142-retrospective.md` ← LOST (orphaned untracked)
- `scripts/game/README-deploy-handle-connect-fix.md` ← LOST (orphaned untracked)

## Next Steps

1. **Commit iter-142 audit docs** to current branch with a descriptive message
2. **Verify git status is clean** (`git status` shows no modifications or untracked files)
3. **Proceed with branch switches/merges** safely

---

**Prepared for**: CI/CD branch consolidation (iter-142)  
**Status**: Ready for branch preservation protocol
