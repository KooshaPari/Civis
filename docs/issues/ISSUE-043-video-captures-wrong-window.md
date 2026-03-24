# ISSUE-043: /prove-features captures entire desktop instead of game window

**Status**: Open
**Priority**: High (blocks video quality)
**Complexity**: Medium
**Estimate**: 2 hours

---

## Problem Description

The `/prove-features` command uses `gdigrab -i desktop` to record video footage. This captures the **entire desktop**, including whatever application is in the foreground, rather than specifically targeting the DINO game window.

### Observed Behavior

- `ffmpeg -f gdigrab -i desktop ...` records from (0, 0) to (screen_width, screen_height)
- If user switches windows during recording, unrelated applications appear in the video
- If DINO window is not fullscreen, surrounding desktop/taskbar is visible
- Demo video quality is unprofessional: shows browser tabs, Discord, other windows

### Impact

- Proof-of-features videos contain clutter, distracting from feature demonstrations
- Recording must be babysitted: user cannot alt-tab away during 28s recording
- Multiple retakes often needed to get clean capture

---

## Root Cause

**gdigrab device** (FFmpeg DirectDraw screen capture):
- Default: captures entire virtual desktop
- No built-in window targeting
- Operator error: current spec uses `-i desktop` with no offset/size filtering

**Workaround exists** but not yet implemented:
- `gdigrab` supports `-offset_x`, `-offset_y`, `-video_size` parameters
- These allow targeting a specific rectangular region
- Combined with Win32 `GetWindowRect()`, can capture ONLY the game window

---

## Solution

### Use gdigrab offset + size parameters to target game window

**Win32 Window Detection**:
```powershell
Add-Type -AssemblyName System.Windows.Forms

# Get DINO window
$proc = Get-Process | Where-Object { $_.Name -like "*Diplomacy*" }
$hwnd = $proc.MainWindowHandle

# Get window bounds in screen coordinates
$rect = [System.Windows.Forms.Screen]::FromHandle($hwnd).Bounds

$offsetX = $rect.Left
$offsetY = $rect.Top
$videoWidth = $rect.Width
$videoHeight = $rect.Height
```

**Corrected ffmpeg command**:
```powershell
$ffmpeg = "C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe"

& $ffmpeg `
    -f gdigrab `
    -offset_x $offsetX `
    -offset_y $offsetY `
    -video_size "${videoWidth}x${videoHeight}" `
    -framerate 30 `
    -i desktop `
    -t 28 `
    -vcodec libx264 `
    -preset ultrafast `
    "$rawFile"
```

**Key parameters**:
- `-offset_x X`: Left edge of capture region (game window's left coordinate)
- `-offset_y Y`: Top edge of capture region (game window's top coordinate)
- `-video_size WxH`: Width × Height of capture region (game window's dimensions)
- `-i desktop`: Still uses desktop as source, but with clipping applied

### Result

- Only the DINO game window is recorded
- Desktop, taskbar, alt+tab interactions outside game are not visible
- Works regardless of game window position on screen
- User can freely interact with other applications without affecting video

---

## Acceptance Criteria

- [ ] **Targeted capture**: Video shows ONLY the DINO game window, no surrounding desktop
- [ ] **Position-independent**: Game window can be at any screen position; offset detection adjusts automatically
- [ ] **Multi-monitor safe**: Correctly handles game window on secondary monitor
- [ ] **Maximized window**: Works with fullscreen and windowed modes
- [ ] **No distortion**: Output video has correct aspect ratio (no stretching/cropping)
- [ ] **Error handling**: Graceful fallback if window detection fails (log warning, use legacy `-i desktop`)

---

## Implementation Tasks

### Task 1: Window Detection Utility

**File**: `src/Tools/GameControlCli/WindowUtils.ps1`

```powershell
function Get-GameWindowBounds {
    <#
    .SYNOPSIS
    Get DINO game window position and size in screen coordinates.

    .OUTPUTS
    [PSCustomObject] with properties: Left, Top, Width, Height
    #>
    param()

    try {
        Add-Type -AssemblyName System.Windows.Forms

        $proc = Get-Process | Where-Object { $_.Name -like "*Diplomacy*" } | Select-Object -First 1
        if (-not $proc) {
            throw "DINO process not found"
        }

        $hwnd = $proc.MainWindowHandle
        if (-not $hwnd) {
            throw "DINO window handle not valid"
        }

        $rect = [System.Windows.Forms.Screen]::FromHandle($hwnd).Bounds

        return [PSCustomObject]@{
            Left   = $rect.Left
            Top    = $rect.Top
            Width  = $rect.Width
            Height = $rect.Height
            Handle = $hwnd
        }
    }
    catch {
        Write-Warning "Failed to get window bounds: $_"
        return $null
    }
}

function Assert-WindowBounds {
    <#
    .SYNOPSIS
    Validate window bounds are sensible (non-negative, within screen bounds).
    #>
    param(
        [PSCustomObject]$Bounds
    )

    if (-not $Bounds) { return $false }
    if ($Bounds.Left -lt -1000) { return $false }    # off-screen left
    if ($Bounds.Top -lt -1000) { return $false }     # off-screen top
    if ($Bounds.Width -le 0 -or $Bounds.Width -gt 4000) { return $false }
    if ($Bounds.Height -le 0 -or $Bounds.Height -gt 4000) { return $false }

    return $true
}
```

### Task 2: Update Prove-Features Recording Logic

**File**: `src/Tools/GameControlCli/Prove-Features.ps1`

**Before**:
```powershell
$rawFile = "$tmpDir\raw.mp4"
$rec = Start-Process -FilePath $ffmpeg `
  -ArgumentList "-f gdigrab -framerate 30 -i desktop -t 28 -vcodec libx264 -preset ultrafast `"$rawFile`"" `
  -PassThru -WindowStyle Hidden
```

**After**:
```powershell
$bounds = Get-GameWindowBounds
if (-not (Assert-WindowBounds $bounds)) {
    Write-Warning "Window bounds invalid; falling back to full desktop capture"
    $ffmpegArgs = "-f gdigrab -framerate 30 -i desktop -t 28 -vcodec libx264 -preset ultrafast `"$rawFile`""
} else {
    $ffmpegArgs = @(
        "-f", "gdigrab",
        "-offset_x", $bounds.Left,
        "-offset_y", $bounds.Top,
        "-video_size", "$($bounds.Width)x$($bounds.Height)",
        "-framerate", "30",
        "-i", "desktop",
        "-t", "28",
        "-vcodec", "libx264",
        "-preset", "ultrafast",
        "`"$rawFile`""
    ) -join " "
}

$rec = Start-Process -FilePath $ffmpeg `
  -ArgumentList $ffmpegArgs `
  -PassThru -WindowStyle Hidden
```

### Task 3: Update SPEC Document

Update `SPEC-prove-features-video-pipeline.md` section 3.1 to include:
- Window detection code
- Offset parameters explanation
- Fallback behavior
- Multi-monitor considerations

### Task 4: Add Error Handling

Log window detection:
```powershell
Write-Host "Detected game window: $($bounds.Width)x$($bounds.Height) @ ($($bounds.Left),$($bounds.Top))"
```

Log fallback if detection fails:
```powershell
if ($bounds -eq $null) {
    Write-Warning "Window detection failed; using full desktop (includes taskbar/clutter)"
}
```

### Task 5: Testing

Create manual test:
1. Launch DINO in windowed mode (not fullscreen)
2. Run `dinoforge prove-features`
3. Move DINO window to different screen position mid-recording
4. Open browser/Discord in background
5. Verify output video contains ONLY the game window, no clutter

Create script test:
```powershell
# Test window detection
$bounds = Get-GameWindowBounds
Assert-WindowBounds $bounds | Should -Be $true
$bounds.Width | Should -Be 1920  # typical
$bounds.Height | Should -Be 1080
```

---

## Workarounds (Before Fix)

Until this is implemented:

1. **Full-screen the game**: Reduces visual clutter, makes window bounds == screen bounds
2. **Close other applications**: Run demo in isolation, minimizes distraction
3. **Manual cropping**: Post-process video with ffmpeg `crop` filter
   ```bash
   ffmpeg -i raw.mp4 -vf crop=1920:1080:0:0 cropped.mp4
   ```

---

## Related Issues

- ISSUE-044: /prove-features captions disappear mid-video (timing drift)
- ISSUE-045: SAPI voice quality unacceptable (addressed by ADR-007)

---

## References

- ffmpeg gdigrab docs: https://ffmpeg.org/ffmpeg-devices.html#gdigrab
- gdigrab offset examples: https://trac.ffmpeg.org/wiki/Capture/Desktop#Windows
- Win32 GetWindowRect: https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect

---

## Labels

`video-capture` `ffmpeg` `windows` `prove-features` `high-priority` `ux-improvement`
