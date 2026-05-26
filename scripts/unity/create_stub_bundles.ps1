#!/usr/bin/env pwsh
<#
.SYNOPSIS
Create stub UnityFS bundles for missing visual_asset references

.DESCRIPTION
For visual_asset IDs referenced in YAML but missing from the bundles directory,
creates minimal valid UnityFS stub bundles that can be loaded at runtime.

This is a temporary solution for incomplete pack asset sets.

.PARAMETER RepoRoot
Path to DINOForge repo root (default: C:\Users\koosh\Dino)

.PARAMETER PackName
Pack to generate stubs for (warfare-starwars, warfare-modern)

#>
param(
    [string]$RepoRoot = "C:\Users\koosh\Dino",
    [string]$PackName = "warfare-starwars"
)

# Minimal UnityFS bundle stub (empty serialized data)
# This is a valid but empty Unity AssetBundle that can be loaded
$stubBundleHex = @"
55 6E 69 74 79 46 53 00 00 00 00 08 30 2E 30 2E
30 00 32 30 32 31 2E 33 2E 34 35 66 31 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00
00 00 00 00 00 00 00 00 00 00
"@ -replace '\s+', ' ' -split ' ' | ForEach-Object { [byte][Convert]::ToInt32($_, 16) }

function Get-MissingVisualAssets {
    param([string]$PackRoot)

    $referenced = @{}
    $searchDirs = @("units", "buildings", "factions")

    foreach ($dir in $searchDirs) {
        $dirPath = Join-Path $PackRoot $dir
        if (-not (Test-Path $dirPath)) { continue }

        Get-ChildItem -Path $dirPath -Filter "*.yaml" -Recurse | ForEach-Object {
            $content = Get-Content $_.FullName -Raw
            $content -split '\n' | ForEach-Object {
                if ($_ -match 'visual_asset:\s*(\S+)') {
                    $assetId = $Matches[1]
                    if (-not $referenced.ContainsKey($assetId)) {
                        $referenced[$assetId] = $true
                    }
                }
            }
        }
    }

    return $referenced.Keys
}

function Create-StubBundle {
    param([string]$BundlePath, [string]$AssetId)

    try {
        # Create minimal stub bundle
        $stubData = $stubBundleHex

        [System.IO.File]::WriteAllBytes($BundlePath, $stubData)

        # Create corresponding manifest
        $manifestPath = "$BundlePath.manifest"
        $manifestContent = @"
ManifestFileVersion: 0
CRC: 0
Hashes:
  AssetFileHash:
    serializedVersion: 2
    Hash: 0000000000000000000000000000000
  TypeTreeHash:
    serializedVersion: 2
    Hash: 0000000000000000000000000000000
HashAppended: 0
ClassTypes: []
SerializeReferenceClassIdentifiers: []
Assets: []
Dependencies: []
"@
        Set-Content -Path $manifestPath -Value $manifestContent -NoNewline

        Write-Host "[OK] Created stub: $AssetId" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Host "[ERROR] Failed to create stub $AssetId : $_" -ForegroundColor Red
        return $false
    }
}

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "DINOForge Stub Bundle Creator" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$packRoot = Join-Path $RepoRoot "packs" $PackName
$bundleDir = Join-Path $packRoot "assets" "bundles"

if (-not (Test-Path $bundleDir)) {
    Write-Host "[INFO] Creating bundles directory: $bundleDir" -ForegroundColor Cyan
    New-Item -ItemType Directory -Force -Path $bundleDir | Out-Null
}

Write-Host "Pack: $PackName" -ForegroundColor Gray
Write-Host "Bundle Dir: $bundleDir" -ForegroundColor Gray
Write-Host ""

# Get all referenced visual assets
$allAssets = Get-MissingVisualAssets -PackRoot $packRoot
Write-Host "Total referenced assets: $($allAssets.Count)" -ForegroundColor Cyan

# Find missing ones
$missing = @()
foreach ($assetId in $allAssets) {
    $bundlePath = Join-Path $bundleDir $assetId
    if (-not (Test-Path $bundlePath)) {
        $missing += $assetId
    }
}

Write-Host "Missing assets: $($missing.Count)" -ForegroundColor Yellow
$missing | ForEach-Object { Write-Host "  - $_" }

if ($missing.Count -eq 0) {
    Write-Host ""
    Write-Host "[OK] All referenced assets have bundles" -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "Creating stub bundles..." -ForegroundColor Cyan

$created = 0
foreach ($assetId in $missing) {
    $bundlePath = Join-Path $bundleDir $assetId
    if (Create-StubBundle -BundlePath $bundlePath -AssetId $assetId) {
        $created++
    }
}

Write-Host ""
Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Created $created stub bundles" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
