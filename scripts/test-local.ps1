#Requires -Version 5.1
<#
.SYNOPSIS
    DINOForge local test runner — runs all offline test suites.

.DESCRIPTION
    Runs unit + integration tests (no game required).
    Optionally runs E2E pytest suite if game is running.

.PARAMETER Fast
    Skip slow tests (benchmarks, property tests).

.PARAMETER E2E
    Also run Python E2E suite (requires game running).

.PARAMETER Filter
    xUnit test filter expression (e.g. "FullyQualifiedName~Registry").

.EXAMPLE
    pwsh -File scripts/test-local.ps1
    pwsh -File scripts/test-local.ps1 -Fast
    pwsh -File scripts/test-local.ps1 -E2E
#>
param(
    [switch]$Fast,
    [switch]$E2E,
    [string]$Filter = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Write-Step($msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Pass($msg) { Write-Host "  PASS: $msg" -ForegroundColor Green }
function Write-Fail($msg) { Write-Host "  FAIL: $msg" -ForegroundColor Red }

$failed = @()

# ── Build ──────────────────────────────────────────────────────────────────────
Write-Step "Building DINOForge.sln..."
$buildArgs = @("build", "$RepoRoot/src/DINOForge.sln", "-c", "Release", "--verbosity", "minimal")
& dotnet @buildArgs
if ($LASTEXITCODE -ne 0) { Write-Fail "Build failed"; exit 1 }
Write-Pass "Build OK"

# ── Unit Tests ─────────────────────────────────────────────────────────────────
Write-Step "Running unit tests (DINOForge.Tests)..."
$testArgs = @(
    "test", "$RepoRoot/src/Tests/DINOForge.Tests.csproj",
    "--no-build", "--verbosity", "normal"
)
if ($Filter) { $testArgs += @("--filter", $Filter) }
if ($Fast)   { $testArgs += @("--filter", "Category!=Slow") }
& dotnet @testArgs
if ($LASTEXITCODE -ne 0) { $failed += "unit" } else { Write-Pass "Unit tests OK" }

# ── Integration Tests ──────────────────────────────────────────────────────────
Write-Step "Running integration tests (DINOForge.Tests.Integration)..."
$intArgs = @(
    "test", "$RepoRoot/src/Tests/Integration/DINOForge.Tests.Integration.csproj",
    "--no-build", "--verbosity", "normal"
)
& dotnet @intArgs
if ($LASTEXITCODE -ne 0) { $failed += "integration" } else { Write-Pass "Integration tests OK" }

# ── Python E2E (optional) ──────────────────────────────────────────────────────
if ($E2E) {
    Write-Step "Running Python E2E tests..."
    $gameRunning = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
    if (-not $gameRunning) {
        Write-Host "  SKIP: game not running (pass -E2E only with game active)" -ForegroundColor Yellow
    } else {
        & pwsh -File "$RepoRoot/scripts/test-e2e.ps1" -Verbose
        if ($LASTEXITCODE -ne 0) { $failed += "e2e" } else { Write-Pass "E2E tests OK" }
    }
}

# ── Summary ────────────────────────────────────────────────────────────────────
Write-Host ""
if ($failed.Count -eq 0) {
    Write-Host "ALL TESTS PASSED" -ForegroundColor Green
    exit 0
} else {
    Write-Fail "FAILED: $($failed -join ', ')"
    exit 1
}
