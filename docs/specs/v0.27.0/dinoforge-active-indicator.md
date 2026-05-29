# SW-002: DINOForge Active Indicator

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 1 — Foundation
**Story Points**: 3
**Priority**: P1

---

## User Story

As a **mod player**, I want to see a DINOForge or active-mod indicator in the game window title
and/or window icon so that I can confirm at a glance — before the menu loads — that DINOForge
is loaded and which mod pack is active.

## Background

Currently there is no visible platform-level indicator that DINOForge has loaded. Players
rely on the injected "Mods" button appearing in the menu (which has a 5-second delay per
SPEC-002 F-01) or checking `BepInEx/LogOutput.log`. An immediate window-level signal removes
ambiguity during testing and regular play.

## Acceptance Criteria

### Scenario 1 — Window title shows DINOForge platform marker

**Given** DINOForge `Plugin.Awake()` has executed,
**When** the game window is visible (any scene),
**Then** the window title contains "| DINOForge v{version}" appended after the vanilla DINO
title (e.g. "Diplomacy is Not an Option | DINOForge v0.27.0").

### Scenario 2 — Active total-conversion name in title

**Given** a total-conversion pack (e.g. `warfare-starwars`) is the active pack,
**When** the game window is visible,
**Then** the window title additionally reflects the active pack's display name
(e.g. "Diplomacy is Not an Option | Star Wars: Clone Wars | DINOForge v0.27.0").

### Scenario 3 — Without DINOForge, title is unchanged

**Given** DINOForge is NOT loaded (vanilla DINO),
**When** the game window is visible,
**Then** the window title is the vanilla DINO title with no DINOForge marker.

### Scenario 4 — Indicator updates on hot reload

**Given** DINOForge is loaded and the player triggers a hot reload (`DINOForge_HotReload`
signal), changing the active pack,
**When** hot reload completes,
**Then** the window title reflects the newly active pack name within 2 seconds.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | Window title modification via `Application.productName` setter or Win32 `SetWindowTextW`. |
| F-02 | Title format: `"{vanilla title} | {active_pack.name} | DINOForge v{version}"`. If no TC pack active, omit the pack segment. |
| F-03 | Title is set in `Plugin.Awake()` immediately after version info is available. |
| F-04 | Title is updated by `RuntimeDriver` after pack loading completes and after hot reload. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Win32 `SetWindowTextW` must be called on the main thread or from a P/Invoke that is thread-safe for the title-bar API. |
| N-02 | If `Application.productName` setter is not available (Unity 2021.3 restriction), fall back to Win32 P/Invoke `FindWindow` + `SetWindowText`. |
| N-03 | The title modification must not affect save-file compatibility or Steam overlay behavior. |

## Engine Quirks / Dependencies

- `MonoBehaviour.Update()` never fires — title set is one-shot in Awake and on pack-load event.
- Unity 2021.3 `Application.productName` is read-only at runtime on some platforms; Win32
  `SetWindowTextW(FindWindow(null, currentTitle), newTitle)` is the reliable fallback for Windows.
- Test by reading `GetWindowText` back after setting — do not trust `Application.productName` as
  confirmation.

## Definition of Done

- [ ] Window title shows DINOForge marker immediately after BepInEx loads the plugin.
- [ ] Window title shows active TC pack name after packs load.
- [ ] Window title updates after hot reload.
- [ ] Vanilla DINO title (no DINOForge) unchanged when DINOForge is absent.
- [ ] Screenshot of window title visible in taskbar deposited in `docs/proof/`.
- [ ] `dotnet test` green — unit tests for `WindowTitleService.Format()`.

## Related

- `SPEC-002-native-menu-injector.md`
- CLAUDE.md — Game Launch Protocol (window title check)
