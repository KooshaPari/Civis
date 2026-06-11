# SPEC: Fix F10 Mod Menu Panel Blank Pack Labels

**Status**: DRAFT  
**Date**: 2026-05-25  
**Relates to**: Task #880, iter-147  

## Problem Statement

The F10 ModMenuPanel creates pack list items that log correct text content (`text='warfare-starwars'`, `fontSize=13`, `color=RGBA(0.910,0.835,0.690,1.000)`, `font=Arial`) but render as blank/empty labels in-game. Users can click the empty space to select packs, confirming the GameObjects exist and have correct layout dimensions, but no text glyphs are visible.

## Root Cause Analysis

The investigation examined three files and identified a **compound root cause** with three contributing factors, ranked by severity:

### RC-1 (PRIMARY): Font atlas not rebuilt after dynamic font creation in BepInEx context

**File**: `src/Runtime/UI/UiBuilder.cs`, lines 135-146

The font resolution chain in `MakeText()` calls `Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf")` first, which returns `null` on Unity 2021.3 (that name is Unity 2023+). It then falls through to `Resources.GetBuiltinResource<Font>("Arial.ttf")`, which returns a font object. However, in BepInEx plugin context on Mono CLR, the returned font may be a dynamic font whose texture atlas has not been populated with the required glyphs.

Unity's `Text` component calls `Font.RequestCharactersInTexture()` to populate the atlas during `OnPopulateMesh()`, which normally runs during `Canvas.Update()` in the PlayerLoop. Since DINO replaces Unity's PlayerLoop, `Canvas.Update()` execution timing is non-standard. The font atlas may not contain the glyphs needed for pack names at the time the text mesh is generated.

**Evidence**: The `Font.textureRebuilt` callback is never subscribed to by `MakeText()`. In standard Unity, UGUI's `Text` component internally handles atlas rebuilds, but in a custom PlayerLoop environment, this rebuild may be deferred or skipped entirely.

Additionally, `MakeText()` creates a new `Font.CreateDynamicFontFromOSFont("Arial", fontSize)` as a last-resort fallback. Each call with a different `fontSize` creates a NEW font object (fontSize 13 for pack names, 12 for headers, 11 for versions), wasting memory and each having its own empty atlas.

### RC-2 (CONTRIBUTING): HorizontalLayoutGroup missing explicit `childControlWidth`/`childControlHeight`

**File**: `src/Runtime/UI/ModMenuPanel.cs`, lines 712-716

```csharp
HorizontalLayoutGroup hlg = card.AddComponent<HorizontalLayoutGroup>();
hlg.spacing = 6f;
hlg.padding = new RectOffset(pack.IsEnabled ? 10 : 6, 6, 4, 4);
hlg.childForceExpandWidth = false;
hlg.childForceExpandHeight = true;
```

The `childControlWidth` and `childControlHeight` properties are never set. In Unity 2021.3, when added via `AddComponent<HorizontalLayoutGroup>()`, these default to `false` for backward compatibility. This means:

- The layout group does NOT control child RectTransform sizes
- Children retain whatever `sizeDelta` was set by `MakeText()` (200px wide)
- The 200px text width exceeds the card width of 212px (ListWidth - 8), meaning text overflows the card bounds
- Combined with the ScrollView's `Mask` component, overflow content is clipped to nothing

Setting `childControlWidth = true` would let the layout group properly size children within the available card width.

### RC-3 (CONTRIBUTING): Mask/ScrollView viewport may have zero effective height during initial build

**File**: `src/Runtime/UI/ModMenuPanel.cs`, lines 382-391 and `UiBuilder.cs` lines 228-271

The ScrollView viewport's `sizeDelta` is set to `Vector2.zero` (line 387) and relies on anchors `(0,0)-(1,1)` to stretch-fill the parent ListPane. The ListPane has `flexibleHeight = 1` within a body HorizontalLayoutGroup. During `Build()`, the panel starts with `_canvasGroup.alpha = 0` (line 100), and layout is computed before the panel is shown.

If the ListPane's height has not been computed by the layout system when `RebuildPackList()` runs (line 110), the Mask clips all content to a zero-height rect. `ForceRebuildLayoutImmediate(_listContent)` (line 662) only rebuilds the inner content, NOT the outer layout chain that determines viewport height.

## Proposed Fix

### Fix 1: Cache and reuse the resolved font (RC-1)

In `UiBuilder.cs`, resolve the font ONCE in a static field and reuse it. Subscribe to `Font.textureRebuilt` to mark dirty any Text components that need atlas updates.

```csharp
// Add static field at class level:
private static Font? _cachedFont;

// Replace lines 135-146 in MakeText with:
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
```

Additionally, after setting `t.font`, explicitly request the characters to ensure the atlas is populated:

```csharp
// After setting font and text, force atlas population:
if (t.font != null && !string.IsNullOrEmpty(text))
{
    t.font.RequestCharactersInTexture(text, fontSize, t.fontStyle);
}
```

And subscribe to `Font.textureRebuilt` once (in a static initializer or on first MakeText call) to handle atlas rebuilds:

```csharp
private static bool _fontCallbackRegistered;

// Inside MakeText, after font assignment:
if (!_fontCallbackRegistered && t.font != null)
{
    Font.textureRebuilt += OnFontTextureRebuilt;
    _fontCallbackRegistered = true;
}

private static void OnFontTextureRebuilt(Font changedFont)
{
    // Force all active Text components using this font to rebuild their meshes
    foreach (var text in Resources.FindObjectsOfTypeAll<Text>())
    {
        if (text.font == changedFont)
            text.SetAllDirty();
    }
}
```

> **WARNING**: `Resources.FindObjectsOfTypeAll` MUST be called from the main thread only. Since `Font.textureRebuilt` fires on the main thread in Unity, this is safe. However, if DINO's custom PlayerLoop changes this behavior, guard with a main-thread check.

### Fix 2: Set explicit `childControlWidth`/`childControlHeight` on HLG (RC-2)

In `ModMenuPanel.cs`, line 716, add:

```csharp
hlg.childControlWidth = true;
hlg.childControlHeight = true;
```

This allows the HorizontalLayoutGroup to properly distribute space among children based on their LayoutElement settings (minWidth, flexibleWidth).

### Fix 3: Rebuild full panel layout, not just content (RC-3)

In `ModMenuPanel.cs`, at the end of `RebuildPackList()` (line 662), change:

```csharp
// BEFORE:
UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_listContent);

// AFTER:
// Rebuild the entire panel hierarchy so viewport/mask dimensions are correct
// before rebuilding inner content.
if (_panelRt != null)
    UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_panelRt);
UnityEngine.UI.LayoutRebuilder.ForceRebuildLayoutImmediate(_listContent);
```

### Fix 4: Remove dead `verticalOverflow` double-set (cleanup)

In `UiBuilder.cs`, remove line 151 (the `Truncate` assignment that is immediately overridden on line 152):

```csharp
// BEFORE (lines 151-152):
t.verticalOverflow = VerticalWrapMode.Truncate;
t.verticalOverflow = VerticalWrapMode.Overflow;

// AFTER:
t.verticalOverflow = VerticalWrapMode.Overflow;
```

### Fix 5: Correct font name resolution order (optimization)

In `UiBuilder.cs`, try `"Arial.ttf"` first since DINO uses Unity 2021.3:

```csharp
// BEFORE (lines 135-138):
Font? resolvedFont = Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf");
if (resolvedFont == null)
    resolvedFont = Resources.GetBuiltinResource<Font>("Arial.ttf");

// AFTER:
Font? resolvedFont = Resources.GetBuiltinResource<Font>("Arial.ttf");
if (resolvedFont == null)
    resolvedFont = Resources.GetBuiltinResource<Font>("LegacyRuntime.ttf");
```

## Files Changed

| File | Change | Risk |
|------|--------|------|
| `src/Runtime/UI/UiBuilder.cs` | Font caching + atlas request + callback + cleanup | Medium (static state, main-thread constraint) |
| `src/Runtime/UI/ModMenuPanel.cs` | HLG childControl flags + full-panel layout rebuild | Low (additive, layout-only) |

## Verification Plan

1. **Build**: `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release` must pass
2. **Deploy**: Copy DLL to `BepInEx/plugins/` (DeployToGame=true or manual)
3. **Launch**: Start DINO, wait for main menu
4. **F10 test**: Press F10, verify pack labels show text ("warfare-starwars", version numbers)
5. **Scroll test**: If more than ~12 packs, scroll the list and verify labels remain visible
6. **Font log check**: Read `dinoforge_debug.log` for `font=Arial` in `BuildPackListItem` log lines
7. **Memory check**: Verify no `Font.CreateDynamicFontFromOSFont` per-call allocations in subsequent F10 opens (search log for "CreateDynamicFont" calls)

### Automated Verification (CI-safe)

Since this is a runtime-only visual fix, CI verification is limited to:
- Build succeeds
- No new compiler warnings in `src/Runtime/UI/`
- Font caching unit test: mock `Resources.GetBuiltinResource` returns null, verify `CreateDynamicFontFromOSFont` called only once across multiple `MakeText` invocations (requires Unity test runner)

### In-Game Verification Checklist

- [ ] Pack names visible in F10 panel
- [ ] Version labels visible next to pack names
- [ ] "Loaded Packs" header text visible
- [ ] Error/conflict badges render correctly
- [ ] Text survives panel hide/show cycle (F10 toggle)
- [ ] Text survives scene transition (main menu to gameplay and back)
- [ ] No gray-freeze or hang after fix (regression check)

## Risk Assessment

- **Fix 1 (font caching)**: Low risk. Static field is thread-safe for reads (font is immutable after creation). `Font.textureRebuilt` callback is a standard Unity pattern. The `FindObjectsOfTypeAll` call in the callback could be expensive if many Text components exist, but this fires rarely (only on atlas rebuild).

- **Fix 2 (childControl flags)**: Low risk. Makes the HLG behavior explicit rather than relying on Unity version-dependent defaults. May slightly change layout dimensions of children -- the LayoutElement minWidth/flexibleWidth values should produce correct results.

- **Fix 3 (full panel rebuild)**: Minimal risk. Rebuilding the full panel is more work than just the content, but `ForceRebuildLayoutImmediate` is designed for this use case. It's already called in `Show()`.

- **Fix 4/5 (cleanup)**: Zero risk. Dead code removal and reordering.
