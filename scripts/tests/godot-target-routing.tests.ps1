#requires -Version 7.0
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

function Assert-True {
    param(
        [Parameter(Mandatory)][bool]$Condition,
        [Parameter(Mandatory)][string]$Message
    )
    if (-not $Condition) {
        throw "ASSERTION FAILED: $Message"
    }
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$justLines = Get-Content -LiteralPath (Join-Path $repoRoot 'justfile')
$recipeIndex = [Array]::IndexOf($justLines, 'godot-test:')
Assert-True ($recipeIndex -ge 0 -and $recipeIndex + 1 -lt $justLines.Count) 'godot-test recipe missing'
$recipeLine = $justLines[$recipeIndex + 1]
$recipeMatch = [regex]::Match($recipeLine, "^\s*powershell\s+-NoProfile\s+-ExecutionPolicy\s+Bypass\s+-Command\s+'(?<command>.*)'\s*$")
Assert-True $recipeMatch.Success 'godot-test recipe command shape changed unexpectedly'
$command = $recipeMatch.Groups['command'].Value
Assert-True ($command -match '\$env:CARGO_TARGET_DIR') 'recipe does not inspect CARGO_TARGET_DIR'
Assert-True ($command -match 'target-godot-smoke') 'recipe fallback is missing'
Assert-True ($command -match 'exit \$LASTEXITCODE') 'recipe does not propagate Cargo exit code'

$powershell = (Get-Command powershell.exe -CommandType Application -ErrorAction Stop | Select-Object -First 1).Path
$caseRoot = Join-Path (Join-Path $repoRoot '..\agents\sandbox') ("godot-target-routing-" + [guid]::NewGuid().ToString('N'))
$shimRoot = Join-Path $caseRoot 'shim'
$explicitLogPath = Join-Path $caseRoot 'explicit.log'
$fallbackLogPath = Join-Path $caseRoot 'fallback.log'
$failureLogPath = Join-Path $caseRoot 'failure.log'
$null = New-Item -ItemType Directory -Path $shimRoot -Force
$shimPath = Join-Path $shimRoot 'cargo.cmd'
Set-Content -LiteralPath $shimPath -Encoding ascii -Value @(
    '@echo off',
    'echo TARGET=%CARGO_TARGET_DIR%> "%CARGO_TARGET_ROUTING_LOG%"',
    'exit /b %FAKE_CARGO_EXIT%'
)

$oldTarget = $env:CARGO_TARGET_DIR
$oldLog = $env:CARGO_TARGET_ROUTING_LOG
$oldExit = $env:FAKE_CARGO_EXIT
$oldPath = $env:PATH

function Restore-TestEnvironment {
    if ($null -eq $oldTarget) { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue } else { $env:CARGO_TARGET_DIR = $oldTarget }
    if ($null -eq $oldLog) { Remove-Item Env:CARGO_TARGET_ROUTING_LOG -ErrorAction SilentlyContinue } else { $env:CARGO_TARGET_ROUTING_LOG = $oldLog }
    if ($null -eq $oldExit) { Remove-Item Env:FAKE_CARGO_EXIT -ErrorAction SilentlyContinue } else { $env:FAKE_CARGO_EXIT = $oldExit }
    $env:PATH = $oldPath
}

try {
    $env:PATH = "$shimRoot$([IO.Path]::PathSeparator)$oldPath"

    $env:CARGO_TARGET_DIR = 'G:\synthetic godot target'
    $env:FAKE_CARGO_EXIT = '0'
    $env:CARGO_TARGET_ROUTING_LOG = $explicitLogPath
    & $powershell -NoProfile -ExecutionPolicy Bypass -Command $command
    $explicitCode = $LASTEXITCODE
    $explicitLog = Get-Content -LiteralPath $explicitLogPath -Raw
    Assert-True ($explicitCode -eq 0) "explicit-target synthetic Cargo exit code was $explicitCode"
    Assert-True ($explicitLog.Trim() -eq 'TARGET=G:\synthetic godot target') "explicit target was not preserved: $explicitLog"

    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
    $env:CARGO_TARGET_ROUTING_LOG = $fallbackLogPath
    & $powershell -NoProfile -ExecutionPolicy Bypass -Command $command
    $fallbackCode = $LASTEXITCODE
    $fallbackLog = Get-Content -LiteralPath $fallbackLogPath -Raw
    Assert-True ($fallbackCode -eq 0) "fallback synthetic Cargo exit code was $fallbackCode"
    Assert-True ($fallbackLog.Trim() -eq 'TARGET=target-godot-smoke') "fallback target was not applied: $fallbackLog"

    $env:CARGO_TARGET_DIR = 'G:\synthetic failure target'
    $env:FAKE_CARGO_EXIT = '23'
    $env:CARGO_TARGET_ROUTING_LOG = $failureLogPath
    & $powershell -NoProfile -ExecutionPolicy Bypass -Command $command
    $failureCode = $LASTEXITCODE
    $failureLog = Get-Content -LiteralPath $failureLogPath -Raw
    Assert-True ($failureCode -eq 23) "Cargo failure was not propagated; observed exit code $failureCode"
    Assert-True ($failureLog.Trim() -eq 'TARGET=G:\synthetic failure target') "failure case changed explicit target: $failureLog"

    Write-Output "godot target-routing synthetic tests passed; artifacts retained at $caseRoot"
}
finally {
    Restore-TestEnvironment
    Write-Output "synthetic case artifacts retained at $caseRoot"
}
