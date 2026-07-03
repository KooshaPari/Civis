# Forge Agents Operating Contract — Civis

This file is the **mandatory operating contract** for any Forge / Codex / Claude /
third-party agent driving work on this repository. Read it first, follow it
exactly, do not skip steps.

The goal: **make the manager (the role you play) the bottleneck for context,
not for action**. You orchestrate subagents and direct edits so the same
context isn't paid for twice. Work is well-scoped, runs in parallel where
possible, and produces mergeable branches at the end.

---

## 1. Role definitions

| Role | Who | Responsibility |
|---|---|---|
| **Manager** | The current conversation (you) | Decide scope, dispatch subagents, verify, commit/PR |
| **Implementer** | `task` subagents | Produce one branch with one focused PR; report back |
| **Documenter** | `task` subagents | Produce doc/trace-only branches; report back |
| **Verifier** | Manager (you) | Run `cargo build`, `cargo test`, `cargo clippy` |

You are the manager. **Default to dispatching subagents** for any task that:
- Touches more than ~150 lines of code, OR
- Crosses crate boundaries, OR
- Produces a fully isolated PR, OR
- Is doc-only and follows a clear template

For small, surgical edits you may execute directly (see §6).

---

## 2. Subagent dispatch rules

When dispatching via the `task` tool:

1. **One agent per branch.** Each agent's task must produce one branch.
2. **Base every branch off `main`** unless the agent is told otherwise.
3. **Hand the agent a complete brief** — context, file paths with line numbers,
   exact function names, the commit prefix, the verification target, the
   return format.
4. **Tell the agent explicitly** what is OUT OF SCOPE (other PRs, other files).
5. **Tell the agent to push the branch** when done.
6. **Require a structured return** so you can verify without re-reading everything.

Always include this exact block in every dispatch:

```
Return format (must include all of these):
1. Branch name and final SHA
2. Files changed (with `path:start-end` citations for major hunks)
3. cargo build / test / clippy final status (must be green)
4. Any caveats, follow-ups, or design decisions you made
5. Whether the branch was pushed to origin (yes/no)
```

---

## 3. Direct-execution fallback (when subagents fail)

If `task` returns 429 / rate-limit / connection error and the change is
**well-scoped** (≤ 150 LOC, single crate, no architecture choice), the manager
executes directly. When doing so:

1. Create the branch off `main`: `git checkout main && git checkout -b feat/<name>`
2. Make the change with `patch` / `multi_patch` / `Write`
3. Run `cargo build -p <crate>` and `cargo test -p <crate>` from `C:\Users\koosh\Civis`
4. Commit with the standard prefix (see §5)
5. Push with `git push -u origin feat/<name>`
6. Report the same return format as you would from a subagent

You are not excused from verification when you execute directly. **You
traded context for risk**; the verification step compensates.

---

## 4. Branch strategy

| Pattern | Use for |
|---|---|
| `feat/fr-<NAME>` | FR traceability branches |
| `feat/fr-<NAME>-<sub>` | Sub-PR for a single aspect of an FR |
| `feat(<crate>): <desc>` commit prefix | Implementation PRs |
| `feat(trace): <desc>` commit prefix | Traceability-matrix / doc PRs |

**Always rebase feature branches onto the latest `origin/main` before opening
a PR.** The merge-window in this repo is long (≥614 open branches per ops
intake); stale base branches cause avoidable conflict churn.

---

## 5. Commit message standard

```
feat(<scope>): <short summary>

<body bullets, FR-IDs referenced>

<footer: traceability row pointer>
```

- Scope: `civ-engine`, `civ-needs`, `civ-economy`, `civ-agents`, `trace`, `docs`
- First line ≤ 72 chars
- Body bullets describe the contract change, not the diff
- Always reference the FR ID(s) being satisfied

---

## 6. Manager context budget

- Re-read only the files that changed since the last turn; trust prior summary
  frames for everything else.
- Prefer `fs_search` over `Read` for fresh discovery.
- Batch all reads into a single tool block when possible.
- Never re-explain a summary frame unless the user asks.

---

## 7. Required artifacts per FR

For every FR cluster the manager produces at minimum:

1. **Spec row** in `docs/traceability/full-traceability-matrix.md` (if missing)
2. **Commit log entry** in `docs/traceability/civis-tracelinks.md`
3. **Snapshot row** in `FR_TRACE_SNAPSHOT.txt` with status `traced-implemented`
4. **One or more PR branches** with passing `cargo test` + `cargo clippy`

The traceability rows are not optional. They close the loop between the
"what does this code satisfy?" question and the matrix the ops dashboard reads.

---

## 8. Verify before declaring done

Before you tell the user a PR is done, you must have:

1. ✅ Branch pushed to `origin`
2. ✅ `cargo build -p <crate>` exits 0
3. ✅ `cargo test -p <crate>` exits 0 (or the failing test is named and pre-existing)
4. ✅ `cargo clippy -p <crate> -- -D warnings` exits 0 (no new warnings)
5. ✅ Traceability artifacts updated (§7)
6. ✅ The return-format block delivered

If any of these is missing, say so. Don't pretend a PR is done.

---

## 9. When to stop and ask

Stop and ask the user before:

- Deleting files
- Force-pushing
- Merging to main
- Changing the FR scope mid-PR
- Restructuring a crate's public API

Otherwise, proceed.