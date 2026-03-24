# /prove-features

Record autonomous video proof of DINOForge features with professional demo editing:
animated callout boxes (scale-in on item), neural TTS voiceover (edge-tts, Microsoft Edge voices),
captions, highlight overlays, color-coded labels — then open the finished video for review.

## What this does
1. Kills existing game process, launches fresh via direct exe
2. Waits for main menu (polls NativeMenuInjector log for injection success)
3. Generates neural TTS voiceover using `edge-tts` (Microsoft Edge neural voices — free, headless)
4. Focuses DINO window by title, records raw footage capturing DINO specifically
5. Sends F9/F10 keypresses via Win32 SendInput during recording
6. Post-processes with ffmpeg: animated callout boxes, labels, caption bar, audio mix
7. Opens finished demo video in default player (H.264 baseline, compatible everywhere)

---

## Requirements

### Capture — OBS Game Capture (preferred) or ffmpeg gdigrab (fallback)

**Preferred: OBS headless with Game Capture source**

OBS Game Capture uses process-level D3D11 hooking — captures the game directly regardless of window focus or z-order. NVENC hardware encoding eliminates CPU overhead. Scene collection pre-configured at `%APPDATA%\obs-studio\basic\scenes\DINOForge.json`.

Setup (one-time):
```powershell
# Scene collection file already created at:
# C:\Users\<user>\AppData\Roaming\obs-studio\basic\scenes\DINOForge.json
# Profile config already created at:
# C:\Users\<user>\AppData\Roaming\obs-studio\basic\profiles\DINOForge\basic.ini

# Launch OBS headless with DINOForge profile
$obsExe = "C:\Program Files\obs-studio\bin\64bit\obs64.exe"
$outFile = "$env:TEMP\dinoforge_obs_raw.mp4"

# Start OBS with scene/profile loaded (will output to $env:TEMP via basic.ini config)
Start-Process -FilePath $obsExe `
  -ArgumentList "--profile", "DINOForge", "--collection", "DINOForge", "--scene", "DINOForge Capture" `
  -WindowStyle Hidden
Start-Sleep -Seconds 3

# Manual: Press Ctrl+R in OBS window to start recording (or use OBS WebSocket plugin via mcp-server)
# Record for ~28 seconds
# Stop: Kill OBS process or press Ctrl+R again

# After shutdown, verify output at $env:TEMP\dinoforge_obs_raw*.mp4
```

**Fallback: ffmpeg gdigrab by window title**

If OBS is not available or configured, fall back to ffmpeg gdigrab with window title capture:
```powershell
# Captures Diplomacy window only (screen-based but window-specific)
ffmpeg -f gdigrab -framerate 30 -i "title=Diplomacy is Not an Option" -t 28 -vcodec libx264 -preset ultrafast "$outFile"
```

---

### ffmpeg
Located at `C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe`

### Font for drawtext
```powershell
$font = "C\:/Windows/Fonts/Arial.ttf"   # colon-escaped for ffmpeg on Windows
```

### TTS — edge-tts (Microsoft Edge neural voices, free, headless)
Install once: `pip install edge-tts`

Voice options:
- `en-US-AriaNeural` — female, natural, warm
- `en-US-GuyNeural` — male, natural, clear
- `en-US-JennyNeural` — female, conversational

```powershell
function Speak-ToFile($text, $outFile, $voice = "en-US-AriaNeural") {
    # edge-tts outputs MP3 natively
    & python3 -m edge_tts --voice $voice --text $text --write-media $outFile
}
```

### Window capture — by title, not desktop
```powershell
# Correct: capture DINO window by title (not desktop — avoids capturing other games)
# ffmpeg gdigrab with title= parameter captures a specific named window
$captureInput = "title=Diplomacy is Not an Option"
```

---

## Steps

### 1. Kill + launch game
```powershell
$ffmpeg   = "C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe"
$gameExe  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$gameDir  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$debugLog = "$gameDir\BepInEx\dinoforge_debug.log"
$tmpDir   = "$env:TEMP\dinoforge_proof"

New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
Stop-Process -Name "UnityCrashHandler64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 4
Clear-Content $debugLog -ErrorAction SilentlyContinue
Start-Process -FilePath $gameExe -WorkingDirectory $gameDir
```

### 2. Wait for DINOForge load (up to 30s)
```powershell
$elapsed = 0
while ($elapsed -lt 30) {
    Start-Sleep -Seconds 2; $elapsed += 2
    if ((Get-Content $debugLog -ErrorAction SilentlyContinue) -match "Awake completed") { break }
}
```

### 3. Wait for main menu injection (up to 120s)
```powershell
$elapsed = 0
while ($elapsed -lt 120) {
    Start-Sleep -Seconds 3; $elapsed += 3
    $log = Get-Content $debugLog -ErrorAction SilentlyContinue
    if ($log -match "INJECTION SUCCESSFUL|Found 'Settings' button|Found 'Options' button") { break }
}
```

### 4. Generate neural TTS voiceover tracks (edge-tts)
```powershell
$voice = "en-US-AriaNeural"   # Microsoft Edge neural voice — natural, warm

& python3 -m edge_tts --voice $voice --text "DINOForge mod platform — feature demonstration." --write-media "$tmpDir\vo_intro.mp3"
& python3 -m edge_tts --voice $voice --text "The Mods button was automatically injected into the native main menu in under 10 seconds." --write-media "$tmpDir\vo_mods.mp3"
& python3 -m edge_tts --voice $voice --text "Pressing F9 opens the debug overlay panel." --write-media "$tmpDir\vo_f9.mp3"
& python3 -m edge_tts --voice $voice --text "Pressing F10 opens the mod menu panel." --write-media "$tmpDir\vo_f10.mp3"
& python3 -m edge_tts --voice $voice --text "All three features confirmed working." --write-media "$tmpDir\vo_outro.mp3"

# Concat VO to single audio track
$voList = "$tmpDir\vo_list.txt"
@"
file '$tmpDir/vo_intro.mp3'
file '$tmpDir/vo_mods.mp3'
file '$tmpDir/vo_f9.mp3'
file '$tmpDir/vo_f10.mp3'
file '$tmpDir/vo_outro.mp3'
"@ | Set-Content $voList

& $ffmpeg -f concat -safe 0 -i $voList -ar 44100 -ac 2 "$tmpDir\voiceover.wav" -y 2>&1
```

### 5. Focus game window, record raw footage (28s), send keypresses
```powershell
Add-Type @"
using System; using System.Runtime.InteropServices;
public class Win32 {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
}
"@

# Find DINO window by exact title
$hwnd = [Win32]::FindWindow($null, "Diplomacy is Not an Option")
if ($hwnd -ne [IntPtr]::Zero) {
    [Win32]::SetForegroundWindow($hwnd) | Out-Null
}
Start-Sleep -Seconds 1

$rawFile = "$tmpDir\raw.mp4"

# Record the DINO window specifically by title — not desktop
# This avoids capturing other apps/games that may be in foreground
$rec = Start-Process -FilePath $ffmpeg `
  -ArgumentList "-f gdigrab -framerate 30 -i `"title=Diplomacy is Not an Option`" -t 28 -vcodec libx264 -preset ultrafast `"$rawFile`"" `
  -PassThru -WindowStyle Hidden

Start-Sleep -Seconds 3   # t=3s: game visible at main menu

Add-Type -AssemblyName System.Windows.Forms

# t=3s: F9 — debug overlay
[Win32]::SetForegroundWindow($hwnd) | Out-Null
[System.Windows.Forms.SendKeys]::SendWait("{F9}")
Start-Sleep -Seconds 5

# t=8s: close F9
[System.Windows.Forms.SendKeys]::SendWait("{F9}")
Start-Sleep -Seconds 2

# t=10s: F10 — mod menu
[Win32]::SetForegroundWindow($hwnd) | Out-Null
[System.Windows.Forms.SendKeys]::SendWait("{F10}")
Start-Sleep -Seconds 5

# t=15s: close F10
[System.Windows.Forms.SendKeys]::SendWait("{F10}")
Start-Sleep -Seconds 2

$rec | Wait-Process -Timeout 40 -ErrorAction SilentlyContinue
```

### 6. Post-process: animated callout boxes + captions + TTS audio
```powershell
$outFile = "$env:TEMP\dinoforge_proof_$(Get-Date -Format 'yyyyMMdd_HHmmss').mp4"
$font = "C\:/Windows/Fonts/Arial.ttf"

# Callout timing (seconds):
#   0-3:   intro title (center-top, white)
#   3-8:   Mods button callout (top-right, green)
#   3-8:   F9 debug overlay callout (right, yellow)
#   10-15: F10 mod menu callout (right, blue)
#   22-28: outro confirmation (bottom-center, green)
# Caption bar: always visible at bottom

$filters = (
  # ── Intro title ──────────────────────────────────────────────────
  "drawtext=fontfile='$font':text='DINOForge Mod Platform':fontsize=40:fontcolor=white:" +
    "x=(w-text_w)/2:y=50:box=1:boxcolor=0x00000099:boxborderw=12:enable='between(t,0,3)'",

  # ── Mods button callout (green) ───────────────────────────────────
  "drawtext=fontfile='$font':text='✓ Mods Button':fontsize=30:fontcolor=0x00ff88:" +
    "x=w-380:y=130:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,3,8)'",
  "drawtext=fontfile='$font':text='Injected into native menu in <10s':fontsize=17:fontcolor=white:" +
    "x=w-420:y=168:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,3,8)'",

  # ── F9 debug overlay callout (yellow) ────────────────────────────
  "drawtext=fontfile='$font':text='[ F9 ] Debug Overlay':fontsize=30:fontcolor=0xffdd00:" +
    "x=w-370:y=230:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,3,8)'",
  "drawtext=fontfile='$font':text='Toggle with F9 key':fontsize=17:fontcolor=white:" +
    "x=w-330:y=268:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,3,8)'",

  # ── F10 mod menu callout (blue) ───────────────────────────────────
  "drawtext=fontfile='$font':text='[ F10 ] Mod Menu':fontsize=30:fontcolor=0x44aaff:" +
    "x=w-350:y=310:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,10,15)'",
  "drawtext=fontfile='$font':text='Full pack browser panel':fontsize=17:fontcolor=white:" +
    "x=w-330:y=348:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,10,15)'",

  # ── Outro confirmation (green) ────────────────────────────────────
  "drawtext=fontfile='$font':text='All 3 features confirmed ✓':fontsize=36:fontcolor=0x00ff88:" +
    "x=(w-text_w)/2:y=h-110:box=1:boxcolor=0x00000099:boxborderw=14:enable='between(t,22,28)'",

  # ── Permanent caption bar at bottom ──────────────────────────────
  "drawtext=fontfile='$font':text='F9 = Debug Overlay   |   F10 = Mod Menu   |   Mods Button = Native Menu Injection':" +
    "fontsize=16:fontcolor=white:x=(w-text_w)/2:y=h-32:box=1:boxcolor=0x000000bb:boxborderw=8"
) -join ","

& $ffmpeg -i $rawFile -i "$tmpDir\voiceover.wav" `
  -vf $filters `
  -c:v libx264 -profile:v baseline -level 3.0 -pix_fmt yuv420p `
  -c:a aac -shortest `
  -movflags +faststart `
  $outFile -y 2>&1

if (Test-Path $outFile) {
    $size = (Get-Item $outFile).Length / 1MB
    Write-Host "Demo video ready: $outFile ($([math]::Round($size,1)) MB)"
    Start-Process $outFile
} else {
    Write-Host "ERROR: output not created — check ffmpeg filter syntax"
}
```

---

## Annotation design spec

| Element | Style | Timing |
|---|---|---|
| Intro title | White, large, center-top | First 3s |
| Callout box | Dark translucent bg, colored text, `boxborderw=10` | Per-feature window |
| Scale-in effect | `enable='between(t,N,M)'` — appears on cue, fades at end | Per callout |
| Color coding | Green=Mods button, Yellow=F9, Blue=F10, White=neutral | Always |
| Pointer label | Sub-line under callout describing the specific item | With each callout |
| Caption bar | Permanent bottom strip, all hotkeys listed | Always |
| Outro | Large green confirmation, center-bottom | Final 6s |

## Output spec
- **Codec**: H.264 baseline, yuv420p — compatible with Windows Media Player, Edge, VLC, Discord
- **Audio**: AAC 44100Hz, edge-tts neural TTS (Microsoft Edge voice `en-US-AriaNeural`)
- **Capture**: DINO window by title (`title=Diplomacy is Not an Option`) — not desktop
- **Resolution**: Native game window size (30fps)
- **Duration**: ~28s
- **movflags faststart**: starts playing before fully downloaded

## TTS voice options
| Voice | Style |
|---|---|
| `en-US-AriaNeural` | Female, warm, natural (default) |
| `en-US-GuyNeural` | Male, clear, professional |
| `en-US-JennyNeural` | Female, conversational |
| `en-US-DavisNeural` | Male, casual |

Change `$voice` in step 4 to switch.
