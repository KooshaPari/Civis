<#
.SYNOPSIS
  EPIC-027 — Rasterize per-pack main-menu takeover SVG art into the PNGs the pack ships.

.DESCRIPTION
  Converts the source SVG menu art (logo, full-bleed background, button frames) under
  packs/<id>/assets/svg/ into raw PNGs under packs/<id>/assets/ui/. Those PNGs are loaded
  at runtime by MainMenuThemer (Texture2D.LoadImage -> Sprite.Create) to perform the
  visual main-menu takeover for total_conversion packs. Raw PNG is the lowest-friction path
  for 2D menu art (no Unity AssetBundle / Addressables build required).

  Rasterizer auto-detected in priority order: inkscape > resvg > rsvg-convert > magick.

.PARAMETER Pack
  Pack id to rasterize: 'warfare-starwars', 'warfare-modern', or 'all' (default).

.EXAMPLE
  pwsh scripts/art/rasterize-menu-takeover.ps1 -Pack warfare-starwars
#>
param(
    [ValidateSet('warfare-starwars', 'warfare-modern', 'all')]
    [string]$Pack = 'all'
)

$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).Path

function Get-Rasterizer {
    foreach ($c in @('inkscape', 'resvg', 'rsvg-convert', 'magick')) {
        $cmd = Get-Command $c -ErrorAction SilentlyContinue
        if ($cmd) { return @{ Kind = $c; Path = $cmd.Source } }
    }
    throw "No SVG rasterizer found. Install one of: inkscape | resvg | rsvg-convert (librsvg) | magick (ImageMagick)."
}

function Invoke-Raster($tool, $svg, $png, $w, $h) {
    switch ($tool.Kind) {
        'inkscape'     { & $tool.Path $svg --export-type=png --export-area-page --export-width=$w --export-height=$h --export-filename=$png | Out-Null }
        'resvg'        { & $tool.Path $svg $png -w $w -h $h | Out-Null }
        'rsvg-convert' { & $tool.Path -w $w -h $h $svg -o $png | Out-Null }
        'magick'       { & $tool.Path -background none $svg -resize "${w}x${h}!" $png | Out-Null }
    }
    if ($LASTEXITCODE -ne 0) { throw "Rasterize failed: $svg" }
    Write-Host ("  OK {0,-16} {1,8} bytes" -f (Split-Path $png -Leaf), (Get-Item $png).Length)
}

# Slot map: output-name -> @(svg-relative-path, width, height)
$packs = @{
    'warfare-starwars' = @{
        'menu_logo.png'  = @('logo-title.svg', 1600, 600)
        'menu_bg.png'    = @('ui/loading-republic.svg', 1920, 1080)
        'btn_normal.png' = @('ui/button-normal.svg', 256, 96)
        'btn_hover.png'  = @('ui/button-hover.svg', 256, 96)
    }
    'warfare-modern'   = @{
        'menu_logo.png'  = @('logo-title.svg', 1600, 500)
        'menu_bg.png'    = @('ui/loading-western.svg', 1920, 1080)
        'btn_normal.png' = @('ui/button-normal.svg', 256, 96)
        'btn_hover.png'  = @('ui/button-hover.svg', 256, 96)
    }
}

$tool = Get-Rasterizer
Write-Host "[rasterize-menu-takeover] Using $($tool.Kind) at $($tool.Path)"

$targets = if ($Pack -eq 'all') { $packs.Keys } else { @($Pack) }
foreach ($p in $targets) {
    $svgDir = Join-Path $repoRoot "packs\$p\assets\svg"
    $uiDir  = Join-Path $repoRoot "packs\$p\assets\ui"
    New-Item -ItemType Directory -Force -Path $uiDir | Out-Null
    Write-Host "[$p] -> $uiDir"
    foreach ($out in $packs[$p].Keys) {
        $spec = $packs[$p][$out]
        $svg  = Join-Path $svgDir $spec[0]
        if (-not (Test-Path -LiteralPath $svg)) { Write-Warning "  skip (missing svg): $svg"; continue }
        Invoke-Raster $tool $svg (Join-Path $uiDir $out) $spec[1] $spec[2]
    }
}
Write-Host "[rasterize-menu-takeover] Done."
