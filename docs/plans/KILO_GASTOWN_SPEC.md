# Kilo Gastown Methodology Spec

**Status:** ACTIVE
**Rig ID:** `6c6d4555-91e8-4f06-a974-018cf3e766d2`
**Town ID:** `78a8d430-a206-4a25-96c0-5cd9f5caf984`
**Owner:** All DINOForge Agents
**Applies to:** All bead and convoy operations in this rig

---

## Overview

**Kilo Gastown** is the agent orchestration layer for this rig. It provides a structured system for distributing, tracking, and merging work across multiple autonomous agents operating in parallel. All methodology mechanics described here are powered by Gastown tooling (gt_* tools available in Claude Code).

This document explains how Kilo Gastown mechanics apply specifically to the DINOForge project.

---

## Rig Topology

| Field | Value |
|-------|-------|
| Rig ID | `6c6d4555-91e8-4f06-a974-018cf3e766d2` |
| Town ID | `78a8d430-a206-4a25-96c0-5cd9f5caf984` |
| Primary branch | `main` |
| Agent identity format | `Polecat-<N>-polecat-<rig-id>@<town-id>` |

---

## Core Concepts

### Beads

A **bead** is the atomic unit of work in Kilo Gastown. Each bead represents a single, focused task assigned to one agent.

**Bead types:**

| Type | Description |
|------|-------------|
| `issue` | A single task (one agent, one deliverable) |
| `merge_request` | A review request for a completed branch |
| `convoy` | A batch of related beads grouped under a feature umbrella |

**Bead lifecycle:**

```
open → in_progress → in_review → closed
```

| State | Meaning |
|-------|---------|
| `open` | Queued, not yet started |
| `in_progress` | An agent is actively working on it |
| `in_review` | Work complete, pushed to review queue |
| `closed` | Merged or rejected |

**Bead tools:**

- `gt_sling` — Dispatch a single bead to an agent (creates `issue` bead)
- `gt_bead_status` — Inspect current state of any bead by ID
- `gt_bead_close` — Mark a bead as completed

### Convoys

A **convoy** groups related beads under a shared feature branch. Convoys enable parallel agent work on a larger feature while maintaining a coherent merge target.

**Convoy naming convention:**
```
convoy/<feature-name>/<convoy-id>/head
```

**Convoy tools:**

- `gt_sling_batch` — Dispatch multiple beads at once (creates a convoy)
- `gt_list_convoys` — List all convoys with their status and progress
- Convoys track `ready_to_land` metadata — when set, the convoy branch can be merged

**Example convoy for this rig:**
```
convoy/agileplus-kilo-specs-dino/381d5195/head
```

---

## Merge Modes

Kilo Gastown supports two merge strategies:

### Review-Then-Land (Default)

The agent pushes their branch and calls `gt_done`. The Refinery reviews the branch. If approved, it lands on `main`. If rework is needed, the bead returns to `in_progress` with feedback.

```
Agent pushes branch → gt_done → bead enters in_review → Refinery approves → lands on main
```

### Review-and-Merge

The Refinery reviews and merges in one step. Used when the branch is pre-verified (e.g., passing CI, following conventions).

```
ready_to_land=1 → Refinery merges without additional review cycle
```

**Who decides which mode applies:**
- Beads with `ready_to_land: 1` metadata use review-and-merge
- All other beads use review-then-land

---

## How Kilo Applies to DINOForge

### Agent Identity in This Rig

Every agent in this rig is a **polecat**:

```
Polecat-<N>-polecat-<rig-id>@<town-id>
```

Current hooked bead identifies you by your `agent_bead_id`. Use `gt_prime` to re-orient at the start of any session.

### Dispatching Work

When you receive a bead via `gt_prime`:

1. **Claim it** — the bead is already hooked to you; do not re-assign
2. **Explore** — read CLAUDE.md and relevant source files before writing
3. **Implement** — make focused, working commits; push after each
4. **Verify** — run `dotnet build`, `dotnet test`, `dotnet format --verify-no-changes`
5. **Done** — call `gt_done` with your branch name

### Delegating Sub-Tasks

If a bead requires work outside your domain, use `gt_mail_send` to notify the responsible agent. For batched work across multiple agents, use `gt_sling_batch` to create a convoy and sling individual beads to each agent.

### Pre-Submission Gates

Before calling `gt_done`, run all applicable quality gates:

| Gate | Command |
|------|---------|
| Build | `dotnet build src/DINOForge.sln` |
| Test | `dotnet test src/DINOForge.sln` |
| Format check | `dotnet format src/DINOForge.sln --verify-no-changes` |
| Pack validation | `dotnet run --project src/Tools/PackCompiler -- validate packs/` |

### Tracking Convoys

To see all active convoys and their status:

```
gt_list_convoys
```

This returns all open convoys with their feature branch names, bead counts, and `ready_to_land` status.

---

## Bead-to-DINOForge Domain Mapping

When slinging beads for DINOForge work, use the domain ownership map to route correctly:

| Domain | Agent Role | Files |
|--------|-----------|-------|
| ECS bridge, BepInEx | `runtime-specialist` | src/Runtime/ |
| Registry, SDK, schemas | `sdk-architect` | src/SDK/ |
| Warfare domain | `warfare-designer` | src/Domains/Warfare/ |
| Content packs | `pack-builder` | packs/ |
| CLI / tooling / MCP | `toolsmith` | src/Tools/ |
| Tests, CI/CD | `qa-engineer` | src/Tests/ |
| Documentation | `docs-curator` | docs/ |

---

## Coordination Patterns

### Starting a Session

```
gt_prime          # Get hooked bead, open beads, mail
gt_status "..."   # Emit status to dashboard
```

### Announcing Work

```
gt_status "Writing unit tests for PackCompiler"   # Phase transition
gt_checkpoint      # After significant progress
```

### Requesting Help

```
gt_mail_send      # Send a typed message to another agent
gt_escalate       # Create an escalation bead for blocked issues
```

### Finishing Work

```
git add . && git commit -m "feat(sdk): add UnitRegistry wildcard query
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
git push origin <your-branch>
gt_done --branch <your-branch>
```

---

## Integration with Existing AGENTS.md

Kilo Gastown mechanics are **orthogonal** to the domain ownership rules in `AGENTS.md`. The bead system provides the orchestration layer; the Agent Roster provides the ownership layer. When working a bead:

1. Follow Kilo Gastown workflow (this spec) for dispatch, tracking, and merge
2. Follow AGENTS.md ownership rules for what files you may modify
3. Follow CLAUDE.md for code style, build commands, and governance

---

## Quick Reference

| Need | Tool |
|------|------|
| Get current context | `gt_prime` |
| Check bead status | `gt_bead_status <bead_id>` |
| Dispatch single bead | `gt_sling` |
| Dispatch batch (convoy) | `gt_sling_batch` |
| List all convoys | `gt_list_convoys` |
| Send coordination message | `gt_mail_send` |
| Emit status update | `gt_status "..."` |
| Write crash-recovery data | `gt_checkpoint` |
| Signal work complete | `gt_done --branch <name>` |
| Report blocked issue | `gt_escalate` |

---

**Spec Owner:** Kilo Gastown (rig orchestration)
**Last Updated:** 2026-03-31
**Related:** `AGENTS.md`, `CLAUDE.md`, `docs/plans/PLAN-agent-tooling-evolution.md`
