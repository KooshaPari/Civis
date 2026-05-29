param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$InputDir,

    [Parameter(Mandatory = $true, Position = 1)]
    [string]$OutputDir,

    [Parameter(Position = 2)]
    [string]$Sizes = "256"
)

$ErrorActionPreference = 'Stop'

function Write-Info {
    param([string]$Message)
    Write-Host "[rasterize-svg] $Message"
}

function Write-Warn {
    param([string]$Message)
    Write-Warning "[rasterize-svg] $Message"
}

function Get-ToolCommand {
    param([string[]]$Candidates)

    foreach ($candidate in $Candidates) {
        $cmd = Get-Command $candidate -ErrorAction SilentlyContinue
        if ($null -ne $cmd) {
            return $cmd.Source
        }
    }

    return $null
}

function Parse-Sizes {
    param([string]$SizesText)

    $values = @()
    foreach ($part in ($SizesText -split '[,; ]+')) {
        if ([string]::IsNullOrWhiteSpace($part)) {
            continue
        }

        $value = 0
        if (-not [int]::TryParse($part, [ref]$value) -or $value -le 0) {
            throw "Invalid size value '$part'. Sizes must be positive integers separated by commas, semicolons, or spaces."
        }

        if ($values -notcontains $value) {
            $values += $value
        }
    }

    if ($values.Count -eq 0) {
        throw "No valid sizes provided."
    }

    return $values
}

$resolvedInputDir = (Resolve-Path -LiteralPath $InputDir).Path
if (-not (Test-Path -LiteralPath $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir | Out-Null
}
$resolvedOutputDir = (Resolve-Path -LiteralPath $OutputDir).Path
$sizesList = Parse-Sizes -SizesText $Sizes
$multiSize = $sizesList.Count -gt 1

$inkscape = Get-ToolCommand @('inkscape')
$resvg = Get-ToolCommand @('resvg')
$rsvgConvert = Get-ToolCommand @('rsvg-convert')
$magick = Get-ToolCommand @('magick')

$tool = $null
$toolKind = $null

if ($inkscape) {
    $tool = $inkscape
    $toolKind = 'inkscape'
} elseif ($resvg) {
    $tool = $resvg
    $toolKind = 'resvg'
} elseif ($rsvgConvert) {
    $tool = $rsvgConvert
    $toolKind = 'rsvg-convert'
} elseif ($magick) {
    $tool = $magick
    $toolKind = 'magick'
}

if (-not $tool) {
    Write-Warn "No SVG rasterizer found."
    Write-Warn "Install hints: winget install Inkscape.Inkscape | choco install inkscape | apt install inkscape"
    Write-Warn "Alternative install hints: winget install linebender.resvg | choco install resvg | apt install librsvg2-bin"
    exit 1
}

Write-Info "Using $toolKind at '$tool'"
Write-Info "Input: $resolvedInputDir"
Write-Info "Output: $resolvedOutputDir"
Write-Info "Sizes: $($sizesList -join ', ')"

$svgFiles = Get-ChildItem -LiteralPath $resolvedInputDir -Recurse -File -Filter '*.svg'
if ($svgFiles.Count -eq 0) {
    Write-Warn "No SVG files found under '$resolvedInputDir'."
    exit 0
}

foreach ($svg in $svgFiles) {
    $relativePath = $svg.FullName.Substring($resolvedInputDir.Length).TrimStart('\', '/')
    $relativeParent = Split-Path -Path $relativePath -Parent
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($svg.Name)
    $destinationDir = if ([string]::IsNullOrWhiteSpace($relativeParent)) {
        $resolvedOutputDir
    } else {
        Join-Path $resolvedOutputDir $relativeParent
    }

    if (-not (Test-Path -LiteralPath $destinationDir)) {
        New-Item -ItemType Directory -Path $destinationDir -Force | Out-Null
    }

    foreach ($size in $sizesList) {
        $suffix = if ($multiSize) { "-$size" } else { "" }
        $outputPath = Join-Path $destinationDir ($baseName + $suffix + '.png')

        switch ($toolKind) {
            'inkscape' {
                & $tool $svg.FullName --export-type=png --export-area-page --export-width=$size --export-filename=$outputPath | Out-Null
            }
            'resvg' {
                & $tool $svg.FullName $outputPath -w $size | Out-Null
            }
            'rsvg-convert' {
                & $tool -w $size $svg.FullName -o $outputPath | Out-Null
            }
            'magick' {
                & $tool $svg.FullName -background none -alpha set -resize "${size}x${size}" $outputPath | Out-Null
            }
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Rasterization failed for '$($svg.FullName)' at size $size using $toolKind."
        }

        Write-Info "Wrote $outputPath"
    }
}
