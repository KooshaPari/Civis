<#
.SYNOPSIS
    Rasterize the hand-made HUD SVGs (clients/bevy-ref/assets/ui/**.svg) to PNG
    so the egui HUD can load real raster icons.

.DESCRIPTION
    Walks every *.svg under the UI asset root and emits a sibling *.png. Uses
    rsvg-convert when available (crisp, fast), falling back to ImageMagick
    `magick`. Each SVG's natural size is honoured; pass -Scale to upscale for
    HiDPI icons (default 2x for crisp icons at small HUD sizes).

    Deterministic: same SVG in => same PNG out (no timestamps embedded). Safe to
    re-run; only rewrites PNGs whose SVG is newer unless -Force is given.

.PARAMETER UiRoot
    Root directory to scan. Defaults to the bevy-ref UI assets dir.

.PARAMETER Scale
    Integer upscale factor applied to the SVG's intrinsic size. Default 2.

.PARAMETER Force
    Re-rasterize even when the PNG is newer than its SVG.

.EXAMPLE
    pwsh Tools/rasterize-svg.ps1
    pwsh Tools/rasterize-svg.ps1 -Scale 3 -Force
#>
[CmdletBinding()]
param(
    [string]$UiRoot,
    [int]$Scale = 2,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $UiRoot) {
    $UiRoot = Join-Path $repoRoot 'clients/bevy-ref/assets/ui'
}
if (-not (Test-Path $UiRoot)) {
    throw "UI asset root not found: $UiRoot"
}

# --- Resolve a rasterizer backend -------------------------------------------
function Resolve-Tool([string[]]$candidates) {
    foreach ($c in $candidates) {
        $cmd = Get-Command $c -ErrorAction SilentlyContinue
        if ($cmd) { return $cmd.Source }
    }
    # Known Windows install locations as a fallback.
    $known = @(
        'C:\iverilog\gtkwave\bin\rsvg-convert.exe',
        'C:\Program Files\ImageMagick-7.1.0-Q16-HDRI\magick.exe'
    )
    foreach ($k in $known) { if (Test-Path $k) { return $k } }
    return $null
}

$rsvg  = Resolve-Tool @('rsvg-convert')
$magick = Resolve-Tool @('magick', 'convert')

if (-not $rsvg -and -not $magick) {
    throw "No SVG rasterizer found. Install librsvg (rsvg-convert) or ImageMagick (magick)."
}

$backend = if ($rsvg) { "rsvg-convert ($rsvg)" } else { "magick ($magick)" }
Write-Host "[rasterize-svg] backend: $backend  scale=${Scale}x  root=$UiRoot"

# --- Rasterize one SVG -------------------------------------------------------
function Convert-One([string]$svg, [string]$png) {
    if ($rsvg) {
        # -z scales the intrinsic size; rsvg honours width/height/viewBox.
        & $rsvg -z $Scale -o $png $svg
        if ($LASTEXITCODE -ne 0) { throw "rsvg-convert failed on $svg" }
    }
    else {
        # ImageMagick: -density boosts SVG raster resolution; -background none
        # preserves transparency.
        $density = 96 * $Scale
        & $magick -background none -density $density $svg $png
        if ($LASTEXITCODE -ne 0) { throw "magick failed on $svg" }
    }
}

$svgs = Get-ChildItem -Path $UiRoot -Recurse -Filter '*.svg' -File
$done = 0; $skipped = 0; $failed = 0

foreach ($svg in $svgs) {
    $png = [System.IO.Path]::ChangeExtension($svg.FullName, '.png')
    if ((-not $Force) -and (Test-Path $png)) {
        $pngItem = Get-Item $png
        if ($pngItem.LastWriteTimeUtc -ge $svg.LastWriteTimeUtc) {
            $skipped++
            continue
        }
    }
    try {
        Convert-One $svg.FullName $png
        $rel = $png.Substring($repoRoot.Length + 1)
        $size = (Get-Item $png).Length
        Write-Host ("  [ok] {0} ({1} bytes)" -f $rel, $size)
        $done++
    }
    catch {
        Write-Warning "  [fail] $($svg.FullName): $_"
        $failed++
    }
}

Write-Host "[rasterize-svg] rasterized=$done skipped(up-to-date)=$skipped failed=$failed total=$($svgs.Count)"
if ($failed -gt 0) { exit 1 }
