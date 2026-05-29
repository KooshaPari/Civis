# SW-012: Audio Takeover (Music + SFX per Mod)

**Status**: Proposed
**Date**: 2026-05-28
**Author**: DINOForge Agents
**Epic**: [EPIC-027 — True Full-Conversion Experience](../v0.27.0-full-conversion-epic.md)
**Sprint**: 4 — Mechanics
**Story Points**: 8
**Priority**: P2

---

## User Story

As a **mod player**, I want the background music and unit sound effects to change when a
total-conversion pack is active — so that DINO's medieval orchestral score does not undercut
the Star Wars or modern military atmosphere.

## Background

Music replacement design is covered in `docs/design/main-menu-takeover.md §4`. Key decision:
`AudioClip` must come from a Unity 2021.3.45f2 AssetBundle — `.ogg`/`.wav` cannot be decoded
into an `AudioClip` at runtime on Mono without a third-party library.

Audio replacement is split into two tiers:
- **Tier 1 (v0.27.0)**: Menu music swap + gameplay ambient music swap.
- **Tier 2 (v0.28.0)**: Unit SFX replacement per unit type (requires per-unit AudioClip mapping).

## Acceptance Criteria

### Scenario 1 — Star Wars menu music plays on main menu

**Given** `warfare-starwars` is active and `audio/menu_theme.unity3d` is present in the pack,
**When** the player is on the main menu,
**Then** a Republic-themed orchestral piece plays instead of DINO's medieval theme.
**And** volume is 0.8 as declared in `ui_theme.audio.volume`.

### Scenario 2 — Music crossfades from vanilla to mod

**Given** `warfare-starwars` is active and `audio.fade_in_ms: 2000` is declared,
**When** the main menu scene activates and `MainMenuReskinner` finds the vanilla AudioSource,
**Then** the vanilla track fades out and the mod track fades in over ~2 seconds (not a hard cut).

### Scenario 3 — Warfare Modern menu music plays

**Given** `warfare-modern` is active and `audio/menu_theme.unity3d` is present,
**When** the player is on the main menu,
**Then** a tension-atmosphere military ambient track plays instead of DINO's theme.

### Scenario 4 — Gameplay ambient music swaps

**Given** `warfare-starwars` is active and `audio/gameplay_theme.unity3d` is declared,
**When** the player is in an active gameplay session,
**Then** the mod's gameplay music plays instead of DINO's in-game music.
(Scene detection: scene name contains "GameWorld" or "Gameplay".)

### Scenario 5 — Missing audio bundle degrades gracefully

**Given** a pack declares `audio.menu_music` but the bundle file does not exist,
**When** `MainMenuReskinner` attempts to load the audio,
**Then** DINO's vanilla music plays (no exception), and a WARNING appears in BepInEx log.

### Scenario 6 — Audio is re-applied after scene reload

**Given** the player quits to main menu from gameplay and reloads,
**When** the MainMenu scene activates again,
**Then** the mod music plays again (not the vanilla track).

## Functional Requirements

| ID | Requirement |
|----|-------------|
| F-01 | `ui_theme.audio.menu_music` and `ui_theme.audio.gameplay_music` schema fields added to `schemas/ui_theme.schema.json`. |
| F-02 | `FindMenuMusicSource()` scans `FindObjectsOfType<AudioSource>()` for a looping track > 30s duration (main thread only). |
| F-03 | Music swap via `audioSource.Stop()` → swap `clip` → `Play()`. |
| F-04 | Crossfade (if `fade_in_ms > 0`) implemented without `Thread.Sleep` — use a System.Threading.Timer or coroutine-free approach on main thread. |
| F-05 | Audio bundles stored in `packs/<id>/assets/audio/`; built with Unity 2021.3.45f2. |
| F-06 | SFX replacement (per-unit AudioClip) deferred to v0.28.0 — no scope here. |

## Non-Functional Requirements

| ID | Requirement |
|----|-------------|
| N-01 | `FindObjectsOfType<AudioSource>()` called on main thread only (inside scene-change callback or RuntimeDriver pump). |
| N-02 | AudioClip from bundle must be loaded with `bundle.LoadAsset<AudioClip>(name)` — only Unity 2021.3.45f2 bundles supported. |
| N-03 | All shipped audio is original composition or CC0 licensed. No copyrighted soundtracks. |

## Asset Requirements

### warfare-starwars (`packs/warfare-starwars/assets/audio/`)

- `menu_theme.unity3d` — AssetBundle, AudioClip, Republic orchestral, loop > 60s
- `gameplay_theme.unity3d` — AssetBundle, AudioClip, battle orchestral, loop > 90s

### warfare-modern (`packs/warfare-modern/assets/audio/`)

- `menu_theme.unity3d` — AssetBundle, AudioClip, military ambient, loop > 60s, fade-in 1.5s
- `gameplay_theme.unity3d` — AssetBundle, AudioClip, tense action loop > 90s

## Engine Quirks / Dependencies

- DINO may use multiple AudioSources for music layers; `FindMenuMusicSource()` uses the
  loop+duration heuristic — confirm with a `dinoforge dump --type AudioSource` run before shipping.
- `AudioSource.Stop()` + clip swap + `Play()` is the reliable pattern; DOTween is not
  guaranteed to be present in DINO (confirm with assembly scan before using it for crossfade).
- Depends on ThemeEngine Phase 1 (SW-006) for scene-change hook.
- Music re-application on scene reload uses `SceneManager.activeSceneChanged` (same hook).

## Definition of Done

- [ ] SW menu music plays on main menu (external judge receipt — audio described in screenshot caption).
- [ ] Modern menu music plays on main menu.
- [ ] Gameplay ambient music swaps for both mods.
- [ ] Crossfade works without hard cut (observation in live game).
- [ ] Missing bundle degrades gracefully (unit test + log confirmation).
- [ ] Schema additions for `audio` block validated by `PackCompiler`.
- [ ] All shipped audio is original / CC0 (license declaration in pack README).
- [ ] `dotnet test` green.

## Related

- `docs/design/main-menu-takeover.md §4` (music swap spec)
- `docs/design/identity-starwars.md §4` (loading screen tip + audio notes)
- `docs/design/identity-modern.md §4` (loading screen + audio notes)
- SW-006 (ThemeEngine — scene-change hook provider)
- SW-003 (real asset bundles — audio bundles follow same pipeline)
