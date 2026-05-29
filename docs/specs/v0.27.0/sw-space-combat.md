# SW-013: SW Space Combat (R&D / Stretch)

**Status**: Proposed (Stretch)
**AgilePlus WP State**: planned
**Sequence**: 13
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**AgilePlus Feature Slug**: epic-027-full-conversion
**Sprint**: 5 — Stretch
**Story Points**: 13
**Priority**: P3 — Stretch; not a v0.27.0 release blocker
**File Scope**:
  - `docs/specs/v0.28.0/sw-space-combat-design.md` (R&D deliverable)
  - `src/Runtime/SpaceCombat/SpaceCombatHudPanel.cs` (if prototype approved)
  - `packs/warfare-starwars/pack.yaml` (optional `space_combat:` block)
  - `src/Tests/SpaceCombatTests.cs` (if prototype implemented)
**Depends On**: [SW-006]
**Requirements**: EPIC-027-NFR-001, EPIC-027-NFR-002, EPIC-027-NFR-005

---

## User Story

As a **Star Wars mod player**, I want an optional space combat layer — orbital battles
between capital ships that influence ground reinforcements — so that the Clone Wars mod
has a second strategic plane distinct from the ground RTS.

## Background

This is a research spike. DINO's ECS has no concept of a "space" or "orbital" plane. Adding
one requires either:
- A separate scene managed by DINOForge (complex, risk of save-file side effects).
- A HUD-level abstraction that simulates "space combat" as a resource-affecting minigame
  without requiring a real second scene (simpler, safer).

The R&D spike determines which approach is feasible before committing implementation points.

## Acceptance Criteria

### Scenario 1 — R&D spike deliverable (minimum for this sprint)

**Given** Sprint 5 completes,
**When** the team reviews the space combat R&D document,
**Then** `docs/specs/v0.28.0/sw-space-combat-design.md` exists with:
- A clear recommendation: HUD-minigame approach vs. separate-scene approach.
- A risk assessment for each approach.
- A revised story-point estimate for full implementation.
- A list of DINO ECS components / scene names that would need to be involved.

### Scenario 2 — Prototype orbital HUD overlay (if R&D passes)

**Given** the HUD-minigame approach is chosen,
**When** a "space battle" event triggers in the SW campaign,
**Then** a DINOForge-owned HUD panel appears showing two capital ships (Republic Venator vs.
CIS Lucrehulk) with health bars and an "Engage" button, resolving after 30 seconds into
a reinforcement bonus or penalty.

### Scenario 3 — Ground campaign unaffected if space combat is disabled

**Given** the `sw_space_combat_enabled: false` config entry is set,
**When** a SW campaign runs,
**Then** no space combat panel appears and ground combat is unchanged.

## Functional Requirements (if beyond R&D)

| ID | Requirement |
|----|-------------|
| F-01 | R&D document produced before any implementation begins. |
| F-02 | Space combat declared in `pack.yaml` under an optional `space_combat:` block. |
| F-03 | HUD panel (if approach chosen) uses ThemeEngine palette and is a `DINOForge_`-prefixed GameObject. |
| F-04 | Space combat outcome affects only reinforcement rate, not direct unit stat. |
| F-05 | Feature flag: `[SpaceCombat] enabled = false` in BepInEx config disables the feature entirely. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | R&D spike not to exceed 2 agent-days. If approach unclear, defer to v0.28.0. |
| N-02 | No changes to DINO's ECS Fight or ResourceDelivery systems. |

## Engine Quirks / Dependencies

- DINO has 6 ECS worlds; confirm "space" is not an existing world via `dinoforge dump`.
- HUD panels follow Pattern #235 (EventSystem guard before GraphicRaycaster).
- ThemeEngine from SW-006 must be active for styled panels.
- This story has no hard dependencies — it is blocked only by R&D decision.

## Definition of Done (Sprint 5 minimum)

- [ ] R&D document `docs/specs/v0.28.0/sw-space-combat-design.md` produced.
- [ ] If prototype implemented: HUD overlay appears and resolves without crash (screenshot proof).
- [ ] Feature flag disables space combat cleanly.
- [ ] `dotnet test` green.

## Evidence Requirements

| Requirement ID | Evidence Type | Artifact Path Pattern | Transition Gate |
|----------------|---------------|-----------------------|-----------------|
| EPIC-027-NFR-001 | ManualAttestation | `docs/specs/v0.28.0/sw-space-combat-design.md` exists with recommendation, risk assessment, revised estimate, and relevant ECS component list | Implementing → Validated |
| EPIC-027-NFR-002 | CiOutput | `dotnet test` green; no regressions from v0.26.0 (R&D spike adds 0 production code — CI gate is pass-through) | Implementing → Validated |
| EPIC-027-NFR-005 | CiOutput | CI build log (netstandard2.0 TFM check; prototype panel, if implemented, does not add compile-time TMPro refs) | Implementing → Validated |
| SW-013 | ManualAttestation | If prototype implemented: `docs/proof/judge-receipts/SW-013-space-combat-prototype.md` (HUD overlay appears; resolves after 30s; feature flag disables cleanly — screenshot) | Implementing → Validated |
| SW-013 | ReviewApproval | PR URL (auto-detected from WorkPackage.pr_url) | Validated → Shipped |
| SW-013 | CiOutput | GitHub Actions run URL (dotnet test green) | Implementing → Validated |

## Related

- `docs/design/identity-starwars.md` (capital ship concept art)
- SW-006 (ThemeEngine — HUD overlay theming)
- SW-011 (aerial combat — related tactical layer)
