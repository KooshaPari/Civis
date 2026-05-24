#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Run all Pester unit tests under tests/unit (Windows-safe discovery).

.DESCRIPTION
    PowerShell does not expand globs when passed to Invoke-Pester, so this script
    discovers tests/unit/*.ps1 via Get-ChildItem and invokes Pester on the full paths.

    Requires Pester 3.x (tests use Pester 3 Should syntax). Install with:
      Install-Module Pester -RequiredVersion 3.4.0 -Scope CurrentUser -Force

.EXAMPLE
    pwsh -File scripts/qa/run-unit-pester.ps1
#>
#Requires -Version 5.1

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

if (-not (Get-Module Pester -ListAvailable | Where-Object { $_.Version.Major -lt 4 })) {
    throw @'
Pester 3.x is required (unit tests use legacy Should syntax).
Install: Install-Module Pester -RequiredVersion 3.4.0 -Scope CurrentUser -Force
'@
}

Import-Module Pester -MaximumVersion 3.99.99 -Force

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
Set-Location $repoRoot

$unitDir = Join-Path $repoRoot 'tests\unit'
$testScripts = @(Get-ChildItem -Path $unitDir -Filter '*.ps1' -File | Sort-Object Name)

if ($testScripts.Count -eq 0) {
    throw "No unit test scripts found under $unitDir"
}

Write-Host "Running $($testScripts.Count) Pester script(s) from tests/unit..." -ForegroundColor Cyan
$testScripts | ForEach-Object { Write-Host "  $($_.Name)" }

$result = Invoke-Pester -Path $testScripts.FullName -PassThru

$failed = $result.FailedCount
$passed = $result.PassedCount
$total = $result.TotalCount

if ($total -eq 0) {
    throw 'Pester discovered zero tests (check script paths and Pester version)'
}

Write-Host "Pester: $passed passed, $failed failed, $total total" -ForegroundColor $(if ($failed -gt 0) { 'Red' } else { 'Green' })

if ($failed -gt 0) {
    exit 1
}

exit 0
