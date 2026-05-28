#!/usr/bin/env pwsh
<#
.SYNOPSIS
Generate a static web index of available DINOForge packs for kooshapari.github.io/Dino/packs

.DESCRIPTION
Scans the packs/ directory, reads pack.yaml metadata, and generates:
- docs/packs/index.md (VitePress grid page)
- docs/packs/<pack-id>.md (detailed per-pack pages)
- docs/packs/registry.json (machine-readable index)

.EXAMPLE
./scripts/generate-pack-index.ps1
#>

param(
    [switch]$Verbose
)

$ErrorActionPreference = 'Stop'
$WarningPreference = 'Continue'

$repoRoot = Resolve-Path (Split-Path -Parent $PSScriptRoot)
$packsDir = Join-Path -Path $repoRoot -ChildPath 'packs'
$docsDir = Join-Path -Path $repoRoot -ChildPath 'docs'
$docsPacksDir = Join-Path -Path $docsDir -ChildPath 'packs'
$publicPacksDir = Join-Path -Path $docsDir -ChildPath '.vitepress' | Join-Path -ChildPath 'public' | Join-Path -ChildPath 'packs'

Write-Host "Generating pack index..." -ForegroundColor Cyan
Write-Host "  Repo root: $repoRoot"
Write-Host "  Packs dir: $packsDir"
Write-Host "  Docs dir: $docsPacksDir"

# Ensure output directories exist
New-Item -ItemType Directory -Path $docsPacksDir -Force | Out-Null
New-Item -ItemType Directory -Path $publicPacksDir -Force | Out-Null

# Load pack.yaml files
$packs = @()
$packsById = @{}

Get-ChildItem -Path $packsDir -Directory | Where-Object { $_.Name -notlike '_*' } | ForEach-Object {
    $packDir = $_.FullName
    $packYaml = Join-Path -Path $packDir -ChildPath 'pack.yaml'

    if (-not (Test-Path $packYaml)) {
        Write-Warning "No pack.yaml found in $($_.Name)"
        return
    }

    # Parse YAML (basic parsing for essential fields)
    $yamlLines = Get-Content $packYaml
    $yaml = $yamlLines -join "`n"

    # Extract key fields using regex (multiline flag for cross-line matching)
    $pack = @{
        id = ''
        name = ''
        version = '0.0.0'
        type = 'content'
        author = 'DINOForge'
        description = ''
        framework_version = '>=0.1.0'
        packDir = $packDir
        packPath = $_.FullName
    }

    # Parse YAML fields - must match at column 0 (no leading whitespace) to avoid nested config keys
    for ($i = 0; $i -lt $yamlLines.Count; $i++) {
        $line = $yamlLines[$i]

        if ($line -match '^id:\s+(.+)$') {
            $pack.id = $matches[1].Trim() -replace '"'
        }
        elseif ($line -match '^name:\s+(.+)$') {
            $pack.name = $matches[1].Trim() -replace '"'
        }
        elseif ($line -match '^version:\s+(.+)$') {
            $pack.version = $matches[1].Trim() -replace '"'
        }
        elseif ($line -match '^author:\s+(.+)$') {
            $pack.author = $matches[1].Trim() -replace '"'
        }
        elseif ($line -match '^framework_version:\s+"(.+?)"') {
            $pack.framework_version = $matches[1]
        }
        elseif ($line -match '^type:\s+(.+)$') {
            $pack.type = $matches[1].Trim() -replace '"'
        }
        elseif ($line -match '^\s*description:\s*\|\s*$') {
            # Found description block, extract until next key
            $descLines = @()
            for ($j = $i + 1; $j -lt $yamlLines.Count; $j++) {
                $descLine = $yamlLines[$j]
                # Stop at next key (line starting with word-char and contains colon, at column 0)
                if ($descLine -match '^[a-zA-Z_][a-zA-Z0-9_]*:' -and -not ($descLine -match '^\s+')) {
                    break
                }
                # Add description lines (strip leading indent)
                if ($descLine.Trim()) {
                    $descLines += $descLine.Trim()
                }
            }
            if ($descLines.Count -gt 0) {
                $pack.description = ($descLines -join ' ').Trim()
                # Limit description to first 150 characters for index
                if ($pack.description.Length -gt 150) {
                    $pack.description = $pack.description.Substring(0, 150) + '...'
                }
            }
        }
    }

    # Fallback to directory name if id not found
    if (-not $pack.id) { $pack.id = $_.Name }
    if (-not $pack.name) { $pack.name = $_.Name }

    # Count entities (units, buildings, etc.)
    $pack.unitCount = (Get-ChildItem -Path (Join-Path -Path $packDir -ChildPath 'units') -Filter '*.yaml' -ErrorAction SilentlyContinue | Measure-Object).Count
    $pack.buildingCount = (Get-ChildItem -Path (Join-Path -Path $packDir -ChildPath 'buildings') -Filter '*.yaml' -ErrorAction SilentlyContinue | Measure-Object).Count
    $pack.factionCount = (Get-ChildItem -Path (Join-Path -Path $packDir -ChildPath 'factions') -Filter '*.yaml' -ErrorAction SilentlyContinue | Measure-Object).Count

    # Check for screenshots
    $screenshotDir = Join-Path -Path $packDir -ChildPath 'screenshots'
    if (Test-Path $screenshotDir) {
        $pack.screenshots = @(Get-ChildItem -Path $screenshotDir -Include '*.png', '*.jpg', '*.webp' -ErrorAction SilentlyContinue)
    } else {
        $pack.screenshots = @()
    }

    $packs += $pack
    $packsById[$pack.id] = $pack
}

# Sort packs: type > name
$packs = $packs | Sort-Object -Property @{ Expression = { $_.type }; Ascending = $true }, @{ Expression = { $_.name }; Ascending = $true }

Write-Host "Found $($packs.Count) packs" -ForegroundColor Green

# Generate registry.json
$registry = @{
    version = '1.0'
    generated = (Get-Date -Format 'O').ToString()
    packs = $packs | ForEach-Object {
        @{
            id = $_.id
            name = $_.name
            version = $_.version
            type = $_.type
            author = $_.author
            framework_version = $_.framework_version
            unitCount = $_.unitCount
            buildingCount = $_.buildingCount
            factionCount = $_.factionCount
            screenshotCount = $_.screenshots.Count
            url = "/Dino/packs/$($_.id)"
        }
    }
}

$registryPath = Join-Path -Path $docsPacksDir -ChildPath 'registry.json'
$registry | ConvertTo-Json -Depth 10 | Set-Content $registryPath -Encoding UTF8
Write-Host "  registry.json written to $registryPath" -ForegroundColor Green

# Generate per-pack detail pages
$packs | ForEach-Object {
    $pack = $_

    # Build gallery section if screenshots exist
    $gallerySection = ""
    if ($pack.screenshots -and $pack.screenshots.Count -gt 0) {
        $gallerySection += "`n## Gallery`n`n"
        $gallerySection += "<div style='display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem;'>`n`n"

        foreach ($screenshot in $pack.screenshots | Sort-Object -Property Name) {
            $relPath = "../../packs/$($pack.id)/screenshots/$($screenshot.Name)"
            $gallerySection += "<div style='border: 1px solid #ddd; border-radius: 4px; overflow: hidden;'>`n"
            $gallerySection += "  <img src='$relPath' alt='Gameplay screenshot' style='width: 100%; aspect-ratio: 16/9; object-fit: cover;' />`n"
            $gallerySection += "</div>`n`n"
        }

        $gallerySection += "</div>`n"
    }

    $detailMd = @"
---
title: "$($pack.name)"
layout: doc
---

# $($pack.name)

**Version:** $($pack.version) | **Type:** $($pack.type) | **Author:** $($pack.author)

**Framework:** $($pack.framework_version)

## Overview

$($pack.description)

## Content Summary

| Category | Count |
|----------|-------|
| Units | $($pack.unitCount) |
| Buildings | $($pack.buildingCount) |
| Factions | $($pack.factionCount) |
$gallerySection
## Installation

Install via the DINOForge installer or manual:

\`\`\`bash
dinoforge pack install $($pack.id)
\`\`\`

Or via the in-game mod manager (F10), search for "$($pack.name)".

## Configuration

This pack may provide in-game settings accessible via the F10 mod panel.

## Dependencies

This pack is self-contained with no required dependencies.

## Compatibility

- Framework version: $($pack.framework_version)
- Minimum DINOForge: 0.5.0
- Game: Diplomacy is Not an Option (Unity 2021.3+)

## Support

For issues, bug reports, or feature requests, visit the [DINOForge GitHub repository](https://github.com/KooshaPari/Dino).

"@

    $pagePath = Join-Path -Path $docsPacksDir -ChildPath "$($pack.id).md"
    $detailMd | Set-Content $pagePath -Encoding UTF8
    Write-Host "  Generated $($pack.id).md" -ForegroundColor Green
}

# Generate index.md with grid
$indexMd = @"
---
title: Pack Registry
layout: doc
---

# DINOForge Pack Registry

Browse all available content packs for DINOForge. Packs extend gameplay with new units, buildings, factions, economies, scenarios, and more.

## Available Packs

"@

# Group packs by type
$packsByType = $packs | Group-Object -Property type | Sort-Object -Property Name

foreach ($typeGroup in $packsByType) {
    $typeName = $typeGroup.Name
    $typeDisplay = switch ($typeName) {
        'total_conversion' { 'Total Conversions' }
        'content' { 'Content Packs' }
        'balance' { 'Balance Packs' }
        'scenario' { 'Scenario Packs' }
        'utility' { 'Utility Packs' }
        'ruleset' { 'Ruleset Packs' }
        default { $typeName }
    }

    $indexMd += "`n### $typeDisplay`n`n"

    foreach ($pack in $typeGroup.Group) {
        $desc = $pack.description -split "`n" | Select-Object -First 1
        $contentSummary = ""
        if ($pack.unitCount -gt 0) { $contentSummary += "$($pack.unitCount) units " }
        if ($pack.buildingCount -gt 0) { $contentSummary += "$($pack.buildingCount) buildings " }
        if ($pack.factionCount -gt 0) { $contentSummary += "$($pack.factionCount) factions" }
        $contentSummary = $contentSummary -replace '\s+$', ''

        $indexMd += "#### [$($pack.name)]($($pack.id).md)`n`n"
        $indexMd += "**Version:** $($pack.version) | **By:** $($pack.author)`n`n"
        $indexMd += "$($desc)`n`n"
        $indexMd += "Content: $($contentSummary)`n`n"
    }
}

$indexMd += @"

## Machine-Readable Registry

A JSON registry is available at [`/packs/registry.json`](/packs/registry.json) for programmatic access (e.g., package managers, CLI tools, launchers).

## Creating Your Own Pack

Learn how to create a custom pack in the [Pack Author Guide](/guides/your-first-mod).

## Submit Your Pack

Community-created packs are welcome! Open a GitHub issue or PR to add your pack to the registry.

"@

$indexPath = Join-Path -Path $docsPacksDir -ChildPath 'index.md'
$indexMd | Set-Content $indexPath -Encoding UTF8
Write-Host "  Generated index.md" -ForegroundColor Green

Write-Host "Pack index generation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Generated files:" -ForegroundColor Cyan
Write-Host "  - docs/packs/index.md"
Write-Host "  - docs/packs/{pack-id}.md (for each pack)"
Write-Host "  - docs/packs/registry.json"
Write-Host ""
Write-Host "To view locally: npm run docs:dev"
Write-Host "To deploy: push to main branch (GitHub Pages auto-deploys)"
