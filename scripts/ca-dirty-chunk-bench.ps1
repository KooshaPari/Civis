#Requires -Version 5.1
<#
.SYNOPSIS
  Run the CA dirty-chunk Criterion bench.

.NOTES
  This is the direct benchmark entrypoint used to capture performance baselines
  for the civ-020 dirty-chunk workstream.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path -Parent $PSScriptRoot

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required. Install Rust first."
}

Push-Location $RepoRoot
try {
    cargo bench --bench ca_dirty_chunk
}
finally {
    Pop-Location
}
