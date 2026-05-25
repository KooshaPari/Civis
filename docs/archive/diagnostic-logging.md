# Comprehensive Mods Button Diagnostics

## Overview

A comprehensive diagnostic logging system has been added to track the full lifecycle of the Mods button from injection through click handling. This allows us to definitively distinguish between:
- **Session 1**: Old logs from a previous game session (logs are cached/stale)
- **Session 2**: New logs from the current game session (injection happened THIS run)

## Session Identification

Every diagnostic log now includes a **unique 8-character session ID** that persists for the entire game session:

```
[NativeMenuInjector::a1b2c3d4] ===== PLUGIN SESSION START ===== Awake at 14:32:15.847 UTC
[NativeMenuInjector::a1b2c3d4] INJECTION ATTEMPT #1 at 14:32:16.052 UTC
[NativeMenuInjector::a1b2c3d4] ✓✓✓✓✓ MODS BUTTON INJECTION FULLY SUCCESSFUL
```

The same session ID will appear on ALL logs until you restart the game. A different session ID = a new run.

## Key Diagnostic Points

### 1. Plugin Startup (NativeMenuInjector)

- **Awake**: Timestamp + unique session ID
- **Start**: Called immediately after scene initialization
- **Session marker**: First log uniquely identifies THIS run

### 2. Injection Attempts (Numbered)

Each call to `TryInjectMenuButton()` is numbered sequentially:

```
[NativeMenuInjector::a1b2c3d4] ═══ INJECTION ATTEMPT #1 at 14:32:16.052 UTC ═══
[NativeMenuInjector::a1b2c3d4] Attempt#1: Scan started — found 15 canvases total
[NativeMenuInjector::a1b2c3d4] Attempt#1   Canvas 'MainMenuUI': NAME MATCHED ✓
[NativeMenuInjector::a1b2c3d4] Attempt#1   Canvas 'MainMenuUI': NO Settings/Options button found
[NativeMenuInjector::a1b2c3d4] Attempt#2 ✓✓✓ SUCCESS FOUND Settings button...
```

**Why it matters**: Shows if injection is actually being attempted on each frame vs. running old cached logs.

### 3. Button Injection Steps (Step 1-8)

Each `InjectButton()` call breaks down into 8 discrete steps:

1. **Clone button** - Create a copy of Settings/Options button
2. **Position** - Place Mods button after Settings in hierarchy
3. **Activate** - Ensure `.activeSelf` and `.interactable` are true
4. **CanvasGroup** - Configure raycast blocking on parent groups
5. **Raycast diagnostics** - Verify `raycastTarget`, parent visibility, GraphicRaycaster enabled
6. **Wire onClick** - Attach `OnModsButtonClicked` listener
7. **EventSystem selection** - Set as selected object, isolate from navigation graph
8. **Final verification** - Log all button state flags

Each step is clearly marked as OK or with a specific error:

```
[NativeMenuInjector::a1b2c3d4] Attempt#1   STEP 1 OK: Clone successful: 'DINOForge_ModsButton'
[NativeMenuInjector::a1b2c3d4] Attempt#1   STEP 3 OK: Button activated: active=True, interactable=True
[NativeMenuInjector::a1b2c3d4] Attempt#1   STEP 5: Raycast diagnostics...
[NativeMenuInjector::a1b2c3d4] Attempt#1     ⚠ raycastTarget is FALSE - ENABLING
[NativeMenuInjector::a1b2c3d4] Attempt#1   STEP 7 OK: EventSystem configuration complete
[NativeMenuInjector::a1b2c3d4] Attempt#1   STEP 8 OK: All checks passed
```

### 4. Button Click Logging

Every click on the Mods button is logged with a click counter and timestamp:

```
[NativeMenuInjector::a1b2c3d4] ═══ MODS BUTTON CLICKED #1 at 14:32:45.123 UTC ═══
[NativeMenuInjector::a1b2c3d4] Click#1   overlay.IsVisible BEFORE toggle: False
[NativeMenuInjector::a1b2c3d4] Click#1   overlay.IsVisible AFTER toggle: True
[NativeMenuInjector::a1b2c3d4] Click#1 ✓ Mods menu TOGGLED successfully
```

**Critical**: If you click the Mods button and see NO click log with the current session ID, the click handler was never called — the button wasn't properly wired.

### 5. Global UI Event Interceptor (UiEventInterceptor)

A new component logs **ALL button clicks** on the canvas to see what's actually being clicked:

```
[UiEventInterceptor::x9y8z7w6] ===== UI EVENT INTERCEPTOR STARTED ===== at 14:32:15.900 UTC
[UiEventInterceptor::x9y8z7w6] Found 12 buttons in scene at start. Hooking click interceptors...
[UiEventInterceptor::x9y8z7w6] ⚡ BUTTON CLICK #1 at 14:32:46.050 UTC
[UiEventInterceptor::x9y8z7w6]   Button name: 'OptionsButton'
[UiEventInterceptor::x9y8z7w6]   GameObject path: 'MainCanvas/Menu/OptionButton'
[UiEventInterceptor::x9y8z7w6]   Active: True, Interactable: True
[UiEventInterceptor::x9y8z7w6]   OnClick listeners: 3
```

**What this shows**: Even if the Mods button click handler doesn't fire, you'll see what button WAS clicked instead (e.g., Options button).

## Diagnostic Workflow

### Question 1: Was the button injected THIS run?

**Look for**:
```
[NativeMenuInjector::SESSION_ID] ===== PLUGIN SESSION START =====
[NativeMenuInjector::SESSION_ID] INJECTION ATTEMPT #1
```

If you see the same session ID and INJECTION ATTEMPT #1, it happened THIS run. If the timestamp is from 20 minutes ago and you just tested the button, the injection is NOT happening.

### Question 2: Where did injection fail?

**Look for**:
```
[NativeMenuInjector::SESSION_ID] Attempt#1   STEP X ⚠ FAILED
```

The failure will point to exactly which step broke:
- STEP 1: CloneButton returned null (bad template button?)
- STEP 2: RectTransform missing or parent mismatch
- STEP 3: gameObject.SetActive failed
- STEP 4: CanvasGroup configuration issue
- STEP 5: Raycast target disabled (code now auto-fixes)
- STEP 6: onClick.AddListener threw exception
- STEP 7: EventSystem null or SetSelectedGameObject failed
- STEP 8: Final state verification caught inconsistency

### Question 3: Did the button exist after injection?

**Look for**:
```
[NativeMenuInjector::SESSION_ID] Attempt#1   STEP 8 OK: Final button state verification
[NativeMenuInjector::SESSION_ID] Attempt#1     - gameObject.activeSelf: True
[NativeMenuInjector::SESSION_ID] Attempt#1     - gameObject.activeInHierarchy: True
[NativeMenuInjector::SESSION_ID] Attempt#1     - interactable: True
[NativeMenuInjector::SESSION_ID] Attempt#1     - targetGraphic.raycastTarget: True
```

If any of these are False, the button is not clickable.

### Question 4: Was the button destroyed after injection?

**Look for**:
```
[NativeMenuInjector::SESSION_ID] ⚠ INJECTED BUTTON WAS DESTROYED! Resetting injection flag
```

This indicates the button hierarchy was torn down (scene change or canvas rebuild).

### Question 5: Did clicking the button call the handler?

**Look for** (with your session ID):
```
[NativeMenuInjector::SESSION_ID] ═══ MODS BUTTON CLICKED #1 at HH:MM:SS.fff UTC ═══
[NativeMenuInjector::SESSION_ID] Click#1 ✓ Mods menu TOGGLED successfully
```

If you see NO click log after clicking the Mods button, one of:
- The OnClick listener wasn't attached (STEP 6 failed)
- The button is being destroyed before the click reaches the handler
- The click is being routed to a different button (check UiEventInterceptor)

### Question 6: Is something else being clicked instead?

**Look for** (UiEventInterceptor logs):
```
[UiEventInterceptor::SESSION_ID] ⚡ BUTTON CLICK #1 at 14:32:46.050 UTC
[UiEventInterceptor::SESSION_ID]   Button name: 'OptionsButton'
```

If you're clicking where the Mods button should be and OptionsButton is logged instead, the Mods button either:
- Never got injected
- Was destroyed
- Is layered behind the Options button (z-order issue)

## Log Output Location

BepInEx logs are written to:
```
DINO_INSTALL_DIR/BepInEx/LogOutput.log
```

Log lines include timestamps at the microsecond level, making it easy to correlate with game events.

## Summary

The diagnostic system makes it impossible to confuse:
- **Old logs from a previous run** (different or missing session ID, old timestamps)
- **Current run logs** (matching session ID, fresh timestamps)
- **Where in the pipeline the failure occurs** (STEP X with specific reason)
- **What the final button state is** (all 7 flags verified)
- **Whether clicks are being routed correctly** (global interceptor shows all clicks)

Use the session ID as the primary distinguisher when reading logs.
