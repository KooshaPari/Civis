# DINOForge Mutation Testing Script
# Runs Stryker.NET mutation tests and fails if mutation score < 70%

param(
    [string]$Project = "src/SDK/DINOForge.SDK.csproj",
    [int]$MinScore = 70
)

$ErrorActionPreference = "Stop"
$strykerVersion = "3.13.1"

Write-Host "=== DINOForge Mutation Testing ===" -ForegroundColor Cyan
Write-Host ""

# Check if Stryker.NET is installed at the pinned version
Write-Host "Ensuring Stryker.NET $strykerVersion is installed..." -ForegroundColor Yellow
dotnet tool update -g dotnet-stryker --version $strykerVersion
if ($LASTEXITCODE -ne 0) {
    dotnet tool install -g dotnet-stryker --version $strykerVersion
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to install Stryker.NET $strykerVersion" -ForegroundColor Red
        exit 1
    }
}

# Build the project first
Write-Host "Building project: $Project" -ForegroundColor Cyan
dotnet build $Project -c Release --verbosity minimal
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed" -ForegroundColor Red
    exit 1
}

# Run Stryker mutation testing
Write-Host ""
Write-Host "Running mutation tests..." -ForegroundColor Cyan
$outputDir = "StrykerOutput"
if (Test-Path $outputDir) {
    Add-Type -AssemblyName Microsoft.VisualBasic
    [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory(
        (Resolve-Path $outputDir).ProviderPath,
        [Microsoft.VisualBasic.FileIO.UIOption]::OnlyErrorDialogs,
        [Microsoft.VisualBasic.FileIO.RecycleOption]::SendToRecycleBin)
}

dotnet stryker --project $Project --output $outputDir
$strykerExitCode = $LASTEXITCODE

# Parse mutation score from report
$reportPath = Join-Path $outputDir "mutation-report.json"
$mutationScore = 0
$killed = 0
$survived = 0
$total = 0

if (Test-Path $reportPath) {
    $report = Get-Content $reportPath | ConvertFrom-Json
    
    # Sum up all file-level mutants
    $files = $report.files.PSObject.Properties
    foreach ($file in $files) {
        $mutants = $file.Value.mutants
        foreach ($mutant in $mutants) {
            $total++
            switch ($mutant.status) {
                "Killed" { $killed++ }
                "Survived" { $survived++ }
            }
        }
    }
    
    if ($total -gt 0) {
        $mutationScore = [math]::Round(($killed / $total) * 100, 2)
    }
}

Write-Host ""
Write-Host "=== Mutation Test Results ===" -ForegroundColor Cyan
Write-Host "Total mutants:     $total"
Write-Host "Killed:            $killed"
Write-Host "Survived:          $survived"
Write-Host "Mutation score:    $mutationScore%"

# Check if score meets threshold
if ($mutationScore -lt $MinScore) {
    Write-Host ""
    Write-Host "ERROR: Mutation score ($mutationScore%) is below minimum threshold ($MinScore%)" -ForegroundColor Red
    Write-Host "Please add more tests or fix surviving mutants." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "SUCCESS: Mutation score ($mutationScore%) meets threshold ($MinScore%)" -ForegroundColor Green
exit 0
