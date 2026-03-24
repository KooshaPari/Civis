# ISSUE-044: InitialGameLoader Splash Screen Requires User Click

**Status**: Reported
**Priority**: High
**Component**: Game Initialization (Unity Engine, Not DINOForge)
**Related ADR**: ADR-014-automated-feature-testing
**Blocking**: `/prove-features` autonomous skill, regression test automation

---

## Problem Description

DINO's InitialGameLoader splash screen (displayed at game startup before the main menu) requires a physical user click (or key press) to advance to the main menu. This blocks automated testing scenarios where agents need to autonomously prove features without user interaction.

### Current Behavior

1. User launches game via `launch-game.ps1` or Steam
2. InitialGameLoader scene appears (Unity Engine logo, splash artwork, music)
3. **Game waits for user input** (mouse click or any key press)
4. Only after user interaction does the game transition to the main menu
5. BepInEx / DINOForge fully initialize in background (invisible to user at this stage)

### Impact on Automation

- **`/prove-features` skill**: Cannot proceed past InitialGameLoader without user interaction
- **Regression tests**: Cannot run autonomously (blocked at splash screen)
- **CI/CD pipelines**: Cannot validate features in automated workflows
- **Agent-driven testing**: Violates "never ask user to launch/interact with game" principle (see MEMORY.md)

### Root Cause

InitialGameLoader is part of the DINO game engine (not DINOForge). It likely calls `WaitForInput()` or similar blocking pattern:

```csharp
// Hypothetical InitialGameLoader code (game engine, not ours)
public class InitialGameLoader : MonoBehaviour
{
    private void Update()
    {
        if (Input.anyKey || Input.GetMouseButton(0))
        {
            // Advance to main menu
            SceneManager.LoadScene("MainMenu");
        }
    }
}
```

This is a **game-level design choice**, not a DINOForge platform feature. We cannot remove it without modifying the game itself.

---

## Acceptance Criteria

The `/prove-features` skill is considered **fully autonomous** when:

1. **Hands-free execution**: `dinoforge prove-features` runs without any user interaction
2. **Completes main menu tests**: Feature verification (F9/F10 keys, Mods button) proceeds after splash screen
3. **Completes gameplay tests**: Enters a gameplay scene, verifies F9/F10 in-game, captures video
4. **Video proof complete**: Full feature test recorded to `videos/prove-features-*.mp4`
5. **No user intervention**: Splash screen automatically bypassed as part of test setup

---

## Proposed Solutions

### Option 1: Win32 SendInput Mouse Click Simulation (Recommended)

Simulate a mouse click at the center of the game window to advance the splash screen.

**Pros:**
- Works reliably across Windows versions
- Minimal code overhead
- No patch to game required
- Fully autonomous

**Cons:**
- Win32 P/Invoke required (already used for F9/F10 in RuntimeDriver)
- Requires accurate window detection (game may not be focused)

**Implementation**:

```csharp
// File: src/Tools/Cli/Commands/ProveFeatures/AutoAdvanceSplash.cs

[DllImport("user32.dll")]
private static extern IntPtr FindWindow(string? lpClassName, string lpWindowName);

[DllImport("user32.dll")]
private static extern void SetCursorPos(int X, int Y);

[DllImport("user32.dll")]
private static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, IntPtr dwExtraInfo);

private const uint MOUSEEVENTF_LEFTDOWN = 2;
private const uint MOUSEEVENTF_LEFTUP = 4;

public static void AdvanceSplashScreen()
{
    // Find game window
    var gameWindow = FindWindow(null, "Diplomacy is Not an Option");
    if (gameWindow == IntPtr.Zero)
    {
        Console.WriteLine("ERROR: Game window not found");
        return;
    }

    // Get window center
    var centerX = 640;   // Approximate, depends on resolution
    var centerY = 360;

    // Simulate mouse click
    SetCursorPos(centerX, centerY);
    mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, IntPtr.Zero);
    System.Threading.Thread.Sleep(50);
    mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, IntPtr.Zero);

    Console.WriteLine("[SplashBypass] Mouse click simulated at ({0}, {1})", centerX, centerY);
}
```

**Integration into `/prove-features`**:

```csharp
// File: src/Tools/Cli/Commands/ProveFeatures/ProveFeature.cs

public async Task<int> Execute()
{
    // Phase 0: Wait for game window to appear (InitialGameLoader)
    Console.WriteLine("[prove-features] Waiting for game window...");
    await Task.Delay(3000);  // Initial delay for game launch

    // Phase 1: Advance splash screen
    Console.WriteLine("[prove-features] Advancing splash screen (auto-click)...");
    AutoAdvanceSplash.AdvanceSplashScreen();
    await Task.Delay(3000);  // Wait for main menu to load

    // Phase 2+: Feature tests (F9/F10, Mods button, etc.)
    // ... (existing test code)
}
```

**Testing this approach**:
- [ ] Verify mouse click reaches center of game window
- [ ] Verify click occurs after InitialGameLoader is visible
- [ ] Verify click advances to main menu without errors
- [ ] Verify game is responsive after auto-click

### Option 2: Harmony Patch on InitialGameLoader.Update

Patch the game's InitialGameLoader to auto-advance on startup (instead of waiting for user input).

**Pros:**
- Direct control over splash behavior
- Clean integration once patched

**Cons:**
- Requires identifying exact method signature of InitialGameLoader
- Might break if game updates
- Violates ADR-005 (no Harmony patches on external systems)
- May require multiple patches for different game versions

**Implementation sketch**:

```csharp
[HarmonyPatch(typeof(InitialGameLoader), nameof(InitialGameLoader.Update))]
public static class InitialGameLoaderPatches
{
    public static bool Prefix()
    {
        // Skip the input wait, proceed to next scene
        SceneManager.LoadScene("MainMenu");
        return false;  // Skip original Update
    }
}
```

**Status**: Not recommended (violates ADR-005, fragile).

### Option 3: Accept Limitation & Document

Document this as a known limitation of autonomous testing and require manual game launch for final validation.

**Pros:**
- Zero code overhead
- Fully transparent

**Cons:**
- Breaks "never ask user to launch/interact" principle
- Cannot integrate `/prove-features` into CI/CD
- Defeats the purpose of autonomous feature validation

**Status**: Not acceptable for this project (agent-driven development).

---

## Recommended Solution

**Use Option 1 (Win32 SendInput mouse click simulation)** because:

1. ✓ Aligns with agent-driven development model (fully autonomous)
2. ✓ Uses existing Win32 P/Invoke infrastructure (same as F9/F10 RuntimeDriver)
3. ✓ No patch to game required (external solution)
4. ✓ Works across game versions (no fragile patch needed)
5. ✓ Can be integrated into launch-game.ps1 or prove-features directly

### Implementation Path

1. Create `AutoAdvanceSplash.cs` in `src/Tools/Cli/Commands/ProveFeatures/`
2. Add P/Invoke stubs for `FindWindow`, `SetCursorPos`, `mouse_event`
3. Call `AutoAdvanceSplash.AdvanceSplashScreen()` in `/prove-features` after game launch
4. Add 3-second delay to allow game window to become active
5. Test with multiple game window sizes/resolutions
6. Document in `.claude/commands/prove-features.md`

### Integration into `launch-game.md`

Update the `.claude/commands/launch-game.md` command to include splash auto-advance:

```powershell
# File: .claude/commands/launch-game.md

1. Launch game exe
2. Wait 3 seconds for game window to appear
3. Call AutoAdvanceSplash.AdvanceSplashScreen() via MCP tool
4. Wait 3 seconds for main menu to load
5. Return "Game ready for testing" status
```

---

## Verification Checklist

- [ ] Win32 FindWindow correctly locates game window by title
- [ ] Mouse click is simulated at screen center
- [ ] Click advances splash screen to main menu
- [ ] Main menu appears within 10 seconds of auto-click
- [ ] `/prove-features` runs fully autonomous (no user input required)
- [ ] Feature tests (F9/F10, Mods button) complete successfully post-splash
- [ ] Gameplay entry succeeds (if scenario loaded)
- [ ] Video capture contains full feature test

---

## Timeline & Priority

| Phase | Task | Priority | Owner |
|-------|------|----------|-------|
| **Phase 1** | Implement AutoAdvanceSplash | High | CLI Agent |
| **Phase 2** | Integrate into /prove-features | High | QA Agent |
| **Phase 3** | Test with multiple resolutions | Medium | QA Agent |
| **Phase 4** | Document in launch-game.md | Low | Docs Agent |

---

## Related Documents

- **SPEC-runtime-features-baseline.md**: Feature 1 (F9/F10), Feature 2 (panels), Feature 3 (Mods button)
- **MEMORY.md**: "Agent testing" principle — never ask user to launch/interact
- **ADR-005**: No Harmony patches on external systems
- **launch-game.md**: Game launch command

---

## Ownership & Maintenance

**Component Owner**: Launcher / CLI Tools
**Test Owner**: QA Agent (/prove-features)
**Platform**: Windows 11 Pro (primary), Linux (secondary via WSL2)
**Last Updated**: 2026-03-24
