# Hidden Desktop Rendering Test Plan

**Status**: Prototype Ready
**Date Created**: 2026-03-25
**Purpose**: Validate whether Unity D3D11 games render on Win32 hidden desktops
**Criticality**: Gate for DINOForge agent isolation feature

---

## Executive Summary

This document describes the validation test for a critical architectural decision: **can agents run the game invisibly using Win32 hidden desktops?**

If **YES** → Agents can launch the game completely invisible, enabling zero-focus-interference automation
If **NO** → Fall back to separate Windows user account for isolation

The decision directly impacts:
- Agent operational model (visible window vs. invisible)
- User experience (keyboard/mouse safety during automation)
- Architecture (desktop management APIs vs. user credential handling)

---

## What We're Testing

**Hypothesis**: Unity 2021.3.45f2 with D3D11 renderer produces frame output when launched on a Win32 hidden desktop (created via `CreateDesktop` API).

**Success Criteria**:
- Game process launches successfully on hidden desktop
- Game window appears (even though desktop is hidden)
- Screenshot captured from hidden window contains non-black pixels
- Indicates D3D11 is rendering frames even when desktop is invisible

**Failure Criteria**:
- Game fails to launch
- Game launches but window doesn't appear
- Screenshot is entirely black (no rendering detected)
- Indicates hidden desktop breaks D3D11 rendering pipeline

---

## Test Architecture

```
┌─────────────────────────────────────────────────────┐
│        PowerShell Test Script                       │
│  hidden_desktop_test.ps1 (≈500 lines)              │
└────────────┬────────────────────────────────────────┘
             │
             ├─► P/Invoke Layer (Win32)
             │   • CreateDesktop() - create hidden desktop
             │   • CreateProcess() - launch game with lpDesktop set
             │   • FindWindow() - locate game window
             │   • GetDC()/CreateCompatibleDC() - GDI graphics capture
             │   • BitBlt() - copy framebuffer to compatible bitmap
             │   • FromHbitmap() - convert to managed Bitmap
             │   • GetPixel() - analyze pixel content
             │
             ├─► Test Sequence (≈30 seconds total)
             │   1. Create "DINOForge_Test_XXXX" hidden desktop
             │   2. Launch game exe via CreateProcess(STARTUPINFO.lpDesktop=hidden)
             │   3. Wait 15s for game window to appear
             │   4. If window found, wait 3s for rendering startup
             │   5. Capture screenshot via GDI BitBlt
             │   6. Analyze: is it mostly black or rendered?
             │   7. Kill game process
             │   8. Destroy desktop
             │
             └─► Output
                 • Screenshot: $env:TEMP\DINOForge\hidden_desktop_test.png
                 • Exit code: 0 (SUCCESS), 1 (FAILURE), 2 (ERROR)
                 • Console report: detailed findings + implications
```

---

## How to Run

### Prerequisites
- Windows 10/11 Pro or higher (hidden desktops require admin/special privileges)
- PowerShell 5.1+ (pwsh v7+ recommended)
- Game installed at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- Adequate disk space in `$env:TEMP\` (≈5MB for screenshot)

### Command
```powershell
# Navigate to repo root
cd C:\Users\koosh\Dino

# Run test
pwsh -File scripts/game/hidden_desktop_test.ps1

# Or with verbose output
pwsh -File scripts/game/hidden_desktop_test.ps1 -Verbose

# Or with custom game path
pwsh -File scripts/game/hidden_desktop_test.ps1 `
    -GameExePath "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe" `
    -GameDir "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
```

### Expected Runtime
- Total: ~20-30 seconds
- Game launch: 2-3 seconds
- Window appearance: 3-5 seconds (configurable via `-WaitSeconds`)
- Screenshot capture: <1 second
- Cleanup: 1-2 seconds

---

## Understanding Results

### ✓ SUCCESS

```
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
  2. Add desktop management to Runtime bridge
  3. Document in README: 'Agent Mode' (hidden desktop rendering)
  4. Create slash command: /launch-game-agent
```

**What this means**:
- The game's D3D11 renderer produces frames even when the desktop is hidden from view
- From a security/UX perspective, we can run the game completely invisible
- Agents can interact with the game without stealing focus or interfering with user's keyboard/mouse
- This is the **ideal** scenario for agent automation

**Implementation path**:
```csharp
// In MCP GameLauncher
var desktop = Win32.CreateDesktop("DINOForge_Agent_" + Guid.NewGuid());
var process = Win32.CreateProcess(
    exe: gamePath,
    startupInfo: new STARTUPINFO {
        lpDesktop = "WinSta0\\" + desktopName,
        dwFlags = STARTF_USESTDHANDLES
    }
);
// Game renders invisibly; agent can capture and interact
var screenshot = Win32.BitBlt(...);
// When done, destroy process and desktop
Win32.TerminateProcess(process);
Win32.CloseDesktop(desktop);
```

---

### ✗ FAILURE

```
Status: ✗ FAILURE

Finding:
  Unity D3D11 rendering DOES NOT work on hidden desktops
  (or game failed to launch entirely)

Implications:
  • Hidden desktop approach is NOT viable
  • Must fall back to separate Windows user account
  • Each game instance requires dedicated user session

Next Steps:
  1. Implement user account isolation (via runas.exe)
  2. Set up test game directory at known path per user
  3. Update launcher to use separate credential token
  4. Document in README: 'Isolated Mode' (separate user account)
  5. Create slash command: /launch-game-isolated
```

**What this means**:
- Either the hidden desktop creation failed, the game couldn't launch on it, or D3D11 refuses to render to an invisible surface
- This is actually **not unexpected** — many GPU drivers require a visible window/desktop for full rendering support
- The fallback strategy is to use a separate Windows user account

**Fallback implementation**:
```powershell
# Instead of hidden desktop, use separate user account
$cred = Get-Credential -UserName "DINOForge_Agent" -Message "Agent account"
Start-Process -FilePath $gamePath `
    -Credential $cred `
    -WorkingDirectory $gameDir `
    -WindowStyle Hidden
```

Considerations:
- Requires creating a second Windows user (or using existing automation account)
- Agent runs under that user's session (separate desktop, separate processes)
- Screenshot capture requires inter-process communication or RPC
- More complex than hidden desktop approach but known to work

---

## Interpreting the Screenshot

If the test **succeeds**, examine the screenshot:

**Expected (SUCCESS indicator)**:
- Game menu visible (main menu with "Start Game" button)
- Or: game in-gameplay (units, buildings, resource bars visible)
- Color: mix of faction colors, UI elements
- Not uniform black (0,0,0)

**Unexpected (FAILURE indicator)**:
- Entirely black or dark gray
- No UI elements visible
- Blank/uninitialized state
- Suggests: game rendered to memory, not to screen, or D3D11 not producing output

---

## Debugging Failed Tests

If the test **fails**, use this troubleshooting checklist:

### Game Window Doesn't Appear
```powershell
# Check if game even starts
# Run with verbose output
pwsh -File scripts/game/hidden_desktop_test.ps1 -Verbose

# Look for:
# • "Game window found after X.Xs" = window appeared (good)
# • "Game window not found after 15s" = window never appeared (bad)

# If window never appears:
# 1. Verify game.exe path is correct
# 2. Check if game is already running (kill it first)
# 3. Verify game directory has BepInEx plugins
# 4. Check Event Viewer > Windows Logs > System for game launch errors
```

### Screenshot Captured But Is Black
```powershell
# This might mean:
# 1. Game rendered to memory buffer, not screen
# 2. D3D11 device initialization failed
# 3. GPU driver doesn't support hidden desktop rendering
# 4. Game is running but in an uninitialized state

# Next attempt:
# Increase WaitSeconds to 30
pwsh -File scripts/game/hidden_desktop_test.ps1 -WaitSeconds 30

# If still black after 30s, D3D11/driver issue confirmed
```

### Screenshot File Not Saved
```powershell
# Check output directory
Test-Path $env:TEMP\DINOForge\
Get-ChildItem $env:TEMP\DINOForge\

# If directory doesn't exist or is inaccessible:
# Create it manually
New-Item -ItemType Directory -Path "$env:TEMP\DINOForge" -Force

# Check file permissions
icacls "$env:TEMP\DINOForge"
```

### P/Invoke Errors
```
"Failed to create desktop: error code 1314"
```

This usually means insufficient privileges. Hidden desktop creation requires:
- Admin mode, OR
- SE_CREATEPERMANENT_NAME privilege

**Solution**: Run PowerShell as Administrator
```powershell
# From elevated prompt
pwsh -NoProfile -Command "& {
    Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process
    & 'C:\Users\koosh\Dino\scripts\game\hidden_desktop_test.ps1'
}"
```

---

## Technical Details

### Win32 P/Invoke Signatures Used

| API | Purpose | Notes |
|-----|---------|-------|
| `CreateDesktop()` | Create hidden desktop | Returns IntPtr handle |
| `CreateProcess()` | Launch game on desktop | Via STARTUPINFO.lpDesktop parameter |
| `FindWindow()` | Locate window by title | Polls for "Diplomacy is Not an Option" |
| `GetDC()` / `CreateCompatibleDC()` | Get device contexts | For screen capture |
| `CreateCompatibleBitmap()` | Create memory bitmap | To hold captured pixels |
| `BitBlt()` | Copy framebuffer | From window DC to memory bitmap |
| `FromHbitmap()` | Convert HBITMAP | To managed System.Drawing.Bitmap |
| `GetPixel()` | Analyze pixels | Detect non-black regions |
| `CloseDesktop()` | Destroy desktop | Cleanup |

### Why These Choices?

**CreateDesktop API**:
- Only way to create a desktop invisible to user
- Alternative: RDP/VNC would require network, visible window, or 3rd-party tools
- Direct API access = no dependencies, no overhead

**GDI BitBlt (not DXGI)**:
- Works from any window, any rendering API
- DXGI would require DXGI1.2+ and specific GPU capabilities
- BitBlt = universal, proven method

**Polling for Window** (not WaitForInputIdle):
- Game runs ECS in background; no guaranteed input idle state
- Manual window find + retry = reliable for game launch

### Known Limitations

1. **Admin Privilege**: Hidden desktop creation may require elevation
2. **Accessibility**: Cannot interact with hidden desktop via normal keyboard input
3. **GPU Driver Dependent**: Some drivers may not support rendering to invisible desktops
4. **Single Desktop Instance**: Only one hidden desktop per test run (destroy before next test)

---

## Success Criteria (Detailed)

### Phase 1: Desktop Creation ✓
- CreateDesktop() call succeeds (returns valid IntPtr)
- Desktop handle is valid and can be used

**Failure modes**:
- Error code 1314 (insufficient privilege) → need elevation
- Error code 5 (access denied) → permission issues
- Null/zero handle → desktop creation rejected by OS

### Phase 2: Process Launch ✓
- CreateProcess() succeeds with lpDesktop set
- Process ID is returned and is valid
- Game process appears in Task Manager

**Failure modes**:
- Error code 2 (file not found) → bad game path
- Error code 5 (access denied) → permission issues
- CreateProcess returns false → OS rejected request

### Phase 3: Window Appearance ✓
- FindWindow() locates a window with title "Diplomacy is Not an Option"
- Window is valid (IsWindow() returns true)
- Window handle is not zero

**Failure modes**:
- FindWindow() returns null after 15s → game didn't create main window
- Window is invalid or destroyed immediately → game crashed

### Phase 4: Screenshot Capture ✓
- GetDC() succeeds (valid device context)
- GetWindowRect() returns valid dimensions (width &gt; 0, height &gt; 0)
- BitBlt() copies pixels without error
- Bitmap is saved to PNG file successfully

**Failure modes**:
- GetDC() returns null → device access denied
- Negative/zero dimensions → window is degenerate
- BitBlt() fails → framebuffer access denied
- File save fails → I/O or permission error

### Phase 5: Content Analysis ✓
- Screenshot is successfully loaded as Bitmap
- Pixel sampling detects non-black pixels (R>10 or G>10 or B>10)
- At least 3 of 5 sample points are non-black
- Indicates rendered content (not uninitialized buffer)

**Failure modes**:
- All sampled pixels are black → no rendering
- Only 1-2 non-black pixels → possibly noise/artifact

---

## Architecture Decision Record (ADR)

**Title**: Hidden Desktop Rendering for Agent Isolation

**Status**: Testing in Progress

**Context**:
- Agents need to run the game autonomously without visual/focus interference
- Three options for isolation:
  1. Hidden desktop (Win32 CreateDesktop)
  2. Separate user account (runas.exe)
  3. Remote desktop/VNC (external tool)

**Decision**:
- Implement hidden desktop first (simplest, no dependencies)
- Test whether D3D11 rendering works
- If test succeeds → adopt hidden desktop approach
- If test fails → implement user account fallback

**Consequences**:
- **If SUCCESS**:
  - Desktop API calls in MCP launcher
  - Runtime bridge gains desktop management APIs
  - Zero user focus interference
  - Implementation &lt; 100 lines of code

- **If FAILURE**:
  - User account/credential management in launcher
  - Complex credential handling
  - Requires second Windows user account
  - Implementation ~300+ lines of code

**Related Decisions**:
- Agent operational model (visible vs. invisible)
- Screenshot/input capture mechanism (GDI vs. DXGI vs. RDP)
- Launch safety (focus stealers, keyboard hijacking)

---

## Integration with DINOForge

### Files to Update (upon SUCCESS)

1. **src/Tools/McpServer/Program.cs**
   - Add DesktopManager service
   - Register CreateDesktop/CloseDesktop RPC calls

2. **src/Tools/McpServer/Tools/GameLauncher.cs** (new file)
   - Update launch logic to use hidden desktop by default
   - Add flag: `--agent-mode` for invisible launch
   - Add flag: `--visible` to override (for debugging)

3. **README.md**
   - Document "Agent Mode" feature
   - Explain hidden desktop isolation
   - Show example: `game_launch --agent-mode`

4. **docs/AGENT_ISOLATION.md** (new)
   - Deep dive into hidden desktop architecture
   - P/Invoke reference
   - Troubleshooting guide

5. **.claude/commands/launch-game-agent** (new skill)
   - Wraps `game_launch --agent-mode`
   - Verifies desktop created successfully
   - Captures proof screenshot

### Files to Update (upon FAILURE)

1. **src/Tools/McpServer/Tools/GameLauncher.cs**
   - Implement user account fallback
   - Use runas.exe or Win32 CreateProcessAsUser()
   - Handle credential management

2. **docs/AGENT_ISOLATION.md**
   - Document user account approach
   - Setup guide for second Windows user
   - RPC/inter-process communication strategy

3. **.claude/commands/launch-game-isolated** (new skill)
   - Wraps user account launch
   - Manages credentials securely

---

## Metrics & Logging

The script logs the following to console (and can be captured to file):

```
[HH:mm:ss] Hidden Desktop Rendering Test for DINOForge
[HH:mm:ss] Game: G:\SteamLibrary\...
[HH:mm:ss] Output: $env:TEMP\DINOForge\hidden_desktop_test.png
✓ Created hidden desktop: DINOForge_Test_XXXX
✓ Launched game on hidden desktop (PID: XXXXX)
✓ Game window found after X.Xs
[HH:mm:ss] Game window found, waiting for rendering...
[HH:mm:ss] Capturing screenshot...
✓ Screenshot saved: $env:TEMP\DINOForge\hidden_desktop_test.png (1920x1080)

========================================
TEST RESULT
========================================

Status: ✓ SUCCESS
```

**To capture logs to file**:
```powershell
pwsh -File scripts/game/hidden_desktop_test.ps1 -Verbose 2>&1 | Tee-Object -FilePath hidden_desktop_test_log.txt
```

---

## References

### Win32 Documentation
- [CreateDesktop](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createdesktopw)
- [CreateProcess](https://docs.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessa)
- [BitBlt](https://docs.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt)

### Related Research
- DINO Runtime Execution Model (`docs/sessions/project_dino_runtime_execution_model.md`)
- DINOForge Agent Governance (`CLAUDE.md`)
- Game Bridge Protocol (`src/Bridge/Protocol/`)

---

## Sign-Off

**Test Created**: 2026-03-25
**Test Framework**: PowerShell 5.1+ with Win32 P/Invoke
**Status**: Ready for Execution
**Owner**: DINOForge Development (Koosha Paridehpour)

**Next Action**: Run `pwsh -File scripts/game/hidden_desktop_test.ps1` and document results in this file.
