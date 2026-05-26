# ▶ RUN THE HIDDEN DESKTOP TEST NOW

## One Command to Execute

```powershell
cd C:\Users\koosh\Dino
pwsh -File scripts/game/hidden_desktop_test.ps1
```

**Runtime**: 20-30 seconds (fully automated)

---

## What Happens Automatically

1. Creates invisible desktop named `DINOForge_Test_XXXX`
2. Launches game on that desktop
3. Waits 15 seconds for game window to appear
4. Captures a screenshot of the game window
5. Analyzes if the screenshot contains rendered content (non-black)
6. Kills the game and destroys the desktop
7. **Prints final result: ✓ SUCCESS or ✗ FAILURE**

---

## If You Get an Error

### Error: "Failed to create desktop: error code 1314"

**Solution**: Run PowerShell as Administrator

1. Right-click PowerShell → "Run as Administrator"
2. Run the command above again

---

## Result Interpretation

### ✓ SUCCESS = Hidden Desktop Works!

```
Status: ✓ SUCCESS

Finding:
  Unity D3D11 rendering WORKS on hidden desktops
```

**What it means**:
- Game renders frames even when the desktop is invisible
- Agents can run the game completely invisibly
- Zero risk of focus stealing or keyboard/mouse interference
- **This is the best possible outcome**

---

### ✗ FAILURE = Hidden Desktop Doesn't Work

```
Status: ✗ FAILURE

Finding:
  Unity D3D11 rendering DOES NOT work on hidden desktops
```

**What it means**:
- GPU driver or D3D11 doesn't support rendering to invisible surfaces
- Must use separate Windows user account instead
- More complex but known to work
- **This is still acceptable; just use fallback approach**

---

## Where's the Screenshot?

After test completes, screenshot is saved to:
```
$env:TEMP\DINOForge\hidden_desktop_test.png
```

To view it:
```powershell
explorer "$env:TEMP\DINOForge\"
```

**On SUCCESS**: You'll see the game rendered (menu or gameplay)
**On FAILURE**: You'll see a black/empty image (no rendering)

---

## After the Test Completes

1. **Note the result** (SUCCESS or FAILURE)
2. **Review the screenshot** to confirm
3. **Read the appropriate guide** below:

### If SUCCESS → Read This
- `HIDDEN_DESKTOP_TEST_QUICKSTART.md` (Success section)
- `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md` (Integration path → If SUCCESS)

### If FAILURE → Read This
- `HIDDEN_DESKTOP_TEST_QUICKSTART.md` (Failure section)
- `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md` (Integration path → If FAILURE)

### Want Full Details?
- `HIDDEN_DESKTOP_TEST_PLAN.md` (complete architecture, debugging, ADR)
- `HIDDEN_DESKTOP_PINVOKE_REFERENCE.md` (all Win32 API calls explained)

---

## Key Files

| File | What It Is |
|------|-----------|
| `scripts/game/hidden_desktop_test.ps1` | **The Test** (run this) |
| `HIDDEN_DESKTOP_TEST_QUICKSTART.md` | Quick reference (read after test) |
| `HIDDEN_DESKTOP_TEST_PLAN.md` | Complete plan (if you want details) |
| `HIDDEN_DESKTOP_PINVOKE_REFERENCE.md` | API reference (for developers) |
| `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md` | Summary of what was built |

---

## Expected Output

Here's what you'll see when you run the test:

```
[HH:mm:ss] Hidden Desktop Rendering Test for DINOForge
[HH:mm:ss] Game: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe
[HH:mm:ss] Output: C:\Users\koosh\AppData\Local\Temp\DINOForge\hidden_desktop_test.png
✓ Created hidden desktop: DINOForge_Test_5847
✓ Launched game on hidden desktop (PID: 12345)
[HH:mm:ss] Waiting for game to load...
✓ Game window found after 4.2s
[HH:mm:ss] Game window found, waiting for rendering...
[HH:mm:ss] Capturing screenshot...
✓ Screenshot saved: C:\Users\koosh\AppData\Local\Temp\DINOForge\hidden_desktop_test.png (1920x1080)
[HH:mm:ss] Cleaning up...

========================================
TEST RESULT
========================================

Status: ✓ SUCCESS

Finding:
  Unity D3D11 rendering WORKS on hidden desktops

Implications:
  • Agents can launch game completely invisibly
  • Zero window focus interference with user's session
  • Safe for concurrent automation + user work
  • Implementation: Use CreateDesktop API in game launcher

Next Steps:
  1. Update GameLauncher in MCP server to use hidden desktops by default
  2. Add desktop management to Runtime bridge (create/destroy/list)
  3. Document in README: 'Agent Mode' (hidden desktop rendering)
  4. Create slash command: /launch-game-agent (uses hidden desktop)
```

---

## Time to Run

- **Actual test**: ~15 seconds (game launch + screenshot)
- **Overhead** (setup + cleanup): ~5-10 seconds
- **Total**: ~20-30 seconds

---

## What Comes Next

**If SUCCESS**:
1. Update MCP launcher to create hidden desktops by default
2. Add `/launch-game-agent` slash command
3. Update README.md documentation
4. Create `docs/AGENT_ISOLATION.md`

**If FAILURE**:
1. Implement user account fallback in launcher
2. Add `/launch-game-isolated` slash command
3. Update README.md documentation
4. Create `docs/AGENT_ISOLATION.md`

Both paths are documented in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`.

---

## Questions?

- **How does it work?** → See `HIDDEN_DESKTOP_TEST_PLAN.md`
- **What are the P/Invoke calls?** → See `HIDDEN_DESKTOP_PINVOKE_REFERENCE.md`
- **What if something fails?** → See "Debugging Failed Tests" in `HIDDEN_DESKTOP_TEST_PLAN.md`
- **What do I do with the result?** → See "Integration Path" in `HIDDEN_DESKTOP_DELIVERY_SUMMARY.md`

---

## TL;DR

**Just run this:**
```powershell
cd C:\Users\koosh\Dino
pwsh -File scripts/game/hidden_desktop_test.ps1
```

**It will tell you** if agents can run the game invisibly (SUCCESS) or need to use a separate user account (FAILURE).

**That's it.** ✓

---

**Created**: 2026-03-25
**For**: Immediate execution
**Owner**: DINOForge Development
