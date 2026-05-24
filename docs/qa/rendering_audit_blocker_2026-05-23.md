# Rendering Audit Blocker

**Date**: 2026-05-23  
**Scope**: Screenshot-based rendering/lighting audit for the current Star Wars asset path  
**Status**: METADATA FIXED, IN-GAME VISUAL STILL MANUAL — NOT VERIFIED LIVE

## Observed Symptoms

- Black billboard-like artifacts appear where slope smoothing was expected.
- Shader / lighting toggles appear to have no visible effect.
- Noon sun or other static light appears present, but map objects remain flat-colored.
- No visible dynamic light response was evident from the screenshot evidence.

## Evidence From Repo State

### Fixed (repo metadata / import path)

- `AssetImportService` defaults imported materials to **URP Lit** (`Universal Render Pipeline/Lit`) in `src/Tools/PackCompiler/Services/AssetImportService.cs:275-289`.
- All nine warfare-starwars unit imported JSON files under `packs/warfare-starwars/assets/imported/` that declare `materials[].shaderName` use **URP Lit**, not `Standard` (e.g. `rep_clone_commando.json:15`).
- Regression guard: `src/Tests/WarfareStarwarsImportedShaderTests.cs` asserts no imported JSON uses `shaderName` `Standard` and that `AssetImportService` keeps the URP Lit default.

### Still open (in-game / Unity manual work)

- The Star Wars asset pipeline explicitly requires **URP Lit materials created and assigned in Unity**, with lighting-sensitive setup handled in the editor, not by a runtime toggle, in `packs/warfare-starwars/assets/ASSET_PIPELINE.md:196-219`.
- The same pipeline marks assets as placeholders and calls for later manual material work and validation, not an already-complete lighting pass, in `packs/warfare-starwars/assets/BUILD_CHECKLIST.md:269-272`.
- Ten building LOD0 imported JSON stubs still have empty `materials` arrays (placeholders); they do not carry authored shader/texture setup until Unity prefab work completes.
- Runtime VFX fallback still defaults to `Particles/Standard Unlit`, then `Standard` if the particle shader is missing, in `src/Runtime/VFX/VFXPrefabFactory.cs:109-141` (separate from pack mesh import metadata).

## Audit Conclusion

The prior `Standard` shader metadata on imported warfare-starwars JSON is corrected in-repo. That removes one likely cause of flat fallback-looking units **at the metadata layer**, but it does **not** by itself prove in-game URP Lit materials, textures, or lighting response.

Screenshot symptoms remain consistent with placeholder or manually incomplete Unity content:

1. Placeholder/import fallback geometry still in use.
2. Billboard or particle-like content rendered as a flat fallback.
3. Unity scene objects not yet using authored URP Lit materials/textures (manual Step 3 in ASSET_PIPELINE.md).
4. A missing slope-smoothing asset/pipeline step rather than a live toggle bug.

**Summary**: metadata fixed, in-game visual still manual.

## Automated capture (bridge — not visual sign-off)

| Item | Detail |
|------|--------|
| **xUnit test** | `GameLaunchAssetSwapTests.AssetVisualAcceptance_WarfareStarwarsLoaded_CapturesScreenshotWithReceipt` |
| **Prerequisite** | `DINO_GAME_PATH` set and bridge healthy (`GameLaunchFixture`); soft-skips when game unavailable |
| **Readiness gate** | `status.LoadedPacks` contains `warfare-starwars` **or** `query_entities` returns `rep_clone_trooper` |
| **Screenshot** | Bridge `screenshot` RPC via `GameClient.ScreenshotAsync` |
| **Evidence** | `docs/qa/evidence/asset-swap/warfare-starwars-asset-visual.png` + `visual-acceptance-receipt.json` (temp dir if repo evidence path is not writable) |
| **Review** | Human pass/fail still per [asset-visual-acceptance.md](../reference/asset-visual-acceptance.md); receipt `overall_pass` only means capture succeeded |

## Acceptance Criteria

- World geometry uses the authored material path, not the default `Standard` fallback.
- The relevant scene objects react visibly to sun or light direction changes.
- Shader / lighting toggles produce an observable change in captured screenshots.
- Black billboard artifacts are eliminated or traced to a known placeholder asset with a tracked replacement.
- A follow-up screenshot set confirms slope transitions render as mesh/terrain smoothing rather than black quads.
