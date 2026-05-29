# SW-001: Native Mods Page Visible

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 1 — Foundation
**Story Points**: 8
**Priority**: P0 — Sprint blocker

---

## User Story

As a **mod player**, I want clicking the "Mods" button in DINO's main menu to open a visible,
native-styled mods page — not a blank overlay — so that I can see my installed packs, their
status, and toggle them without memorizing the F10 hotkey.

## Background

The MODS button (injected by `NativeMenuInjector`, confirmed working in iter-146 commit
`cafd2b70`) currently opens `ModMenuPanel` — a floating DFCanvas overlay. Two problems
documented in `SPEC-fix-mods-button-ux.md`:

1. The overlay is visually distinct from DINO's native settings-page style (jarring context switch).
2. Under certain pack states the panel renders blank (labels missing, scroll area empty).

This story makes the Mods page feel like a first-class DINO screen by fixing both issues.
It is a Sprint 1 blocker because the Mods page is the primary mod-management surface for all
v0.27.0 UX goals.

## Acceptance Criteria

### Scenario 1 — Mods button opens a styled, populated page

**Given** `warfare-starwars` and `warfare-modern` packs are installed and `dinoforge status`
reports both Active,
**When** the player clicks the "Mods" button in DINO's main menu,
**Then** a mods page opens that:
- fills the same screen region as DINO's native Settings/Options page (not a floating overlay),
- lists every loaded pack with display name, version, and active/inactive status badge,
- matches the active total-conversion palette or DINOForge default theme,
- is navigable by mouse click.

### Scenario 2 — No blank state under any pack configuration

**Given** any combination of packs (including zero packs beyond the built-in default),
**When** the Mods page opens,
**Then** the page is never blank — it shows at minimum the DINOForge header and a
"No additional packs loaded" message when no user packs are present.

### Scenario 3 — Page closes cleanly and returns input to menu

**Given** the Mods page is open,
**When** the player clicks "Back" or presses Escape,
**Then** the Mods page closes and all native main menu buttons are clickable
(no input swallowed, no EventSystem focus lost).

### Scenario 4 — Hover effects on the Mods button match native buttons

**Given** the Mods button is visible in the main menu,
**When** the player moves the cursor over the Mods button,
**Then** the hover visual effect matches adjacent native DINO buttons
(confirmed against live `dinoforge ui-tree` dump — do not hardcode without verifying).

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | Mods page renders within 500 ms of button click on a 60 FPS host. |
| F-02 | Pack list entries include: display name, version, type badge, active/disabled badge. |
| F-03 | Page layout uses `ResolvedTheme` palette (primary / secondary / text colors). |
| F-04 | Page is scrollable when pack count exceeds visible area. |
| F-05 | Mods button hover state must not block raycasts to sibling buttons (Pattern #235). |
| F-06 | Mods page does not duplicate entries if `NativeMenuInjector` re-scans while page is open. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | All UGUI construction and canvas walks execute on the Unity main thread. |
| N-02 | Page `GameObject` named `DINOForge_ModsPage` so `CanvasWalker` skips it. |
| N-03 | Page is destroyed (not hidden) on close to avoid memory accumulation. |
| N-04 | No Harmony patches on DINO's UI systems — `EventTrigger` / `Button.onClick` only. |

## Engine Quirks / Dependencies

- `MonoBehaviour.Update()` never fires — page construction is one-shot inside the button
  click handler (main thread, `EventTrigger.PointerClick`).
- Confirm DINO's Settings page `RectTransform` anchor via `dinoforge ui-tree` before sizing
  the Mods page.
- `NativeMenuInjector` hover fix (`SPEC-fix-mods-button-ux.md`) should land in the same
  sprint to avoid regressing iter-146 behavior.
- Graceful fallback to DINOForge default palette if ThemeEngine (SW-006) not yet active.
- Pattern #235: any transparent Image over buttons with `raycastTarget = true` kills all
  button clicks silently — guard every injected element.

## Definition of Done

- [ ] Clicking Mods button opens `DINOForge_ModsPage` (no blank state under any pack config).
- [ ] Pack list shows all loaded packs with correct name, version, and status.
- [ ] Hover effects match native DINO buttons (external screenshot proof).
- [ ] Closing page returns full input control to main menu.
- [ ] `dotnet test` green — unit tests for `ModsPageController` construction + pack entry population.
- [ ] In-game screenshot with Mods page open deposited in `docs/proof/`
  + external judge receipt in `docs/proof/judge-receipts/`.

## Related

- `SPEC-002-native-menu-injector.md`
- `SPEC-fix-mods-button-ux.md`
- `docs/design/main-menu-takeover.md §1.2`
- Pattern #235 (GraphicRaycaster without EventSystem guard)
- SW-006 (ThemeEngine — palette provider)
