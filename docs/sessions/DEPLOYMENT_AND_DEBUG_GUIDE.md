# F10 Panel Rendering - Deployment & Debug Guide

## Quick Summary

A complete trace has been added to the ModMenuPanel rendering pipeline to diagnose why pack list items remain invisible despite the UiBuilder.MakeText() fix being present and compiled.

**Status**: Code compiled and ready, DLL locked by running game instance.

---

## Step 1: Deploy the Updated DLL

### Option A: Using Script (Recommended)
```bash
cd C:\Users\koosh\Dino
chmod +x deploy-debug.sh
./deploy-debug.sh
```

The script will:
1. Show source DLL timestamp (should be Mar 12 04:06)
2. Show current deployed DLL timestamp
3. Prompt you to close the game
4. Copy the new DLL to the game's BepInEx plugins folder
5. Show the new deployed DLL timestamp

### Option B: Manual Deployment
```bash
# Close the game first!
cp C:\Users\koosh\Dino\src\Runtime\bin\Debug\netstandard2.0\DINOForge.Runtime.dll \
   "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\"

# Verify
ls -lh "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll"
```

**CRITICAL**: The DLL must be closed first (game not running). The Mono runtime locks it.

---

## Step 2: Launch Game and Open F10 Panel

1. Start Diplomacy is Not an Option (Steam)
2. Wait for main menu
3. Press **F10** to open the mod menu panel
4. You should see the panel appear with header and footer

---

## Step 3: Examine BepInEx Logs

### Log File Location
```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log
```

### Search for Trace Markers

```bash
# On Windows (PowerShell)
Select-String -Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log" `
  -Pattern "ModMenuPanel.SetPacks|ModMenuPanel.RebuildPackList|ModMenuPanel.BuildPackListItem"

# On Linux/macOS
grep "ModMenuPanel.SetPacks\|RebuildPackList\|BuildPackListItem" \
  "G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/LogOutput.log"
```

### Expected Output Samples

**Good (Items rendering successfully):**
```
╔════════════════════════════════════════════════════════════════════════════════════╗
║ [ModMenuPanel.SetPacks] ENTRY                                                       ║
╚════════════════════════════════════════════════════════════════════════════════════╝
  Before: 0 packs, After: 6 packs
  _listContent: READY (name=Content, active=True)
  _selectedPackIndex: 0
  Pack list:
    • economy-balanced (ID: economy-balanced, enabled: True)
    • warfare-modern (ID: warfare-modern, enabled: True)
[ModMenuPanel.SetPacks] Calling RebuildPackList()...
[ModMenuPanel.RebuildPackList] START: _packs.Count=6, _listContent=Content, active=True
[ModMenuPanel.RebuildPackList] _listContent RectTransform: position=(0, 0), sizeDelta=(210, 0)
[ModMenuPanel.RebuildPackList] Clearing 0 existing items
[ModMenuPanel.RebuildPackList] After clear: childCount=0. Now rendering 6 pack(s)...
[ModMenuPanel.RebuildPackList] Creating item 0: 'economy-balanced' (ID: economy-balanced)
[ModMenuPanel.BuildPackListItem] Starting item 0: 'economy-balanced' (enabled=True, selected=True)
[ModMenuPanel.BuildPackListItem] Item 0 card created: sizeDelta=(0, 40), active=True
[ModMenuPanel.BuildPackListItem] Item 0 LayoutElement set: minHeight=40, preferredHeight=40
[ModMenuPanel.BuildPackListItem] Item 0 nameText created: text='economy-balanced', fontSize=13, color=RGBA(0.91, 0.84, 0.69, 1.00), sizeDelta=(200, 17), font=Arial
[ModMenuPanel.BuildPackListItem] Item 0 nameText LayoutElement: minWidth=100, flexibleWidth=1
...
[ModMenuPanel.RebuildPackList] COMPLETE: childCount=6. Listing items:
  Item 0: name=PackItem_economy-balanced, active=True, sizeDelta=(0, 40), childCount=2
  Item 1: name=PackItem_warfare-modern, active=True, sizeDelta=(0, 40), childCount=2
[ModMenuPanel.SetPacks] RefreshDetail() complete. EXIT.
```

---

## Step 4: Analyze the Results

### Case A: SetPacks() NOT in Log

**Diagnosis**: SetPacks() is never being called at all.

**Root Cause**: Issue is not in the rendering pipeline, but in the pack loading system or ModMenuOverlayProxy.

**Action Items**:
1. Check `ModMenuOverlayProxy.cs` - does it call `panel.SetPacks()`?
2. Check when packs are loaded - is it BEFORE DFCanvas is initialized?
3. Search log for "ModMenuOverlayProxy" or "PackManager" to see if loading works
4. Verify RuntimeDriver properly connects ModMenuPanel to ModMenuOverlayProxy

---

### Case B: _listContent is NULL

**Diagnosis**:
```
[ModMenuPanel.SetPacks] _listContent is NULL
[ModMenuPanel.RebuildPackList] _listContent is NULL — UI not initialized yet.
```

**Root Cause**: SetPacks() is being called before DFCanvas.Build() completes.

**Action Items**:
1. Check initialization order in RuntimeDriver
2. DFCanvas.Start() should complete before any SetPacks() calls
3. Add timing checks to see which happens first
4. Consider deferring SetPacks() call until DFCanvas.IsReady == true

---

### Case C: Items Created But With Zero Height

**Diagnosis**:
```
[ModMenuPanel.RebuildPackList] COMPLETE: childCount=6. Listing items:
  Item 0: name=PackItem_economy-balanced, active=True, sizeDelta=(0, 40), childCount=2
  Item 0 nameText created: ... sizeDelta=(200, 17), ...
```

Items ARE created but rendered in the scroll view, yet they appear invisible.

**Possible Issues**:

1. **ContentSizeFitter not expanding container**
   - Check: _listContent.sizeDelta is still (210, 0) - zero height!
   - VerticalLayoutGroup is calculating preferred heights, but ContentSizeFitter isn't applying
   - Solution: Force LayoutRebuilder.ForceRebuildLayoutHierarchy() after item creation

2. **VerticalLayoutGroup forcing children to zero height**
   - Check: vlg.childForceExpandHeight is False (correct)
   - Check: vlg.childForceExpandWidth is True (correct)
   - Issue: Maybe LayoutGroup isn't recalculating after items added

3. **Parent container not active**
   - Check if scrollRect.gameObject.activeSelf is false
   - Check if any parent up the hierarchy is inactive

4. **Clipping/mask not rendering**
   - Scroll view has a Mask component
   - Check if mask is preventing rendering

**Debug Additions Needed**:
```csharp
// After rendering items, force layout recalculation
LayoutRebuilder.ForceRebuildLayoutHierarchy(_listContent);
_log?.LogInfo($"[ModMenuPanel.RebuildPackList] After LayoutRebuilder: _listContent.sizeDelta={_listContent.sizeDelta}");
```

---

### Case D: Items Visible But Text Not Showing

**Diagnosis**:
```
[ModMenuPanel.BuildPackListItem] Item 0 nameText created: text='economy-balanced', fontSize=13, color=RGBA(0.91, 0.84, 0.69, 1.00), sizeDelta=(200, 17), font=Arial
```

Items are visible (background color shows) but pack names are not.

**Possible Issues**:

1. **Text color has zero alpha**
   - Check: color=RGBA(..., 0.00) instead of 1.00
   - Solution: Verify UiBuilder.TextPrimary has alpha=1.0

2. **Font is null**
   - Check: font=Arial in log
   - If font=null, text won't render
   - Solution: Verify Resources.GetBuiltinResource<Font>("Arial.ttf") works

3. **Text component disabled**
   - Check: gameObject.activeSelf
   - Add to logging: `_log?.LogInfo($"  nameText active: {nameText.gameObject.activeSelf}");`

4. **Text overflow settings**
   - horizontalOverflow: Wrap
   - verticalOverflow: Overflow
   - These might be clipping the text

---

### Case E: No Error But Items Still Invisible

If logs show everything created correctly but still no visual:

1. **Check RectTransform anchoring**
   - Items use HorizontalLayoutGroup childForceExpandWidth=false
   - Verify items are anchored correctly

2. **Check Canvas rendering**
   - Check DFCanvas._canvas.enabled
   - Check Canvas.renderMode = ScreenSpaceOverlay
   - Check Canvas.sortingOrder = 100

3. **Check CanvasScaler**
   - referenceResolution = 1920x1080
   - May be scaling all elements down to 0

---

## Step 5: Rebuild with Additional Logging

If Case A-E don't reveal the problem, add more logging:

### In RebuildPackList(), after creating each item:
```csharp
LayoutRebuilder.ForceRebuildLayoutHierarchy(_listContent);
_log?.LogInfo($"[RebuildPackList] After item {i}: _listContent.sizeDelta={_listContent.sizeDelta}");
```

### In BuildPackListItem(), after Text creation:
```csharp
_log?.LogInfo($"  nameText active: {nameText.gameObject.activeSelf}");
_log?.LogInfo($"  nameText font: {nameText.font?.name ?? "NULL"}");
_log?.LogInfo($"  nameText color alpha: {nameText.color.a}");
_log?.LogInfo($"  nameText raycast: {nameText.raycastTarget}");
```

### In BuildListPane(), after ScrollRect creation:
```csharp
_log?.LogInfo($"  scrollRect.gameObject.active: {scrollRect.gameObject.activeSelf}");
_log?.LogInfo($"  scrollRect parent active: {scrollRect.gameObject.transform.parent?.gameObject.activeSelf}");
```

---

## Files in This Debug Package

| File | Purpose |
|------|---------|
| `RENDERING_TRACE_IMPLEMENTATION.md` | Complete implementation details of all logging added |
| `TRACE_SUMMARY.txt` | Quick reference summary |
| `DEPLOYMENT_AND_DEBUG_GUIDE.md` | This file - deployment and interpretation guide |
| `deploy-debug.sh` | Bash script to deploy the DLL with user prompts |
| `src/Runtime/UI/ModMenuPanel.cs` | Modified with comprehensive logging |

---

## Key Code Locations

### Source Code
- **ModMenuPanel.cs**: L106-144 (SetPacks)
- **ModMenuPanel.cs**: L312-330 (BuildListPane)
- **ModMenuPanel.cs**: L451-508 (RebuildPackList)
- **ModMenuPanel.cs**: L510-567 (BuildPackListItem)
- **UiBuilder.cs**: L125 (sizeDelta fix)

### Compiled DLL
```
C:\Users\koosh\Dino\src\Runtime\bin\Debug\netstandard2.0\DINOForge.Runtime.dll
(217 KB, Mar 12 04:06)
```

### Deployment Target
```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\plugins\DINOForge.Runtime.dll
```

### Game Logs
```
G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log
```

---

## Troubleshooting Checklist

- [ ] DLL deployed successfully (new timestamp shows Mar 12 04:06 or later)
- [ ] Game started without crashing
- [ ] F10 panel appears (with header "DINOForge" and "Close" button)
- [ ] BepInEx LogOutput.log contains "ModMenuPanel.SetPacks" entries
- [ ] Pack list items were created (childCount=6 or higher in logs)
- [ ] Item Text components created with sizeDelta=(200, 17)
- [ ] All items have active=True in logs
- [ ] VerticalLayoutGroup and ContentSizeFitter present on content container

If any of these fail, refer to the corresponding case (A-E) above.

---

## Next Steps After Reading Logs

1. **If Case A** (SetPacks not called):
   - Investigate ModMenuOverlayProxy connection
   - Check RuntimeDriver initialization order

2. **If Case B** (_listContent NULL):
   - Add timing checks to DFCanvas.Start() completion
   - Defer SetPacks() call until UI ready

3. **If Case C** (zero height items):
   - Add LayoutRebuilder.ForceRebuildLayoutHierarchy() calls
   - Check ContentSizeFitter vertical fit setting

4. **If Case D** (text not visible):
   - Verify UiBuilder color constants have alpha=1.0
   - Check font loading (Resources.GetBuiltinResource)

5. **If Case E** (everything seems right but still invisible):
   - Check Canvas/CanvasScaler settings
   - Verify RectTransform anchoring on items
   - Check parent hierarchy activation

