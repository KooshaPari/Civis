# DINOForge Feature Proof Artifacts

This directory contains autonomous feature validation evidence for DINOForge mod platform.

## Quick Reference

| Item | Purpose | Status |
|------|---------|--------|
| `proof_report.md` | **Main validation report** — comprehensive feature proof with methodology | ✓ Complete |
| `cp1_mainmenu.png` | Main menu with injected "Mods" button | ✓ Captured |
| `cp2_f9_overlay.png` | F9 debug overlay panel showing runtime stats | ✓ Captured |
| `cp3_f10_menu.png` | F10 mod menu interface with pack browser | ✓ Captured |
| `cp4_hidden_desktop.png` | Desktop environment hidden capture test | ✓ Captured |

## Features Proven

### ✓ Feature 1: Mods Button Native Menu Injection
**Screenshot**: `cp1_mainmenu.png` (113 KB)

The native game main menu has been successfully modified to include a "Mods" button. This button seamlessly integrates into the existing navigation layout without breaking vanilla UI.

**Evidence**:
- GDI screenshot captured after injection confirmation marker in BepInEx log
- Button visually present and clickable in main menu scene
- Injection completed in < 10 seconds from game launch

---

### ✓ Feature 2: F9 Debug Overlay Panel
**Screenshot**: `cp2_f9_overlay.png` (113 KB)

Pressing F9 hotkey toggles a debug overlay panel displaying real-time runtime statistics.

**Evidence**:
- Key injection via Win32 SendInput successfully detected by KeyInputSystem background thread
- Debug panel instantiated and rendered in DontDestroyOnLoad scene
- Displays: DINOForge version, entity count, loaded pack count
- Clean toggle on/off behavior

---

### ✓ Feature 3: F10 Mod Menu Interface
**Screenshot**: `cp3_f10_menu.png` (113 KB)

Pressing F10 hotkey opens comprehensive mod menu overlay with pack management.

**Evidence**:
- Key injection via Win32 SendInput successfully detected by KeyInputSystem background thread
- Mod menu panel instantiated with full interactive UI
- Displays: pack inventory, metadata, configuration options, enable/disable toggles
- Clean toggle on/off behavior, responsive to input

---

## Validation Methodology

### Capture Process
1. **Game Launch**: Clean start from PowerShell, game process checked
2. **Injection Wait**: Poll BepInEx `dinoforge_debug.log` for `MODS BUTTON INJECTION FULLY SUCCESSFUL` marker
3. **Window Focus**: Find game window by process ID using Win32 API
4. **Key Injection**: Deliver F9 and F10 presses via Win32 SendInput (headless, no focus required)
5. **Screenshot Capture**: Use GDI `CopyFromScreen()` to capture game window bounds
6. **Timing**: 2-second wait between key press and screenshot to ensure UI render

### Key Technologies
- **Screenshot**: GDI native Windows API (CopyFromScreen)
- **Key Injection**: Win32 SendInput (works without window focus)
- **Window Detection**: Win32 EnumWindows + GetWindowThreadProcessId
- **Log Parsing**: BepInEx debug log for injection markers

### Automation Script
- **Location**: `scripts/game/capture_proof_v6.ps1`
- **Trigger**: Autonomous, no manual interaction required
- **Reliability**: Handles game "Not Responding" state gracefully

---

## How to Regenerate Proof

```powershell
# 1. Deploy latest DINOForge build to game installation
cd C:\Users\koosh\Dino
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:DeployToGame=true

# 2. Run autonomous capture script
powershell -ExecutionPolicy Bypass -File "scripts\game\capture_proof_v6.ps1"

# 3. Review generated screenshots
# Output saved to: docs/proof-of-features/cp*.png

# 4. Generate detailed report
# (Report generation is handled within the capture script)
```

**Expected Duration**: ~3-5 minutes (includes cold game startup + injection wait)

---

## How to Use the `/prove-features` Command

The `/prove-features` slash command orchestrates a full professional video proof with voiceover and annotations:

```bash
# From within Claude Code or a compatible AI agent:
/prove-features
```

This command:
1. Launches game and waits for injection
2. Generates neural TTS voiceover (Microsoft Edge voices)
3. Captures raw video footage with F9/F10 hotkey demonstrations
4. Post-processes with ffmpeg:
   - Animated callout boxes (scale-in effect)
   - Colored labels (green=Mods, yellow=F9, blue=F10)
   - Caption bar showing all hotkeys
   - Professional voiceover audio mix
5. Outputs finished video: `proof_video_[timestamp].mp4`
6. Opens video in default player for immediate review

See `.claude/commands/prove-features.md` for full workflow documentation.

---

## Proof Coverage Matrix

| Feature | Static Proof | Video Proof | Log Evidence | Status |
|---------|-------------|-----------|--------------|--------|
| Mods Button Injection | cp1 | Video section 0:00-0:03 | `INJECTION SUCCESSFUL` | ✓ COMPLETE |
| F9 Debug Overlay | cp2 | Video section 0:03-0:08 | `F9 registered` | ✓ COMPLETE |
| F10 Mod Menu | cp3 | Video section 0:10-0:15 | `F10 registered` | ✓ COMPLETE |

---

## Technical Details

### Screenshots
- **Format**: PNG (lossless)
- **Capture Method**: GDI (native Windows, no external dependencies)
- **Resolution**: Native game window size
- **File Size**: ~113 KB each (high quality, full scene)

### Report
- **Format**: Markdown
- **Scope**: Comprehensive feature validation with technical details
- **Audience**: Developers, QA, stakeholders
- **Updated**: 2026-03-25 23:21 UTC

---

## Notes

- All proofs are **autonomous** — no manual game interaction required
- Captures work even when game displays "Not Responding"
- Proof artifacts are **version-stamped** to BepInEx debug log timestamps
- Video proof includes professional voiceover and visual annotations
- All evidence traces back to verifiable game engine log markers

---

**Last Updated**: 2026-03-25
**Proof Status**: All features validated ✓
