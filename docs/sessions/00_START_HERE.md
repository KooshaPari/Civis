# START HERE — Hidden Desktop Test Prototype

**Status**: ✓ READY FOR IMMEDIATE EXECUTION
**Created**: 2026-03-25
**Purpose**: Validate whether Unity D3D11 games render on Win32 hidden desktops (critical gate for agent isolation)

---

## Quick Start (5 Minutes)

### Step 1: Run the Test
```powershell
cd C:\Users\koosh\Dino
pwsh -File scripts/game/hidden_desktop_test.ps1
```

**That's it.** The test runs automatically. Takes 20-30 seconds.

### Step 2: Understand the Result

The test will print either:

```
Status: ✓ SUCCESS
Finding: Unity D3D11 rendering WORKS on hidden desktops
```

OR

```
Status: ✗ FAILURE
Finding: Unity D3D11 rendering DOES NOT work on hidden desktops
```

### Step 3: Read the Appropriate Guide

- **If SUCCESS**: Read "Integration Path → If SUCCESS" in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`
- **If FAILURE**: Read "Integration Path → If FAILURE" in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`

---

## What This Does

**Question**: Can agents run the game completely invisibly on a hidden Windows desktop?

**Test**: Creates a Win32 hidden desktop, launches the game on it, captures a screenshot, and checks if D3D11 actually rendered anything.

**Impact**:
- **SUCCESS** → Agents can run invisible, zero focus interference ✓
- **FAILURE** → Must use separate user accounts, more complex but workable ✓

---

## Files You're Working With

| File | What It Is | Read If... |
|------|-----------|-----------|
| `scripts/game/hidden_desktop_test.ps1` | **The Test** | You want to understand how it works |
| `RUN_HIDDEN_DESKTOP_TEST_NOW.md` | Quick start | You want to run it right now |
| `HIDDEN_DESKTOP_TEST_QUICKSTART.md` | One-page reference | You want to understand results |
| `HIDDEN_DESKTOP_TEST_PLAN.md` | Full documentation | You want complete architecture/debugging |
| `HIDDEN_DESKTOP_PINVOKE_REFERENCE.md` | P/Invoke API reference | You want to modify or port the test |
| `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md` | Summary & integration | You want to know what to do next |
| `HIDDEN_DESKTOP_FILES_MANIFEST.md` | File index | You want to navigate all files |
| **00_START_HERE.md** | This file | You're reading it now |

---

## The Test in 30 Seconds

```
1. Create invisible desktop "DINOForge_Test_XXXX"
   ↓
2. Launch game on that hidden desktop
   ↓
3. Wait for game window to appear (up to 15s)
   ↓
4. Capture screenshot via GDI BitBlt
   ↓
5. Analyze: does it contain rendered content (non-black)?
   ↓
6. Kill game, destroy desktop
   ↓
7. Print result: ✓ SUCCESS or ✗ FAILURE
```

---

## Expected Output

```
[HH:mm:ss] Hidden Desktop Rendering Test for DINOForge
[HH:mm:ss] Game: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\...
[HH:mm:ss] Output: C:\Users\koosh\AppData\Local\Temp\DINOForge\hidden_desktop_test.png

✓ Created hidden desktop: DINOForge_Test_5847
✓ Launched game on hidden desktop (PID: 12345)
✓ Game window found after 4.2s
✓ Screenshot saved: C:\Users\koosh\AppData\Local\Temp\DINOForge\hidden_desktop_test.png (1920x1080)

========================================
TEST RESULT
========================================

Status: ✓ SUCCESS

Finding:
  Unity D3D11 rendering WORKS on hidden desktops

(or ✗ FAILURE if rendering doesn't work on hidden desktop)
```

---

## What Happens If...

### Error: "Failed to create desktop: error code 1314"
**Solution**: Run PowerShell as Administrator

### Error: "Game window not found after 15s"
**Solution**: Game took too long, or game crashed. Try again or increase wait time to 30s.

### Screenshot is black/blank
**Result**: This is FAILURE (D3D11 not rendering to hidden desktop). Use user account fallback.

---

## Next Steps Based on Result

### ✓ SUCCESS (Hidden Desktop Works!)

**What it means**:
- Game renders invisibly on hidden desktop
- Agents can run completely invisible
- Zero focus interference
- This is the best outcome

**What to do**:
1. Update MCP launcher to create hidden desktops
2. Create `/launch-game-agent` slash command
3. Update README.md with "Agent Mode" feature
4. See detailed plan in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`

### ✗ FAILURE (Hidden Desktop Doesn't Work)

**What it means**:
- GPU driver/D3D11 doesn't support invisible rendering
- Must use separate Windows user account
- More complex but known to work
- This is an acceptable outcome

**What to do**:
1. Implement user account fallback in launcher
2. Create `/launch-game-isolated` slash command
3. Update README.md with "Isolated Mode" feature
4. See detailed plan in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`

---

## Where's the Screenshot?

After test completes:
```
$env:TEMP\DINOForge\hidden_desktop_test.png
```

To view it:
```powershell
explorer "$env:TEMP\DINOForge\"
```

---

## Complete File Structure

```
C:\Users\koosh\Dino\
├─ scripts/game/
│  └─ hidden_desktop_test.ps1 ................... Main test (634 lines)
│
└─ docs/sessions/
   ├─ 00_START_HERE.md .......................... This file
   ├─ RUN_HIDDEN_DESKTOP_TEST_NOW.md ........... Quick start
   ├─ HIDDEN_DESKTOP_TEST_QUICKSTART.md ....... One-page ref
   ├─ HIDDEN_DESKTOP_TEST_PLAN.md ............. Full plan
   ├─ HIDDEN_DESKTOP_PINVOKE_REFERENCE.md .... API reference
   ├─ HIDDEN_DESKTOP_DELIVERY_SUMMARY.md ..... Summary
   ├─ HIDDEN_DESKTOP_PROTOTYPE.md ............. Full synthesis
   └─ HIDDEN_DESKTOP_FILES_MANIFEST.md ....... File index

Screenshot output:
   $env:TEMP\DINOForge\hidden_desktop_test.png
```

---

## Reading Guide

**For Different Roles**:

| You Are | Read This First | Then Read |
|---------|-----------------|-----------|
| **In a hurry** | This file | RUN_HIDDEN_DESKTOP_TEST_NOW.md |
| **Tester** | RUN_HIDDEN_DESKTOP_TEST_NOW.md | HIDDEN_DESKTOP_TEST_QUICKSTART.md |
| **Developer** | HIDDEN_DESKTOP_TEST_QUICKSTART.md | HIDDEN_DESKTOP_PINVOKE_REFERENCE.md |
| **Architect** | HIDDEN_DESKTOP_DELIVERY_SUMMARY.md | HIDDEN_DESKTOP_TEST_PLAN.md |
| **Integrator** | HIDDEN_DESKTOP_DELIVERY_SUMMARY.md | (Integration path section) |

---

## Key Technical Facts

- **P/Invoke**: Pure Win32 API (CreateDesktop, CreateProcess, BitBlt)
- **Dependencies**: None (standard PowerShell + System.Drawing)
- **Automation**: Fully automated, no user interaction
- **Duration**: ~20-30 seconds
- **Output**: Screenshot proof + console result
- **Cross-platform**: Windows only (Win32 APIs)
- **Admin required**: Maybe (for desktop creation)

---

## Support

| Question | Answer |
|----------|--------|
| How do I run it? | `pwsh -File scripts/game/hidden_desktop_test.ps1` |
| How long does it take? | 20-30 seconds |
| What could go wrong? | See troubleshooting in HIDDEN_DESKTOP_TEST_QUICKSTART.md |
| What does SUCCESS mean? | Hidden desktop rendering works → use that approach |
| What does FAILURE mean? | User account fallback needed → more complex but works |
| Where's the screenshot? | `$env:TEMP\DINOForge\hidden_desktop_test.png` |
| What do I do next? | Read the integration plan in HIDDEN_DESKTOP_DELIVERY_SUMMARY.md |

---

## Summary

**Before**: Don't know if hidden desktops work with Unity D3D11
**After**: You'll know for sure (SUCCESS or FAILURE)
**Time**: 30 seconds + 5 minutes to read results

**Command**:
```powershell
pwsh -File scripts/game/hidden_desktop_test.ps1
```

**That's all you need.**

---

**Created**: 2026-03-25
**Owner**: DINOForge Development
**Status**: ✓ Ready to Go

Go run the test. Everything is automated. The result will tell you exactly what to do next.
