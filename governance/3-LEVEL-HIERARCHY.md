# 3-Level Agent Swarm Hierarchy (Phenotype Org Standard)

> **Status:** Canonical pattern (2026-06-16). Built on top of the existing `~/.claude/plugins/dispatch/` plugin — no new dispatch plumbing required.

## Summary

The Phenotype fleet operates on a **3-level agent swarm hierarchy**:

```
Level 1 (Opus)   — coordinator
   ↓ delegates
Level 2 (Sonnet) — team lead
   ↓ dispatches via dispatch plugin (CLI/MCP/skill)
Level 3 (Haiku)  — dispatch-worker leaves (run docs/audits/scripts/score.py)
```

The `dispatch` plugin (`~/.claude/plugins/dispatch/plugin.json`) is the **canonical leaf runner**. This governance doc defines the team-level pattern on top of it.

## Why a 3-level hierarchy

- **Single-level haiku dispatch** flattens the orchestration; Sonnet is wasted.
- **2-level (Opus + Sonnet)** under-uses Opus for synthesis work.
- **3-level** keeps Opus on strategy, Sonnet on worktract ownership, Haiku on high-volume leaf work.
- Each level has a clear role and a clear escalation path.

## The dispatch plugin (existing, do not re-build)

The `dispatch` plugin already provides everything for leaves:
- **MCP server**: `dispatch` registered in `~/.claude/settings.json` as `mcpServers.dispatch`
- **Skill**: `dispatch` in the harness skill list (see `Skill({skill: "dispatch"})`)
- **CLI**: `dispatch-worker` at `~/.local/bin/dispatch-worker`
- **Slash command**: `/dispatch` at `~/.claude/commands/dispatch.md`
- **Pre-flight**: `curl -sf http://localhost:20128/v1/models` to check omniRoute
- **Default model**: `Main` (omniroute-main) on omniRoute provider

## Levels

### Level 1 — Opus coordinator

- Reads DAG-SSOT (`/Users/kooshapari/CodeProjects/Phenotype/repos/.remember/NEXT-STEPS-DAG.md`)
- Reads backlog head (`/Users/kooshapari/CodeProjects/Phenotype/repos/docs/audits/BACKLOG.md`, first 200 lines only)
- Splits work into team-sized chunks (5-30 repos of one language)
- Dispatches each chunk to a Sonnet team lead via the `Agent` tool
- Aggregates team-lead output

**Hard rules:**
- Never do leaf work yourself.
- Never write to a per-repo worktree directly.
- Never push, force-push, or reset any branch.
- Reserve Opus for synthesis-critical: writing contracts, deciding strategy, aggregating team output.

### Level 2 — Sonnet team lead

- Receives a chunk of N repos from the coordinator
- Dispatches each repo to a `dispatch-worker` (Haiku tier)
- Aggregates the JSON rows
- Detects regressions (pillar was >0, now 0)
- Reports back to the coordinator

**Default tract sizes:**
- Small (recommended first run): 5 repos, 1-3 leaves in parallel
- Medium: 10-20 repos
- Large: 30+ repos — split into sub-tracts first

**Hard rules:**
- Never push, force-push, or reset.
- Never modify the coordinator's branch.
- If 4+ leaves fail, escalate to the coordinator (do not keep retrying).

### Level 3 — Haiku / kimi / minimax dispatch-worker

- Receives ONE repo name from the team lead
- Runs `python3 /Users/kooshapari/CodeProjects/Phenotype/repos/docs/audits/scripts/score.py --repo <name>`
- Emits one JSON row to stdout
- Stops

**Hard rules:**
- One repo per invocation.
- No file writes outside `/tmp`.
- No git operations.
- If the script hangs >60s, kill and return `{"error": "timeout", "mean": null}`.

## Worktree pattern

| Level | Path | Branch |
|---|---|---|
| Coordinator | `repos/.worktrees/audit-30pillar/` | `audit/30-pillar-fleet` |
| Team lead | `repos/.worktrees/audit-30pillar/<lang>/` | `audit/30-pillar-fleet/<lang>` |
| Leaf | ephemeral (no worktree) | n/a |

The leaf does no edits, so it does not need a worktree.

## Default policy

```json
{
  "min_concurrent_leaves": 10,
  "min_worker_tier": "haiku",
  "opus_reserved_for": "synthesis-critical only",
  "default_provider": "omniroute",
  "default_model": "Main"
}
```

## Failure modes

| Failure | Action |
|---|---|
| omniRoute down | `dispatch` plugin pre-flight restarts it (`nohup omniroute --no-open`) |
| 4+ leaves fabrication | Team lead marks items `skip: fabrication`, escalates |
| Leaf 60s timeout | Return `{"error": "timeout"}`, team lead retries |
| Coordinator 4+ teams report 0 repos | Drop the task, log, escalate to user |

## Adoption

- Tier 1 (mature, score ≥ 0.50): 5 repos — fully adopt
- Tier 2 (working, 0.30-0.50): 30 repos — adopt as a tract
- Tier 3 (early stage, <0.30): 76 repos — adopt via the audit worktree

## Related

- Skill: `~/.claude/skills/three-level-swarm/SKILL.md`
- Plugin: `~/.claude/plugins/dispatch/plugin.json`
- Memory: `~/.claude/memory/MEMORY.md` (3-level entries)
- Trail (current): the user can opt into a `~/.claude/memory/three-level-trail.md` if desired
- Companion: NOT to be confused with `DietrichGebert/ponytail` (different concept, different name)
