param(
    [Parameter(Mandatory=$true)][int]$ProcessId,
    [Parameter(Mandatory=$true)][string]$OutPath
)
# MiniDumpWriteDump P/Invoke — full-memory dump, self-owned file handle/DACL.
$src = @"
using System;
using System.Runtime.InteropServices;
public static class MiniDumper {
    [DllImport("dbghelp.dll", SetLastError=true)]
    public static extern bool MiniDumpWriteDump(
        IntPtr hProcess, uint pid, IntPtr hFile, int dumpType,
        IntPtr exceptionParam, IntPtr userStreamParam, IntPtr callbackParam);
}
"@
Add-Type -TypeDefinition $src -ErrorAction SilentlyContinue
$proc = Get-Process -Id $ProcessId
# MiniDumpWithFullMemory(0x2) | WithHandleData(0x4) | WithThreadInfo(0x1000) | WithFullMemoryInfo(0x800)
$dumpType = 0x2 -bor 0x4 -bor 0x1000 -bor 0x800
$fs = [System.IO.File]::Create($OutPath)
try {
    $ok = [MiniDumper]::MiniDumpWriteDump($proc.Handle, [uint32]$ProcessId, $fs.SafeFileHandle.DangerousGetHandle(), $dumpType, [IntPtr]::Zero, [IntPtr]::Zero, [IntPtr]::Zero)
    if (-not $ok) { $err = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error(); throw "MiniDumpWriteDump failed, Win32 error $err" }
} finally { $fs.Close() }
$mb = [math]::Round((Get-Item $OutPath).Length/1MB,1)
Write-Output "DUMP_OK $OutPath ${mb}MB"
