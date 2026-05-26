# Decomposition Map: NativeMenuInjector.InjectButton Method

## Current State

| Property | Value |
|----------|-------|
| **File** | `src/Runtime/UI/NativeMenuInjector.cs` |
| **Method** | `InjectButton(Button settingsButton, long attemptId)` |
| **Line Range** | 487–788 (inclusive) |
| **Actual LOC** | 302 lines |
| **Method Signature** | `private void InjectButton(Button settingsButton, long attemptId)` |
| **Body Size** | 298 lines (accounting for method header) |

---

## Identified Code Clusters

The `InjectButton` method contains **7 distinct cohesive clusters** separated by logical concerns:

### Cluster 1: Null Guard & Clone Source Resolution (Lines 488–517)
- **Purpose**: Validate inputs and decide cloning strategy
- **Shared state touched**: `_allOptionsButtons` (read-only)
- **Local state**: `modsButton`, `cloneSource`, `positionAfterSibling`
- **Notes**: Sets up decision tree for 1-vs-many Options buttons

### Cluster 2: Duplicate-Prevention Guard (Lines 519–539)
- **Purpose**: Check if a Mods button already exists in parent; if so, re-enforce its state
- **Shared state touched**: `_injectedButton`, `_injected`
- **Return path**: Early exit if button exists + already-active
- **Calls helper**: `EnforceModsButtonState()`

### Cluster 3: Button Cloning & Visual Style Sync (Lines 541–570)
- **Purpose**: Clone the source button, register for text interception, sync visuals
- **Calls helpers**: `NativeUiHelper.CloneButton()`, `SyncButtonVisualStyle()`
- **State mutation**: `RepurposedModsButtonGoName` (public static)
- **Notes**: Handles both legacy Text and TMPro text components via reflection

### Cluster 4: Button Positioning & Layout Rebuild (Lines 572–633)
- **Purpose**: Position Mods button before/after sibling, force canvas layout rebuild
- **Calls helpers**: `NativeUiHelper.PositionAfterSibling()`, `NativeUiHelper.PositionBeforeSibling()`, `LayoutRebuilder.ForceRebuildLayoutImmediate()`
- **Canvas state**: Calls `Canvas.ForceUpdateCanvases()`
- **Notes**: Multi-level hierarchy traversal (parent + grandparent)

### Cluster 5: Button Interactivity Setup (Lines 635–653)
- **Purpose**: Ensure button is active, interactable, and CanvasGroup is not blocking
- **Local checks**: `gameObject.SetActive()`, `interactable`, `CanvasGroup` diagnostics
- **Notes**: Mostly diagnostic logging; minimal state mutation

### Cluster 6: Raycast & EventSystem Diagnostics (Lines 655–766)
- **Purpose**: Verify button is raycast-able and configure EventSystem navigation isolation
- **Complexity**: 7 sub-steps (6.1–6.8), deep hierarchy checks, exception handling
- **State mutation**: `Navigation.mode = None`, optional `GraphicRaycaster.enabled = true`
- **Risk**: EventSystem fixes wrapped in try/catch (exception swallow Pattern #104)

### Cluster 7: Final Button State Verification & Commit (Lines 768–788)
- **Purpose**: Comprehensive post-injection state dump, then set `_injected` / `_injectedButton` flags
- **State mutation**: `_injected = true`, `_injectedButton = modsButton`
- **Logging**: 7-line final state dump (targetGraphic.raycastTarget, navigation.mode, etc.)
- **No return**: Falls through to implicit return (success)

---

## Proposed Helper Methods

| Cluster | Helper Name | Parameters | Return | Purpose |
|---------|------------|-----------|--------|---------|
| 1 | `ResolveCloneSourceButton()` | `Button settingsButton, long attemptId` | `(Button cloneSource, Button? positionAfterSibling)` | Decide whether to clone from settingsButton or last Options button |
| 2 | `CheckForExistingModsButton()` | `Button cloneSource, long attemptId` | `bool isExisting` | Return true if already-injected Mods button found + re-enforced |
| 3 | `CloneAndRegisterModsButton()` | `Button cloneSource, long attemptId` | `Button modsButton` | Clone button, register name for text interception, sync visuals |
| 4 | `PositionAndRebuildLayout()` | `Button modsButton, Button settingsButton, Button? positionAfterSibling, long attemptId` | `void` | Position button (before/after sibling), force layout rebuild on parent hierarchy |
| 5 | `EnsureButtonInteractivity()` | `Button modsButton, long attemptId` | `void` | Activate button, set interactable, configure CanvasGroup |
| 6 | `ValidateRaycastAndEventSystem()` | `Button modsButton, Button settingsButton, long attemptId` | `void` | Check raycast targets, parent CanvasGroups, GraphicRaycaster, navigation isolation |
| 7 | `CommitInjectionAndLog()` | `Button modsButton, long attemptId` | `void` | 7-line state verification dump, set `_injected` / `_injectedButton` flags |

---

## Shared State (Constraints)

Fields that **multiple clusters touch**:

1. **`_injectedButton`** — set in Cluster 2 (early exit), Cluster 7 (final commit)
2. **`_injected`** — read in Cluster 2 guard, set in Cluster 7 (final commit)
3. **`RepurposedModsButtonGoName`** — set in Cluster 3 (for Harmony patch), may be read by ModsButtonTextPatch
4. **`_allOptionsButtons`** — read in Cluster 1 (decision), referenced in Cluster 2 guard (if multiple)
5. **`_menuHost`** — not touched in `InjectButton`, but available for future callback wiring (Cluster 7 logging accesses it)

**Mutation ordering constraint**: `_injected` and `_injectedButton` MUST be set together at the end (Cluster 7) — setting one before the other creates a race where `Update()` observes partial injection state.

---

## Post-Refactor Method Size Estimate

Current: **302 lines**  
Post-refactor (expected): **~45 lines**

```csharp
private void InjectButton(Button settingsButton, long attemptId)
{
    try
    {
        if (settingsButton == null) { LogWarning(...); return; }
        LogInfo($"InjectButton starting...");

        if (_injected && _injectedButton != null) {
            LogInfo($"Already injected + button alive, skipping");
            return;
        }

        var (cloneSource, positionAfterSibling) = ResolveCloneSourceButton(settingsButton, attemptId);

        if (CheckForExistingModsButton(cloneSource, attemptId))
            return;  // Early exit; CheckForExistingModsButton re-enforces state

        Button modsButton = CloneAndRegisterModsButton(cloneSource, attemptId);
        if (modsButton == null) { LogWarning(...); return; }

        PositionAndRebuildLayout(modsButton, settingsButton, positionAfterSibling, attemptId);
        EnsureButtonInteractivity(modsButton, attemptId);
        ValidateRaycastAndEventSystem(modsButton, settingsButton, attemptId);
        
        RewireModsButtonClick(modsButton, attemptId);  // Already exists

        CommitInjectionAndLog(modsButton, attemptId);
    }
    catch (Exception ex)
    {
        LogWarning($"[...] InjectButton EXCEPTION: {ex.Message}\n{ex.StackTrace}");
    }
}
```

---

## Risks & Non-Negotiable Behaviors

A future refactor MUST preserve:

1. **Button cloning order**: Always clone from source button (never repurpose the original Options button).
2. **Position precedence**: When 2+ Options buttons exist, Mods button AFTER last Options (not before); when 1 or 0 exist, Mods button BEFORE Settings.
3. **Exception propagation**: All exceptions must be caught and logged; method never throws to caller (graceful failure pattern).
4. **Text enforcement order**: Text must be set AFTER cloning (not before), because clone inherits source text ("Options").
5. **Navigation isolation**: EventSystem.currentSelectedGameObject must NOT be force-selected into the Mods button — this couples it to native menu flows.
6. **Raycast diagnostics**: Even if all checks pass, GraphicRaycaster on parent canvas may be disabled — must be flagged/fixed.
7. **State mutation atomicity**: `_injected` and `_injectedButton` set together at end, OR both stay false if any intermediate step fails.
8. **Layout rebuild scope**: Must rebuild parent AND grandparent RectTransform, plus call `Canvas.ForceUpdateCanvases()` (not just one level).

---

## Test Coverage

**Current test count**: 0 (no existing unit or integration tests for `NativeMenuInjector`)

**To enable verification, a future refactor should**:
1. Extract helpers to `private` (keep internal `TryInjectMenuButton()` public for testing)
2. Add test fixtures for:
   - Null settingsButton guard
   - Existing Mods button re-enforcement path
   - 1-button positioning (before Settings)
   - 2+ button positioning (after last Options)
   - EventSystem exception handling
   - CanvasGroup/GraphicRaycaster state checks

**Pre-refactor baseline**: Zero test methods, method exercised only via manual F10 toggle in-game.

---

## Implementation Notes

- Each helper is `private` (no external API surface).
- All logging/diagnostics preserved from original (copy line-for-line into helpers).
- No behavioral change — pure mechanical refactor for readability.
- Pair with #206 LogWarning cleanup (Pattern #74 — convert 4 `.Message` sites to full ex.ToString()).
- Consider adding `[MethodImpl(MethodImplOptions.AggressiveInlining)]` to small helpers (Clusters 5, 7) if hot-path optimization desired (measure first).

