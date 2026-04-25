# AgilePlus Methodology Specification for DINOForge

**Status:** Active
**Owner:** DINOForge Agent Orchestration
**Repository:** `kooshapari/agileplus`
**Integration Point:** `docs/specs/` → AgilePlus stories

---

## Overview

**AgilePlus** is the spec-driven development engine used for DINOForge project management. It provides user story management, sprint tracking, spec management, and roadmap visualization as a lightweight PM dashboard.

This document describes how AgilePlus methodology is applied specifically to the DINOForge (Dino) repository.

---

## AgilePlus in the Dino Ecosystem

### Core Philosophy

AgilePlus operationalizes five complementary development methodologies in the Dino codebase:

| Methodology | Application in Dino |
|-------------|---------------------|
| **SDD** (Spec-Driven Development) | JSON/YAML schemas drive the development pipeline; schemas are source-of-truth for data shapes |
| **BDD** (Behavior-Driven Development) | Acceptance criteria defined in specs before implementation; feature behavior described in plain language |
| **TDD** (Test-Driven Development) | Unit tests for all public API surfaces; 1,017+ tests validate behavior before deployment |
| **DDD** (Domain-Driven Design) | Bounded contexts: Warfare, Economy, Scenario, UI domains with explicit ownership |
| **ADD** (Agent-Driven Development) | Fully agent-authored codebase; all contributions via autonomous agents |
| **CDD** (Contract-Driven Development) | Schemas as contracts between packs and engine; breaking changes require migration |

### AgilePlus Dashboard

- **Location:** `C:\Users\koosh\agileplus`
- **Launch:** `cd C:\Users\koosh\agileplus && bun run dev`
- **Purpose:** User stories, sprint tracking, spec management, roadmap visualization
- **Spec Mapping:** Specs in `docs/specs/` map to AgilePlus stories

---

## Spec Management Workflow

### Spec File Structure

All specs live in `docs/specs/` and follow a standardized format:

```
docs/specs/
  SPEC-001-example.md       # Numbered spec documents
  SPEC-002-another.md
  ...
```

### Spec-to-Story Mapping

Each spec document contains metadata that maps to AgilePlus:

```markdown
---
spec_id: SPEC-001
title: Feature Name
user_story: "As a [role], I want [feature] so that [benefit]"
sprint: Q2-2026-W01
status: implemented
---
```

### Spec Lifecycle

```
Draft → In Review → Implemented → Verified
  ↑______________|              ↓
        Rework                  Released
```

---

## Sprint Integration

### Sprint Structure

Sprints are organized by week within quarters:

```
Q2-2026
├── W01 (Mar 24-28)
├── W02 (Mar 31 - Apr 4)
├── W03 (Apr 7-11)
└── W04 (Apr 14-18)
```

### Sprint Ceremonies (Automated)

| Ceremony | Automation | Frequency |
|----------|-----------|-----------|
| Sprint Planning | Agents read specs, self-assign beads | Bi-weekly |
| Daily Standup | `gt_status` updates from each agent | Daily |
| Review | Refinery merges reviewed branches | Continuous |
| Retrospective | `docs/sessions/` session logs analyzed | Bi-weekly |

---

## Agent Collaboration Under AgilePlus

### Work Item Flow

```
AgilePlus Story
      ↓
Convoy Bead (gt:convoy label)
      ↓
Polecat Agent Hook (gt_bead_status: in_progress)
      ↓
Implementation (spec → code → test)
      ↓
Commit + Push + gt_done
      ↓
Refinery Review (gt_request_changes or merge)
      ↓
Sprint Completion Logged
```

### Bead Types

| Type | Purpose | Example |
|------|---------|---------|
| `issue` | Single task implementation | "Add AgilePlus spec to Dino" |
| `convoy` | Cross-repo coordination | "AgilePlus + Kilo Specs: Dino" |
| `feature` | New feature development | "Add asset swap system" |

### Agent Commands

| Command | Purpose |
|---------|---------|
| `/status` | Project health summary |
| `/validate` | Validate all packs |
| `/test` | Run all tests |
| `/build-all` | Build all solutions |

---

## Quality Gates

All work items must pass these gates before `gt_done`:

1. **Test Gate:** `dotnet test src/DINOForge.sln` — 0 failures
2. **Lint Gate:** `dotnet format --verify-no-changes`
3. **Spec Gate:** Implementation matches spec acceptance criteria
4. **Documentation Gate:** Public API docs updated (XML comments)

---

## Roadmap Visualization

AgilePlus provides visual roadmap tracking:

- **Milestone Board:** M0-M14 milestones tracked with completion status
- **Sprint Burndown:** Story points completed per sprint
- **Spec Coverage:** Specs with/without implementation
- **Agent Activity:** Real-time agent work status via `gt_status`

### Current Milestone Status

| Milestone | Description | Status |
|-----------|-------------|--------|
| M0 | Reverse-Engineering Harness | Done |
| M1 | Runtime Scaffold | Done |
| M2 | Generic Mod SDK | Done |
| M3 | Dev Tooling | Done |
| M4 | Warfare Domain | Done |
| M5 | Example Packs | Done |
| M6 | In-Game Mod Menu + HMR | Done |
| M7 | Installer + Universe Bible | Done |
| M8 | Runtime Integration | Done |
| M9 | Desktop Companion | Done |
| M10 | Fuzzing | Done |
| M11 | Test Coverage | Done |
| M12 | Pack Submodule Management | Done |
| M13 | Asset Browser + Mod Manager | Done |
| M14 | Asset Library & Catalog | Done |

---

## AgilePlus + Kilo Gastown Integration

Dino is a **Kilo Gastown** rig (town `78a8d430`), meaning:

- **Rig ID:** `6c6d4555-91e8-4f06-a974-018cf3e766d2`
- **Town:** `78a8d430-a206-4a25-96c0-5cd9f5caf984`
- **Coordination:** Convoy beads coordinate work across multiple repos
- **Agent Pool:** Multiple polecat agents work in parallel via `gt_sling`/`gt_sling_batch`

### Convoy Workflow

```
AgilePlus Story
      ↓
Convoy (multi-repo work item)
      ↓
Each repo's polecat processes its portion
      ↓
All portions complete → Story done
```

---

## Anti-Patterns (Blacklist)

These patterns are prohibited under AgilePlus methodology:

| Pattern | Prohibition |
|---------|------------|
| "Please launch the game" | User interaction required — use autonomous launch |
| "Click the X button" | Manual interaction — use GameClient API |
| "Test it yourself" | No proof — generate autonomous video proof |
| "You should do X" | Doesn't delegate — create subagent for X |
| "Let me know if it works" | Requires user feedback — auto-check via health API |

---

## See Also

- [CLAUDE.md](./CLAUDE.md) — Project governance and operational rules
- [AGENTS.md](./AGENTS.md) — Agent roster and domain ownership
- [docs/specs/](./specs/) — Spec document directory
- [docs/plans/](./plans/) — Plan document directory
- AgilePlus Dashboard: `C:\Users\koosh\agileplus`

---

**Spec Owner:** DINOForge Agent Orchestration
**Last Updated:** 2026-03-31
**Next Review:** 2026-04-14