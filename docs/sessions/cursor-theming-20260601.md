# Cursor Theming — 2026-06-01

## Goal
- Verify whether DINO or runtime code already set cursor textures via `Cursor.SetCursor` and implement a runtime cursor hook for Star Wars pack assets.
- Ensure SW-themed cursors apply on gameplay and menu scene transitions.
- Add lightweight cursor visuals under `packs/warfare-starwars/assets/ui/` if missing.

## API search findings
- `Cursor.SetCursor`/`UnityEngine.Cursor` references in source: none found in runtime gameplay codepaths before this change.
- `SceneManager.activeSceneChanged` and `sceneLoaded` hooks already exist in `src/Runtime/Plugin.cs`.
- Existing pack art loading pattern found in `src/Runtime/UI/MainMenuThemer.cs` (`LoadSpriteFromPack`) and leveraged for path conventions.

## Runtime changes
- Added `src/Runtime/UI/TcCursorApplicator.cs`.
- Hooked scene transitions in:
  - `Plugin.OnActiveSceneChanged`
  - `Plugin.OnSceneLoaded`
  - `RuntimeDriver.OnRuntimeDriverSceneChanged`
- Added per-frame input-driven fallback refresh in runtime loop:
  - `RuntimeDriver.Initialize()` while-loop calls `TcCursorApplicator.UpdateFromInput(...)`.
- Cursor source is resolved from:
  - `<BepInEx>/dinoforge_packs/warfare-starwars/assets/ui/`
- Added asset paths:
  - `cursor_default.png`
  - `cursor_attack.png`
  - `cursor_target.png`

## Cursor assets created
- New files created in `packs/warfare-starwars/assets/ui/`:
  - `cursor_default.png`
  - `cursor_attack.png`
  - `cursor_target.png`

## Validation
- Build requested by user: `dotnet build src/Runtime` (run in final report with exit result).
