# SPEC: Fix F9 Debug Overlay Panel Empty Content

**Status**: DRAFT  
**Date**: 2026-05-25  
**Relates to**: iter-147 maturity wave, SPEC-fix-f10-blank-labels  

## Problem Statement

The F9 debug overlay (`DebugPanel`) renders a visible panel frame (dark background, header with title "DINOForge Debug") but all section content is invisible. The five collapsible sections (Platform Status, ECS Worlds, Systems, Archetypes, Errors) show their toggle headers but the text rows within each expanded section display no visible glyphs. The panel is structurally present (scroll works, close button works) but content text is blank.

## Root Cause Analysis

The investigation examined `src/Runtime/UI/DebugPanel.cs`, `src/Runtime/UI/UiBuilder.cs`, `src/Runtime/UI/DFCanvas.cs`, and the F9 key handler in `src/Runtime/Plugin.cs`. The root cause is a **compound issue** with one primary cause shared with the F10 panel and two contributing factors specific to the DebugPanel.

### RC-1 (PRIMARY, SHARED): Font atlas not rebuilt in BepInEx + DINO custom PlayerLoop context

**Identical to SPEC-fix-f10-blank-labels RC-1.** Both panels use `UiBuilder.MakeText()` which suffers from the same font atlas population failure.

`UiBuilder.MakeText()` (lines 135-146) resolves a font via a four-step fallback chain:
1. `Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf")` -- returns `null` (Unity 2023+ name)
2. `Resources.GetBuiltinResource<Font>("Arial.ttf")` -- returns a Font object
3. `Resources.Load<Font>("Arial")` -- skipped (step 2 succeeded)
4. `Font.CreateDynamicFontFromOSFont("Arial", fontSize)` -- skipped (step 2 succeeded)

The returned font is a dynamic font whose texture atlas is populated on demand via `Font.RequestCharactersInTexture()`, which Unity's `Text` component calls internally during `Canvas.Update()` in the standard PlayerLoop. Since DINO replaces Unity's PlayerLoop, `Canvas.Update()` execution timing is non-standard. The font atlas may not contain the required glyphs when the text mesh is generated.

Additionally, **every** `MakeText()` call resolves the font independently (no caching), and `Font.textureRebuilt` is never subscribed to -- so if the atlas IS rebuilt later, existing Text components are not notified to regenerate their meshes.

The DebugPanel calls `MakeText()` dozens of times per refresh (header, chevrons, section titles, every info row key/value pair, pack names, world names, system names). Every single one of these is affected.

### RC-2 (CONTRIBUTING, DEBUGPANEL-SPECIFIC): sizeDelta override erases offsetMax in BuildScrollContent

**File**: `src/Runtime/UI/DebugPanel.cs`, lines 247-252

```csharp
RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
scrollRt.anchorMin = Vector2.zero;
scrollRt.anchorMax = Vector2.one;
scrollRt.offsetMin = Vector2.zero;
scrollRt.offsetMax = new Vector2(0f, -(HeaderHeight + 1f));
scrollRt.sizeDelta = Vector2.zero;  // <-- BUG: overwrites offsetMax
```

The scroll viewport is anchored to fill the parent panel (`anchorMin=0,0` to `anchorMax=1,1`). Line 251 correctly offsets the top edge by `-(HeaderHeight + 1f)` = -45px so the scroll area starts below the header. However, line 252 then sets `sizeDelta = Vector2.zero`, which in a stretch-anchored RectTransform resets BOTH `offsetMin` and `offsetMax` to zero, undoing the header offset.

**Impact**: The scroll viewport overlaps the header by 45px at the top. Content rendered in the first 45px of the scroll area is hidden behind the opaque header panel. This does not cause total emptiness (scrolling down would still reveal content), but it hides the first two sections partially, contributing to the "empty" appearance when the font atlas issue makes all text invisible anyway.

**Fix**: Remove line 252. The `offsetMin`/`offsetMax` values already correctly define the rect. `sizeDelta` should not be set when using stretch anchors with explicit offsets.

### RC-3 (CONTRIBUTING, DEBUGPANEL-SPECIFIC): Destroy + rebuild cycle on every 0.5s refresh creates GC pressure and layout flicker

**File**: `src/Runtime/UI/DebugPanel.cs`, lines 274-330

Every 0.5 seconds (`RefreshInterval`), `RefreshContent()` destroys ALL children of `_contentRoot` and rebuilds the entire section tree from scratch:

```csharp
// Destroy previous content
for (int i = _contentRoot.childCount - 1; i >= 0; i--)
{
    Destroy(_contentRoot.GetChild(i).gameObject);
}
// Then rebuild all sections...
```

`UnityEngine.Object.Destroy()` is deferred -- the object is not actually removed until the end of the current frame. This means during the same frame, both old (being-destroyed) and new children coexist under `_contentRoot`. The `VerticalLayoutGroup` on the content sees double the children, computes incorrect preferred sizes, and `ContentSizeFitter` may collapse to zero height if the layout math produces a conflicting result.

Additionally, the font atlas issue compounds here: every rebuild creates new `Text` components that need fresh atlas entries. If the atlas rebuild callback is not wired, these new Text components never get their glyphs populated.

**Impact**: Even if the font issue were resolved, the destroy-rebuild cycle could cause 1-frame flickers where content disappears. More critically, it means the font atlas must be populated on EVERY refresh, not just the first build.

## Relationship to F10 Panel Issue

| Aspect | F9 DebugPanel | F10 ModMenuPanel |
|--------|---------------|------------------|
| Primary cause | Font atlas (RC-1) | Font atlas (RC-1) |
| Same UiBuilder.MakeText? | Yes | Yes |
| Additional layout bug | sizeDelta override (RC-2) | childControlWidth not set |
| Refresh pattern | Destroy/rebuild every 0.5s (RC-3) | Static build, no periodic refresh |
| Fix dependency | Fixing RC-1 in UiBuilder fixes BOTH panels | Same |

**The F9 and F10 panels share the same primary root cause.** Fixing `UiBuilder.MakeText()` once resolves the font atlas issue for both panels. The DebugPanel has two additional contributing bugs (RC-2, RC-3) that should be fixed independently.

## Proposed Fix

### Fix 1: Font caching + atlas request + rebuild callback in UiBuilder (SHARED with F10)

As specified in `SPEC-fix-f10-blank-labels.md` Fix 1. Cache the resolved font in a static field, call `Font.RequestCharactersInTexture()` after setting text, and subscribe to `Font.textureRebuilt` once.

**File**: `src/Runtime/UI/UiBuilder.cs`

```csharp
// Add at class level:
private static Font? _cachedFont;
private static bool _fontCallbackRegistered;

// Replace per-call font resolution in MakeText (lines 135-146) with:
if (_cachedFont == null)
{
    _cachedFont = Resources.GetBuiltinResource<Font>("Arial.ttf");
    if (_cachedFont == null)
        _cachedFont = Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf");
    if (_cachedFont == null)
        _cachedFont = Resources.Load<Font>("Arial");
    if (_cachedFont == null)
        _cachedFont = Font.CreateDynamicFontFromOSFont("Arial", 14);
}
if (_cachedFont != null)
    t.font = _cachedFont;

// After setting t.text and t.font, force atlas population:
if (t.font != null && !string.IsNullOrEmpty(text))
{
    t.font.RequestCharactersInTexture(text, fontSize, t.fontStyle);
}

// Register callback once:
if (!_fontCallbackRegistered && t.font != null)
{
    Font.textureRebuilt += OnFontTextureRebuilt;
    _fontCallbackRegistered = true;
}
```

```csharp
// Add static callback method:
private static void OnFontTextureRebuilt(Font changedFont)
{
    // Main thread only (Font.textureRebuilt fires on main thread).
    foreach (var text in Resources.FindObjectsOfTypeAll<Text>())
    {
        if (text.font == changedFont)
            text.SetAllDirty();
    }
}
```

Also fix the dead double-assignment (lines 151-152):
```csharp
// BEFORE:
t.verticalOverflow = VerticalWrapMode.Truncate;
t.verticalOverflow = VerticalWrapMode.Overflow;

// AFTER:
t.verticalOverflow = VerticalWrapMode.Overflow;
```

And correct the resolution order for Unity 2021.3 (try `Arial.ttf` first since `LegacyRuntime.ttf` is a 2023+ name).

### Fix 2: Remove sizeDelta override in BuildScrollContent (RC-2)

**File**: `src/Runtime/UI/DebugPanel.cs`, line 252

```csharp
// BEFORE (lines 247-252):
RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
scrollRt.anchorMin = Vector2.zero;
scrollRt.anchorMax = Vector2.one;
scrollRt.offsetMin = Vector2.zero;
scrollRt.offsetMax = new Vector2(0f, -(HeaderHeight + 1f));
scrollRt.sizeDelta = Vector2.zero;

// AFTER (remove line 252):
RectTransform scrollRt = scrollRect.GetComponent<RectTransform>();
scrollRt.anchorMin = Vector2.zero;
scrollRt.anchorMax = Vector2.one;
scrollRt.offsetMin = Vector2.zero;
scrollRt.offsetMax = new Vector2(0f, -(HeaderHeight + 1f));
```

### Fix 3: Use DestroyImmediate or guard layout during refresh cycle (RC-3)

**File**: `src/Runtime/UI/DebugPanel.cs`, lines 285-288

Replace `Destroy()` with `DestroyImmediate()` in the refresh loop so old children are removed before new ones are created, preventing double-child layout corruption:

```csharp
// BEFORE:
for (int i = _contentRoot.childCount - 1; i >= 0; i--)
{
    Destroy(_contentRoot.GetChild(i).gameObject);
}

// AFTER:
for (int i = _contentRoot.childCount - 1; i >= 0; i--)
{
    UnityEngine.Object.DestroyImmediate(_contentRoot.GetChild(i).gameObject);
}
```

**Note**: `DestroyImmediate` is generally discouraged in production Unity code because it can break iteration and cause issues if references are held elsewhere. However, in this context, we iterate in reverse order (safe), the children are UI elements with no external references, and the alternative (deferred Destroy causing ghost children during layout) is worse. The conventional alternative is to disable the LayoutGroup before destroying and re-enable after rebuilding, but `DestroyImmediate` is simpler for a debug panel.

### Fix 4 (Optional): Reduce refresh frequency or use dirty-flag pattern

The 0.5-second full-rebuild creates unnecessary GC pressure and compounds the font atlas issue. A more robust approach would be to only update text values in-place rather than destroying and recreating the entire hierarchy:

```csharp
// Store references to value Text components
private readonly Dictionary<string, Text> _valueTexts = new(StringComparer.Ordinal);

// In AddInfoRow, register the value Text:
_valueTexts[$"{key}"] = valText;

// In RefreshContent, update existing text instead of rebuilding:
if (_valueTexts.TryGetValue("FPS", out Text fpsText))
    fpsText.text = $"{fps:F0}";
```

This is a larger refactor and is optional for the initial fix. The destroy/rebuild approach works correctly once RC-1 and RC-3 are fixed.

## Files Changed

| File | Change | Risk |
|------|--------|------|
| `src/Runtime/UI/UiBuilder.cs` | Font caching, atlas request, textureRebuilt callback, cleanup | Medium (static state; main-thread constraint on callback) |
| `src/Runtime/UI/DebugPanel.cs` | Remove sizeDelta override (line 252); DestroyImmediate in refresh | Low (layout fix + immediate cleanup) |

## Verification Plan

### Build Verification (CI-safe)

1. `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release` exits 0
2. No new compiler warnings in `src/Runtime/UI/`

### In-Game Verification

1. **Deploy**: `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true -p:TargetFramework=netstandard2.0`
2. **Launch**: Start DINO, wait for main menu
3. **F9 test**: Press F9 -- verify panel slides in with visible content:
   - [ ] "DINOForge Debug" header text visible
   - [ ] "Platform Status" section header with chevron visible
   - [ ] Version, FPS, GC Heap, Bridge status rows show text
   - [ ] Loaded packs count and names visible
   - [ ] "ECS Worlds" section shows "No worlds found" or world data
4. **Toggle test**: Press F9 to hide, F9 again to show -- text remains visible
5. **Section collapse**: Click a section header -- content hides/shows, chevron updates
6. **Scroll test**: If content exceeds panel height, scroll works and text remains visible
7. **Scene transition**: Enter gameplay, F9 -- ECS Worlds/Systems/Archetypes populate
8. **Regression**: No gray-freeze or hang (check for `mono_jit_cleanup` block pattern)

### Log Verification

Check `dinoforge_debug.log` for:
- `[DebugPanel.RefreshContent] Building content. _modPlatform=SET` (confirms refresh fires)
- No `[DebugPanel.RefreshContent] _contentRoot is NULL` warnings
- `[DebugPanel] Show() called. ModPlatform=set` (confirms ModPlatform wired)

## Risk Assessment

- **Fix 1 (font caching)**: Medium risk. The `Font.textureRebuilt` callback uses `Resources.FindObjectsOfTypeAll<Text>()` which must run on the main thread -- this is guaranteed because `Font.textureRebuilt` fires on the main thread in Unity. The static font cache is safe for concurrent reads (Font is effectively immutable after creation). The `RequestCharactersInTexture` call adds minimal overhead per text element creation.

- **Fix 2 (sizeDelta removal)**: Minimal risk. Removes a line that overwrites correct offset values. The stretch anchors already define the rect correctly.

- **Fix 3 (DestroyImmediate)**: Low risk. Reverse-order iteration prevents index issues. UI elements have no external references that would break. The debug panel is not performance-critical.

## Dependencies

- Fix 1 is shared with `SPEC-fix-f10-blank-labels.md`. Implementing either spec's Fix 1 resolves the font issue for BOTH panels. The fixes should be coordinated to avoid duplicate changes to `UiBuilder.cs`.
