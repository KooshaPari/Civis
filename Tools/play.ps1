<#
.SYNOPSIS
    Build and launch the Civis Bevy standalone client.

.DESCRIPTION
    Production-grade launcher for `just play`. Kills any existing
    civ-standalone process, builds the release binary, launches it
    detached (stderr -> .process-compose/logs/civ-standalone.log),
    prints PID + "Game ready", then optionally tails the log.

.PARAMETER Profile
    Cargo profile: 'release' (default) or 'debug'.

.PARAMETER LogLevel
    RUST_LOG value. Default: 'info'.

.PARAMETER Backtrace
    RUST_BACKTRACE value: '1' (default) or 'full'.

.PARAMETER NoTail
    If set, returns immediately after launch without tailing.

.EXAMPLE
    pwsh Tools/play.ps1
    pwsh Tools/play.ps1 -Profile debug -LogLevel 'info,civ_bevy_ref=debug'
    pwsh Tools/play.ps1 -LogLevel 'info,civ_bevy_ref=debug,wgpu=warn' -Backtrace full
#>
[CmdletBinding()]
param(
    [ValidateSet('release', 'debug')]
    [string]$Profile = 'release',

    [string]$LogLevel = 'info',

    [ValidateSet('1', 'full')]
    [string]$Backtrace = '1',

    [switch]$NoTail
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Resolve repo root robustly — handles paths with spaces on any drive.
$RepoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$LogDir   = [System.IO.Path]::Combine($RepoRoot, '.process-compose', 'logs')
$PidDir   = [System.IO.Path]::Combine($RepoRoot, '.process-compose', 'pids')
$LogFile  = [System.IO.Path]::Combine($LogDir, 'civ-standalone.log')
$ErrFile  = [System.IO.Path]::Combine($LogDir, 'civ-standalone.err.log')
$PidFile  = [System.IO.Path]::Combine($PidDir, 'civ-standalone.pid')

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
New-Item -ItemType Directory -Force -Path $PidDir | Out-Null

function Write-Step([string]$Message) {
    Write-Host "[play] $Message" -ForegroundColor Cyan
}

function Write-Ok([string]$Message) {
    Write-Host "[play] $Message" -ForegroundColor Green
}

function Write-Err([string]$Message) {
    Write-Host "[play] $Message" -ForegroundColor Red
}

# --- 1. Kill stale civ-standalone processes ---
Write-Step "Killing any stale civ-standalone processes..."
# -ErrorAction SilentlyContinue: Get-Process emits a non-terminating error when no match;
# suppress it so "process not found" is silent and non-erroring.
$stale = @(Get-Process -Name 'civ-standalone' -ErrorAction SilentlyContinue)
if ($stale.Count -gt 0) {
    foreach ($p in $stale) {
        try {
            Stop-Process -Id $p.Id -Force -ErrorAction Stop
            Write-Ok "  killed pid $($p.Id)"
        }
        catch {
            # Process may have exited between enumeration and kill — not fatal.
            Write-Ok "  pid $($p.Id) already gone"
        }
    }
    Start-Sleep -Milliseconds 500
} else {
    Write-Ok "  none running"
}

# Clean up any PID-file-tracked process from a previous session.
if (Test-Path -LiteralPath $PidFile) {
    $rawPid = ((Get-Content -LiteralPath $PidFile -ErrorAction SilentlyContinue) -join '').Trim()
    if ($rawPid -match '^\d+$') {
        $trackedProc = Get-Process -Id ([int]$rawPid) -ErrorAction SilentlyContinue
        if ($trackedProc) {
            Stop-Process -Id ([int]$rawPid) -Force -ErrorAction SilentlyContinue
            Write-Ok "  killed tracked pid $rawPid"
        }
    }
    Remove-Item -LiteralPath $PidFile -Force -ErrorAction SilentlyContinue
}

# --- 2. Build ---
# Use [System.IO.Path]::Combine so paths with spaces are handled correctly on all drives.
$targetDir = [System.IO.Path]::Combine($RepoRoot, 'target', $Profile)
$exePath   = [System.IO.Path]::Combine($targetDir, 'civ-standalone.exe')

Write-Step "Building civ-standalone ($Profile)..."
Push-Location $RepoRoot
try {
    $buildArgs = @(
        'build',
        '-p', 'civ-bevy-ref',
        '--features', 'bevy,egui',
        '--bin', 'civ-standalone'
    )
    if ($Profile -eq 'release') { $buildArgs += '--release' }

    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Err "cargo build failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $exePath)) {
    Write-Err "Expected binary not found: $exePath"
    exit 1
}
Write-Ok "Built: $exePath"

# --- 3. Launch detached, redirect logs ---
Write-Step "Launching civ-standalone (RUST_LOG=$LogLevel, RUST_BACKTRACE=$Backtrace)..."

if (Test-Path -LiteralPath $LogFile) { Clear-Content -LiteralPath $LogFile }
if (Test-Path -LiteralPath $ErrFile) { Clear-Content -LiteralPath $ErrFile }

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $exePath
$startInfo.WorkingDirectory = $RepoRoot
$startInfo.UseShellExecute = $false
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$startInfo.CreateNoWindow = $false
$startInfo.EnvironmentVariables['RUST_LOG'] = $LogLevel
$startInfo.EnvironmentVariables['RUST_BACKTRACE'] = $Backtrace

$proc = [System.Diagnostics.Process]::new()
$proc.StartInfo = $startInfo
$proc.EnableRaisingEvents = $true

# Async stream copy so the buffers don't deadlock.
$outAction = {
    if ($null -ne $EventArgs.Data) {
        Add-Content -LiteralPath $Event.MessageData -Value $EventArgs.Data
    }
}
Register-ObjectEvent -InputObject $proc -EventName OutputDataReceived `
    -Action $outAction -MessageData $LogFile | Out-Null
Register-ObjectEvent -InputObject $proc -EventName ErrorDataReceived `
    -Action $outAction -MessageData $ErrFile | Out-Null

[void]$proc.Start()
$proc.BeginOutputReadLine()
$proc.BeginErrorReadLine()

Set-Content -LiteralPath $PidFile -Value $proc.Id
Write-Ok "Launched pid $($proc.Id) -> $LogFile"
Write-Host "[play] Game ready (pid $($proc.Id))." -ForegroundColor Green

if ($NoTail) {
    exit 0
}

# --- 4. Tail until process exits or user Ctrl+C ---
Write-Step "Tailing log (Ctrl+C to detach; game keeps running)..."
Write-Host ""

$tailJob = Start-Job -ArgumentList $LogFile -ScriptBlock {
    param($Out)
    # Wait up to 10 s for the log file to appear.
    $deadline = (Get-Date).AddSeconds(10)
    while ((-not (Test-Path -LiteralPath $Out)) -and ((Get-Date) -lt $deadline)) {
        Start-Sleep -Milliseconds 100
    }
    if (Test-Path -LiteralPath $Out) {
        Get-Content -LiteralPath $Out -Wait -Tail 0
    }
}

try {
    while (-not $proc.HasExited) {
        Receive-Job -Job $tailJob | ForEach-Object { Write-Host $_ }
        Start-Sleep -Milliseconds 200
    }
    # Drain final output.
    Receive-Job -Job $tailJob | ForEach-Object { Write-Host $_ }

    Write-Host ""
    if ($proc.ExitCode -eq 0) {
        Write-Ok "civ-standalone exited cleanly."
    } else {
        Write-Err "civ-standalone exited with code $($proc.ExitCode)."
    }
    exit $proc.ExitCode
}
finally {
    Stop-Job  -Job $tailJob -ErrorAction SilentlyContinue
    Remove-Job -Job $tailJob -Force -ErrorAction SilentlyContinue
    Get-EventSubscriber | Where-Object { $_.SourceObject -eq $proc } |
        Unregister-Event -ErrorAction SilentlyContinue
}
