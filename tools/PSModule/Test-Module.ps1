#!/usr/bin/env pwsh
# Quick test of DINOForge module syntax

$modulePath = $PSScriptRoot
Write-Host "Testing module at: $modulePath"

# Try parsing
try {
    $content = Get-Content (Join-Path $modulePath "DINOForge.psm1") -Raw
    $tokens = [System.Management.Automation.PSParser]::Tokenize($content, [ref]$null)
    Write-Host "✓ File parses successfully"
    Write-Host "  Total tokens: $($tokens.Count)"
} catch {
    Write-Error "Parse error: $_"
    exit 1
}

# Try importing
try {
    Import-Module (Join-Path $modulePath "DINOForge.psm1") -Force
    Write-Host "✓ Module imported successfully"
} catch {
    Write-Error "Import error: $_"
    exit 1
}

# Check exported functions
$commands = Get-Command -Module DINOForge
Write-Host "✓ Exported commands: $($commands.Count)"
$commands | ForEach-Object { Write-Host "  - $($_.Name)" }

Write-Host ""
Write-Host "All tests passed!" -ForegroundColor Green
