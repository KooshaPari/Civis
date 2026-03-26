#!/usr/bin/env pwsh
<#
.SYNOPSIS
Build DINOForge asset bundles using Unity in batch mode (headless)

.DESCRIPTION
Runs a Unity Editor project in batch mode to compile asset bundles for DINOForge packs.
Bundles are built with ChunkBasedCompression for optimal runtime loading.

.PARAMETER UnityPath
Path to Unity.exe (default: C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe)

.PARAMETER ProjectPath
Path to the Bundle Builder project (default: scripts/unity/BundleBuilder relative to repo root)

.PARAMETER RepoRoot
Path to DINOForge repo root (default: C:\Users\koosh\Dino)

.PARAMETER LogFile
Output log file for Unity build (default: scripts/unity/unity_build.log relative to repo root)

.EXAMPLE
.\build_bundles.ps1
# Uses all defaults

.EXAMPLE
.\build_bundles.ps1 -Verbose
# Run with verbose output and show build log

#>
param(
    [string]$UnityPath = "C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe",
    [string]$ProjectPath = "",
    [string]$RepoRoot = "C:\Users\koosh\Dino",
    [string]$LogFile = ""
)

# Resolve default paths if not provided
if ([string]::IsNullOrEmpty($ProjectPath)) {
    $ProjectPath = Join-Path $RepoRoot "scripts\unity\BundleBuilder"
}

if ([string]::IsNullOrEmpty($LogFile)) {
    $LogFile = Join-Path $RepoRoot "scripts\unity\unity_build.log"
}

# Ensure log directory exists
$LogDir = Split-Path -Parent $LogFile
if (-not (Test-Path $LogDir)) {
    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
}

# Verify Unity exists
if (-not (Test-Path $UnityPath)) {
    Write-Host "[ERROR] Unity not found at: $UnityPath" -ForegroundColor Red
    exit 1
}

# Verify project exists
if (-not (Test-Path $ProjectPath)) {
    Write-Host "[ERROR] Project not found at: $ProjectPath" -ForegroundColor Red
    exit 1
}

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "DINOForge Asset Bundle Builder" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Unity Path:    $UnityPath" -ForegroundColor Gray
Write-Host "Project Path:  $ProjectPath" -ForegroundColor Gray
Write-Host "Repo Root:     $RepoRoot" -ForegroundColor Gray
Write-Host "Log File:      $LogFile" -ForegroundColor Gray
Write-Host ""
Write-Host "[Unity] Starting batch build..." -ForegroundColor Cyan

# Remove old log
if (Test-Path $LogFile) {
    Remove-Item $LogFile -Force
}

# Launch Unity in batch mode
$unityArgs = @(
    "-batchmode",
    "-nographics",
    "-projectPath", $ProjectPath,
    "-executeMethod", "DINOForgeBundleBuilder.BuildAllBundles",
    "-logFile", $LogFile,
    "-quit"
)

try {
    $startTime = Get-Date
    & $UnityPath $unityArgs
    $exitCode = $LASTEXITCODE
    $elapsed = (Get-Date) - $startTime

    Write-Host ""
    Write-Host "Build completed in $($elapsed.TotalSeconds) seconds" -ForegroundColor Gray

    # Show log
    if (Test-Path $LogFile) {
        Write-Host ""
        Write-Host "[Log Output]:" -ForegroundColor Cyan
        Get-Content $LogFile | Select-Object -Last 50 | ForEach-Object {
            if ($_ -match "Error|error|ERROR|Exception") {
                Write-Host $_ -ForegroundColor Red
            }
            elseif ($_ -match "Success|success|SUCCESS|\[DINOForge\]") {
                Write-Host $_ -ForegroundColor Green
            }
            else {
                Write-Host $_
            }
        }

        # Final status
        Write-Host ""
        if ($exitCode -eq 0) {
            Write-Host "[SUCCESS] Asset bundles built successfully!" -ForegroundColor Green

            # Report bundle statistics
            $bundleDir = Join-Path $RepoRoot "packs\warfare-starwars\assets\bundles"
            if (Test-Path $bundleDir) {
                $bundles = Get-ChildItem $bundleDir -File | Where-Object { $_.Extension -eq "" } | Measure-Object
                $totalSize = (Get-ChildItem $bundleDir -File | Measure-Object -Property Length -Sum).Sum
                $totalSizeMB = [math]::Round($totalSize / 1MB, 2)

                Write-Host ""
                Write-Host "Bundle Statistics:" -ForegroundColor Cyan
                Write-Host "  Total bundles: $($bundles.Count)"
                Write-Host "  Total size: $totalSizeMB MB"
            }

            exit 0
        }
        else {
            Write-Host "[FAILED] Build failed with exit code $exitCode" -ForegroundColor Red
            exit 1
        }
    }
    else {
        Write-Host "[ERROR] Log file not created: $LogFile" -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Host "[ERROR] Exception during build: $_" -ForegroundColor Red
    exit 1
}
