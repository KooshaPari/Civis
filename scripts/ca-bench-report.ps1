#Requires -Version 5.1
<#
.SYNOPSIS
  Summarize CA dirty-chunk Criterion output into a markdown report.

.NOTES
  This script consumes the existing Criterion results from the dirty-chunk
  bench and writes a lightweight repo-local report for quick comparison.
#>
[CmdletBinding()]
param(
    [string] $CriterionRoot = 'target/criterion',
    [string] $Output = 'target/ca-dirty-chunk.report.md'
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot

function Format-Nanos {
    param([double] $Value)

    if ($Value -ge 1000000000) { return ('{0:N2} s' -f ($Value / 1000000000)) }
    if ($Value -ge 1000000) { return ('{0:N2} ms' -f ($Value / 1000000)) }
    if ($Value -ge 1000) { return ('{0:N2} us' -f ($Value / 1000)) }
    return ('{0:N0} ns' -f $Value)
}

function Read-Estimates {
    param([string] $Path)

    $json = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    [pscustomobject]@{
        MeanNs = [double] $json.mean.point_estimate
        MedianNs = [double] $json.median.point_estimate
        StdDevNs = [double] $json.mean.standard_error
    }
}

Push-Location $RepoRoot
try {
    if (-not (Test-Path -LiteralPath $CriterionRoot)) {
        throw "Criterion output not found at $CriterionRoot. Run scripts/ca-dirty-chunk-bench.ps1 first."
    }

    $rootPath = (Resolve-Path $CriterionRoot).Path
    $rows = Get-ChildItem -LiteralPath $CriterionRoot -Recurse -Filter estimates.json |
        Where-Object { $_.FullName -match '[\\/](new|base)[\\/]estimates\.json$' } |
        ForEach-Object {
            $kind = if ($_.FullName -match '[\\/]new[\\/]estimates\.json$') { 'new' } else { 'base' }
            $benchmarkPath = Split-Path -Parent (Split-Path -Parent $_.FullName)
            [pscustomobject]@{
                BenchmarkPath = $benchmarkPath.Substring($rootPath.Length + 1)
                Kind = $kind
                Estimates = Read-Estimates -Path $_.FullName
            }
        }

    if (-not $rows) {
        throw "No Criterion estimate files were found under $CriterionRoot."
    }

    $latest = $rows |
        Group-Object BenchmarkPath |
        ForEach-Object {
            $new = $_.Group | Where-Object Kind -eq 'new' | Select-Object -First 1
            $base = $_.Group | Where-Object Kind -eq 'base' | Select-Object -First 1
            $picked = if ($new) { $new } elseif ($base) { $base } else { $_.Group | Select-Object -First 1 }
            [pscustomobject]@{
                Benchmark = $_.Name
                MeanNs = $picked.Estimates.MeanNs
                MedianNs = $picked.Estimates.MedianNs
                StdDevNs = $picked.Estimates.StdDevNs
                Source = $picked.Kind
            }
        } |
        Sort-Object MeanNs

    $lines = @(
        '# CA dirty-chunk benchmark report'
        ''
        ('Generated: {0:yyyy-MM-dd HH:mm:ss zzz}' -f (Get-Date))
        ''
        '| Benchmark | Source | Mean | Median | Std Dev |'
        '|----------|--------|------|--------|---------|'
    )

    foreach ($row in $latest) {
        $lines += ('| {0} | {1} | {2} | {3} | {4} |' -f `
            $row.Benchmark, $row.Source, (Format-Nanos $row.MeanNs), (Format-Nanos $row.MedianNs), (Format-Nanos $row.StdDevNs))
    }

    $outputDir = Split-Path -Parent $Output
    if ($outputDir) {
        New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    }
    Set-Content -LiteralPath $Output -Value $lines -NoNewline:$false
    Write-Host "Wrote $Output"
}
finally {
    Pop-Location
}
