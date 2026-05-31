#requires -Version 5.1
<#
.SYNOPSIS
  Offline TMP SDF font-asset bake for the Star Wars menu (Option A).

.DESCRIPTION
  TMP_FontAsset.CreateFontAsset() returns null at runtime in DINO for OS-dynamic
  fonts. This script bakes the SDF atlas offline in Unity 2021.3.45f1 (where the
  Editor atlas-generator works), wraps it in a version-locked AssetBundle, and
  drops the bundle into the warfare-starwars pack at assets/ui/sw_menu_font.

  Steps:
    1. Copy menu_font.ttf -> unity-assetbundle-builder/Assets/Fonts/menu_font.ttf
    2. Unity batchmode -> BakeTmpFontAsset.BakeHeadless (creates SDF .asset)
    3. Unity batchmode -> BuildAssetBundles.BuildHeadless (emits the bundle)
    4. Copy the 'sw_menu_font' bundle into the pack assets dir.

  Bundle filename == Addressable/bundle key == the value ui_theme.font points at.

.NOTES
  Run from the repo root. Do NOT deploy/launch — reconcile owns the deploy lane.
#>
[CmdletBinding()]
param(
    [string]$Unity = 'C:\Program Files\Unity\Hub\Editor\2021.3.45f1\Editor\Unity.exe',
    [string]$RepoRoot = (Resolve-Path "$PSScriptRoot\..\..").Path
)

$ErrorActionPreference = 'Stop'

$proj      = Join-Path $RepoRoot 'unity-assetbundle-builder'
$srcTtf    = Join-Path $RepoRoot 'packs\warfare-starwars\assets\ui\menu_font.ttf'
$dstTtf    = Join-Path $proj 'Assets\Fonts\menu_font.ttf'
$bundleOut = Join-Path $proj 'AssetBundles\sw_menu_font'
$packUiDir = Join-Path $RepoRoot 'packs\warfare-starwars\assets\ui'
$logBake   = Join-Path $RepoRoot 'docs\sessions\tmp-font-bake.log'
$logBundle = Join-Path $RepoRoot 'docs\sessions\tmp-font-bundle.log'

if (-not (Test-Path $Unity))  { throw "Unity 2021.3.45f1 not found at $Unity" }
if (-not (Test-Path $srcTtf)) { throw "Source font not found: $srcTtf" }

New-Item -ItemType Directory -Force -Path (Split-Path $dstTtf) | Out-Null
Copy-Item $srcTtf $dstTtf -Force
Write-Host "[bake] copied menu_font.ttf into Unity project"

# NOTE: do NOT pass -noUpm — TextMeshPro (com.unity.textmeshpro) is a UPM registry
# package; -noUpm skips resolution so `using TMPro;` fails to compile (#965 bake bug).
Write-Host "[bake] running BakeTmpFontAsset.BakeHeadless ..."
& $Unity -batchmode -nographics -quit `
    -projectPath $proj `
    -executeMethod BakeTmpFontAsset.BakeHeadless `
    -logFile $logBake
if ($LASTEXITCODE -ne 0) { throw "Bake failed (exit $LASTEXITCODE). See $logBake" }

Write-Host "[bake] running BuildAssetBundles.BuildHeadless ..."
& $Unity -batchmode -nographics -quit `
    -projectPath $proj `
    -executeMethod BuildAssetBundles.BuildHeadless `
    -logFile $logBundle
if ($LASTEXITCODE -ne 0) { throw "Bundle build failed (exit $LASTEXITCODE). See $logBundle" }

if (-not (Test-Path $bundleOut)) { throw "Bundle not produced at $bundleOut" }

New-Item -ItemType Directory -Force -Path $packUiDir | Out-Null
Copy-Item $bundleOut (Join-Path $packUiDir 'sw_menu_font') -Force
Write-Host "[bake] DONE -> packs\warfare-starwars\assets\ui\sw_menu_font"
Write-Host "[bake] commit the bundle, then reconcile deploys + live-verifies."
