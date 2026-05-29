#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Auto-generates a VitePress-ready stats dashboard for DINOForge.

.DESCRIPTION
    Scans the repository to gather code metrics, pack inventory, CI status,
    and git velocity. Writes output to docs/dashboard.md for VitePress.

.PARAMETER OutputPath
    Path to write the markdown report. Default: docs/dashboard.md

.PARAMETER SkipGitHub
    If set, skip GitHub API calls (useful in offline/testing mode).

.EXAMPLE
    .\scripts\dinoforge-stats.ps1 -OutputPath docs/dashboard.md

.NOTES
    Requires: PowerShell 7+, git, gh CLI (unless -SkipGitHub)
    Generated: UTC timestamp appended to report
#>

param(
    [string]$OutputPath = "docs/dashboard.md",
    [switch]$SkipGitHub
)

$ErrorActionPreference = 'Stop'
$gitRoot = git rev-parse --show-toplevel 2>$null
$RepoRoot = if ($gitRoot) { $gitRoot } else { (Get-Location).Path }

function Get-CodeStats {
    Write-Host "  Scanning code stats..." -ForegroundColor Cyan

    $csFiles = @(Get-ChildItem -Path "$RepoRoot/src" -Filter *.cs -Recurse -ErrorAction SilentlyContinue)
    $testFiles = @(Get-ChildItem -Path "$RepoRoot/src/Tests" -Filter *.cs -Recurse -ErrorAction SilentlyContinue)

    $totalLines = 0
    $csFiles | ForEach-Object {
        try {
            $totalLines += (Get-Content $_.FullName | Measure-Object -Line).Lines
        } catch { }
    }

    # Count [Fact] and [Theory] attributes
    $testCount = 0
    $testFiles | ForEach-Object {
        try {
            $testCount += ((Get-Content $_.FullName -Raw) | Select-String -Pattern '\[(Fact|Theory)\]' -AllMatches).Matches.Count
        } catch { }
    }

    return @{
        TotalFiles = $csFiles.Count
        TestFiles = $testFiles.Count
        TotalLines = $totalLines
        TestCases = $testCount
    }
}

function Get-PackStats {
    Write-Host "  Scanning pack inventory..." -ForegroundColor Cyan

    $packsDir = "$RepoRoot/packs"
    if (-not (Test-Path $packsDir)) {
        return @{ Packs = @(); TotalBundles = 0; TotalBundleSize = 0 }
    }

    $packs = @()
    $totalBundles = 0
    $totalBundleSize = 0

    Get-ChildItem -Path $packsDir -Directory | Where-Object { $_.Name -notmatch '^_' } | ForEach-Object {
        $packDir = $_.FullName
        $packName = $_.Name

        # Try to read pack.yaml
        $packYaml = Join-Path $packDir "pack.yaml"
        $version = "unknown"
        $type = "unknown"

        if (Test-Path $packYaml) {
            $yamlContent = Get-Content $packYaml -Raw
            if ($yamlContent -match 'version:\s*([^\s]+)') {
                $version = $matches[1]
            }
            if ($yamlContent -match 'type:\s*(\w+)') {
                $type = $matches[1]
            }
        }

        # Count definitions (units, buildings, weapons, etc.)
        $unitCount = 0
        $buildingCount = 0

        $definitionsDir = Join-Path $packDir "definitions"
        if (Test-Path $definitionsDir) {
            $unitCount = @(Get-ChildItem -Path "$definitionsDir/units" -Filter *.yaml -ErrorAction SilentlyContinue).Count
            $buildingCount = @(Get-ChildItem -Path "$definitionsDir/buildings" -Filter *.yaml -ErrorAction SilentlyContinue).Count
        }

        # Count asset bundles
        $bundlesDir = Join-Path $packDir "assets/bundles"
        $bundles = @(Get-ChildItem -Path $bundlesDir -File -ErrorAction SilentlyContinue)
        $bundleSize = 0
        $bundles | ForEach-Object {
            $totalBundles++
            $bundleSize += $_.Length
            $totalBundleSize += $_.Length
        }

        # Pack dir size (MB)
        $packDirSize = 0
        Get-ChildItem -Path $packDir -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object {
            $packDirSize += $_.Length
        }
        $packDirSizeMB = [Math]::Round($packDirSize / 1MB, 2)

        $packs += @{
            Name = $packName
            Version = $version
            Type = $type
            Units = $unitCount
            Buildings = $buildingCount
            Bundles = $bundles.Count
            BundleSizeMB = [Math]::Round($bundleSize / 1MB, 2)
            TotalSizeMB = $packDirSizeMB
        }
    }

    return @{
        Packs = $packs
        TotalBundles = $totalBundles
        TotalBundleSize = [Math]::Round($totalBundleSize / 1MB, 2)
    }
}

function Get-GitVelocity {
    Write-Host "  Analyzing git velocity..." -ForegroundColor Cyan

    $sevenDaysAgo = (Get-Date).AddDays(-7).ToString("yyyy-MM-dd")
    $commitsLast7Days = @(git log --since="$sevenDaysAgo" --oneline 2>$null)

    # Top contributors (last 30 days)
    $contributors = @()
    git log --since="30 days ago" --format="%an" 2>$null | Group-Object | Sort-Object Count -Descending | Select-Object -First 5 | ForEach-Object {
        $contributors += @{ Name = $_.Name; Commits = $_.Count }
    }

    return @{
        CommitsLast7Days = $commitsLast7Days.Count
        TopContributors = $contributors
    }
}

function Get-ReleaseStats {
    Write-Host "  Checking releases..." -ForegroundColor Cyan

    $latestTag = git describe --tags --abbrev=0 2>$null
    $latestTagDate = $null
    $daysSinceRelease = "unknown"

    if (-not $latestTag) {
        $latestTag = "no tags"
    } else {
        $tagDate = git log -1 --format=%aI $latestTag 2>$null
        if ($tagDate) {
            $latestTagDate = [DateTime]::Parse($tagDate)
            $daysSinceRelease = [Math]::Floor(((Get-Date) - $latestTagDate).TotalDays)
        }
    }

    return @{
        LatestTag = $latestTag
        DaysSinceRelease = $daysSinceRelease
    }
}

function Get-PatternCatalogCount {
    Write-Host "  Counting Pattern Catalog entries..." -ForegroundColor Cyan

    $claudemd = "$RepoRoot/CLAUDE.md"
    if (-not (Test-Path $claudemd)) {
        return 0
    }

    $content = Get-Content $claudemd -Raw
    $matches = [regex]::Matches($content, '### Pattern #\d+:')
    return $matches.Count
}

function Get-GitHubStats {
    param([bool]$Skip)

    if ($Skip) {
        Write-Host "  Skipping GitHub API calls..." -ForegroundColor Yellow
        return @{
            LatestRuns = @()
            OpenIssues = "N/A"
        }
    }

    Write-Host "  Fetching GitHub Actions & issues..." -ForegroundColor Cyan

    # Latest 5 runs
    $runs = @()
    try {
        $runData = gh run list --limit 5 --json name,status,conclusion,createdAt 2>$null
        if ($runData) {
            $runData | ConvertFrom-Json | ForEach-Object {
                $runs += @{
                    Name = $_.name
                    Status = $_.status
                    Conclusion = $_.conclusion
                    CreatedAt = $_.createdAt
                }
            }
        }
    } catch {
        Write-Warning "Failed to fetch GitHub runs: $_"
    }

    # Open issues
    $issueCount = 0
    try {
        $issueCount = (gh issue list --limit 0 2>$null | Measure-Object -Line).Lines
    } catch {
        Write-Warning "Failed to fetch GitHub issues: $_"
    }

    return @{
        LatestRuns = $runs
        OpenIssues = $issueCount
    }
}

function Build-Dashboard {
    param(
        [hashtable]$CodeStats,
        [hashtable]$PackStats,
        [hashtable]$GitVelocity,
        [hashtable]$ReleaseStats,
        [int]$PatternCatalogCount,
        [hashtable]$GitHubStats
    )

    $timestamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss UTC")

    $sb = New-Object System.Text.StringBuilder
    $sb.AppendLine("# DINOForge Stats Dashboard") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("> Auto-generated dashboard. Last updated: **$timestamp**") | Out-Null
    $sb.AppendLine() | Out-Null

    $sb.AppendLine("## Code Metrics") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Metric | Value |") | Out-Null
    $sb.AppendLine("|--------|-------|") | Out-Null
    $sb.AppendLine("| C# Files | $($CodeStats.TotalFiles) |") | Out-Null
    $sb.AppendLine("| Test Files | $($CodeStats.TestFiles) |") | Out-Null
    $sb.AppendLine("| Lines of C# Code | $($CodeStats.TotalLines) |") | Out-Null
    $sb.AppendLine("| Test Cases | $($CodeStats.TestCases) |") | Out-Null
    $sb.AppendLine() | Out-Null

    $sb.AppendLine("## Pack Inventory") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("**Total Packs**: $($PackStats.Packs.Count) | **Total Bundles**: $($PackStats.TotalBundles) | **Bundle Size**: $($PackStats.TotalBundleSize) MB") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Pack | Version | Type | Units | Buildings | Bundles | Size (MB) |") | Out-Null
    $sb.AppendLine("|------|---------|------|-------|-----------|---------|-----------|") | Out-Null

    $PackStats.Packs | ForEach-Object {
        $sb.AppendLine("| $($_.Name) | $($_.Version) | $($_.Type) | $($_.Units) | $($_.Buildings) | $($_.Bundles) | $($_.TotalSizeMB) |") | Out-Null
    }

    $sb.AppendLine() | Out-Null
    $sb.AppendLine("## Git Velocity") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Metric | Value |") | Out-Null
    $sb.AppendLine("|--------|-------|") | Out-Null
    $sb.AppendLine("| Commits (Last 7 Days) | $($GitVelocity.CommitsLast7Days) |") | Out-Null
    $sb.AppendLine() | Out-Null

    $sb.AppendLine("### Top Contributors (Last 30 Days)") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Contributor | Commits |") | Out-Null
    $sb.AppendLine("|-------------|---------|") | Out-Null

    $GitVelocity.TopContributors | ForEach-Object {
        $sb.AppendLine("| $($_.Name) | $($_.Commits) |") | Out-Null
    }

    $sb.AppendLine() | Out-Null
    $sb.AppendLine("## Releases") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Metric | Value |") | Out-Null
    $sb.AppendLine("|--------|-------|") | Out-Null
    $sb.AppendLine("| Latest Tag | $($ReleaseStats.LatestTag) |") | Out-Null
    $sb.AppendLine("| Days Since Release | $($ReleaseStats.DaysSinceRelease) |") | Out-Null
    $sb.AppendLine() | Out-Null

    $sb.AppendLine("## Quality & Governance") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("| Metric | Value |") | Out-Null
    $sb.AppendLine("|--------|-------|") | Out-Null
    $sb.AppendLine("| Pattern Catalog Entries | $PatternCatalogCount |") | Out-Null
    $sb.AppendLine("| Open Issues | $($GitHubStats.OpenIssues) |") | Out-Null
    $sb.AppendLine() | Out-Null

    if ($GitHubStats.LatestRuns.Count -gt 0) {
        $sb.AppendLine("### Latest GitHub Actions Runs") | Out-Null
        $sb.AppendLine() | Out-Null
        $sb.AppendLine("| Workflow | Status | Conclusion | Created |") | Out-Null
        $sb.AppendLine("|----------|--------|-----------|---------|") | Out-Null
        $GitHubStats.LatestRuns | ForEach-Object {
            $created = [DateTime]::Parse($_.CreatedAt).ToString("yyyy-MM-dd HH:mm")
            $sb.AppendLine("| $($_.Name) | $($_.Status) | $($_.Conclusion) | $created |") | Out-Null
        }
        $sb.AppendLine() | Out-Null
    }

    $sb.AppendLine("---") | Out-Null
    $sb.AppendLine() | Out-Null
    $sb.AppendLine("**Generated by**: DINOForge Stats Dashboard (`scripts/dinoforge-stats.ps1`)") | Out-Null

    return $sb.ToString()
}

# Main
Write-Host "DINOForge Stats Dashboard Generator" -ForegroundColor Green
Write-Host "====================================" -ForegroundColor Green
Write-Host ""

try {
    $codeStats = Get-CodeStats
    $packStats = Get-PackStats
    $gitVelocity = Get-GitVelocity
    $releaseStats = Get-ReleaseStats
    $patternCount = Get-PatternCatalogCount
    $githubStats = Get-GitHubStats -Skip $SkipGitHub

    $dashboard = Build-Dashboard -CodeStats $codeStats -PackStats $packStats `
                                  -GitVelocity $gitVelocity -ReleaseStats $releaseStats `
                                  -PatternCatalogCount $patternCount -GitHubStats $githubStats

    # Write to file
    $outputDir = Split-Path -Parent $OutputPath
    if (-not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
    }

    Set-Content -Path $OutputPath -Value $dashboard -Encoding UTF8

    Write-Host "✓ Dashboard written to: $OutputPath" -ForegroundColor Green
    Write-Host ""
    Write-Host "Stats Summary:" -ForegroundColor Green
    Write-Host "  C# Files:     $($codeStats.TotalFiles)"
    Write-Host "  Test Cases:   $($codeStats.TestCases)"
    Write-Host "  Packs:        $($packStats.Packs.Count)"
    Write-Host "  Commits (7d): $($gitVelocity.CommitsLast7Days)"
    Write-Host "  Patterns:     $patternCount"

} catch {
    Write-Error "Failed to generate dashboard: $_"
    exit 1
}
