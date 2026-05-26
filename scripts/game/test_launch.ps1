$ErrorActionPreference = "Stop"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

# Launch TEST instance on hidden desktop
$testExe = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\Diplomacy is Not an Option.exe"
if (-not (Test-Path $testExe)) {
    Write-Host "ERROR: TEST exe not found: $testExe"
    exit 1
}

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Launching TEST instance on hidden desktop..."

$psScript = @"
param(`$ExePath, `$DesktopName)
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
`$desktop = [Win32Desktop]::CreateDesktop(`$DesktopName, [IntPtr]::Zero, [IntPtr]::Zero, 0, 0x01FF, [IntPtr]::Zero)
if (`$desktop -eq [IntPtr]::Zero) { Write-Output "ERROR: CreateDesktop failed"; exit 1 }
`$si = New-Object Win32Desktop+STARTUPINFO
`$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf(`$si)
`$si.lpDesktop = `$DesktopName
`$si.dwFlags = 0x00000001
`$si.wShowWindow = 0
`$pi = New-Object Win32Desktop+PROCESS_INFORMATION
`$exeDir = Split-Path `$ExePath -Parent
`$cmdLine = "`$ExePath -popupwindow"
`$ok = [Win32Desktop]::CreateProcess(`$ExePath, `$cmdLine, [IntPtr]::Zero, [IntPtr]::Zero, `$false, 0x00000010, [IntPtr]::Zero, `$exeDir, [ref]`$si, [ref]`$pi)
if (`$ok) { Write-Output "PID:`$(`$pi.dwProcessId)" } else { Write-Output "ERROR: CreateProcess failed" }
"@

$result = Start-Process -FilePath "powershell" -ArgumentList "-ExecutionPolicy", "Bypass", "-Command", "`$psScript = @'`r`n$psScript`r`n'@; & { `$psScript } -ExePath '$testExe' -DesktopName 'DINOForge_Agent_Test'" -NoNewWindow -Wait -PassThru
Write-Host "Result: $($result.ExitCode)"
