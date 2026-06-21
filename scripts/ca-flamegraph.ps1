#Requires -Version 5.1
<#
.SYNOPSIS
  Profile the CA dirty-chunk bench with cargo-flamegraph.

.NOTES
  Produces a flamegraph SVG for the CA dirty-chunk benchmark so the profiling
  workflow is reproducible from the repo itself.
#>
[CmdletBinding()]
param(
    [string] $Output = 'target/ca-dirty-chunk.flamegraph.svg'
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required. Install Rust first."
}
if (-not (Get-Command cargo-flamegraph -ErrorAction SilentlyContinue)) {
    throw "cargo-flamegraph is required. Install with: cargo install flamegraph"
}

Push-Location $RepoRoot
try {
    cargo flamegraph --bench ca_dirty_chunk --output $Output
}
finally {
    Pop-Location
}
