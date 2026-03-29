#!/usr/bin/env pwsh
<#
Test script to verify asset pipeline CLI commands are properly registered.
#>

$PackCompilerExe = "C:\Users\koosh\Dino\src\Tools\PackCompiler\bin\Release\net11.0\DINOForge.Tools.PackCompiler.exe"

Write-Host "Testing PackCompiler asset commands..." -ForegroundColor Cyan

# Test 1: Show help for assets command
Write-Host "`nTest 1: assets --help" -ForegroundColor Yellow
& $PackCompilerExe assets --help 2>&1 | head -20

# Test 2: Show help for bundles command
Write-Host "`nTest 2: bundles --help" -ForegroundColor Yellow
& $PackCompilerExe bundles --help 2>&1 | head -20

# Test 3: Show root help
Write-Host "`nTest 3: --help (root)" -ForegroundColor Yellow
& $PackCompilerExe --help 2>&1 | head -30

Write-Host "`nDone!" -ForegroundColor Green
