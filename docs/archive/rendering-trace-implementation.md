# F10 Panel Rendering Pipeline - Complete Trace Implementation

## Status
- **Date**: 2026-03-12
- **Issue**: Pack list sidebar remains completely invisible despite UIBuilder.MakeText() fix
- **Approach**: Complete instrumentation of rendering pipeline with BepInEx-compatible logging

## Changes Made

### 1. UiBuilder.cs (VERIFIED FIX PRESENT)
**Location**: `C:\Users\koosh\Dino\src\Runtime\UI\UiBuilder.cs:125`

```csharp
RectTransform rt = go.GetComponent<RectTransform>();
rt.sizeDelta = new Vector2(200f, fontSize + 4f);  // Ensure text has visible dimensions
```

✓ Fix is present in source code (verified Mar 12 03:40)
✓ Fix was compiled into DLL (Mar 12 04:06)

---

### 2. ModMenuPanel.cs - Comprehensive Logging Added

#### SetPacks() Entry Logging
**Location**: Lines 106-141

```csharp
public void SetPacks(IEnumerable<PackDisplayInfo> packs)
{
    // ... state updates ...

    _log?.LogInfo($"╔════════════════════════════════════════════════════════════════════════════════════╗");
    _log?.LogInfo($"║ [ModMenuPanel.SetPacks] ENTRY                                                       ║");
    _log?.LogInfo($"╚════════════════════════════════════════════════════════════════════════════════════╝");
    _log?.LogInfo($"  Before: {beforeCount} packs, After: {_packs.Count} packs");
    _log?.LogInfo($"  _listContent: {(_listContent != null ? ... : "NULL")}");
    _log?.LogInfo($"  _selectedPackIndex: {_selectedPackIndex}");

    if (_packs.Count > 0)
    {
        _log?.LogInfo($"  Pack list:");
        foreach (var p in _packs)
        {
            _log?.LogInfo($"    • {p.Name} (ID: {p.Id}, enabled: {p.IsEnabled})");
        }
    }

    // Calls RebuildPackList() with logging
    RebuildPackList();
    RefreshDetail();
    _log?.LogInfo($"[ModMenuPanel.SetPacks] EXIT.");
}
```

**Expected Log Output**:
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
    ...
```

---

#### BuildListPane() Scroll View Logging
**Location**: Lines 312-330

After MakeScrollView() creation, logs:
- Content container state: name, active, anchor/pivot
- ScrollRect component configuration
- ContentSizeFitter and VerticalLayoutGroup presence

```csharp
_listContent = content;
_log?.LogInfo($"[ModMenuPanel.BuildListPane] Scroll view initialized successfully.");
_log?.LogInfo($"  content.name={content.name}");
_log?.LogInfo($"  content.active={content.gameObject.activeSelf}");
_log?.LogInfo($"  content.anchorMin={content.anchorMin}, anchorMax={content.anchorMax}");
_log?.LogInfo($"  content.sizeDelta={content.sizeDelta}");
_log?.LogInfo($"  content.anchoredPosition={content.anchoredPosition}");
_log?.LogInfo($"  ScrollRect.vertical={scrollRect.vertical}");

ContentSizeFitter csf = content.GetComponent<ContentSizeFitter>();
VerticalLayoutGroup vlg = content.GetComponent<VerticalLayoutGroup>();
_log?.LogInfo($"  content has ContentSizeFitter: {csf != null}");
_log?.LogInfo($"  content has VerticalLayoutGroup: {vlg != null}");
```

**Expected Output**:
```
[ModMenuPanel.BuildListPane] Scroll view initialized successfully.
  content.name=Content
  content.active=True
  content.anchorMin=(0, 1), anchorMax=(1, 1)
  content.sizeDelta=(210, 0)
  content.anchoredPosition=(0, 0)
  ScrollRect.vertical=True
  content has ContentSizeFitter: True (verticalFit=PreferredSize)
  content has VerticalLayoutGroup: True (childForceExpandHeight=False, spacing=2)
```

---

#### RebuildPackList() Execution Logging
**Location**: Lines 442-480

Logs for every call:
1. _listContent state (BEFORE rendering)
2. RectTransform details (position, size, anchor)
3. Clearing operation (how many items being destroyed)
4. Item creation progress
5. AFTER rendering: childCount and each item's details

```csharp
private void RebuildPackList()
{
    if (_listContent == null)
    {
        _log?.LogWarning("[ModMenuPanel.RebuildPackList] _listContent is NULL ...");
        return;
    }

    _log?.LogInfo($"[ModMenuPanel.RebuildPackList] START: _packs.Count={_packs.Count}, _listContent={_listContent.name}, active={_listContent.gameObject.activeSelf}");
    _log?.LogInfo($"[ModMenuPanel.RebuildPackList] _listContent RectTransform: position={_listContent.anchoredPosition}, sizeDelta={_listContent.sizeDelta}");
    _log?.LogInfo($"[ModMenuPanel.RebuildPackList] Clearing {_listContent.childCount} existing items");

    // Destroy items...

    _log?.LogInfo($"[ModMenuPanel.RebuildPackList] After clear: childCount={_listContent.childCount}. Now rendering {_packs.Count} pack(s)...");

    for (int i = 0; i < _packs.Count; i++)
    {
        _log?.LogInfo($"[ModMenuPanel.RebuildPackList] Creating item {i}: '{_packs[i].Name}' (ID: {_packs[i].Id})");
        BuildPackListItem(_packs[i], i);
    }

    _log?.LogInfo($"[ModMenuPanel.RebuildPackList] COMPLETE: childCount={_listContent.childCount}. Listing items:");
    for (int i = 0; i < _listContent.childCount; i++)
    {
        Transform child = _listContent.GetChild(i);
        RectTransform childRt = child.GetComponent<RectTransform>();
        _log?.LogInfo($"  Item {i}: name={child.name}, active={child.gameObject.activeSelf}, sizeDelta={childRt.sizeDelta}, childCount={child.childCount}");
    }
}
```

**Expected Output**:
```
[ModMenuPanel.RebuildPackList] START: _packs.Count=6, _listContent=Content, active=True
[ModMenuPanel.RebuildPackList] _listContent RectTransform: position=(0, 0), sizeDelta=(210, 0)
[ModMenuPanel.RebuildPackList] Clearing 0 existing items
[ModMenuPanel.RebuildPackList] After clear: childCount=0. Now rendering 6 pack(s)...
[ModMenuPanel.RebuildPackList] Creating item 0: 'economy-balanced' (ID: economy-balanced)
[ModMenuPanel.RebuildPackList] Creating item 1: 'warfare-modern' (ID: warfare-modern)
...
[ModMenuPanel.RebuildPackList] COMPLETE: childCount=6. Listing items:
  Item 0: name=PackItem_economy-balanced, active=True, sizeDelta=(0, 40), childCount=2
  Item 1: name=PackItem_warfare-modern, active=True, sizeDelta=(0, 40), childCount=2
```

---

#### BuildPackListItem() Creation Logging
**Location**: Lines 483-535

For EVERY item created, logs:
1. Item start message (name, index, enabled/selected state)
2. Card panel creation (GameObject created, RectTransform.sizeDelta)
3. LayoutElement configuration
4. Pack name Text component creation (text content, fontSize, color, RectTransform.sizeDelta, font)
5. Text LayoutElement configuration

```csharp
private void BuildPackListItem(PackDisplayInfo pack, int index)
{
    if (_listContent == null) { /* ... */ return; }

    _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Starting item {index}: '{pack.Name}' (enabled={pack.IsEnabled}, selected={index == _selectedPackIndex})");

    // ... color setup ...

    GameObject card = UiBuilder.MakePanel(_listContent, $"PackItem_{pack.Id}", bgColor, new Vector2(0f, ItemHeight));
    RectTransform cardRt = card.GetComponent<RectTransform>();
    _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} card created: sizeDelta={cardRt.sizeDelta}, active={card.activeSelf}");

    LayoutElement cardLe = card.AddComponent<LayoutElement>();
    cardLe.minHeight = ItemHeight;
    cardLe.preferredHeight = ItemHeight;
    cardLe.flexibleWidth = 1f;
    _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} LayoutElement set: minHeight={cardLe.minHeight}, preferredHeight={cardLe.preferredHeight}");

    // ... border, layout group ...

    Color nameColor = pack.IsEnabled ? UiBuilder.TextPrimary : UiBuilder.TextSecondary;
    Text nameText = UiBuilder.MakeText(card.transform, "PackName", pack.Name, 13,
        nameColor, bold: isSelected);
    RectTransform nameTextRt = nameText.GetComponent<RectTransform>();
    _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} nameText created: text='{pack.Name}', fontSize={nameText.fontSize}, color={nameColor}, sizeDelta={nameTextRt.sizeDelta}, font={nameText.font?.name}");

    // ... text styling ...

    LayoutElement nameLe = nameText.gameObject.AddComponent<LayoutElement>();
    nameLe.minWidth = 100f;
    nameLe.flexibleWidth = 1f;
    _log?.LogInfo($"[ModMenuPanel.BuildPackListItem] Item {index} nameText LayoutElement: minWidth={nameLe.minWidth}, flexibleWidth={nameLe.flexibleWidth}");

    // ... badges, version, button ...
}
```

**Expected Output** (for each item):
```
[ModMenuPanel.BuildPackListItem] Starting item 0: 'economy-balanced' (enabled=True, selected=True)
[ModMenuPanel.BuildPackListItem] Item 0 card created: sizeDelta=(0, 40), active=True
[ModMenuPanel.BuildPackListItem] Item 0 LayoutElement set: minHeight=40, preferredHeight=40
[ModMenuPanel.BuildPackListItem] Item 0 nameText created: text='economy-balanced', fontSize=13, color=RGBA(0.91, 0.84, 0.69, 1.00), sizeDelta=(200, 17), font=Arial
[ModMenuPanel.BuildPackListItem] Item 0 nameText LayoutElement: minWidth=100, flexibleWidth=1
```

---

## How to Deploy and Test

### Step 1: Close the Game
The game must be closed before deploying the new DLL (Unity Mono runtime locks the DLL).

### Step 2: Run Deploy Script
```bash
cd C:\Users\koosh\Dino
./deploy-debug.sh
```

Or manually:
```bash
cp src/Runtime/bin/Debug/netstandard2.0/DINOForge.Runtime.dll \
   "G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/plugins/"
```

### Step 3: Start Game and Open F10 Panel
- Start the game
- Press F10 to open the mod menu panel
- Look for the pack list on the left side

### Step 4: Read BepInEx Log
- Check: `G:/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/LogOutput.log`
- Search for: `[ModMenuPanel.SetPacks]` to see the entire trace

---

## Expected Behavior

### Scenario A: SetPacks() is Called (SHOULD SEE THIS)
```
╔════════════════════════════════════════════════════════════════════════════════════╗
║ [ModMenuPanel.SetPacks] ENTRY                                                       ║
╚════════════════════════════════════════════════════════════════════════════════════╝
  Before: 0 packs, After: 6 packs
  _listContent: READY (name=Content, active=True)
  ...
[ModMenuPanel.RebuildPackList] START: _packs.Count=6, ...
[ModMenuPanel.BuildPackListItem] Starting item 0: 'economy-balanced' ...
[ModMenuPanel.BuildPackListItem] Item 0 nameText created: text='economy-balanced', fontSize=13, color=..., sizeDelta=(200, 17), font=Arial
...
[ModMenuPanel.RebuildPackList] COMPLETE: childCount=6. Listing items:
  Item 0: name=PackItem_economy-balanced, active=True, sizeDelta=(0, 40), childCount=2
```

### Scenario B: SetPacks() NOT Called (CRITICAL BUG)
- Search for `SetPacks` in the log
- If NO log entries appear, the problem is NOT in the rendering pipeline
- Instead, the issue is in ModMenuOverlayProxy or the pack loading system
- Check: Does ModMenuOverlayProxy.SetPacks() actually get called?

### Scenario C: _listContent is NULL (DIFFERENT BUG)
```
[ModMenuPanel.SetPacks] _listContent: NULL
[ModMenuPanel.RebuildPackList] _listContent is NULL — UI not initialized yet.
```

- This means DFCanvas.Start() didn't complete before SetPacks() was called
- Check timing: When is SetPacks() called vs when is DFCanvas initialized?

### Scenario D: Items Created But Not Visible (ROOT CAUSE OF CURRENT BUG)
```
[ModMenuPanel.RebuildPackList] COMPLETE: childCount=6. Listing items:
  Item 0: name=PackItem_economy-balanced, active=True, sizeDelta=(0, 40), childCount=2
  Item 0 nameText created: ... sizeDelta=(200, 17), font=Arial
```

Items ARE created with non-zero sizes, but still invisible. Then we look for:
1. **Clipping/Layout Issues**:
   - Is _listContent.sizeDelta still (220, 0)? (zero height!)
   - VerticalLayoutGroup not recalculating layout
   - ContentSizeFitter not expanding

2. **Hierarchy Issues**:
   - Parent(s) not active
   - Hierarchy broken (items not actually as children of _listContent)

3. **Canvas/Rendering Issues**:
   - Canvas not rendering overlay
   - Sorting order wrong
   - Text color has 0 alpha

---

## Root Cause Diagnosis

Based on the logs, determine the exact failure mode:

| Log Evidence | Problem | Next Step |
|---|---|---|
| `SetPacks` not in log | SetPacks never called | Check ModMenuOverlayProxy |
| `_listContent: NULL` | UI not initialized before SetPacks | Check timing in RuntimeDriver |
| Items created, sizeDelta=(0, 40) but not visible | Layout group forcing zero height | Check VerticalLayoutGroup settings |
| Items visible on scroll but not in list | Parent not active | Check _listContent.gameObject.activeSelf |
| Text created with correct sizeDelta but not visible | Text color/font issue | Check fontSize, color alpha, font |

---

## Files Modified

1. **src/Runtime/UI/ModMenuPanel.cs**
   - SetPacks(): Added comprehensive entry/exit logging with all state
   - BuildListPane(): Added scroll view component verification
   - RebuildPackList(): Added complete lifecycle logging with item enumeration
   - BuildPackListItem(): Added per-item creation logging

2. **src/Runtime/UI/UiBuilder.cs** (ALREADY FIXED)
   - MakeText(): Already has `rt.sizeDelta = new Vector2(200f, fontSize + 4f);`

---

## Compilation Status

✓ Source code modified and saved
✓ Project built successfully (Mar 12 04:06)
✓ DLL compiled with logging
✓ Ready for deployment once game is closed

**Next**: Deploy the DLL and check the logs in BepInEx/LogOutput.log

