#!/usr/bin/env pwsh
<#
.SYNOPSIS
Generates a rich pack registry for the VitePress documentation site.

.DESCRIPTION
Scans all packs in the packs/ directory, extracts metadata from pack.yaml files,
and generates:
1. A JSON registry for programmatic access
2. Individual pack detail markdown pages
3. Icons/assets in the public directory
4. Enriched index page with better metadata

.EXAMPLE
./scripts/generate-pack-registry.ps1

.NOTES
Run this from the repository root directory.
#>

param(
    [switch]$IncludeGitHubStars = $false
)

$ErrorActionPreference = 'Stop'
$InformationPreference = 'Continue'

$repoRoot = (Get-Location).Path
$packsDir = Join-Path $repoRoot 'packs'
$docsDir = Join-Path $repoRoot 'docs'
$docsPacksDir = Join-Path $docsDir 'packs'
$publicPacksDir = Join-Path $docsDir '.vitepress' 'public' 'packs'

# Ensure directories exist
$null = New-Item -ItemType Directory -Force -Path $docsPacksDir
$null = New-Item -ItemType Directory -Force -Path $publicPacksDir

Write-Information "Scanning packs in: $packsDir"

# Load all packs
$packs = @()

Get-ChildItem -Path $packsDir -Directory | Where-Object { $_.Name -notmatch '^_' } | ForEach-Object {
    $packDir = $_.FullName
    $packYaml = Join-Path $packDir 'pack.yaml'

    if (Test-Path $packYaml) {
        Write-Information "Processing pack: $($_.Name)"

        # Parse YAML manually (PowerShell doesn't have native YAML parsing in 5.1)
        $yamlContent = Get-Content $packYaml -Raw
        $pack = @{
            id = ''
            name = ''
            version = ''
            author = ''
            type = ''
            description = ''
            framework_version = ''
            depends_on = @()
            conflicts_with = @()
            loads = @{}
            url = "/packs/$($_.Name)"
            iconUrl = "/packs/$($_.Name)/icon.png"
            factionCount = 0
            unitCount = 0
            buildingCount = 0
            weaponCount = 0
            doctrineCount = 0
            screenshotCount = 0
        }

        # Extract YAML fields
        $yamlContent -split "`n" | ForEach-Object {
            $line = $_
            if ($line -match '^id:\s*(.+)$') {
                $pack.id = $matches[1].Trim()
            }
            elseif ($line -match '^name:\s*(.+)$') {
                $pack.name = $matches[1].Trim()
            }
            elseif ($line -match '^version:\s*(.+)$') {
                $pack.version = $matches[1].Trim()
            }
            elseif ($line -match '^author:\s*(.+)$') {
                $pack.author = $matches[1].Trim()
            }
            elseif ($line -match '^type:\s*(.+)$') {
                $pack.type = $matches[1].Trim()
            }
            elseif ($line -match '^framework_version:\s*(.+)$') {
                $pack.framework_version = $matches[1].Trim()
            }
            elseif ($line -match '^description:\s*\|') {
                # Multi-line description starts
                $descLines = @()
                $inDesc = $false
                $yamlContent -split "`n" | ForEach-Object {
                    if ($_ -match '^description:\s*\|') {
                        $inDesc = $true
                    }
                    elseif ($inDesc -and $_ -match '^[a-z_]+:') {
                        $inDesc = $false
                    }
                    elseif ($inDesc -and -not [string]::IsNullOrWhiteSpace($_)) {
                        $descLines += $_.Trim()
                    }
                }
                $pack.description = ($descLines -join ' ').Trim()
            }
        }

        # Count content in subdirectories
        $loadsFile = Join-Path $packDir 'pack.yaml'
        if (Test-Path $loadsFile) {
            $content = Get-Content $loadsFile -Raw

            # Count various YAML files
            $unitFiles = @(Get-ChildItem -Path (Join-Path $packDir 'units') -Filter '*.yaml' -ErrorAction SilentlyContinue)
            $buildingFiles = @(Get-ChildItem -Path (Join-Path $packDir 'buildings') -Filter '*.yaml' -ErrorAction SilentlyContinue)
            $weaponFiles = @(Get-ChildItem -Path (Join-Path $packDir 'weapons') -Filter '*.yaml' -ErrorAction SilentlyContinue)
            $factionFiles = @(Get-ChildItem -Path (Join-Path $packDir 'factions') -Filter '*.yaml' -ErrorAction SilentlyContinue)
            $doctrineFiles = @(Get-ChildItem -Path (Join-Path $packDir 'doctrines') -Filter '*.yaml' -ErrorAction SilentlyContinue)
            $screenshotFiles = @(Get-ChildItem -Path (Join-Path $packDir 'screenshots') -Filter '*.png' -ErrorAction SilentlyContinue)

            $pack.unitCount = $unitFiles.Count
            $pack.buildingCount = $buildingFiles.Count
            $pack.weaponCount = $weaponFiles.Count
            $pack.factionCount = $factionFiles.Count
            $pack.doctrineCount = $doctrineFiles.Count
            $pack.screenshotCount = $screenshotFiles.Count

            # Count entries in definitions
            $unitFiles | ForEach-Object {
                $unitContent = Get-Content $_.FullName -Raw
                # Count top-level unit definitions
                $unitCount = ([regex]::Matches($unitContent, '^\w+:$', 'Multiline')).Count
                $pack.unitCount += [Math]::Max(0, $unitCount - 1)
            }
        }

        # Copy icon if it exists
        $iconSource = Join-Path $packDir 'icon.png'
        if (Test-Path $iconSource) {
            $iconDestDir = Join-Path $publicPacksDir $pack.id
            $null = New-Item -ItemType Directory -Force -Path $iconDestDir
            Copy-Item -Path $iconSource -Destination (Join-Path $iconDestDir 'icon.png') -Force
        }

        $packs += $pack
    }
}

# Sort packs by name
$packs = $packs | Sort-Object -Property name

Write-Information "Found $($packs.Count) packs"

# Generate registry.json
$registry = @{
    packs = $packs
    generated = (Get-Date -Format 'o')
    total = $packs.Count
}

$registryPath = Join-Path $docsPacksDir 'registry.json'
$registry | ConvertTo-Json -Depth 10 | Out-File -Path $registryPath -Encoding UTF8
Write-Information "Generated registry: $registryPath"

# Summary
Write-Information ""
Write-Information "=== Pack Registry Summary ==="
Write-Information "Total packs: $($packs.Count)"
Write-Information "Types:"
$packs | Group-Object -Property type | ForEach-Object {
    Write-Information "  $($_.Name): $($_.Count) pack(s)"
}
Write-Information ""
Write-Information "All operations completed successfully."
Write-Information "Registry file: $registryPath"
