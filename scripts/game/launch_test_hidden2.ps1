Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class DesktopLauncher {
    [DllImport("user32.dll")] public static extern IntPtr CreateDesktop(string lpszDesktop, IntPtr lpszDevice, IntPtr pDevmode, int dwFlags, uint dwDesiredAccess, IntPtr lpsa);
    [DllImport("user32.dll")] public static extern bool CloseDesktop(IntPtr hDesktop);
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] public static extern bool CreateProcess(string lpAppName, System.Text.StringBuilder lpCmdLine, IntPtr lpPA, IntPtr lpTA, bool bInherit, uint dwFlags, IntPtr lpEnv, string lpCurDir, ref STARTUPINFO lpSI, out PROCESS_INFORMATION lpPI);
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct STARTUPINFO { public int cb; public string lpReserved; public string lpDesktop; public string lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError; }
    [StructLayout(LayoutKind.Sequential)] public struct PROCESS_INFORMATION { public IntPtr hProcess, hThread; public int dwProcessId, dwThreadId; }
}
"@

$exe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option.exe"
$desktopName = "DINOForge_Agent_Test"

if (-not (Test-Path $exe)) {
    Write-Host "[ERROR] TEST exe not found: $exe"
    exit 1
}

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Creating desktop '$desktopName'..."
$desktop = [DesktopLauncher]::CreateDesktop($desktopName, [IntPtr]::Zero, [IntPtr]::Zero, 0, 0x01FF, [IntPtr]::Zero)
if ($desktop -eq [IntPtr]::Zero) {
    Write-Host "[ERROR] CreateDesktop failed"
    exit 1
}

$si = New-Object DesktopLauncher+STARTUPINFO
$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($si)
$si.lpDesktop = $desktopName
$si.dwFlags = 0x00000001
$si.wShowWindow = 0

$pi = New-Object DesktopLauncher+PROCESS_INFORMATION
$exeDir = Split-Path $exe -Parent

# Try 1: lpAppName=exe, lpCmdLine="-popupwindow"
$cmdLine = "`"$exe`" -popupwindow"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Try 1: CreateProcess(app='$exe', cmdline=$cmdLine)"
$ok = [DesktopLauncher]::CreateProcess($exe, $cmdLine, [IntPtr]::Zero, [IntPtr]::Zero, $false, 0x00000010, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi)

if ($ok) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] SUCCESS PID: $($pi.dwProcessId)"
} else {
    Write-Host "[ERROR] CreateProcess failed (Try 1)"
    # Try 2: lpAppName=null
    $cmdLine2 = "`"$exe`" -popupwindow"
    $pi2 = New-Object DesktopLauncher+PROCESS_INFORMATION
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Try 2: CreateProcess(null, cmdline=$cmdLine2)"
    $ok2 = [DesktopLauncher]::CreateProcess([null], $cmdLine2, [IntPtr]::Zero, [IntPtr]::Zero, $false, 0x00000010, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi2)
    if ($ok2) {
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] SUCCESS PID: $($pi2.dwProcessId)"
    } else {
        Write-Host "[ERROR] CreateProcess failed (Try 2)"
        # Try 3: no args
        $pi3 = New-Object DesktopLauncher+PROCESS_INFORMATION
        $cmdLine3 = "`"$exe`""
        Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Try 3: CreateProcess(app='$exe', cmdline='$cmdLine3')"
        $ok3 = [DesktopLauncher]::CreateProcess($exe, $cmdLine3, [IntPtr]::Zero, [IntPtr]::Zero, $false, 0x00000010, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi3)
        if ($ok3) {
            Write-Host "[$(Get-Date -Format 'HH:mm:ss')] SUCCESS (no args) PID: $($pi3.dwProcessId)"
        } else {
            Write-Host "[ERROR] All CreateProcess attempts failed"
        }
    }
}
