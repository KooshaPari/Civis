# In-Game 2D Art Takeover Specification

**Status**: DRAFT — iter-148  
**Scope**: Total-conversion packs; applies equally to partial reskins  
**Related**: `AssetSwapSystem.cs`, `MainMenuThemer.cs`, `UiAssets.cs`, `TotalConversionManifest.cs`

---

## 1. Problem Statement

`AssetSwapSystem` handles 3D RenderMesh swaps on ECS entities (Phase 1: disk-patch Addressables bundle; Phase 2: live `SetSharedComponentData` on matched entities). That path covers meshes and materials on gameplay units.

It does NOT touch 2D art. With a total conversion active, the following surfaces remain vanilla DINO:

| Surface | Current state |
|---------|---------------|
| Main-menu background image | Vanilla background PNG |
| Loading-screen background | Vanilla loading art |
| UI panel sprites (9-slice backgrounds, button frames, dividers) | Vanilla grey/brown panels |
| Faction emblems / crests on HUD | Vanilla icons |
| Unit portraits and class icons in build panel / HUD | Vanilla DINO art |
| Cursor (hardware or software) | Vanilla arrow |
| Fonts (title font, body font on menus) | Vanilla TextMeshPro font assets |

`MainMenuThemer` partially addresses the main menu by tinting the background `Image` component and recolouring buttons. `UiAssets` loads Kenney CC0 sprites for DINOForge's own overlay panels. Neither mechanism lets a mod pack inject custom PNG art into DINO's native UI hierarchy.

---

## 2. How DINO Loads 2D Art — Analysis

### 2.1 Addressables (primary path)

DINO uses Unity Addressables v1.21.18. All game art (textures, sprites, sprite atlases) lives in `.bundle` files under `StreamingAssets/aa/StandaloneWindows64/`. The catalog at `StreamingAssets/aa/catalog.json` maps string keys to bundle paths.

At runtime, DINO calls `Addressables.LoadAssetAsync<Sprite>(key)` or `Addressables.LoadAssetAsync<Texture2D>(key)`. The key is a custom opaque string (not a Unity asset path) — see `AddressablesCatalog.cs` analysis.

**Intercepting `Addressables.LoadAssetAsync`**:

Unity Addressables is implemented as a managed C# library (`UnityEngine.AddressableAssets.dll`). It is patchable with Harmony. The primary entry points are:

```csharp
// overloads to patch (both forward to the same internal IResourceLocator chain):
Addressables.LoadAssetAsync<TObject>(object key)
Addressables.LoadAssetAsync<TObject>(IResourceLocation location)
```

A Harmony **Prefix** on the `object key` overload with `TObject = Sprite` or `TObject = Texture2D` can:
1. Check if a 2D replacement is registered for that key in `SpriteSwapRegistry`.
2. If yes, short-circuit: set `__result` to a completed `AsyncOperationHandle<T>` wrapping the mod sprite, return `false` to skip the original.
3. If no, return `true` (normal path).

The complication is that `LoadAssetAsync` is generic. Harmony cannot patch open generic methods. The fix is to patch the non-generic internal dispatcher (`ResourceManager.StartOperation` or `AddressablesImpl.LoadAssetAsync`) and filter by result type at the Prefix site, OR patch at the `IResourceLocator.Locate` layer. The most reliable approach for Unity 2021.3 is to patch `AddressablesImpl.LoadAssetAsync(IResourceLocation)` after the key has been resolved to a location — at that point the result type is known from the location's `ResourceType` property.

### 2.2 Resources.Load (secondary/legacy path)

Some UI sprites may load via `Resources.Load<Sprite>("path")`. `ResourcesUnloadGuardPatch.cs` already Harmony-patches `Resources.UnloadAsset`; extending to `Resources.Load` is feasible.

However, `Resources.Load` requires assets to be in a `Resources/` folder inside the game data — DINO likely moves all art to Addressables (v1.21.18 is post-Resources era). This is a fallback consideration only.

### 2.3 Direct `AssetBundle.LoadAsset<Sprite>` calls

Specialty UI code (e.g., loading screens) may load bundles directly. `AssetBundleLoadGuardPatch` already intercepts `AssetBundle.LoadFromFile`. A complementary intercept at `AssetBundle.LoadAsset<T>` (and the `LoadAllAssets` variant) would cover this path.

### 2.4 Walking live `Image`/`SpriteRenderer` components (scan-and-replace)

`MainMenuThemer` already does a post-load scan: `FindObjectsOfType<Canvas>()` → `GetComponentsInChildren<Image>()` → mutate `.sprite`. This works for panels and backgrounds that are already instantiated and whose address-key is not needed.

Scan-and-replace is the right fallback for surfaces that have already loaded. The limitation is that the image must be uniquely identifiable (by GameObject path, component name, or sprite name).

---

## 3. Recommended Interception Strategy

Three mechanisms are needed — each covering a different surface at a different lifecycle moment. They are listed in order of application.

### Strategy A — Addressables Harmony Prefix (Tier 1, for keyed sprites)

**What it covers**: Any sprite/texture loaded by key (most UI art, loading screens, faction emblems, unit icons in the build panel).

**Mechanism**:

```csharp
[HarmonyPatch]
internal static class AddressablesSpritePatch
{
    // Patch target: AddressablesImpl.LoadAsset<TObject>(IResourceLocation location)
    // Unity.Addressables.AddressablesImpl is internal; resolve by type name at Awake time.
    // key fallback: also patch the (object key) overload and check key.ToString() in SpriteSwapRegistry.

    static bool Prefix(object key, ref AsyncOperationHandle __result)
    {
        string keyStr = key?.ToString() ?? "";
        Sprite? replacement = SpriteSwapRegistry.GetReplacement(keyStr);
        if (replacement == null) return true; // pass-through

        // Wrap in a completed AsyncOperationHandle using Addressables.ResourceManager.
        __result = Addressables.ResourceManager
            .CreateCompletedOperation(replacement, errorMsg: null);
        return false; // skip original
    }
}
```

The `SpriteSwapRegistry` (new, SDK layer, mirrors `AssetSwapRegistry` design) maps `addressKey → Sprite` (loaded from pack PNG at scene-load time, before Addressables calls fire).

**Timing**: Register replacements during `Plugin.Awake()` / `ContentLoader.LoadPack()` — before the scene that needs them is loaded. Main-menu sprites register at startup. Loading-screen sprites register immediately after `SceneManager.activeSceneChanged` fires for the loading scene, before Addressables loads its content.

**Pros**: Intercepts the load before Unity allocates GPU memory for the vanilla sprite. No scan needed. Works on any surface that uses Addressables regardless of scene layout.

**Cons**: Requires reverse-engineering the Addressables key for each target sprite (discoverable via the DumpTools CLI or the `game_dump_state` MCP tool). The key space is opaque. First use requires a key-discovery pass.

### Strategy B — Post-Load Component Walk (Tier 2, for identified Image components)

**What it covers**: Main-menu background, button sprites, panel sprites, loading-screen full-canvas art. These are stable UGUI hierarchies with predictable `GameObject.name` paths.

**Mechanism**: Extend `MainMenuThemer.ApplyToMainMenu` (which already does this for tints) to also swap `.sprite` on identified `Image` components. A new `TcUiSpritePass` class (counterpart to `MainMenuThemer`) walks:

1. `FindObjectsOfType<Canvas>()` filtered to known canvas names (`MainMenuCanvas`, loading canvas).
2. For each `Image` whose `GameObject.name` or `sprite.name` matches a registry entry, replace `.sprite` with the loaded mod sprite.

**Timing**: Called from `NativeMenuInjector.TryInjectMenuButton` callback chain and from a `SceneManager.activeSceneChanged` subscriber in `RuntimeDriver`.

**Pros**: No key-discovery needed. Works on GameObjects that have no Addressables key (procedurally instantiated UI).

**Cons**: Fragile to DINO UI refactors. Must be re-validated when DINO updates. Only works on already-loaded components.

### Strategy C — Cursor Replacement (Tier 3, OS-level)

**What it covers**: The game cursor.

**Mechanism**: Unity's `Cursor.SetCursor(Texture2D, hotspot, CursorMode)` replaces the hardware cursor. Call from `Plugin.Awake()` after loading the mod cursor PNG.

```csharp
if (SpriteSwapRegistry.TryGetCursorTexture(out Texture2D? cursorTex, out Vector2 hotspot))
    Cursor.SetCursor(cursorTex, hotspot, CursorMode.Auto);
```

---

## 4. Pack Declaration — Schema Extension

### 4.1 `TcAssetReplacements` Model Extension

Extend `TotalConversionManifest.TcAssetReplacements` with a structured `ui` sub-object (the current `Ui` dict is too flat). The new shape:

```csharp
public sealed class TcAssetReplacements
{
    // existing
    public Dictionary<string, string> Textures { get; set; }
    public Dictionary<string, string> Audio { get; set; }

    // extended
    public TcUiReplacements Ui { get; set; } = new();
}

public sealed class TcUiReplacements
{
    // Addressables-key → pack-relative PNG path  (Strategy A)
    [YamlMember(Alias = "keyed_sprites")]
    public Dictionary<string, string> KeyedSprites { get; set; } = new(StringComparer.Ordinal);

    // Named surface slots → pack-relative PNG path  (Strategy B, well-known slots)
    [YamlMember(Alias = "surfaces")]
    public TcUiSurfaces Surfaces { get; set; } = new();

    // Cursor (Strategy C)
    [YamlMember(Alias = "cursor")]
    public TcCursorDef? Cursor { get; set; }

    // Font asset bundle path → TMP font asset name (not covered in this pass; tracked separately)
    [YamlMember(Alias = "fonts")]
    public Dictionary<string, string> Fonts { get; set; } = new(StringComparer.Ordinal);
}

public sealed class TcUiSurfaces
{
    // Each field maps to a discoverable GameObject path / sprite name in DINO's canvas hierarchy.
    // Value: pack-relative path to a replacement PNG (loaded at runtime via Texture2D.LoadImage).
    [YamlMember(Alias = "menu_background")]    public string? MenuBackground { get; set; }
    [YamlMember(Alias = "loading_background")] public string? LoadingBackground { get; set; }
    [YamlMember(Alias = "loading_bar")]        public string? LoadingBar { get; set; }
    [YamlMember(Alias = "panel_bg")]           public string? PanelBg { get; set; }
    [YamlMember(Alias = "button_normal")]      public string? ButtonNormal { get; set; }
    [YamlMember(Alias = "button_hover")]       public string? ButtonHover { get; set; }
    [YamlMember(Alias = "button_pressed")]     public string? ButtonPressed { get; set; }
    [YamlMember(Alias = "faction_emblem_player")]  public string? FactionEmblemPlayer { get; set; }
    [YamlMember(Alias = "faction_emblem_enemy")]   public string? FactionEmblemEnemy { get; set; }
    [YamlMember(Alias = "unit_portrait_prefix")]   public string? UnitPortraitPrefix { get; set; }
    // unit portraits: <prefix><unit_id>.png — e.g. "ui/portraits/" → "ui/portraits/clone-trooper.png"
}

public sealed class TcCursorDef
{
    [YamlMember(Alias = "png")] public string Png { get; set; } = "";
    [YamlMember(Alias = "hotspot_x")] public float HotspotX { get; set; } = 0f;
    [YamlMember(Alias = "hotspot_y")] public float HotspotY { get; set; } = 0f;
}
```

### 4.2 Example Pack YAML Block

```yaml
asset_replacements:
  ui:
    # Strategy A — Addressables key interception
    keyed_sprites:
      "Assets/UI/Textures/MainMenuBg.png": "ui/menu_bg.png"
      "Assets/UI/Icons/FactionEmblem_player.png": "ui/republic-emblem.png"
      "Assets/UI/Icons/FactionEmblem_enemy.png": "ui/cis-emblem.png"
      "Assets/UI/Loading/LoadingScreen_default.png": "ui/loading_bg.png"

    # Strategy B — well-known surface slots
    surfaces:
      menu_background:    "ui/menu_bg.png"
      loading_background: "ui/loading_bg.png"
      panel_bg:           "ui/panel.png"
      button_normal:      "ui/button_normal.png"
      button_hover:       "ui/button_hover.png"
      button_pressed:     "ui/button_pressed.png"
      faction_emblem_player: "ui/republic-emblem.png"
      faction_emblem_enemy:  "ui/cis-emblem.png"
      unit_portrait_prefix:  "ui/portraits/"

    # Strategy C — cursor
    cursor:
      png: "ui/cursor.png"
      hotspot_x: 0
      hotspot_y: 0

    # Future — font replacement (out of scope this pass)
    fonts: {}
```

---

## 5. Loading Textures from Pack Files into Unity at Runtime

`UiAssets.LoadSprite` and `UiAssets.LoadSlicedSprite` (already in `src/Runtime/UI/Assets/UiAssets.cs`) demonstrate the canonical pattern:

```csharp
byte[] data = File.ReadAllBytes(absolutePngPath);
var tex = new Texture2D(2, 2, TextureFormat.RGBA32, mipChain: false)
{
    filterMode = FilterMode.Bilinear,
    wrapMode   = TextureWrapMode.Clamp
};
bool ok = tex.LoadImage(data);   // Unity decompresses PNG/JPG in-place
Sprite sprite = Sprite.Create(tex,
    new Rect(0, 0, tex.width, tex.height),
    new Vector2(0.5f, 0.5f),
    pixelsPerUnit: 100f);
```

`Texture2D.LoadImage` runs on the main thread and is synchronous. For packs with many sprites, pre-load during the loading screen while DINO's own loading is happening (the loading screen is the longest idle window).

For 9-slice sprites (panels, buttons), use `Sprite.Create` with a `Vector4 border` argument:

```csharp
Sprite.Create(tex,
    new Rect(0, 0, tex.width, tex.height),
    pivot: new Vector2(0.5f, 0.5f),
    pixelsPerUnit: 100f,
    extrude: 0,
    SpriteMeshType.FullRect,
    border: new Vector4(left, bottom, right, top));
```

The border values (pixels) must be declared in the pack manifest alongside the PNG path:

```yaml
panel_bg:
  png: "ui/panel.png"
  border: 12          # uniform 9-slice border in pixels
```

---

## 6. Performance and Memory

### 6.1 Loading cost

PNG decode via `Texture2D.LoadImage` is fast (< 5 ms for a 512×512 RGBA PNG on a modern machine). Pre-load the full set during the transition to the main menu scene, not on first access. Loading 20 sprites at 512×512 = ~20 ms — acceptable during a scene load.

### 6.2 Memory per sprite

A 512×512 RGBA32 texture = 1 MB GPU memory. A full 2D skin with 20 sprites at varied sizes ≈ 5–10 MB — trivial compared to DINO's 6.3 GB bundle footprint.

### 6.3 Unloading vanilla textures

When Strategy A intercepts an Addressables load, Unity never allocates the vanilla texture into GPU memory for that call. This is the most memory-efficient path.

For Strategy B (post-load scan), the vanilla `Image.sprite` reference is replaced but the original sprite/texture remains in memory until the scene unloads (Unity reference-counts assets from Addressables). Call `Addressables.Release(originalHandle)` to signal the refcount — but only if the original was loaded via Addressables and the handle is accessible. For sprites baked into bundles that are always-resident, do not attempt manual release.

### 6.4 Sprite cache

Maintain a `Dictionary<string, Sprite>` (keyed by pack-relative PNG path, `StringComparer.Ordinal`) in `SpriteSwapRegistry` so the same PNG is never decoded twice per session. Invalidate on `HotReload` trigger (write `DINOForge_HotReload` signal).

### 6.5 Scene change handling

On `SceneManager.activeSceneChanged`, re-apply Strategy B walk. Strategy A replacements are self-sustaining (the intercept fires on every `LoadAssetAsync` call). Cache only sprites; do not re-read PNG bytes on scene change — load once at pack-load time.

---

## 7. Build Sequence

Implement and test in this order. Each step is independently shippable and verifiable.

### Step 1 — Main menu background (Strategy B, `TintBackground` upgrade)

Extend `MainMenuThemer.TintBackground` to replace `.sprite` on the largest `Image` in the main menu canvas when the pack declares `surfaces.menu_background`. This is the highest-visibility surface and reuses the code path that already works.

Verification: screenshot main menu with warfare-starwars pack active; background image should be the mod PNG, not a dark tint.

### Step 2 — Loading screen background (Strategy B)

Subscribe to `SceneManager.sceneLoaded`. When the scene name contains "Loading", scan for a fullscreen `Image` and replace its sprite with `surfaces.loading_background`. Apply before the scene's own `Start()` methods run (use `loadSceneMode == LoadSceneMode.Single` filter).

### Step 3 — Button and panel sprites (Strategy B)

Extend the `RestyleSelectables` pass in `MainMenuThemer` to also replace `.image.sprite` on buttons when `surfaces.button_normal/hover/pressed` are declared.

### Step 4 — Cursor (Strategy C)

Add `Cursor.SetCursor(...)` call at the end of `Plugin.Awake()` after `ContentLoader` runs. Re-apply on scene change (Unity resets cursor on scene load).

### Step 5 — Faction emblems and unit portraits (Strategy A)

Requires key-discovery pass:
1. Run `dinoforge dump --type Sprite` against a running DINO instance to harvest all Addressables keys that resolve to sprites.
2. Map discovered keys to semantic names (faction emblem, unit portrait ID, etc.).
3. Publish a `docs/reference/dino-sprite-key-map.yaml` reference file.
4. Wire `AddressablesSpritePatch` Harmony prefix using those keys.

### Step 6 — UI panel sprites (Strategy A + B hybrid)

Sprite atlases are the hardest case: DINO likely packs many UI sprites into a single `SpriteAtlas` loaded by one Addressables key. Replacing the whole atlas (Strategy A on the atlas key) replaces all sprites in it — correct for a total conversion, but the mod must supply a compatible atlas (same sprite names, same pivot points). Publish atlas specification in `docs/reference/dino-ui-atlas-spec.yaml` once discovered.

### Step 7 — Fonts

TMP font assets are `TMP_FontAsset` objects inside Addressables bundles. Replacement follows the same Strategy A pattern (`LoadAssetAsync<TMP_FontAsset>`) but requires building a TMP font asset in Unity 2021.3 from a `.ttf` and packaging it in a mod AssetBundle. Out of scope for initial delivery; track as separate issue.

---

## 8. 2D Asset Manifest — What a Mod Must Supply

A total conversion that fully reskins DINO's 2D art supplies:

```
packs/<pack-id>/
  ui/
    menu_bg.png              # 1920×1080 or 2560×1440 — main menu full-bleed background
    loading_bg.png           # 1920×1080 — loading screen background
    loading_bar.png          # ~400×20 px, 9-slice — progress bar fill
    panel.png                # 256×256 min, 9-slice — generic UI panel background
    button_normal.png        # 256×64 min, 9-slice — button idle
    button_hover.png         # 256×64 min, 9-slice — button highlighted
    button_pressed.png       # 256×64 min, 9-slice — button pressed
    republic-emblem.png      # 128×128 — player faction emblem (HUD + score panel)
    cis-emblem.png           # 128×128 — enemy faction emblem
    cursor.png               # 32×32 — software cursor (32-bit RGBA)
    portraits/
      <unit-id>.png          # 64×64 per unit — unit class portrait for build panel
```

All PNGs:
- Format: 32-bit RGBA PNG (Unity `LoadImage` handles both RGB and RGBA; use RGBA for transparency).
- Power-of-two dimensions preferred (not required by Unity, but avoids texture size warnings).
- Background art: 1920×1080 minimum; 2560×1440 for high-DPI monitors.
- Icons: power-of-two, 128×128 or 256×256.
- 9-slice art: declare `border` value (pixels) in pack YAML alongside the PNG path.

---

## 9. New Runtime Components Required

| Component | Location | Purpose |
|-----------|----------|---------|
| `SpriteSwapRegistry` | `src/SDK/Assets/` | Thread-safe registry of `addressKey → Sprite` and `surfaceSlot → Sprite` replacements. Mirrors `AssetSwapRegistry` design. |
| `AddressablesSpritePatch` | `src/Runtime/Bridge/` | Harmony Prefix on `AddressablesImpl.LoadAssetAsync`. |
| `TcSpriteLoader` | `src/Runtime/Bridge/` | Loads PNG bytes → `Texture2D` → `Sprite` from pack directory at scene load. Populates `SpriteSwapRegistry`. |
| `TcUiSpritePass` | `src/Runtime/UI/` | Walks live UGUI hierarchy and applies `TcUiSurfaces` slot replacements (extends `MainMenuThemer`). |
| `TcCursorApplicator` | `src/Runtime/UI/` | Calls `Cursor.SetCursor` and re-applies on scene change. |

Schema changes:
- Extend `TcAssetReplacements` in `src/SDK/Models/TotalConversionManifest.cs` with `TcUiReplacements`, `TcUiSurfaces`, `TcCursorDef`.
- Update `schemas/total-conversion.schema.json` with the new `asset_replacements.ui` shape.

---

## 10. Key Open Questions (Require In-Game Discovery)

These must be answered with a DumpTools or UnityExplorer session before Step 5 can be implemented:

1. **What Addressables keys does DINO use for the main menu background, loading screen, faction emblem sprites?** Run `dinoforge dump --type Sprite` and inspect `catalog.json` m_InternalIds.
2. **Is DINO's UI art in a SpriteAtlas or individual sprites?** Check `dinoforge assetctl asset-library --type SpriteAtlas`.
3. **What is the `AddressablesImpl` type path in DINO's Unity version?** Needed to resolve the Harmony patch target. Likely `UnityEngine.AddressableAssets.AddressablesImpl` (internal) — confirm with `AppDomain.CurrentDomain.GetAssemblies()` scan.
4. **Do `Addressables.LoadAssetAsync` calls fire before or after `SceneManager.sceneLoaded`?** Determines whether Strategy A or B fires first for loading screens.

---

## Summary: Recommended Primary Strategy

**Strategy B (post-load component walk) for Phase 1** delivery of menu background, loading screen, buttons, and panels. This reuses `MainMenuThemer`'s existing scan loop, requires no key-discovery work, and is proven to work against DINO's UGUI structure.

**Strategy A (Addressables Harmony prefix) for Phase 2** — faction emblems, unit portraits, and any sprite whose `Image` component is difficult to identify by GameObject path. Requires the key-discovery pass (Step 5 above).

**Strategy C (Cursor.SetCursor) as standalone** — one-shot at startup, trivial to add.

The two strategies are complementary and non-conflicting: Strategy B catches already-loaded images; Strategy A intercepts future loads. A total conversion will need both for complete coverage.
