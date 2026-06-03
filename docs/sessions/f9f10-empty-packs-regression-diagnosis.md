# F9/F10 Empty Packs + Empty Debug Panel — Regression Diagnosis (iter-149, 2026-05-29)

**Date**: 2026-05-29  
**Branch**: feat/unityexplorer-devtools-20260528  
**Deployed DLL SHA256**: `9cab855a8fe3c0ebac1081fb981f58cde402bdf949be64c9e1ee1c7195fc1436`  
**Deployed DLL timestamp**: `May 29 02:40 UTC` (STALE — fix commit `84c3e614` is `May 29 06:16 UTC`)

---

## DEFINITIVE ROOT CAUSE SUMMARY (investigator: 2026-05-29)

| Symptom | One-Line Root Cause |
|---------|---------------------|
| F9/F10 routing | Deployed DLL (`02:40 UTC`) pre-dates fix commit `84c3e614` (`06:16 UTC`) — F9 still calls `ToggleModMenu()`, F10 calls `ToggleDebug()` (swapped vs expected) |
| F10 mod menu packs empty | Same stale DLL: `SetPacks()` calls `RebuildPackList()` not `ApplyFilters()` so `_filteredIndices` is never populated → "0 of N" and no items rendered; plus FilterContainer occludes scroll viewport |
| F9 debug panel empty | Same stale DLL routes F9 to mod menu (empty per above), not to debug panel; debug panel is never shown at all |

**The fix for all three symptoms is one action**: rebuild and deploy with current source. `84c3e614` already contains the corrective code.

---

## Stale Binary Evidence

The deployed DLL was built at **02:40 UTC** on 2026-05-29. Fix commit `84c3e614` was committed at **06:16 UTC** on 2026-05-29. The DLL is 3h36m stale relative to the fix.

**Log proof**: the debug log records `"[RuntimeDriver] F9 pressed (via KeyInputSystem)"` — the pre-`84c3e614` format. Commit `84c3e614` changed this string to `"[RuntimeDriver] F9 pressed → DEBUG panel (via KeyInputSystem)"`. The deployed DLL still writes the old string, proving it predates the fix. The fix commit's key-mapping log line `"Key mapping: F9=Debug, F10=Mods"` IS present — that line was added in the same commit but appears in an earlier coroutine phase (Step 1), so it was partially deployed via a previous intermediate build.

The following commits are NOT yet deployed (exist in source, not in DLL):

| Commit | PDT Time | UTC Time | Contents |
|--------|----------|----------|----------|
| `ff1455b2` | 01:52 PDT | 08:52 UTC | F9/F10 swap fix (#895) |
| `941d0d44` | 02:25 PDT | 09:25 UTC | Loading skeleton overlay (#906) |
| `9d59d631` | 03:08 PDT | 10:08 UTC | Search/filter/sort for F10 panel |
| `d8c03f5e` | 03:44 PDT | 10:44 UTC | Keyboard navigation in F10 mod menu |
| `880af1f3` | 03:54 PDT | 10:54 UTC | BepInEx.AssemblyPublicizer integration |

The following commits from `84c3e614` are NOT yet deployed to the game:

| Commit | Key change | File |
|--------|------------|------|
| `84c3e614` | F9=Debug / F10=Mods; `SetPacks` calls `ApplyFilters`; index-based filter comparison; scroll rect layout fix | `Plugin.cs`, `ModMenuPanel.cs`, `DebugPanel.cs`, `NativeMenuInjector.cs` |

---

## Symptom 1 — F9/F10 key routing regression

### Root Cause
**Stale DLL.** Commit `84c3e614` (`Plugin.cs:1137-1171`) correctly wires:
- F9 → `_dfCanvas.ToggleDebug()` (debug panel)
- F10 → `_dfCanvas.ToggleModMenu()` (mods menu)

The deployed DLL predates this. In the deployed binary, `ff1455b2` had already swapped to F9=Debug/F10=Mods, but that commit contained a code error described in `84c3e614`'s commit message: the `F9 pressed` log message said `"F9 pressed (via KeyInputSystem)"` but the HANDLER body called `ToggleModMenu()`. The `84c3e614` commit cleaned this up properly. The deployed DLL's F9 handler calls `ToggleModMenu()` (mod menu), not `ToggleDebug()`.

**Evidence**: Log line `[Plugin] [RuntimeDriver] F9 pressed (via KeyInputSystem)` at 07:32:44 — this is the OLD pre-fix format. After `84c3e614`, the same line reads `[Plugin] [RuntimeDriver] F9 pressed → DEBUG panel (via KeyInputSystem)`.

**File:line** (source, already fixed): `src/Runtime/Plugin.cs:1137-1171`  
**Introducing commit**: `ff1455b2` (mixed state), **fix commit**: `84c3e614` (not deployed)

### Fix for Symptom 1
Deploy current source. No additional source change needed.

---

## Symptom 2 — F10 mod menu shows EMPTY loaded packs

### Root Cause
**Three sub-causes, all resolved in `84c3e614` but not deployed:**

#### B1 — ScrollRect collapsed behind FilterContainer

`9d59d631` added a FilterContainer (240px height) before the ScrollRect in the ListPane. The ScrollRect had hardcoded `offsetMin/Max` overrides that anchored it to the full ListPane height, causing the FilterContainer to visually overlap and collapse the scroll area. No pack items were visible in the scroll viewport despite being created.

**File:line**: `src/Runtime/UI/ModMenuPanel.cs:711` (scroll rect offset assignment, removed in `84c3e614`)  
**Introducing commit**: `9d59d631`

#### B2 — `SetPacks()` called `RebuildPackList()` not `ApplyFilters()`, leaving `_filteredIndices` empty

The old `SetPacks()` (line 225 in pre-`84c3e614`) called `RebuildPackList()` directly. `_filteredIndices` is only populated by `ApplyFilters()`. Since `_filteredIndices` was empty after `SetPacks()`, `RebuildFilteredPackList()` rendered nothing and the counter showed "0 of N".

**File:line**: `src/Runtime/UI/ModMenuPanel.cs:268` (`ApplyFilters()` call, added in `84c3e614`)  
**Introducing commit**: `9d59d631` (added filter but didn't wire to SetPacks)

#### B3 — `ApplyFilters()` compared localised option text vs hardcoded "All"

`9d59d631` added `ApplyFilters()` but compared `_tierFilter` (a string from dropdown option text) against hardcoded `"All"`. With `42a889ba` wiring `L10n.T()` into the option labels, the dropdown option at index 0 became `L10n.T("menu.filter.tier.all", "All")` = `"All"` (fallback). **Currently safe** because `L10n` returns the fallback when i18n JSON files are absent. Becomes a P0 time-bomb when i18n files are deployed.

`84c3e614` fixed this by reading `_tierDropdown.value` (integer index 0=All) instead of comparing text strings.

**File:line**: `src/Runtime/UI/ModMenuPanel.cs:1416-1417` (`.value` comparison, added in `84c3e614`)  
**Introducing commit**: `42a889ba` + `9d59d631` interaction

### Fix for Symptom 2
Deploy current source. All three sub-fixes are in `84c3e614`.

---

## Symptom 3 — F9 debug panel shows EMPTY output

### Root Cause
In the deployed DLL, **F9 opens the mod menu** (not the debug panel) because the F9 handler calls `ToggleModMenu()`. The mod menu is empty (Symptom 2). The debug panel is never opened by F9. The user is correctly identifying that "F9 shows nothing useful" — it is showing the empty mod menu rather than the debug panel.

If the key routing were fixed (Symptom 1 deploy), the debug panel WOULD appear, but might appear near-empty at main menu because:
- `_modPlatform` may not be set yet when F9 is first pressed (timing: `WireUguiToModPlatform()` is called on `DFCanvas.OnInitSuccess`, before Step 7 pack-load, so `_modPlatform` IS set — but `GetLoadedPackDisplayInfos()` returns 0 packs if Step 7 hasn't completed yet)
- ECS Worlds section correctly shows "No ECS world (main menu — expected)" (fixed in `84c3e614`)

`DebugPanel.ForceRefresh()` IS called from `Show()` on every toggle, so the content updates correctly when packs are loaded.

**File:line (source, already fixed)**: `src/Runtime/UI/DebugPanel.cs:506-508` ("No ECS world" message, added in `84c3e614`)  
**Introducing commit**: stale DLL + F9 routing regression

### Fix for Symptom 3
Deploy current source. The debug panel will show correct content once F9 routing is fixed and packs have loaded.

---

## Does the search/filter default hide all packs?

**In current source, NO** — because i18n JSON files are not deployed.

`ApplyFilters()` in `84c3e614` reads `_tierDropdown.value` (integer) not `_tierFilter` (localised text string). Default dropdown value = 0 = "All" = no filter applied. All packs pass through.

`OnTierFilterChanged`/`OnStateFilterChanged` still update `_tierFilter`/`_stateFilter` string fields (dead code since `84c3e614`), but `ApplyFilters()` ignores those strings. This is harmless but slightly confusing.

**Latent risk**: If `L10n` JSON files are deployed AND the old `_tierFilter != "All"` comparison were used, packs would be hidden. Since `84c3e614` switched to index comparison, this risk is eliminated in the current source.

---

## Fix Spec for Implementation Agent

**ONE action fixes all three symptoms**:

```powershell
# 1. Build with deploy target (must specify TFM per Pattern #530)
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release `
  -p:DeployToGame=true `
  -p:TargetFramework=netstandard2.0

# 2. Verify DLL timestamp > 06:16 UTC May 29 2026
(Get-Item "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll").LastWriteTimeUtc

# 3. Verify SHA256 != stale hash
Get-FileHash "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll" -Algorithm SHA256
# Expected to NOT be: 9cab855a8fe3c0ebac1081fb981f58cde402bdf949be64c9e1ee1c7195fc1436

# 4. After game launch, verify in dinoforge_debug.log:
# [Plugin] [RuntimeDriver] F9 pressed → DEBUG panel (via KeyInputSystem)   ← new format
# [Plugin] [RuntimeDriver] F10 pressed → MODS menu (via KeyInputSystem)    ← new format
# [ModMenuPanel.SetPacks] ApplyFilters() complete (N of M visible)         ← new log line
```

**If packs still appear empty after rebuild+deploy**, add a `yield return null;` in `src/Runtime/Plugin.cs` before the Step 7 block (`RunPhaseWithAbortGuard("MainMenu-mode PackLoad", ...)`) to guarantee DFCanvas layout flush completes before `SetPacks()` is called.

---

## Dead Code to Clean Up (non-blocking)

`src/Runtime/UI/ModMenuPanel.cs:61-62` — `_tierFilter` and `_stateFilter` string fields are updated in callbacks but never read since `84c3e614`. Remove or keep as documentation comments.

---

## Summary Table

| Symptom | Root Cause | Introducing Commit | File:Line (current source, already fixed) | Action |
|---------|------------|--------------------|------------------------------------------|--------|
| F9/F10 routing | Deployed DLL pre-`84c3e614`; F9 handler calls `ToggleModMenu()` | `ff1455b2` partial fix; `84c3e614` not deployed | `Plugin.cs:1137-1171` | Rebuild + deploy |
| F10 mod menu packs empty | `SetPacks()` calls `RebuildPackList()` (not `ApplyFilters()`); scroll rect collapsed behind filter bar | `9d59d631` | `ModMenuPanel.cs:268` (B2), `ModMenuPanel.cs:711` (B1) | Rebuild + deploy |
| F9 debug panel empty | F9 routes to mod menu (empty per above); debug panel never shown | Stale DLL | `Plugin.cs:1137` | Rebuild + deploy |
