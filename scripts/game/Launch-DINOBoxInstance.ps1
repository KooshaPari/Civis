<#
.SYNOPSIS
    Launch a single DINOBox instance with optional hidden desktop isolation.

.DESCRIPTION
    Launches a game instance from a DINOBox with support for:
    - Isolated execution via Win32 CreateDesktop (hidden launch)
    - Configurable timeout for startup verification
    - Pipe name isolation validation

.PARAMETER BoxPath
    Root path to the DINOBox instance.

.PARAMETER PipeName
    Named pipe name for bridge communication (from pool).

.PARAMETER Hidden
    Launch on isolated hidden Win32 desktop (default: true).

.PARAMETER TimeoutSeconds
    Timeout for launch verification (default: 30).

.EXAMPLE
    $pool = .\New-DINOBoxPool.ps1 -Count 2
    $box = $pool[1]
    .\Launch-DINOBoxInstance.ps1 -BoxPath $box.BoxPath -PipeName $box.PipeName
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$BoxPath,

    [Parameter(Mandatory = $true)]
    [string]$PipeName,

    [switch]$Hidden = $true,

    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $BoxPath)) {
    throw "Box path not found: $BoxPath"
}

$gameExe = Join-Path $BoxPath "Diplomacy is Not an Option.exe"
if (-not (Test-Path $gameExe)) {
    throw "Game executable not found: $gameExe"
}

Write-Host "Launching DINOBox instance..."
Write-Host "  Box: $BoxPath"
Write-Host "  Pipe: $PipeName"
Write-Host "  Hidden: $Hidden"

# If hidden launch, use Win32 CreateDesktop approach
if ($Hidden) {
    Write-Host "  Using hidden desktop isolation..."

    # Use a minimal C# helper to CreateDesktop and launch
    $desktopName = "DINOBox_$(Get-Random)"

    $launcher = @"
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

public class DesktopLauncher
{
    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr CreateDesktop(string lpszDesktop, IntPtr lpszDevice, IntPtr pDevmode, uint dwFlags, uint dwDesiredAccess);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool CloseDesktop(IntPtr hDesktop);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr GetThreadDesktop(uint dwThreadId);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern uint GetCurrentThreadId();

    [DllImport("user32.dll", SetLastError = true)]
    private static extern bool SetThreadDesktop(IntPtr hDesktop);

    public static void Main(string[] args)
    {
        string exePath = args[0];
        string workDir = args[1];

        // Create hidden desktop
        IntPtr hDesktop = CreateDesktop("$desktopName", IntPtr.Zero, IntPtr.Zero, 0, 0x00000001 | 0x00000010);
        if (hDesktop == IntPtr.Zero)
        {
            Console.WriteLine("Failed to create desktop");
            return;
        }

        try
        {
            // Switch to new desktop
            uint threadId = GetCurrentThreadId();
            IntPtr oldDesktop = GetThreadDesktop(threadId);
            if (!SetThreadDesktop(hDesktop))
            {
                Console.WriteLine("Failed to set thread desktop");
                return;
            }

            // Launch game on this desktop
            ProcessStartInfo psi = new ProcessStartInfo
            {
                FileName = exePath,
                WorkingDirectory = workDir,
                UseShellExecute = false,
                CreateNoWindow = true
            };

            Process p = Process.Start(psi);
            if (p != null)
            {
                Console.WriteLine("Game started with PID: " + p.Id);
            }

            // Keep desktop alive
            System.Threading.Thread.Sleep(TimeSpan.FromSeconds(60));
        }
        finally
        {
            CloseDesktop(hDesktop);
        }
    }
}
"@

    # Compile and run helper
    $csharpFile = Join-Path $env:TEMP "DINOBoxLauncher_$([guid]::NewGuid().ToString().Substring(0,8)).cs"
    Set-Content -Path $csharpFile -Value $launcher
    try {
        & csc.exe $csharpFile 2>&1 | Out-Null
        $exeName = $csharpFile -replace "\.cs$", ".exe"
        & $exeName $gameExe $BoxPath
        Remove-Item $exeName -Force -ErrorAction SilentlyContinue # remove-item-ok: temp-cleanup-ok: ephemeral csc-compiled launcher binary, not a repo artifact
    } finally {
        Remove-Item $csharpFile -Force -ErrorAction SilentlyContinue # remove-item-ok: temp-cleanup-ok: ephemeral csc source temp file, not a repo artifact
    }
} else {
    # Normal launch
    $proc = Start-Process -FilePath $gameExe -WorkingDirectory $BoxPath -PassThru
    Write-Host "  Game process started (PID: $($proc.Id))"
}

# Wait for game to be ready (poll for pipe availability)
Write-Host "Waiting for bridge to be ready..."

$startTime = Get-Date
$pipeReady = $false

while ((Get-Date) - $startTime -lt (New-TimeSpan -Seconds $TimeoutSeconds)) {
    try {
        $pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", $PipeName, [System.IO.Pipes.PipeDirection]::InOut, [System.IO.Pipes.PipeOptions]::Asynchronous)
        $pipe.Connect(100)
        $pipe.Close()
        $pipeReady = $true
        Write-Host "[OK] Bridge pipe is ready"
        break
    } catch {
        Start-Sleep -Milliseconds 500
    }
}

if (-not $pipeReady) {
    Write-Host "[WARN] Bridge pipe not ready after $TimeoutSeconds seconds (game may still be initializing)"
}

Write-Host "[OK] Launch complete"
