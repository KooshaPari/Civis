#Requires -Version 5.1
<#
.SYNOPSIS
    Generates 24x24 placeholder PNG badge files for the DINOForge badge system.

.DESCRIPTION
    Creates solid-colour 24x24 PNG files with centred initial letter(s) for each
    badge defined in the DINOForge badge catalogue.  Output goes to assets/badges/
    at the repo root so the DeployBadgeAssets MSBuild target can copy them into
    BepInEx/plugins/dinoforge-ui-assets/badges/ during a deploy build.

    This script uses System.Drawing (GDI+) which is available on Windows without
    any additional dependencies.

.PARAMETER OutputDir
    Override the default output directory (repo_root/assets/badges/).

.EXAMPLE
    .\scripts\generate-badge-placeholders.ps1
#>
param(
    [string]$OutputDir = (Join-Path $PSScriptRoot "..\assets\badges")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Add-Type -AssemblyName System.Drawing

$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force $OutputDir | Out-Null
}

# Badge definitions: name -> (background colour, text label)
$badges = [ordered]@{
    'verified-author'      = @{ Bg = [System.Drawing.Color]::FromArgb(255, 46,  204, 113); Label = 'V'  }
    'popular'              = @{ Bg = [System.Drawing.Color]::FromArgb(255, 255, 153,   0); Label = 'P'  }
    'editors-choice'       = @{ Bg = [System.Drawing.Color]::FromArgb(255, 255, 215,   0); Label = 'E'  }
    'early-access'         = @{ Bg = [System.Drawing.Color]::FromArgb(255,  51, 153, 255); Label = 'EA' }
    'total-conversion'     = @{ Bg = [System.Drawing.Color]::FromArgb(255, 153,  51, 204); Label = 'TC' }
    'compatibility-tested' = @{ Bg = [System.Drawing.Color]::FromArgb(255,  46, 204, 113); Label = 'C'  }
    'default'              = @{ Bg = [System.Drawing.Color]::FromArgb(255, 128, 128, 128); Label = '?'  }
}

$size = 24

foreach ($name in $badges.Keys) {
    $def  = $badges[$name]
    $bg   = $def.Bg
    $label = $def.Label

    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g   = [System.Drawing.Graphics]::FromImage($bmp)

    # Smooth rendering
    $g.SmoothingMode   = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAlias

    # Fill circle background
    $brush = New-Object System.Drawing.SolidBrush($bg)
    $g.FillEllipse($brush, 0, 0, $size - 1, $size - 1)
    $brush.Dispose()

    # Draw initial letter(s), centred
    $fontSize = if ($label.Length -gt 1) { [float]8 } else { [float]11 }
    $font   = New-Object System.Drawing.Font -ArgumentList 'Arial', $fontSize, ([System.Drawing.FontStyle]::Bold)
    $white  = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
    $sf     = New-Object System.Drawing.StringFormat
    $sf.Alignment     = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect   = New-Object System.Drawing.RectangleF -ArgumentList ([float]0), ([float]0), ([float]$size), ([float]$size)
    $g.DrawString($label, $font, $white, $rect, $sf)
    $font.Dispose()
    $white.Dispose()
    $sf.Dispose()

    $g.Dispose()

    $outPath = Join-Path $OutputDir "$name.png"
    $bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()

    Write-Host "  Generated: $outPath"
}

Write-Host ""
Write-Host "Badge placeholders written to: $OutputDir"
Write-Host "Total: $($badges.Count) files"
