<#
.SYNOPSIS
    Build and launch the Civis Bevy standalone client.

.DESCRIPTION
    Production-grade launcher for `just play`. Kills any existing
    civ-standalone process, builds the release binary, launches it
    detached, and tails the log to stdout.

.PARAMETER Profile
    Cargo profile: 'release' (default) or 'debug'.

.PARAMETER LogLevel
    RUST_LOG value. Default: 'info'.

.PARAMETER NoTail
    If set, returns immediately after launch without tailing.

.EXAMPLE
    pwsh Tools/play.ps1
    pwsh Tools/play.ps1 -Profile debug -LogLevel 'info,civ_bevy_ref=debug'
#>
[CmdletBinding()]
param(
    [ValidateSet('release', 'debug')]
    [string]$Profile = 'release',

    [string]$LogLevel = 'info',

    [switch]$NoTail
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDir = Join-Path $RepoRoot '.process-compose/logs'
$PidDir = Join-Path $RepoRoot '.process-compose/pids'
$LogFile = Join-Path $LogDir 'civ-standalone.log'
$ErrFile = Join-Path $LogDir 'civ-standalone.err.log'
$PidFile = Join-Path $PidDir 'civ-standalone.pid'

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
$stale = Get-Process -Name 'civ-standalone' -ErrorAction SilentlyContinue
if ($stale) {
    $stale | ForEach-Object {
        try {
            Stop-Process -Id $_.Id -Force -ErrorAction Stop
            Write-Ok "  killed pid $($_.Id)"
        }
        catch {
            Write-Err "  failed to kill pid $($_.Id): $_"
        }
    }
    Start-Sleep -Milliseconds 500
} else {
    Write-Ok "  none running"
}

if (Test-Path $PidFile) {
    $oldPid = Get-Content $PidFile -ErrorAction SilentlyContinue
    if ($oldPid) {
        $oldProc = Get-Process -Id $oldPid -ErrorAction SilentlyContinue
        if ($oldProc) {
            Stop-Process -Id $oldPid -Force -ErrorAction SilentlyContinue
            Write-Ok "  killed tracked pid $oldPid"
        }
    }
    Remove-Item $PidFile -Force -ErrorAction SilentlyContinue
}

# --- 2. Build ---
$profileFlag = if ($Profile -eq 'release') { '--release' } else { '' }
$targetDir = Join-Path $RepoRoot "target/$Profile"
$exePath = Join-Path $targetDir 'civ-standalone.exe'

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

if (-not (Test-Path $exePath)) {
    Write-Err "Expected binary not found: $exePath"
    exit 1
}
Write-Ok "Built: $exePath"

# --- 3. Launch detached, redirect logs ---
Write-Step "Launching civ-standalone (RUST_LOG=$LogLevel)..."

if (Test-Path $LogFile) { Clear-Content $LogFile }
if (Test-Path $ErrFile) { Clear-Content $ErrFile }

$env:RUST_LOG = $LogLevel
$env:RUST_BACKTRACE = '1'

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $exePath
$startInfo.WorkingDirectory = $RepoRoot
$startInfo.UseShellExecute = $false
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$startInfo.CreateNoWindow = $false
$startInfo.EnvironmentVariables['RUST_LOG'] = $LogLevel
$startInfo.EnvironmentVariables['RUST_BACKTRACE'] = '1'

$proc = [System.Diagnostics.Process]::new()
$proc.StartInfo = $startInfo
$proc.EnableRaisingEvents = $true

# Async stream copy so the buffers don't deadlock.
$outAction = {
    if ($EventArgs.Data -ne $null) {
        Add-Content -Path $Event.MessageData -Value $EventArgs.Data
    }
}
Register-ObjectEvent -InputObject $proc -EventName OutputDataReceived `
    -Action $outAction -MessageData $LogFile | Out-Null
Register-ObjectEvent -InputObject $proc -EventName ErrorDataReceived `
    -Action $outAction -MessageData $ErrFile | Out-Null

[void]$proc.Start()
$proc.BeginOutputReadLine()
$proc.BeginErrorReadLine()

Set-Content -Path $PidFile -Value $proc.Id
Write-Ok "Launched pid $($proc.Id) -> $LogFile"

if ($NoTail) {
    exit 0
}

# --- 4. Tail until process exits or user Ctrl+C ---
Write-Step "Tailing log (Ctrl+C to detach; game keeps running)..."
Write-Host ""

$tailJob = Start-Job -ArgumentList $LogFile, $ErrFile -ScriptBlock {
    param($Out, $Err)
    # Wait for files to exist
    while (-not (Test-Path $Out)) { Start-Sleep -Milliseconds 100 }
    Get-Content -Path $Out, $Err -Wait -Tail 0
}

try {
    while (-not $proc.HasExited) {
        Receive-Job -Job $tailJob | ForEach-Object { Write-Host $_ }
        Start-Sleep -Milliseconds 200
    }
    # Drain final output
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
    Stop-Job -Job $tailJob -ErrorAction SilentlyContinue
    Remove-Job -Job $tailJob -Force -ErrorAction SilentlyContinue
    Get-EventSubscriber | Where-Object { $_.SourceObject -eq $proc } |
        Unregister-Event -ErrorAction SilentlyContinue
}
