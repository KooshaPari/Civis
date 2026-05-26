#!/usr/bin/env pwsh
# DINOForge CLI Test Script
# Verifies that all CLI commands work correctly

param(
    [switch]$Verbose
)

$ErrorActionPreference = 'Stop'

Write-Host "╔════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║        DINOForge CLI Test Suite                        ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$tests = @(
    @{ Name = "Help"; Command = @("dotnet", "run", "--project", "src/Tools/PackCompiler", "--", "--help") },
    @{ Name = "Validate Pack"; Command = @("dotnet", "run", "--project", "src/Tools/PackCompiler", "--", "validate", "packs/warfare-starwars") },
    @{ Name = "Unit Tests"; Command = @("dotnet", "test", "src/DINOForge.sln") }
)

$passed = 0
$failed = 0

foreach ($test in $tests) {
    Write-Host "Testing: $($test.Name)" -ForegroundColor Yellow
    $start = Get-Date
    try {
        $output = & $test.Command[0] ($test.Command | Select-Object -Skip 1) 2>&1
        $elapsed = (Get-Date) - $start
        Write-Host "  ✅ PASS ($($elapsed.TotalSeconds.ToString('F2'))s)" -ForegroundColor Green
        $passed++
    }
    catch {
        Write-Host "  ❌ FAIL" -ForegroundColor Red
        if ($Verbose) {
            Write-Host "  Error: $_" -ForegroundColor Red
        }
        $failed++
    }
    Write-Host ""
}

Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Results: $passed passed, $failed failed" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Red" })
Write-Host "════════════════════════════════════════════════════════" -ForegroundColor Cyan

exit $(if ($failed -eq 0) { 0 } else { 1 })
