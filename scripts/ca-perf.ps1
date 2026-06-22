#Requires -Version 5.1
<#
.SYNOPSIS
  Run the CA dirty-chunk benchmark, report, and flamegraph workflow.

.NOTES
  This is the combined profiling entrypoint for the civ-020 dirty-chunk
  workstream. It runs the benchmark first, emits the markdown report, then
  produces the flamegraph.
#>
[CmdletBinding()]
param(
    [string] $Output = 'target/ca-dirty-chunk.flamegraph.svg',
    [string] $Report = 'target/ca-dirty-chunk.report.md'
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot

Push-Location $RepoRoot
try {
    Write-Host '==> CA dirty-chunk benchmark'
    & "$PSScriptRoot/ca-dirty-chunk-bench.ps1"

    Write-Host '==> CA dirty-chunk report'
    & "$PSScriptRoot/ca-bench-report.ps1"

    Write-Host '==> CA dirty-chunk flamegraph'
    & "$PSScriptRoot/ca-flamegraph.ps1" -Output $Output
}
finally {
    Pop-Location
}
