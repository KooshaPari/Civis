<#
.SYNOPSIS
    Export the hand-authored Civis holocron brand SVG to PNG/ICO raster assets.

.DESCRIPTION
    Source of truth: assets/brand/civis-logo.svg (AI-coded vector, not generated).
    Emits icon-{16,24,32,48,64,128,256}.png, icon.png (512), app.ico, civis-banner.png.

.PARAMETER Svg
    Path to civis-logo.svg. Defaults to assets/brand/civis-logo.svg.

.PARAMETER OutDir
    Output directory. Defaults to assets/brand.
#>
[CmdletBinding()]
param(
    [string]$Svg    = (Join-Path $PSScriptRoot '..\assets\brand\civis-logo.svg'),
    [string]$OutDir = (Join-Path $PSScriptRoot '..\assets\brand')
)

$ErrorActionPreference = 'Stop'
$Svg    = (Resolve-Path $Svg).Path
$OutDir = (Resolve-Path $OutDir).Path
$sizes  = @(16, 24, 32, 48, 64, 128, 256, 512)

function Find-Tool([string]$name) {
    $c = Get-Command $name -ErrorAction SilentlyContinue
    if ($c) { return $c.Source }
    $cargoBin = Join-Path $HOME ".cargo\bin\$name.exe"
    if (Test-Path $cargoBin) { return $cargoBin }
    return $null
}

$resvg  = Find-Tool 'resvg'
$rsvg   = Find-Tool 'rsvg-convert'
$magick = Find-Tool 'magick'

function Convert-SvgToPng([string]$src, [string]$dst, [int]$w, [int]$h) {
    if ($resvg)  { & $resvg  -w $w -h $h $src $dst; return }
    if ($rsvg)   { & $rsvg   -w $w -h $h $src -o $dst; return }
    if ($magick) { & $magick -background none -density 768 $src -resize "${w}x${h}" $dst; return }
    throw "No SVG renderer found (resvg / rsvg-convert / magick)."
}

$activeRenderer = @($resvg, $rsvg, $magick) | Where-Object { $_ } | Select-Object -First 1
if (-not $activeRenderer) { throw 'No SVG rasterizer available.' }
Write-Host "Renderer: $activeRenderer"
Write-Host "Source  : $Svg"

$pngBySize = @{}
foreach ($s in $sizes) {
    $dst = Join-Path $OutDir "icon-$s.png"
    Convert-SvgToPng $Svg $dst $s $s
    $pngBySize[$s] = $dst
    Write-Host "  PNG  ${s}x${s} -> $(Split-Path $dst -Leaf)"
}
Copy-Item $pngBySize[512] (Join-Path $OutDir 'icon.png') -Force
Write-Host "  PNG  -> icon.png (512)"

$ico = Join-Path $OutDir 'app.ico'
$icoSizes = @(16, 24, 32, 48, 64, 128, 256)
if (-not $magick) { throw 'ImageMagick (magick) required to build multi-res app.ico.' }
$inputs = $icoSizes | ForEach-Object { $pngBySize[$_] }
& $magick $inputs $ico
if ($LASTEXITCODE -ne 0) { throw "magick ICO build failed" }
Write-Host "  ICO  -> app.ico ($([string]::Join('/', $icoSizes)))"

$bannerSvg = Join-Path $OutDir 'civis-banner.svg'
$bannerPng = Join-Path $OutDir 'civis-banner.png'
if (Test-Path $bannerSvg) {
    Convert-SvgToPng $bannerSvg $bannerPng 640 160
    Write-Host "  PNG  -> civis-banner.png (640x160)"
}

Write-Host "`nDone. Assets in $OutDir"
