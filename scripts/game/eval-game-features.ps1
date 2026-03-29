#Requires -Version 5.1
<#
.SYNOPSIS
    DINOForge Extended Feature Evaluation Helper.

.DESCRIPTION
    Pre-flight checks and test compilation for the /eval-game-features command.
    Does NOT run the actual evaluation (that's orchestrated by Claude via MCP).
    Instead, this script:
    - Verifies MCP server health
    - Builds test projects to ensure no compilation errors
    - Prepares output directories
    - Provides instructions for running the full evaluation

.NOTES
    Run from repo root:
        powershell -ExecutionPolicy Bypass -File scripts/game/eval-game-features.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Configuration ──────────────────────────────────────────────────────────────
$repoRoot  = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$outDir    = "$env:TEMP\DINOForge\capture"
$proofDir  = "$repoRoot\docs\proof-of-features"
$testProj  = "$repoRoot\src\Tests\DINOForge.Tests.csproj"
$mcpUrl    = "http://127.0.0.1:8765/health"

Write-Host "DINOForge Extended Feature Evaluation — Pre-flight Check" -ForegroundColor Cyan
Write-Host "=========================================================" -ForegroundColor Cyan
Write-Host ""

# ── Step 1: Verify directories exist ───────────────────────────────────────────
Write-Host "Step 1: Preparing output directories..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
New-Item -ItemType Directory -Force -Path $proofDir | Out-Null
Write-Host "✓ Output directories ready: $outDir" -ForegroundColor Green
Write-Host ""

# ── Step 2: Check MCP server health ────────────────────────────────────────────
Write-Host "Step 2: Checking MCP server health..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri $mcpUrl -TimeoutSec 3 -ErrorAction Stop
    Write-Host "✓ MCP server is running on http://127.0.0.1:8765" -ForegroundColor Green
} catch {
    Write-Host "✗ MCP server not responding on $mcpUrl" -ForegroundColor Red
    Write-Host ""
    Write-Host "To start MCP server:" -ForegroundColor Yellow
    Write-Host "  cd src/Tools/DinoforgeMcp"
    Write-Host "  python -m dinoforge_mcp.server"
    Write-Host ""
    exit 1
}
Write-Host ""

# ── Step 3: Build test projects ────────────────────────────────────────────────
Write-Host "Step 3: Building test projects..." -ForegroundColor Yellow
try {
    Push-Location $repoRoot
    dotnet build $testProj -c Release --verbosity minimal | Out-Null
    Write-Host "✓ Test projects compiled successfully" -ForegroundColor Green
    Pop-Location
} catch {
    Write-Host "✗ Test project build failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Run full build to diagnose:" -ForegroundColor Yellow
    Write-Host "  dotnet build src/DINOForge.sln"
    Write-Host ""
    exit 1
}
Write-Host ""

# ── Step 4: Verify game is running ─────────────────────────────────────────────
Write-Host "Step 4: Checking if game is running..." -ForegroundColor Yellow
$gameProc = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
if ($null -eq $gameProc) {
    Write-Host "⚠ Game is not currently running" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To launch game:" -ForegroundColor Cyan
    Write-Host "  /launch-game (in Claude Code)"
    Write-Host "  OR"
    Write-Host "  /prove-features (to boot game to stable state first)"
    Write-Host ""
} else {
    Write-Host "✓ Game is running (PID: $($gameProc.Id))" -ForegroundColor Green
}
Write-Host ""

# ── Step 5: Summary ────────────────────────────────────────────────────────────
Write-Host "=========================================================" -ForegroundColor Cyan
Write-Host "Pre-flight checks complete" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Ensure game is running: /launch-game or /prove-features"
Write-Host "  2. Run evaluation: /eval-game-features (in Claude Code)"
Write-Host "  3. Results will be saved to: docs/proof-of-features/extended_<timestamp>/"
Write-Host ""
Write-Host "The /eval-game-features command orchestrates:" -ForegroundColor Gray
Write-Host "  • Stat override test (A-2)"
Write-Host "  • Hot reload test (A-3)"
Write-Host "  • Economy pack test (A-4)"
Write-Host "  • Scenario pack test (A-5)"
Write-Host "  • Asset swap test (A-6)"
Write-Host ""
