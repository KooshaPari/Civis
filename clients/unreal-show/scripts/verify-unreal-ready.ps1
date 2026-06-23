#Requires -Version 5.1
<#
.SYNOPSIS
  Pre-flight check for CivShow before UE finishes downloading (no UBT required).
#>
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$fail = 0

function Check([string] $Label, [bool] $Ok) {
    if ($Ok) {
        Write-Host "[ok] $Label" -ForegroundColor Green
    }
    else {
        Write-Host "[!!] $Label" -ForegroundColor Red
        $script:fail++
    }
}

$required = @(
    'CivShow.uproject',
    'Source\CivShow\CivShow.cpp',
    'Source\CivShow\CivShow.h',
    'Source\CivShow.Target.cs',
    'Source\CivShowEditor.Target.cs',
    'Source\CivShow\CivWsClient.cpp',
    'Source\CivShow\CivShowGameMode.cpp',
    'Source\Civis\rust-shim\Cargo.toml',
    'Source\Civis\lib\civis_unreal_ffi.lib'
)

foreach ($rel in $required) {
    Check $rel (Test-Path (Join-Path $Root $rel))
}

& (Join-Path $PSScriptRoot 'build.ps1') -SkipUe | Out-Host
Check 'rust-shim build.ps1 -SkipUe' ($LASTEXITCODE -eq 0)

& (Join-Path $PSScriptRoot 'detect-ue.ps1') | Out-Host
if ($LASTEXITCODE -eq 0) {
    Write-Host '[ok] UE 5.4 detected — run build.ps1 without -SkipUe' -ForegroundColor Green
}
else {
    Write-Host '[..] UE not installed yet (expected while downloading)' -ForegroundColor Yellow
}

if ($fail -gt 0) {
    Write-Host "verify-unreal-ready: $fail check(s) failed" -ForegroundColor Red
    exit 1
}
Write-Host 'verify-unreal-ready: all offline checks passed' -ForegroundColor Cyan
exit 0
