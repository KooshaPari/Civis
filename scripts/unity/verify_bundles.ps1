#!/usr/bin/env pwsh
<#
.SYNOPSIS
Verify DINOForge asset bundles are valid and match YAML references

.DESCRIPTION
Checks that:
1. All bundles have valid UnityFS magic bytes
2. .manifest files exist for each bundle
3. All visual_asset references in YAML have corresponding bundles
4. Reports bundle sizes and statistics

.PARAMETER RepoRoot
Path to DINOForge repo root (default: C:\Users\koosh\Dino)

.PARAMETER PackName
Specific pack to verify (default: all packs)

#>
param(
    [string]$RepoRoot = "C:\Users\koosh\Dino",
    [string]$PackName = ""
)

function Test-UnityBundle {
    param([string]$FilePath)

    if (-not (Test-Path $FilePath)) {
        return $false
    }

    try {
        $bytes = [System.IO.File]::ReadAllBytes($FilePath) | Select-Object -First 7
        $magic = [System.Text.Encoding]::ASCII.GetString($bytes[0..6])
        return $magic -eq "UnityFS"
    }
    catch {
        return $false
    }
}

function Get-VisualAssets {
    param([string]$PackRoot)

    $assets = @{}
    $searchDirs = @("units", "buildings", "factions")

    foreach ($dir in $searchDirs) {
        $dirPath = Join-Path $PackRoot $dir
        if (-not (Test-Path $dirPath)) { continue }

        Get-ChildItem -Path $dirPath -Filter "*.yaml" -Recurse | ForEach-Object {
            $content = Get-Content $_.FullName -Raw
            $content -split '\n' | ForEach-Object {
                if ($_ -match 'visual_asset:\s*(\S+)') {
                    $assetId = $Matches[1]
                    if (-not $assets.ContainsKey($assetId)) {
                        $assets[$assetId] = 0
                    }
                    $assets[$assetId]++
                }
            }
        }
    }

    return $assets
}

function Verify-Pack {
    param([string]$PackName)

    $packRoot = Join-Path $RepoRoot "packs" $PackName
    $bundleDir = Join-Path $packRoot "assets" "bundles"

    if (-not (Test-Path $bundleDir)) {
        Write-Host "  [WARN] No bundles directory: $bundleDir" -ForegroundColor Yellow
        return
    }

    Write-Host ""
    Write-Host "Verifying Pack: $PackName" -ForegroundColor Cyan
    Write-Host "Bundle Directory: $bundleDir" -ForegroundColor Gray

    # Get referenced assets from YAML
    $yamlAssets = Get-VisualAssets -PackRoot $packRoot
    Write-Host "  Referenced in YAML: $($yamlAssets.Count) unique assets" -ForegroundColor Gray

    # Get actual bundle files (exclude .manifest and metadata)
    $bundles = Get-ChildItem -Path $bundleDir -File |
               Where-Object { $_.Extension -eq "" -and $_.Name -ne "AssetBundles" } |
               Select-Object -ExpandProperty Name

    Write-Host "  Actual bundles: $($bundles.Count) files" -ForegroundColor Gray

    # Verify each YAML reference has a bundle
    $missingBundles = @()
    foreach ($assetId in $yamlAssets.Keys) {
        $bundlePath = Join-Path $bundleDir $assetId
        if (-not (Test-Path $bundlePath)) {
            $missingBundles += $assetId
        }
    }

    if ($missingBundles.Count -gt 0) {
        Write-Host "  [ERROR] Missing bundles: $($missingBundles.Count)" -ForegroundColor Red
        $missingBundles | ForEach-Object {
            Write-Host "    - $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  [OK] All YAML references have bundles" -ForegroundColor Green
    }

    # Verify bundle validity
    $validBundles = 0
    $invalidBundles = @()

    foreach ($bundle in $bundles) {
        $bundlePath = Join-Path $bundleDir $bundle
        if (Test-UnityBundle -FilePath $bundlePath) {
            $validBundles++
        } else {
            $invalidBundles += $bundle
        }
    }

    if ($invalidBundles.Count -gt 0) {
        Write-Host "  [WARN] Invalid bundles: $($invalidBundles.Count)" -ForegroundColor Yellow
        $invalidBundles | ForEach-Object {
            Write-Host "    - $_" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  [OK] All bundles have valid UnityFS magic bytes" -ForegroundColor Green
    }

    # Verify .manifest files
    $missingManifests = @()
    foreach ($bundle in $bundles) {
        $manifestPath = Join-Path $bundleDir "$bundle.manifest"
        if (-not (Test-Path $manifestPath)) {
            $missingManifests += $bundle
        }
    }

    if ($missingManifests.Count -gt 0) {
        Write-Host "  [WARN] Missing manifests: $($missingManifests.Count)" -ForegroundColor Yellow
    } else {
        Write-Host "  [OK] All bundles have .manifest files" -ForegroundColor Green
    }

    # Statistics
    $totalSize = (Get-ChildItem -Path $bundleDir -File | Measure-Object -Property Length -Sum).Sum
    $totalSizeMB = [math]::Round($totalSize / 1MB, 2)

    Write-Host ""
    Write-Host "  Statistics:" -ForegroundColor Cyan
    Write-Host "    Total bundles: $($bundles.Count)"
    Write-Host "    Valid bundles: $validBundles"
    Write-Host "    Total size: $totalSizeMB MB"
}

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "DINOForge Bundle Verification" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Get all packs to verify
if ([string]::IsNullOrEmpty($PackName)) {
    $packsRoot = Join-Path $RepoRoot "packs"
    $packs = Get-ChildItem -Path $packsRoot -Directory | Select-Object -ExpandProperty Name
} else {
    $packs = @($PackName)
}

foreach ($pack in $packs) {
    if ($pack -match "warfare") {
        Verify-Pack -PackName $pack
    }
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Verification complete" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
