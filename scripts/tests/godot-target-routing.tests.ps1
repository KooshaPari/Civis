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
# Recipe body may be platform-guarded (os_family() conditional). Collect all
# body lines so the Windows powershell shape is still observable even when the
# recipe is split across an `if/else` branch.
$recipeBodyLines = @()
for ($i = $recipeIndex + 1; $i -lt $justLines.Count; $i++) {
    $line = $justLines[$i]
    if ($line -match '^[^\s]' -and $line -notmatch '^#') { break }
    $recipeBodyLines += $line
}
$recipeBody = $recipeBodyLines -join "`n"
# Source-level assertions: the raw justfile must declare both branches and the
# unix branch must wire bash-style CARGO_TARGET_DIR defaulting so a caller can
# still override it.
Assert-True ($recipeBody -match 'os_family\(\)\s*==\s*"windows"') 'recipe is not platform-guarded'
Assert-True ($recipeBody -match '\$env:CARGO_TARGET_DIR') 'recipe does not inspect CARGO_TARGET_DIR'
Assert-True ($recipeBody -match 'target-godot-smoke') 'recipe fallback is missing'
Assert-True ($recipeBody -match 'exit \$LASTEXITCODE') 'recipe does not propagate Cargo exit code'
$unixMatch = [regex]::Match($recipeBody, 'CARGO_TARGET_DIR=\\"\$\{CARGO_TARGET_DIR:-target-godot-smoke\}\\"\s+cargo\s+test\s+--manifest-path\s+clients/godot-ref/rust/Cargo\.toml\s+-j\s+1')
Assert-True $unixMatch.Success 'godot-test recipe unix branch missing or changed'
# Rendered-output assertion: ask `just -n` to print the evaluated recipe body
# for the current platform. On Windows, the rendered line must still match the
# single-line powershell contract so the synthetic cargo shim below can observe
# CARGO_TARGET_DIR propagation end-to-end. `--show` only prints the raw source
# (still escaped); -n is what the shell actually sees.
$justExe = (Get-Command 'just' -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1)
if (-not $justExe) { $justExe = Get-Item 'C:\Users\koosh\agents\sandbox\tools\just-1.58.0\bin\just.exe' -ErrorAction SilentlyContinue }
Assert-True ($null -ne $justExe) 'just binary not found for rendered-recipe check'
$justPath = if ($justExe -is [System.Management.Automation.ApplicationInfo]) { $justExe.Path } else { $justExe.FullName }
$rendered = (& $justPath -f (Join-Path $repoRoot 'justfile') -n godot-test 2>&1) -join "`n"
$renderedMatch = [regex]::Match($rendered, "powershell\s+-NoProfile\s+-ExecutionPolicy\s+Bypass\s+-Command\s+'(?<command>[^']*)'")
Assert-True $renderedMatch.Success 'rendered godot-test recipe does not match Windows powershell contract'
$command = $renderedMatch.Groups['command'].Value
Assert-True ($command -match '\$env:CARGO_TARGET_DIR') 'rendered recipe does not inspect CARGO_TARGET_DIR'
Assert-True ($command -match 'target-godot-smoke') 'rendered recipe fallback is missing'
Assert-True ($command -match 'exit \$LASTEXITCODE') 'rendered recipe does not propagate Cargo exit code'

$powershell = (Get-Command powershell.exe -CommandType Application -ErrorAction Stop | Select-Object -First 1).Path
$caseRoot = Join-Path (Join-Path ([Environment]::GetFolderPath('UserProfile')) 'agents\sandbox') ("godot-target-routing-" + [guid]::NewGuid().ToString('N'))
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
