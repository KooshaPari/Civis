<#
.SYNOPSIS
    Start the playCUA bare-cua-native server for DINOForge isolated game capture.

.DESCRIPTION
    Wraps the freshly-built playCUA binary at C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe.
    Starts the JSON-RPC server (default 127.0.0.1:9000) so that DINOForge's PlayCUABackend
    can route game_launch / game_screenshot calls through it.

    Pre-flight checks the binary exists and that no existing process is already bound to the port.
    On success, prints PID=<n> for capture by callers.

.PARAMETER Listen
    The address:port to bind. Defaults to 127.0.0.1:9000 — the address PlayCUABackend expects.

.PARAMETER Foreground
    Run the binary in the current console instead of as a background process.

.EXAMPLE
    pwsh scripts/start-playcua.ps1
    Starts the server in the background on the default port and prints the PID.

.EXAMPLE
    pwsh scripts/start-playcua.ps1 -Foreground
    Starts the server attached to the current console.
#>
[CmdletBinding()]
param(
    [string]$Listen = "127.0.0.1:9000",
    [switch]$Foreground
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$BinaryPath = "C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe"

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "playCUA binary not found at expected path: $BinaryPath. Build first: cd C:\Users\koosh\playcua_ci_test\native; cargo build --release. See docs/TRUTH_TABLE.md."
}

$parts = $Listen -split ":"
if ($parts.Count -ne 2) {
    throw "Listen must be host:port (got '$Listen')."
}
$portInt = [int]$parts[1]

$existing = Get-NetTCPConnection -State Listen -LocalPort $portInt -ErrorAction SilentlyContinue
if ($existing) {
    $pidExisting = $existing | Select-Object -First 1 -ExpandProperty OwningProcess
    Write-Host "playCUA already running on port $portInt (PID=$pidExisting). Reusing."
    Write-Output "PID=$pidExisting"
    return
}

if ($Foreground) {
    Write-Host "Starting playCUA in foreground on $Listen"
    & $BinaryPath --listen $Listen
    return
}

$proc = Start-Process -FilePath $BinaryPath -ArgumentList @("--listen", $Listen) -PassThru -WindowStyle Hidden

$deadline = (Get-Date).AddSeconds(5)
while ((Get-Date) -lt $deadline) {
    $bound = Get-NetTCPConnection -State Listen -LocalPort $portInt -ErrorAction SilentlyContinue
    if ($bound) { break }
    Start-Sleep -Milliseconds 200
}

$bound = Get-NetTCPConnection -State Listen -LocalPort $portInt -ErrorAction SilentlyContinue
if (-not $bound) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    throw "playCUA failed to bind $Listen within 5 seconds. Started PID was $($proc.Id) (now killed)."
}

Write-Host "Started playCUA on $Listen (PID=$($proc.Id))."
Write-Output "PID=$($proc.Id)"
