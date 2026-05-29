# Loading-Screen Takeover System — Design Document

**Status**: Design / Pre-implementation  
**Target**: DINOForge v0.27.0  
**Affects**: `src/Runtime/UI/`, `src/SDK/Models/`, `schemas/pack-manifest.schema.json`

---

## 1. Problem Statement

The existing `ModLoadingOverlay` is a semi-transparent black panel with static text
(title, subtitle, progress line). It covers the game's own loading canvas but does
not brand the experience, does not show tips or lore, and has no mechanism for
total-conversion packs to replace it with their own art.

Goals:

- Replace the plain dark overlay with a full DINOForge-branded loading screen by
  default.
- Allow any pack of type `total_conversion` to declare its own loading-screen art,
  logo, and tips in `pack.yaml` — no C# authoring required.
- Keep the implementation inside the existing BepInEx/netstandard2.0 constraint
  (no TMP, no Addressables at load time).
- Survive the Unity / DINO lifecycle constraints (no `Start()` / `Update()` until
  `PlayerLoop` is injected; `MonoBehaviour.Update()` never fires before the ECS
  world exists).

---

## 2. Default DINOForge Loading Screen

### 2.1 Visual Layout (1920×1080 reference)

```
┌────────────────────────────────────────────────────────────────┐
│  [full-bleed background image: df-loading-bg.png]              │
│                                                                  │
│  ┌──────────────────────────────────┐  (center-of-screen card) │
│  │  [logo sprite: df-logo.png]      │                           │
│  │  DINOForge                       │  48px bold, cyan          │
│  │  Mod Platform                    │  20px regular, white/70%  │
│  │                                  │                           │
│  │  ──────────────────────          │  separator line           │
│  │                                  │                           │
│  │  [rotating tip text]             │  16px, white/85%, italic  │
│  │  "Tip: Pack load order affects…" │                           │
│  │                                  │                           │
│  │  [progress bar]  ████░░  3 / 9   │                           │
│  │  Loading pack: Modern Warfare    │  14px, gray               │
│  └──────────────────────────────────┘                           │
│                                                                  │
│  [spinner: df-spinner.png, rotating at bottom-right of card]   │
│  v0.27.0                           │  12px, bottom-right corner │
└────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Breakdown

| Element | Implementation | Notes |
|---|---|---|
| Full-bleed background | `Image` on a `ScreenSpaceOverlay` Canvas, `sortingOrder 9998` | Lower than `DFCanvas` (32767) so regular HUD still renders on top once loading ends |
| Background texture | `Texture2D` loaded via `File.ReadAllBytes` + `LoadImage` from `BepInEx/plugins/dinoforge-ui-assets/loading-bg.png` | Fallback: solid `#0D1117` if file absent |
| Logo sprite | Same disk-load pattern via `UiAssets` | Fallback: `DINOForge` text rendered in cyan Arial |
| Progress bar | Two stacked `Image` objects: track + fill, fill `fillMethod = Horizontal` | Animated with `Mathf.Lerp` to avoid jarring jumps |
| Tip text | `Text` component, 16px italic white; rotates on a 6-second timer | Timer driven by the `LoadingScreenController` coroutine (see §4) |
| Spinner | A `RawImage` with a `Texture2D` rotated each `Update()` tick | Tick driven by `IEnumerator` since `Update()` is not reliable; use `WaitForEndOfFrame` inside the coroutine |
| Version label | `Text`, bottom-right, `PluginInfo.VERSION` | |

### 2.3 Canvas Hierarchy

```
DINOForge_LoadingScreen (DontDestroyOnLoad, HideFlags.HideAndDontSave)
  └── Canvas (ScreenSpaceOverlay, sortingOrder 9998)
        ├── Background (Image, stretch-fill)
        ├── Card (VerticalLayoutGroup, anchored center, 760×420)
        │     ├── Logo (Image or Text)
        │     ├── TitleText (Text)
        │     ├── SubtitleText (Text)
        │     ├── Separator (Image, 1px height, white/30%)
        │     ├── TipText (Text)
        │     ├── ProgressTrack (Image)
        │     │     └── ProgressFill (Image, fillMethod Horizontal)
        │     ├── ProgressLabel (Text)
        │     └── SpinnerHost (RawImage, rotating)
        └── VersionLabel (Text, bottom-right anchor)
```

---

## 3. Per-Mod Themed Loading Screen

### 3.1 pack.yaml Schema Addition

A new optional top-level key `loading_screen` is added to the pack manifest:

```yaml
# pack.yaml
id: warfare-starwars
name: Star Wars — Clone Wars
version: 1.2.0
type: total_conversion
# ...

loading_screen:
  background: assets/loading/hyperspace-bg.png   # relative to pack root; 1920×1080 PNG
  logo:        assets/loading/sw-logo.png         # relative to pack root; max 512×256 PNG
  tips:
    - "The Republic's clone army was grown on Kamino in just ten years."
    - "Battle droids are cheap — Clone Troopers are worth every credit."
    - "T-series tactical droids can override any droid unit on the field."
    - "AT-TE walkers fire both forward and backward — flank with caution."
  tip_rotation_seconds: 7       # optional; default 6
  accent_color: "#FFE81F"       # optional hex; tints progress bar + separator; default DINOForge cyan
  overlay_opacity: 0.90         # optional 0.0–1.0; background image overlay alpha; default 0.95
```

**Vanilla/non-total-conversion packs** MUST NOT declare `loading_screen`; the
validator rejects the key on packs where `type != total_conversion`.

### 3.2 Pack-Manifest Schema Changes (`schemas/pack-manifest.schema.json`)

Add after the existing `loads` property:

```json
"loading_screen": {
  "type": "object",
  "description": "Optional themed loading screen for total_conversion packs.",
  "properties": {
    "background": {
      "type": "string",
      "description": "Path (relative to pack root) to a 1920×1080 PNG background image."
    },
    "logo": {
      "type": "string",
      "description": "Path (relative to pack root) to a max-512×256 PNG logo image."
    },
    "tips": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1,
      "description": "List of rotating tip/lore strings."
    },
    "tip_rotation_seconds": {
      "type": "number",
      "minimum": 2,
      "maximum": 30,
      "default": 6,
      "description": "Seconds between tip rotations."
    },
    "accent_color": {
      "type": "string",
      "pattern": "^#[0-9A-Fa-f]{6}$",
      "description": "Hex color for progress bar and separator. Default: DINOForge cyan #33CCFF."
    },
    "overlay_opacity": {
      "type": "number",
      "minimum": 0,
      "maximum": 1,
      "default": 0.95,
      "description": "Alpha of the background image overlay (0 = transparent, 1 = opaque)."
    }
  },
  "additionalProperties": false
}
```

Add to `if/then` conditional validation or use a discriminator check:

```json
"if": {
  "properties": { "type": { "not": { "const": "total_conversion" } } }
},
"then": {
  "properties": { "loading_screen": false }
}
```

### 3.3 SDK Model Changes

Add to `src/SDK/Models/PackManifest.cs`:

```csharp
/// <summary>Optional themed loading screen declared by total_conversion packs.</summary>
[YamlMember(Alias = "loading_screen")]
public LoadingScreenConfig? LoadingScreen { get; set; }
```

New file `src/SDK/Models/LoadingScreenConfig.cs`:

```csharp
/// <summary>
/// Declares a pack-authored loading screen for total_conversion packs.
/// Resolved by <c>LoadingScreenController</c> in the Runtime when the
/// pack is active before the first ECS world becomes ready.
/// </summary>
public sealed class LoadingScreenConfig
{
    [YamlMember(Alias = "background")]   public string? Background { get; set; }
    [YamlMember(Alias = "logo")]         public string? Logo { get; set; }
    [YamlMember(Alias = "tips")]         public List<string> Tips { get; set; } = new();
    [YamlMember(Alias = "tip_rotation_seconds")] public float TipRotationSeconds { get; set; } = 6f;
    [YamlMember(Alias = "accent_color")] public string? AccentColor { get; set; }
    [YamlMember(Alias = "overlay_opacity")] public float OverlayOpacity { get; set; } = 0.95f;
}
```

---

## 4. Lifecycle & Injection Points

### 4.1 DINO Scene Names (known from iter-144 probes)

| Scene | Purpose | When DINOForge cares |
|---|---|---|
| `(empty / Boot)` | Unity splash / app init | Plugin.Awake fires here |
| `InitialGameLoader` | Game's own loading screen (asset catalog warm-up) | **Primary injection point** |
| `MainMenu` | Main menu | Loading screen must be hidden by this point |
| `GameWorld` / `Gameplay` | Active game map | Not relevant to loading screen |

The `activeSceneChanged` subscription installed by `Plugin.StartResurrectionWatcher()`
already fires reliably on all transitions (confirmed iter-144). This is the
authoritative hook.

### 4.2 Injection Sequence

```
Plugin.Awake()
  │
  ├── Create DINOForge_Root (DontDestroyOnLoad)
  ├── Add RuntimeDriver
  └── Subscribe activeSceneChanged → OnActiveSceneChanged
        │
        ▼ (fires for each scene)
  OnActiveSceneChanged(old, new)
        │
        ├── if newScene.name == "InitialGameLoader" OR oldScene is empty
        │     → LoadingScreenController.ShowForScene("InitialGameLoader")
        │
        └── if newScene.name == "MainMenu"
              → LoadingScreenController.FadeOut()
```

**`RuntimeDriver.InitializeRoutine()`** (the existing coroutine) should call
`LoadingScreenController.Show()` at its very first `yield return null`, before
`ModPlatform.Initialize()`, and call `LoadingScreenController.SetPackProgress()`
during pack enumeration. On `ModPlatform.OnPacksLoaded`, it calls
`LoadingScreenController.BeginFadeOut()`.

Specifically, the recommended insertion point in `InitializeRoutine` is:

```
yield return null  // (existing: first yield)
→ INSERT: LoadingScreenController.EnsureVisible()    // creates overlay if not yet created
yield return null
→ "ModPlatform.Initialize" phase
→  during pack loop: LoadingScreenController.SetPackProgress(i, total, packName)
yield return null
→ (after packs loaded) LoadingScreenController.BeginFadeOut()
```

### 4.3 Total-Conversion Theme Resolution

Before `LoadingScreenController.EnsureVisible()` creates the canvas, it queries
`ModPlatform` (if already initialized) or reads pack YAML directly for any
`type: total_conversion` pack that declares `loading_screen`. Priority:

1. First active `total_conversion` pack with a valid `loading_screen.background`.
2. Default DINOForge theme if none found.

Since `ModPlatform.Initialize` happens **after** the first show, the controller
must perform a **lightweight pre-scan**: read only the `pack.yaml` of packs in the
packs directory without full content loading, extract `type` and `loading_screen`
fields. This is ~5ms for 9 packs and does not require `ContentLoader`.

```csharp
// In LoadingScreenController.EnsureVisible()
LoadingScreenTheme theme = ThemeScanner.ScanForActiveTheme(packsDir);
// ThemeScanner: reads each pack.yaml via YamlDotNet, checks type == total_conversion,
// returns first LoadingScreenConfig found (respecting disabled_packs.json).
```

---

## 5. Native Loading Background — Swap vs. Overlay

### 5.1 What the Game Does

DINO's `InitialGameLoader` scene contains a Unity `Canvas` with a background
`Image` using a sprite from the Addressables catalog key
`Assets/Resources/UI/LoadingScreen/...` (exact key TBD — requires in-game probe
with UnityExplorer). The game fades this canvas in/out independently.

### 5.2 Overlay Strategy (Recommended — Tier 1)

**Place our canvas at `sortingOrder 9998`**, one below `DFCanvas` (32767) but
above the game's loading canvas (typically 0–100). Our full-bleed background
image paints over the game's art entirely. No Addressables manipulation; no
Harmony patch on the loader scene.

Advantages:
- Works regardless of the game's loading canvas structure.
- Survives catalog changes between DINO game updates.
- No risk of breaking vanilla loading flow.
- Fully testable without a live game instance.

Disadvantage:
- The game's loading canvas still renders behind ours (wasted GPU fill). Negligible
  on the loading screen (no frame-rate concern).

### 5.3 Native Swap Strategy (Tier 2 — Optional Enhancement)

If a future iteration wants to fully replace the native canvas (e.g., to remove
the game's spinner or progress text):

1. In `OnActiveSceneChanged` for `InitialGameLoader`, traverse
   `Resources.FindObjectsOfTypeAll<Canvas>()` and disable the game's loading
   Canvas (`canvas.enabled = false`).

   **Caveat**: `Resources.FindObjectsOfTypeAll` from a background thread deadlocks
   during asset loading (documented in CLAUDE.md and Memory). This call must happen
   on the main thread — the `activeSceneChanged` handler fires on the main thread,
   so it is safe.

2. Use a Harmony postfix on `AsyncOperation.allowSceneActivation` setter or on
   `UnityEngine.UI.Image.set_sprite` to intercept the background texture assignment.

   **Risk**: these are Unity internals that change between versions. The postfix
   approach is fragile across DINO updates.

**Recommendation**: implement Tier 1 only. Leave a `// TODO(loading-screen-tier2)`
comment for the Addressables/Harmony approach. Document the scene canvas path
once probed with UnityExplorer.

---

## 6. Tip-Text System

### 6.1 Built-in DINOForge Tips

Stored as a static `string[]` in `LoadingScreenController`. Categories:

**Platform tips** (always shown in default theme):
- "Packs load in dependency order — conflicts are detected automatically."
- "Press F9 to open the mod menu at any time during gameplay."
- "Hot reload watches your pack files — save a YAML to see changes live."
- "Use `dinoforge verify-pack` in the CLI to validate before deploying."
- "Pack IDs must be unique and lowercase — hyphens and underscores allowed."
- "Total-conversion packs can declare their own loading screen in pack.yaml."

**Gameplay-lore tips** (contextual; shown when a warfare/vanilla pack is active):
- "The enemy never stops. Upgrade walls before wave 5 or fall."
- "Pikemen are cost-efficient against cavalry — don't ignore them."
- "Resource delivery buildings increase income without increasing upkeep."
- "Tower upgrades stack multiplicatively — prioritize range first."

### 6.2 Faction-Specific Lore Tips

For the Star Wars pack (and future theme packs), tips come from `pack.yaml`
directly (see §3.1). No C# code is needed per faction — the pack author writes
the tips in YAML.

For vanilla factions (Enemy, Player), the built-in gameplay tips above apply.

### 6.3 Tip Rotation Algorithm

```csharp
// In LoadingScreenController:
private string[] _tips;         // merged: builtin + pack.tips
private int _tipIndex;
private float _tipTimer;
private float _tipRotationSeconds = 6f;

// Called every WaitForEndOfFrame in the animation coroutine:
_tipTimer += Time.unscaledDeltaTime;
if (_tipTimer >= _tipRotationSeconds)
{
    _tipIndex = (_tipIndex + 1) % _tips.Length;
    SetTipText(_tips[_tipIndex]);
    _tipTimer = 0f;
    // Optional: brief fade-out/in on _tipCanvasGroup
}
```

`Time.unscaledDeltaTime` is used so tip rotation is not affected by `Time.timeScale`
(which DINO may set to 0 during loading).

---

## 7. Asset Manifest

All assets are shipped in `BepInEx/plugins/dinoforge-ui-assets/loading/` and
deployed by the `DeployUiAssets` MSBuild target (same target that deploys
`kenney-input-prompts/`).

| File | Resolution | Format | Notes |
|---|---|---|---|
| `loading-bg.png` | 1920×1080 | PNG-24, no alpha | Default DINOForge background; dark tech/hex grid aesthetic |
| `loading-bg-overlay.png` | 1920×1080 | PNG-32 | Optional vignette/gradient overlay (alpha layer); composited on top of bg |
| `df-logo.png` | 512×256 | PNG-32 | DINOForge wordmark + dino icon; white on transparent |
| `spinner.png` | 128×128 | PNG-32 | Single-frame spinner; rotated by code at ~1.5 rot/s |
| `progress-track.png` | 1×8 | PNG-32 | Rounded rectangle, 9-slice; mid-gray |
| `progress-fill.png` | 1×8 | PNG-32 | Rounded rectangle, 9-slice; accent color |

**Per-pack loading assets** (declared in `loading_screen:` of `pack.yaml`):

| Expected file | Max size | Format | Notes |
|---|---|---|---|
| `background` path | 1920×1080 | PNG-24 or JPEG | Loaded via `File.ReadAllBytes` + `Texture2D.LoadImage` |
| `logo` path | 512×256 px | PNG-32 | Overlaid on card; transparent background required |

**Loading order for textures** (avoids blocking the main thread):

1. `LoadingScreenController` starts a `System.Threading.Tasks.Task` to read bytes
   from disk.
2. On completion (checked each `WaitForEndOfFrame` yield), calls
   `Texture2D.LoadImage(bytes)` on the main thread.
3. If the file read fails or is too slow (>3s), falls back to the default theme
   asset.

---

## 8. New Class: `LoadingScreenController`

Location: `src/Runtime/UI/LoadingScreenController.cs`

Responsibilities:
- Create and own the `DINOForge_LoadingScreen` GameObject.
- Run the tip-rotation coroutine via `StartCoroutine` on the hosting `RuntimeDriver`.
- Expose `Show()`, `SetPackProgress(int, int, string)`, `SetMessage(string)`,
  `BeginFadeOut()`.
- Call `ThemeScanner.ScanForActiveTheme(packsDir)` to select the active theme before
  building the UI.
- Manage the `CanvasGroup` fade-out (0.5s, same pattern as existing
  `ModLoadingOverlay`).
- Destroy the `DINOForge_LoadingScreen` GameObject after fade-out.

`LoadingScreenController` replaces the existing `ModLoadingOverlay` — do not keep
both. The `RuntimeDriver` field `_loadingOverlay` becomes `_loadingScreenController`.

### 8.1 ThemeScanner (static utility)

Location: `src/Runtime/Loading/ThemeScanner.cs`

```csharp
internal static class ThemeScanner
{
    /// <summary>
    /// Scans packs directory for the first active total_conversion pack
    /// with a declared loading_screen. Returns null if none found.
    /// </summary>
    internal static LoadingScreenTheme? ScanForActiveTheme(string packsDir, ISet<string> disabledPackIds);
}

internal sealed class LoadingScreenTheme
{
    public string PackRootDir { get; init; }
    public LoadingScreenConfig Config { get; init; }
}
```

The scanner reads ONLY `pack.yaml` files (not content YAMLs). It uses YamlDotNet
`Deserializer` with a partial deserialization (only `id`, `type`, `loading_screen`
fields populated). This keeps the pre-scan fast.

---

## 9. Integration with Existing Code

### 9.1 RuntimeDriver Changes

- Replace `ModLoadingOverlay? _loadingOverlay` with
  `LoadingScreenController? _loadingScreenController`.
- In `InitializeRoutine`, at the first yield: call
  `_loadingScreenController = LoadingScreenController.Create(gameObject, packsDir, disabledPackIds)`.
- Wire `ModPlatform.OnPackProgress` (new event, analogous to `OnHudCountsChanged`)
  to `_loadingScreenController.SetPackProgress(i, total, name)`.
- After all packs loaded: `_loadingScreenController?.BeginFadeOut()`.

### 9.2 Plugin.OnActiveSceneChanged Changes

Add explicit show/hide triggers:

```csharp
private static void OnActiveSceneChanged(Scene oldScene, Scene newScene)
{
    // ... existing EventSystem reconcile, KeyInputSystem.RecreateInCurrentWorld ...

    // Loading screen visibility:
    if (newScene.name == "InitialGameLoader" || string.IsNullOrEmpty(oldScene.name))
        _loadingScreenControllerRef?.EnsureVisible();

    if (newScene.name == "MainMenu")
        _loadingScreenControllerRef?.BeginFadeOut();
}
```

`_loadingScreenControllerRef` is a static reference set when the controller is
created (same pattern as `SharedBridgeServer`).

---

## 10. Injection-Point Recommendation

**Use `SceneManager.activeSceneChanged` (already subscribed in `Plugin.cs`) as the
primary trigger**, not a Harmony patch on the loader scene. This is the lowest-risk,
most maintainable path:

- The `activeSceneChanged` handler fires on the main thread.
- It already fires reliably for all DINO scene transitions (confirmed iter-144).
- It survives `RuntimeDriver` destruction (static subscription on `Plugin` class).
- No Addressables or Harmony involvement means zero breakage risk from DINO updates.

The loading screen canvas is created (or made visible) in the `InitialGameLoader`
scene change callback and faded out in the `MainMenu` scene change callback. The
`RuntimeDriver.InitializeRoutine` coroutine drives progress updates independently.

This gives two independent hide triggers (scene change + pack-load completion),
so the overlay is guaranteed to disappear even if pack loading finishes before
the `MainMenu` scene fires.

---

## 11. Acceptance Criteria

- [ ] Default DINOForge branded loading screen renders during plugin init
      (dark background image, logo, progress bar, tip text, spinner visible).
- [ ] `SetPackProgress(3, 9, "Modern Warfare")` updates the progress bar and label.
- [ ] Tips rotate on a 6-second timer without `Thread.Sleep`.
- [ ] `BeginFadeOut()` fades the canvas to alpha 0 over 0.5s then destroys it.
- [ ] A `total_conversion` pack with `loading_screen:` declared in `pack.yaml`
      replaces the background image, logo, and tips with its own assets.
- [ ] A `content` pack with `loading_screen:` in `pack.yaml` fails
      `PackCompiler validate` with a schema error.
- [ ] If `background` asset file is missing, falls back to default DINOForge theme
      without exception.
- [ ] Loading screen does not appear after `MainMenu` scene is active.
- [ ] No `Update()` / `Start()` / coroutine dependency; all timing via
      `WaitForEndOfFrame` yields and `Time.unscaledDeltaTime`.
- [ ] `dotnet test src/DINOForge.sln` passes after implementation
      (new unit tests for `ThemeScanner` and `LoadingScreenConfig` validation).
