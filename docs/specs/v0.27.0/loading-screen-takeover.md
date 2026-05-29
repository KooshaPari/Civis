# SW-004: Loading Screen Takeover

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 2 — Identity
**Story Points**: 13
**Priority**: P1

---

## User Story

As a **mod player**, I want DINOForge to display a branded loading screen during game
initialization — and for my active total-conversion pack to replace that screen with its own
art, logo, and lore tips — so that the modded experience begins from the very first frame,
not the moment the main menu appears.

## Background

Full design spec: `docs/design/loading-screen-system.md`. Key decisions already made:
- **Tier 1 (Overlay strategy)**: DINOForge places its canvas at `sortingOrder 9998`, painting
  over DINO's loading canvas. No Harmony patches, no Addressables manipulation.
- **Two-phase display**: Show on `InitialGameLoader` scene, fade out on `MainMenu` scene.
- **Pre-scan**: `ThemeScanner` reads only `pack.yaml` headers (type + loading_screen fields)
  before full `ContentLoader` runs — lightweight, ~5ms for 9 packs.
- **Replaces** existing `ModLoadingOverlay` entirely.
- Pack declares `loading_screen:` block only if `type: total_conversion` (validated by schema).

## Acceptance Criteria

### Scenario 1 — Default DINOForge loading screen renders

**Given** DINOForge is installed with no total-conversion pack declaring `loading_screen:`,
**When** the game starts (BepInEx Plugin.Awake fires),
**Then**:
- A full-screen DINOForge branded canvas appears over DINO's loading screen.
- Canvas shows: DINOForge logo, title "DINOForge", subtitle "Mod Platform", progress bar,
  rotating tip text (6-second interval), spinner, version label.
- Progress bar advances as packs load (e.g. "Loading pack: Modern Warfare 3/9").
- Canvas fades out over 0.5s when `MainMenu` scene activates.
- Canvas is destroyed after fade-out.

### Scenario 2 — Total-conversion pack replaces background, logo, and tips

**Given** `warfare-starwars` is the active TC pack and its `pack.yaml` declares `loading_screen:`,
**When** the game starts,
**Then**:
- Background image is the pack's `bg-loading-republic.png` (hyperspace streaks, faction-tinted).
- Logo is the pack's SW logo.
- Tip text cycles through tips declared in `pack.yaml` loading_screen.tips array.
- Accent color on progress bar and separator matches `accent_color: "#FFE81F"`.
- External judge screenshot confirms loading screen reads as Star Wars, not generic DINOForge.

### Scenario 3 — Missing asset gracefully degrades

**Given** a TC pack's `loading_screen.background` references a file that does not exist,
**When** the game starts,
**Then**:
- No exception is thrown.
- The default DINOForge background renders in place of the missing asset.
- A `[LoadingScreen] WARNING: background asset not found` message appears in BepInEx log.

### Scenario 4 — Non-total-conversion pack cannot declare loading_screen

**Given** a pack of `type: content` has a `loading_screen:` block in `pack.yaml`,
**When** `PackCompiler validate` runs on that pack,
**Then** validation fails with a schema error mentioning `loading_screen` is not allowed
on non-total-conversion packs.

### Scenario 5 — Loading screen absent after main menu loads

**Given** the loading screen was visible during startup,
**When** the `MainMenu` scene is active and the fade-out completes,
**Then** no DINOForge loading canvas, overlay, or `DINOForge_LoadingScreen` GameObject exists
in the scene hierarchy.

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `LoadingScreenController` replaces `ModLoadingOverlay` — both MUST NOT coexist. |
| F-02 | `ThemeScanner.ScanForActiveTheme(packsDir)` is called before building the canvas, returns `null` if no TC pack declares `loading_screen:`. |
| F-03 | Canvas hierarchy matches `docs/design/loading-screen-system.md §2.3`. |
| F-04 | Tip rotation uses `Time.unscaledDeltaTime` (not `Thread.Sleep`, not wall-clock). |
| F-05 | Progress bar uses `Mathf.Lerp` to smooth pack-count updates (no jarring jumps). |
| F-06 | `loading_screen:` key rejected by schema validator for `type != total_conversion`. |
| F-07 | Background texture loaded via async `Task` (disk read) + main-thread `Texture2D.LoadImage`; fallback if > 3s or error. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | Canvas `sortingOrder 9998` — below DFCanvas (32767), above DINO loader (0–100). |
| N-02 | `DontDestroyOnLoad` on the loading screen GameObject; destroyed explicitly after fade. |
| N-03 | No `Start()` / `Update()` / coroutine dependency beyond `WaitForEndOfFrame` yield. |
| N-04 | `netstandard2.0` only — no direct TMP or Addressables compile references. |

## Engine Quirks / Dependencies

- `MonoBehaviour.Update()` never fires — tip rotation runs in the existing RuntimeDriver
  `WaitForEndOfFrame` coroutine path.
- `SceneManager.activeSceneChanged` is the hook; `sceneLoaded` is unreliable (iter-144 #546).
- DINO scene names: `InitialGameLoader` = primary injection, `MainMenu` = fade trigger
  (confirmed iter-144; confirm again with `dinoforge ui-tree` before hardcoding).
- `Resources.FindObjectsOfTypeAll` from background thread DEADLOCKS — all canvas creation
  on main thread inside `activeSceneChanged` callback.

## Definition of Done

- [ ] Default DINOForge loading screen visible during plugin init (screenshot proof).
- [ ] SW pack loading screen shows faction art, logo, and tips (external judge receipt).
- [ ] Missing-asset fallback verified in test (no exception, default background shown).
- [ ] Schema rejects `loading_screen:` on content packs.
- [ ] Canvas destroyed after fade-out (no memory leak).
- [ ] `dotnet test` green — `ThemeScanner` + `LoadingScreenConfig` validation unit tests.

## Related

- `docs/design/loading-screen-system.md` (full design spec)
- `docs/design/identity-starwars.md §4` (loading screen art concept)
- `docs/design/identity-modern.md §4` (loading screen art concept)
- SW-005 (brand identity assets used by this screen)
