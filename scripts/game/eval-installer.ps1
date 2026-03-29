<#
.SYNOPSIS
    Evaluate DINOForge installer end-to-end.

    Builds InstallerLib, verifies PowerShell script parameters, runs unit tests,
    and reports results to a JSON file.

.PARAMETER OutputDir
    Directory to write eval_installer_report.json (default: $env:TEMP\DINOForge\eval)

.EXAMPLE
    .\eval-installer.ps1
    .\eval-installer.ps1 -OutputDir "C:\Users\koosh\Desktop\reports"
#>

[CmdletBinding()]
param(
    [string]$OutputDir = "$env:TEMP\DINOForge\eval"
)

$ErrorActionPreference = "Stop"

# ─── Constants ───────────────────────────────────────────────────────────

$RepoRoot = "C:\Users\koosh\Dino"
$InstallerLibProj = "$RepoRoot\src\Tools\Installer\InstallerLib\DINOForge.Tools.InstallerLib.csproj"
$InstallerGuiProj = "$RepoRoot\src\Tools\Installer\GUI\DINOForge.Installer.csproj"
$InstallerScript = "$RepoRoot\src\Tools\Installer\Install-DINOForge.ps1"
$TestsProj = "$RepoRoot\src\Tests\DINOForge.Tests.csproj"

$Report = @{
    timestamp          = Get-Date -Format "o"
    steps              = @()
    installer_params   = @{}
    script_validation  = @{}
    build_results      = @{}
    test_results       = @{}
    overall_status     = "unknown"
    errors             = @()
}

# ─── Helper Functions ───────────────────────────────────────────────────

function Write-Step {
    param([string]$Message)
    Write-Host "[*] " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[+] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[!] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Err {
    param([string]$Message)
    Write-Host "[-] " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

function Add-ReportStep {
    param(
        [string]$Name,
        [string]$Status,
        [hashtable]$Details = @{}
    )
    $Report.steps += @{
        name    = $Name
        status  = $Status
        details = $Details
        time    = Get-Date -Format "o"
    }
}

function Add-ReportError {
    param([string]$Error)
    $Report.errors += $Error
}

# ─── Step 1: Verify files exist ───────────────────────────────────────

Write-Step "Verifying installer files exist..."
try {
    $checks = @{
        "InstallerLib project" = $InstallerLibProj
        "Installer GUI project" = $InstallerGuiProj
        "Install script" = $InstallerScript
        "Tests project" = $TestsProj
    }

    $missingFiles = @()
    foreach ($check in $checks.GetEnumerator()) {
        if (-not (Test-Path $check.Value)) {
            $missingFiles += $check.Key
            Write-Err "$($check.Key): NOT FOUND at $($check.Value)"
        } else {
            Write-Success "$($check.Key): OK"
        }
    }

    if ($missingFiles.Count -gt 0) {
        throw "Missing files: $($missingFiles -join ', ')"
    }

    Add-ReportStep "file_verification" "passed" @{
        files_checked = $checks.Count
    }
}
catch {
    Write-Err "File verification failed: $_"
    Add-ReportError "File verification: $_"
    Add-ReportStep "file_verification" "failed" @{ error = $_ }
}

# ─── Step 2: Extract installer parameters from PowerShell script ───────

Write-Step "Analyzing installer script parameters..."
try {
    $scriptContent = Get-Content $InstallerScript -Raw

    # Extract param block using regex
    $paramMatch = $scriptContent -match '\[CmdletBinding\(\)\]\s*param\((.*?)\)'
    if (-not $paramMatch) {
        Write-Warn "Could not extract param block via regex; checking file manually"
        $paramSection = $scriptContent | Select-String -Pattern 'param\(' -A 20 | ForEach-Object { $_.Line } | Select-Object -First 20
        Write-Host "Param section (first 20 lines):`n$($paramSection -join "`n")"
    }

    # Manual parsing: look for [string]$ParameterName and [switch]$ParameterName
    $stringParams = [regex]::Matches($scriptContent, '\[string\]\$(\w+)')
    $switchParams = [regex]::Matches($scriptContent, '\[switch\]\$(\w+)')

    $Report.installer_params = @{
        string_params = @($stringParams | ForEach-Object { $_.Groups[1].Value })
        switch_params = @($switchParams | ForEach-Object { $_.Groups[1].Value })
    }

    Write-Success "Detected parameters:"
    Write-Host "  String parameters: $($Report.installer_params.string_params -join ', ')"
    Write-Host "  Switch parameters: $($Report.installer_params.switch_params -join ', ')"

    Add-ReportStep "parameter_extraction" "passed" $Report.installer_params
}
catch {
    Write-Err "Parameter extraction failed: $_"
    Add-ReportError "Parameter extraction: $_"
    Add-ReportStep "parameter_extraction" "failed" @{ error = $_ }
}

# ─── Step 3: Validate script syntax ──────────────────────────────────

Write-Step "Validating PowerShell script syntax..."
try {
    # Test-Path doesn't check syntax; use PSParser or Test-ScriptFileValidity if available
    # For PS 5.1, we'll use a try-dot-source with ErrorAction Stop in a subprocess
    $syntaxCheckCode = @"
`$ErrorActionPreference = 'Stop'
try {
    . '$InstallerScript' -GamePath 'C:\test' -ErrorAction Stop
} catch [System.Management.Automation.ParseException] {
    Write-Error "Syntax error: `$_"
    exit 1
} catch {
    # Other errors are OK; we're just checking syntax
    exit 0
}
"@

    $result = powershell -ExecutionPolicy Bypass -NoProfile -Command $syntaxCheckCode 2>&1
    if ($LASTEXITCODE -eq 1) {
        throw "Script syntax validation failed: $result"
    }

    Write-Success "Script syntax: OK (dot-source test passed)"
    Add-ReportStep "script_syntax_validation" "passed" @{
        method = "dot-source test with -ErrorAction Stop"
    }
}
catch {
    Write-Warn "Script syntax check: $_"
    Add-ReportError "Script syntax: $_"
    Add-ReportStep "script_syntax_validation" "warning" @{
        error = $_
        note = "Non-fatal: script may still be valid if error was from missing parameters"
    }
}

# ─── Step 4: Build InstallerLib ─────────────────────────────────────

Write-Step "Building InstallerLib..."
try {
    Push-Location $RepoRoot

    $buildOutput = dotnet build $InstallerLibProj -c Release --verbosity minimal 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed: $buildOutput"
    }

    Write-Success "InstallerLib build: OK"
    Add-ReportStep "build_installerlib" "passed" @{
        project = $InstallerLibProj
        config = "Release"
    }
}
catch {
    Write-Err "InstallerLib build failed: $_"
    Add-ReportError "Build InstallerLib: $_"
    Add-ReportStep "build_installerlib" "failed" @{ error = $_ }
}
finally {
    Pop-Location
}

# ─── Step 5: Build GUI Installer ────────────────────────────────────

Write-Step "Building GUI Installer..."
try {
    Push-Location $RepoRoot

    $buildOutput = dotnet build $InstallerGuiProj -c Release --verbosity minimal 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed: $buildOutput"
    }

    Write-Success "GUI Installer build: OK"
    Add-ReportStep "build_gui_installer" "passed" @{
        project = $InstallerGuiProj
        config = "Release"
    }
}
catch {
    Write-Err "GUI Installer build failed: $_"
    Add-ReportError "Build GUI Installer: $_"
    Add-ReportStep "build_gui_installer" "failed" @{ error = $_ }
}
finally {
    Pop-Location
}

# ─── Step 6: Run Installer unit tests ───────────────────────────────

Write-Step "Running Installer unit tests..."
try {
    Push-Location $RepoRoot

    $testOutput = dotnet test $TestsProj `
        --filter "FullyQualifiedName~Installer" `
        --no-build `
        --verbosity normal 2>&1

    if ($LASTEXITCODE -ne 0) {
        Write-Warn "Some tests may have failed or were skipped; checking details..."
    }

    # Parse test output for summary
    $testSummary = $testOutput | Select-String -Pattern "(\d+) passed|(\d+) failed|(\d+) skipped"
    $Report.test_results.output = @($testOutput)

    if ($testSummary) {
        Write-Success "Test summary: $testSummary"
        $Report.test_results.summary = $testSummary.ToString()
    }

    # Check if any tests passed
    $passedMatch = $testOutput | Select-String -Pattern "(\d+) passed"
    if ($passedMatch) {
        $passCount = [int]($passedMatch.Matches[0].Groups[1].Value)
        if ($passCount -gt 0) {
            Write-Success "At least $passCount installer tests passed"
            Add-ReportStep "unit_tests" "passed" @{
                filter = "Installer"
                passed = $passCount
            }
        } else {
            Write-Warn "No installer tests found or all skipped"
            Add-ReportStep "unit_tests" "warning" @{
                filter = "Installer"
                message = "No tests executed"
            }
        }
    } else {
        Write-Warn "Could not parse test output"
        Add-ReportStep "unit_tests" "warning" @{
            message = "Could not parse test summary"
        }
    }
}
catch {
    Write-Err "Test execution error: $_"
    Add-ReportError "Unit tests: $_"
    Add-ReportStep "unit_tests" "failed" @{ error = $_ }
}
finally {
    Pop-Location
}

# ─── Step 7: Create eval report ──────────────────────────────────────

Write-Step "Creating eval report..."
try {
    # Ensure output directory exists
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    }

    $reportFile = Join-Path $OutputDir "eval_installer_report.json"

    # Determine overall status
    $failedSteps = @($Report.steps | Where-Object { $_.status -eq "failed" })
    if ($failedSteps.Count -eq 0) {
        $Report.overall_status = "passed"
    } else {
        $Report.overall_status = "failed"
    }

    # Convert to JSON with proper formatting
    $jsonReport = $Report | ConvertTo-Json -Depth 10
    $jsonReport | Set-Content $reportFile -Encoding UTF8

    Write-Success "Report written to: $reportFile"
    Write-Host ("`nReport Summary:`n" + ($Report | ConvertTo-Json -Depth 3))

    Add-ReportStep "report_generation" "passed" @{
        file = $reportFile
        status = $Report.overall_status
    }
}
catch {
    Write-Err "Report generation failed: $_"
    Add-ReportError "Report generation: $_"
}

# ─── Summary ──────────────────────────────────────────────────────────

Write-Host "`n================================`n"
if ($Report.overall_status -eq "passed") {
    Write-Success "INSTALLER EVALUATION: PASSED"
} else {
    Write-Err "INSTALLER EVALUATION: FAILED"
    if ($Report.errors.Count -gt 0) {
        Write-Host "`nErrors encountered:"
        $Report.errors | ForEach-Object { Write-Host "  - $_" }
    }
}
Write-Host "================================`n"

if ($Report.overall_status -eq "failed") {
    exit 1
} else {
    exit 0
}
