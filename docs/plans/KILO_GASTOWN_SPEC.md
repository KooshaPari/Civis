# Kilo Gastown Methodology — DINOForge Rig Specification

**Status:** Active
**Town ID:** `78a8d430-a206-4a25-96c0-5cd9f5caf984`
**Rig ID:** `6c6d4555-91e8-4f06-a974-018cf3e766d2`
**Stack:** DINO game mod, C#/.NET, .NET 11 preview

---

## Overview

Kilo Gastown is a multi-agent orchestration system for coordinating parallel software engineering work across a shared git repository. It uses a "town" metaphor: a persistent rig where specialized agents ("polecats") work on independent beads (work items) simultaneously, coordinated through a shared bead pool and convoy-based feature branches.

This document describes the Kilo Gastown methodology as implemented in the DINOForge rig.

---

## Core Concepts

### Town (Rig)

A **town** (rig) is a persistent containerized environment hosting multiple concurrent agents. Each town has:
- A unique `town_id` for routing
- A shared git repository (worktree per agent)
- A bead pool of work items
- A set of specialized agent roles

**DINOForge Town:**
- Rig ID: `6c6d4555-91e8-4f06-a974-018cf3e766d2`
- Town ID: `78a8d430-a206-4a25-96c0-5cd9f5caf984`
- Stack: DINO game, C#/.NET, .NET 11 preview

### Beads (Work Items)

A **bead** is the atomic unit of work in Gastown. Each bead has:
- `bead_id` — unique identifier
- `type` — `issue`, `merge_request`, `convoy`, or `triage`
- `status` — `open`, `in_progress`, `in_review`, `completed`, `cancelled`
- `title` / `body` — description
- `assignee_agent_bead_id` — which agent owns it
- `priority` — `low`, `medium`, `high`, `critical`
- `metadata` — typed hash (convoy_id, feature_branch, etc.)
- Timestamps: `created_at`, `updated_at`, `closed_at`

**Bead Types:**
- `issue` — Feature request, bug, or task
- `merge_request` — Review submission from an agent
- `convoy` — Feature branch grouping multiple related beads
- `triage` — System health check (e.g., stuck agent detection)

### Convoys (Feature Branches)

A **convoy** groups related beads under a shared feature branch. Convoys ensure atomic delivery of multi-bead features and prevent merge conflicts by serializing dependent changes.

```
convoy_id: 381d5195-27f8-4843-9efe-62f11234815e
feature_branch: convoy/agileplus-kilo-specs-dino/381d5195/head
```

Agents working on convoy beads branch from the convoy's feature branch, not from `main`.

### Agent Roles

Each agent has a specialized domain:

| Role | Domain | Tools |
|------|--------|-------|
| `runtime-specialist` | ECS bridge, BepInEx | src/Runtime/ |
| `sdk-architect` | Registry, SDK, schemas | src/SDK/ |
| `warfare-designer` | Warfare domain, balance | src/Domains/Warfare/ |
| `pack-builder` | Content packs, YAML | packs/ |
| `toolsmith` | CLI tools, PackCompiler | src/Tools/ |
| `qa-engineer` | Tests, CI/CD | src/Tests/ |
| `docs-curator` | Documentation, VitePress | docs/ |

---

## GUPP Principle

**GUPP: Get Underway, Produce immediately, Pass the bead on.**

> "Work is on your hook — execute immediately. Do not announce what you will do; just do it. When you receive a bead (work item), start working on it right away. No preamble, no status updates, no asking for permission. Produce code, commits, and results."

GUPP is the core behavioral contract for every polecat:

1. **Hooked bead arrives** → begin immediately, no confirmation needed
2. **Produce** → write code, tests, commits; no status announcements
3. **Pass on** → call `gt_done` to push branch and submit for review

---

## Agent Lifecycle

### 1. Prime (`gt_prime`)

Called at session start to retrieve full context:
- Agent identity and role
- Hooked bead (current work item)
- Undelivered mail
- All open beads in the rig

### 2. Work

Execute the hooked bead's requirements:
- Read relevant files
- Implement changes
- Write tests
- Commit frequently (small, focused commits)

### 3. Checkpoint (`gt_checkpoint`)

Save crash-recovery state after significant milestones:
```json
{ "hooked_bead_id": "...", "status": "...", "file_created": "...", "commit_hash": "..." }
```

### 4. Quality Gates (Pre-Submission)

Before calling `gt_done`, run all required gates:
- `task quality` — code quality checks
- `dotnet test src/DINOForge.sln` — all tests pass
- `dotnet format --verify-no-changes` — no formatting issues

### 5. Done (`gt_done`)

Signal completion:
1. Push branch to remote
2. Call `gt_done` with branch name
3. Bead transitions to `in_review`
4. Refinery picks up for merge review

---

## Coordination Tools

### Message Passing

| Tool | Purpose |
|------|---------|
| `gt_mail_send` | Send persistent typed message to another agent |
| `gt_mail_check` | Read and acknowledge undelivered mail |
| `gt_nudge` | Real-time wake-up signal (delivered at agent's next idle moment) |

### Bead Management

| Tool | Purpose |
|------|---------|
| `gt_bead_status` | Read full details of any bead by ID |
| `gt_bead_close` | Mark a bead as completed (used by refinery) |
| `gt_done` | Push branch + submit bead for review |

### Work Delegation

| Tool | Purpose |
|------|---------|
| `gt_sling` | Delegate a bead to another agent in the rig |
| `gt_sling_batch` | Delegate multiple beads at once |

### Escalation

| Tool | Purpose |
|------|---------|
| `gt_escalate` | Create an escalation bead for blocked/stuck issues |
| `gt_triage_resolve` | Resolve a triage request (restart, close, escalate) |

### State

| Tool | Purpose |
|------|---------|
| `gt_status` | Emit plain-language status update to dashboard |
| `gt_checkpoint` | Write crash-recovery data to agent record |

---

## Convoy Pattern

Convoys group related beads under a single feature branch:

```
convoy/convoy-name/convoy_id/head
```

**Convoy lifecycle:**
1. Convoy bead created with `feature_branch` metadata
2. Agents branch from the convoy feature branch
3. All beads in convoy are developed in parallel
4. Convoy merges atomically when all beads complete

**Why convoys?**
- Atomic multi-bead feature delivery
- Prevents conflicting changes from parallel agents
- Provides clear feature branch boundaries

---

## Merge Modes

Agents have two merge strategies:

| Mode | Description |
|------|-------------|
| **Rebase** (default) | `git pull --rebase origin main` before starting; linear history |
| **Merge** | `git pull origin main` with merge commit; preserved history |

The DINOForge rig uses **rebase** by default to maintain clean linear history.

---

## Progress Tracking

### Dashboard Visibility

`gt_status` emits plain-language updates:
> "Writing unit tests for the API endpoints."
> "Fixing 3 TypeScript errors before committing."

These appear on the orchestration dashboard for teammates to follow.

### Checkpoint Data

`gt_checkpoint` stores JSON recovery data:
```json
{
  "hooked_bead_id": "146575f3-6e40-47dc-9adc-4318f1aa7016",
  "status": "in_progress",
  "file_created": "docs/plans/KILO_GASTOWN_SPEC.md",
  "commit_hash": "a1b2c3d"
}
```

On container restart, the next agent session reads this data and resumes from the checkpoint.

---

## DINOForge Integration

The DINOForge project uses Kilo Gastown for:

### Content Development
- **Pack authors** (pack-builder) create unit/building YAML definitions
- **SDK validation** (sdk-architect) ensures schema compliance
- **Integration tests** (qa-engineer) verify content loads in-game

### Feature Development
- **Domain specialists** implement new game systems
- **CLI tools** (toolsmith) build automation
- **MCP server** exposes 13 game tools to running game session

### Multi-Agent Scenarios
- **Convoy**: `convoy/methodology-dino` groups methodology docs
- **Convoy**: `convoy/agileplus-kilo-specs-dino` coordinates cross-repo spec work
- **Parallel beads**: multiple agents work on different domains simultaneously

---

## Anti-Patterns (Blacklist)

These patterns are prohibited:

| Pattern | Reason | Correct Approach |
|---------|--------|-----------------|
| "Please launch the game" | User interaction | Autonomous exe launch |
| "Click the X button" | Manual interaction | GameClient API |
| "Test it yourself" | No proof | Automated test + video |
| "Let me know if it works" | Requires user | Auto health-check |
| "Run this command manually" | User must act | Automate command |

---

## File Ownership

To prevent conflicts, each file has a designated owner:

| Layer | Owner | Files |
|-------|-------|-------|
| Runtime | runtime-specialist | src/Runtime/ |
| SDK | sdk-architect | src/SDK/ |
| Domain | warfare-designer | src/Domains/Warfare/ |
| Packs | pack-builder | packs/ |
| Tools | toolsmith | src/Tools/ |
| Tests | qa-engineer | src/Tests/ |
| Docs | docs-curator | docs/, CHANGELOG.md |

Agents must not modify files outside their domain without permission.

---

## Handoff Protocol

When completing work:
1. Run `dotnet test src/DINOForge.sln` — verify 0 failures
2. Run `dotnet format --verify-no-changes`
3. Update CHANGELOG.md [Unreleased] section
4. Commit with descriptive message + `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
5. `git push origin <branch>`
6. Call `gt_done` with branch name

---

## References

- **AGENTS.md** — DINOForge agent collaboration guide (roles, roster, coordination)
- **CLAUDE.md** — Governance, build commands, architecture
- **gt_prime** context — Full bead pool and agent state
- **Kilo Gastown MCP Tools** — 13 game tools for runtime interaction

---

**Spec Owner:** docs-curator
**Last Updated:** 2026-03-31
**Used By:** All DINOForge polecat agents
