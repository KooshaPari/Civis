# SW-006: Full UI/UX Reskin Engine

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 2 — Identity
**Story Points**: 21
**Priority**: P1

---

## User Story

As a **mod author**, I want a declarative `ui_theme` block in my `pack.yaml` to restyle
every DINO UI surface — main menu, HUD, build menu, unit panel, tooltips, dialogs,
loading screens — so that a total conversion can deliver a cohesive visual overhaul
without writing any C# code.

As a **mod player**, I want the entire in-game UI to reflect the active total-conversion
theme — not just the main menu — so that the game feels immersive throughout a full session.

## Background

Full design spec: `docs/design/ui-ux-reskin-system.md`. Architecture decisions made:
- **ThemeEngine** owned by RuntimeDriver; replaces MainMenuThemer as the single entry point.
- **7 ordered phases** (0–7), each independently shippable and verifiable in-game.
- **ISurfaceReskinner** plugin pattern — new surfaces require a new class, not an engine change.
- **ResolvedTheme** computed once on pack change, immutable (Pattern #123).
- **3-layer priority merge**: DINOForge default → active TC pack → user override.
- **Per-canvas dedupe** via `_styledCanvasIds` HashSet — each canvas styled once per scene.
- No Harmony patches; no per-frame work (`Update()` never fires anyway).

v0.27.0 targets Phases 0–6 (Phase 7 = user-override layer deferred to v0.28.0).

## Acceptance Criteria

### Scenario 1 — Phase 0: Schema and resolver foundation (no visible change)

**Given** Phase 0 is implemented,
**When** `dotnet test` runs,
**Then**:
- `schemas/ui_theme.schema.json` exists and validates a well-formed `ui_theme` block.
- `UiTheme`, `ResolvedTheme`, `ThemeResolver` compile and all unit tests pass.
- `IThemeAssetResolver` resolves a test sprite from a raw PNG path.
- FsCheck property tests confirm merge algorithm is associative and last-writer-wins.

### Scenario 2 — Phase 1: Main menu reskinner parity

**Given** Phase 1 is implemented and `warfare-starwars` is active,
**When** the player reaches the main menu,
**Then**:
- Main menu looks identical to iter-146 baseline (no regression in title/button reskin).
- `ThemeEngine.Tick` drives the reskin, not the legacy `MainMenuThemer`.
- Screenshot diff against `docs/screenshots/iter146_mods_button_verified.png` shows no regressions.

### Scenario 3 — Phase 2: In-game HUD and resource bar reskinned

**Given** Phase 2 is implemented and `warfare-starwars` is active,
**When** the player is in an active gameplay session,
**Then**:
- HUD panels display the faction palette (Republic gold frames, navy backgrounds).
- Resource icons are replaced with mod icons (e.g. rations icon instead of food).
- Resource counter numerals use the mod font (Exo 2) where available.
- External judge screenshot confirms the HUD reads as Star Wars themed.

### Scenario 4 — Phase 6: Loading screen reskinner

**Given** Phase 6 is implemented,
**When** the active-scene changes to the loading scene,
**Then**:
- `LoadingScreenReskinner` applies the `loading_screen` surface theme.
- Background image is replaced with the mod background.
- Progress bar fill tinted to `accent_color`.

### Scenario 5 — Graceful degrade without assets

**Given** a theme declares a sprite slot whose source file does not exist,
**When** `ThemeEngine.Tick` applies the theme,
**Then**:
- The vanilla sprite remains unchanged (no exception, no null reference).
- A `[ThemeEngine] WARNING: sprite not found` message appears in the BepInEx log.
- All other styled elements are unaffected.

### Scenario 6 — Performance budget

**Given** the ThemeEngine applies a full Star Wars theme to a newly loaded gameplay scene,
**When** the application completes styling all canvases in that scene,
**Then** total elapsed time is < 16 ms (measured via F9 overlay profiling, not self-reporting).

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `ThemeEngine` and all surface reskinners owned by `RuntimeDriver`. |
| F-02 | `MainMenuThemer` is refactored as `MainMenuReskinner : ISurfaceReskinner`; the old class removed. |
| F-03 | `schemas/ui_theme.schema.json` added to the 29-schema set; `PackCompiler validate` checks it. |
| F-04 | `IThemeAssetResolver` resolves sprites in priority order: Addressables key → bundle:asset → bare bundle → raw PNG. |
| F-05 | Raw PNG path: `packs/<id>/assets/ui/<name>.png` → `Texture2D.LoadImage` → `Sprite.Create`. |
| F-06 | Sprites cached by `sourceRef` in `IThemeAssetResolver`; never decoded twice per session. |
| F-07 | `ResolvedTheme` computed once on pack-set change; not per Tick. |
| F-08 | Surface detectors confirmed via live `dinoforge ui-tree` dump before each phase ships. |
| F-09 | `FakeSurfaceReskinner` and `FakeThemeAssetResolver` test doubles ship in `src/Tests/Mocks/` (Pattern #125). |
| F-10 | `surface_detectors.json` is data-driven, loadable from `BepInEx/plugins/`, overridable per game patch. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | No per-frame allocation in ThemeEngine Tick hot path. |
| N-02 | `_styledCanvasIds` ensures each canvas styled once per scene. |
| N-03 | Runtime DLL stays `netstandard2.0`; all TMPro/Addressables access via reflection. |
| N-04 | All canvas walks on the main thread — never from the Win32 background watcher thread. |

## Phase Delivery Plan

| Phase | Surfaces | Verifiable gate |
|---|---|---|
| 0 | Schema + resolver + tests | `dotnet test` green |
| 1 | Main menu (parity) | screenshot vs iter-146 baseline |
| 2 | HUD + resource bar | in-game gameplay screenshot |
| 3 | Build menu + unit panel | screenshot on unit selection |
| 4 | Dialogs + pause menu | screenshot on Esc press |
| 5 | Tooltips + notifications | screenshot on hover |
| 6 | Loading screen background swap | screenshot during InitialGameLoader |

All phases target v0.27.0. Phase 7 (user overrides, BepInEx config) deferred to v0.28.0.

## Engine Quirks / Dependencies

- `SceneManager.activeSceneChanged` (static) is the reliable hook — `sceneLoaded` is not.
- `GetComponentsInChildren<T>(true)` is the approved canvas walk (main thread only).
- Tooltip/dialog canvases are transient — detected and re-styled on each Tick appearance.
- Performance concern: `FindObjectsOfType<Canvas>()` scans all active canvases — only for
  top-level discovery; cached per scene thereafter.

## Definition of Done

- [ ] Phases 0–6 all implemented and independently screenshot-verified.
- [ ] `MainMenuThemer` removed from codebase; `MainMenuReskinner` passes all old tests.
- [ ] `schemas/ui_theme.schema.json` registered; `PackCompiler validate` exercises it.
- [ ] FsCheck merge-algorithm property tests added.
- [ ] `FakeSurfaceReskinner` + `FakeThemeAssetResolver` in `src/Tests/Mocks/`.
- [ ] External judge receipts for Phase 2 HUD (SW) and Phase 2 HUD (Modern).
- [ ] `dotnet test` green; performance budget < 16 ms confirmed via F9 overlay.

## Related

- `docs/design/ui-ux-reskin-system.md` (full architectural spec)
- `docs/design/main-menu-takeover.md §2` (background swap implementation)
- SW-005 (brand identity assets consumed by this engine)
- SW-004 (loading screen takeover uses Phase 6 reskinner)
- Pattern #99 (StringComparer in Dictionary), #123 (immutable collections), #125 (test doubles)
