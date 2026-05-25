#Requires -Version 5.1
<#
.SYNOPSIS
    Capture proof-of-feature screenshots for DINOForge mod injection.

.DESCRIPTION
    This script:
    1. Kills all DINO instances, waits, and launches a fresh game
    2. Waits for injection confirmation (MODS BUTTON INJECTION FULLY SUCCESSFUL)
    3. Waits up to 5 minutes for the game to become "Responding"
    4. Takes cp1_mainmenu screenshot
    5. Sends F9 and takes cp2_f9_overlay screenshot
    6. Sends F10 and takes cp3_f10_menu screenshot
    7. Kills the game and reports results

.PARAMETER Verbose
    Enable detailed console output (timestamps at each step)
#>

param(
    [switch]$Verbose
)

# Configuration
$OutDir = "C:\Users\koosh\Dino\docs\proof-of-features"
$GameExe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$GameDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$LogFile = "$GameDir\BepInEx\LogOutput.log"
$BepInExDir = "$GameDir\BepInEx"
$ReqFile = "$BepInExDir\dinoforge_screenshot_request.txt"
$DoneFile = "$BepInExDir\dinoforge_screenshot_done.txt"

function Log-Step {
    param([string]$Message)
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] $Message"
}

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WF8 {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    public delegate bool EnumWindowsProc(IntPtr h, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc fn, IntPtr lp);
    public static IntPtr FindWindowByPid(uint targetPid) {
        IntPtr found = IntPtr.Zero;
        EnumWindows(delegate(IntPtr h, IntPtr lp) {
            if (!IsWindowVisible(h)) return true;
            uint pid = 0; GetWindowThreadProcessId(h, out pid);
            if (pid == targetPid) { found = h; return false; }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
"@ -ErrorAction Stop

Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public class FK8 {
    [StructLayout(LayoutKind.Sequential)] struct KI { public ushort vk, sc; public uint fl, t; public IntPtr ex; }
    [StructLayout(LayoutKind.Explicit)]   struct IU { [FieldOffset(0)] public KI ki; }
    [StructLayout(LayoutKind.Sequential)] struct IN { public uint type; public IU u; }
    [DllImport("user32.dll")] static extern uint SendInput(uint n, IN[] inp, int sz);
    public static void Press(ushort vk) {
        var inp = new IN[2];
        inp[0].type = 1; inp[0].u.ki.vk = vk;
        inp[1].type = 1; inp[1].u.ki.vk = vk; inp[1].u.ki.fl = 2;
        SendInput(2, inp, System.Runtime.InteropServices.Marshal.SizeOf(typeof(IN)));
    }
}
"@ -ErrorAction Stop

function Get-Game-Process {
    return Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | Where-Object { $_.Id -gt 0 } | Select-Object -First 1
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
    Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
    Start-Sleep -Seconds 3
    $proc = Get-Game-Process
    if ($proc) { Log-Step "WARNING: Game process still exists after kill!"; return $false }
    Log-Step "Game process confirmed dead."
    return $true
}

function Launch-Game {
    Log-Step "Launching game..."
    $script:proc = Start-Process -FilePath $GameExe -WorkingDirectory $GameDir -PassThru
    $script:pid2 = $script:proc.Id
    Log-Step "Game launched. PID=$($script:pid2)"
    return $true
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

function Wait-For-Injection {
    param([int]$TimeoutSeconds = 720, [DateTime]$logBasetime)

    Log-Step "Waiting up to $TimeoutSeconds seconds for injection confirmation... (logBasetime=$($logBasetime.ToString('HH:mm:ss')))"

    $stopTime = [DateTime]::Now.AddSeconds($TimeoutSeconds)
    $injected = $false
    $checkCount = 0
    while ([DateTime]::Now -lt $stopTime) {
        # Kill any spurious extra game instances
        Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue |
            Where-Object { $_.Id -ne $script:pid2 } |
            ForEach-Object { try { $_.Kill() } catch {} }

        # Keep game focused
        $hw = [WF8]::FindWindowByPid([uint32]$script:pid2)
        if ($hw -ne [IntPtr]::Zero) { FocusGame $hw }

        if (-not $injected -and (Test-Path $LogFile)) {
            try {
                $checkCount++
                $item = Get-Item $LogFile -ErrorAction SilentlyContinue
                if ($checkCount -le 3 -or $checkCount % 10 -eq 0) {
                    $lw = $item.LastWriteTime.ToString('HH:mm:ss'); $bt = $logBasetime.ToString('HH:mm:ss'); $cmp = $item.LastWriteTime -gt $logBasetime
                    Log-Step "  Log check #${checkCount}: LastWrite=$lw > basetime=$bt = $cmp"
                }
                if ($item -and $item.LastWriteTime -gt $logBasetime) {
                    $fs = [System.IO.File]::Open($LogFile, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite)
                    $reader = New-Object System.IO.StreamReader($fs, [System.Text.Encoding]::UTF8)
                    $c = $reader.ReadToEnd()
                    $reader.Close(); $fs.Close()
                    $hasInject = $c -and $c.Contains("MODS BUTTON INJECTION FULLY SUCCESSFUL")
                    $fl = $item.Length
                    if ($checkCount -le 3 -or $checkCount % 10 -eq 0) {
                        Log-Step "  HasInjectionString=$hasInject (fileLen=$fl)"
                    }
                    if ($hasInject) {
                        $injected = $true
                        Log-Step "Injection confirmed!"
                    }
                }
            } catch { $err = $_.Exception.Message; Log-Step "  Log check exception: $err" }
        }
        if ($injected) { return $true }
        Start-Sleep -Seconds 2
    }
    Log-Step "ERROR: Injection confirmation not found within $TimeoutSeconds seconds"
    return $false
}

function Wait-For-Main-Menu {
    # Wait a fixed time after injection for the main menu to fully render.
    # The game stays "Not Responding" during loading - we cannot rely on Responding status.
    # 3 minutes is typically sufficient for the loading screen to complete.
    param([int]$WaitSeconds = 30, [System.IntPtr]$hwnd = [IntPtr]::Zero)
    Log-Step "Waiting $WaitSeconds seconds for main menu to finish loading (keeping game focused)..."
    $stop = [DateTime]::Now.AddSeconds($WaitSeconds)
    $lastFocus = [DateTime]::MinValue
    while ([DateTime]::Now -lt $stop) {
        $now = [DateTime]::Now
        if (($now - $lastFocus).TotalSeconds -ge 5) {
            $hw = [WF8]::FindWindowByPid([uint32]$script:pid2)
            if ($hw -ne [IntPtr]::Zero) { FocusGame $hw }
            $lastFocus = $now
            $remaining = [int]($stop - $now).TotalSeconds
            # Check if game is responding (loaded faster than expected)
            $gp = Get-Process -Id $script:pid2 -ErrorAction SilentlyContinue
            if ($gp -and $gp.Responding) {
                Log-Step "Game is Responding after $([int]($WaitSeconds - $remaining))s - main menu ready!"
                return $true
            }
            Log-Step "  Still loading... ${remaining}s remaining (Responding=$($gp.Responding))"
        }
        Start-Sleep -Milliseconds 500
    }
    Log-Step "Fixed wait complete - proceeding with screenshot capture regardless of Responding state"
    return $true
}

function Request-Screenshot {
    param([string]$OutputPath, [int]$TimeoutSeconds = 60)
    Log-Step "Requesting screenshot: $OutputPath"
    Remove-Item $DoneFile -Force -ErrorAction SilentlyContinue
    Remove-Item $OutputPath -Force -ErrorAction SilentlyContinue
    $hw = [WF8]::FindWindowByPid([uint32]$script:pid2)
    FocusGame $hw
    Start-Sleep -Milliseconds 300
    [System.IO.File]::WriteAllText($ReqFile, $OutputPath, [System.Text.Encoding]::UTF8)
    $dl = [DateTime]::Now.AddSeconds($TimeoutSeconds)
    while ([DateTime]::Now -lt $dl) {
        if (Test-Path $DoneFile) {
            Remove-Item $DoneFile -Force -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 800
            if (Test-Path $OutputPath) {
                $item = Get-Item $OutputPath
                if ($item.Length -gt 50000) {
                    Log-Step "  Done: $($item.Length) bytes"
                    return $true
                } else {
                    Log-Step "  File $($item.Length) bytes - too small, retrying..."
                }
            }
        }
        $hw = [WF8]::FindWindowByPid([uint32]$script:pid2)
        FocusGame $hw
        Start-Sleep -Milliseconds 300
    }
    Log-Step "  TIMEOUT after ${TimeoutSeconds}s"
    if (Test-Path $OutputPath) { $sz = (Get-Item $OutputPath).Length; Log-Step "  File on disk: $sz bytes" }
    return $false
}

Log-Step "========== DINOForge Proof-of-Features Capture v6 =========="

# Kill any existing game instances
Log-Step "Killing existing game instances..."
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
Remove-Item $ReqFile -Force -ErrorAction SilentlyContinue
Remove-Item $DoneFile -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

# Launch game
$logBasetime = if (Test-Path $LogFile) { (Get-Item $LogFile).LastWriteTime } else { [DateTime]::Now }
Launch-Game | Out-Null

# Find HWND and focus immediately
$focusDeadline = [DateTime]::Now.AddSeconds(20)
while ([DateTime]::Now -lt $focusDeadline) {
    $hw = [WF8]::FindWindowByPid([uint32]$script:pid2)
    if ($hw -ne [IntPtr]::Zero) {
        FocusGame $hw
        Log-Step "Focused HWND=0x$($hw.ToString('X'))"
        break
    }
    Start-Sleep -Milliseconds 200
}

# Wait for injection (720s - cold-start loading can take 6+ minutes due to Addressables bundle loading)
$injected = Wait-For-Injection -TimeoutSeconds 720 -logBasetime $logBasetime
if (-not $injected) {
    Log-Step "INJECTION FAILED - killing game"
    Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
    exit 1
}
$injectedAt = [DateTime]::Now

# Wait for main menu to fully render - game stays "Not Responding" during loading even after injection
# We wait up to 5 minutes for Responding=True, or give up and try anyway after timeout
Wait-For-Main-Menu -WaitSeconds 30 | Out-Null

# Get final HWND
$gp = Get-Process -Id $script:pid2 -ErrorAction SilentlyContinue
$hwnd = [WF8]::FindWindowByPid([uint32]$script:pid2)
if ($hwnd -eq [IntPtr]::Zero -and $gp) { $hwnd = $gp.MainWindowHandle }
Log-Step "Window: HWND=0x$($hwnd.ToString('X')) Responding=$($gp.Responding) Title='$($gp.MainWindowTitle)'"

# Ensure output dir exists
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir -Force | Out-Null }

# cp1: main menu
Log-Step "--- cp1: main menu screenshot ---"
$ok1 = Request-Screenshot "$OutDir\cp1_mainmenu.png" 60

# F9 overlay
Log-Step "Sending F9..."
FocusGame $hwnd
[FK8]::Press(0x78)
Start-Sleep -Seconds 12
Log-Step "--- cp2: F9 overlay screenshot ---"
$ok2 = Request-Screenshot "$OutDir\cp2_f9_overlay.png" 60

# F10 menu
Log-Step "Sending F10..."
FocusGame $hwnd
[FK8]::Press(0x79)
Start-Sleep -Seconds 12
Log-Step "--- cp3: F10 menu screenshot ---"
$ok3 = Request-Screenshot "$OutDir\cp3_f10_menu.png" 60

# Kill game
Log-Step "Killing game..."
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }

# Results
Log-Step "Results: ok1=$ok1 ok2=$ok2 ok3=$ok3"
Get-ChildItem $OutDir -Filter "cp*.png" -ErrorAction SilentlyContinue | Sort-Object Name | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
