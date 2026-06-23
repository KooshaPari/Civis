# Emit `.ci/quality-manifest.json` after local quality gates (Windows-friendly).
$ErrorActionPreference = "Continue"
Set-Location (git rev-parse --show-toplevel)

$results = @{}
$Failed = $false

function Invoke-Gate {
    param([string]$Name, [scriptblock]$Block)
    & $Block
    if ($LASTEXITCODE -ne 0 -and $null -ne $LASTEXITCODE) {
        $results[$Name] = @{ status = "fail"; detail = "exit $LASTEXITCODE" }
        Write-Host "  fail $Name"
        $script:Failed = $true
        return
    }
    $results[$Name] = @{ status = "pass"; detail = "" }
    Write-Host "  pass $Name"
}

Write-Host "==> civis quality manifest (local gates)"

if (Get-Command just -ErrorAction SilentlyContinue) {
    Invoke-Gate "civis_3d_verify" { just civis-3d-verify }
} else {
    Invoke-Gate "rust_fmt" { cargo fmt --check }
    Invoke-Gate "rust_clippy" { cargo clippy --workspace --all-targets -- -D warnings }
    Invoke-Gate "rust_test" { cargo test --workspace }
    Invoke-Gate "godot_test" { Push-Location clients/godot-ref/rust; cargo test; Pop-Location }
}

Invoke-Gate "web_test" { Push-Location web; npm test; Pop-Location }
Invoke-Gate "dashboard_typecheck" {
    Push-Location web/dashboard
    bun install --frozen-lockfile
    bun run typecheck
    Pop-Location
}

$repoRoot = (git rev-parse --show-toplevel).Trim()
$optionalUnreal = & (Join-Path $PSScriptRoot 'Invoke-OptionalUnrealGates.ps1') -RepoRoot $repoRoot
# Guard against null return when Invoke-OptionalUnrealGates.ps1 exits early
# (no Unreal install detected and CIVIS_QUALITY_UNREAL is unset).
if ($null -eq $optionalUnreal) { $optionalUnreal = @{} }
foreach ($entry in $optionalUnreal.GetEnumerator()) {
    $results[$entry.Key] = $entry.Value
    if ($entry.Value.status -eq 'fail') { $Failed = $true }
    $label = $entry.Value.status
    Write-Host "  $label $($entry.Key)"
}

$gatesJson = @{}
foreach ($entry in $results.GetEnumerator()) {
    $gatesJson[$entry.Key] = $entry.Value
}
$env:QUALITY_GATES_JSON = ($gatesJson | ConvertTo-Json -Compress -Depth 5)
$env:MANIFEST_PATH = ".ci/quality-manifest.json"
New-Item -ItemType Directory -Force -Path .ci | Out-Null
python (Join-Path $PSScriptRoot "write-quality-manifest.py")
$writeExit = $LASTEXITCODE
if ($Failed -or $writeExit -ne 0) { exit 1 }
