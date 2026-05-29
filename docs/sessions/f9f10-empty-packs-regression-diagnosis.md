# F9/F10 Empty Packs Regression Diagnosis

**Date**: 2026-05-28  
**Branch**: feat/unityexplorer-devtools-20260528  
**Deployed DLL hash (MD5)**: `67254532F266C089713FC66CFF863542`  
**Deployed DLL timestamp**: 2026-05-28 04:04 AM PDT (11:04 UTC)

---

## Stale Binary Context (Critical)

The deployed DLL was built at **11:04 UTC** on 2026-05-28. The following commits modified the relevant UI files BEFORE that timestamp and ARE in the deployed binary:

| Commit | PDT Time | UTC Time | Contents |
|--------|----------|----------|----------|
| `ff1455b2` | 01:52 PDT | 08:52 UTC | F9/F10 swap fix (#895) |
| `941d0d44` | 02:25 PDT | 09:25 UTC | Loading skeleton overlay (#906) |
| `9d59d631` | 03:08 PDT | 10:08 UTC | Search/filter/sort for F10 panel |
| `d8c03f5e` | 03:44 PDT | 10:44 UTC | Keyboard navigation in F10 mod menu |
| `880af1f3` | 03:54 PDT | 10:54 UTC | BepInEx.AssemblyPublicizer integration |

The following commits are NOT in the deployed binary:

| Commit | PDT Time | Contents |
|--------|----------|----------|
| `6dd47121` | 04:10 PDT | Conflict resolution buttons (#903) |
| `8de6a5cb` | later | Zebra rows, gradient bg |
| `b2d220ae` | later | Template/tutorial pack |
| `ae912208` | later | Pack signing |
| `313834bd` | 05:22 PDT | Profiles save/load (#918) |
| `320d3245` | later | Telemetry F10 tab (#921) |
| `42a889ba` | later | i18n wiring |
| `8416db5b` | later | Security patches |

---

## Symptom 1: F9/F10 menus appear swapped

### Root Cause
**NOT a code regression.** Commit `ff1455b2` (01:52 PDT) correctly assigned:
- **F9** → `_dfCanvas.ToggleModMenu()` (Mod Menu)
- **F10** → `_dfCanvas.ToggleDebug()` (Debug Panel)

This commit IS in the deployed DLL. The prior state (before `ff1455b2`) had them reversed. The user's mental model may still reflect the pre-fix F10=ModMenu expectation.

**File:Line**: `src/Runtime/Plugin.cs:1134-1168`  
**Introducing Commit**: N/A (perception regression, not a code regression)

### Fix Spec
None needed. The mapping is correct. Update user docs/tooltip if the user expects F10=ModMenu, or swap back if the original expectation is the canonical design.

---

## Symptom 2: F9 mod menu shows EMPTY loaded packs

### Root Cause
**Commit `9d59d631` introduced a two-part defect:**

#### Part A: FilterContainer overlaps ScrollRect viewport (visual occlusion)

`BuildListFilters(pane.transform)` inserts a FilterContainer (`LayoutElement.preferredHeight = 140f`) into the ListPane VLG **before** the ScrollRect. The ListPane VLG has `childControlHeight = false`, which means it stacks children positionally but does not resize them. The ScrollRect is configured with parent-fill anchors (`anchorMin = Vector2.zero, anchorMax = Vector2.one, offsetMax = (0, -32)`) which anchor it to the **full ListPane** rectangle, NOT to the space below the FilterContainer.

Result: the ScrollRect's top edge starts at `ListPane.top - 32px` (just below the ListHeader), while the FilterContainer occupies `ListPane.top` through `ListPane.top + 140px`. The FilterContainer visually overlays the first 140px of the scroll viewport, hiding the top 3-4 pack items (items 0–3 in the 40px-per-item list). The pack items ARE created (`_listContent.childCount=9` confirmed by log), but the top items are occluded.

**File:Line**: `src/Runtime/UI/ModMenuPanel.cs:690` (`BuildListFilters(pane.transform)` call in `BuildListPane`)  
**Fix**: Set the ScrollRect's `offsetMin` to start BELOW the FilterContainer: `scrollRt.offsetMin = new Vector2(0f, 0f)` → `scrollRt.offsetMin = new Vector2(0f, -(HeaderHeight + FilterContainerHeight))` OR restructure `BuildListPane` to nest the FilterContainer and ScrollRect in a container that uses `childControlHeight = true`, `childForceExpandHeight = false` so the VLG properly stacks them. Alternatively, change the ScrollRect anchors from fill (`0,0→1,1`) to top-anchored and add an explicit negative offsetMin of `172f` (32 header + 140 filters):

```csharp
// In BuildListPane, after BuildListFilters and MakeScrollView:
scrollRt.anchorMin = new Vector2(0f, 0f);
scrollRt.anchorMax = new Vector2(1f, 1f);
scrollRt.offsetMin = new Vector2(0f, 0f);
scrollRt.offsetMax = new Vector2(0f, -(ListHeaderHeight + FilterContainerHeight));
// where ListHeaderHeight = 32f, FilterContainerHeight = 140f (or 212f in profiles build)
```

#### Part B: Pack counter text always shows "0 of N" (never updated on initial load)

`_filteredIndices` is populated only by `ApplyFilters()`, which is only called from dropdown/search `onValueChanged` callbacks. `SetPacks()` calls `RebuildPackList()` (renders all packs directly) but does NOT call `ApplyFilters()`, so `_filteredIndices` remains empty. `_listCounterText` is set only in `ApplyFilters()`, so it always reads "0 of M" until the user manually changes a filter.

**File:Line**: `src/Runtime/UI/ModMenuPanel.cs:234-270` (`SetPacks` method)  
**Fix**: Call `ApplyFilters()` at the END of `SetPacks()` instead of `RebuildPackList()`:

```csharp
public void SetPacks(IEnumerable<PackDisplayInfo> packs)
{
    _presenter.SetPacks(packs);
    // ... existing logging ...
    ApplyFilters();          // replaces RebuildPackList() — ApplyFilters calls RebuildFilteredPackList internally
    RefreshDetail();
}
```

This also ensures `_filteredIndices` is always consistent with the current packs. Requires `ApplyFilters()` to handle empty packs gracefully (it already does).

**Introducing Commit**: `9d59d631 feat(ui): mod browser search/filter/sort for F10 panel`

---

## Symptom 3: F10 debug panel shows empty output

### Root Cause
**Two independent causes:**

#### Cause A: F10 opens DebugPanel which requires `SetModPlatform` to populate

`DebugPanel.RefreshContent()` returns early if `_modPlatform == null` (line `src/Runtime/UI/DebugPanel.cs:275`). `SetModPlatform` is called from `WireUguiToModPlatform` only when `_dfCanvas.DebugPanel != null`. If the user opens the debug panel BEFORE `WireUguiToModPlatform` completes (or before Step 7 MainMenu-mode pack load), `_modPlatform` is null → the panel shows only the "Platform Status" header section with "ModPlatform: not available" rather than empty. This could appear near-empty depending on the section toggle state (`_showPlatform = true` but expanded content is just the "not available" text).

In practice, `WireUguiToModPlatform` fires from `DFCanvas.OnInitSuccess` during coroutine Step 2, which sets `_dfCanvas.DebugPanel.SetModPlatform(platform)`. The timing gap between `OnInitSuccess` and `F10` press is large enough that the panel should have `_modPlatform` set. **This is not the primary cause.**

#### Cause B: `DebugPanel.ForceRefresh()` conditional in F10 handler

```csharp
// Plugin.cs:1147-1168
Bridge.KeyInputSystem.OnF10Pressed = () =>
{
    _dfCanvas.ToggleDebug();
    if (_dfCanvas.DebugPanel != null && _dfCanvas.DebugPanel.IsVisible)
    {
        _dfCanvas.DebugPanel.ForceRefresh();
    }
    ...
};
```

`ToggleDebug()` calls `DebugPanel.Show()` which sets `_targetVisible = true` THEN calls `ForceRefresh()` via `Show()` itself (see `DebugPanel.Show()` line 113: calls `ForceRefresh()`). So `ForceRefresh()` IS called on show. The conditional in the F10 handler provides a SECOND refresh for the case when the panel was already visible (toggling off then on within the same F10 press — but `ToggleDebug()` only calls `Show()` or `Hide()`).

**Primary Issue**: `DebugPanel.BuildWorldsContent` calls `Unity.Entities.World.All` and `em.GetAllEntities(Allocator.Temp)`. At main menu, the ECS world may not exist yet, causing `BuildSection` to show `"No worlds found"`. With `_showPlatform = true`, `_showWorlds = true`, `_showSystems = false`, `_showArchetypes = false`, `_showErrors = false` — the panel shows Platform Status + ECS Worlds. If Platform Status fails to find `_modPlatform`, the section shows "not available". If ECS Worlds shows "No worlds found", both visible sections appear nearly empty. This is a **data issue, not a code regression** — it depends on game state.

**File:Line**: `src/Runtime/UI/DebugPanel.cs:489-493` (NoPlatform text), `src/Runtime/UI/DebugPanel.cs:500-504` (No worlds found)

**Fix Spec**:
- Add `ForceRefresh()` call to `Show()` in `ModMenuPanel` (analogous to `DebugPanel.Show()` which already calls it).
- For DebugPanel empty appearance at main menu: acceptable behavior since no ECS world exists at main menu. Consider showing a different message like "ECS world available in gameplay" instead of just "No worlds found."

**Introducing Commits**: This behavior is pre-existing and not a regression introduced this session. The panel correctly shows the runtime state.

---

## Summary Table

| Symptom | Root Cause | Introducing Commit | File:Line | Fix Type |
|---------|------------|--------------------|-----------|----------|
| F9/F10 swapped | NOT a regression; correct mapping is F9=ModMenu, F10=Debug since `ff1455b2` | N/A — perception only | `Plugin.cs:1134-1168` | No code change needed |
| F9 mod menu empty packs (visual) | FilterContainer (140px) overlaps ScrollRect viewport — top 3-4 items occluded | `9d59d631` | `ModMenuPanel.cs:690` | Fix ScrollRect offsetMax to `-(32+140)` |
| F9 mod menu counter shows "0 of 9" | `SetPacks()` calls `RebuildPackList()` instead of `ApplyFilters()` — `_filteredIndices` never populated | `9d59d631` | `ModMenuPanel.cs:265` | Call `ApplyFilters()` from `SetPacks()` |
| F10 debug panel near-empty | No ECS world at main menu → WorldsContent shows "No worlds found"; if `_modPlatform` also not yet set, Platform section also minimal | Pre-existing behavior | `DebugPanel.cs:500-504` | UX improvement: show "available in gameplay" message |

---

## Does a default filter value hide all packs?

**Yes, conditionally.** In the CURRENT source (not deployed), commit `42a889ba` (i18n) changed the tier dropdown option at index 0 from `"All"` to `L10n.T("menu.filter.tier.all", "All")` which resolves to **`"All Tiers"`** from `en-US.json` line 4. The `ApplyFilters()` comparison is still `if (_tierFilter != "All")` (hardcoded English). If the i18n JSON files were deployed to `BepInEx/dinoforge-i18n/`, the filter condition `"All Tiers" != "All"` would be `true` and ALL packs would be excluded.

However, **the i18n files are NOT deployed** (directory `BepInEx/dinoforge-i18n/` does not exist in the game install), so `L10n.T(...)` returns the fallback value `"All"` for all keys, and the comparisons still work correctly.

**This is a latent time-bomb**: if the i18n files are ever deployed, `ApplyFilters()` will hide all packs because:
- `_tierFilter` = `"All Tiers"` (from `en-US.json`) ≠ `"All"` (hardcoded)  
- `_stateFilter` = `"All States"` (from `en-US.json`) ≠ `"All"` (hardcoded)

**Fix for latent bug** (commit `42a889ba` in-tree but not deployed):  
`src/Runtime/UI/ModMenuPanel.cs:1427` — change `if (_tierFilter != "All")` to use the option index instead of text comparison:

```csharp
// In ApplyFilters():
// BEFORE (fragile text comparison):
if (_tierFilter != "All") { ... }
if (_stateFilter != "All") { ... }

// AFTER (index-based, i18n-safe):
bool tierFilterActive = _tierDropdown != null && _tierDropdown.value != 0;
bool stateFilterActive = _stateDropdown != null && _stateDropdown.value != 0;
if (tierFilterActive) { ... }
if (stateFilterActive) { ... }
```

This makes filtering comparison independent of locale strings.

---

## Recommended Fix Priority

1. **P0 (immediate)**: Fix ScrollRect layout in `BuildListPane` — pack items are being occluded. File `ModMenuPanel.cs`, function `BuildListPane`, line ~712 (scrollRt.offsetMax assignment).
2. **P0 (immediate)**: Fix `SetPacks()` to call `ApplyFilters()` instead of `RebuildPackList()` so pack counter is always accurate and `_filteredIndices` is always populated.
3. **P1 (before i18n deploy)**: Fix `ApplyFilters()` to use dropdown index (not text) for filter comparisons — prevents all-packs-hidden regression when i18n files are deployed.
4. **P2 (UX)**: Improve DebugPanel empty-state messaging at main menu (no ECS world).
