<#
.SYNOPSIS
    Generate the hand-authored map2d marker/legend SVGs under
    clients/bevy-ref/assets/ui/map2d/, then (optionally) rasterise them to PNG.

.DESCRIPTION
    The 2D alternate map view (src/map2d.rs) paints its live markers as crisp
    egui vector shapes, but the *marker language* is documented as real,
    gradient-rich SVG sprites so the look is asset-authorable and the procedural
    overlay stays in sync with a designable legend. This script writes a small,
    clean set of SVGs (agent, house/city, tree, road, water tile, land tile)
    with gradients + soft shadows — genuinely nice, not flat blocks — and then
    calls Tools/rasterize-svg.ps1 to emit high-res PNGs (default 4x).

    Deterministic: same script => same SVGs. Safe to re-run.

.PARAMETER NoRaster
    Only write SVGs; skip the PNG rasterise step.

.PARAMETER Scale
    Upscale factor passed to rasterize-svg.ps1 (default 4 for crisp HiDPI).

.EXAMPLE
    pwsh Tools/gen-map2d-svg.ps1
    pwsh Tools/gen-map2d-svg.ps1 -Scale 6 -NoRaster
#>
[CmdletBinding()]
param(
    [switch]$NoRaster,
    [int]$Scale = 4
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$outDir = Join-Path $repoRoot 'clients/bevy-ref/assets/ui/map2d'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Each entry: filename => SVG body (32x32 viewBox, crisp, gradient-driven).
$svgs = [ordered]@{
    'agent.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <radialGradient id="a" cx="40%" cy="35%" r="75%">
      <stop offset="0%" stop-color="#ffe9b0"/>
      <stop offset="55%" stop-color="#f4a13c"/>
      <stop offset="100%" stop-color="#c2641a"/>
    </radialGradient>
    <filter id="s" x="-30%" y="-30%" width="160%" height="160%">
      <feDropShadow dx="0" dy="1" stdDeviation="1.1" flood-color="#0a0e14" flood-opacity="0.5"/>
    </filter>
  </defs>
  <circle cx="16" cy="16" r="9" fill="url(#a)" stroke="#1b1f28" stroke-width="1.4" filter="url(#s)"/>
  <circle cx="13" cy="13" r="2.4" fill="#fff" fill-opacity="0.55"/>
</svg>
'@
    'house.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <linearGradient id="roof" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#e07a55"/>
      <stop offset="100%" stop-color="#a8482b"/>
    </linearGradient>
    <linearGradient id="wall" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#f0d8b8"/>
      <stop offset="100%" stop-color="#c9a87e"/>
    </linearGradient>
  </defs>
  <rect x="9" y="16" width="14" height="11" rx="1.2" fill="url(#wall)" stroke="#3a2a1d" stroke-width="1"/>
  <path d="M6 17 L16 7 L26 17 Z" fill="url(#roof)" stroke="#3a2a1d" stroke-width="1" stroke-linejoin="round"/>
  <rect x="14" y="20" width="4" height="7" fill="#5a3d28"/>
</svg>
'@
    'tree.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <radialGradient id="c" cx="42%" cy="35%" r="70%">
      <stop offset="0%" stop-color="#5fbf57"/>
      <stop offset="100%" stop-color="#216b2c"/>
    </radialGradient>
  </defs>
  <rect x="14.5" y="19" width="3" height="8" rx="1" fill="#6b4a2b"/>
  <circle cx="16" cy="14" r="8.5" fill="url(#c)" stroke="#14401c" stroke-width="1.1"/>
</svg>
'@
    'road.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <linearGradient id="r" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#6b6f76"/>
      <stop offset="100%" stop-color="#43474d"/>
    </linearGradient>
  </defs>
  <rect x="2" y="13" width="28" height="6" rx="3" fill="url(#r)"/>
  <line x1="5" y1="16" x2="27" y2="16" stroke="#e8d28a" stroke-width="1.4" stroke-dasharray="3 3"/>
</svg>
'@
    'tile-water.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <linearGradient id="w" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#2f74c4"/>
      <stop offset="100%" stop-color="#11335f"/>
    </linearGradient>
  </defs>
  <rect width="32" height="32" fill="url(#w)"/>
  <path d="M3 11 q4 -3 8 0 t8 0 t8 0" fill="none" stroke="#bcd8ff" stroke-opacity="0.35" stroke-width="1.2"/>
  <path d="M3 21 q4 -3 8 0 t8 0 t8 0" fill="none" stroke="#bcd8ff" stroke-opacity="0.25" stroke-width="1.2"/>
</svg>
'@
    'tile-land.svg' = @'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <defs>
    <linearGradient id="l" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#5aa04a"/>
      <stop offset="100%" stop-color="#2f6a2a"/>
    </linearGradient>
  </defs>
  <rect width="32" height="32" fill="url(#l)"/>
  <circle cx="9" cy="11" r="1.4" fill="#2a5520" fill-opacity="0.5"/>
  <circle cx="22" cy="20" r="1.6" fill="#2a5520" fill-opacity="0.5"/>
  <circle cx="16" cy="26" r="1.2" fill="#2a5520" fill-opacity="0.4"/>
</svg>
'@
}

foreach ($name in $svgs.Keys) {
    $path = Join-Path $outDir $name
    Set-Content -Path $path -Value ($svgs[$name].Trim() + "`n") -Encoding utf8 -NoNewline
    Write-Host "  [svg] clients/bevy-ref/assets/ui/map2d/$name"
}
Write-Host "[gen-map2d-svg] wrote $($svgs.Count) SVGs to $outDir"

if (-not $NoRaster) {
    $raster = Join-Path $PSScriptRoot 'rasterize-svg.ps1'
    Write-Host "[gen-map2d-svg] rasterising at ${Scale}x via rasterize-svg.ps1 ..."
    try {
        & pwsh $raster -UiRoot $outDir -Scale $Scale -Force
    } catch {
        Write-Warning "Rasterise skipped (no rsvg/magick backend?): $_"
    }
}
