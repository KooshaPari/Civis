#!/usr/bin/env pwsh
<#
.SYNOPSIS
Dry-run NuGet package build — builds packages locally without uploading.

.DESCRIPTION
This script demonstrates the full NuGet packaging pipeline used by release.yml:
- Packs all publishable libraries (SDK, Bridge.Protocol, Bridge.Client, Installer, etc.)
- Displays package metadata
- Lists generated files and sizes
- Does NOT upload to nuget.org

Use this to verify package contents before triggering the actual release.

.PARAMETER OutputDir
Output directory for generated packages (default: ./nuget-dry-run-output)

.PARAMETER Version
Override package version (optional). If not specified, reads from .csproj files.

.PARAMETER SkipRestore
Skip dotnet restore step (useful if dependencies already restored)

.EXAMPLE
./nuget-dry-run.ps1
# Builds all packages into ./nuget-dry-run-output

./nuget-dry-run.ps1 -Version 0.24.0
# Builds all packages with version 0.24.0

./nuget-dry-run.ps1 -SkipRestore
# Skips restore, uses cached packages
#>

param(
    [string]$OutputDir = "./nuget-dry-run-output",
    [string]$Version = "",
    [switch]$SkipRestore
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "DINOForge NuGet Dry-Run Build" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host ""

# Cleanup previous run
if (Test-Path $OutputDir) {
    Write-Host "Removing previous output: $OutputDir" -ForegroundColor Yellow
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
Write-Host "Output directory: $(Resolve-Path $OutputDir)" -ForegroundColor Green
Write-Host ""

# Packages to build (from release.yml)
$packages = @(
    "src/SDK/DINOForge.SDK.csproj",
    "src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj",
    "src/Bridge/Client/DINOForge.Bridge.Client.csproj",
    "src/Tools/Installer/InstallerLib/DINOForge.Tools.Installer.csproj",
    "src/Templates/DINOForge.Templates.csproj",
    "src/Domains/Warfare/DINOForge.Domains.Warfare.csproj",
    "src/Domains/Economy/DINOForge.Domains.Economy.csproj",
    "src/Domains/Scenario/DINOForge.Domains.Scenario.csproj",
    "src/Domains/UI/DINOForge.Domains.UI.csproj"
)

Write-Host "Packages to build:" -ForegroundColor Cyan
$packages | ForEach-Object { Write-Host "  - $_" }
Write-Host ""

# Step 1: Restore (optional)
if (-not $SkipRestore) {
    Write-Host "Step 1: Restoring dependencies..." -ForegroundColor Cyan
    try {
        dotnet restore src/DINOForge.CI.NoRuntime.sln --verbosity minimal
        Write-Host "Restore completed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "Restore failed: $_" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
}

# Step 2: Pack each package
Write-Host "Step 2: Packing NuGet packages..." -ForegroundColor Cyan
$packCount = 0
$failedPacks = @()

foreach ($projPath in $packages) {
    if (-not (Test-Path $projPath)) {
        Write-Host "  SKIP: $projPath (not found)" -ForegroundColor Yellow
        continue
    }

    $projName = Split-Path $projPath -Leaf
    Write-Host ""
    Write-Host "  Packing: $projName" -ForegroundColor Cyan

    try {
        $cmd = "dotnet pack '$projPath' -c Release --output '$OutputDir' --no-restore --verbosity minimal"
        if ($Version) {
            $cmd += " -p:PackageVersion=$Version"
        }
        Invoke-Expression $cmd
        $packCount++
        Write-Host "    [OK] Packed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "    [FAILED] $_" -ForegroundColor Red
        $failedPacks += $projPath
    }
}

Write-Host ""
Write-Host "Step 3: Generated Artifacts" -ForegroundColor Cyan

# List generated files
$nupkgs = Get-ChildItem $OutputDir -Filter "*.nupkg" -ErrorAction SilentlyContinue
$snupkgs = Get-ChildItem $OutputDir -Filter "*.snupkg" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Release Packages (.nupkg):" -ForegroundColor Green
if ($nupkgs) {
    $nupkgs | ForEach-Object {
        $sizeMB = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  $($_.Name) ($sizeMB MB)"
    }
}
else {
    Write-Host "  (none found)"
}

Write-Host ""
Write-Host "Symbol Packages (.snupkg):" -ForegroundColor Green
if ($snupkgs) {
    $snupkgs | ForEach-Object {
        $sizeMB = [math]::Round($_.Length / 1MB, 2)
        Write-Host "  $($_.Name) ($sizeMB MB)"
    }
}
else {
    Write-Host "  (none found - enable IncludeSymbols in .csproj)"
}

# Step 4: Display package metadata
Write-Host ""
Write-Host "Step 4: Package Metadata" -ForegroundColor Cyan

if ($nupkgs) {
    # Pick first nupkg to display metadata
    $firstNupkg = $nupkgs[0]
    Write-Host ""
    Write-Host "Sample metadata from: $($firstNupkg.Name)" -ForegroundColor Cyan

    # Extract metadata from nuspec inside nupkg (it's a ZIP file)
    try {
        $tmpDir = Join-Path $env:TEMP "nuspec-extract-$(Get-Random)"
        Expand-Archive -Path $firstNupkg.FullName -DestinationPath $tmpDir -Force
        $nuspecFile = Get-ChildItem $tmpDir -Filter "*.nuspec" | Select-Object -First 1

        if ($nuspecFile) {
            [xml]$nuspec = Get-Content $nuspecFile.FullName
            $metadata = $nuspec.package.metadata

            Write-Host "  ID: $($metadata.id)"
            Write-Host "  Version: $($metadata.version)"
            Write-Host "  Authors: $($metadata.authors)"
            Write-Host "  Description: $($metadata.description)"
            Write-Host "  License: $($metadata.licenseExpression)"
            Write-Host "  Repository: $($metadata.repositoryUrl)"

            if ($metadata.dependencies.group) {
                Write-Host "  Dependencies:"
                $metadata.dependencies.group.dependency | ForEach-Object {
                    Write-Host "    - $($_.id) $($_.version)"
                }
            }
        }

        Remove-Item $tmpDir -Recurse -Force
    }
    catch {
        Write-Host "  (Could not extract metadata: $_)" -ForegroundColor Yellow
    }
}

# Step 5: Summary
Write-Host ""
Write-Host "Summary" -ForegroundColor Cyan
Write-Host "======="
Write-Host "Total packages built: $packCount"
Write-Host "Release packages (.nupkg): $($nupkgs.Count)"
Write-Host "Symbol packages (.snupkg): $($snupkgs.Count)"
Write-Host "Total output size: $(([math]::Round(((Get-ChildItem $OutputDir -File | Measure-Object -Property Length -Sum).Sum / 1MB), 2))) MB"
Write-Host ""

if ($failedPacks.Count -gt 0) {
    Write-Host "FAILED PACKS:" -ForegroundColor Red
    $failedPacks | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}

Write-Host "All packages built successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Review generated packages in: $(Resolve-Path $OutputDir)"
Write-Host "  2. Verify package contents (check dependencies, metadata, etc.)"
Write-Host "  3. To publish: git tag v0.24.0 && git push origin v0.24.0"
Write-Host ""
Write-Host "NOTE: This was a DRY-RUN. No packages were uploaded to nuget.org." -ForegroundColor Yellow
