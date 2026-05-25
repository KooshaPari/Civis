#!/usr/bin/env pwsh
<#
.SYNOPSIS
Evaluation pipeline for DINOForge Desktop Companion.

Orchestrates: build → test (FlaUI) → VLM screenshot validation → report JSON.

.PARAMETER OutputDir
Directory to write eval_companion_report.json and screenshots.
Default: C:\Users\koosh\Dino\docs\eval-reports\companion

.PARAMETER SkipBuild
If set, skip the companion build step (assumes already built).

.PARAMETER SkipTests
If set, skip FlaUI tests (use only VLM validation).

.PARAMETER Verbose
Print detailed build and test output.

.EXAMPLE
.\eval-companion.ps1 -OutputDir ./docs/eval-reports/companion

.EXAMPLE
.\eval-companion.ps1 -SkipBuild -Verbose

#>

param(
    [string]$OutputDir = "C:\Users\koosh\Dino\docs\eval-reports\companion",
    [switch]$SkipBuild,
    [switch]$SkipTests,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version 3.0

# ─────────────────────────────────────────────────────────────────────────────
# Configuration
# ─────────────────────────────────────────────────────────────────────────────

$RepoRoot = "C:\Users\koosh\Dino"
$CompanionProjPath = "$RepoRoot\src\Tools\DesktopCompanion\DesktopCompanion.csproj"
$CompanionBinPath = "$RepoRoot\src\Tools\DesktopCompanion\bin\Release\net11.0-windows10.0.26100.0\DINOForge.DesktopCompanion.exe"
$UiAutomationProjPath = "$RepoRoot\src\Tests\UiAutomation\DINOForge.Tests.UiAutomation.csproj"
$ReportPath = "$OutputDir\eval_companion_report.json"
$ScreenshotsDir = "$OutputDir\screenshots"

$Timestamp = Get-Date -Format "o"
$StartTime = Get-Date

# ─────────────────────────────────────────────────────────────────────────────
# Helper Functions
# ─────────────────────────────────────────────────────────────────────────────

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $prefix = "[$Level $(Get-Date -Format 'HH:mm:ss')]"
    Write-Host "$prefix $Message"
}

function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path -Path $Path -PathType Container)) {
        New-Item -Path $Path -ItemType Directory -Force -ErrorAction Stop | Out-Null
        Write-Log "Created directory: $Path"
    }
}

function Invoke-BuildCompanion {
    Write-Log "Building DesktopCompanion..."

    $buildCmd = @(
        "dotnet", "build",
        "`"$CompanionProjPath`"",
        "-c", "Release",
        "--verbosity", "minimal"
    )

    if ($Verbose) {
        Write-Log "Build command: $($buildCmd -join ' ')"
    }

    & @buildCmd 2>&1 | ForEach-Object {
        if ($Verbose) { Write-Log $_ }
    }

    if ($LASTEXITCODE -ne 0) {
        throw "Build failed with exit code $LASTEXITCODE"
    }

    if (-not (Test-Path -Path $CompanionBinPath -PathType Leaf)) {
        throw "Companion exe not found after build: $CompanionBinPath"
    }

    Write-Log "Build successful: $CompanionBinPath"
    return $CompanionBinPath
}

function Invoke-UiAutomationTests {
    param([string]$CompanionExePath)

    Write-Log "Running UI automation tests..."
    Write-Log "COMPANION_EXE = $CompanionExePath"

    $env:COMPANION_EXE = $CompanionExePath

    $testCmd = @(
        "dotnet", "test",
        "`"$UiAutomationProjPath`"",
        "--verbosity", "normal",
        "--logger", "console;verbosity=minimal"
    )

    if ($Verbose) {
        Write-Log "Test command: $($testCmd -join ' ')"
    }

    $testOutput = @()
    & @testCmd 2>&1 | Tee-Object -Variable testOutput | ForEach-Object {
        if ($Verbose) { Write-Host $_ }
    }

    $exitCode = $LASTEXITCODE

    # Parse test results from output
    $results = Parse-TestOutput -Output $testOutput
    $results["exit_code"] = $exitCode

    Write-Log "Test run complete (exit code: $exitCode)"
    Write-Log "Results: Passed=$($results['passed']), Failed=$($results['failed']), Total=$($results['total'])"

    return $results
}

function Parse-TestOutput {
    param([object[]]$Output)

    $passed = 0
    $failed = 0
    $skipped = 0
    $durationMs = 0

    $text = $Output -join "`n"

    # Pattern: "X passed, Y failed, Z skipped in A ms"
    if ($text -match '(\d+)\s+passed') {
        $passed = [int]$matches[1]
    }

    if ($text -match '(\d+)\s+failed') {
        $failed = [int]$matches[1]
    }

    if ($text -match '(\d+)\s+skipped') {
        $skipped = [int]$matches[1]
    }

    if ($text -match 'in\s+(\d+)\s+ms') {
        $durationMs = [int]$matches[1]
    }

    $total = $passed + $failed + $skipped

    return @{
        total = $total
        passed = $passed
        failed = $failed
        skipped = $skipped
        duration_ms = $durationMs
        test_output = $text
    }
}

function Build-Report {
    param(
        [string]$CompanionExePath,
        [hashtable]$BuildResults,
        [hashtable]$TestResults,
        [hashtable]$PageValidation
    )

    $elapsed = ((Get-Date) - $StartTime).TotalMilliseconds

    $buildStatus = if ($BuildResults["success"]) { "success" } else { "failure" }
    $testStatus = if ($TestResults["exit_code"] -eq 0) { "passed" } else { "failed" }

    $overallStatus = if ($BuildResults["success"] -and $testResults["exit_code"] -eq 0) {
        "pass"
    } else {
        "fail"
    }

    $report = @{
        timestamp = $Timestamp
        companion_exe = $CompanionExePath
        build_status = $buildStatus
        test_results = @{
            total_tests = $TestResults["total"]
            passed = $TestResults["passed"]
            failed = $TestResults["failed"]
            skipped = $TestResults["skipped"]
            duration_ms = $TestResults["duration_ms"]
            exit_code = $TestResults["exit_code"]
        }
        page_validation = $PageValidation["pages"] ?? @()
        overall_status = $overallStatus
        total_duration_ms = [int]$elapsed
        notes = @(
            "Build: $buildStatus",
            "Tests: $testStatus",
            "Companion: $CompanionExePath",
            if (-not $overallStatus.Equals("pass")) {
                "EVALUATION FAILED - see details above"
            }
        ) -join " | "
    }

    return $report
}

function Write-JsonReport {
    param([string]$Path, [object]$Report)

    $json = $Report | ConvertTo-Json -Depth 10
    $json | Set-Content -Path $Path -Encoding UTF8 -Force
    Write-Log "Report written to: $Path"
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

try {
    Write-Log "=== DINOForge Desktop Companion Evaluation Pipeline ==="
    Write-Log "Output directory: $OutputDir"
    Write-Log "Timestamp: $Timestamp"

    Ensure-Directory -Path $OutputDir
    Ensure-Directory -Path $ScreenshotsDir

    # ─── Build ────────────────────────────────────────────────────────────────

    $buildResults = @{ success = $true; exe_path = "" }

    if ($SkipBuild) {
        Write-Log "Skipping build (--SkipBuild)"
        if (Test-Path -Path $CompanionBinPath -PathType Leaf) {
            $buildResults["exe_path"] = $CompanionBinPath
            Write-Log "Using existing exe: $CompanionBinPath"
        } else {
            Write-Log "ERROR: --SkipBuild set but exe not found: $CompanionBinPath" -Level "ERROR"
            $buildResults["success"] = $false
        }
    } else {
        try {
            $companionExe = Invoke-BuildCompanion
            $buildResults["success"] = $true
            $buildResults["exe_path"] = $companionExe
        } catch {
            Write-Log "Build failed: $_" -Level "ERROR"
            $buildResults["success"] = $false
            $buildResults["error"] = $_.Exception.Message
        }
    }

    # ─── Tests ────────────────────────────────────────────────────────────────

    $testResults = @{
        total = 0
        passed = 0
        failed = 0
        skipped = 0
        duration_ms = 0
        exit_code = -1
        test_output = ""
    }

    if (-not $SkipTests -and $buildResults["success"]) {
        try {
            $testResults = Invoke-UiAutomationTests -CompanionExePath $buildResults["exe_path"]
        } catch {
            Write-Log "Tests failed: $_" -Level "ERROR"
            $testResults["error"] = $_.Exception.Message
            $testResults["exit_code"] = 1
        }
    } elseif ($SkipTests) {
        Write-Log "Skipping tests (--SkipTests)"
    } else {
        Write-Log "Skipping tests (build failed)" -Level "WARN"
    }

    # ─── Page Validation (placeholder) ────────────────────────────────────────
    # TODO: Integrate with game_screenshot and game_analyze_screen MCP tools

    $pageValidation = @{
        pages = @(
            @{
                page_name = "Dashboard"
                navigation_id = "NavDashboard"
                screenshot_path = "screenshots/dashboard.png"
                vlm_analysis = "(placeholder: pending MCP integration)"
                validation_status = "pending"
            },
            @{
                page_name = "PackList"
                navigation_id = "NavPackList"
                screenshot_path = "screenshots/packlist.png"
                vlm_analysis = "(placeholder: pending MCP integration)"
                validation_status = "pending"
            },
            @{
                page_name = "DebugPanel"
                navigation_id = "NavDebugPanel"
                screenshot_path = "screenshots/debugpanel.png"
                vlm_analysis = "(placeholder: pending MCP integration)"
                validation_status = "pending"
            },
            @{
                page_name = "Settings"
                navigation_id = "NavSettings"
                screenshot_path = "screenshots/settings.png"
                vlm_analysis = "(placeholder: pending MCP integration)"
                validation_status = "pending"
            }
        )
    }

    # ─── Report ───────────────────────────────────────────────────────────────

    $report = Build-Report `
        -CompanionExePath $buildResults["exe_path"] `
        -BuildResults $buildResults `
        -TestResults $testResults `
        -PageValidation $pageValidation

    Write-JsonReport -Path $ReportPath -Report $report

    # ─── Exit ──────────────────────────────────────────────────────────────────

    Write-Log ""
    Write-Log "=== Evaluation Complete ==="
    Write-Log "Overall Status: $($report.overall_status)"
    Write-Log "Report: $ReportPath"

    if ($report.overall_status -eq "pass") {
        Write-Log "SUCCESS" -Level "INFO"
        exit 0
    } else {
        Write-Log "FAILURE" -Level "ERROR"
        exit 1
    }
}
catch {
    Write-Log "Fatal error: $_" -Level "ERROR"
    Write-Log $_.ScriptStackTrace -Level "ERROR"
    exit 1
}
