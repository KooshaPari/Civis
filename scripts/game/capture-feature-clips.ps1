#Requires -Version 5.1
<#
.SYNOPSIS
    DINOForge Prove-Features Pipeline — Phase 1: Game Capture.

.DESCRIPTION
    Launches the game, records three separate feature clips via DINOForge CLI
    (ScreenRecorderLib + WindowRecordingSource — true window-isolated capture),
    injects keys via Win32 SendInput (no window focus required), and outputs
    raw clips + metadata to $env:TEMP\DINOForge\capture\.

    VLM validation (game_analyze_screen MCP) is orchestrated by the caller
    (.claude/commands/prove-features.md) between recording steps, since
    MCP tool access lives in the Claude session. This script signals readiness
    and provides the clip output path; the caller validates and writes
    validate_report.json.

.NOTES
    Run from repo root:
        powershell -ExecutionPolicy Bypass -File scripts/game/capture-feature-clips.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Configuration ──────────────────────────────────────────────────────────────
$cliExe    = "$PSScriptRoot\..\..\src\Tools\Cli\bin\x64\Release\net11.0\DINOForge.Tools.Cli.exe"
$gameExe   = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$gameDir   = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$debugLog  = "$gameDir\BepInEx\dinoforge_debug.log"
$logOutput = "$gameDir\BepInEx\LogOutput.log"
$outDir    = "$env:TEMP\DINOForge\capture"
$gameTitle = "Diplomacy is Not an Option"

New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# ── Win32 SendInput: inject keys without requiring window focus ────────────────
Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Threading;

public class Win32Input {
    [StructLayout(LayoutKind.Sequential)]
    struct INPUT {
        public int type;
        public KEYBDINPUT ki;
        public long reserved;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct KEYBDINPUT {
        public ushort wVk;
        public ushort wScan;
        public uint   dwFlags;
        public uint   time;
        public IntPtr dwExtraInfo;
    }

    const int  INPUT_KEYBOARD  = 1;
    const uint KEYEVENTF_KEYUP = 0x0002;

    [DllImport("user32.dll", SetLastError = true)]
    static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    public static void PressKey(ushort vk) {
        int sz = Marshal.SizeOf(typeof(INPUT));
        var down = new INPUT { type = INPUT_KEYBOARD, ki = new KEYBDINPUT { wVk = vk } };
        var up   = new INPUT { type = INPUT_KEYBOARD, ki = new KEYBDINPUT { wVk = vk, dwFlags = KEYEVENTF_KEYUP } };
        SendInput(1, new[] { down }, sz);
        Thread.Sleep(80);
        SendInput(1, new[] { up }, sz);
    }
}
"@

$VK_F9  = [uint16]0x78
$VK_F10 = [uint16]0x79

# ── Helpers ────────────────────────────────────────────────────────────────────

function Wait-ForLog {
    param(
        [string]$LogPath,
        [string]$Pattern,
        [int]   $TimeoutSec
    )
    $elapsed = 0
    while ($elapsed -lt $TimeoutSec) {
        Start-Sleep -Seconds 2
        $elapsed += 2
        try {
            $stream  = [System.IO.File]::Open($LogPath, 'Open', 'Read', 'ReadWrite')
            $reader  = New-Object System.IO.StreamReader($stream)
            $content = $reader.ReadToEnd()
            $reader.Close(); $stream.Close()
            if ($content -match $Pattern) { return $true }
        } catch { }
    }
    return $false
}

function Invoke-RecordSync {
    param(
        [string]$OutputPath,
        [int]   $DurationSec
    )
    Write-Host "[Capture] Using DINOForge CLI ScreenRecorderLib (window-isolated): '$gameTitle'"
    # Kill any existing CLI processes first
    Stop-Process -Name "DINOForge.Tools.Cli" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
    $cliArgs = "record --output `"$OutputPath`" --duration $DurationSec --width 1280 --height 800 --framerate 30"
    Start-Process -FilePath $cliExe -ArgumentList $cliArgs -Wait -NoNewWindow
}

function Start-RecordAsync {
    param(
        [string]$OutputPath,
        [int]   $DurationSec
    )
    Write-Host "[Capture] Using DINOForge CLI ScreenRecorderLib (window-isolated): '$gameTitle'"
    # Kill any existing CLI processes first
    Stop-Process -Name "DINOForge.Tools.Cli" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
    $cliArgs = "record --output `"$OutputPath`" --duration $DurationSec --width 1280 --height 800 --framerate 30"
    return Start-Process -FilePath $cliExe -ArgumentList $cliArgs -PassThru -NoNewWindow
}

# ── Step 1: Kill any existing game instances ───────────────────────────────────
Write-Host "[Phase 1] Stopping existing game processes..."
Stop-Process -Name $gameTitle        -Force -ErrorAction SilentlyContinue
Stop-Process -Name "UnityCrashHandler64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# ── Step 2: Launch game ────────────────────────────────────────────────────────
Write-Host "[Phase 1] Launching game..."
if (Test-Path $debugLog)  { Clear-Content $debugLog  -ErrorAction SilentlyContinue }
if (Test-Path $logOutput) { Clear-Content $logOutput -ErrorAction SilentlyContinue }
Start-Process -FilePath $gameExe -WorkingDirectory $gameDir

# ── Step 3: Boot Stage A — Wait for DINOForge Awake ───────────────────────────
Write-Host "[Phase 1] Waiting for DINOForge Awake (30s timeout)..."
$ok = Wait-ForLog -LogPath $debugLog -Pattern "Awake completed" -TimeoutSec 30
if (-not $ok) {
    Write-Error "TIMEOUT: DINOForge 'Awake completed' not detected within 30s."
    exit 1
}
Write-Host "[Phase 1] DINOForge awake confirmed."

# ── Step 4: Boot Stage B — Wait for Mods button injection ─────────────────────
Write-Host "[Phase 1] Waiting for Mods button injection (720s timeout)..."
$ok = Wait-ForLog -LogPath $logOutput -Pattern "MODS BUTTON INJECTION FULLY SUCCESSFUL" -TimeoutSec 720
if (-not $ok) {
    Write-Error "TIMEOUT: Mods button injection not confirmed within 720s."
    exit 1
}
Write-Host "[Phase 1] Mods button injection confirmed."
Start-Sleep -Seconds 2

# ── Step 5: Record raw_mods.mp4 (main menu, 6s) ───────────────────────────────
Write-Host "[Phase 1] Recording raw_mods.mp4 (6s, main menu)..."
$modsOut = "$outDir\raw_mods.mp4"
Invoke-RecordSync -OutputPath $modsOut -DurationSec 6
if (-not (Test-Path $modsOut) -or (Get-Item $modsOut).Length -lt 4096) {
    Write-Error "raw_mods.mp4 missing or empty. CLI recording may have failed."
    exit 1
}
Write-Host "[Phase 1] raw_mods.mp4 ready ($([math]::Round((Get-Item $modsOut).Length / 1KB, 0)) KB)"

# Signal: ready for VLM validation of mods
"mods_recorded" | Set-Content "$outDir\signal_mods.txt"

# ── Step 6: Record raw_f9.mp4 (F9 debug overlay, 8s) ─────────────────────────
Write-Host "[Phase 1] Recording raw_f9.mp4 (8s, F9 overlay)..."
$f9Out = "$outDir\raw_f9.mp4"
$proc  = Start-RecordAsync -OutputPath $f9Out -DurationSec 8

# Inject F9 after 0.5s so the recording captures the key press event
Start-Sleep -Milliseconds 500
[Win32Input]::PressKey($VK_F9)
Write-Host "[Phase 1] F9 injected via SendInput (no focus required)"

$proc.WaitForExit(25000) | Out-Null
if (-not (Test-Path $f9Out) -or (Get-Item $f9Out).Length -lt 4096) {
    Write-Error "raw_f9.mp4 missing or empty."
    exit 1
}
Write-Host "[Phase 1] raw_f9.mp4 ready ($([math]::Round((Get-Item $f9Out).Length / 1KB, 0)) KB)"

# Signal: ready for VLM validation of F9
"f9_recorded" | Set-Content "$outDir\signal_f9.txt"

# Reset F9 overlay before next recording
Start-Sleep -Seconds 1
[Win32Input]::PressKey($VK_F9)
Start-Sleep -Seconds 2

# ── Step 7: Record raw_f10.mp4 (F10 mod menu, 8s) ────────────────────────────
Write-Host "[Phase 1] Recording raw_f10.mp4 (8s, F10 mod menu)..."
$f10Out = "$outDir\raw_f10.mp4"
$proc   = Start-RecordAsync -OutputPath $f10Out -DurationSec 8

Start-Sleep -Milliseconds 500
[Win32Input]::PressKey($VK_F10)
Write-Host "[Phase 1] F10 injected via SendInput (no focus required)"

$proc.WaitForExit(25000) | Out-Null
if (-not (Test-Path $f10Out) -or (Get-Item $f10Out).Length -lt 4096) {
    Write-Error "raw_f10.mp4 missing or empty."
    exit 1
}
Write-Host "[Phase 1] raw_f10.mp4 ready ($([math]::Round((Get-Item $f10Out).Length / 1KB, 0)) KB)"

"f10_recorded" | Set-Content "$outDir\signal_f10.txt"

# ── Summary ────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "===== Phase 1 Complete ================================================="
Write-Host "Output directory : $outDir"
Write-Host "  raw_mods.mp4   : $modsOut"
Write-Host "  raw_f9.mp4     : $f9Out"
Write-Host "  raw_f10.mp4    : $f10Out"
Write-Host ""
Write-Host "NEXT: Orchestrator must call game_analyze_screen for each feature"
Write-Host "      and write validate_report.json to $outDir"
Write-Host "========================================================================"
exit 0
