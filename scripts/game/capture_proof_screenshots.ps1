# capture_proof_screenshots.ps1
# Captures 3 proof-of-feature screenshots:
#   cp1_mainmenu.png  - Main menu with MODS button
#   cp2_f9_overlay.png - F9 debug overlay
#   cp3_f10_menu.png  - F10 mod menu
# Uses DXGI-compatible approach: forces windowed mode via registry BEFORE launch.

$OutDir   = "C:\Users\koosh\Dino\docs\proof-of-features"
$Ffmpeg   = "c:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe"
$Magick   = "c:\program files\imagemagick-7.1.0-q16-hdri\magick.exe"
$GameExe  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$GameDir  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$LogFile  = "$GameDir\BepInEx\LogOutput.log"
$SteamExe = "C:\Program Files (x86)\Steam\steam.exe"
$RegPath  = "HKCU:\SOFTWARE\Chudo-Yudo Games\Machine Mind"

function Log { param([string]$m); Write-Host "[$([DateTime]::Now.ToString('HH:mm:ss'))] $m" }

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public class WC {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int left, top, right, bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll", CharSet=CharSet.Auto)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
    public delegate bool EnumWindowsProc(IntPtr h, IntPtr lp);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc fn, IntPtr lp);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    public static void MinimizeByPid(uint pid) {
        EnumWindows(delegate(IntPtr h, IntPtr lp) {
            if (!IsWindowVisible(h)) return true;
            uint wp = 0; GetWindowThreadProcessId(h, out wp);
            if (wp == pid) { ShowWindow(h, 6); }
            return true;
        }, IntPtr.Zero);
    }
}
"@ -ErrorAction Stop

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class FK {
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

function CaptureGame {
    param([string]$out, [string]$label, [IntPtr]$hwnd)
    Log "Capturing $label..."
    Get-Process -Name "Steam","steamwebhelper" -EA SilentlyContinue | ForEach-Object {
        [WC]::MinimizeByPid([uint32]$_.Id)
    }
    Start-Sleep -Milliseconds 600
    $r = New-Object WC+RECT
    [WC]::GetWindowRect($hwnd, [ref]$r) | Out-Null
    $cw = $r.right - $r.left; $ch = $r.bottom - $r.top
    $cx = $r.left; $cy = $r.top
    Log "  rect: ${cw}x${ch} at (${cx},${cy})"
    if ($cw -lt 100 -or $ch -lt 100) { Log "  ERROR: rect too small (${cw}x${ch}) - window not found correctly"; return }
    $tmp = "$env:TEMP\dinoforge_cap_$([System.IO.Path]::GetFileNameWithoutExtension($out)).png"
    & $Ffmpeg -f gdigrab -framerate 1 -i desktop -frames:v 1 -y $tmp 2>&1 | Out-Null
    $fullSz = if (Test-Path $tmp) { (Get-Item $tmp).Length } else { 0 }
    Log "  Desktop capture: $fullSz bytes"
    if ($fullSz -lt 50000) { Log "  WARNING: desktop capture may be black (exclusive fullscreen) - size $fullSz bytes"; }
    & $Magick $tmp -crop "${cw}x${ch}+${cx}+${cy}" +repage $out 2>&1 | Out-Null
    if (Test-Path $out) {
        Log "  OK: $((Get-Item $out).Length) bytes -> $out"
    } else {
        Copy-Item $tmp $out -Force -EA SilentlyContinue
        Log "  Fallback (full desktop): $((Get-Item $out).Length) bytes"
    }
    Remove-Item $tmp -Force -EA SilentlyContinue
}

# ---- STEP 1: FIX REGISTRY (windowed mode = 3) ----
Log "Setting Unity windowed mode in registry (FullScreenMode=3=Windowed)..."
if (-not (Test-Path $RegPath)) { New-Item -Path $RegPath -Force | Out-Null }
Set-ItemProperty -Path $RegPath -Name 'Screenmanager Fullscreen mode_h3630240806' -Value 3 -Type DWord -Force
Set-ItemProperty -Path $RegPath -Name 'Screenmanager Resolution Width_h182942802' -Value 1280 -Type DWord -Force -EA SilentlyContinue
Set-ItemProperty -Path $RegPath -Name 'Screenmanager Resolution Height_h182942802' -Value 720 -Type DWord -Force -EA SilentlyContinue
Set-ItemProperty -Path $RegPath -Name 'Screenmanager Is Fullscreen mode' -Value 0 -Type DWord -Force -EA SilentlyContinue
$fsModeVal = (Get-ItemProperty -Path $RegPath -EA SilentlyContinue).'Screenmanager Fullscreen mode_h3630240806'
Log "Registry set: Fullscreen mode = $fsModeVal (3=Windowed)"

# ---- STEP 2: KILL EXISTING GAME INSTANCES ----
Log "Killing ALL game instances..."
Get-Process -Name "Diplomacy is Not an Option" -EA SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
Start-Sleep -Seconds 3
$remaining = Get-Process -Name "Diplomacy*" -EA SilentlyContinue
if ($remaining) { $remaining | ForEach-Object { try { $_.Kill() } catch {} }; Start-Sleep 2 }
Log "Game processes cleared"

# ---- STEP 3: ENSURE STEAM RUNNING ----
$steamProc = Get-Process -Name "steam" -EA SilentlyContinue | Select-Object -First 1
if (-not $steamProc) {
    Log "Starting Steam..."
    Start-Process -FilePath $SteamExe -ArgumentList "-silent"
    Start-Sleep -Seconds 15
    Log "Steam started"
} else {
    Log "Steam running (PID=$($steamProc.Id))"
}
Get-Process -Name "steam" -EA SilentlyContinue | ForEach-Object {
    [WC]::MinimizeByPid([uint32]$_.Id)
}
Start-Sleep -Milliseconds 500

# ---- STEP 4: LAUNCH GAME ----
$logBasetime = if (Test-Path $LogFile) { (Get-Item $LogFile).LastWriteTime } else { Get-Date }
Log "Log baseline: $logBasetime"
Log "Launching game (windowed 1280x720)..."
$proc = Start-Process -FilePath $GameExe -WorkingDirectory $GameDir -ArgumentList "-window-mode windowed -screen-width 1280 -screen-height 720" -PassThru
$launchedPid = $proc.Id
Log "PID=$launchedPid"

# ---- STEP 5: WAIT FOR INJECTION (90s) ----
$deadline = (Get-Date).AddSeconds(90)
$injected = $false
$launchTime = Get-Date
while ((Get-Date) -lt $deadline) {
    Get-Process -Name "Diplomacy is Not an Option" -EA SilentlyContinue |
        Where-Object { $_.Id -ne $launchedPid } |
        ForEach-Object { try { $_.Kill(); Log "Killed competing PID=$($_.Id)" } catch {} }
    Get-Process | Where-Object { $_.MainWindowTitle -like "*Fatal*" } -EA SilentlyContinue | ForEach-Object { try { $_.Kill() } catch {} }
    if (-not $injected -and (Test-Path $LogFile)) {
        $item = Get-Item $LogFile -EA SilentlyContinue
        if ($item -and $item.LastWriteTime -gt $logBasetime) {
            $c = Get-Content $LogFile -Raw -EA SilentlyContinue
            if ($c -and $c.Contains("MODS BUTTON INJECTION FULLY SUCCESSFUL")) {
                $injected = $true; Log "INJECTED!"
            }
        }
    }
    if ($injected) { break }
    Start-Sleep -Seconds 2
}

if (-not $injected) {
    Log "INJECTION FAILED - check BepInEx\LogOutput.log"
    Stop-Process -Id $launchedPid -Force -EA SilentlyContinue
    exit 1
}

Log "Waiting 40s for main menu to fully render..."
Start-Sleep -Seconds 40

# ---- STEP 6: FIND WINDOW ----
$gameProc = Get-Process -Id $launchedPid -EA SilentlyContinue
$hwnd = if ($gameProc) { $gameProc.MainWindowHandle } else { [IntPtr]::Zero }
if ($hwnd -eq [IntPtr]::Zero) {
    # Fallback: search by title
    $byTitle = Get-Process -Name "Diplomacy is Not an Option" -EA SilentlyContinue | Where-Object { $_.MainWindowTitle -notlike "*Fatal*" } | Select-Object -First 1
    $hwnd = if ($byTitle) { $byTitle.MainWindowHandle } else { [IntPtr]::Zero }
}
if ($hwnd -eq [IntPtr]::Zero) { Log "ERROR: game HWND not found"; exit 1 }
Log "HWND=0x$($hwnd.ToString('X')) Title='$($gameProc.MainWindowTitle)'"

# Report window style/rect
Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public class WS2 {
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int n);
}
"@ -EA SilentlyContinue
$style = [WS2]::GetWindowLong($hwnd, -16)
$rCheck = New-Object WC+RECT
[WC]::GetWindowRect($hwnd, [ref]$rCheck) | Out-Null
Log "Window style=0x$($style.ToString('X8')) IsPopup=$(($style -band [int]0x80000000) -ne 0) HasCaption=$(($style -band 0x00C00000) -ne 0)"
Log "Window rect=$($rCheck.left),$($rCheck.top) -> $($rCheck.right),$($rCheck.bottom) size=$($rCheck.right-$rCheck.left)x$($rCheck.bottom-$rCheck.top)"

# Minimize Steam windows
Get-Process -Name "steam","steamwebhelper","GameOverlayUI" -EA SilentlyContinue | ForEach-Object {
    [WC]::MinimizeByPid([uint32]$_.Id)
}
Start-Sleep -Milliseconds 800

# Ensure output dir
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir -Force | Out-Null }

# ---- STEP 7: CAPTURE cp1_mainmenu ----
CaptureGame "$OutDir\cp1_mainmenu.png" "cp1_mainmenu" $hwnd

# ---- STEP 8: F9 OVERLAY ----
Log "Sending F9..."
[FK]::Press(0x78)
Start-Sleep -Seconds 5
CaptureGame "$OutDir\cp2_f9_overlay.png" "cp2_f9_overlay" $hwnd

# ---- STEP 9: F10 MENU ----
Log "Sending F10..."
[FK]::Press(0x79)
Start-Sleep -Seconds 5
CaptureGame "$OutDir\cp3_f10_menu.png" "cp3_f10_menu" $hwnd

# ---- DONE ----
Log "Killing game..."
Stop-Process -Id $launchedPid -Force -EA SilentlyContinue

Log "Done. Screenshots:"
Get-ChildItem $OutDir -Filter "cp*.png" | Select-Object Name, Length, LastWriteTime | Format-Table

Log "REMINDER: If screenshots are still black, game may still run exclusive fullscreen."
Log "The MCP server game_screenshot tool now uses DXGI Desktop Duplication as primary capture."
Log "Use: dotnet run --project src/Tools/McpServer -- (then call game_screenshot via MCP)"
