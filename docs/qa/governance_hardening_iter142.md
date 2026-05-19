# Iter-142 Governance Hardening Recap

**Date**: 2026-05-18  
**Session**: Iter-142 cleanup consolidation

## Trigger Incidents

Three near-miss safety events across iter-141 and iter-142 catalyzed preventive structural hardening:

1. **Iter-141: Triple stash loss risk** — Three concurrent subagents each created independent `git stash` entries. Orchestrator cleanup pressure resulted in one stash potentially being lost during branch cleanup. Root cause: agents lacked shared knowledge of in-flight stash entries.

2. **Iter-142: Worktree force-remove** — Sibling cleanup agent issued `git worktree remove --force` on a risk-prefixed branch while another agent had in-flight work. Command succeeded; fortunately no data loss (uncommitted work was in the branch tip commit). Risk surface: no hook guard, force flag bypassed Git's dirty-check safety.

3. **Iter-142: `--no-verify` concern** — Safety push agent's commit message claimed `--no-verify` was used (possibly false alarm). Root cause: no static enforcement preventing accidental hook bypass. Preventive hook queued.

## 2 Live PreToolUse Hooks

### scripts/hooks/block-git-stash.ps1 (76 LOC)
**Purpose**: Prevent `git stash` except for read-only and auto-routing subcommands.

**Design**:
- Intercepts Bash commands matching `\bgit\s+stash\b`
- Allows `git stash list`, `git stash show`, `git stash branch` (safe conversions)
- Blocks bare `git stash` (save), `git stash pop`, `git stash drop`
- Returns exit code 2 with instructive message linking to governance doc
- Pairs with auto-route pattern: save changes to `stash/auto-<timestamp>-<reason>` branches

**Test coverage**: 4/4 smoke tests pass (list, show, branch, drop blocking)

**Doctrine**: `docs/qa/governance_stash_block.md`

---

### scripts/hooks/guard-git-worktree.ps1 (100 LOC)
**Purpose**: Guard `git worktree remove --force` on high-risk branches.

**Design**:
- Intercepts `git worktree remove --force` (and `-f` shorthand)
- Allows non-force removes (Git refuses if worktree has uncommitted changes — safe)
- Blocks force-removes on branches matching risk prefixes: `fix/`, `feat/`, `safety/`, `stash/`, `merge/`, `release/`, `patch/`, `hotfix/`
- Allows force-removes on `agent-*` paths (automated cleanup, by design)
- Returns exit code 2 with safety checkpoint message

**Test coverage**: 4/4 smoke tests pass (risk prefix blocks, agent-path allows, non-force allows)

**Doctrine**: `docs/qa/governance_worktree_guard.md`

---

## Hook Chain Configuration

**Status**: Hooks exist and tested. `.claude/settings.json` **PreToolUse configuration is NOT YET IN PLACE** (factual gap vs. MEMORY.md claim).

**Current settings.json hookage**:
- `SessionStart`: MCP startup script
- `UserPromptSubmit`: MCP startup script
- `PostToolUse[Edit|Write]`: dotnet format

**Next step (v0.26.0)**: Add PreToolUse[Bash] array to settings.json with both hooks sequenced:
```json
"PreToolUse": [
  {
    "matcher": "Bash",
    "hooks": [
      { "type": "script", "path": "scripts/hooks/block-git-stash.ps1" },
      { "type": "script", "path": "scripts/hooks/guard-git-worktree.ps1" }
    ]
  }
]
```

---

## CLAUDE.md Integration Status

**Governance docs cross-referenced**: YES (both feedback files mentioned in MEMORY.md)  
**CLAUDE.md sections added**: NO — governance narrative not yet embedded in main agent contract

**Recommended additions** (not yet done):
- Line ~96: "Git Stash Auto-Routing" section + reference to governance_stash_block.md
- Line ~118: "Git Worktree Guard" section + reference to governance_worktree_guard.md
- Line ~TBD: feedback_no_verify_forbidden reference (queued for v0.26.0)

---

## Lessons Distilled

| Lesson | Application |
|--------|-------------|
| **Concurrent agents under pressure damage in-flight work** | Both stash + worktree incidents involved cleanup race conditions; hooks are the structural defense |
| **Documentation alone is insufficient** | Feedback .md files establish policy; hooks enforce it asynchronously |
| **Conflicting agent reports are normal** | Orchestrator must maintain ground-truth log; verifier agents spot-check branches/logs |
| **Branches survive git worktree removal** | Force-remove only deletes the worktree checkout; branch + commits remain recoverable on origin |
| **Preventive hooks > reactive fixes** | block-no-verify queued before any real `--no-verify` incident (ounce of prevention) |

---

## Forward Plan (v0.26.0)

- [ ] Add PreToolUse[Bash] hooks to `.claude/settings.json`
- [ ] Land `block-no-verify.ps1` hook (prevents `git commit --no-verify` / `git push --no-verify`)
- [ ] Audit `.claude/settings.json` quarterly for hook drift
- [ ] Consider `block-git-push-force.ps1` (redundant with Git Safety Protocol in CLAUDE.md, but structural enforcement desirable)

---

## Metadata

**Hook files**: `scripts/hooks/block-git-stash.ps1` (76 LOC), `scripts/hooks/guard-git-worktree.ps1` (100 LOC)

**Governance docs**: `docs/qa/governance_stash_block.md`, `docs/qa/governance_worktree_guard.md`

**Related memory feedback**: `feedback_never_delete_repo_artifacts.md`, `feedback_stash_auto_route_to_branch.md`, `feedback_worktree_boundary.md`
