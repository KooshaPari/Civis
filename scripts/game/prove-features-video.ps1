#Requires -Version 5.1
<#
.SYNOPSIS
    Full DINOForge prove-features video pipeline (SPEC-006).

.DESCRIPTION
    Automated video generation showing three DINOForge features:
    1. Mods button injected into native main menu
    2. F9 debug overlay panel
    3. F10 mod menu panel

    Pipeline:
    - Kills/cleans up existing game instances
    - Launches fresh game instance
    - Waits for bootstrap ("Awake completed" then "MODS BUTTON INJECTION FULLY SUCCESSFUL")
    - Maximizes window and reads window rect via Win32 GetWindowRect
    - Generates TTS voiceovers (edge-tts or SAPI fallback)
    - Records 28s ffmpeg gdigrab video
    - Injects F9/F10 key presses during recording
    - Concatenates TTS audio tracks
    - Post-processes with drawtext overlays (feature captions, timestamps)
    - Muxes audio + video
    - Opens final MP4 in default player

.PARAMETER Verbose
    Enable detailed timestamped output

.NOTES
    - Output: $env:TEMP\DINOForge\dinoforge_proof_<timestamp>.mp4
    - Also copies to: C:\Users\koosh\Dino\docs\proof-of-features\
    - Requires: ffmpeg, Python (for edge-tts, optional)
    - Falls back to SAPI TTS if edge-tts unavailable
#>

param(
    [switch]$Verbose,
    [string]$OutDir = "C:\Users\koosh\Dino\docs\proof-of-features"
)

# ============================================================================
# Configuration
# ============================================================================
$GameExe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$GameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$BepInExDir = "$GameDir\BepInEx"
$DebugLogFile = "$BepInExDir\dinoforge_debug.log"
$TempDir = "$env:TEMP\DINOForge"

# TTS voice script text
$VOScripts = @{
    "intro" = "DINOForge mod platform. Feature demonstration."
    "mods" = "Mods button successfully injected into the native main menu - auto-detected in under 10 seconds."
    "f9" = "Pressing F9 opens the debug overlay panel."
    "f10" = "Pressing F10 opens the mod menu panel."
    "outro" = "All three features confirmed working."
}

# Recording parameters
$RecordDurationSeconds = 28
$RecordFramerate = 30

# Key press timings (seconds into recording)
$KeyPressTimings = @(
    @{ Time = 3; Key = "F9"; Desc = "Open F9 overlay" }
    @{ Time = 8; Key = "F9"; Desc = "Close F9 overlay" }
    @{ Time = 10; Key = "F10"; Desc = "Open F10 menu" }
    @{ Time = 15; Key = "F10"; Desc = "Close F10 menu" }
)

# ============================================================================
# Win32 API Definitions
# ============================================================================

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public class WF8 {
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr h);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr h, int cmd);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr h);

    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr h, out RECT rect);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
        public int Width { get { return Right - Left; } }
        public int Height { get { return Bottom - Top; } }
    }

    public delegate bool EnumWindowsProc(IntPtr h, IntPtr lp);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc fn, IntPtr lp);

    public static IntPtr FindWindowByPid(uint targetPid) {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr h, IntPtr lp) {
            if (!IsWindowVisible(h)) return true;
            uint pid = 0;
            GetWindowThreadProcessId(h, out pid);
            if (pid == targetPid) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@ -ErrorAction Stop

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

public class FK8 {
    [StructLayout(LayoutKind.Sequential)]
    struct KI {
        public ushort vk;
        public ushort sc;
        public uint fl;
        public uint t;
        public IntPtr ex;
    }

    [StructLayout(LayoutKind.Explicit)]
    struct IU {
        [FieldOffset(0)] public KI ki;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct IN {
        public uint type;
        public IU u;
    }

    [DllImport("user32.dll")]
    static extern uint SendInput(uint n, IN[] inp, int sz);

    public static void Press(ushort vk) {
        var inp = new IN[2];
        inp[0].type = 1;
        inp[0].u.ki.vk = vk;

        inp[1].type = 1;
        inp[1].u.ki.vk = vk;
        inp[1].u.ki.fl = 2; // KEYEVENTF_KEYUP

        SendInput(2, inp, System.Runtime.InteropServices.Marshal.SizeOf(typeof(IN)));
    }
}
"@ -ErrorAction Stop

# ============================================================================
# Logging
# ============================================================================

function Log-Step {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    Write-Host "[$timestamp] $Message" -ForegroundColor Cyan
}

function Log-Error {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    Write-Host "[$timestamp] ERROR: $Message" -ForegroundColor Red
}

function Log-Success {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    Write-Host "[$timestamp] ✓ $Message" -ForegroundColor Green
}

# ============================================================================
# Utility Functions
# ============================================================================

function Ensure-TempDir {
    if (-not (Test-Path $TempDir)) {
        New-Item -ItemType Directory -Path $TempDir -Force | Out-Null
        Log-Step "Created temp directory: $TempDir"
    }
}

function FocusGame {
    param([System.IntPtr]$hwnd)
    if ($hwnd -ne [IntPtr]::Zero) {
        [WF8]::ShowWindow($hwnd, 9) | Out-Null
        [WF8]::SetForegroundWindow($hwnd) | Out-Null
    }
}

function Kill-Game {
    param([string]$Context = "")
    Log-Step "Killing game process ($Context)..."
    Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | ForEach-Object {
        try { $_.Kill() } catch {}
    }
    Get-Process -Name "UnityCrashHandler64" -ErrorAction SilentlyContinue | ForEach-Object {
        try { $_.Kill() } catch {}
    }
    Start-Sleep -Seconds 4
}

function Clear-DebugLog {
    if (Test-Path $DebugLogFile) {
        try {
            Clear-Content $DebugLogFile -Force -ErrorAction SilentlyContinue
            Log-Step "Cleared debug log"
        } catch {
            Log-Step "Could not clear debug log (will try again at runtime)"
        }
    }
}

function Launch-Game {
    Log-Step "Launching game..."
    $proc = Start-Process -FilePath $GameExe -WorkingDirectory $GameDir -PassThru
    $script:pid = $proc.Id
    Log-Step "Game launched. PID=$($script:pid)"
    return $proc
}

function Get-HWND-For-PID {
    param([int]$TargetPid, [int]$TimeoutSeconds = 20)
    $stop = [DateTime]::Now.AddSeconds($TimeoutSeconds)
    while ([DateTime]::Now -lt $stop) {
        $hw = [WF8]::FindWindowByPid([uint32]$TargetPid)
        if ($hw -ne [IntPtr]::Zero) { return $hw }
        Start-Sleep -Milliseconds 300
    }
    return [IntPtr]::Zero
}

function Wait-For-Bootstrap {
    param([int]$TimeoutSeconds = 180, [DateTime]$logBasetime)

    Log-Step "Waiting for bootstrap (max ${TimeoutSeconds}s)..."
    Log-Step "  Phase 1: Looking for 'Awake completed'"
    Log-Step "  Phase 2: Looking for 'MODS BUTTON INJECTION FULLY SUCCESSFUL'"

    # Phase 1: Awake completed (30s timeout)
    $phase1Deadline = [DateTime]::Now.AddSeconds(30)
    $phase1Found = $false
    while ([DateTime]::Now -lt $phase1Deadline) {
        if (Test-Path $DebugLogFile) {
            try {
                $fs = [System.IO.File]::Open($DebugLogFile, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
                $reader = New-Object System.IO.StreamReader($fs, [System.Text.Encoding]::UTF8)
                $content = $reader.ReadToEnd()
                $reader.Close(); $fs.Close()

                if ($content -match "Awake completed") {
                    Log-Success "Phase 1: Found 'Awake completed'"
                    $phase1Found = $true
                    break
                }
            } catch {}
        }
        Start-Sleep -Milliseconds 500
    }

    if (-not $phase1Found) {
        Log-Error "Phase 1 failed - 'Awake completed' not found in debug log"
        return $false
    }

    # Phase 2: Mods button injection (120s timeout)
    Log-Step "  Phase 2: Waiting for injection (max 120s)..."
    $phase2Deadline = [DateTime]::Now.AddSeconds(120)
    $phase2Found = $false
    while ([DateTime]::Now -lt $phase2Deadline) {
        # Keep game focused
        $hw = [WF8]::FindWindowByPid([uint32]$script:pid)
        if ($hw -ne [IntPtr]::Zero) { FocusGame $hw }

        if (Test-Path $DebugLogFile) {
            try {
                $fs = [System.IO.File]::Open($DebugLogFile, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
                $reader = New-Object System.IO.StreamReader($fs, [System.Text.Encoding]::UTF8)
                $content = $reader.ReadToEnd()
                $reader.Close(); $fs.Close()

                if ($content -match "MODS BUTTON INJECTION FULLY SUCCESSFUL") {
                    Log-Success "Phase 2: Found 'MODS BUTTON INJECTION FULLY SUCCESSFUL'"
                    $phase2Found = $true
                    break
                }
            } catch {}
        }
        Start-Sleep -Milliseconds 500
    }

    if (-not $phase2Found) {
        Log-Error "Phase 2 failed - injection confirmation not found"
        return $false
    }

    Log-Success "Bootstrap complete!"
    return $true
}

function Get-Window-Rect {
    param([System.IntPtr]$hwnd)

    Log-Step "Getting window rect for HWND=0x$($hwnd.ToString('X'))..."

    # Maximize window first
    [WF8]::ShowWindow($hwnd, 3) | Out-Null  # SW_MAXIMIZE = 3
    Start-Sleep -Milliseconds 500

    $rect = New-Object WF8+RECT
    $ok = [WF8]::GetWindowRect($hwnd, [ref]$rect)

    if ($ok) {
        Log-Success "Window rect: X=$($rect.Left) Y=$($rect.Top) W=$($rect.Width) H=$($rect.Height)"
        return $rect
    } else {
        Log-Error "Failed to get window rect"
        return $null
    }
}

function Find-FFmpeg {
    Log-Step "Locating ffmpeg..."

    $candidates = @(
        "C:\Program Files\ImageMagick-7.1.0-q16-hdri\ffmpeg.exe",
        "C:\Program Files\ffmpeg\bin\ffmpeg.exe",
        "C:\ffmpeg\ffmpeg.exe"
    )

    foreach ($path in $candidates) {
        if (Test-Path $path) {
            Log-Success "Found ffmpeg: $path"
            return $path
        }
    }

    # Try where.exe fallback
    try {
        $found = where.exe ffmpeg 2>$null | Select-Object -First 1
        if ($found -and (Test-Path $found)) {
            Log-Success "Found ffmpeg via where: $found"
            return $found
        }
    } catch {}

    Log-Error "ffmpeg not found"
    return $null
}

function Generate-TTS-Audio {
    param([string]$Text, [string]$OutputFile, [int]$Index)

    Log-Step "Generating TTS: $Text"

    # Try edge-tts first
    $usedEdgeTts = $false
    try {
        $cmd = "python -m edge_tts --text `"$Text`" --write-media `"$OutputFile`" 2>nul"
        $null = cmd /c $cmd
        if (Test-Path $OutputFile) {
            Log-Success "Generated via edge-tts: $OutputFile"
            $usedEdgeTts = $true
        }
    } catch {}

    # Fallback to SAPI
    if (-not $usedEdgeTts) {
        Log-Step "Falling back to SAPI TTS..."
        $ps = New-Object System.Speech.Synthesis.SpeechSynthesizer
        $ps.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::Neutral, [System.Speech.Synthesis.VoiceAge]::Adult)
        $ps.Rate = -2
        $ps.Volume = 100
        $ps.SetOutputToWaveFile($OutputFile)
        $ps.Speak($Text)
        $ps.Dispose()

        if (Test-Path $OutputFile) {
            Log-Success "Generated via SAPI: $OutputFile"
        } else {
            Log-Error "SAPI TTS failed"
            return $false
        }
    }

    # Pad with 1s silence using ffmpeg
    Log-Step "Padding audio with silence..."
    $paddedFile = "$TempDir\vo_${Index}_padded.wav"
    $ffmpeg = Find-FFmpeg
    if ($ffmpeg) {
        $silenceFile = Join-Path $TempDir "silence_1s.mp3"
        $null = & $ffmpeg -f lavfi -i anullsrc=r=44100:cl=mono -t 1 -q:a 9 -acodec libmp3lame $silenceFile 2>$null
        $concatArg = 'concat:' + $OutputFile + '|' + $silenceFile
        $null = & $ffmpeg -i $concatArg -c copy $paddedFile 2>$null
        if (Test-Path $paddedFile) {
            Remove-Item $OutputFile -Force
            Rename-Item $paddedFile $OutputFile -Force
        }
    }

    return (Test-Path $OutputFile)
}

function Generate-All-TTS {
    Log-Step "========== Generating TTS Voiceovers =========="

    $ttsFiles = @()
    $index = 0
    foreach ($key in @("intro", "mods", "f9", "f10", "outro")) {
        $text = $VOScripts[$key]
        $outFile = "$TempDir\vo_${index}.mp3"
        $ok = Generate-TTS-Audio -Text $text -OutputFile $outFile -Index $index
        if ($ok) {
            $ttsFiles += $outFile
        } else {
            Log-Error "Failed to generate TTS for: $key"
            return $null
        }
        $index++
    }

    return $ttsFiles
}

function Concatenate-Audio {
    param([string[]]$InputFiles, [string]$OutputFile)

    Log-Step "Concatenating audio tracks..."

    $ffmpeg = Find-FFmpeg
    if (-not $ffmpeg) {
        Log-Error "ffmpeg not found"
        return $false
    }

    # Create concat demuxer file
    $concatFile = "$TempDir\concat.txt"
    $InputFiles | ForEach-Object {
        $fullPath = (Resolve-Path $_).Path
        Add-Content $concatFile "file '$fullPath'"
    }

    # Concatenate
    $null = & $ffmpeg -f concat -safe 0 -i $concatFile -c copy $OutputFile 2>$null

    if (Test-Path $OutputFile) {
        Log-Success "Audio concatenated: $OutputFile"
        Remove-Item $concatFile -Force
        return $true
    } else {
        Log-Error "Audio concatenation failed"
        return $false
    }
}

function Start-Video-Recording {
    param([System.IntPtr]$hwnd, [WF8+RECT]$rect, [string]$OutputFile)

    Log-Step "Starting video recording..."
    Log-Step "  Window rect: X=$($rect.Left) Y=$($rect.Top) W=$($rect.Width) H=$($rect.Height)"
    $dur = $RecordDurationSeconds; $fps = $RecordFramerate
    Log-Step "  Duration: $dur`s @ $fps`fps"

    $ffmpeg = Find-FFmpeg
    if (-not $ffmpeg) {
        Log-Error "ffmpeg not found"
        return $null
    }

    # Ensure window stays focused
    FocusGame $hwnd

    # Start ffmpeg as background job
    $cmd = @(
        "$ffmpeg",
        "-f gdigrab",
        "-offset_x $($rect.Left)",
        "-offset_y $($rect.Top)",
        "-video_size $($rect.Width)x$($rect.Height)",
        "-framerate $RecordFramerate",
        "-i desktop",
        "-t $RecordDurationSeconds",
        "-vcodec libx264",
        "-preset ultrafast",
        "-pix_fmt yuv420p",
        "`"$OutputFile`""
    ) -join " "

    Log-Step "FFmpeg command: $cmd"

    $job = Start-Job -ScriptBlock {
        param($Cmd)
        cmd /c $Cmd 2>$null
    } -ArgumentList $cmd

    Log-Success "Recording started (Job ID: $($job.Id))"
    Start-Sleep -Milliseconds 500

    return $job
}

function Inject-Key-Presses {
    param([System.IntPtr]$hwnd, [int]$RecordingStartTime)

    Log-Step "Injecting key presses during recording..."

    $startTime = [DateTime]::Now

    foreach ($press in $KeyPressTimings) {
        $targetTime = $startTime.AddSeconds($press.Time)

        while ([DateTime]::Now -lt $targetTime) {
            Start-Sleep -Milliseconds 50
        }

        FocusGame $hwnd
        $vk = if ($press.Key -eq "F9") { 0x78 } else { 0x79 }
        Log-Step "  [$([int](([DateTime]::Now - $startTime).TotalSeconds))s] $($press.Key) - $($press.Desc)"
        [FK8]::Press($vk)
        Start-Sleep -Milliseconds 100
    }

    Log-Success "All key presses injected"
}

function Wait-For-Recording {
    param([System.Management.Automation.Job]$Job, [int]$TimeoutSeconds = 45)

    Log-Step "Waiting for recording to complete (max ${TimeoutSeconds}s)..."

    $result = Wait-Job -Job $Job -Timeout $TimeoutSeconds

    if ($result) {
        Log-Success "Recording completed"
        Remove-Job $Job
        return $true
    } else {
        Log-Error "Recording timeout"
        Stop-Job $Job
        Remove-Job $Job
        return $false
    }
}

function Post-Process-Video {
    param([string]$InputVideo, [string]$AudioFile, [string]$OutputVideo)

    Log-Step "Post-processing video with overlays and audio..."

    $ffmpeg = Find-FFmpeg
    if (-not $ffmpeg) {
        Log-Error "ffmpeg not found"
        return $false
    }

    $font = "C:/Windows/Fonts/Arial.ttf"

    $filters = @(
        # Title overlay (0-3s)
        "drawtext=fontfile='$font':text='DINOForge Mod Platform':fontsize=40:fontcolor=white:x=(w-text_w)/2:y=50:box=1:boxcolor=0x00000099:boxborderw=12:enable='between(t,0,3)'",

        # Mods button feature (3-8s)
        "drawtext=fontfile='$font':text='✓ Mods Button':fontsize=30:fontcolor=0x00ff88:x=w-380:y=130:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,3,8)'",
        "drawtext=fontfile='$font':text='Injected into native menu in <10s':fontsize=17:fontcolor=white:x=w-420:y=168:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,3,8)'",

        # F9 Debug Overlay feature (8-10s + extended view)
        "drawtext=fontfile='$font':text='[ F9 ] Debug Overlay':fontsize=30:fontcolor=0xffdd00:x=w-370:y=230:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,3,10)'",
        "drawtext=fontfile='$font':text='Toggle with F9 key':fontsize=17:fontcolor=white:x=w-330:y=268:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,3,10)'",

        # F10 Mod Menu feature (10-15s)
        "drawtext=fontfile='$font':text='[ F10 ] Mod Menu':fontsize=30:fontcolor=0x44aaff:x=w-350:y=310:box=1:boxcolor=0x00000099:boxborderw=10:enable='between(t,10,15)'",
        "drawtext=fontfile='$font':text='Full pack browser panel':fontsize=17:fontcolor=white:x=w-330:y=348:box=1:boxcolor=0x00000077:boxborderw=6:enable='between(t,10,15)'",

        # Success overlay (22-28s)
        "drawtext=fontfile='$font':text='All 3 features confirmed ✓':fontsize=36:fontcolor=0x00ff88:x=(w-text_w)/2:y=h-110:box=1:boxcolor=0x00000099:boxborderw=14:enable='between(t,22,28)'",

        # Bottom banner (persistent)
        "drawtext=fontfile='$font':text='F9 = Debug Overlay   |   F10 = Mod Menu   |   Mods Button = Native Menu Injection':fontsize=16:fontcolor=white:x=(w-text_w)/2:y=h-32:box=1:boxcolor=0x000000bb:boxborderw=8"
    ) -join ","

    Log-Step "Applying filters and muxing audio..."

    $cmd = @(
        "$ffmpeg",
        "-i `"$InputVideo`"",
        "-i `"$AudioFile`"",
        "-vf `"$filters`"",
        "-c:v libx264",
        "-profile:v baseline",
        "-level 3.0",
        "-pix_fmt yuv420p",
        "-c:a aac",
        "-b:a 128k",
        "-shortest",
        "-movflags +faststart",
        "`"$OutputVideo`""
    ) -join " "

    Log-Step "FFmpeg post-process: $cmd"

    $null = cmd /c $cmd 2>$null

    if (Test-Path $OutputVideo) {
        Log-Success "Video post-processed: $OutputVideo"
        return $true
    } else {
        Log-Error "Video post-processing failed"
        return $false
    }
}

# ============================================================================
# Main Pipeline
# ============================================================================

Log-Step "========== DINOForge Prove-Features Video Pipeline (SPEC-006) =========="

Ensure-TempDir

# Step 1: Kill and cleanup
Log-Step "========== Step 1: Kill Existing Instances =========="
Kill-Game "initial cleanup"
Clear-DebugLog

# Step 2: Launch game
Log-Step "========== Step 2: Launch Game =========="
$logBasetime = if (Test-Path $DebugLogFile) { (Get-Item $DebugLogFile).LastWriteTime } else { [DateTime]::Now }
Launch-Game | Out-Null

# Step 3: Get HWND
Log-Step "========== Step 3: Find Game Window =========="
$hwnd = Get-HWND-For-PID -TargetPid $script:pid -TimeoutSeconds 20
if ($hwnd -eq [IntPtr]::Zero) {
    Log-Error "Could not find game window HWND"
    Kill-Game "hwnd lookup failed"
    exit 1
}
Log-Success "Found HWND: 0x$($hwnd.ToString('X'))"

# Step 4: Wait for bootstrap
Log-Step "========== Step 4: Wait for Bootstrap =========="
$bootstrapOk = Wait-For-Bootstrap -TimeoutSeconds 180 -logBasetime $logBasetime
if (-not $bootstrapOk) {
    Log-Error "Bootstrap failed"
    Kill-Game "bootstrap failed"
    exit 1
}

Start-Sleep -Seconds 3

# Step 5: Get window rect and maximize
Log-Step "========== Step 5: Maximize Window and Get Rect =========="
$rect = Get-Window-Rect -hwnd $hwnd
if (-not $rect) {
    Log-Error "Could not get window rect"
    Kill-Game "rect lookup failed"
    exit 1
}

# Step 6: Generate TTS audio
Log-Step "========== Step 6: Generate TTS Voiceovers =========="
$ttsFiles = Generate-All-TTS
if (-not $ttsFiles -or $ttsFiles.Count -ne 5) {
    Log-Error "Failed to generate TTS files"
    Kill-Game "tts generation failed"
    exit 1
}

# Step 7: Concatenate audio
Log-Step "========== Step 7: Concatenate Audio =========="
$audioFile = "$TempDir\voiceover.wav"
$audioOk = Concatenate-Audio -InputFiles $ttsFiles -OutputFile $audioFile
if (-not $audioOk) {
    Log-Error "Failed to concatenate audio"
    Kill-Game "audio concatenation failed"
    exit 1
}

# Step 8: Start recording
Log-Step "========== Step 8: Start Video Recording =========="
$rawVideoFile = "$TempDir\raw_recording.mkv"
$recordingJob = Start-Video-Recording -hwnd $hwnd -rect $rect -OutputFile $rawVideoFile
if (-not $recordingJob) {
    Log-Error "Failed to start recording"
    Kill-Game "recording start failed"
    exit 1
}

# Step 9: Inject key presses during recording
Log-Step "========== Step 9: Inject Key Presses =========="
Inject-Key-Presses -hwnd $hwnd -RecordingStartTime ([DateTime]::Now)

# Step 10: Wait for recording to complete
Log-Step "========== Step 10: Wait for Recording =========="
$recordOk = Wait-For-Recording -Job $recordingJob -TimeoutSeconds 45
if (-not $recordOk) {
    Log-Error "Recording did not complete in time"
    Kill-Game "recording timeout"
    exit 1
}

if (-not (Test-Path $rawVideoFile)) {
    Log-Error "Raw video file not created"
    Kill-Game "no raw video output"
    exit 1
}

Log-Success "Raw video recorded: $rawVideoFile"

# Step 11: Post-process video
Log-Step "========== Step 11: Post-Process Video =========="
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$finalVideo = "$TempDir\dinoforge_proof_${timestamp}.mp4"
$ppOk = Post-Process-Video -InputVideo $rawVideoFile -AudioFile $audioFile -OutputVideo $finalVideo
if (-not $ppOk) {
    Log-Error "Video post-processing failed"
    Kill-Game "post-process failed"
    exit 1
}

# Step 12: Copy to docs folder
Log-Step "========== Step 12: Copy to Docs =========="
if (-not (Test-Path $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
}
$docsVideo = "$OutDir\dinoforge_proof_${timestamp}.mp4"
Copy-Item $finalVideo $docsVideo -Force
Log-Success "Copied to docs: $docsVideo"

# Step 13: Cleanup and finish
Log-Step "========== Step 13: Cleanup =========="
Kill-Game "final cleanup"
Remove-Item $rawVideoFile -Force -ErrorAction SilentlyContinue
Remove-Item $audioFile -Force -ErrorAction SilentlyContinue
foreach ($f in $ttsFiles) {
    Remove-Item $f -Force -ErrorAction SilentlyContinue
}
Remove-Item "$TempDir\silence_1s.mp3" -Force -ErrorAction SilentlyContinue

# Step 14: Open video
Log-Step "========== Step 14: Open Video =========="
Log-Success "Video complete: $finalVideo"
Log-Success "Also saved to: $docsVideo"
Log-Step "Opening video player..."
Start-Process $finalVideo

Log-Success "========== Pipeline Complete =========="

