$OutDir     = "C:\Users\koosh\Dino\docs\proof-of-features"
$GameExe    = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$GameDir    = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$LogFile    = "$GameDir\BepInEx\LogOutput.log"
$BepInExDir = "$GameDir\BepInEx"
$ReqFile    = "$BepInExDir\dinoforge_screenshot_request.txt"
$DoneFile   = "$BepInExDir\dinoforge_screenshot_done.txt"
$AutoCP1    = "$BepInExDir\cp1_mods_injected.png"

function Log { param([string]$m); Write-Host "[$([DateTime]::Now.ToString('HH:mm:ss'))] $m" }

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WF7 {
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
public class FK7 {
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

function FocusGame { param([IntPtr]$hwnd)
    if ($hwnd -ne [IntPtr]::Zero) {
        [WF7]::ShowWindow($hwnd, 9) | Out-Null
        [WF7]::SetForegroundWindow($hwnd) | Out-Null
    }
}

function RequestScreenshot {
    param([string]$outPath, [string]$label, [IntPtr]$hwnd)
    Log "Requesting screenshot: $label"
    Remove-Item $DoneFile -Force -EA SilentlyContinue
    Remove-Item $outPath -Force -EA SilentlyContinue
    FocusGame $hwnd
    Start-Sleep -Milliseconds 300
    Set-Content -Path $ReqFile -Value $outPath -Encoding UTF8
    $requestTime = Get-Date
    $dl = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $dl) {
        if (Test-Path $DoneFile) {
            Remove-Item $DoneFile -Force -EA SilentlyContinue
            Start-Sleep -Milliseconds 800
            if (Test-Path $outPath) {
                $item = Get-Item $outPath
                if ($item.Length -gt 50000) {
                    Log "  Done: $($item.Length) bytes at $($item.LastWriteTime)"
                    return $true
                } else {
                    Log "  File size $($item.Length) - too small, waiting..."
                }
            }
        }
        FocusGame $hwnd
        Start-Sleep -Milliseconds 300
    }
    Log "  TIMEOUT"
    if (Test-Path $outPath) { $sz = (Get-Item $outPath).Length; Log "  File: $sz bytes" }
    return $false
}

# Kill existing
Log "Killing existing game instances..."
Get-Process -Name "Diplomacy is Not an Option" -EA SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
# Also delete old auto-checkpoint
Remove-Item $AutoCP1 -Force -EA SilentlyContinue
Start-Sleep -Seconds 3

# Launch
$logBasetime = if (Test-Path $LogFile) { (Get-Item $LogFile).LastWriteTime } else { Get-Date }
Log "Launching game..."
$proc = Start-Process -FilePath $GameExe -WorkingDirectory $GameDir -PassThru
$pid2 = $proc.Id
Log "PID=$pid2"

# Immediately focus
$focusDeadline = (Get-Date).AddSeconds(20)
while ((Get-Date) -lt $focusDeadline) {
    $hwnd = [WF7]::FindWindowByPid([uint32]$pid2)
    if ($hwnd -ne [IntPtr]::Zero) {
        FocusGame $hwnd
        Log "Focused HWND=0x$($hwnd.ToString('X'))"
        break
    }
    Start-Sleep -Milliseconds 200
}

# Wait for injection
Log "Waiting for injection (90s)..."
$deadline = (Get-Date).AddSeconds(90)
$injected = $false
$injectedAt = $null
while ((Get-Date) -lt $deadline) {
    Get-Process -Name "Diplomacy is Not an Option" -EA SilentlyContinue |
        Where-Object { $_.Id -ne $pid2 } |
        ForEach-Object { try { $_.Kill() } catch {} }
    $hw = [WF7]::FindWindowByPid([uint32]$pid2)
    if ($hw -ne [IntPtr]::Zero) { FocusGame $hw }
    if (-not $injected -and (Test-Path $LogFile)) {
        $item = Get-Item $LogFile -EA SilentlyContinue
        if ($item -and $item.LastWriteTime -gt $logBasetime) {
            $c = Get-Content $LogFile -Raw -EA SilentlyContinue
            if ($c -and $c.Contains("MODS BUTTON INJECTION FULLY SUCCESSFUL")) {
                $injected = $true; $injectedAt = Get-Date; Log "INJECTED at $injectedAt!"
            }
        }
    }
    if ($injected) { break }
    Start-Sleep -Seconds 2
}
if (-not $injected) { Log "INJECTION FAILED"; Stop-Process -Id $pid2 -Force -EA SilentlyContinue; exit 1 }

# Now wait for the auto-checkpoint screenshot (cp1_mods_injected.png) to appear
Log "Waiting for auto-checkpoint screenshot cp1_mods_injected.png (up to 60s)..."
$autoDeadline = (Get-Date).AddSeconds(60)
$autoGot = $false
while ((Get-Date) -lt $autoDeadline) {
    $hw = [WF7]::FindWindowByPid([uint32]$pid2)
    if ($hw -ne [IntPtr]::Zero) { FocusGame $hw }
    if (Test-Path $AutoCP1) {
        $autoItem = Get-Item $AutoCP1
        if ($autoItem.Length -gt 1000 -and $autoItem.LastWriteTime -gt $injectedAt) {
            Log "Auto-checkpoint ready: $($autoItem.Length) bytes"
            $autoGot = $true
            break
        }
    }
    Start-Sleep -Seconds 1
}
if (-not $autoGot) { Log "Auto-checkpoint did not appear - proceeding anyway" }

$gp = Get-Process -Id $pid2 -EA SilentlyContinue
$hwnd = [WF7]::FindWindowByPid([uint32]$pid2)
if ($hwnd -eq [IntPtr]::Zero -and $gp) { $hwnd = $gp.MainWindowHandle }
Log "Window: HWND=0x$($hwnd.ToString('X')) Responding=$($gp.Responding) Title='$($gp.MainWindowTitle)'"

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir -Force | Out-Null }

# If we got the auto-checkpoint, copy it as cp1
if ($autoGot) {
    Copy-Item $AutoCP1 "$OutDir\cp1_mainmenu.png" -Force
    Log "Copied auto-checkpoint as cp1_mainmenu.png"
    $ok1 = $true
} else {
    $ok1 = RequestScreenshot "$OutDir\cp1_mainmenu.png" "cp1_mainmenu" $hwnd
}

Log "Sending F9..."
FocusGame $hwnd
[FK7]::Press(0x78)
Start-Sleep -Seconds 10
$ok2 = RequestScreenshot "$OutDir\cp2_f9_overlay.png" "cp2_f9_overlay" $hwnd

Log "Sending F10..."
FocusGame $hwnd
[FK7]::Press(0x79)
Start-Sleep -Seconds 10
$ok3 = RequestScreenshot "$OutDir\cp3_f10_menu.png" "cp3_f10_menu" $hwnd

Log "Killing game..."
Stop-Process -Id $pid2 -Force -EA SilentlyContinue

Log "Results: ok1=$ok1 ok2=$ok2 ok3=$ok3"
Get-ChildItem $OutDir -Filter "cp*.png" | Sort-Object Name | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
