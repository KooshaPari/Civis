# Native Mods Page — Blank Menu Diagnosis

**Date**: 2026-05-28
**Branch**: feat/unityexplorer-devtools-20260528
**Symptom**: Clicking the "Mods" button blanks the native DINO main menu (content goes away) but NO mods page appears.

---

## 1. Root Cause

**`NativeMainMenuModMenu.CanUseNativeScreen` is hardcoded to `false`** (`src/Runtime/UI/NativeMainMenuModMenu.cs:15`), which means `ContextualModMenuHost.Toggle()` never calls `_nativeHost.Toggle()` and never shows the `NativeModsPage`. Instead it falls through to `_overlayHost.Toggle()` — the DFCanvas IMGUI/UGUI floating panel.

The floating overlay then fires `ModMenuOverlay.Toggle()` / `ModMenuPanel.Toggle()`, which flips `_visible = true` on the overlay. That overlay is a **DontDestroyOnLoad full-screen canvas** layered on top of every scene. However, the Mods button's click path in `NativeMenuInjector.OnModsButtonClicked` **does NOT hide the DINO native main-menu content container before showing the overlay**. What causes the blank screen is separate and happens regardless of which host is used:

The Mods button in DINO is cloned/injected by `NativeMenuInjector`. DINO's native main-menu buttons, when clicked, call their own selection handler which **hides the main-menu button container** (standard DINO UI transition behaviour — the menu hides itself when navigating to a submenu). The DINOForge click handler fires `_menuHost.Toggle()` *after* DINO's own handler already ran and hid the container. So the sequence is:

1. User clicks the cloned "Mods" button.
2. DINO's native menu system fires first (inherited onClick from the Options clone): it hides the main-menu content container as if navigating to the Options submenu.
3. DINOForge's `RewireModsButtonClick` calls `modsButton.onClick.RemoveAllListeners()` and re-adds only `OnModsButtonClicked` — but this only removes *runtime* listeners; **persistent (serialized) listeners cloned from the donor button are NOT removed by `RemoveAllListeners()`**.
4. `OnModsButtonClicked` calls `_menuHost.Toggle()` → `ContextualModMenuHost.Toggle()` → because `CanUseNativeScreen == false` → `_overlayHost.Toggle()`.
5. The DFCanvas overlay becomes visible, but it is a small floating panel (680×560 px at screen centre), not a full-screen page. It does NOT restore the hidden native main-menu content.
6. The screen shows: blank/dark native menu area (content hidden by step 2) + small overlay panel (often invisible if EventSystem focus is wrong or if the overlay itself was already toggled open and is now toggled closed).

**The combination of two independent bugs produces the "blank menu, no mods page" visual:**

- **Bug A (primary, visible)**: The cloned button's **persistent onClick listener** (serialized into the donor Options button, which opens the DINO Options/Settings screen) is NOT removed by `RemoveAllListeners()`. It fires before DINOForge's listener, hides the native main-menu content, and may also load the native Options screen (which then immediately hides/unloads because DINOForge's click fires next and conflicts). See `CloneAndRegisterModsButton` → `RewireModsButtonClick` — `RemoveAllListeners()` clears only runtime-added (`AddListener`) handlers, not the cloned `PersistentCallCount` serialized handlers on the donor button.
- **Bug B (secondary, invisible)**: `NativeMainMenuModMenu.CanUseNativeScreen` is permanently `false`, so `NativeModsPage` is never instantiated or shown regardless. The `NativeModsPage.Show()` method and its `BuildUI()` exist and are complete, but are unreachable from any click path.

---

## 2. Detailed Call Chain

```
User click
  └─ [DINO persistent onClick on cloned button] ← NOT removed by RemoveAllListeners()
       └─ DINO hides main-menu content container (or navigates to Options screen)
  └─ NativeMenuInjector.OnModsButtonClicked()          [NativeMenuInjector.cs:1241]
       └─ _menuHost.Toggle()                            [NativeMenuInjector.cs:1267]
            └─ ContextualModMenuHost.Toggle()           [ContextualModMenuHost.cs:66]
                 if (_nativeHost.CanUseNativeScreen)    [ContextualModMenuHost.cs:68]
                   → FALSE (hardcoded)                  [NativeMainMenuModMenu.cs:15]
                 else
                   └─ _overlayHost.Toggle()             [ContextualModMenuHost.cs:75]
                        └─ ModMenuPanel.Toggle()  OR  ModMenuOverlay.Toggle()
                             → _visible = !_visible
                             → DFCanvas floating panel appears (if not already open)

NativeModsPage.Show() → NEVER CALLED
NativeModsPage.BuildUI() → NEVER CALLED
```

---

## 3. Why Each Hypothesis Was Eliminated

| Hypothesis | Status | Evidence |
|---|---|---|
| (a) Cloned Options canvas hidden/disabled | Not applicable — `NativeModsPage` is never instantiated at all | No `NativeModsPage shown` in debug log; `CanUseNativeScreen == false` |
| (b) Page parented under hidden content container | Not applicable — page is never created | Same |
| (c) Page Canvas zero size / off-screen | Not applicable | Same |
| (d) Clone stripped Canvas component | Not applicable | Same |
| **(e) Content container hidden (hid too much)** | **CONFIRMED — primary visible symptom** | DINO's serialized persistent listener on the cloned button hides the native menu content; `RemoveAllListeners()` does not clear persistent listeners |

---

## 4. Fix Spec (Step by Step)

### Step 1: Remove persistent onClick listeners from the cloned button

**File**: `src/Runtime/UI/NativeMenuInjector.cs`
**Method**: `RewireModsButtonClick` (~line 1281) and `CloneAndRegisterModsButton` (~line 924)

`Button.onClick.RemoveAllListeners()` only removes listeners added via `AddListener()`. Persistent (serialized) listeners cloned from the donor must be cleared separately via the `UnityEngine.Events.UnityEventBase` internal persistent calls list.

**Add this helper and call it from `RewireModsButtonClick`:**

```csharp
private static void ClearAllButtonListeners(Button button)
{
    // Remove runtime listeners
    button.onClick.RemoveAllListeners();
    // Remove persistent (serialized/cloned) listeners
    // UnityEngine.Events.UnityEventBase exposes GetPersistentEventCount() + RemovePersistentListener(int)
    int persistentCount = button.onClick.GetPersistentEventCount();
    for (int i = persistentCount - 1; i >= 0; i--)
        UnityEngine.Events.UnityEventTools.RemovePersistentListener(button.onClick, i);
}
```

Call `ClearAllButtonListeners(modsButton)` in `RewireModsButtonClick` **before** `button.onClick.AddListener(OnModsButtonClicked)`.

Also call it inside `EnforceModsButtonState` which has the same pattern (`modsButton.onClick.RemoveAllListeners()`).

### Step 2: Enable `NativeMainMenuModMenu.CanUseNativeScreen` and wire `NativeModsPage`

**File**: `src/Runtime/UI/NativeMainMenuModMenu.cs`

The `NativeModsPage` MonoBehaviour is fully implemented with `Show(Canvas, GameObject?)` and `Hide()`. Wire it into `NativeMainMenuModMenu`:

```csharp
public sealed class NativeMainMenuModMenu : IModMenuHost
{
    public bool CanUseNativeScreen => _modsPage != null;  // true once wired

    private NativeModsPage? _modsPage;
    private Canvas? _mainMenuCanvas;
    private GameObject? _mainMenuContent;

    /// <summary>
    /// Called by RuntimeDriver / NativeMenuInjector once the main-menu canvas is found.
    /// Must supply the Canvas to parent into and the content container to hide.
    /// </summary>
    public void Attach(NativeModsPage page, Canvas canvas, GameObject? content)
    {
        _modsPage = page;
        _mainMenuCanvas = canvas;
        _mainMenuContent = content;
    }

    public void Show()
    {
        IsVisible = true;
        _modsPage?.Show(_mainMenuCanvas!, _mainMenuContent);
    }

    public void Hide()
    {
        IsVisible = false;
        _modsPage?.Hide();
    }

    public void Toggle()
    {
        if (IsVisible) Hide(); else Show();
    }

    public void SetPacks(IEnumerable<PackDisplayInfo> packs)
        => _modsPage?.SetPacks(packs.ToArray() /* IReadOnlyList conversion */);
    // ...
}
```

### Step 3: Create and attach `NativeModsPage` from `NativeMenuInjector` after successful injection

**File**: `src/Runtime/UI/NativeMenuInjector.cs`
**After**: `CommitInjectionAndLog(modsButton, attemptId)` succeeds

```csharp
// After injection succeeds, wire the NativeModsPage to the native menu host.
TryAttachNativeModsPage(canvas, attemptId);
```

New method `TryAttachNativeModsPage`:
```csharp
private void TryAttachNativeModsPage(Canvas mainMenuCanvas, long attemptId)
{
    try
    {
        // Find the content container (the direct parent of the injected button's
        // parent — typically the VerticalLayoutGroup holding all menu buttons).
        GameObject? contentContainer = FindMainMenuContent(mainMenuCanvas);

        // AddComponent on the persistent DINOForge root, NOT on mainMenuCanvas
        // (avoid parenting under a scene-unloaded canvas).
        NativeModsPage page = Plugin.PersistentRoot.AddComponent<NativeModsPage>();
        page.SetLogger(_log!);
        page.OnBackClicked = () =>
        {
            // Re-show the native content when user presses Back
            if (contentContainer != null) contentContainer.SetActive(true);
        };

        // Wire callbacks through the menu host
        page.OnPackToggled = (id, val) => _menuHost?.OnPackToggled?.Invoke(id, val);
        page.OnReloadRequested = () => _menuHost?.OnReloadRequested?.Invoke();

        // Attach to NativeMainMenuModMenu so ContextualModMenuHost routes correctly
        if (_menuHost is ContextualModMenuHost cmh)
        {
            // Reach the native host via reflection or expose it via a new internal property
            // (preferred: add internal NativeMainMenuModMenu NativeHost property to ContextualModMenuHost)
            cmh.NativeHost.Attach(page, mainMenuCanvas, contentContainer);
        }

        LogInfo($"[NativeMenuInjector::{_sessionId}] Attempt#{attemptId} NativeModsPage wired — CanUseNativeScreen now true");
    }
    catch (Exception ex)
    {
        LogWarning($"[NativeMenuInjector::{_sessionId}] TryAttachNativeModsPage EXCEPTION: {ex}");
    }
}
```

### Step 4: Implement `FindMainMenuContent`

**File**: `src/Runtime/UI/NativeMenuInjector.cs`

```csharp
/// <summary>
/// Locate the main-menu content container to hide when the mods page is shown.
/// Strategy: the injected Mods button's grandparent (parent of its layout parent)
/// is the highest-level button-list container that should be hidden.
/// Falls back to the sibling parent if grandparent is the Canvas root itself.
/// </summary>
private GameObject? FindMainMenuContent(Canvas mainMenuCanvas)
{
    if (_injectedButton == null) return null;
    Transform? parent = _injectedButton.transform.parent;
    if (parent == null) return null;

    // Try grandparent first (the content area, one level above the button row)
    Transform? grandparent = parent.parent;
    if (grandparent != null && grandparent != mainMenuCanvas.transform)
        return grandparent.gameObject;

    // Fallback: hide just the direct parent (button row)
    return parent.gameObject;
}
```

### Step 5: Expose `NativeHost` on `ContextualModMenuHost`

**File**: `src/Runtime/UI/ContextualModMenuHost.cs`

Add:
```csharp
/// <summary>Exposes the native host for late-wiring after canvas discovery.</summary>
internal NativeMainMenuModMenu NativeHost => _nativeHost;
```

### Step 6: Correct parenting for `NativeModsPage._root`

`NativeModsPage.BuildUI(parent)` creates `_root` as a child of `parent` (the Canvas `transform`). The RectTransform is set to `anchorMin=0,0`, `anchorMax=1,1`, `offset=0` — correct for a full-screen overlay within the canvas.

**However**, `Show()` passes `mainMenuCanvas.transform` as parent (line 109). If `mainMenuContent` is a child of the Canvas, hiding it **also hides** any sibling NativeModsPage root that happens to share the same parent only if the content object is an *ancestor* of the root. Since `_root` is parented directly to the Canvas transform and `mainMenuContent` is parented to the Canvas transform as a sibling (not ancestor), hiding `mainMenuContent` does **not** hide `_root`. This is safe.

**One risk remains**: if `mainMenuCanvas` is a World-Space canvas, the `sortingOrder` of `_root` could be behind other elements. The fix: after `BuildUI`, set the Canvas component's `sortingOrder` on the root if needed:

```csharp
// In BuildUI, after _root is created:
// NativeModsPage renders as a plain RectTransform child (no separate Canvas),
// so sortingOrder is inherited from the parent Canvas. That is correct — we
// want to render within the MainMenu canvas, above its other children.
// Ensure _root is last sibling so it draws on top:
_root.transform.SetAsLastSibling();
```

### Step 7: Verify no GraphicRaycaster is needed

`NativeModsPage._root` does NOT add its own Canvas component — it is a plain `RectTransform` child of the main-menu Canvas. Therefore it uses the main-menu Canvas's existing `GraphicRaycaster`. No additional raycaster is needed. Buttons inside `_root` will receive clicks via the parent Canvas's raycaster as long as the main-menu Canvas already has one (confirmed by `ValidateRaycastAndEventSystem` logging at injection time).

---

## 5. Summary of Files to Change

| File | Change |
|------|--------|
| `src/Runtime/UI/NativeMenuInjector.cs` | (1) Add `ClearAllButtonListeners` helper; call from `RewireModsButtonClick` and `EnforceModsButtonState`. (2) Add `TryAttachNativeModsPage` + `FindMainMenuContent`; call after `CommitInjectionAndLog`. |
| `src/Runtime/UI/NativeMainMenuModMenu.cs` | Replace stub `Show/Hide/Toggle/SetPacks` with real implementation that delegates to `NativeModsPage`; add `Attach(page, canvas, content)` method; change `CanUseNativeScreen` to `=> _modsPage != null`. |
| `src/Runtime/UI/ContextualModMenuHost.cs` | Add `internal NativeMainMenuModMenu NativeHost => _nativeHost;` property. |
| `src/Runtime/UI/NativeModsPage.cs` | Add `_root.transform.SetAsLastSibling()` at end of `BuildUI` so it draws above other Canvas children. No other changes needed — `Show/Hide/BuildUI` are correct. |

---

## 6. What NOT to Change

- `NativeModsPage.BuildUI()` layout, RectTransform anchors, and sizing are correct. The full-screen `anchorMin=0,0 / anchorMax=1,1 / offset=0` approach is the right way to fill the parent canvas. Do not switch to a separate Canvas.
- `NativeModsPage.Show(canvas, content)` correctly hides `mainMenuContent` before showing `_root`. The hide/show logic is sound; the issue is it is never called.
- The DFCanvas overlay fallback (`ModMenuPanel`, `ModMenuOverlay`) should remain the pause-menu path — `NativeModsPage` is for the main-menu scene only.

---

## 7. Log Evidence

The `dinoforge_debug.log` contains **zero occurrences** of `NativeModsPage shown`, `OnModsButtonClicked`, or `CanUseNativeScreen`. This confirms:
- The `OnModsButtonClicked` handler either does not fire (button click captured by DINO's persistent handler first and page navigated away before DINOForge handler runs) **or** fires but the overlay Toggle path is taken silently (the overlay `_visible` flag flips but the overlay panel has zero visual size at the main-menu scene because the DFCanvas root may not be in view over the full-screen native canvas, or is toggled off immediately by a re-entrant click event caused by the DINO persistent listener's navigation).
- `NativeModsPage.Show()` is **never reached**, consistent with `CanUseNativeScreen == false`.
