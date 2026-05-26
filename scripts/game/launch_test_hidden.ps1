Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32Desktop {
    [DllImport("user32.dll")] public static extern IntPtr CreateDesktop(string lpszDesktop, IntPtr lpszDevice, IntPtr pDevmode, int dwFlags, uint dwDesiredAccess, IntPtr lpsa);
    [DllImport("user32.dll")] public static extern bool CloseDesktop(IntPtr hDesktop);
    [DllImport("kernel32.dll")] public static extern bool CreateProcess(string lpAppName, string lpCmdLine, IntPtr lpPA, IntPtr lpTA, bool bInherit, uint dwFlags, IntPtr lpEnv, string lpCurDir, ref STARTUPINFO lpSI, out PROCESS_INFORMATION lpPI);
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Auto)] public struct STARTUPINFO { public int cb; public string lpReserved; public string lpDesktop; public string lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError; }
    [StructLayout(LayoutKind.Sequential)] public struct PROCESS_INFORMATION { public IntPtr hProcess, hThread; public int dwProcessId, dwThreadId; }
}
"@

$exe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option.exe"
$desktopName = "DINOForge_Agent_Test"

if (-not (Test-Path $exe)) {
    Write-Host "ERROR: TEST exe not found: $exe"
    exit 1
}

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Creating hidden desktop '$desktopName'..."
$desktop = [Win32Desktop]::CreateDesktop($desktopName, [IntPtr]::Zero, [IntPtr]::Zero, 0, 0x01FF, [IntPtr]::Zero)
if ($desktop -eq [IntPtr]::Zero) {
    Write-Host "ERROR: CreateDesktop failed"
    exit 1
}
Write-Host "Desktop created: $desktop"

$si = New-Object Win32Desktop+STARTUPINFO
$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($si)
$si.lpDesktop = $desktopName
$si.dwFlags = 0x00000001
$si.wShowWindow = 0

$pi = New-Object Win32Desktop+PROCESS_INFORMATION

$cmdLine = "`"$exe`" -popupwindow"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Launching: $exe"
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Desktop: $desktopName"

$exeDir = Split-Path $exe -Parent
# lpAppName=exe means Windows uses it directly; lpCmdLine just the args
$cmdLineArgs = [System.Text.StringBuilder]::new("-popupwindow")
$ok = [Win32Desktop]::CreateProcess($exe, $cmdLineArgs, [IntPtr]::Zero, [IntPtr]::Zero, $false, 0x00000010, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi)

if ($ok) {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] SUCCESS - PID: $($pi.dwProcessId)"
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] TEST instance running on hidden desktop '$desktopName'"
} else {
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] ERROR: CreateProcess failed"
}
