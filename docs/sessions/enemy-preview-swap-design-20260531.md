# Enemy Incoming Wave Preview Swap — Investigation & Design (2026-05-31)

## 1) Render-path investigation results

- I searched `C:\Users\koosh\Dino\src\Runtime` for: `preview`, `thumbnail`, `WavePreview`, `enemy`, `Portrait`, `RenderTexture`, and `PreviewRenderUtility`.
- No class/method in runtime implements an `"incoming enemies"`/`"wave preview"` panel.
- No native symbol match exists for `incoming` wave UI rendering or for an `Image`/`RawImage` path tied to enemy spawn previews.

What I found:

1. `src/Runtime/Bridge/AssetSwapSystem.cs`
   - This system swaps pending asset requests and tries ECS entity mesh replacement (`RenderMesh` path).
   - It also patches vanilla bundles on disk when possible, but does not route through a dedicated "incoming wave UI thumbnail" surface.
   - The in-code notes explicitly indicate mesh-swap behavior is entity-focused and that some render-mesh variants are explicitly not yet fully implemented.
2. `src/Runtime/Bridge/WaveInjector.cs`
   - Handles queueing and spawning of incoming-wave unit groups and delays.
   - No preview/screenshot rendering behavior found.
3. `src/Runtime/UI/ModMenuPanel.cs`
   - Contains general mod-menu screenshot gallery support and uses `Image` sprites loaded from filesystem screenshots.
   - This is a configuration-facing mod UI, not the launch-time "incoming enemies" preview.
4. `src/Runtime/ModPlatform.cs`
   - Registers wave-related systems and loads metadata (`Safe-swallow: UI preview only` appears in helper methods), but does not contain preview-thumbnails UI code for incoming enemy waves.

## 2) Conclusion: why #986 does not affect this UI

Task #986 swap engine is scoped to live gameplay ECS assets (render mesh and related runtime registry paths).  
The launch-time incoming enemy preview appears to be rendered in a separate native/UI path that is not represented by the current `src/Runtime` implementation.

Therefore: **existing AssetSwap logic in `AssetSwapSystem` is expected to miss this thumbnail surface.**

## 3) Surface classification (based on repo evidence)

Because runtime code does not contain the target panel, we cannot conclusively classify from code whether the surface is:
- a sprite-based `Image`, or
- a preview `RenderTexture` rendered from a camera.

Given current evidence:
- There is no direct `Image` assignment tied to incoming-wave data in runtime.
- There is also no camera/RT assignment path for wave-forecast UI in runtime.

So the surface is likely in native game UI code outside this repo/worktree.

## 4) Concrete fix strategy

### Primary (smallest blast radius once target symbol is identified)

1. **Find native launch panel + target node**
   - At runtime, discover the preview card container and the thumbnail `Image`/`RawImage` component(s) used by incoming-wave UI.
2. **Add runtime hook: `EnemyWavePreviewSwapper`**
   - Resolve preview nodes by type/name and bind by unit-id order from `WaveDefinition`/scenario wave data.
   - Map preview image source to unit sprite assets when available.
3. **Implement sprite source selection**
   - Use unit `visuals.icon` first, then fallback to pack-level `icon`/`texture` map, then fallback to existing vanilla texture.
4. **Fallback if no sprite path exists**
   - Keep native behavior and emit structured warning log.

### Alternate (if preview uses RT + prefab camera)

1. Resolve preview camera prefab/producer component used by incoming-wave UI.
2. Ensure swapped mesh path is used for the preview world by:
   - forcing the preview entity spawn from swapped bundle path, or
   - injecting swapped prefab references into that preview renderer path.
3. Keep `AssetSwapSystem` behavior unchanged for gameplay entities.

## 5) Follow-up / blocked items

- No safe runtime implementation is possible yet without the concrete UI symbol path.
- If native preview is sprite-based and sprites are missing for some mod units, follow-up work is needed to expose authoritative sprite assets in unit models:
  - add pack-side preview sprites/icons in a documented field/contract,
  - add resolver glue in runtime.
- A runtime hook stub should be added once symbol/class location is known.

