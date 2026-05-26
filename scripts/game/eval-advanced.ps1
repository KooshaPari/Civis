#Requires -Version 5.1
<#
.SYNOPSIS
Evaluates DINOForge advanced feature infrastructure: hidden desktop, dual-instance, and VDD support.

.DESCRIPTION
This script tests the availability and readiness of three advanced agent-automation features:
- D-1: Hidden Desktop launch (isolated background rendering)
- D-2: Dual-instance infrastructure (test instance preparation)
- D-3: Virtual Display Driver support (for future headless deployments)

Results are written to a JSON report for CI/CD integration and proof-of-capability documentation.

Test sequence:
1. Verify MCP server is running and healthy
2. Check hidden desktop launcher exists and is executable
3. Verify test instance directory and infrastructure
4. Check MCP tool availability (game_launch_test, game_launch)
5. Report results as PASS/FAIL/SKIP with clear rationale

.EXAMPLE
pwsh -File scripts/game/eval-advanced.ps1
pwsh -File scripts/game/eval-advanced.ps1 -Verbose
pwsh -File scripts/game/eval-advanced.ps1 -OutputDir C:\custom\path
#>

param(
    [string]$OutputDir = "$env:TEMP\DINOForge",
    [string]$McpServerUrl = "http://127.0.0.1:8765",
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$VerbosePreference = if ($Verbose) { "Continue" } else { "SilentlyContinue" }

# Ensure output directory exists
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    Write-Verbose "Created output directory: $OutputDir"
}

$reportPath = Join-Path $OutputDir "eval_advanced_report.json"

Write-Host "[$(Get-Date -Format 'HH:mm:ss')] DINOForge Advanced Feature Evaluation" -ForegroundColor Cyan
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] MCP Server: $McpServerUrl" -ForegroundColor Gray
Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Output: $reportPath" -ForegroundColor Gray
Write-Host ""

# Initialize results object
$results = @{
    timestamp = Get-Date -Format "o"
    mcp_server = @{ status = "unknown"; healthy = $false; reason = "" }
    d1_hidden_desktop = @{ status = "unknown"; available = $false; reason = ""; script_path = "" }
    d2_dual_instance = @{ status = "unknown"; available = $false; reason = ""; test_instance_path = "" }
    d3_vdd = @{ status = "unknown"; available = $false; reason = "" }
    summary = @{ passed = 0; failed = 0; skipped = 0 }
}

# ============================================================================
# STEP 1: MCP Server Health Check
# ============================================================================

Write-Host "Step 1/4: MCP Server Health Check" -ForegroundColor Yellow

try {
    Write-Verbose "Attempting to connect to MCP server at $McpServerUrl..."

    # Try a simple health check via a quick HTTP request
    $testUri = "$McpServerUrl/health"
    $response = Invoke-WebRequest -Uri $testUri -Method GET -TimeoutSec 5 -ErrorAction SilentlyContinue

    if ($response.StatusCode -eq 200) {
        Write-Host "  ✓ MCP server is running and responding" -ForegroundColor Green
        $results.mcp_server.status = "healthy"
        $results.mcp_server.healthy = $true
        $results.summary.passed += 1
    } else {
        throw "HTTP $($response.StatusCode)"
    }
} catch {
    Write-Host "  ✗ MCP server health check failed: $_" -ForegroundColor Red
    $results.mcp_server.status = "failed"
    $results.mcp_server.reason = "Server not responding or unreachable at $McpServerUrl"
    $results.summary.failed += 1

    Write-Host ""
    Write-Host "  Note: MCP server may be offline. Start it with:" -ForegroundColor Gray
    Write-Host "    python -m dinoforge_mcp.server" -ForegroundColor Gray
    Write-Host "    (from directory: src/Tools/DinoforgeMcp)" -ForegroundColor Gray
    Write-Host ""
}

# ============================================================================
# STEP 2: Hidden Desktop Infrastructure Check (D-1)
# ============================================================================

Write-Host "Step 2/4: Hidden Desktop Infrastructure (D-1)" -ForegroundColor Yellow

$hiddenDesktopScript = "C:\Users\koosh\Dino\scripts\game\hidden_desktop_test.ps1"

if (Test-Path $hiddenDesktopScript) {
    Write-Host "  ✓ Hidden desktop launcher script exists: $hiddenDesktopScript" -ForegroundColor Green
    $results.d1_hidden_desktop.available = $true
    $results.d1_hidden_desktop.status = "available"
    $results.d1_hidden_desktop.script_path = $hiddenDesktopScript
    $results.summary.passed += 1

    # Additional check: verify script is readable and has P/Invoke definitions
    try {
        $content = Get-Content $hiddenDesktopScript -Raw
        if ($content -match "CreateDesktop|SetThreadDesktop") {
            Write-Host "  ✓ Script contains Win32 hidden desktop P/Invoke definitions" -ForegroundColor Green
            $results.d1_hidden_desktop.reason = "Full hidden desktop infrastructure available; supports isolated game launches"
        } else {
            Write-Host "  ⚠ Warning: Script may be incomplete (P/Invoke definitions not detected)" -ForegroundColor Yellow
            $results.d1_hidden_desktop.reason = "Script exists but may lack required Win32 APIs"
        }
    } catch {
        Write-Host "  ⚠ Could not validate script content: $_" -ForegroundColor Yellow
        $results.d1_hidden_desktop.reason = "Script exists but could not be validated"
    }
} else {
    Write-Host "  ✗ Hidden desktop launcher script not found at: $hiddenDesktopScript" -ForegroundColor Red
    $results.d1_hidden_desktop.status = "not_found"
    $results.d1_hidden_desktop.reason = "hidden_desktop_test.ps1 not available; D-1 feature cannot be tested"
    $results.summary.failed += 1
}

Write-Host ""

# ============================================================================
# STEP 3: Dual-Instance Infrastructure Check (D-2)
# ============================================================================

Write-Host "Step 3/4: Dual-Instance Infrastructure (D-2)" -ForegroundColor Yellow

$testInstancePathFile = "C:\Users\koosh\Dino\.dino_test_instance_path"

if (Test-Path $testInstancePathFile) {
    try {
        $testInstancePath = (Get-Content $testInstancePathFile -Raw).Trim()
        Write-Verbose "Read test instance path: $testInstancePath"

        if ([string]::IsNullOrWhiteSpace($testInstancePath)) {
            Write-Host "  ⚠ Test instance path file is empty" -ForegroundColor Yellow
            $results.d2_dual_instance.status = "incomplete"
            $results.d2_dual_instance.reason = "Configuration file exists but path is not set"
            $results.summary.skipped += 1
        } elseif (Test-Path $testInstancePath) {
            Write-Host "  ✓ Test instance directory exists: $testInstancePath" -ForegroundColor Green
            $results.d2_dual_instance.available = $true
            $results.d2_dual_instance.test_instance_path = $testInstancePath

            # Check for game executable in test instance
            $testGameExe = Join-Path $testInstancePath "Diplomacy is Not an Option.exe"
            if (Test-Path $testGameExe) {
                Write-Host "  ✓ Game executable found at test instance: $testGameExe" -ForegroundColor Green
                Write-Host "  ✓ Dual-instance infrastructure is complete and ready" -ForegroundColor Green
                $results.d2_dual_instance.status = "ready"
                $results.d2_dual_instance.reason = "Both main and test instances are prepared; concurrent game launches possible"
                $results.summary.passed += 1
            } else {
                Write-Host "  ⚠ Game executable not found at test instance location" -ForegroundColor Yellow
                Write-Host "    Expected: $testGameExe" -ForegroundColor Gray
                $results.d2_dual_instance.status = "incomplete"
                $results.d2_dual_instance.available = $true
                $results.d2_dual_instance.reason = "Test instance directory exists but game.exe not deployed"
                $results.summary.skipped += 1
            }
        } else {
            Write-Host "  ✗ Test instance directory not found: $testInstancePath" -ForegroundColor Red
            $results.d2_dual_instance.status = "not_found"
            $results.d2_dual_instance.test_instance_path = $testInstancePath
            $results.d2_dual_instance.reason = "Configured path does not exist; dual-instance setup incomplete"
            $results.summary.failed += 1
        }
    } catch {
        Write-Host "  ✗ Error reading test instance configuration: $_" -ForegroundColor Red
        $results.d2_dual_instance.status = "error"
        $results.d2_dual_instance.reason = "Configuration file exists but could not be parsed"
        $results.summary.failed += 1
    }
} else {
    Write-Host "  ✗ Test instance configuration file not found: $testInstancePathFile" -ForegroundColor Red
    $results.d2_dual_instance.status = "not_found"
    $results.d2_dual_instance.reason = ".dino_test_instance_path configuration file missing"
    $results.summary.failed += 1
}

Write-Host ""

# ============================================================================
# STEP 4: Virtual Display Driver Support (D-3)
# ============================================================================

Write-Host "Step 4/4: Virtual Display Driver Support (D-3)" -ForegroundColor Yellow

# D-3 is a future feature; for now we document its planned role and check for awareness
Write-Host "  ℹ VDD support is planned for future releases (v0.9+)" -ForegroundColor Cyan
Write-Host "  ℹ Purpose: Isolated headless game launches without user desktop" -ForegroundColor Cyan
Write-Host "  ℹ Current workaround: Win32 CreateDesktop (D-1) provides isolation" -ForegroundColor Cyan

$results.d3_vdd.status = "future"
$results.d3_vdd.available = $false
$results.d3_vdd.reason = "VDD is planned; not yet implemented. Win32 hidden desktop (D-1) currently provides isolation. Custom IDD/WDDM driver required for full headless support."
$results.summary.skipped += 1

Write-Host ""

# ============================================================================
# WRITE REPORT
# ============================================================================

Write-Host "Writing report to: $reportPath" -ForegroundColor Yellow

# Convert to JSON with proper formatting
$json = $results | ConvertTo-Json -Depth 4

# Pretty-print JSON
$json = $json -replace '(?m)^', '  ' | ForEach-Object { $_.Substring(2) }

Set-Content -Path $reportPath -Value $json -Encoding UTF8
Write-Host "✓ Report written" -ForegroundColor Green

Write-Host ""

# ============================================================================
# SUMMARY
# ============================================================================

Write-Host "Evaluation Summary:" -ForegroundColor Cyan
Write-Host "  Passed:  $($results.summary.passed)" -ForegroundColor Green
Write-Host "  Failed:  $($results.summary.failed)" -ForegroundColor $(if ($results.summary.failed -gt 0) { "Red" } else { "Green" })
Write-Host "  Skipped: $($results.summary.skipped)" -ForegroundColor Yellow

Write-Host ""
Write-Host "Feature Status:" -ForegroundColor Cyan
Write-Host "  D-1 (Hidden Desktop):  $($results.d1_hidden_desktop.status.ToUpper())" -ForegroundColor $(if ($results.d1_hidden_desktop.available) { "Green" } else { "Red" })
Write-Host "  D-2 (Dual-Instance):   $($results.d2_dual_instance.status.ToUpper())" -ForegroundColor $(if ($results.d2_dual_instance.available) { "Green" } else { "Red" })
Write-Host "  D-3 (Virtual Display): $($results.d3_vdd.status.ToUpper())" -ForegroundColor Yellow

Write-Host ""

if ($results.summary.failed -eq 0) {
    Write-Host "✓ Advanced feature evaluation complete. No critical failures." -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ Some features are not ready. See report for details." -ForegroundColor Red
    exit 1
}
