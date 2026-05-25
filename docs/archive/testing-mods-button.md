# Testing the Mods Button - Diagnostic Checklist

## Pre-Test Setup

1. **Delete old logs** to avoid confusion:
   ```bash
   rm BepInEx/LogOutput.log
   ```

2. **Rebuild the plugin**:
   ```bash
   dotnet build src/DINOForge.sln -c Debug
   ```

3. **Copy DLL to game**:
   ```bash
   cp src/Runtime/bin/Debug/netstandard2.0/DINOForge.Runtime.dll BepInEx/plugins/
   ```

4. **Restart the game** to get a new session ID.

## Testing Flow

### Step 1: Watch for Plugin Startup

1. Game starts
2. Open `BepInEx/LogOutput.log`
3. **Look for**:
   ```
   [NativeMenuInjector::XXXXXXXX] ===== PLUGIN SESSION START ===== Awake at HH:MM:SS.fff UTC
   ```
   - Write down the session ID (8 chars after ::)
   - All subsequent logs MUST have this same ID
   - If logs have a different or missing ID, they're from an old session

### Step 2: Watch for Injection Attempts

1. **Look for**:
   ```
   [NativeMenuInjector::XXXXXXXX] ═══ INJECTION ATTEMPT #1 at HH:MM:SS.fff UTC ═══
   ```

2. **Scan the output for each step**:
   - STEP 1 (Clone): Should say "OK" or "FAILED"
   - STEP 2 (Position): Should say "OK" or "WARN"
   - STEP 3 (Activate): Should say "OK"
   - STEP 4 (CanvasGroup): Should say "OK" or "INFO"
   - STEP 5 (Raycast): Should say "OK"
   - STEP 6 (Wire onClick): Should say "OK"
   - STEP 7 (EventSystem): Should say "OK"
   - STEP 8 (Verification): Should show all flags TRUE

3. **Success indicator**:
   ```
   [NativeMenuInjector::XXXXXXXX] Attempt#1 ✓✓✓✓✓✓ MODS BUTTON INJECTION FULLY SUCCESSFUL ✓✓✓✓✓✓
   ```

### Step 3: Verify Button State

In the logs, find the final button state verification (STEP 8):

```
[NativeMenuInjector::XXXXXXXX] Attempt#1     - gameObject.activeSelf: True
[NativeMenuInjector::XXXXXXXX] Attempt#1     - gameObject.activeInHierarchy: True
[NativeMenuInjector::XXXXXXXX] Attempt#1     - interactable: True
[NativeMenuInjector::XXXXXXXX] Attempt#1     - navigation.mode: None
[NativeMenuInjector::XXXXXXXX] Attempt#1     - targetGraphic.raycastTarget: True
[NativeMenuInjector::XXXXXXXX] Attempt#1     - sibling_index: N
[NativeMenuInjector::XXXXXXXX] Attempt#1     - overlay_ref: READY
```

**All must be True/READY**. If any is False/NULL, button won't be clickable.

### Step 4: Click the Mods Button

1. In the game, navigate to the main menu or pause menu
2. Look for the Mods button (should be next to Settings/Options)
3. **Try to click it**
4. **Watch for hover highlight** (red overlay per original Button settings)

### Step 5: Check If Click Was Registered

1. **Open logs again** and search for your session ID:
   ```
   [NativeMenuInjector::XXXXXXXX] ═══ MODS BUTTON CLICKED #1 at HH:MM:SS.fff UTC ═══
   ```

2. **Three possible outcomes**:

   **OUTCOME A: Click log appears**
   ```
   [NativeMenuInjector::XXXXXXXX] Click#1   overlay.IsVisible BEFORE toggle: False
   [NativeMenuInjector::XXXXXXXX] Click#1   overlay.IsVisible AFTER toggle: True
   [NativeMenuInjector::XXXXXXXX] Click#1 ✓ Mods menu TOGGLED successfully
   ```
   → **SUCCESS**: Button worked! Menu should appear.

   **OUTCOME B: Click log doesn't appear, but another button was clicked**
   ```
   [UiEventInterceptor::XXXXXXXX] ⚡ BUTTON CLICK #1 at HH:MM:SS.fff UTC
   [UiEventInterceptor::XXXXXXXX]   Button name: 'OptionsButton'
   ```
   → **CLICK ROUTING ISSUE**: Your click hit the Options button, not Mods. This means:
      - Mods button isn't in the scene where you think it is
      - Z-order issue (Options is on top)
      - Mods button was destroyed

   **OUTCOME C: No click logs at all**
   → **NOT CLICKING AT ALL**: Either you didn't click the button, or the button isn't rendering. Check visually for the button.

### Step 6: Check Global Click Interceptor

The `UiEventInterceptor` logs every button click in the scene:

```
[UiEventInterceptor::XXXXXXXX] Found 12 buttons in scene at start. Hooking click interceptors...
```

This is useful to verify the button exists and can be found by the system.

## Troubleshooting Matrix

| Symptom | Check | Fix |
|---------|-------|-----|
| No injection logs at all | Session ID in first log | Game restarted? Plugin rebuilt? |
| STEP X ⚠ FAILED | Error message | See specific step fix below |
| Button state FALSE | STEP 8 verification | Re-examine STEP that set it |
| Hover highlight doesn't work | Visual inspection | Check Button.colors.highlightedColor |
| Click goes to Options instead | UiEventInterceptor logs | Button destroyed or not injected |
| No click log appears | OnModsButtonClicked code | onClick listener not wired (STEP 6) |

## STEP-specific Fixes

### STEP 1 Failed: Clone Button Returned Null
**Reason**: Settings/Options button not found properly
**Check**:
- Is `FindSettingsButton()` actually finding a button?
- Look for "Found 'Options' button" log
**Fix**: Manually locate the Settings button in scene hierarchy and update `NativeUiHelper.FindButtonByText` search

### STEP 2 Failed: Positioning Issue
**Reason**: RectTransform missing or parent mismatch
**Check**: STEP 2 log for which component is null
**Fix**: Verify Settings button has RectTransform (it should)

### STEP 3 Failed: Cannot Activate Button
**Reason**: gameObject.SetActive() threw exception
**Check**: Exception message in logs
**Fix**: Rare; likely a Unity version issue

### STEP 4 Failed: CanvasGroup Configuration
**Reason**: No CanvasGroup (OK - not required) or configuration failed
**Check**: Log message explains which part failed
**Fix**: Usually not a blocker unless log says blocksRaycasts=FALSE and you can't fix it

### STEP 5 Failed: Raycast Issue
**Reason**: No GraphicRaycaster on parent Canvas
**Check**: Log shows which raycast component is missing
**Fix**: Code auto-enables missing GraphicRaycaster

### STEP 6 Failed: onClick.AddListener Threw
**Reason**: onClick is null or listener registration failed
**Check**: Exception message in logs
**Fix**: Rare; indicates corrupted Button component

### STEP 7 Failed: EventSystem Issue
**Reason**: EventSystem.current is null or SetSelectedGameObject failed
**Check**: Log shows which part failed
**Fix**: May be timing issue (EventSystem not ready yet); injection will retry in 2 seconds

### STEP 8 Failed: Final Verification
**Reason**: One or more button state flags are wrong
**Check**: Which flag is FALSE/NULL
**Fix**: Examine the code that sets that flag; usually STEP 3 or 7 issue

## Log Analysis Template

Use this when filing a bug or debugging:

```
Session ID: XXXXXXXX (8-char code after ::)
Game start time: HH:MM:SS.fff UTC
Injection timestamp: HH:MM:SS.fff UTC
Injection attempt: #N
Last successful step: STEP X
Failed step: STEP Y (or "No failure")
Final button state:
  - activeSelf: [TRUE/FALSE]
  - activeInHierarchy: [TRUE/FALSE]
  - interactable: [TRUE/FALSE]
  - raycastTarget: [TRUE/FALSE]
  - navigation.mode: [None/Automatic/Vertical/Horizontal/Explicit]
Click attempts: N
Clicks registered: N
Menu visible after click: [YES/NO/UNKNOWN]
Global click intercept showed: [Button name or "no clicks"]
```

## Questions to Answer

1. **Is the session ID consistent?** (All logs from same run?)
2. **Which STEP failed first?** (If any)
3. **Are all button state flags TRUE?** (If injection succeeded)
4. **Did clicking log a MODS BUTTON CLICKED event?** (With same session ID)
5. **What button does the interceptor show being clicked?** (If not Mods)
