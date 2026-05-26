
<#
.SYNOPSIS
Run DINOForge E2E test suite via pytest.

.DESCRIPTION
Executes pytest on the E2E test suite in src/Tests/e2e/.
Supports filtering by markers (e2e, vlm, slow) and includes game process checks.

.PARAMETER Filter
Pytest marker filter: 'e2e', 'vlm', 'slow', 'not vlm', 'e2e and not slow', etc.
Default: 'e2e' (all E2E tests)

.PARAMETER Verbose
Enable verbose output and detailed tracebacks (-vv, --tb=long)

.PARAMETER ScreenshotDir
Directory to save test screenshots. Default: temp directory.

.PARAMETER RequireGame
Skip tests if game.exe is not running (default: true)

.EXAMPLE
.\scripts\test-e2e.ps1
Run all E2E tests (default marker filter)

.EXAMPLE
.\scripts\test-e2e.ps1 -Filter "vlm" -Verbose
Run only VLM tests with verbose output

.EXAMPLE
.\scripts\test-e2e.ps1 -Filter "e2e and not slow"
Run fast E2E tests only (skip VLM, slow tests)

.EXAMPLE
.\scripts\test-e2e.ps1 -Filter "test_game_status" -RequireGame $false
Run specific test module, allow game offline
#>

param(
    [string]$Filter = "e2e",
    [switch]$Verbose,
    [string]$ScreenshotDir,
    [bool]$RequireGame = $true
)

$ErrorActionPreference = "Stop"

$REPO_ROOT = Split-Path -Parent $PSScriptRoot
$E2E_DIR = Join-Path $REPO_ROOT "src/Tests/e2e"
$REQUIREMENTS_FILE = Join-Path $E2E_DIR "requirements.txt"

Write-Host "DINOForge E2E Test Suite" -ForegroundColor Cyan
Write-Host "========================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check Python
Write-Host "Checking Python installation..." -ForegroundColor Yellow
$pythonCmd = Get-Command python -ErrorAction SilentlyContinue
if (-not $pythonCmd) {
    Write-Error "Python not found. Install Python 3.10+ and add to PATH."
}
$pythonVersion = & python --version 2>&1
Write-Host "✓ $pythonVersion" -ForegroundColor Green

# Step 2: Check game running (if required)
if ($RequireGame) {
    Write-Host "Checking if game is running..." -ForegroundColor Yellow
    $gameProcess = Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
    if (-not $gameProcess) {
        Write-Warning "Game process not found!"
        Write-Host "Starting game tests without game running will cause skips." -ForegroundColor Yellow
        Write-Host "To run tests, launch: G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
        Write-Host ""
        $continue = Read-Host "Continue anyway? (y/n)"
        if ($continue -ne "y" -and $continue -ne "Y") {
            exit 1
        }
    } else {
        Write-Host "✓ Game is running (PID: $($gameProcess.Id))" -ForegroundColor Green
    }
}

# Step 3: Install dependencies
Write-Host "Checking Python dependencies..." -ForegroundColor Yellow
if (Test-Path $REQUIREMENTS_FILE) {
    Write-Host "Installing from $REQUIREMENTS_FILE" -ForegroundColor Cyan
    & python -m pip install -q -r $REQUIREMENTS_FILE
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install dependencies"
    }
    Write-Host "✓ Dependencies installed" -ForegroundColor Green
} else {
    Write-Error "requirements.txt not found at $REQUIREMENTS_FILE"
}

# Step 4: Prepare pytest arguments
$pytestArgs = @(
    $E2E_DIR,
    "-v",
    "-m", $Filter,
    "--tb=short"
)

if ($Verbose) {
    $pytestArgs += @("-vv", "--tb=long")
}

if ($ScreenshotDir) {
    $env:PYTEST_SCREENSHOT_DIR = $ScreenshotDir
}

# Step 5: Run tests
Write-Host ""
Write-Host "Running pytest with marker: $Filter" -ForegroundColor Yellow
Write-Host "pytest $($pytestArgs -join ' ')" -ForegroundColor Gray
Write-Host ""

& python -m pytest @pytestArgs

# Summary
$exitCode = $LASTEXITCODE
Write-Host ""
if ($exitCode -eq 0) {
    Write-Host "✓ All tests passed!" -ForegroundColor Green
} else {
    Write-Host "✗ Tests failed (exit code: $exitCode)" -ForegroundColor Red
}

exit $exitCode
